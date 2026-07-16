import 'package:saver_gallery/saver_gallery.dart';

/// 把已收到的本机媒体文件保存到系统相册。相册写入权限已在采集步骤声明。
typedef GallerySaveFn = Future<bool> Function({
  required String filePath,
  required String fileName,
});

class MediaGallerySaver {
  const MediaGallerySaver();

  /// 保存成功返回 true。图片与视频统一走 saveFile(按 mime 归相册)。
  Future<bool> saveToGallery({
    required String filePath,
    required String fileName,
  }) async {
    final result = await SaverGallery.saveFile(
      filePath: filePath,
      fileName: fileName,
      skipIfExists: false,
    );
    return result.isSuccess;
  }
}
