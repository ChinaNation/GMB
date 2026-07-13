import 'dart:io';

import 'package:path_provider/path_provider.dart';

import 'package:citizenapp/8964/models/square_models.dart';

/// 草稿媒体本地持久化：把 image_picker 的临时文件复制到应用文档目录下的草稿目录，
/// 避免退出/重启后 OS 清理临时缓存导致草稿媒体丢失。
class ComposeDraftMedia {
  const ComposeDraftMedia();

  static Future<Directory> _root() async {
    final docs = await getApplicationDocumentsDirectory();
    final dir = Directory('${docs.path}/square_drafts');
    if (!await dir.exists()) await dir.create(recursive: true);
    return dir;
  }

  static Future<Directory> _draftDir(String draftId) async {
    final dir = Directory('${(await _root()).path}/$draftId');
    if (!await dir.exists()) await dir.create(recursive: true);
    return dir;
  }

  /// 已在本草稿目录内则原样返回（幂等）；否则复制进去并返回持久路径草稿。
  static Future<SquareLocalMediaDraft> persist(
    String draftId,
    SquareLocalMediaDraft media,
  ) async {
    final dir = await _draftDir(draftId);
    if (media.path.startsWith('${dir.path}/')) return media;
    final ext = media.fileExt.isNotEmpty ? '.${media.fileExt}' : '';
    final target =
        File('${dir.path}/${DateTime.now().microsecondsSinceEpoch}$ext');
    await File(media.path).copy(target.path);
    return SquareLocalMediaDraft(
      mediaKind: media.mediaKind,
      path: target.path,
      fileName: media.fileName,
      contentType: media.contentType,
      byteSize: media.byteSize,
      durationSeconds: media.durationSeconds,
    );
  }

  /// 删除一条草稿的整个媒体目录（删草稿或发布成功后调用）。
  static Future<void> deleteDir(String draftId) async {
    final dir = Directory('${(await _root()).path}/$draftId');
    if (await dir.exists()) await dir.delete(recursive: true);
  }
}
