import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_webrtc/flutter_webrtc.dart';

import '../chat_media_limits.dart';
import 'chat_cloud_transport.dart';

typedef ChatAttachmentReceiver = Future<void> Function({
  required String senderAccountId,
  required String conversationId,
  required String attachmentId,
  required String fileName,
  required String contentType,
  required String filePath,
  required int byteSize,
});

/// DataChannel 附件帧只描述设备间传输，不允许出现云端对象引用。
class ChatWebrtcAttachmentFrame {
  const ChatWebrtcAttachmentFrame._();

  static Map<String, dynamic> start({
    required String conversationId,
    required String attachmentId,
    required String fileName,
    required String contentType,
    required int byteSize,
  }) =>
      {
        'kind': 'attachment_start',
        'conversation_id': conversationId,
        'attachment_id': attachmentId,
        'file_name': fileName,
        'content_type': contentType,
        'byte_size': byteSize,
      };

  /// 接收端回给发送端的续传偏移:本地同 attachment_id 的 `.part` 已存字节数。
  /// 发送端据此 `openRead(offset)` 只补缺口,不从头重传。
  static Map<String, dynamic> resume({required int resumeOffset}) => {
        'kind': 'attachment_resume',
        'resume_offset': resumeOffset,
      };
}

/// 接收端已完整落盘的媒体临时文件句柄。
class ChatReceivedAttachment {
  const ChatReceivedAttachment({
    required this.conversationId,
    required this.attachmentId,
    required this.fileName,
    required this.contentType,
    required this.filePath,
    required this.byteSize,
  });

  final String conversationId;
  final String attachmentId;
  final String fileName;
  final String contentType;
  final String filePath;
  final int byteSize;
}

/// 接收端媒体**流式落盘 + 大小门控(门②)**。
///
/// 把 WebRTC 分片直写临时文件,只维护运行字节计数做门控,内存里不堆整文件——
/// 5GB 媒体也不会 OOM。门控用 `content_type` 定额:
///   - `attachment_start` 声明就超限(或缺失)→ 拒收,连临时文件都不建;
///   - 累积字节超限 → 立即中止 + 删临时(防发送方谎报小 byte_size 却狂发);
///   - `attachment_end` 时字节数须与声明**精确一致**,否则视为截断/损坏丢弃。
/// 与 WebRTC 解耦以便单测。
class ChatAttachmentReceiveBuffer {
  ChatAttachmentReceiveBuffer({
    required this.tempDirectory,
    int Function(String mime)? limitForMime,
  }) : _limitForMime = limitForMime ?? ChatMediaLimits.forMime;

  final String tempDirectory;

  /// 按 mime 取上限。默认走单源 [ChatMediaLimits.forMime];测试可注入小额度以
  /// 驱动累积超限中止,无需真的流 100MB。
  final int Function(String mime) _limitForMime;

  IOSink? _sink;
  String? _tempPath;
  Map<String, dynamic>? _header;
  int _running = 0;
  int _limit = 0;
  int _resumeOffset = 0;
  bool _rejected = false;

  bool get rejected => _rejected;
  int get running => _running;

  /// 本次 start 时同 attachment_id 的 `.part` 已存字节数;发送端据此续流。
  int get resumeOffset => _resumeOffset;
  String? get tempPath => _tempPath;

  Future<void> start(Map<String, dynamic> header, String transferId) async {
    await _closeSink(); // 关旧 sink,但保留 partial(可能正是要续传的)
    _header = header;
    _running = 0;
    _resumeOffset = 0;
    _rejected = false;
    final contentType =
        header['content_type']?.toString() ?? 'application/octet-stream';
    final declared = (header['byte_size'] as num?)?.toInt() ?? -1;
    _limit = _limitForMime(contentType);
    if (declared < 0 || declared > _limit) {
      _rejected = true;
      return;
    }
    // 按 attachment_id 命名 partial,同一媒体跨传输尝试可复用以断点续传。
    final attachmentId = header['attachment_id']?.toString() ?? transferId;
    final path = '$tempDirectory/${_safeSegment(attachmentId)}.part';
    final file = File(path);
    await file.parent.create(recursive: true);
    var existing = 0;
    if (await file.exists()) {
      existing = await file.length();
      // 现有 partial 比声明还大 = 陈旧/异常,清掉从头来。
      if (existing > declared) {
        await _deleteTemp(path);
        existing = 0;
      }
    }
    _tempPath = path;
    _resumeOffset = existing;
    _running = existing;
    // 追加写:已存字节保留,只续写缺口。
    _sink = file.openWrite(mode: FileMode.writeOnlyAppend);
  }

