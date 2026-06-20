import 'package:wuminapp_mobile/citizen/public/data/public_institution_repository.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_provinces.dart';

/// 机构目录只读反查结果(省/市/法定代表人),源自公权目录本地 Isar 库。
class SfidDirectoryInfo {
  const SfidDirectoryInfo(
      {this.provinceName, this.cityName, this.legalRepName});

  final String? provinceName;
  final String? cityName;
  final String? legalRepName;
}

/// 机构目录只读反查:复用公权目录本地 Isar 库(与 SFID subjects 同源),按
/// `sfid_number` 取省/市/法定代表人。
///
/// 中文注释:治理详情借此与公权详情**统一展示「法定代表人 / 所属地」**——治理内置
/// 机构(国储会/省储会/省储行)都带真实 SFID 号且在公权确定性目录内,可直接反查。
/// 反查前先 `ensureSynced`(版本驱动增量同步,版本没变秒过);反查不到(如链上
/// 注册机构账户不在确定性目录)返回 null,调用方留空。
class SfidDirectoryLookup {
  SfidDirectoryLookup({PublicInstitutionRepository? repository})
      : _repo = repository ?? PublicInstitutionRepository();

  final PublicInstitutionRepository _repo;

  Future<SfidDirectoryInfo?> lookup(String sfidNumber) async {
    try {
      await _repo.ensureSynced();
    } on Exception {
      // 同步失败不致命:按现有库继续查,查不到则留空。
    }
    final entity = await _repo.getBySfid(sfidNumber);
    if (entity == null) return null;
    // 机构只存 code(ADR-021);省名走链上常量、市名查字典 join。
    return SfidDirectoryInfo(
      provinceName: provinceFullNameByCode(entity.provinceCode),
      cityName: await _repo.cityName(entity.provinceCode, entity.cityCode),
      legalRepName: entity.legalRepName,
    );
  }
}
