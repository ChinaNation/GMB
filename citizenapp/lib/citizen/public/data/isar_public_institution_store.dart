// 公权机构目录本地存储 —— Isar 实现(ADR-018 §九)。
//
// 省份规范顺序与各省版本戳复用 AppKvEntity(不新增 schema);
// 机构与订阅各自 collection。全部本地读写。

import 'dart:convert';

import 'package:isar_community/isar.dart';
import 'package:citizenapp/isar/wallet_isar.dart';

import 'public_institution_dto.dart';
import 'public_institution_store.dart';

const String _kProvinceOrderKey = 'public_institutions:provinces';
String _provinceVersionKey(String province) =>
    'public_institutions:version:$province';

class IsarPublicInstitutionStore implements PublicInstitutionStore {
  IsarPublicInstitutionStore({Isar? isar}) : _injected = isar;

  final Isar? _injected;

  Future<Isar> _db() async => _injected ?? await WalletIsar.instance.db();

  Future<T> _write<T>(Future<T> Function(Isar isar) action) async {
    final injected = _injected;
    if (injected != null) {
      return injected.writeTxn(() => action(injected));
    }
    return WalletIsar.instance.writeTxn(action);
  }

  /// 单事务批量上限:大数据包(数十万条)分块写,避免巨型事务卡 UI / 占内存。
  static const int _upsertChunk = 2000;

  @override
  Future<void> upsertInstitutions(
    List<PublicInstitutionDto> items, {
    required String catalogVersion,
  }) async {
    if (items.isEmpty) return;
    final now = DateTime.now().millisecondsSinceEpoch;
    // 走唯一索引批量 upsert(putAllByCidNumber),无需逐条 findFirst;
    // 分块成多个小事务,首次灌大包不卡 UI、不撑内存。
    for (var start = 0; start < items.length; start += _upsertChunk) {
      final end = (start + _upsertChunk).clamp(0, items.length);
      final entities = items
          .sublist(start, end)
          .map((dto) => dto.toEntity(
                catalogVersion: catalogVersion,
                updatedAtMillis: now,
              ))
          .toList(growable: false);
      await _write((isar) async {
        await isar.publicInstitutionEntitys.putAllByCidNumber(entities);
      });
    }
  }

  @override
  Future<void> setProvinceOrder(List<String> provinces) async {
    await _write((isar) async {
      final existing = await isar.appKvEntitys
          .filter()
          .keyEqualTo(_kProvinceOrderKey)
          .findFirst();
      final entity = (existing ?? AppKvEntity())
        ..key = _kProvinceOrderKey
        ..stringValue = jsonEncode(provinces);
      await isar.appKvEntitys.put(entity);
    });
  }

  @override
  Future<List<String>> listProvinces() async {
    final isar = await _db();
    final meta = await isar.appKvEntitys
        .filter()
        .keyEqualTo(_kProvinceOrderKey)
        .findFirst();
    final raw = meta?.stringValue;
    if (raw != null && raw.isNotEmpty) {
      final decoded = jsonDecode(raw) as List<dynamic>;
      return decoded.map((e) => e as String).toList(growable: false);
    }
    // 回退:无 manifest 时用已落库机构去重省 code(顺序不保证规范)。
    final all = await isar.publicInstitutionEntitys.where().findAll();
    final seen = <String>{};
    final out = <String>[];
    for (final e in all) {
      if (seen.add(e.provinceCode)) out.add(e.provinceCode);
    }
    return out;
  }

  @override
  Future<List<String>> listCities(String provinceCode) async {
    final isar = await _db();
    final rows = await isar.publicInstitutionEntitys
        .filter()
        .provinceCodeEqualTo(provinceCode)
        .findAll();
    // 按 cityCode 去重(市 code 省内唯一);名字由调用方查字典 join。
    final seen = <String>{};
    final out = <String>[];
    for (final e in rows) {
      if (e.cityCode.isNotEmpty && seen.add(e.cityCode)) out.add(e.cityCode);
    }
    return out;
  }

  @override
  Future<List<PublicInstitutionEntity>> listInstitutionsByCity(
    String provinceCode,
    String cityCode,
  ) async {
    final isar = await _db();
    return isar.publicInstitutionEntitys
        .filter()
        .provinceCodeEqualTo(provinceCode)
        .and()
        .cityCodeEqualTo(cityCode)
        .findAll();
  }

  @override
  Future<PublicInstitutionEntity?> getByCid(String cidNumber) async {
    final isar = await _db();
    return isar.publicInstitutionEntitys
        .filter()
        .cidNumberEqualTo(cidNumber)
        .findFirst();
  }

