// 公权机构目录增量同步(ADR-018 §九 混合模式 ②)。
//
// 中文注释:版本(MAX updated_at)无变化则跳过(零拉取);有变化时走 **keyset + since
// 增量**——带本地版本 since 去问,后端只回 updated_at 之后变过的行(通常空或几条),
// keyset 翻页直到无更多,upsert 写回 + 推进版本戳。首次(无本地版本)since=null 即全量。
// R3:走 SFID HTTP,不扫链。

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
    final localVersion = await store.provinceVersion(province);
    if (remoteVersion != null && remoteVersion == localVersion) {
      return false;
    }
    final newVersion =
        remoteVersion ?? DateTime.now().millisecondsSinceEpoch.toString();

    // 有本地版本→只拉 updated_at 之后的增量;无本地版本→全量(since=null)。
    final since = localVersion;
    final changed = <PublicInstitutionDto>[];
    String? afterSfid;
    while (true) {
      final page = await api.fetchPage(
        provinceName: province,
        sinceVersion: since,
        afterSfid: afterSfid,
        pageSize: 500,
      );
      changed.addAll(page.items);
      if (!page.hasMore || page.items.isEmpty) break;
      afterSfid = page.nextCursor ?? page.items.last.sfidNumber;
    }

    if (changed.isNotEmpty) {
      await store.upsertInstitutions(changed, catalogVersion: newVersion);
    }
    await store.setProvinceVersion(province, newVersion);
    return true;
  }
}
