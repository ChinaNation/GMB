import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_webrtc/flutter_webrtc.dart';

import '../chat_media_limits.dart';
import 'chat_cloud_transport.dart';

typedef ChatAttachmentReceiver = Future<void> Function({
  required String senderAccount,
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
  bool _rejected = false;

  bool get rejected => _rejected;
  int get running => _running;
  String? get tempPath => _tempPath;

  Future<void> start(Map<String, dynamic> header, String transferId) async {
    await _discard();
    _header = header;
    _running = 0;
    _rejected = false;
    final contentType =
        header['content_type']?.toString() ?? 'application/octet-stream';
    final declared = (header['byte_size'] as num?)?.toInt() ?? -1;
    _limit = _limitForMime(contentType);
    if (declared < 0 || declared > _limit) {
      _rejected = true;
      return;
    }
    final path = '$tempDirectory/${_safeSegment(transferId)}.part';
    final file = File(path);
    await file.parent.create(recursive: true);
    _tempPath = path;
    _sink = file.openWrite();
  }

  Future<void> addChunk(List<int> chunk) async {
    if (_rejected || _sink == null) return;
    _running += chunk.length;
    if (_running > _limit) {
      _rejected = true;
      await _discard();
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
      await _discard();
      return null;
    }
    await sink.flush();
    await sink.close();
    final declared = (header['byte_size'] as num?)?.toInt() ?? -1;
    if (_running != declared) {
      _tempPath = null;
      await _deleteTemp(tempPath);
      return null;
    }
    _tempPath = null;
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

  /// 关流并删除未完成的临时文件(拒收 / 复用前清理 / 通道断开时)。
  Future<void> _discard() async {
    final sink = _sink;
    _sink = null;
    if (sink != null) {
      try {
        await sink.close();
      } on FileSystemException {
        // 关流失败不阻断清理。
      }
    }
    final tempPath = _tempPath;
    _tempPath = null;
    if (tempPath != null) {
      await _deleteTemp(tempPath);
    }
  }

  Future<void> dispose() => _discard();

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
}

/// WebRTC DataChannel附件传输。SDP/ICE只经Worker瞬时转发，附件只落两端设备。
class ChatWebrtcTransport {
  ChatWebrtcTransport({
    required this.ownerAccount,
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

  final String ownerAccount;
  final ChatCloudTransport cloud;
  final ChatAttachmentReceiver onAttachment;

  /// 接收端字节流落盘的临时目录(App 私有,由运行态注入)。
  final String tempDirectory;

  final Map<String, _PeerTransfer> _peers = {};

  Future<void> sendAttachment({
    required String recipientAccount,
    required String conversationId,
    required String attachmentId,
    required String fileName,
    required String contentType,
    required String sourcePath,
    required int byteSize,
  }) async {
    final transferId = '$attachmentId-${DateTime.now().microsecondsSinceEpoch}';
    final peer = await _createPeer(transferId, recipientAccount);
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
    // 从源文件流式读取并分片发送:整文件绝不进内存;每片前按背压节流。
    await for (final block in File(sourcePath).openRead()) {
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
    await _closePeer(transferId);
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
      String senderAccount, Map<String, dynamic> signal) async {
    final kind = signal['kind']?.toString();
    final transferId = signal['transfer_id']?.toString() ?? '';
    if (transferId.isEmpty) return;
    if (kind == 'offer') {
      final peer = await _createPeer(transferId, senderAccount);
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
        recipientAccount: senderAccount,
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
      String transferId, String peerAccount) async {
    final existing = _peers[transferId];
    if (existing != null) return existing;
    final connection = await createPeerConnection({'iceServers': _iceServers});
    final peer = _PeerTransfer(transferId, peerAccount, connection);
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
        recipientAccount: peerAccount,
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
        recipientAccount: peer.peerAccount,
        signal: offer,
      );
      for (final candidate in peer.localCandidates) {
        await cloud.sendSignal(
          recipientAccount: peer.peerAccount,
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
                peerAccount: peer.peerAccount,
                transferId: peer.id,
                message: message,
                sendAck: () => channel.send(
                  RTCDataChannelMessage(jsonEncode({'kind': 'attachment_ack'})),
                ),
                onPeerAck: () {
                  if (!peer.ack.isCompleted) peer.ack.complete();
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
    required String peerAccount,
    required String transferId,
    required RTCDataChannelMessage message,
    required Future<void> Function() sendAck,
    void Function()? onPeerAck,
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
      case 'attachment_end':
        final received = await buffer.finish();
        if (received == null) return; // 拒收 / 截断:不回调、不 ack。
        await onAttachment(
          senderAccount: peerAccount,
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
    }
  }

  Future<void> _closePeer(String transferId) async {
    final peer = _peers.remove(transferId);
    await peer?.buffer?.dispose();
    await peer?.channel?.close();
    await peer?.connection.close();
  }

  Future<void> dispose() async {
    for (final id in _peers.keys.toList(growable: false)) {
      await _closePeer(id);
    }
  }
}

class _PeerTransfer {
  _PeerTransfer(this.id, this.peerAccount, this.connection);

  final String id;
  final String peerAccount;
  final RTCPeerConnection connection;
  final Completer<void> open = Completer<void>();
  final Completer<void> ack = Completer<void>();
  final List<Map<String, dynamic>> localCandidates = [];
  RTCDataChannel? channel;
  ChatAttachmentReceiveBuffer? buffer;

  /// 逐帧串行处理链。
  Future<void> tail = Future<void>.value();
}