  @override
  Future<List<PublicInstitutionEntity>> listByInstitutionCodes(
    Set<String> institutionCodes,
  ) async {
    if (institutionCodes.isEmpty) return const [];
    final isar = await _db();
    // institutionCode 已建索引(ADR-028 P2);anyOf 走索引匹配,非全表扫。
    return isar.publicInstitutionEntitys
        .filter()
        .anyOf(institutionCodes,
            (q, code) => q.institutionCodeEqualTo(code))
        .findAll();
  }

  @override
  Future<List<PublicInstitutionEntity>> listByProvinceAndCodes(
    String provinceCode,
    Set<String> institutionCodes,
  ) async {
    if (institutionCodes.isEmpty) return const [];
    final isar = await _db();
    // provinceCode + institutionCode 均有索引;省内按码 anyOf,高效(ADR-028 P3)。
    return isar.publicInstitutionEntitys
        .filter()
        .provinceCodeEqualTo(provinceCode)
        .and()
        .anyOf(institutionCodes,
            (q, code) => q.institutionCodeEqualTo(code))
        .findAll();
  }

  @override
  Future<List<PublicInstitutionEntity>> institutionsOfProvince(
    String provinceCode,
  ) async {
    final isar = await _db();
    return isar.publicInstitutionEntitys
        .filter()
        .provinceCodeEqualTo(provinceCode)
        .findAll();
  }

  @override
  Future<List<String>> cidsOfProvince(String provinceCode) async {
    final rows = await institutionsOfProvince(provinceCode);
    return rows.map((e) => e.cidNumber).toList(growable: false);
  }

  @override
  Future<void> deleteByCids(List<String> cids) async {
    if (cids.isEmpty) return;
    for (var start = 0; start < cids.length; start += _upsertChunk) {
      final end = (start + _upsertChunk).clamp(0, cids.length);
      final chunk = cids.sublist(start, end);
      await _write((isar) async {
        await isar.publicInstitutionEntitys.deleteAllByCidNumber(chunk);
      });
    }
  }

  @override
  Future<int> institutionCount() async {
    final isar = await _db();
    return isar.publicInstitutionEntitys.count();
  }

  @override
  Future<String?> provinceVersion(String province) async {
    final isar = await _db();
    final meta = await isar.appKvEntitys
        .filter()
        .keyEqualTo(_provinceVersionKey(province))
        .findFirst();
    return meta?.stringValue;
  }

  @override
  Future<void> setProvinceVersion(String province, String version) async {
    await _write((isar) async {
      final key = _provinceVersionKey(province);
      final existing =
          await isar.appKvEntitys.filter().keyEqualTo(key).findFirst();
      final entity = (existing ?? AppKvEntity())
        ..key = key
        ..stringValue = version;
      await isar.appKvEntitys.put(entity);
    });
  }

  @override
  Future<void> subscribe(String walletPubkeyHex, String cidNumber) async {
    final key = subscriptionKeyOf(walletPubkeyHex, cidNumber);
    await _write((isar) async {
      final existing = await isar.publicInstitutionSubscriptionEntitys
          .filter()
          .subscriptionKeyEqualTo(key)
          .findFirst();
      if (existing != null) return;
      final entity = PublicInstitutionSubscriptionEntity()
        ..subscriptionKey = key
        ..walletPubkeyHex = walletPubkeyHex
        ..cidNumber = cidNumber
        ..subscribedAtMillis = DateTime.now().millisecondsSinceEpoch;
      await isar.publicInstitutionSubscriptionEntitys.put(entity);
    });
  }

  @override
  Future<void> unsubscribe(String walletPubkeyHex, String cidNumber) async {
    final key = subscriptionKeyOf(walletPubkeyHex, cidNumber);
    await _write((isar) async {
      final existing = await isar.publicInstitutionSubscriptionEntitys
          .filter()
          .subscriptionKeyEqualTo(key)
          .findFirst();
      if (existing != null) {
        await isar.publicInstitutionSubscriptionEntitys.delete(existing.id);
      }
    });
  }

  @override
  Future<bool> isSubscribed(String walletPubkeyHex, String cidNumber) async {
    final isar = await _db();
    final hit = await isar.publicInstitutionSubscriptionEntitys
        .filter()
        .subscriptionKeyEqualTo(subscriptionKeyOf(walletPubkeyHex, cidNumber))
        .findFirst();
    return hit != null;
  }

  @override
  Future<List<PublicInstitutionEntity>> listSubscribed(
    String walletPubkeyHex,
  ) async {
    final isar = await _db();
    final subs = await isar.publicInstitutionSubscriptionEntitys
        .filter()
        .walletPubkeyHexEqualTo(walletPubkeyHex)
        .findAll();
    final out = <PublicInstitutionEntity>[];
    for (final sub in subs) {
      final inst = await isar.publicInstitutionEntitys
          .filter()
          .cidNumberEqualTo(sub.cidNumber)
          .findFirst();
      if (inst != null) out.add(inst);
    }
    return out;
  }
}
