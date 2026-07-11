import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;

import 'package:citizenapp/rpc/chain_rpc.dart';

/// 立法投票阶段(链端 votingengine STAGE_LEG_*,Proposal.stage 字节)。
class LegStage {
  static const int house = 10; // 院内表决
  static const int referendum = 11; // 特别案公投
  static const int sign = 12; // 行政首长签署
  static const int override_ = 13; // 三人会签
  static const int guard = 14; // 护宪大法官终审
}

/// 提案状态(VotingEngine.Proposals 第 3 字节)。
class LegProposalStatus {
  static const int voting = 0;
  static const int passed = 1;
  static const int rejected = 2;
}

/// 院/机构引用(机构码 + 账户 hex)。
class LegHouseRef {
  const LegHouseRef({required this.code, required this.accountHex});
  final String code;
  final String accountHex;
}

/// 立法提案元数据(legislation-vote LegMeta 的客户端镜像)。
class LegMeta {
  const LegMeta({
    required this.voteType,
    required this.houses,
    required this.currentHouse,
    required this.referendumRequired,
    required this.executive,
    required this.legislature,
    required this.needsGuard,
  });

  final int voteType; // 0常规/1常规教育/2重要/3重要教育/4特别
  final List<LegHouseRef> houses; // houses[0]=发起院
  final int currentHouse;
  final bool referendumRequired;
  final LegHouseRef executive; // 行政签署机构
  final LegHouseRef? legislature; // 两院级立法院(单院=null)
  final bool needsGuard; // 修宪→护宪终审
}

/// 提案核心阶段/状态(VotingEngine.Proposals 头三字节)。
class LegProposalState {
  const LegProposalState({
    required this.kind,
    required this.stage,
    required this.status,
  });
  final int kind;
  final int stage;
  final int status;
}

