import 'dart:io';
import 'dart:typed_data';

import 'package:citizenapp/chat/media/attachment_vault.dart';
import 'package:flutter_test/flutter_test.dart';

/// 附件本地静止态加密验收。
///
/// 重点是**明文生命周期**：长期缓存必须是密文；解出的明文只能活在专用短命目录，
/// 用完即删、异常路径也删、崩溃残留由启动清理兜底。
void main() {
  late Directory root;
  late Directory cacheDir;
  late Directory plainDir;
  final key = List<int>.generate(32, (i) => (i * 7) % 256);

  setUp(() {
    root = Directory.systemTemp.createTempSync('attach_vault_');
    cacheDir = Directory('${root.path}/cache')..createSync(recursive: true);
    plainDir = Directory('${root.path}/cache/${AttachmentVault.plainDirName}');
  });

  tearDown(() {
    if (root.existsSync()) root.deleteSync(recursive: true);
  });

  File writePlain(String name, List<int> bytes) {
    final f = File('${root.path}/$name')..createSync(recursive: true);
    f.writeAsBytesSync(bytes);
    return f;
  }

  test('封存后长期缓存是密文，且明文源被删除', () async {
    final secret =
        Uint8List.fromList(List<int>.generate(4096, (i) => (i * 13) % 256));
    final source = writePlain('src.bin', secret);
    final cachePath = '${cacheDir.path}/conv/att-1_photo.jpg';

    await AttachmentVault.seal(
      plainSource: source,
      cachePath: cachePath,
      key: key,
    );

    // 明文源已删
    expect(source.existsSync(), isFalse);
    // 明文路径本身不存在，只有 .enc
    expect(File(cachePath).existsSync(), isFalse);
    final cipher = AttachmentVault.cipherFileOf(cachePath);
    expect(cipher.existsSync(), isTrue);

    // 密文内容不等于明文
    final cipherBytes = cipher.readAsBytesSync();
    expect(cipherBytes, isNot(secret));
    expect(await AttachmentVault.hasCipher(cachePath), isTrue);
  });

  test('解密到短命明文并可完整还原', () async {
    final secret =
        Uint8List.fromList(List<int>.generate(9000, (i) => (i * 31) % 256));
    final cachePath = '${cacheDir.path}/conv/att-2_clip.mp4';
    await AttachmentVault.seal(
      plainSource: writePlain('src2.bin', secret),
      cachePath: cachePath,
      key: key,
    );

    final plain = await AttachmentVault.openPlain(
      cachePath: cachePath,
      key: key,
      plainDirectory: plainDir,
    );
    expect(plain.readAsBytesSync(), secret);
    // 明文只落专用目录，且保留扩展名供播放器选解码器
    expect(plain.path.startsWith(plainDir.path), isTrue);
    expect(plain.path.endsWith('.mp4'), isTrue);

    await AttachmentVault.releasePlain(plain);
    expect(plain.existsSync(), isFalse, reason: '用完必须立刻删掉明文');
  });

  test('错误密钥解密失败，且不得留下半截明文', () async {
    final cachePath = '${cacheDir.path}/conv/att-3.bin';
    await AttachmentVault.seal(
      plainSource: writePlain('src3.bin', List<int>.filled(5000, 7)),
      cachePath: cachePath,
      key: key,
    );

    final wrongKey = List<int>.generate(32, (i) => 255 - i);
    await expectLater(
      AttachmentVault.openPlain(
        cachePath: cachePath,
        key: wrongKey,
        plainDirectory: plainDir,
      ),
      throwsA(anything),
    );
    // 异常路径同样不能把明文留在盘上
    final leftovers =
        plainDir.existsSync() ? plainDir.listSync() : const <FileSystemEntity>[];
    expect(leftovers, isEmpty, reason: '解密失败也必须清掉半截明文');
  });

  test('密文缺失时明确报错，不静默返回空文件', () async {
    await expectLater(
      AttachmentVault.openPlain(
        cachePath: '${cacheDir.path}/conv/missing.bin',
        key: key,
        plainDirectory: plainDir,
      ),
      throwsA(isA<StateError>()),
    );
  });

  test('启动清理兜底：崩溃残留的明文被整目录清空', () async {
    plainDir.createSync(recursive: true);
    File('${plainDir.path}/leaked1.jpg').writeAsStringSync('明文残留');
    File('${plainDir.path}/leaked2.mp4').writeAsStringSync('明文残留');
    expect(plainDir.listSync(), hasLength(2));

    await AttachmentVault.purgePlainDirectory(plainDir);
    expect(plainDir.existsSync(), isFalse);
  });

  test('existingPlain:未解密时返回 null,不触发解密', () async {
    final cachePath = '${cacheDir.path}/conv/att-5.bin';
    await AttachmentVault.seal(
      plainSource: writePlain('src5.bin', List<int>.filled(2048, 3)),
      cachePath: cachePath,
      key: key,
    );
    expect(
      await AttachmentVault.existingPlain(
        cachePath: cachePath,
        plainDirectory: plainDir,
      ),
      isNull,
    );
    // 探测不得产生任何明文
    expect(plainDir.existsSync() ? plainDir.listSync() : const [], isEmpty);
  });

  test('existingPlain:已解密后命中,可复用免二次解密', () async {
    final secret = List<int>.generate(3000, (i) => (i * 11) % 256);
    final cachePath = '${cacheDir.path}/conv/att-6.bin';
    await AttachmentVault.seal(
      plainSource: writePlain('src6.bin', secret),
      cachePath: cachePath,
      key: key,
    );
    final first = await AttachmentVault.openPlain(
      cachePath: cachePath,
      key: key,
      plainDirectory: plainDir,
    );
    final hit = await AttachmentVault.existingPlain(
      cachePath: cachePath,
      plainDirectory: plainDir,
    );
    expect(hit, isNotNull);
    expect(hit!.path, first.path);
    expect(hit.readAsBytesSync(), secret);
    // 用错密钥也能拿到复用句柄——证明这条路径确实没走解密
    final reusedWithWrongKey = await AttachmentVault.existingPlain(
      cachePath: cachePath,
      plainDirectory: plainDir,
    );
    expect(reusedWithWrongKey, isNotNull);
  });

  test('清理不存在的目录是安全的', () async {
    await AttachmentVault.purgePlainDirectory(
      Directory('${root.path}/never-created'),
    );
  });

  test('重复 openPlain 覆盖旧明文，不残留多份', () async {
    final cachePath = '${cacheDir.path}/conv/att-4.bin';
    await AttachmentVault.seal(
      plainSource: writePlain('src4.bin', List<int>.filled(1024, 9)),
      cachePath: cachePath,
      key: key,
    );

    final a = await AttachmentVault.openPlain(
      cachePath: cachePath,
      key: key,
      plainDirectory: plainDir,
    );
    final b = await AttachmentVault.openPlain(
      cachePath: cachePath,
      key: key,
      plainDirectory: plainDir,
    );
    expect(a.path, b.path);
    expect(plainDir.listSync(), hasLength(1));
    await AttachmentVault.releasePlain(b);
    expect(plainDir.listSync(), isEmpty);
  });
}
