import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/signer/signing.dart';

// SCALE 编码原语金标锁(citizenapp ⇔ citizenchain)。
//
// 本文件**直接读真源**,不保存镜像副本(与 Worker / citizenwallet 同策略)。
//
// 为什么需要:`scaleString` / `u64Le` / `_scaleCompact` 是**手写**实现,而链端用
// parity-scale-codec。此前唯一引用它们的 square_action_payload_test.dart 是拿它们去
// **构造期望值**——实现算错期望值同步错,测试照样绿。这些字节直接决定被签 payload。
//
// 真源:citizenchain/runtime/primitives/tests/fixtures/scale_codec_vectors.json
// 生成器:citizenchain/runtime/primitives/tests/scale_codec_golden.rs
//        (SCALE_GOLDEN_UPDATE=1 重新生成)

const String _vectorsPath =
    '../citizenchain/runtime/primitives/tests/fixtures/scale_codec_vectors.json';

String _bytesToHex(List<int> bytes) =>
    bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

void main() {
  final file = File(_vectorsPath);
  final canonical =
      jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
  final compactVectors =
      (canonical['compact_u32'] as List).cast<Map<String, dynamic>>();
  final stringVectors =
      (canonical['scale_string'] as List).cast<Map<String, dynamic>>();
  final u64Vectors = (canonical['u64_le'] as List).cast<Map<String, dynamic>>();

  group('SCALE 编码原语与链端一致(直读 citizenchain 真源)', () {
    test('真源三组向量均可读且非空', () {
      // 读成空数组时下面的循环一条用例都不生成而整体显示通过,这条挡住金标静默失效。
      expect(file.existsSync(), isTrue,
          reason: '真源不可达,cwd=${Directory.current.path}');
      expect(compactVectors, isNotEmpty);
      expect(stringVectors, isNotEmpty);
      expect(u64Vectors, isNotEmpty);
    });

    for (final vector in stringVectors) {
      final value = vector['value'] as String;
      final label = value.length > 24 ? '${value.substring(0, 24)}…' : value;
      test('scaleString(${jsonEncode(label)}) 共 ${vector['utf8_len']} 字节', () {
        expect(_bytesToHex(scaleString(value)), vector['hex']);
      });
    }

    for (final vector in u64Vectors) {
      test('u64Le(${vector['value']})', () {
        expect(_bytesToHex(u64Le(vector['value'] as int)), vector['hex']);
      });
    }
  });

  group('compact 长度前缀覆盖全部三档', () {
    // `_scaleCompact` 是 private,测试够不到;但 scaleString = compact(len) ++ utf8,
    // 用长度恰为向量取值的 ASCII 串反推前缀,即可把三档分支(< 2^6 / < 2^14 / < 2^30)
    // 的边界全部锁住。把 `<` 写成 `<=` 会让 63/64、16383/16384 中的一对编错字节数。
    for (final vector in compactVectors) {
      final length = vector['value'] as int;
      final expectedPrefix = vector['hex'] as String;
      // 4 字节档的上界是 2^30-1,构造那么长的字符串不现实;只覆盖可构造的长度。
      if (length > 65535) continue;
      test('长度 $length 的字符串前缀为 $expectedPrefix', () {
        final encoded = scaleString('x' * length);
        expect(
          _bytesToHex(encoded).substring(0, expectedPrefix.length),
          expectedPrefix,
        );
        // 前缀之后必须是原始字节,长度与前缀声明一致。
        expect(encoded.length, expectedPrefix.length ~/ 2 + length);
      });
    }
  });

  group('SCALE 编码原语的 fail-closed 边界', () {
    // 真源向量只覆盖合法取值。非法输入必须抛错而不是产出错误字节 ——
    // 静默产出会让一笔语义错误的交易被签名。
    test('u64Le 拒绝负数', () {
      expect(() => u64Le(-1), throwsArgumentError);
    });

    // 不测 `_scaleCompact` 的 2^30-1 上界:要触发它得构造 1GB 字符串,
    // 不现实。该上界由链端 `scale_codec_golden.rs` 的 compact 向量与
    // Worker 侧 `scaleCompact(2 ** 30)` 抛错断言共同守住。
  });
}
