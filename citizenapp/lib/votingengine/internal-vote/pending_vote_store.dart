import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:isar_community/isar.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/rpc/transfer_rpc.dart';
import 'package:citizenapp/votingengine/internal-vote/internal_vote_query_service.dart';

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

  /// 提交时使用的 runtime nonce。
  ///
  /// 该字段只用于日志和问题诊断，不能作为投票成功依据；
  /// 投票是否成功必须读取 runtime 投票引擎 storage。
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
      createdAt: DateTime.fromMillisecondsSinceEpoch(json['createdAt'] as int),
    );
  }
}

/// 待确认投票批量检查结果。
class PendingVoteConfirmSummary {
  const PendingVoteConfirmSummary({
    required this.stillPending,
    required this.confirmed,
    required this.lost,
    required this.errored,
  });

  final List<PendingVoteRecord> stillPending;
  final List<PendingVoteRecord> confirmed;
  final List<PendingVoteRecord> lost;
  final List<PendingVoteRecord> errored;
}

/// 待确认投票对应的链上投票记录查询函数。
///
/// 返回 null 表示 runtime 投票引擎尚未记录该管理员投票；返回 true/false
/// 都表示链上已经确认投票。
typedef PendingVoteChainLookup = Future<bool?> Function(
    PendingVoteRecord record);

/// 通用的待确认投票存储。
///
/// 使用 Isar [AppKvEntity] 持久化，key 格式：
/// `pending_vote:{proposalType}:{proposalId}:{walletPubkey}`
///
/// 所有投票页面共用：
/// - 仅在无法立即确认 runtime 投票 storage 时调 [save]
/// - 页面加载时调 [getPending] 判断是否有待确认投票
/// - 刷新时调 [confirmAll] 批量检查链上状态，已确认则自动清除
class PendingVoteStore {
  static final instance = PendingVoteStore._();
  PendingVoteStore._();

  /// 存储 key 前缀。
  static const _prefix = 'pending_vote';

  /// 投票 pending 最长确认窗口。
  ///
  /// GMB 链当前出块可能按分钟级推进，确认窗口必须覆盖多个出块周期。
  /// 超过窗口仍没有 runtime 投票记录，说明这条本地 pending 不能再阻塞
  /// 用户重投，应清掉并显示可重试状态。
  static const _votePendingTimeout = Duration(minutes: 20);

  String _key(String type, int id, String pubkey) =>
      '$_prefix:$type:$id:$pubkey';

  /// 保存一条待确认投票记录。
  Future<void> save(PendingVoteRecord record) async {
    final entity = AppKvEntity()
      ..key = _key(record.proposalType, record.proposalId, record.walletPubkey)
      ..stringValue = jsonEncode(record.toJson());
    await WalletIsar.instance.writeTxn((isar) => isar.appKvEntitys.put(entity));
  }

  /// 查询指定提案的所有待确认投票。
  ///
  /// 返回该提案下所有尚未确认的投票记录（可能有多个管理员分别投票）。
  Future<List<PendingVoteRecord>> getPending(
    String proposalType,
    int proposalId,
  ) async {
    final keyPrefix = '$_prefix:$proposalType:$proposalId:';
    final rows = await WalletIsar.instance.read((isar) {
      return isar.appKvEntitys.filter().keyStartsWith(keyPrefix).findAll();
    });
    return rows
        .where((r) => r.stringValue != null)
        .map((r) => PendingVoteRecord.fromJson(
            jsonDecode(r.stringValue!) as Map<String, dynamic>))
        .toList();
  }

  /// 删除一条待确认记录（已确认或已丢失）。
  Future<void> remove(
      String proposalType, int proposalId, String pubkey) async {
    final key = _key(proposalType, proposalId, pubkey);
    await WalletIsar.instance.writeTxn((isar) async {
      final row = await isar.appKvEntitys.filter().keyEqualTo(key).findFirst();
      if (row != null) {
        await isar.appKvEntitys.delete(row.id);
      }
    });
  }

  /// 批量检查链上确认状态，自动清除已确认 / 已丢失的记录。
  ///
  /// 返回仍处于待确认状态的记录列表。
  Future<List<PendingVoteRecord>> confirmAll(
    String proposalType,
    int proposalId,
    TransferRpc onchainRpc, {
    PendingVoteChainLookup? chainVoteLookup,
  }) async {
    final summary = await confirmAllDetailed(
      proposalType,
      proposalId,
      onchainRpc,
      chainVoteLookup: chainVoteLookup,
    );
    return summary.stillPending;
  }

  /// 批量检查链上确认状态，并返回完整分类结果。
  ///
  /// 详情页需要知道 lost/confirmed 结果来提示用户；投票结果只读
  /// runtime 投票引擎 storage，不再用 nonce 推断投票成功。
  Future<PendingVoteConfirmSummary> confirmAllDetailed(
    String proposalType,
    int proposalId,
    TransferRpc onchainRpc, {
    PendingVoteChainLookup? chainVoteLookup,
  }) async {
    final pending = await getPending(proposalType, proposalId);
    debugPrint(
        '[PendingVote.confirmAll] proposalId=$proposalId pending.len=${pending.length}');
    final stillPending = <PendingVoteRecord>[];
    final confirmed = <PendingVoteRecord>[];
    final lost = <PendingVoteRecord>[];
    final errored = <PendingVoteRecord>[];
    final lookup = chainVoteLookup ?? _defaultInternalVoteLookup;

    for (final record in pending) {
      try {
        // 投票是否成功只能以 runtime 投票引擎 storage 为准。
        // txHash / runtime nonce / 交易池 watch 只能用于诊断，不能直接证明
        // 管理员已经投票。
        final chainVote = await lookup(record);
        if (chainVote != null) {
          confirmed.add(record);
          await remove(proposalType, proposalId, record.walletPubkey);
          continue;
        }

        debugPrint(
            '[PendingVote.confirmAll] pubkey=${record.walletPubkey} usedNonce=${record.usedNonce} txHash=${record.txHash} runtimeVote=null');
        if (DateTime.now().difference(record.createdAt) > _votePendingTimeout) {
          // runtime 没有投票记录且已经超过确认窗口：视为本地这次
          // 提交没有形成有效投票，清掉 pending，避免 UI 无限“投票中”。
          lost.add(record);
          await remove(proposalType, proposalId, record.walletPubkey);
        } else {
          stillPending.add(record);
        }
      } catch (e) {
        // 节点不可达或投票 storage 查询失败时保留，下次刷新继续查 runtime。
        debugPrint(
            '[PendingVote.confirmAll] checkTxStatus 异常,保留记录 ${record.txHash}: $e');
        errored.add(record);
        stillPending.add(record);
      }
    }
    return PendingVoteConfirmSummary(
      stillPending: stillPending,
      confirmed: confirmed,
      lost: lost,
      errored: errored,
    );
  }

  Future<bool?> _defaultInternalVoteLookup(PendingVoteRecord record) {
    return InternalVoteQueryService()
        .fetchAdminVote(record.proposalId, record.walletPubkey);
  }
}
