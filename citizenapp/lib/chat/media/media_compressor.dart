import 'dart:io';

import 'package:flutter_image_compress/flutter_image_compress.dart';
import 'package:path_provider/path_provider.dart';

import '../chat_media_limits.dart';
import '../chat_models.dart';

/// 取文件字节大小(默认走 File.length,可注入测试)。
typedef FileSizeReader = Future<int> Function(String path);

/// 把图片压到目标上限内,返回压缩后文件路径;无法压缩返回 null(默认走
/// flutter_image_compress,可注入测试)。
typedef ImageCompressStep = Future<String?> Function(String path, int limit);

/// 媒体大小门控(采集侧,门①的上游)。
///
/// 定稿策略:**图片仅超限才压缩**——正常图原样直通(尊重 100MB 上限);
/// 只有 >100MB 才压一次,压后仍超则拒。视频/文件**不转码**,超限直接拒。
/// 全程按文件大小(stat)判定,不整解码,100MB 图也不炸内存。
class MediaCompressor {
  MediaCompressor({
    FileSizeReader? sizeOf,
    ImageCompressStep? compressImage,
  })  : _sizeOf = sizeOf ?? _defaultSizeOf,
        _compressImage = compressImage ?? _defaultCompressImage;

  final FileSizeReader _sizeOf;
  final ImageCompressStep _compressImage;

  /// 返回最终待发送文件路径(可能是压缩后的临时文件)。超限且无法压到限内时抛
  /// [ChatMediaTooLargeException]。上限由 [kind] 决定。
  Future<String> ensureWithinLimit({
    required String path,
    required ChatMessageKind kind,
  }) async {
    final limit = ChatMediaLimits.forKind(kind);
    final size = await _sizeOf(path);
    if (size <= limit) {
      return path; // 正常直通,不改动源文件
    }
    if (kind != ChatMessageKind.image) {
      // 视频/文件不转码,超限直接拒。
      throw ChatMediaTooLargeException(
          byteSize: size, limitBytes: limit, kind: kind);
    }
    // 图片超限:压一次。
    final compressed = await _compressImage(path, limit);
    if (compressed == null) {
      throw ChatMediaTooLargeException(
          byteSize: size, limitBytes: limit, kind: kind);
    }
    final newSize = await _sizeOf(compressed);
    if (newSize > limit) {
      throw ChatMediaTooLargeException(
          byteSize: newSize, limitBytes: limit, kind: kind);
    }
    return compressed;
  }
}

Future<int> _defaultSizeOf(String path) => File(path).length();

Future<String?> _defaultCompressImage(String path, int limit) async {
  // 原生降质压缩(不在 Dart 侧整解码);目标限内即可,尽量保尺寸。
  final dir = await getTemporaryDirectory();
  final target =
      '${dir.path}/chat_compress_${DateTime.now().microsecondsSinceEpoch}.jpg';
  final result = await FlutterImageCompress.compressAndGetFile(
    path,
    target,
    quality: 85,
  );
  return result?.path;
}
