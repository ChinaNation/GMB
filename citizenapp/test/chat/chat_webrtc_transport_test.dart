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
      accountId:
          '0x5555555555555555555555555555555555555555555555555555555555555555',
      cloud: ChatCloudTransport(
          accountId:
              '0x5555555555555555555555555555555555555555555555555555555555555555',
          localDeviceId: 'dev'),
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
      required senderAccountId,
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
      peerAccountId: 'peer',
      transferId: 't-reject',
      message: RTCDataChannelMessage(jsonEncode(_startHeader(byteSize: 100))),
      sendAck: ack,
    );
    await transport.handleIncomingFrame(
      buffer: buffer,
      peerAccountId: 'peer',
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
      required senderAccountId,
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
      peerAccountId: 'peer',
      transferId: 't-ok',
      message: RTCDataChannelMessage(jsonEncode(_startHeader(byteSize: 5))),
      sendAck: ack,
    );
    await transport.handleIncomingFrame(
      buffer: buffer,
      peerAccountId: 'peer',
      transferId: 't-ok',
      message: RTCDataChannelMessage.fromBinary(
        Uint8List.fromList(const [104, 101, 108, 108, 111]), // "hello"
      ),
      sendAck: ack,
    );
    await transport.handleIncomingFrame(
      buffer: buffer,
      peerAccountId: 'peer',
      transferId: 't-ok',
      message: RTCDataChannelMessage(jsonEncode({'kind': 'attachment_end'})),
      sendAck: ack,
    );

    expect(gotSize, 5);
    expect(await File(gotPath!).readAsString(), 'hello');
    expect(ackCalls, 1);
  });

  test('断点续传:dispose 保留 partial,新一轮从偏移续写拼出完整文件', () async {
    final root = await Directory.systemTemp.createTemp('gmb-resume-');
    addTearDown(() => root.delete(recursive: true));

    // 第一轮:传 2 字节后"断线"(dispose 只关流、保留 partial)。
    final first = ChatAttachmentReceiveBuffer(tempDirectory: root.path);
    await first.start(_startHeader(byteSize: 5), 't1');
    await first.addChunk(const [104, 101]); // "he"
    await first.dispose();
    final partPath = '${root.path}/attachment-1.part';
    expect(await File(partPath).exists(), isTrue);
    expect(await File(partPath).length(), 2);

    // 第二轮:同 attachment_id → 报偏移 2 → 只续 "llo",追加不覆盖。
    final second = ChatAttachmentReceiveBuffer(tempDirectory: root.path);
    await second.start(_startHeader(byteSize: 5), 't2');
    expect(second.resumeOffset, 2);
    await second.addChunk(const [108, 108, 111]); // "llo"
    final received = await second.finish();
    expect(received, isNotNull);
    expect(await File(received!.filePath).readAsString(), 'hello');
  });

  test('handleIncomingFrame:start 后回报续传偏移(有 partial=已存量)', () async {
    final root = await Directory.systemTemp.createTemp('gmb-frame-resume-');
    addTearDown(() => root.delete(recursive: true));
    final transport = _transport(
        root.path,
        ({
          required senderAccountId,
          required conversationId,
          required attachmentId,
          required fileName,
          required contentType,
          required filePath,
          required byteSize,
        }) async {});

    // 预置 3 字节的 partial(上次断线残留)。
    await File('${root.path}/attachment-1.part').writeAsBytes(const [1, 2, 3]);

    final buffer = ChatAttachmentReceiveBuffer(tempDirectory: root.path);
    int? reported;
    await transport.handleIncomingFrame(
      buffer: buffer,
      peerAccountId: 'peer',
      transferId: 't-resume',
      message: RTCDataChannelMessage(jsonEncode(_startHeader(byteSize: 10))),
      sendAck: () async {},
      sendResume: (offset) async => reported = offset,
    );
    expect(reported, 3);
  });

  test('断点续传:已全量落盘的补发(existing==declared)零新增分片仍交付', () async {
    final root = await Directory.systemTemp.createTemp('gmb-resume-full-');
    addTearDown(() => root.delete(recursive: true));
    // 上一轮已全量落盘但没收到 ack:partial 已是完整 "hello"。
    await File('${root.path}/attachment-1.part')
        .writeAsBytes(const [104, 101, 108, 108, 111]);

    final buffer = ChatAttachmentReceiveBuffer(tempDirectory: root.path);
    await buffer.start(_startHeader(byteSize: 5), 't-full');
    expect(buffer.resumeOffset, 5); // 报满偏移 → 发送端 openRead(5) 空流只发 end
    // 不 addChunk(发送端零字节补发)。
    final received = await buffer.finish();
    expect(received, isNotNull);
    expect(await File(received!.filePath).readAsString(), 'hello');
  });

  test('断点续传:陈旧超大 partial(existing>declared)清档从头重传', () async {
    final root = await Directory.systemTemp.createTemp('gmb-resume-stale-');
    addTearDown(() => root.delete(recursive: true));
    // 残留 20 字节的陈旧 partial,但本次声明只有 5 → 视为异常,清档从头。
    await File('${root.path}/attachment-1.part')
        .writeAsBytes(List<int>.filled(20, 1));

    final buffer = ChatAttachmentReceiveBuffer(tempDirectory: root.path);
    await buffer.start(_startHeader(byteSize: 5), 't-stale');
    expect(buffer.resumeOffset, 0); // 清档,偏移归零
    await buffer.addChunk(const [104, 101, 108, 108, 111]); // "hello"
    final received = await buffer.finish();
    expect(received, isNotNull);
    expect(await File(received!.filePath).readAsString(), 'hello');
  });

  test('sweepStalePartials:删超期 .part、留新鲜的,不碰非 .part', () async {
    final root = await Directory.systemTemp.createTemp('gmb-sweep-');
    addTearDown(() => root.delete(recursive: true));
    final stale = File('${root.path}/old.part');
    await stale.writeAsBytes(const [1]);
    await stale.setLastModified(
      DateTime.now().subtract(const Duration(days: 8)),
    );
    final fresh = File('${root.path}/new.part');
    await fresh.writeAsBytes(const [1]);
    final other = File('${root.path}/keep.bin');
    await other.writeAsBytes(const [1]);
    await other.setLastModified(
      DateTime.now().subtract(const Duration(days: 30)),
    );

    await ChatAttachmentReceiveBuffer.sweepStalePartials(root.path);

    expect(await stale.exists(), isFalse); // 超期 .part 删
    expect(await fresh.exists(), isTrue); // 新鲜 .part 留
    expect(await other.exists(), isTrue); // 非 .part 不碰(即便更旧)
  });
}
