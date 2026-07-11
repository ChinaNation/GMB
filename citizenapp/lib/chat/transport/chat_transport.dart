import '../chat_models.dart';

/// Chat 传输类型。
enum ChatTransportType {
  /// 互联网聊天，Cloudflare 只保存密文 mailbox 和必要投递元数据。
  cloudflare,

  /// 手机近场直连，不经过互联网 mailbox。
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

  /// 全局去重用 envelope ID。
  final String envelopeId;

  /// 实际使用的传输方式。
  final ChatTransportType transportType;

  /// 投递状态。
  final ChatMessageDeliveryState state;

  /// 失败原因，仅用于本机提示和日志。
  final String? errorMessage;
}

/// 待处理的密文 envelope。
class ChatPendingEncryptedEnvelope {
  const ChatPendingEncryptedEnvelope({
    required this.envelopeId,
    required this.envelopeBytes,
  });

  /// 全局去重用 envelope ID。
  final String envelopeId;

  /// 完整的 GMB_CHAT_V1 Protobuf envelope bytes。
  final List<int> envelopeBytes;
}

/// 待上传的附件密文分片描述。
class ChatAttachmentChunkDraft {
  const ChatAttachmentChunkDraft({
    required this.chunkId,
    required this.byteSize,
  });

  /// 分片在当前附件内的稳定编号。
  final String chunkId;

  /// 密文分片字节数。
  final int byteSize;
}

/// Worker 返回的附件上传目标。
class ChatAttachmentUploadTarget {
  const ChatAttachmentUploadTarget({
    required this.chunkId,
    required this.objectKey,
    required this.uploadUrl,
  });

  /// 分片在当前附件内的稳定编号。
  final String chunkId;

  /// R2 对象 key。该 key 只能指向密文分片。
  final String objectKey;

  /// Worker 签发的短期 PUT URL 或本地 dev-put 代理 URL。
  final Uri uploadUrl;
}

/// 附件密文 manifest 和分片上传计划。
class ChatAttachmentUploadPlan {
  const ChatAttachmentUploadPlan({
    required this.attachmentId,
    required this.manifestObjectKey,
    required this.manifestUploadUrl,
    required this.chunks,
  });

  /// 本次附件上传 ID。
  final String attachmentId;

  /// 加密 manifest 的 R2 object key。
  final String manifestObjectKey;

  /// 加密 manifest 的短期上传 URL。
  final Uri manifestUploadUrl;

  /// 加密分片上传目标。
  final List<ChatAttachmentUploadTarget> chunks;
}

/// 已上传密文附件的完成确认输入。
class ChatAttachmentCompleteRequest {
  const ChatAttachmentCompleteRequest({
    required this.attachmentId,
    required this.conversationId,
    required this.manifestObjectKey,
    required this.manifestHash,
    required this.chunkObjectKeys,
  });

  /// 本次附件上传 ID。
  final String attachmentId;

  /// 所属 Chat 会话 ID。
  final String conversationId;

  /// 加密 manifest 的 R2 object key。
  final String manifestObjectKey;

  /// 加密 manifest 的 sha256 hex，用于 envelope 引用和下载校验。
  final String manifestHash;

  /// 加密分片 object key 列表。
  final List<String> chunkObjectKeys;
}

/// 附件密文对象下载授权输入。
class ChatAttachmentDownloadRequest {
  const ChatAttachmentDownloadRequest({
    required this.attachmentId,
    required this.conversationId,
    required this.manifestObjectKey,
    required this.manifestHash,
    required this.chunkObjectKeys,
  });

  /// 本次附件 ID。
  final String attachmentId;

  /// 所属 Chat 会话 ID。
  final String conversationId;

  /// 加密 manifest 的 R2 object key。
  final String manifestObjectKey;

  /// 加密 manifest 的 sha256 hex。
  final String manifestHash;

  /// 加密分片 object key 列表。
  final List<String> chunkObjectKeys;
}

/// Worker 返回的附件下载目标。
class ChatAttachmentDownloadTarget {
  const ChatAttachmentDownloadTarget({
    required this.objectKey,
    required this.downloadUrl,
  });

  /// R2 对象 key。该 key 只能指向密文 manifest 或密文分片。
  final String objectKey;

  /// Worker 签发的短期 GET URL 或本地 dev-get 代理 URL。
  final Uri downloadUrl;
}

/// 附件密文 manifest 和分片下载计划。
class ChatAttachmentDownloadPlan {
  const ChatAttachmentDownloadPlan({
    required this.attachmentId,
    required this.manifestObjectKey,
    required this.manifestDownloadUrl,
    required this.chunks,
  });

  /// 本次附件 ID。
  final String attachmentId;

  /// 加密 manifest 的 R2 object key。
  final String manifestObjectKey;

  /// 加密 manifest 的短期下载 URL。
  final Uri manifestDownloadUrl;

  /// 加密分片下载目标。
  final List<ChatAttachmentDownloadTarget> chunks;
}

/// Chat 传输抽象。
///
/// Cloudflare mailbox、OpenMLS envelope 和近场能力分别接入这里；页面层只依赖
/// 这个接口，避免把投递服务、近场发现和 UI 状态耦合在一起。
abstract class ChatTransport {
  /// 当前传输类型。
  ChatTransportType get type;

  /// 发送已经加密后的 envelope bytes。
  Future<ChatDeliveryResult> sendEncryptedEnvelope({
    required String envelopeId,
    required List<int> envelopeBytes,
  });
}
