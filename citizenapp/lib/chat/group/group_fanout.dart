// 单密文 → N 信封扇出(纯函数,与传输解耦,可测)。
//
// 群消息只加密一次(MLS 群 epoch 密钥),按名册对每个收件人账户封一个 envelope:
// 同一 `mls_wire_message`,不同 `recipient_account_id`。服务端仍按 envelope 内
// `recipient_account_id` 路由,零存储不变。

import '../crypto/mls_session.dart';
import '../proto/chat_envelope.pb.dart';

/// 群扇出结果:每个收件人一个 envelope(密文相同)。
class GroupFanout {
  const GroupFanout._();

  /// 把一条群 wire message 扇成 N 个 envelope。
  ///
  /// [recipientAccountIds] 必须已去重且已排除自己;为空时返回空列表(自言自语场景)。
  /// [messageId] 每条消息唯一,envelope_id = `<messageId>-<index>` 保证全局唯一。
  static List<ChatEnvelope> fanOut({
    required MlsWireMessage wire,
    required List<String> recipientAccountIds,
    required String senderAccountId,
    required String senderDeviceId,
    required String messageId,
    required int nowMillis,
    required int ttlMillis,
  }) {
    final envelopes = <ChatEnvelope>[];
    for (var index = 0; index < recipientAccountIds.length; index++) {
      final recipient = recipientAccountIds[index];
      envelopes.add(
        wire.toEnvelope(
          envelopeId: '$messageId-$index',
          senderAccountId: senderAccountId,
          recipientAccountId: recipient,
          senderDeviceId: senderDeviceId,
          createdAtMillis: nowMillis,
          ttlMillis: ttlMillis,
        ),
      );
    }
    return envelopes;
  }
}
