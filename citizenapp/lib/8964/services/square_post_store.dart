import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:isar_community/isar.dart';

import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/isar/app_isar.dart';

/// 本人已发布广场内容的只读值对象。
///
/// 正文、标题、文章块和媒体声明的唯一内容真源是 [manifestBytes]；调用方若需展示，
/// 必须从这份已经通过哈希校验的原始 manifest 解析，不能另存一套本地正文字段。
class SquareLocalPost {
  SquareLocalPost({
    required this.postId,
    required this.cidNumber,
    required this.accountId,
    required this.postCategory,
    required this.contentFormat,
    required Uint8List manifestBytes,
    required this.contentHash,
    required this.storageReceiptId,
    required this.chainBlock,
    required this.createdAt,
    required this.postState,
  }) : manifestBytes = Uint8List.fromList(manifestBytes);

  final String postId;
  final String cidNumber;
  final String accountId;
  final String postCategory;
  final String contentFormat;
  final Uint8List manifestBytes;
  final String contentHash;
  final String storageReceiptId;
  final int? chainBlock;
  final int createdAt;
  final String postState;
}

/// Worker 本人副本接口的一页结果。
///
/// [nextCursor] 是 Worker 签发的稳定分页游标，客户端只原样回传，不解析或自行构造。
class SquareLocalPostPage {
  const SquareLocalPostPage({
    required this.items,
    required this.nextCursor,
  });

  final List<SquareLocalPost> items;
  final String? nextCursor;
}

/// 已完成一次完整回灌时记录的远端最新发布事实。
///
/// 它只用于缩短后续增量扫描，不是本地内容真源，也不含设备时间。远端为空时
/// [newestPostId] 为 null、[newestCreatedAt] 为 0。
class SquarePostSyncCheckpoint {
  const SquarePostSyncCheckpoint({
    required this.newestPostId,
    required this.newestCreatedAt,
  });

  final String? newestPostId;
  final int newestCreatedAt;
}

/// 发布链路写入本人副本所需的最小边界，便于隔离测试失败与重试语义。
abstract class SquareLocalPostWriter {
  Future<void> save(SquareLocalPost post);
}

/// 单帖删除所需的最小本地边界。
abstract class SquareLocalPostDeletionStore {
  Future<bool> delete({
    required String cidNumber,
    required String postId,
  });
}

/// 注销时按永久 CID 删除全部本人副本和同步检查点的最小边界。
abstract class SquareLocalPostBulkDeletionStore {
  Future<int> deleteAllByCid(String cidNumber);
}

class SquarePostStoreException implements Exception {
  const SquarePostStoreException(this.message);

  final String message;

  @override
  String toString() => message;
}

