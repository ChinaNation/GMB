import 'dart:convert';

import 'package:flutter/foundation.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/isar/app_isar.dart';

enum SquareDraftState {
  localOnly('local_only'),
  chainInBlockUploadPending('chain_in_block_upload_pending');

  const SquareDraftState(this.value);

  final String value;

  static SquareDraftState parse(Object? value) {
    final text = value?.toString();
    return SquareDraftState.values.firstWhere(
      (item) => item.value == text,
      orElse: () => SquareDraftState.localOnly,
    );
  }
}

class SquarePublishDraft {
  SquarePublishDraft({
    required this.ownerAccount,
    required this.postCategory,
    required this.text,
    required List<SquareLocalMediaDraft> mediaDrafts,
    required this.draftState,
    required this.updatedAtMillis,
    this.lastError,
    this.uploadId,
    this.postId,
    this.contentHash,
    this.storageReceiptId,
    this.storageUntil,
    this.txHash,
    this.blockHashHex,
  }) : mediaDrafts = List.unmodifiable(mediaDrafts);

  final String ownerAccount;
  final SquarePostCategory postCategory;
  final String text;
  final List<SquareLocalMediaDraft> mediaDrafts;
  final SquareDraftState draftState;
  final int updatedAtMillis;
  final String? lastError;
  final String? uploadId;
  final String? postId;
  final String? contentHash;
  final String? storageReceiptId;
  final int? storageUntil;
  final String? txHash;
  final String? blockHashHex;

  Map<String, Object?> toJson() => {
        'owner_account': ownerAccount,
        'post_category': postCategory.workerValue,
        'text': text,
        'media_drafts': mediaDrafts.map(_mediaDraftToJson).toList(),
        'draft_state': draftState.value,
        'updated_at_millis': updatedAtMillis,
        if (lastError != null) 'last_error': lastError,
        if (uploadId != null) 'upload_id': uploadId,
        if (postId != null) 'post_id': postId,
        if (contentHash != null) 'content_hash': contentHash,
        if (storageReceiptId != null) 'storage_receipt_id': storageReceiptId,
        if (storageUntil != null) 'storage_until': storageUntil,
        if (txHash != null) 'tx_hash': txHash,
        if (blockHashHex != null) 'block_hash_hex': blockHashHex,
      };

  static SquarePublishDraft? fromJsonString(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return null;
      final ownerAccount = decoded['owner_account']?.toString();
      final text = decoded['text']?.toString();
      final updatedAtMillis = _asInt(decoded['updated_at_millis']);
      if (ownerAccount == null ||
          ownerAccount.isEmpty ||
          text == null ||
          updatedAtMillis == null) {
        return null;
      }
      final media = decoded['media_drafts'];
      final mediaDrafts = media is List
          ? media
              .whereType<Map<String, dynamic>>()
              .map(_mediaDraftFromJson)
              .whereType<SquareLocalMediaDraft>()
              .toList(growable: false)
          : const <SquareLocalMediaDraft>[];
      return SquarePublishDraft(
        ownerAccount: ownerAccount,
        postCategory: _parsePostCategory(decoded['post_category']),
        text: text,
        mediaDrafts: mediaDrafts,
        draftState: SquareDraftState.parse(decoded['draft_state']),
        updatedAtMillis: updatedAtMillis,
        lastError: decoded['last_error']?.toString(),
        uploadId: decoded['upload_id']?.toString(),
        postId: decoded['post_id']?.toString(),
        contentHash: decoded['content_hash']?.toString(),
        storageReceiptId: decoded['storage_receipt_id']?.toString(),
        storageUntil: _asInt(decoded['storage_until']),
        txHash: decoded['tx_hash']?.toString(),
        blockHashHex: decoded['block_hash_hex']?.toString(),
      );
    } catch (e) {
      debugPrint('[SquareDraftStore] 草稿 JSON 解析失败: $e');
      return null;
    }
  }

  static Map<String, Object?> _mediaDraftToJson(SquareLocalMediaDraft draft) =>
      {
        'media_kind': draft.mediaKind.workerValue,
        'path': draft.path,
        'file_name': draft.fileName,
        'content_type': draft.contentType,
        'byte_size': draft.byteSize,
      };

  static SquareLocalMediaDraft? _mediaDraftFromJson(Map<String, dynamic> json) {
    final path = json['path']?.toString();
    final fileName = json['file_name']?.toString();
    final contentType = json['content_type']?.toString();
    final byteSize = _asInt(json['byte_size']);
    if (path == null ||
        path.isEmpty ||
        fileName == null ||
        fileName.isEmpty ||
        contentType == null ||
        contentType.isEmpty ||
        byteSize == null ||
        byteSize <= 0) {
      return null;
    }
    return SquareLocalMediaDraft(
      mediaKind: _parseMediaKind(json['media_kind']),
      path: path,
      fileName: fileName,
      contentType: contentType,
      byteSize: byteSize,
    );
  }

  static SquarePostCategory _parsePostCategory(Object? value) {
    final text = value?.toString();
    return SquarePostCategory.values.firstWhere(
      (item) => item.workerValue == text,
      orElse: () => SquarePostCategory.normal,
    );
  }

  static SquareMediaKind _parseMediaKind(Object? value) {
    final text = value?.toString();
    return SquareMediaKind.values.firstWhere(
      (item) => item.workerValue == text,
      orElse: () => SquareMediaKind.image,
    );
  }

  static int? _asInt(Object? value) {
    if (value == null) return null;
    if (value is int) return value;
    return int.tryParse(value.toString());
  }
}

abstract class SquareDraftRepository {
  Future<void> save(SquarePublishDraft draft);
  Future<SquarePublishDraft?> read(String ownerAccount);
  Future<void> delete(String ownerAccount);
}

class SquareDraftStore implements SquareDraftRepository {
  SquareDraftStore._();

  static final SquareDraftStore instance = SquareDraftStore._();
  static const String _prefix = 'square.publish.draft.';

  @override
  Future<void> save(SquarePublishDraft draft) async {
    await WalletIsar.instance.writeTxn((isar) async {
      final entity =
          await isar.appKvEntitys.getByKey(key(draft.ownerAccount)) ??
              AppKvEntity();
      entity
        ..key = key(draft.ownerAccount)
        ..stringValue = jsonEncode(draft.toJson())
        ..intValue = draft.updatedAtMillis
        ..boolValue = null;
      await isar.appKvEntitys.putByKey(entity);
    });
  }

  @override
  Future<SquarePublishDraft?> read(String ownerAccount) {
    return WalletIsar.instance.read((isar) async {
      final entity = await isar.appKvEntitys.getByKey(key(ownerAccount));
      return SquarePublishDraft.fromJsonString(entity?.stringValue);
    });
  }

  @override
  Future<void> delete(String ownerAccount) async {
    await WalletIsar.instance.writeTxn((isar) async {
      final entity = await isar.appKvEntitys.getByKey(key(ownerAccount));
      if (entity != null) {
        await isar.appKvEntitys.delete(entity.id);
      }
    });
  }

  static String key(String ownerAccount) => '$_prefix$ownerAccount';
}
