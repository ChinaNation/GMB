// 大媒体(>100MB)中转的客户端流式加密。
//
// 一次性随机内容密钥 K,逐块 AES-256-GCM(对齐 ADR-022);nonce = 块序号(K 一次性,
// 序号唯一 → nonce 唯一,安全)。GCM tag 即完整性,被篡改的密文解密即失败,**不另加
// sha256**(延续 2d 定调)。K 只随 E2E 信封传,Cloudflare 只经手密文、拿不到 K。
// 全程 RandomAccessFile 定长分块读写,5GB 不进内存。

import 'dart:io';
import 'dart:math';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';

class MediaRelayCrypto {
  MediaRelayCrypto._();

  /// 默认分块 1 MiB。
  static const int defaultChunkSize = 1024 * 1024;

  /// 每帧头:4 字节大端密文长度。
  static const int _frameHeaderBytes = 4;

  /// GCM tag 16 字节。
  static const int macBytes = 16;

  static final AesGcm _algorithm = AesGcm.with256bits();

  /// 生成一次性内容密钥(32 字节)。
  static List<int> newContentKey() {
    final random = Random.secure();
    return List<int>.generate(32, (_) => random.nextInt(256));
  }

  /// 块序号 → 12 字节 GCM nonce(后 8 字节大端序号)。
  static List<int> _nonceForChunk(int chunkIndex) {
    final nonce = Uint8List(12);
    final view = ByteData.sublistView(nonce);
    view.setUint64(4, chunkIndex, Endian.big);
    return nonce;
  }

  /// 加密单块 → `cipherText || mac(16)`(纯,可测)。
  static Future<List<int>> encryptChunk(
    List<int> key,
    int chunkIndex,
    List<int> plaintext,
  ) async {
    final box = await _algorithm.encrypt(
      plaintext,
      secretKey: SecretKey(key),
      nonce: _nonceForChunk(chunkIndex),
    );
    return <int>[...box.cipherText, ...box.mac.bytes];
  }

  /// 解密单块(`cipherText || mac(16)`);tag 不符抛 [SecretBoxAuthenticationError]。
  static Future<List<int>> decryptChunk(
    List<int> key,
    int chunkIndex,
    List<int> frame,
  ) async {
    if (frame.length < macBytes) {
      throw const FormatException('中转密文块过短');
    }
    final cipherText = frame.sublist(0, frame.length - macBytes);
    final mac = frame.sublist(frame.length - macBytes);
    return _algorithm.decrypt(
      SecretBox(cipherText, nonce: _nonceForChunk(chunkIndex), mac: Mac(mac)),
      secretKey: SecretKey(key),
    );
  }

  /// 流式加密源文件 → 目标文件(定长分块,每块写 `[uint32 帧长][帧]`)。
  /// 返回密文总字节。5GB 不进内存。
  static Future<int> encryptFile({
    required String sourcePath,
    required String destPath,
    required List<int> key,
    int chunkSize = defaultChunkSize,
  }) async {
    final source = await File(sourcePath).open();
    final sink = File(destPath).openWrite();
    var written = 0;
    try {
      final total = await source.length();
      var index = 0;
      for (var offset = 0; offset < total; offset += chunkSize) {
        await source.setPosition(offset);
        final take = min(chunkSize, total - offset);
        final plain = await source.read(take);
        final frame = await encryptChunk(key, index, plain);
        final header = ByteData(_frameHeaderBytes)
          ..setUint32(0, frame.length, Endian.big);
        sink.add(header.buffer.asUint8List());
        sink.add(frame);
        written += _frameHeaderBytes + frame.length;
        index += 1;
      }
    } finally {
      await source.close();
      await sink.close();
    }
    return written;
  }

  /// 流式解密中转密文文件 → 目标明文文件。tag 不符任一块即抛错(中止)。
  static Future<void> decryptFile({
    required String sourcePath,
    required String destPath,
    required List<int> key,
  }) async {
    final source = await File(sourcePath).open();
    final sink = File(destPath).openWrite();
    try {
      final total = await source.length();
      var pos = 0;
      var index = 0;
      while (pos < total) {
        await source.setPosition(pos);
        final header = await source.read(_frameHeaderBytes);
        if (header.length < _frameHeaderBytes) {
          throw const FormatException('中转密文帧头截断');
        }
        final frameLen =
            ByteData.sublistView(Uint8List.fromList(header)).getUint32(0, Endian.big);
        await source.setPosition(pos + _frameHeaderBytes);
        final frame = await source.read(frameLen);
        if (frame.length < frameLen) {
          throw const FormatException('中转密文帧截断');
        }
        final plain = await decryptChunk(key, index, frame);
        sink.add(plain);
        pos += _frameHeaderBytes + frameLen;
        index += 1;
      }
    } finally {
      await source.close();
      await sink.close();
    }
  }
}
