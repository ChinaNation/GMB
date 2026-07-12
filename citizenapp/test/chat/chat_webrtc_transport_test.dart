import 'package:citizenapp/chat/transport/chat_webrtc_transport.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('附件起始帧只包含设备间传输字段', () {
    final frame = ChatWebrtcAttachmentFrame.start(
      conversationId: 'conv-1',
      attachmentId: 'attachment-1',
      fileName: 'photo.jpg',
      contentType: 'image/jpeg',
      byteSize: 4,
    );

    expect(frame['kind'], 'attachment_start');
    expect(frame.containsKey('object_key'), isFalse);
    expect(frame.containsKey('manifest'), isFalse);
    expect(ChatWebrtcAttachmentFrame.isComplete(frame, 4), isTrue);
    expect(ChatWebrtcAttachmentFrame.isComplete(frame, 3), isFalse);
  });
}
