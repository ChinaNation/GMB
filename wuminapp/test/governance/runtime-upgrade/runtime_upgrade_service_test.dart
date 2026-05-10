import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/governance/runtime-upgrade/runtime_upgrade_service.dart';

void main() {
  Uint8List compactLength(int length) {
    if (length < 64) {
      return Uint8List.fromList([length << 2]);
    }
    if (length < 16384) {
      final encoded = (length << 2) | 0x01;
      return Uint8List.fromList([encoded & 0xff, (encoded >> 8) & 0xff]);
    }
    throw ArgumentError('测试辅助函数只覆盖 16KB 以下 Vec');
  }

  Uint8List compactBytes(Uint8List value) {
    return Uint8List.fromList([...compactLength(value.length), ...value]);
  }

  Uint8List storageValue(Uint8List proposalData) {
    return compactBytes(proposalData);
  }

  group('RuntimeUpgradeService 协议升级详情解码', () {
    test('解码带 rt-upg 前缀的协议升级提案摘要', () {
      final service = RuntimeUpgradeService();
      final proposer = Uint8List.fromList(List<int>.generate(32, (i) => i));
      final reason = Uint8List.fromList(utf8.encode('升级协议参数'));
      final codeHash = Uint8List.fromList(List<int>.filled(32, 0xab));
      final proposalData = Uint8List.fromList([
        ...utf8.encode('rt-upg'),
        ...proposer,
        ...compactBytes(reason),
        ...codeHash,
      ]);

      final decoded = service.decodeRuntimeUpgradeStorageValue(
        7,
        storageValue(proposalData),
      );

      expect(decoded, isNotNull);
      expect(decoded!.proposalId, 7);
      expect(decoded.reason, '升级协议参数');
      expect(decoded.codeHashHex, List.filled(32, 'ab').join());
    });

    test('非 rt-upg 提案摘要不按协议升级解码', () {
      final service = RuntimeUpgradeService();
      final proposer = Uint8List.fromList(List<int>.filled(32, 1));
      final reason = Uint8List.fromList(utf8.encode('其他提案'));
      final codeHash = Uint8List.fromList(List<int>.filled(32, 0xcd));
      final proposalData = Uint8List.fromList([
        ...utf8.encode('other'),
        ...proposer,
        ...compactBytes(reason),
        ...codeHash,
      ]);

      final decoded = service.decodeRuntimeUpgradeStorageValue(
        8,
        storageValue(proposalData),
      );

      expect(decoded, isNull);
    });

    test('带旧业务状态字段的协议升级摘要不再兼容', () {
      final service = RuntimeUpgradeService();
      final proposer = Uint8List.fromList(List<int>.generate(32, (i) => i));
      final reason = Uint8List.fromList(utf8.encode('旧摘要'));
      final codeHash = Uint8List.fromList(List<int>.filled(32, 0xef));
      final proposalData = Uint8List.fromList([
        ...utf8.encode('rt-upg'),
        ...proposer,
        ...compactBytes(reason),
        ...codeHash,
        0,
      ]);

      final decoded = service.decodeRuntimeUpgradeStorageValue(
        9,
        storageValue(proposalData),
      );

      expect(decoded, isNull);
    });
  });
}
