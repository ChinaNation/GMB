// LegislationApi 客户端(ADR-028 P3)——读链上法律 + 宪法不可修改条款 manifest。
//
// 中文注释:`list_laws/law/law_version` 走 runtime API(state_call);
// `ConstitutionImmutableManifest` 是 StorageValue(twox128(LegislationYuan)+
// twox128(ConstitutionImmutableManifest)),走 finalized storage 读。全部经
// `legislation_codec` 镜像解码。

import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;

import 'package:citizenapp/legislation/data/law_models.dart';
import 'package:citizenapp/legislation/data/legislation_codec.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/runtime_api.dart';

class LegislationApi {
  LegislationApi({RuntimeApi? runtimeApi, ChainRpc? chainRpc})
      : _api = runtimeApi ?? RuntimeApi(),
        _chainRpc = chainRpc ?? ChainRpc();

  final RuntimeApi _api;
  final ChainRpc _chainRpc;

  // 会话内内存缓存(法律体量大、改动稀;同实例复用避免重拉,尤其宪法 219KB)。
  // 跨会话 Isar 持久缓存留后续(ADR-018 R3)。
  final Map<int, Law> _lawCache = {};
  final Map<String, LawVersion> _versionCache = {};
  ImmutableManifest? _manifestCache;

  /// 某层级+行政区范围下的全部 law_id(`list_laws(tier, scope_code)`)。
  Future<List<int>> listLaws(LawTier tier, int scopeCode) async {
    final raw = await _api.call(
      'LegislationApi_list_laws',
      Uint8List.fromList([tier.index, ..._u32(scopeCode)]),
    );
    return raw == null ? const [] : decodeLawIds(raw);
  }

  /// 法律主记录(`law(law_id)`,不存在返回 null)。
  Future<Law?> law(int lawId) async {
    final cached = _lawCache[lawId];
    if (cached != null) return cached;
    final raw =
        await _api.call('LegislationApi_law', Uint8List.fromList(_u64(lawId)));
    if (raw == null) return null;
    final inner = decodeOptionBytes(raw);
    if (inner == null) return null;
    final law = decodeLaw(inner);
    _lawCache[lawId] = law;
    return law;
  }

  /// 法律某版本正文(`law_version(law_id, version)`,不存在返回 null)。
  Future<LawVersion?> lawVersion(int lawId, int version) async {
    final key = '$lawId:$version';
    final cached = _versionCache[key];
    if (cached != null) return cached;
    final raw = await _api.call(
      'LegislationApi_law_version',
      Uint8List.fromList([..._u64(lawId), ..._u32(version)]),
    );
    if (raw == null) return null;
    final inner = decodeOptionBytes(raw);
    if (inner == null) return null;
    final v = decodeLawVersion(inner);
    _versionCache[key] = v;
    return v;
  }

  /// 宪法不可修改条款 manifest(展示「不可修改」徽章用)。
  Future<ImmutableManifest?> immutableManifest() async {
    if (_manifestCache != null) return _manifestCache;
    final key =
        '0x${_hex(_storageValueKey('LegislationYuan', 'ConstitutionImmutableManifest'))}';
    final raw = await _chainRpc.fetchStorage(key);
    if (raw == null) return null;
    return _manifestCache = decodeImmutableManifest(raw);
  }

  Uint8List _storageValueKey(String pallet, String item) {
    final p = Hasher.twoxx128.hashString(pallet);
    final s = Hasher.twoxx128.hashString(item);
    return Uint8List.fromList([...p, ...s]);
  }

  String _hex(Uint8List b) =>
      b.map((x) => x.toRadixString(16).padLeft(2, '0')).join();

  List<int> _u32(int v) =>
      [v & 0xff, (v >> 8) & 0xff, (v >> 16) & 0xff, (v >> 24) & 0xff];

  List<int> _u64(int v) => List.generate(8, (k) => (v >> (8 * k)) & 0xff);
}