  Future<void> addChunk(List<int> chunk) async {
    if (_rejected || _sink == null) return;
    _running += chunk.length;
    if (_running > _limit) {
      _rejected = true;
      await _deletePartial(); // 谎报小 byte_size 却狂发:中止并删档
      return;
    }
    _sink!.add(chunk);
  }

  Future<ChatReceivedAttachment?> finish() async {
    final sink = _sink;
    final header = _header;
    final tempPath = _tempPath;
    _sink = null;
    if (_rejected || sink == null || header == null || tempPath == null) {
      await _deletePartial();
      return null;
    }
    await sink.flush();
    await sink.close();
    final declared = (header['byte_size'] as num?)?.toInt() ?? -1;
    if (_running != declared) {
      // 大小不符 = 截断/损坏:删档,下次同 attachment_id 从头传。
      _tempPath = null;
      await _deleteTemp(tempPath);
      return null;
    }
    _tempPath = null; // 完整:交调用方移入缓存,dispose 不再触碰
    return ChatReceivedAttachment(
      conversationId: header['conversation_id']?.toString() ?? '',
      attachmentId: header['attachment_id']?.toString() ?? '',
      fileName: header['file_name']?.toString() ?? 'attachment.bin',
      contentType:
          header['content_type']?.toString() ?? 'application/octet-stream',
      filePath: tempPath,
      byteSize: _running,
    );
  }

  /// 关流但**保留** partial(断线/复用前):下次同 attachment_id 续写,断点续传核心。
  Future<void> _closeSink() async {
    final sink = _sink;
    _sink = null;
    if (sink != null) {
      try {
        await sink.flush();
        await sink.close();
      } on FileSystemException {
        // 关流失败不阻断续传:磁盘已存字节仍是有效前缀。
      }
    }
  }

  /// 主动作废:关流并删 partial(拒收 / 累积超限 / 大小不符),下次从头。
  Future<void> _deletePartial() async {
    await _closeSink();
    final tempPath = _tempPath;
    _tempPath = null;
    if (tempPath != null) {
      await _deleteTemp(tempPath);
    }
  }

  /// 通道断开/接收端释放:只关流保留 partial,已存字节等下次续传。
  Future<void> dispose() => _closeSink();

  static Future<void> _deleteTemp(String path) async {
    final file = File(path);
    if (await file.exists()) {
      try {
        await file.delete();
      } on FileSystemException {
        // 临时文件删除失败可忽略,由缓存清理兜底。
      }
    }
  }

  static String _safeSegment(String value) =>
      value.replaceAll(RegExp(r'[^a-zA-Z0-9_.-]'), '_');

  /// 清理被永久放弃的续传残档:删 [tempDirectory] 下 mtime 超 [maxAge] 的 `.part`。
  /// 对端删了会话/待投递行后,其半程 partial 不会再被续写,由此回收磁盘。
  static Future<void> sweepStalePartials(
    String tempDirectory, {
    Duration maxAge = const Duration(days: 7),
  }) async {
    final dir = Directory(tempDirectory);
    if (!await dir.exists()) return;
    final cutoff = maxAge.inMilliseconds;
    await for (final entity in dir.list()) {
      if (entity is! File || !entity.path.endsWith('.part')) continue;
      try {
        final age = DateTime.now().difference(await entity.lastModified());
        if (age.inMilliseconds > cutoff) {
          await entity.delete();
        }
      } on FileSystemException {
        // 单个残档处理失败不阻断整体清扫。
      }
    }
  }
}

/// WebRTC DataChannel附件传输。SDP/ICE只经Worker瞬时转发，附件只落两端设备。
class ChatWebrtcTransport {
  ChatWebrtcTransport({
    required this.accountId,
    required this.cloud,
    required this.onAttachment,
    required this.tempDirectory,
  });

