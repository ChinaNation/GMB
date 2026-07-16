import 'package:image_picker/image_picker.dart';

import '../chat_models.dart';
import 'media_mime.dart';

/// 采集到的媒体文件(路径型,不载入字节)。
class PickedMediaFile {
  const PickedMediaFile({
    required this.path,
    required this.fileName,
    required this.mime,
    required this.kind,
  });

  final String path;
  final String fileName;
  final String mime;
  final ChatMessageKind kind;
}

/// image_picker 的取文件回调(可注入测试)。
typedef XFilePicker = Future<XFile?> Function(ImageSource source);

/// 相册/相机图片、相册/录像视频的采集封装。
///
/// 只负责把 `XFile` 归一为路径型 [PickedMediaFile](kind 以内容 mime 为准,回退到
/// 来源提示),不载入字节;字节由 2a 发送管道从路径流式读取。通用文件走 file_picker
/// (归 chat_page)。native 取文件经可注入 seam,归一逻辑可单测。
class MediaPicker {
  MediaPicker({
    XFilePicker? pickImage,
    XFilePicker? pickVideo,
  })  : _pickImage = pickImage ?? _defaultPickImage,
        _pickVideo = pickVideo ?? _defaultPickVideo;

  final XFilePicker _pickImage;
  final XFilePicker _pickVideo;

  Future<PickedMediaFile?> galleryImage() =>
      _pick(_pickImage, ImageSource.gallery, ChatMessageKind.image);

  Future<PickedMediaFile?> cameraPhoto() =>
      _pick(_pickImage, ImageSource.camera, ChatMessageKind.image);

  Future<PickedMediaFile?> galleryVideo() =>
      _pick(_pickVideo, ImageSource.gallery, ChatMessageKind.video);

  Future<PickedMediaFile?> cameraVideo() =>
      _pick(_pickVideo, ImageSource.camera, ChatMessageKind.video);

  Future<PickedMediaFile?> _pick(
    XFilePicker pick,
    ImageSource source,
    ChatMessageKind kindHint,
  ) async {
    final file = await pick(source);
    if (file == null) return null;
    final fileName = file.name;
    final mime = file.mimeType ?? mimeFromFileName(fileName);
    // kind 以内容 mime 为准;mime 无法判定媒体类型时回退到来源提示。
    final kind = mime.startsWith('image/') || mime.startsWith('video/')
        ? mediaKindFromMime(mime)
        : kindHint;
    return PickedMediaFile(
      path: file.path,
      fileName: fileName,
      mime: mime,
      kind: kind,
    );
  }
}

Future<XFile?> _defaultPickImage(ImageSource source) =>
    ImagePicker().pickImage(source: source);

Future<XFile?> _defaultPickVideo(ImageSource source) =>
    ImagePicker().pickVideo(source: source);
