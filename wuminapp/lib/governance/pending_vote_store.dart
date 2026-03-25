import 'dart:convert';

import 'package:isar/isar.dart';
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';
import 'package:wuminapp_mobile/rpc/onchain.dart';

/// 待确认投票记录。
///
/// 投票交易提交到链上后、尚未被区块打包确认之前的中间状态。
/// 所有投票类型（转账提案、运行时升级等）共用此模型。
class PendingVoteRecord {
  PendingVoteRecord({
    required this.proposalType,
    required this.proposalId,
    required this.walletPubkey,
    required this.approve,
    required this.txHash,
    required this.usedNonce,
    required this.createdAt,
  });

  /// 提案类型标识，如 'transfer'、'runtime_upgrade'。
  final String proposalType;

  /// 提案 ID。
  final int proposalId;

  /// 投票人公钥 hex（不含 0x 前缀）。
  final String walletPubkey;

  /// 赞成 / 反对。
  final bool approve;

  /// 交易哈希（含 0x 前缀）。
  final String txHash;

  /// 提交时使用的 nonce，用于链上确认检查。
  final int usedNonce;

  /// 提交时间。
  final DateTime createdAt;

  Map<String, dynamic> toJson() => {
        'proposalType': proposalType,
        'proposalId': proposalId,
        'walletPubkey': walletPubkey,
        'approve': approve,
        'txHash': txHash,
        'usedNonce': usedNonce,
        'createdAt': createdAt.millisecondsSinceEpoch,
      };

  factory PendingVoteRecord.fromJson(Map<String, dynamic> json) {
    return PendingVoteRecord(
      proposalType: json['proposalType'] as String,
      proposalId: json['proposalId'] as int,
      walletPubkey: json['walletPubkey'] as String,
      approve: json['approve'] as bool,
      txHash: json['txHash'] as String,
      usedNonce: json['usedNonce'] as int,
      createdAt:
          DateTime.fromMillisecondsSinceEpoch(json['createdAt'] as int),
    );
  }
}

/// 通用的待确认投票存储。
///
/// 使用 Isar [AppKvEntity] 持久化，key 格式：
/// `pending_vote:{proposalType}:{proposalId}:{walletPubkey}`
///
/// 所有投票页面共用：
/// - 投票提交成功后调 [save]
/// - 页面加载时调 [getPending] 判断是否有待确认投票
/// - 刷新时调 [confirmAll] 批量检查链上状态，已确认则自动清除
class PendingVoteStore {
  static final instance = PendingVoteStore._();
  PendingVoteStore._();

  /// 存储 key 前缀。
  static const _prefix = 'pending_vote';

  String _key(String type, int id, String pubkey) =>
      '$_prefix:$type:$id:$pubkey';

  /// 保存一条待确认投票记录。
  Future<void> save(PendingVoteRecord record) async {
    final isar = await WalletIsar.instance.db();
    final entity = AppKvEntity()
      ..key = _key(record.proposalType, record.proposalId, record.walletPubkey)
      ..stringValue = jsonEncode(record.toJson());
    await isar.writeTxn(() => isar.appKvEntitys.put(entity));
  }

  /// 查询指定提案的所有待确认投票。
  ///
  /// 返回该提案下所有尚未确认的投票记录（可能有多个管理员分别投票）。
  Future<List<PendingVoteRecord>> getPending(
    String proposalType,
    int proposalId,
  ) async {
    final isar = await WalletIsar.instance.db();
    final keyPrefix = '$_prefix:$proposalType:$proposalId:';
    final rows = await isar.appKvEntitys
        .filter()
        .keyStartsWith(keyPrefix)
        .findAll();
    return rows
        .where((r) => r.stringValue != null)
        .map((r) =>
            PendingVoteRecord.fromJson(jsonDecode(r.stringValue!) as Map<String, dynamic>))
        .toList();
  }

  /// 删除一条待确认记录（已确认或已丢失）。
  Future<void> remove(String proposalType, int proposalId, String pubkey) async {
    final isar = await WalletIsar.instance.db();
    final key = _key(proposalType, proposalId, pubkey);
    final row =
        await isar.appKvEntitys.filter().keyEqualTo(key).findFirst();
    if (row != null) {
      await isar.writeTxn(() => isar.appKvEntitys.delete(row.id));
    }
  }

  /// 批量检查链上确认状态，自动清除已确认 / 已丢失的记录。
  ///
  /// 返回仍处于待确认状态的记录列表。
  Future<List<PendingVoteRecord>> confirmAll(
    String proposalType,
    int proposalId,
    OnchainRpc onchainRpc,
  ) async {
    final pending = await getPending(proposalType, proposalId);
    final stillPending = <PendingVoteRecord>[];

    for (final record in pending) {
      try {
        final result = await onchainRpc.checkTxStatus(
          pubkeyHex: record.walletPubkey,
          usedNonce: record.usedNonce,
          txHash: record.txHash,
        );
        if (result == TxConfirmResult.pending) {
          stillPending.add(record);
        } else {
          // confirmed 或 lost，清除记录
          await remove(proposalType, proposalId, record.walletPubkey);
        }
      } catch (_) {
        // 节点不可达时保留，下次重试
        stillPending.add(record);
      }
    }
    return stillPending;
  }
}