  static const _chunkSize = 64 * 1024;
  static const _timeout = Duration(seconds: 45);
  // 背压水位:发送缓冲超过高水位则暂停灌注,等其回落到低水位再继续。5GB 文件
  // 因此不会把 SCTP 发送缓冲撑爆。
  static const _highWaterBytes = 1 * 1024 * 1024;
  static const _lowWaterBytes = 256 * 1024;
  // 只使用 STUN 发现公网候选；不配置中继 URL、用户名或凭证，附件因此绝不会
  // 经云端中继。直连失败时保留在发送设备，等待接收方网络条件允许后重试。
  static const _iceServers = <Map<String, Object>>[
    <String, Object>{
      'urls': <String>['stun:stun.cloudflare.com:3478'],
    },
  ];

  final String accountId;
  final ChatCloudTransport cloud;
  final ChatAttachmentReceiver onAttachment;

  /// 接收端字节流落盘的临时目录(App 私有,由运行态注入)。
  final String tempDirectory;

  final Map<String, _PeerTransfer> _peers = {};

  Future<void> sendAttachment({
    required String recipientAccountId,
    required String conversationId,
    required String attachmentId,
    required String fileName,
    required String contentType,
    required String sourcePath,
    required int byteSize,
  }) async {
    final transferId = '$attachmentId-${DateTime.now().microsecondsSinceEpoch}';
    final peer = await _createPeer(transferId, recipientAccountId);
    // 任一 await(建连/续传偏移/ack 超时,或发送出错)都必须关闭对端连接,否则
    // 泄漏 _peers 表项与原生 RTCPeerConnection。_closePeer 幂等,成功路径也复用。
    try {
      final channel = await peer.connection.createDataChannel(
        'chat-attachment',
        RTCDataChannelInit()..ordered = true,
      );
      channel.bufferedAmountLowThreshold = _lowWaterBytes;
      peer.channel = channel;
      _bindChannel(peer, channel);
      final offer = await peer.connection.createOffer();
      await peer.connection.setLocalDescription(offer);
      await _sendOfferUntilOpen(
        peer,
        {
          'kind': 'offer',
          'transfer_id': transferId,
          'sdp': offer.sdp,
          'sdp_type': offer.type,
        },
      );
      await channel.send(RTCDataChannelMessage(jsonEncode(
        ChatWebrtcAttachmentFrame.start(
          conversationId: conversationId,
          attachmentId: attachmentId,
          fileName: fileName,
          contentType: contentType,
          byteSize: byteSize,
        ),
      )));
      // 等接收端回报续传偏移(其同 attachment_id 的 .part 已存字节数),据此续流;
      // 超时(对端离线/声明超限拒收未回帧)则抛错,partial 留待下次 peer_ready 续传。
      final reportedOffset = await peer.resumeOffset.future.timeout(_timeout);
      // 对端可被篡改:合法偏移恒在 [0, byteSize],越界即从 0 全量重传。负值若直传
      // openRead 会抛 RangeError(是 Error 非 Exception),逃逸上层 on Exception 门控。
      final resumeOffset = (reportedOffset < 0 || reportedOffset > byteSize)
          ? 0
          : reportedOffset;
      // 从续传偏移起流式读取分片:整文件绝不进内存;每片前按背压节流。已传的
      // [0, resumeOffset) 不再重发,断点续传的核心。
      await for (final block in File(sourcePath).openRead(resumeOffset)) {
        final bytes = block is Uint8List ? block : Uint8List.fromList(block);
        for (var offset = 0; offset < bytes.length; offset += _chunkSize) {
          final end = (offset + _chunkSize).clamp(0, bytes.length);
          await _drainIfNeeded(channel);
          await channel.send(
            RTCDataChannelMessage.fromBinary(
              Uint8List.sublistView(bytes, offset, end),
            ),
          );
        }
      }
      await channel
          .send(RTCDataChannelMessage(jsonEncode({'kind': 'attachment_end'})));
      await peer.ack.future.timeout(_timeout);
    } finally {
      await _closePeer(transferId);
    }
  }

  /// 发送背压:发送缓冲超过高水位时轮询等待其回落到低水位以下再继续灌注。
  Future<void> _drainIfNeeded(RTCDataChannel channel) async {
    var buffered = channel.bufferedAmount ?? 0;
    while (buffered > _highWaterBytes) {
      await Future<void>.delayed(const Duration(milliseconds: 50));
      buffered = await channel.getBufferedAmount();
    }
  }

