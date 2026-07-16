import 'package:flutter_test/flutter_test.dart';
import 'package:image_picker/image_picker.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/media/media_picker.dart';

void main() {
  test('galleryImage:XFile 归一为 image PickedMediaFile', () async {
    ImageSource? usedSource;
    final picker = MediaPicker(
      pickImage: (source) async {
        usedSource = source;
        return XFile('/tmp/photo.jpg');
      },
    );
    final result = await picker.galleryImage();
    expect(usedSource, ImageSource.gallery);
    expect(result, isNotNull);
    expect(result!.kind, ChatMessageKind.image);
    expect(result.mime, 'image/jpeg');
    expect(result.fileName, 'photo.jpg');
    expect(result.path, '/tmp/photo.jpg');
  });

  test('cameraVideo:XFile 归一为 video,来源为 camera', () async {
    ImageSource? usedSource;
    final picker = MediaPicker(
      pickVideo: (source) async {
        usedSource = source;
        return XFile('/tmp/clip.mp4');
      },
    );
    final result = await picker.cameraVideo();
    expect(usedSource, ImageSource.camera);
    expect(result!.kind, ChatMessageKind.video);
    expect(result.mime, 'video/mp4');
  });

  test('取消采集(null)返回 null', () async {
    final picker = MediaPicker(pickImage: (_) async => null);
    expect(await picker.cameraPhoto(), isNull);
  });

  test('mime 无法判定媒体类型时回退到来源提示 kind', () async {
    // .bin 无法判定 → application/octet-stream → kind 回退到 galleryImage 的 image。
    final picker = MediaPicker(pickImage: (_) async => XFile('/tmp/blob.bin'));
    final result = await picker.galleryImage();
    expect(result!.mime, 'application/octet-stream');
    expect(result.kind, ChatMessageKind.image);
  });

  test('XFile 自带 mimeType 时优先采用', () async {
    final picker = MediaPicker(
      pickImage: (_) async => XFile('/tmp/noext', mimeType: 'image/png'),
    );
    final result = await picker.galleryImage();
    expect(result!.mime, 'image/png');
    expect(result.kind, ChatMessageKind.image);
  });
}
