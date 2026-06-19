// 只读派生数据的版本游标存储(与 schemaVersion 解耦)。
//
// 中文注释:行政区/公权机构是只读派生数据(无用户数据),数据新鲜度完全由本
// helper 独立管——和 `wallet_isar.dart` 里 `schemaVersion<7` 的清表逻辑(app
// 结构迁移)无关,绝不互相牵连。复用 [AppKvEntity](不新增 Isar schema):
//   - `<前缀>.data_version`  = 全局包版本(string),相等即整体秒过
//   - `<前缀>.prov_vers`     = per-province ver map 的 JSON(`{"GZ":"abc..."}`)
// 逐省 reconcile 后逐省落 prov_vers,中断可续;全部省过完才落 data_version。
//
// 抽象成接口以便载入逻辑用内存 fake 单测,不依赖 Isar 真库;生产实现见
// [IsarDataVersionKv]。

import 'dart:convert';

import 'package:isar_community/isar.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';

/// 版本游标读写接口。
abstract interface class DataVersionKv {
  /// 默认走全局 [WalletIsar] 的 Isar 实现;`isar` 可注入供集成测试。
  factory DataVersionKv({required String namespace, Isar? isar}) =
      IsarDataVersionKv;

  /// 读全局包版本;从未写过返回 null(首装)。
  Future<String?> readGlobalVersion();

  /// 写全局包版本(全部省 reconcile 成功后才落,作完成标记)。
  Future<void> writeGlobalVersion(String version);

  /// 读 per-province ver map;无 / 解析失败返回空 map(走全量 reconcile)。
  Future<Map<String, String>> readProvinceVersions();

  /// 整表覆盖写 per-province ver map(逐省 reconcile 后每省落一次,中断可续)。
  Future<void> writeProvinceVersions(Map<String, String> versions);
}

/// Isar 实现:复用 [AppKvEntity],不新增 schema。
class IsarDataVersionKv implements DataVersionKv {
  IsarDataVersionKv({required this.namespace, Isar? isar}) : _injected = isar;

  /// 键命名空间,如 `admin_division` / `public_institution`。
  final String namespace;
  final Isar? _injected;

  String get _globalKey => '$namespace.data_version';
  String get _provVersKey => '$namespace.prov_vers';

  Future<Isar> _db() async => _injected ?? await WalletIsar.instance.db();

  Future<T> _write<T>(Future<T> Function(Isar isar) action) async {
    final injected = _injected;
    if (injected != null) {
      return injected.writeTxn(() => action(injected));
    }
    return WalletIsar.instance.writeTxn(action);
  }

  @override
  Future<String?> readGlobalVersion() async {
    final isar = await _db();
    final row =
        await isar.appKvEntitys.filter().keyEqualTo(_globalKey).findFirst();
    return row?.stringValue;
  }

  @override
  Future<void> writeGlobalVersion(String version) async {
    await _write((isar) async {
      final existing =
          await isar.appKvEntitys.filter().keyEqualTo(_globalKey).findFirst();
      final entity = (existing ?? AppKvEntity())
        ..key = _globalKey
        ..stringValue = version;
      await isar.appKvEntitys.put(entity);
    });
  }

  @override
  Future<Map<String, String>> readProvinceVersions() async {
    final isar = await _db();
    final row =
        await isar.appKvEntitys.filter().keyEqualTo(_provVersKey).findFirst();
    return decodeProvinceVersions(row?.stringValue);
  }

  @override
  Future<void> writeProvinceVersions(Map<String, String> versions) async {
    final encoded = jsonEncode(versions);
    await _write((isar) async {
      final existing =
          await isar.appKvEntitys.filter().keyEqualTo(_provVersKey).findFirst();
      final entity = (existing ?? AppKvEntity())
        ..key = _provVersKey
        ..stringValue = encoded;
      await isar.appKvEntitys.put(entity);
    });
  }

  /// 解析 per-province ver map 的 JSON 字符串(无 / 非法返回空 map)。
  static Map<String, String> decodeProvinceVersions(String? raw) {
    if (raw == null || raw.isEmpty) return <String, String>{};
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return <String, String>{};
      return decoded.map((key, value) => MapEntry(key, value.toString()));
    } on FormatException {
      return <String, String>{};
    }
  }
}