  Future<void> handleSignal(
      String senderAccountId, Map<String, dynamic> signal) async {
    final kind = signal['kind']?.toString();
    final transferId = signal['transfer_id']?.toString() ?? '';
    if (transferId.isEmpty) return;
    if (kind == 'offer') {
      final peer = await _createPeer(transferId, senderAccountId);
      peer.connection.onDataChannel = (channel) {
        peer.channel = channel;
        _bindChannel(peer, channel);
      };
      await peer.connection.setRemoteDescription(
        RTCSessionDescription(
            signal['sdp']?.toString(), signal['sdp_type']?.toString()),
      );
      final answer = await peer.connection.createAnswer();
      await peer.connection.setLocalDescription(answer);
      await cloud.sendSignal(
        recipientAccountId: senderAccountId,
        signal: {
          'kind': 'answer',
          'transfer_id': transferId,
          'sdp': answer.sdp,
          'sdp_type': answer.type,
        },
      );
      return;
    }
    final peer = _peers[transferId];
    if (peer == null) return;
    if (kind == 'answer') {
      await peer.connection.setRemoteDescription(
        RTCSessionDescription(
            signal['sdp']?.toString(), signal['sdp_type']?.toString()),
      );
    } else if (kind == 'ice') {
      await peer.connection.addCandidate(RTCIceCandidate(
        signal['candidate']?.toString(),
        signal['sdp_mid']?.toString(),
        (signal['sdp_mline_index'] as num?)?.toInt(),
      ));
    }
  }

  Future<_PeerTransfer> _createPeer(
      String transferId, String peerAccountId) async {
    final existing = _peers[transferId];
    if (existing != null) return existing;
    final connection = await createPeerConnection({'iceServers': _iceServers});
    final peer = _PeerTransfer(transferId, peerAccountId, connection);
    _peers[transferId] = peer;
    connection.onIceCandidate = (candidate) {
      if (candidate.candidate == null) return;
      final signal = <String, dynamic>{
        'kind': 'ice',
        'transfer_id': transferId,
        'candidate': candidate.candidate,
        'sdp_mid': candidate.sdpMid,
        'sdp_mline_index': candidate.sdpMLineIndex,
      };
      peer.localCandidates.add(signal);
      unawaited(cloud.sendSignal(
        recipientAccountId: peerAccountId,
        signal: signal,
      ));
    };
    return peer;
  }

  /// 第一次 offer 会触发无内容推送；接收方启动后，后续 offer 和 ICE 仍只瞬时转发。
  Future<void> _sendOfferUntilOpen(
    _PeerTransfer peer,
    Map<String, dynamic> offer,
  ) async {
    final deadline = DateTime.now().add(_timeout);
    while (!peer.open.isCompleted && DateTime.now().isBefore(deadline)) {
      await cloud.sendSignal(
        recipientAccountId: peer.peerAccountId,
        signal: offer,
      );
      for (final candidate in peer.localCandidates) {
        await cloud.sendSignal(
          recipientAccountId: peer.peerAccountId,
          signal: candidate,
        );
      }
      if (peer.open.isCompleted) break;
      await Future.any<void>([
        peer.open.future,
        Future<void>.delayed(const Duration(seconds: 8)),
      ]);
    }
    if (!peer.open.isCompleted) {
      await _closePeer(peer.id);
      throw TimeoutException('接收设备未连接，附件仍只保留在发送设备');
    }
  }

