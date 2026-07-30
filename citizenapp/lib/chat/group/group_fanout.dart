// 单密文 → N 信封扇出(纯函数,与传输解耦,可测)。
//
// 群消息只加密一次（MLS 群 epoch 密钥），按名册对每个收件人 CID 封一个 envelope：
// 同一 `mls_wire_message`，不同 `recipient_cid_number`，服务端零存储不变。

import '../crypto/mls_session.dart';
import '../proto/chat_envelope.pb.dart';

/// 群扇出结果:每个收件人一个 envelope(密文相同)。
class GroupFanout {
  const GroupFanout._();

  /// 把一条群 wire message 扇成 N 个 envelope。
  ///
  /// [recipientCidNumbers] 必须已去重且已排除自己;为空时返回空列表(自言自语场景)。
  /// [messageId] 每条消息唯一,envelope_id = `<messageId>-<index>` 保证全局唯一。
  static List<ChatEnvelope> fanOut({
    required MlsWireMessage wire,
    required List<String> recipientCidNumbers,
    required String senderCidNumber,
    required String senderDeviceId,
    required String messageId,
    required int nowMillis,
    required int ttlMillis,
  }) {
    final envelopes = <ChatEnvelope>[];
    for (var index = 0; index < recipientCidNumbers.length; index++) {
      final recipient = recipientCidNumbers[index];
      envelopes.add(
        wire.toEnvelope(
          envelopeId: '$messageId-$index',
          senderCidNumber: senderCidNumber,
          recipientCidNumber: recipient,
          senderDeviceId: senderDeviceId,
          createdAtMillis: nowMillis,
          ttlMillis: ttlMillis,
        ),
      );
    }
    return envelopes;
  }
}
