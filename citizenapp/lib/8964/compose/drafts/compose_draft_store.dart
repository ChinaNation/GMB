import 'package:isar_community/isar.dart';

import 'package:citizenapp/8964/compose/drafts/compose_draft.dart';
import 'package:citizenapp/8964/compose/drafts/compose_draft_media.dart';
import 'package:citizenapp/isar/app_isar.dart';

/// 广场草稿箱存储契约（便于测试注入）。
abstract class SquareComposeDraftRepository {
  Future<void> save(SquareComposeDraft draft);
  Future<List<SquareComposeDraft>> list(String accountId);
  Future<void> delete(String accountId, String draftId);
}

/// 多草稿本地持久化：复用 AppKvEntity KV，key 前缀 + intValue=updated_at 排序。
/// 每人上限 [maxPerOwner]，超出淘汰最旧（连媒体目录）。
class SquareComposeDraftStore implements SquareComposeDraftRepository {
  SquareComposeDraftStore._();

  static final SquareComposeDraftStore instance = SquareComposeDraftStore._();

  static const String _prefix = 'square.compose.draft.';
  static const int maxPerOwner = 100;

  static String _key(String accountId, String draftId) =>
      '$_prefix$accountId.$draftId';
  static String _accountPrefix(String accountId) => '$_prefix$accountId.';

  @override
  Future<void> save(SquareComposeDraft draft) async {
    final overflowIds = <String>[];
    await WalletIsar.instance.writeTxn((isar) async {
      final key = _key(draft.accountId, draft.draftId);
      final entity = await isar.appKvEntitys.getByKey(key) ?? AppKvEntity();
      entity
        ..key = key
        ..stringValue = draft.toJsonString()
        ..intValue = draft.updatedAtMillis
        ..boolValue = null;
      await isar.appKvEntitys.putByKey(entity);
      // 上限淘汰最旧：intValue=updated_at 升序（最旧在前），删超额部分。
      final all = await isar.appKvEntitys
          .filter()
          .keyStartsWith(_accountPrefix(draft.accountId))
          .findAll();
      if (all.length > maxPerOwner) {
        all.sort((a, b) => (a.intValue ?? 0).compareTo(b.intValue ?? 0));
        for (var i = 0; i < all.length - maxPerOwner; i++) {
          final decoded = SquareComposeDraft.fromJsonString(all[i].stringValue);
          if (decoded != null) overflowIds.add(decoded.draftId);
          await isar.appKvEntitys.delete(all[i].id);
        }
      }
    });
    // 媒体目录清理放在事务外（文件 IO）。
    for (final id in overflowIds) {
      await ComposeDraftMedia.deleteDir(id);
    }
  }

  @override
  Future<List<SquareComposeDraft>> list(String accountId) {
    return WalletIsar.instance.read((isar) async {
      final entities = await isar.appKvEntitys
          .filter()
          .keyStartsWith(_accountPrefix(accountId))
          .findAll();
      final drafts = entities
          .map((e) => SquareComposeDraft.fromJsonString(e.stringValue))
          .whereType<SquareComposeDraft>()
          .toList();
      // 新→旧。
      drafts.sort((a, b) => b.updatedAtMillis.compareTo(a.updatedAtMillis));
      return drafts;
    });
  }

  @override
  Future<void> delete(String accountId, String draftId) async {
    await WalletIsar.instance.writeTxn((isar) async {
      final entity = await isar.appKvEntitys.getByKey(_key(accountId, draftId));
      if (entity != null) await isar.appKvEntitys.delete(entity.id);
    });
    await ComposeDraftMedia.deleteDir(draftId);
  }
}
