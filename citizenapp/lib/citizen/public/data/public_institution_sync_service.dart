// 公权机构链上投影增量同步(ADR-018 §九 混合模式 ②)。
//
// 本地版本等于 OnChina `chain_projection_state(public-gov)` 链投影版本时跳过;
// 有变化时走 keyset + since_version。OnChina 只是链上状态投影/索引服务,
// 不是公权机构真源;App 本地 Isar 也只是快照缓存。

import 'public_institution_api.dart';
import 'public_institution_dto.dart';
import 'public_institution_store.dart';

class PublicInstitutionSyncService {
  PublicInstitutionSyncService({
    required this.store,
    PublicInstitutionApi? api,
  }) : api = api ?? PublicInstitutionApi();

  final PublicInstitutionStore store;
  final PublicInstitutionApi api;

  /// 同步某省目录。返回 true=有更新(已写回),false=版本一致跳过。
  Future<bool> syncProvince(String province) async {
    final remote = await api.fetchVersion(provinceName: province);
    final remoteVersion = remote.manifestVersion;
    if (remoteVersion == null || remoteVersion.isEmpty) {
      throw StateError('public institution chain projection version missing');
    }
    final localVersion = await store.provinceVersion(province);
    if (remoteVersion == localVersion) {
      return false;
    }

    // 有本地版本→请求链投影增量;无本地版本→全量窗口。
    final since = localVersion;
    final changed = <PublicInstitutionDto>[];
    String? afterCid;
    while (true) {
      final page = await api.fetchPage(
        provinceName: province,
        sinceVersion: since,
        afterCid: afterCid,
        pageSize: 500,
      );
      changed.addAll(page.items);
      if (!page.hasMore || page.items.isEmpty) break;
      afterCid = page.nextCursor ?? page.items.last.cidNumber;
    }

    if (changed.isNotEmpty) {
      await store.upsertInstitutions(changed, catalogVersion: remoteVersion);
    }
    await store.setProvinceVersion(province, remoteVersion);
    return true;
  }
}
