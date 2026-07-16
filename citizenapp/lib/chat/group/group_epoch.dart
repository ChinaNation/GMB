// epoch 有序处理:乱序 Commit 缓冲 + 回放(纯逻辑,注入 process/buffer seam,可测)。
//
// MLS 群解密链要求 Commit 按 epoch 顺序应用;零存储瞬时中转不保证到达序。
// 规则(Rust `group_process` 已给出 status 判定):
//   applied    + commit → 成功应用,epoch 前进;回放缓冲中"下一个 epoch"的项
//   out_of_order        → 该消息 epoch > 当前 epoch,缓冲待补
//   stale               → epoch 已过(密钥已 ratchet),丢弃
// 详见 memory/05-modules/citizenapp/chat/CHAT_GROUP_TECHNICAL.md §6。

import '../crypto/mls_group_boundary.dart';
import '../crypto/mls_session.dart';
import '../proto/chat_envelope.pb.dart';

/// 处理一条 wire,返回 [GroupInbound]。
typedef GroupProcessSeam = Future<GroupInbound> Function(MlsWireMessage wire);

/// 缓冲一条乱序 envelope(键:groupId + messageEpoch)。
typedef GroupBufferPut = Future<void> Function(
  String groupId,
  int messageEpoch,
  ChatEnvelope envelope,
);

/// 取出并删除某 (groupId, messageEpoch) 下最早的一条缓冲 envelope;无则 null。
typedef GroupBufferTake = Future<ChatEnvelope?> Function(
  String groupId,
  int messageEpoch,
);

/// 从 envelope 还原 wire(用于回放)。
typedef WireFromEnvelope = MlsWireMessage Function(ChatEnvelope envelope);

class GroupEpochOrdering {
  const GroupEpochOrdering._();

  /// 有序处理一条入站 envelope:
  /// - out_of_order → 缓冲,返回该结果(调用方据此提示"等待前序")
  /// - applied 且是 Commit → 应用后回放缓冲(填补 epoch 缺口)
  /// - 其余 → 直接返回
  static Future<GroupInbound> processOrdered({
    required MlsWireMessage wire,
    required ChatEnvelope envelope,
    required GroupProcessSeam process,
    required GroupBufferPut bufferPut,
    required GroupBufferTake bufferTake,
    required WireFromEnvelope wireFromEnvelope,
  }) async {
    final result = await process(wire);
    if (result.status == GroupProcessStatus.outOfOrder) {
      await bufferPut(result.groupId, result.messageEpoch, envelope);
      return result;
    }
    if (result.status == GroupProcessStatus.applied &&
        result.kind == GroupInboundKind.commit) {
      await _drain(
        groupId: result.groupId,
        fromEpoch: result.groupEpoch,
        process: process,
        bufferPut: bufferPut,
        bufferTake: bufferTake,
        wireFromEnvelope: wireFromEnvelope,
      );
    }
    return result;
  }

  /// 回放缓冲:从 [fromEpoch] 起,取该 epoch 下缓冲项依次重放。
  /// application 应用后 epoch 不变(继续取同 epoch);Commit 应用后 epoch+1
  /// (移到下一 epoch 继续)。取空即止;每次取出即删除,必然收敛。
  static Future<void> _drain({
    required String groupId,
    required int fromEpoch,
    required GroupProcessSeam process,
    required GroupBufferPut bufferPut,
    required GroupBufferTake bufferTake,
    required WireFromEnvelope wireFromEnvelope,
  }) async {
    var current = fromEpoch;
    while (true) {
      final buffered = await bufferTake(groupId, current);
      if (buffered == null) {
        break;
      }
      final result = await process(wireFromEnvelope(buffered));
      switch (result.status) {
        case GroupProcessStatus.applied:
          // Commit 前进 epoch;application 不变。
          current = result.groupEpoch;
        case GroupProcessStatus.outOfOrder:
          // 理论上不会发生(取的就是当前 epoch);稳妥起见放回并停止,避免丢失。
          await bufferPut(result.groupId, result.messageEpoch, buffered);
          return;
        case GroupProcessStatus.stale:
        case GroupProcessStatus.unknown:
          // 丢弃,继续取同 epoch 下一条。
          break;
      }
    }
  }
}
