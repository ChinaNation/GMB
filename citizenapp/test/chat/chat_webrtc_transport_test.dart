import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:citizenapp/chat/transport/chat_cloud_transport.dart';
import 'package:citizenapp/chat/transport/chat_webrtc_transport.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_webrtc/flutter_webrtc.dart';

ChatWebrtcTransport _transport(
  String tempDir,
  ChatAttachmentReceiver onAttachment,
) =>
    ChatWebrtcTransport(
      ownerAccount: 'me',
      cloud: ChatCloudTransport(ownerAccount: 'me', ownerDeviceId: 'dev'),
      tempDirectory: tempDir,
      onAttachment: onAttachment,
    );

Map<String, dynamic> _startHeader({
  required int byteSize,
  String contentType = 'text/plain',
}) =>
    ChatWebrtcAttachmentFrame.start(
      conversationId: 'conv-1',
      attachmentId: 'attachment-1',
      fileName: 'note.txt',
      contentType: contentType,
      byteSize: byteSize,
    );

void main() {
  test('附件起始帧只包含设备间传输字段', () {
    final frame = _startHeader(byteSize: 4);
    expect(frame['kind'], 'attachment_start');
    expect(frame.containsKey('object_key'), isFalse);
    expect(frame.containsKey('manifest'), isFalse);
  });

  test('接收缓冲流式落盘:完整传输落到临时文件并返回句柄', () async {
    final root = await Directory.systemTemp.createTemp('gmb-recv-ok-');
    addTearDown(() => root.delete(recursive: true));
    final buffer = ChatAttachmentReceiveBuffer(tempDirectory: root.path);

    await buffer.start(_startHeader(byteSize: 5), 'transfer-ok');
    await buffer.addChunk(const [104, 101]); // "he"
    await buffer.addChunk(const [108, 108, 111]); // "llo"
    final received = await buffer.finish();

    expect(received, isNotNull);
    expect(received!.byteSize, 5);
    expect(await File(received.filePath).readAsString(), 'hello');
  });

  test('门②:声明大小超上限直接拒收,连临时文件都不建', () async {
    final root = await Directory.systemTemp.createTemp('gmb-recv-declared-');
    addTearDown(() => root.delete(recursive: true));
    // 注入 8 字节的小额度,模拟"声明就超限"。
    final buffer = ChatAttachmentReceiveBuffer(
      tempDirectory: root.path,
      limitForMime: (_) => 8,
    );

    await buffer.start(_startHeader(byteSize: 100), 'transfer-declared');
    expect(buffer.rejected, isTrue);
    expect(buffer.tempPath, isNull);
    await buffer.addChunk(List<int>.filled(100, 1)); // 后续字节全丢
    final received = await buffer.finish();
    expect(received, isNull);
    // 临时目录里不应留下任何 .part 文件。
    final leftovers = root
        .listSync()
        .whereType<File>()
        .where((f) => f.path.endsWith('.part'));
    expect(leftovers, isEmpty);
  });

  test('门②:谎报小 byte_size 却狂发,累积超上限时中止并删临时文件', () async {
    final root = await Directory.systemTemp.createTemp('gmb-recv-flood-');
    addTearDown(() => root.delete(recursive: true));
    // 额度 8 字节;声明 6(通过 start),但实际累计发 12 → 超额度中止。
    final buffer = ChatAttachmentReceiveBuffer(
      tempDirectory: root.path,
      limitForMime: (_) => 8,
    );

    await buffer.start(_startHeader(byteSize: 6), 'transfer-flood');
    expect(buffer.rejected, isFalse);
    final tempPath = buffer.tempPath;
    expect(tempPath, isNotNull);

    await buffer.addChunk(List<int>.filled(6, 1));
    await buffer.addChunk(List<int>.filled(6, 1)); // 累计 12 > 8 → 中止
    expect(buffer.rejected, isTrue);
    final received = await buffer.finish();
    expect(received, isNull);
    expect(await File(tempPath!).exists(), isFalse);
  });

  test('门②:收到字节数与声明不符(截断/损坏)则丢弃不交付', () async {
    final root = await Directory.systemTemp.createTemp('gmb-recv-trunc-');
    addTearDown(() => root.delete(recursive: true));
    final buffer = ChatAttachmentReceiveBuffer(tempDirectory: root.path);

    await buffer.start(_startHeader(byteSize: 10), 'transfer-trunc');
    final tempPath = buffer.tempPath;
    await buffer.addChunk(const [1, 2, 3]); // 只有 3 字节,声明 10
    final received = await buffer.finish();
    expect(received, isNull);
    expect(await File(tempPath!).exists(), isFalse);
  });

  test('handleIncomingFrame:被拒收的传输既不回调 onAttachment 也不 ack', () async {
    final root = await Directory.systemTemp.createTemp('gmb-frame-reject-');
    addTearDown(() => root.delete(recursive: true));
    var onAttachmentCalls = 0;
    var ackCalls = 0;
    final transport = _transport(root.path, ({
      required senderAccount,
      required conversationId,
      required attachmentId,
      required fileName,
      required contentType,
      required filePath,
      required byteSize,
    }) async {
      onAttachmentCalls += 1;
    });
    // 注入 8 字节额度;声明 100 → start 拒收。篡改的发送方绝不能收到 ack。
    final buffer = ChatAttachmentReceiveBuffer(
      tempDirectory: root.path,
      limitForMime: (_) => 8,
    );
    Future<void> ack() async => ackCalls += 1;

    await transport.handleIncomingFrame(
      buffer: buffer,
      peerAccount: 'peer',
      transferId: 't-reject',
      message: RTCDataChannelMessage(jsonEncode(_startHeader(byteSize: 100))),
      sendAck: ack,
    );
    await transport.handleIncomingFrame(
      buffer: buffer,
      peerAccount: 'peer',
      transferId: 't-reject',
      message: RTCDataChannelMessage(jsonEncode({'kind': 'attachment_end'})),
      sendAck: ack,
    );

    expect(onAttachmentCalls, 0);
    expect(ackCalls, 0);
  });

  test('handleIncomingFrame:完整传输回调 onAttachment 一次并 ack', () async {
    final root = await Directory.systemTemp.createTemp('gmb-frame-ok-');
    addTearDown(() => root.delete(recursive: true));
    String? gotPath;
    int? gotSize;
    var ackCalls = 0;
    final transport = _transport(root.path, ({
      required senderAccount,
      required conversationId,
      required attachmentId,
      required fileName,
      required contentType,
      required filePath,
      required byteSize,
    }) async {
      gotPath = filePath;
      gotSize = byteSize;
    });
    final buffer = ChatAttachmentReceiveBuffer(tempDirectory: root.path);
    Future<void> ack() async => ackCalls += 1;

    await transport.handleIncomingFrame(
      buffer: buffer,
      peerAccount: 'peer',
      transferId: 't-ok',
      message: RTCDataChannelMessage(jsonEncode(_startHeader(byteSize: 5))),
      sendAck: ack,
    );
    await transport.handleIncomingFrame(
      buffer: buffer,
      peerAccount: 'peer',
      transferId: 't-ok',
      message: RTCDataChannelMessage.fromBinary(
        Uint8List.fromList(const [104, 101, 108, 108, 111]), // "hello"
      ),
      sendAck: ack,
    );
    await transport.handleIncomingFrame(
      buffer: buffer,
      peerAccount: 'peer',
      transferId: 't-ok',
      message: RTCDataChannelMessage(jsonEncode({'kind': 'attachment_end'})),
      sendAck: ack,
    );

    expect(gotSize, 5);
    expect(await File(gotPath!).readAsString(), 'hello');
    expect(ackCalls, 1);
  });
}