/// 本人广场内容的本地持久化边界。
///
/// 这里同时承担写入前完整性闸门：只有 Worker 已确认的 `published` 内容可以入库，
/// 且原始 manifest 的 SHA-256、账户、分类和内容形态必须与外层链上锚字段一致。
/// 任何不一致都在进入 Isar 写事务前失败，绝不覆盖已有正确副本。
class SquarePostStore
    implements
        SquareLocalPostWriter,
        SquareLocalPostDeletionStore,
        SquareLocalPostBulkDeletionStore {
  const SquarePostStore();

  static const String publishedState = 'published';
  static const String manifestSchema = 'citizenapp.square.post.v1';
  static const String _syncCheckpointKeyPrefix =
      'square.local_posts.sync_checkpoint.';

  /// 保存或幂等覆盖一条本人已发布内容。
  ///
  /// [createdAt] 必须来自 Worker `square_posts.created_at`，本方法不读取设备时间。
  @override
  Future<void> save(SquareLocalPost post) async {
    await saveAll(<SquareLocalPost>[post]);
  }

  /// 原子保存一整页 Worker 回灌结果。
  ///
  /// 所有条目会先在事务外完成完整性校验；任一条不合法或同页 post_id 重复时整页拒绝，
  /// 保证同步中断后最多留下完整页，不会留下半页空洞。
  Future<void> saveAll(List<SquareLocalPost> posts) async {
    if (posts.isEmpty) return;
    final normalizedPosts = posts.map(_validated).toList(growable: false);
    final postIds = <String>{};
    for (final post in normalizedPosts) {
      if (!postIds.add(post.postId)) {
        throw const SquarePostStoreException('同一回灌页包含重复 post_id');
      }
    }

    await WalletIsar.instance.writeTxn((isar) async {
      final entities = <SquareLocalPostEntity>[];
      for (final normalized in normalizedPosts) {
        final existing =
            await isar.squareLocalPostEntitys.getByPostId(normalized.postId);
        if (existing != null) {
          _assertSamePublishedFact(existing, normalized);
        }
        final entity = existing ?? SquareLocalPostEntity();
        entity
          ..postId = normalized.postId
          ..cidNumber = normalized.cidNumber
          ..accountId = normalized.accountId
          ..postCategory = normalized.postCategory
          ..contentFormat = normalized.contentFormat
          ..manifestBytes = List<int>.from(normalized.manifestBytes)
          ..contentHash = normalized.contentHash
          ..storageReceiptId = normalized.storageReceiptId
          ..chainBlock = normalized.chainBlock
          ..createdAt = normalized.createdAt
          ..postState = normalized.postState;
        entities.add(entity);
      }
      await isar.squareLocalPostEntitys.putAll(entities);
    });
  }

  /// 读取已完成回灌检查点；损坏值 fail-closed，禁止把错误标记当作同步完成。
  Future<SquarePostSyncCheckpoint?> readSyncCheckpoint(String cidNumber) async {
    _validateCidNumber(cidNumber);
    return WalletIsar.instance.read((isar) async {
      final entity = await isar.appKvEntitys
          .getByKey('$_syncCheckpointKeyPrefix$cidNumber');
      final raw = entity?.stringValue;
      if (raw == null) return null;
      try {
        final decoded = jsonDecode(raw);
        if (decoded is! Map<String, dynamic> ||
            decoded['cid_number'] != cidNumber ||
            decoded['newest_created_at'] is! int) {
          throw const SquarePostStoreException('本人副本同步检查点损坏');
        }
        final postId = decoded['newest_post_id'];
        final createdAt = decoded['newest_created_at'] as int;
        final validEmpty = postId == null && createdAt == 0;
        final validPost =
            postId is String && postId.trim().isNotEmpty && createdAt > 0;
        if (!validEmpty && !validPost) {
          throw const SquarePostStoreException('本人副本同步检查点损坏');
        }
        return SquarePostSyncCheckpoint(
          newestPostId: postId as String?,
          newestCreatedAt: createdAt,
        );
      } on SquarePostStoreException {
        rethrow;
      } on Object {
        throw const SquarePostStoreException('本人副本同步检查点损坏');
      }
    });
  }

  /// 仅在所有目标页成功落盘后更新检查点，不写设备时间。
  Future<void> writeSyncCheckpoint({
    required String cidNumber,
    required SquarePostSyncCheckpoint checkpoint,
  }) async {
    _validateCidNumber(cidNumber);
    final postId = checkpoint.newestPostId;
    final validEmpty = postId == null && checkpoint.newestCreatedAt == 0;
    final validPost = postId != null &&
        postId.trim().isNotEmpty &&
        checkpoint.newestCreatedAt > 0;
    if (!validEmpty && !validPost) {
      throw const SquarePostStoreException('本人副本同步检查点不合法');
    }
    await WalletIsar.instance.writeTxn((isar) async {
      final key = '$_syncCheckpointKeyPrefix$cidNumber';
      final entity = await isar.appKvEntitys.getByKey(key) ?? AppKvEntity();
      entity
        ..key = key
        ..stringValue = jsonEncode({
          'cid_number': cidNumber,
          'newest_post_id': postId,
          'newest_created_at': checkpoint.newestCreatedAt,
        })
        ..intValue = null
        ..boolValue = null;
      await isar.appKvEntitys.put(entity);
    });
  }

  Future<SquareLocalPost?> read({
    required String cidNumber,
    required String postId,
  }) async {
    _validateCidNumber(cidNumber);
    _requireNonEmpty(postId, 'post_id');
    return WalletIsar.instance.read((isar) async {
      final entity = await isar.squareLocalPostEntitys.getByPostId(postId);
      if (entity == null || entity.cidNumber != cidNumber) {
        return null;
      }
      return _fromEntity(entity);
    });
  }

  /// 按 Worker 时间倒序、post_id 倒序返回指定 CID 的全部本人副本。
  ///
  /// 排序不用设备时间；post_id 是同毫秒发布时的稳定次级排序键。
  Future<List<SquareLocalPost>> listByCid(String cidNumber) async {
    _validateCidNumber(cidNumber);
    return WalletIsar.instance.read((isar) async {
      final entities = await isar.squareLocalPostEntitys
          .filter()
          .cidNumberEqualTo(cidNumber)
          .findAll();
      entities.sort((left, right) {
        final byCreatedAt = right.createdAt.compareTo(left.createdAt);
        if (byCreatedAt != 0) return byCreatedAt;
        return right.postId.compareTo(left.postId);
      });
      return entities.map(_fromEntity).toList(growable: false);
    });
  }

  /// 仅在帖子确属指定 CID 时删除，禁止只凭全局 post_id 越过归属边界。
  @override
  Future<bool> delete({
    required String cidNumber,
    required String postId,
  }) async {
    _validateCidNumber(cidNumber);
    _requireNonEmpty(postId, 'post_id');
    return WalletIsar.instance.writeTxn((isar) async {
      final entity = await isar.squareLocalPostEntitys.getByPostId(postId);
      if (entity == null || entity.cidNumber != cidNumber) {
        return false;
      }
      return isar.squareLocalPostEntitys.delete(entity.id);
    });
  }

  /// 用户注销的本地清理入口；服务端硬删除成功后按永久 CID 一次删净。
  @override
  Future<int> deleteAllByCid(String cidNumber) async {
    _validateCidNumber(cidNumber);
    return WalletIsar.instance.writeTxn((isar) async {
      final entities = await isar.squareLocalPostEntitys
          .filter()
          .cidNumberEqualTo(cidNumber)
          .findAll();
      final deleted = entities.isEmpty
          ? 0
          : await isar.squareLocalPostEntitys
              .deleteAll(entities.map((entity) => entity.id).toList());
      final checkpoint = await isar.appKvEntitys
          .getByKey('$_syncCheckpointKeyPrefix$cidNumber');
      if (checkpoint != null) {
        await isar.appKvEntitys.delete(checkpoint.id);
      }
      return deleted;
    });
  }

  static SquareLocalPost _validated(SquareLocalPost post) {
    _requireNonEmpty(post.postId, 'post_id');
    _validateCidNumber(post.cidNumber);
    if (!isAccountIdText(post.accountId)) {
      throw const SquarePostStoreException('account_id 格式不合法');
    }
    if (post.postCategory != 'normal' && post.postCategory != 'campaign') {
      throw const SquarePostStoreException('post_category 不合法');
    }
    if (post.contentFormat != 'normal' && post.contentFormat != 'article') {
      throw const SquarePostStoreException('content_format 不合法');
    }
    if (!RegExp(r'^[0-9a-f]{64}$').hasMatch(post.contentHash)) {
      throw const SquarePostStoreException('content_hash 格式不合法');
    }
    _requireNonEmpty(post.storageReceiptId, 'storage_receipt_id');
    if (post.chainBlock != null && post.chainBlock! < 0) {
      throw const SquarePostStoreException('chain_block 不合法');
    }
    if (post.createdAt <= 0) {
      throw const SquarePostStoreException('created_at 不合法');
    }
    if (post.postState != publishedState) {
      throw const SquarePostStoreException('post_state 只允许 published');
    }

    final bytes = Uint8List.fromList(post.manifestBytes);
    final actualHash = sha256.convert(bytes).toString();
    if (actualHash != post.contentHash) {
      throw const SquarePostStoreException('manifest 与 content_hash 不一致');
    }

    final manifest = _decodeManifest(bytes);
    if (manifest['schema'] != manifestSchema) {
      throw const SquarePostStoreException('manifest schema 不合法');
    }
    if (manifest['account_id'] != post.accountId) {
      throw const SquarePostStoreException('manifest account_id 不一致');
    }
    if (manifest['post_category'] != post.postCategory) {
      throw const SquarePostStoreException('manifest post_category 不一致');
    }
    final manifestContentFormat = manifest['content_format'] ?? 'normal';
    if (manifestContentFormat != post.contentFormat) {
      throw const SquarePostStoreException('manifest content_format 不一致');
    }
    if (manifest['text'] is! String || manifest['media_items'] is! List) {
      throw const SquarePostStoreException('manifest 正文或媒体声明不完整');
    }

    return SquareLocalPost(
      postId: post.postId,
      cidNumber: post.cidNumber,
      accountId: post.accountId,
      postCategory: post.postCategory,
      contentFormat: post.contentFormat,
      manifestBytes: bytes,
      contentHash: post.contentHash,
      storageReceiptId: post.storageReceiptId,
      chainBlock: post.chainBlock,
      createdAt: post.createdAt,
      postState: post.postState,
    );
  }

  static Map<String, dynamic> _decodeManifest(Uint8List bytes) {
    try {
      final decoded = jsonDecode(utf8.decode(bytes, allowMalformed: false));
      if (decoded is! Map<String, dynamic>) {
        throw const SquarePostStoreException('manifest 必须是 JSON 对象');
      }
      return decoded;
    } on SquarePostStoreException {
      rethrow;
    } on Object {
      throw const SquarePostStoreException('manifest 不是合法 UTF-8 JSON');
    }
  }

  static SquareLocalPost _fromEntity(SquareLocalPostEntity entity) {
    final post = SquareLocalPost(
      postId: entity.postId,
      cidNumber: entity.cidNumber,
      accountId: entity.accountId,
      postCategory: entity.postCategory,
      contentFormat: entity.contentFormat,
      manifestBytes: Uint8List.fromList(entity.manifestBytes),
      contentHash: entity.contentHash,
      storageReceiptId: entity.storageReceiptId,
      chainBlock: entity.chainBlock,
      createdAt: entity.createdAt,
      postState: entity.postState,
    );
    // 磁盘行也必须重新过完整性闸门；损坏数据不得以空白正文等方式静默降级。
    return _validated(post);
  }

  /// `post_id` 对应链上不可变发布事实；重复同步只允许逐字段完全相同。
  ///
  /// 这既阻止另一 CID 复用同一编号覆盖本人内容，也禁止把“编辑”误实现为原地改正文；
  /// 广场编辑必须发布新 post_id，再按已确认删除流程清理旧帖。
  static void _assertSamePublishedFact(
    SquareLocalPostEntity existing,
    SquareLocalPost incoming,
  ) {
    if (existing.cidNumber != incoming.cidNumber ||
        existing.accountId != incoming.accountId ||
        existing.postCategory != incoming.postCategory ||
        existing.contentFormat != incoming.contentFormat ||
        !_bytesEqual(existing.manifestBytes, incoming.manifestBytes) ||
        existing.contentHash != incoming.contentHash ||
        existing.storageReceiptId != incoming.storageReceiptId ||
        existing.chainBlock != incoming.chainBlock ||
        existing.createdAt != incoming.createdAt ||
        existing.postState != incoming.postState) {
      throw const SquarePostStoreException('post_id 已绑定另一条不可变发布事实');
    }
  }

  static bool _bytesEqual(List<int> left, List<int> right) {
    if (left.length != right.length) return false;
    for (var i = 0; i < left.length; i++) {
      if (left[i] != right[i]) return false;
    }
    return true;
  }

  static void _validateCidNumber(String cidNumber) {
    final bytes = utf8.encode(cidNumber);
    // CID 号码格式真源在 OnChina/链上；App 这里只复用跨端载荷的 1..32 UTF-8
    // 字节边界，不维护第二份号码正则。
    if (cidNumber.trim() != cidNumber || bytes.isEmpty || bytes.length > 32) {
      throw const SquarePostStoreException('cid_number 格式不合法');
    }
  }

  static void _requireNonEmpty(String value, String field) {
    if (value.trim().isEmpty || value.trim() != value) {
      throw SquarePostStoreException('$field 不能为空');
    }
  }
}
