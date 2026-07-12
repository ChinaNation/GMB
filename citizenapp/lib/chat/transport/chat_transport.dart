import '../chat_models.dart';

/// Chat 传输类型。
enum ChatTransportType {
  /// 互联网聊天，Cloudflare 只做当前请求内的密文转发。
  cloudflare,

  /// 手机近场直连。
  nearby,
}

/// Chat 传输结果。
class ChatDeliveryResult {
  const ChatDeliveryResult({
    required this.envelopeId,
    required this.transportType,
    required this.state,
    this.errorMessage,
  });

  final String envelopeId;
  final ChatTransportType transportType;
  final ChatMessageDeliveryState state;
  final String? errorMessage;
}

/// 页面层只依赖加密 Envelope 传输，不接触Cloudflare路由细节。
abstract class ChatTransport {
  ChatTransportType get type;

  Future<ChatDeliveryResult> sendEncryptedEnvelope({
    required String envelopeId,
    required List<int> envelopeBytes,
  });
}
