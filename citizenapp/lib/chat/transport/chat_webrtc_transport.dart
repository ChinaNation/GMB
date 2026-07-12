import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_webrtc/flutter_webrtc.dart';

import 'chat_cloud_transport.dart';

typedef ChatAttachmentReceiver = Future<void> Function({
  required String senderAccount,
  required String conversationId,
  required String attachmentId,
  required String fileName,
  required String contentType,
  required List<int> bytes,
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

  static bool isComplete(Map<String, dynamic>? header, int receivedBytes) {
    return header != null &&
        receivedBytes == (header['byte_size'] as num?)?.toInt();
  }
}

/// WebRTC DataChannel附件传输。SDP/ICE只经Worker瞬时转发，附件只落两端设备。
class ChatWebrtcTransport {
  ChatWebrtcTransport({
    required this.ownerAccount,
    required this.cloud,
    required this.onAttachment,
  });

  static const _chunkSize = 16 * 1024;
  static const _timeout = Duration(seconds: 45);
  // 只使用 STUN 发现公网候选；不配置中继 URL、用户名或凭证，附件因此绝不会
  // 经云端中继。直连失败时保留在发送设备，等待接收方网络条件允许后重试。
  static const _iceServers = <Map<String, Object>>[
    <String, Object>{'urls': <String>['stun:stun.cloudflare.com:3478']},
  ];

  final String ownerAccount;
  final ChatCloudTransport cloud;
  final ChatAttachmentReceiver onAttachment;
  final Map<String, _PeerTransfer> _peers = {};

  Future<void> sendAttachment({
    required String recipientAccount,
    required String conversationId,
    required String attachmentId,
    required String fileName,
    required String contentType,
    required List<int> bytes,
  }) async {
    final transferId = '$attachmentId-${DateTime.now().microsecondsSinceEpoch}';
    final peer = await _createPeer(transferId, recipientAccount);
    final channel = await peer.connection.createDataChannel(
      'chat-attachment',
      RTCDataChannelInit()..ordered = true,
    );
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
    channel.send(RTCDataChannelMessage(jsonEncode(
      ChatWebrtcAttachmentFrame.start(
        conversationId: conversationId,
        attachmentId: attachmentId,
        fileName: fileName,
        contentType: contentType,
        byteSize: bytes.length,
      ),
    )));
    for (var offset = 0; offset < bytes.length; offset += _chunkSize) {
      final end = (offset + _chunkSize).clamp(0, bytes.length);
      channel.send(RTCDataChannelMessage.fromBinary(
          Uint8List.fromList(bytes.sublist(offset, end))));
    }
    channel.send(RTCDataChannelMessage(jsonEncode({'kind': 'attachment_end'})));
    await peer.ack.future.timeout(_timeout);
    await _closePeer(transferId);
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
    channel.onMessage = (message) async {
      if (message.isBinary) {
        peer.received.addAll(message.binary);
        return;
      }
      final decoded = jsonDecode(message.text);
      if (decoded is! Map<String, dynamic>) return;
      if (decoded['kind'] == 'attachment_start') {
        peer.header = decoded;
        peer.received.clear();
      } else if (decoded['kind'] == 'attachment_end') {
        final header = peer.header;
        if (!ChatWebrtcAttachmentFrame.isComplete(
          header,
          peer.received.length,
        )) {
          return;
        }
        final completeHeader = header!;
        await onAttachment(
          senderAccount: peer.peerAccount,
          conversationId: completeHeader['conversation_id']?.toString() ?? '',
          attachmentId: completeHeader['attachment_id']?.toString() ?? '',
          fileName: completeHeader['file_name']?.toString() ?? 'attachment.bin',
          contentType: completeHeader['content_type']?.toString() ??
              'application/octet-stream',
          bytes: List<int>.from(peer.received),
        );
        channel.send(
            RTCDataChannelMessage(jsonEncode({'kind': 'attachment_ack'})));
      } else if (decoded['kind'] == 'attachment_ack' && !peer.ack.isCompleted) {
        peer.ack.complete();
      }
    };
  }

  Future<void> _closePeer(String transferId) async {
    final peer = _peers.remove(transferId);
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
  final List<int> received = [];
  final List<Map<String, dynamic>> localCandidates = [];
  RTCDataChannel? channel;
  Map<String, dynamic>? header;
}
