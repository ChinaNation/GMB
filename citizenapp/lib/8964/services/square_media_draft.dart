import 'package:image_picker/image_picker.dart';
import 'package:path/path.dart' as path;

import 'package:citizenapp/8964/models/square_models.dart';

/// 从相册 [XFile] 构造发布用本地媒体草稿（发动态 / 发文章共用）。
Future<SquareLocalMediaDraft> buildSquareMediaDraft(
  XFile file,
  SquareMediaKind mediaKind,
) async {
  final name = file.name.isNotEmpty ? file.name : path.basename(file.path);
  final contentType = file.mimeType ?? _inferContentType(name, mediaKind);
  return SquareLocalMediaDraft(
    mediaKind: mediaKind,
    path: file.path,
    fileName: name,
    contentType: contentType,
    byteSize: await file.length(),
  );
}

String _inferContentType(String fileName, SquareMediaKind mediaKind) {
  final ext = path.extension(fileName).toLowerCase();
  if (mediaKind == SquareMediaKind.video) {
    return ext == '.webm' ? 'video/webm' : 'video/mp4';
  }
  return switch (ext) {
    '.png' => 'image/png',
    '.webp' => 'image/webp',
    _ => 'image/jpeg',
  };
}