/// 立法投票查询服务(legislation-vote LegMeta/计票/签署账本 + 核心 Proposals 阶段)。
///
/// 立法专属投票状态(LegMeta/各 tally/签署记录)集中在本服务,
/// 不借用 internal-vote 查询(账本结构不同)。核心 Proposal 阶段/状态读
/// VotingEngine.Proposals。
class LegislationVoteQueryService {
  LegislationVoteQueryService({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  static const String _votePallet = 'LegislationVote';

  /// 核心提案阶段/状态(不存在返回 null)。
  Future<LegProposalState?> fetchProposalState(int proposalId) async {
    final key = _mapKey('VotingEngine', 'Proposals', _u64Le(proposalId));
    final data = await _rpc.fetchStorage('0x${_hex(key)}');
    if (data == null || data.length < 3) return null;
    return LegProposalState(kind: data[0], stage: data[1], status: data[2]);
  }

  /// 立法提案元数据(不存在返回 null)。
  Future<LegMeta?> fetchMeta(int proposalId) async {
    final key = _mapKey(_votePallet, 'LegMeta', _u64Le(proposalId));
    final data = await _rpc.fetchStorage('0x${_hex(key)}');
    if (data == null || data.isEmpty) return null;
    final c = _Cursor(data);
    final voteType = c.u8();
    final houses = c.vec(() => LegHouseRef(
          code: _codeStr(c.bytes(4)),
          accountHex: _hex(c.bytes(32)),
        ));
    final currentHouse = c.u32();
    final referendumRequired = c.u8() == 1;
    final executive = LegHouseRef(
      code: _codeStr(c.bytes(4)),
      accountHex: _hex(c.bytes(32)),
    );
    final legislature = c.u8() == 0
        ? null
        : LegHouseRef(
            code: _codeStr(c.bytes(4)), accountHex: _hex(c.bytes(32)));
    final needsGuard = c.u8() == 1;
    return LegMeta(
      voteType: voteType,
      houses: houses,
      currentHouse: currentHouse,
      referendumRequired: referendumRequired,
      executive: executive,
      legislature: legislature,
      needsGuard: needsGuard,
    );
  }

  /// 当前院计票(VoteCountU32:yes u32 + no u32)。
  Future<({int yes, int no})> fetchHouseTally(int proposalId) async {
    final key = _mapKey(_votePallet, 'LegHouseTally', _u64Le(proposalId));
    final data = await _rpc.fetchStorage('0x${_hex(key)}');
    if (data == null || data.length < 8) return (yes: 0, no: 0);
    return (yes: _u32(data, 0), no: _u32(data, 4));
  }

  /// 公投计票(VoteCountU64:yes u64 + no u64)。
  Future<({int yes, int no})> fetchReferendumTally(int proposalId) async {
    final key = _mapKey(_votePallet, 'LegReferendumTally', _u64Le(proposalId));
    final data = await _rpc.fetchStorage('0x${_hex(key)}');
    if (data == null || data.length < 16) return (yes: 0, no: 0);
    return (yes: _u64(data, 0), no: _u64(data, 8));
  }

  /// 某议员/委员对某提案的院内投票(null=未投/true=赞成/false=反对)。
  Future<bool?> fetchHouseVote(int proposalId, String pubkeyHex) async {
    final key = _doubleMapKey(
      _votePallet,
      'LegHouseVotesByAdmin',
      _u64Le(proposalId),
      _hexDecode(pubkeyHex),
    );
    final data = await _rpc.fetchStorage('0x${_hex(key)}');
    if (data == null || data.isEmpty) return null;
    return data[0] == 1;
  }

  /// 三人会签记录(LegOverrideSigns:BoundedVec<(AccountId,bool)>)。
  Future<List<({String pubkeyHex, bool approve})>> fetchOverrideSigns(
      int proposalId) {
    return _fetchSigns('LegOverrideSigns', proposalId);
  }

  /// 护宪大法官终审记录(LegGuardSigns:BoundedVec<(AccountId,bool)>)。
  Future<List<({String pubkeyHex, bool approve})>> fetchGuardSigns(
      int proposalId) {
    return _fetchSigns('LegGuardSigns', proposalId);
  }

  Future<List<({String pubkeyHex, bool approve})>> _fetchSigns(
    String storage,
    int proposalId,
  ) async {
    final key = _mapKey(_votePallet, storage, _u64Le(proposalId));
    final data = await _rpc.fetchStorage('0x${_hex(key)}');
    if (data == null || data.isEmpty) return const [];
    final c = _Cursor(data);
    return c.vec(() => (pubkeyHex: _hex(c.bytes(32)), approve: c.u8() == 1));
  }

  // ──── key 构造 ────

  Uint8List _mapKey(String pallet, String storage, Uint8List keyData) {
    final p = Hasher.twoxx128.hashString(pallet);
    final s = Hasher.twoxx128.hashString(storage);
    final k = _blake2128Concat(keyData);
    return Uint8List.fromList([...p, ...s, ...k]);
  }

  Uint8List _doubleMapKey(
    String pallet,
    String storage,
    Uint8List key1,
    Uint8List key2,
  ) {
    final p = Hasher.twoxx128.hashString(pallet);
    final s = Hasher.twoxx128.hashString(storage);
    final k1 = _blake2128Concat(key1);
    final k2 = _blake2128Concat(key2);
    return Uint8List.fromList([...p, ...s, ...k1, ...k2]);
  }

  Uint8List _blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    return Uint8List.fromList([...hash, ...data]);
  }

  Uint8List _u64Le(int value) {
    final bytes = Uint8List(8);
    ByteData.sublistView(bytes).setUint64(0, value, Endian.little);
    return bytes;
  }

  int _u32(Uint8List data, int offset) =>
      ByteData.sublistView(data).getUint32(offset, Endian.little);

  int _u64(Uint8List data, int offset) =>
      ByteData.sublistView(data).getUint64(offset, Endian.little);

  Uint8List _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final out = Uint8List(h.length ~/ 2);
    for (var i = 0; i < out.length; i++) {
      out[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return out;
  }

  static String _hex(Uint8List b) =>
      b.map((x) => x.toRadixString(16).padLeft(2, '0')).join();

  static String _codeStr(Uint8List b) {
    var end = b.length;
    while (end > 0 && b[end - 1] == 0) {
      end--;
    }
    return String.fromCharCodes(b.sublist(0, end));
  }
}

/// 极简 SCALE 游标(本服务局部用)。
class _Cursor {
  _Cursor(this.data);
  final Uint8List data;
  int _i = 0;

  int u8() => data[_i++];

  int u32() {
    final v = data[_i] |
        (data[_i + 1] << 8) |
        (data[_i + 2] << 16) |
        (data[_i + 3] << 24);
    _i += 4;
    return v;
  }

  int compact() {
    final b0 = data[_i];
    final mode = b0 & 0x03;
    if (mode == 0) {
      _i += 1;
      return b0 >> 2;
    }
    if (mode == 1) {
      final v = (data[_i] | (data[_i + 1] << 8)) >> 2;
      _i += 2;
      return v;
    }
    if (mode == 2) {
      final v = (data[_i] |
              (data[_i + 1] << 8) |
              (data[_i + 2] << 16) |
              (data[_i + 3] << 24)) >>
          2;
      _i += 4;
      return v;
    }
    final n = (b0 >> 2) + 4;
    _i += 1;
    var v = 0;
    for (var k = 0; k < n; k++) {
      v |= data[_i + k] << (8 * k);
    }
    _i += n;
    return v;
  }

  Uint8List bytes(int n) {
    final b = data.sublist(_i, _i + n);
    _i += n;
    return b;
  }

  List<T> vec<T>(T Function() item) {
    final n = compact();
    return [for (var k = 0; k < n; k++) item()];
  }
}
