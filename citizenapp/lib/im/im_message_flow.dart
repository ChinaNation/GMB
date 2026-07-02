import 'dart:convert';

import 'crypto/im_mls_boundary.dart';
import 'im_session_models.dart';
import 'proto/im_envelope.pb.dart';
import 'storage/im_isar_store.dart';
import 'transport/im_private_node_transport.dart';
import 'transport/im_transport.dart';

typedef ImEnvelopeDeliverer = Future<ImDeliveryResult> Function(
  ImEnvelope envelope,
  List<int> envelopeBytes,
);

/// IM 入站处理结果。
class ImIncomingProcessResult {
  const ImIncomingProcessResult({
    required this.envelopeId,
    required this.accepted,
    required this.queuedPending,
    this.plaintext,
  });

  final String envelopeId;
  final bool accepted;
  final bool queuedPending;
  final String? plaintext;
}

/// 公民 IM 消息收发状态机。
///
/// 本类是远程私人通信全节点链路的编排层。它不实现密码学，
/// 只负责把 OpenMLS native、GMB_IM_V1 envelope、本地 Isar 和投递接口串起来。
class ImMessageFlow {
  const ImMessageFlow({
    required ImMlsCryptoBoundary crypto,
    required ImIsarStore store,
    required ImEnvelopeDeliverer deliverer,
    this.defaultTtlMillis = 30 * 24 * 60 * 60 * 1000,
  })  : _crypto = crypto,
        _store = store,
        _deliverer = deliverer;

  final ImMlsCryptoBoundary _crypto;
  final ImIsarStore _store;
  final ImEnvelopeDeliverer _deliverer;
  final int defaultTtlMillis;

  Future<List<ImDeliveryResult>> sendText({
    required String conversationId,
    required String senderChatAccount,
    required String recipientChatAccount,
    required String senderDeviceId,
    ImMlsKeyPackage? recipientKeyPackage,
    required String text,
  }) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final outbound = await _crypto.encrypt(
      conversationId: conversationId,
      recipientChatAccount: recipientChatAccount,
      recipientKeyPackage: recipientKeyPackage,
      plaintext: utf8.encode(text),
    );

    final results = <ImDeliveryResult>[];
    var index = 0;
    for (final wireMessage in outbound.wireMessages) {
      final envelope = wireMessage.toEnvelope(
        envelopeId: _newEnvelopeId(conversationId, now, index),
        senderChatAccount: senderChatAccount,
        recipientChatAccount: recipientChatAccount,
        senderDeviceId: senderDeviceId,
        createdAtMillis: now + index,
        ttlMillis: defaultTtlMillis,
      );
      final envelopeBytes = envelope.writeToBuffer();
      final isApplication =
          wireMessage.messageKind == ImMlsMessageKind.application;
      if (isApplication) {
        await _store.saveOutgoingEnvelope(
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          messageKind: ImMessageKind.text,
          deliveryState: ImMessageDeliveryState.queued,
          plaintext: text,
        );
      } else {
        await _store.queueOutgoingEnvelope(
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          deliveryState: ImMessageDeliveryState.queued,
        );
      }

      final result = await _deliverer(envelope, envelopeBytes);
      await _store.markOutgoingDelivery(
        envelopeId: envelope.envelopeId,
        state: result.state,
        errorMessage: result.errorMessage,
      );
      results.add(result);
      index += 1;
    }
    return results;
  }

  Future<ImIncomingProcessResult> processIncomingEnvelopeBytes(
    List<int> envelopeBytes,
  ) async {
    final envelope = ImEnvelope.fromBuffer(envelopeBytes);
    final wireMessage = imMlsWireMessageFromEnvelope(envelope);
    try {
      final inbound = await _crypto.processIncoming(wireMessage);
      if (inbound.messageKind == ImMlsMessageKind.welcome) {
        final pending =
            await _store.takePendingInbound(envelope.conversationId);
        for (final item in pending) {
          await processIncomingEnvelopeBytes(item.writeToBuffer());
        }
        return ImIncomingProcessResult(
          envelopeId: envelope.envelopeId,
          accepted: true,
          queuedPending: false,
        );
      }

      final plaintext = utf8.decode(inbound.plaintext ?? const []);
      await _store.saveIncomingEnvelope(
        envelope: envelope,
        envelopeBytes: envelopeBytes,
        messageKind: ImMessageKind.text,
        plaintext: plaintext,
      );
      return ImIncomingProcessResult(
        envelopeId: envelope.envelopeId,
        accepted: true,
        queuedPending: false,
        plaintext: plaintext,
      );
    } catch (error) {
      if (wireMessage.messageKind == ImMlsMessageKind.application) {
        await _store.savePendingInbound(
          envelope: envelope,
          envelopeBytes: envelopeBytes,
          reason: error.toString(),
        );
        return ImIncomingProcessResult(
          envelopeId: envelope.envelopeId,
          accepted: false,
          queuedPending: true,
        );
      }
      rethrow;
    }
  }

  Future<int> fetchAndProcessPending({
    required Future<List<ImPrivateNodeEnvelopeDraft>> Function() fetchPending,
    required Future<void> Function(String envelopeId) ackEnvelope,
  }) async {
    final rows = await fetchPending();
    var processed = 0;
    for (final row in rows) {
      final result = await processIncomingEnvelopeBytes(row.encryptedPayload);
      if (result.accepted || result.queuedPending) {
        await ackEnvelope(row.envelopeId);
        processed += 1;
      }
    }
    return processed;
  }

  static Future<ImDeliveryResult> deliverWithPrivateNode({
    required ImPrivateNodeTransport transport,
    required ImPrivateNodeEndpoint remoteEndpoint,
    required ImEnvelope envelope,
  }) {
    return transport.submitDirectEnvelope(
      remoteEndpoint: remoteEndpoint,
      draft: ImPrivateNodeEnvelopeDraft.fromEnvelope(envelope),
    );
  }
}

String _newEnvelopeId(String conversationId, int millis, int index) {
  final normalized = conversationId.replaceAll(RegExp(r'[^a-zA-Z0-9_.-]'), '_');
  return '$normalized-$millis-$index';
}
