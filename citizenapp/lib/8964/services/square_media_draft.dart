import 'dart:io';

import 'package:image_picker/image_picker.dart';
import 'package:path/path.dart' as path;
import 'package:video_player/video_player.dart';

import 'package:citizenapp/8964/models/square_models.dart';

/// 从相册 [XFile] 构造发布用本地媒体草稿（发动态 / 发文章共用）。
Future<SquareLocalMediaDraft> buildSquareMediaDraft(
  XFile file,
  SquareMediaKind mediaKind,
) async {
  final name = file.name.isNotEmpty ? file.name : path.basename(file.path);
  final contentType = file.mimeType ?? _inferContentType(name, mediaKind);
  int? durationSeconds;
  if (mediaKind == SquareMediaKind.video) {
    final controller = VideoPlayerController.file(File(file.path));
    try {
      await controller.initialize();
      durationSeconds = controller.value.duration.inMilliseconds <= 0
          ? null
          : (controller.value.duration.inMilliseconds / 1000).ceil();
    } finally {
      await controller.dispose();
    }
    if (durationSeconds == null) {
      throw const FormatException('无法读取视频时长');
    }
  }
  return SquareLocalMediaDraft(
    mediaKind: mediaKind,
    path: file.path,
    fileName: name,
    contentType: contentType,
    byteSize: await file.length(),
    durationSeconds: durationSeconds,
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
