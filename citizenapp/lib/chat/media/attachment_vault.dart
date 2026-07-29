import 'dart:io';

import 'media_relay_crypto.dart';

/// 聊天附件的本地静止态金库。
///
/// **长期缓存一律密文**(`<原路径>.enc`,复用 [MediaRelayCrypto] 的分块流式
/// AES-256-GCM,5GB 也不进内存);播放/预览时才解密到**短命明文临时文件**。
///
/// 明文窗口是本方案(用户 2026-07-29 选定的方案 A)自觉接受的代价:图片/视频
/// 播放器要的是文件路径而不是内存字节,走内存流对视频的工程代价过高。
/// 代价靠把生命周期管死来压缩:
/// - 明文只落 [plainDirName] 这一个专用目录,与密文缓存物理分开;
/// - 用完即删([releasePlain]),异常路径也删(调用方 try/finally);
/// - App 启动时整目录清空([purgePlainDirectory]),防崩溃残留跨会话存活。
class AttachmentVault {
  const AttachmentVault._();

  /// 密文缓存后缀。与明文路径永不重名,避免"以为加密了其实读的是旧明文"。
  static const String cipherSuffix = '.enc';

  /// 短命明文目录名(位于附件缓存根下)。
  static const String plainDirName = '.plain';

  static File cipherFileOf(String cachePath) => File('$cachePath$cipherSuffix');

  /// 把明文文件加密进长期缓存,成功后**删除明文源**。
  ///
  /// [deleteSource]=false 用于发送端保留用户原始文件的场景。
  static Future<void> seal({
    required File plainSource,
    required String cachePath,
    required List<int> key,
    bool deleteSource = true,
  }) async {
    final target = cipherFileOf(cachePath);
    await target.parent.create(recursive: true);
    await MediaRelayCrypto.encryptFile(
      sourcePath: plainSource.path,
      destPath: target.path,
      key: key,
    );
    if (deleteSource && await plainSource.exists()) {
      await plainSource.delete();
    }
  }

  /// 密文缓存是否已就绪。
  static Future<bool> hasCipher(String cachePath) =>
      cipherFileOf(cachePath).exists();

  /// 已解密好的短命明文文件（不存在返回 null，不触发解密）。
  ///
  /// 供列表渲染这类**高频路径**先探一次，命中就直接复用，避免同一附件被反复
  /// 整文件解密——会话里每条媒体消息都会走一次路径解析，视频动辄上百 MB。
  static Future<File?> existingPlain({
    required String cachePath,
    required Directory plainDirectory,
  }) async {
    final plain = File('${plainDirectory.path}/${_plainName(cachePath)}');
    return await plain.exists() ? plain : null;
  }

  /// 解密到短命明文临时文件并返回它。
  ///
  /// 明文生命周期由「前台存活」策略统一兜底（启动 / 退后台 / 删会话三处 purge），
  /// 调用方不需要逐处交接所有权；[releasePlain] 只用于确定不再需要的即时清理。
  static Future<File> openPlain({
    required String cachePath,
    required List<int> key,
    required Directory plainDirectory,
  }) async {
    final cipher = cipherFileOf(cachePath);
    if (!await cipher.exists()) {
      throw StateError('附件密文缓存不存在: $cachePath');
    }
    await plainDirectory.create(recursive: true);
    final plain = File('${plainDirectory.path}/${_plainName(cachePath)}');
    if (await plain.exists()) {
      await plain.delete();
    }
    try {
      await MediaRelayCrypto.decryptFile(
        sourcePath: cipher.path,
        destPath: plain.path,
        key: key,
      );
    } catch (_) {
      // 解密失败也不能把半截明文留在盘上。
      if (await plain.exists()) {
        await plain.delete();
      }
      rethrow;
    }
    return plain;
  }

  /// 删除某个短命明文文件（用完即调，失败静默——文件可能已被清理）。
  static Future<void> releasePlain(File plain) async {
    try {
      if (await plain.exists()) {
        await plain.delete();
      }
    } on FileSystemException {
      // 已被 purge 或系统清理，无需处理。
    }
  }

  /// 整目录清空短命明文（App 启动 / 退出账户时调）。
  ///
  /// 崩溃或强杀会跳过 [releasePlain]，必须有这道兜底，否则明文会跨会话存活。
  static Future<void> purgePlainDirectory(Directory plainDirectory) async {
    if (!await plainDirectory.exists()) return;
    try {
      await plainDirectory.delete(recursive: true);
    } on FileSystemException {
      // 目录被占用时逐个删，尽力而为。
      await for (final entity in plainDirectory.list()) {
        try {
          await entity.delete(recursive: true);
        } on FileSystemException {
          continue;
        }
      }
    }
  }

  /// 明文临时文件名：保留原扩展名，播放器按扩展名选解码器。
  static String _plainName(String cachePath) {
    final base = cachePath.split(Platform.pathSeparator).last;
    return base.isEmpty ? 'attachment.bin' : base;
  }
}
