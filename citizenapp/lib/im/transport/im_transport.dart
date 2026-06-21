import '../im_session_models.dart';

/// IM 传输类型。
enum ImTransportType {
  /// 手机连接自己的私人通信全节点，再由私人节点直连对方私人节点。
  privateNode,

  /// 手机近场直连，不经过通信全节点。
  nearby,
}

/// IM 传输结果。
class ImDeliveryResult {
  const ImDeliveryResult({
    required this.envelopeId,
    required this.transportType,
    required this.state,
    this.errorMessage,
  });

  /// 全局去重用 envelope ID。
  final String envelopeId;

  /// 实际使用的传输方式。
  final ImTransportType transportType;

  /// 投递状态。
  final ImMessageDeliveryState state;

  /// 失败原因，仅用于本机提示和日志。
  final String? errorMessage;
}

/// IM 传输抽象。
///
/// 真实 P2P、OpenMLS envelope 和近场能力后续分别接入这里；页面层只依赖
/// 这个接口，避免把节点投递、近场发现和 UI 状态耦合在一起。
abstract class ImTransport {
  /// 当前传输类型。
  ImTransportType get type;

  /// 发送已经加密后的 envelope bytes。
  Future<ImDeliveryResult> sendEncryptedEnvelope({
    required String envelopeId,
    required List<int> envelopeBytes,
  });
}