  void _bindChannel(_PeerTransfer peer, RTCDataChannel channel) {
    channel.onDataChannelState = (state) {
      if (state == RTCDataChannelState.RTCDataChannelOpen &&
          !peer.open.isCompleted) {
        peer.open.complete();
      } else if (state == RTCDataChannelState.RTCDataChannelClosed) {
        // 通道关闭(含中途断线):收口该 peer——释放打开的 append sink 与原生连接,
        // 但 dispose 只关流保留 partial。否则同会话对同一 attachment_id 的补发会
        // 在同一 .part 上再开一个 sink(两 sink 同 inode → partial 损坏/泄漏)。
        unawaited(_closePeer(peer.id));
      }
    };
    final buffer = ChatAttachmentReceiveBuffer(tempDirectory: tempDirectory);
    peer.buffer = buffer;
    // 逐帧串行处理:磁盘 I/O 是异步的,必须保证 start 建好 sink 后分片才写入、
    // 且分片按序落盘,否则会丢首片或乱序。
    channel.onMessage = (message) {
      peer.tail = peer.tail
          .then((_) => handleIncomingFrame(
                buffer: buffer,
                peerAccountId: peer.peerAccountId,
                transferId: peer.id,
                message: message,
                sendAck: () => channel.send(
                  RTCDataChannelMessage(jsonEncode({'kind': 'attachment_ack'})),
                ),
                sendResume: (offset) => channel.send(
                  RTCDataChannelMessage(jsonEncode(
                    ChatWebrtcAttachmentFrame.resume(resumeOffset: offset),
                  )),
                ),
                onPeerAck: () {
                  if (!peer.ack.isCompleted) peer.ack.complete();
                },
                onResumeOffset: (offset) {
                  if (!peer.resumeOffset.isCompleted) {
                    peer.resumeOffset.complete(offset);
                  }
                },
              ))
          .catchError((Object _) {});
    };
  }

  /// 处理一帧接收数据(可单测,不依赖真实 DataChannel):二进制→落盘;start/end→
  /// 门控与回调。拒收 / 截断时**既不回调也不 ack**——篡改的发送方不会收到 ack
  /// 误以为超限媒体被接受。
  Future<void> handleIncomingFrame({
    required ChatAttachmentReceiveBuffer buffer,
    required String peerAccountId,
    required String transferId,
    required RTCDataChannelMessage message,
    required Future<void> Function() sendAck,
    Future<void> Function(int resumeOffset)? sendResume,
    void Function()? onPeerAck,
    void Function(int resumeOffset)? onResumeOffset,
  }) async {
    if (message.isBinary) {
      await buffer.addChunk(message.binary);
      return;
    }
    final decoded = jsonDecode(message.text);
    if (decoded is! Map<String, dynamic>) return;
    switch (decoded['kind']) {
      case 'attachment_start':
        await buffer.start(decoded, transferId);
        // 拒收(声明超限)则不回续传帧:发送端等待超时后中止,不建 partial。
        if (!buffer.rejected && sendResume != null) {
          await sendResume(buffer.resumeOffset);
        }
      case 'attachment_end':
        final received = await buffer.finish();
        if (received == null) return; // 拒收 / 截断:不回调、不 ack。
        await onAttachment(
          senderAccountId: peerAccountId,
          conversationId: received.conversationId,
          attachmentId: received.attachmentId,
          fileName: received.fileName,
          contentType: received.contentType,
          filePath: received.filePath,
          byteSize: received.byteSize,
        );
        await sendAck();
      case 'attachment_ack':
        onPeerAck?.call();
      case 'attachment_resume':
        onResumeOffset?.call((decoded['resume_offset'] as num?)?.toInt() ?? 0);
    }
  }

  Future<void> _closePeer(String transferId) async {
    final peer = _peers.remove(transferId);
    if (peer == null) return;
    // 先等串行帧链跑完:让已入队的 finish() 按截断/成功语义先收尾,再关流。否则
    // dispose 先置 _sink=null,随后 finish 读到 null 会误走删档、丢掉本要保留的
    // partial 甚至作废一次已收全的投递。
    await peer.tail.catchError((Object _) {});
    await peer.buffer?.dispose();
    await peer.channel?.close();
    await peer.connection.close();
  }

  Future<void> dispose() async {
    for (final id in _peers.keys.toList(growable: false)) {
      await _closePeer(id);
    }
  }
}

class _PeerTransfer {
  _PeerTransfer(this.id, this.peerAccountId, this.connection);

  final String id;
  final String peerAccountId;
  final RTCPeerConnection connection;
  final Completer<void> open = Completer<void>();
  final Completer<void> ack = Completer<void>();
  // 发送端等接收端回报的续传偏移,拿到才从该偏移起流。
  final Completer<int> resumeOffset = Completer<int>();
  final List<Map<String, dynamic>> localCandidates = [];
  RTCDataChannel? channel;
  ChatAttachmentReceiveBuffer? buffer;

  /// 逐帧串行处理链。
  Future<void> tail = Future<void>.value();
}
