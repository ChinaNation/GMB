import 'dart:io';

import 'package:flutter/material.dart';

import '../media/media_gallery_saver.dart';

/// 图片全屏查看:捏合缩放(InteractiveViewer)+ 保存到相册。
///
/// 用屏幕物理像素做 `cacheWidth` 降采样解码——即便原图 100MB,也只解码到屏幕
/// 尺寸的位图,不整解码。保存动作可注入以便测试。
class ImageViewerPage extends StatelessWidget {
  const ImageViewerPage({
    super.key,
    required this.filePath,
    required this.fileName,
    GallerySaveFn? onSaveToGallery,
  }) : _onSaveToGallery = onSaveToGallery;

  final String filePath;
  final String fileName;
  final GallerySaveFn? _onSaveToGallery;

  Future<void> _save(BuildContext context) async {
    final saver = _onSaveToGallery ?? const MediaGallerySaver().saveToGallery;
    final ok = await saver(filePath: filePath, fileName: fileName);
    if (!context.mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(ok ? '已保存到相册' : '保存失败')),
    );
  }

  @override
  Widget build(BuildContext context) {
    final ratio = MediaQuery.of(context).devicePixelRatio;
    final cacheWidth = (MediaQuery.of(context).size.width * ratio).round();
    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        backgroundColor: Colors.black,
        foregroundColor: Colors.white,
        elevation: 0,
        actions: [
          IconButton(
            tooltip: '保存到相册',
            icon: const Icon(Icons.download_rounded),
            onPressed: () => _save(context),
          ),
        ],
      ),
      body: Center(
        child: InteractiveViewer(
          minScale: 1,
          maxScale: 5,
          child: Image.file(
            File(filePath),
            fit: BoxFit.contain,
            cacheWidth: cacheWidth,
            errorBuilder: (_, __, ___) => const Icon(
              Icons.broken_image_rounded,
              color: Colors.white54,
              size: 48,
            ),
          ),
        ),
      ),
    );
  }
}
