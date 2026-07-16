import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/stickers/sticker_pack.dart';

void main() {
  test('清单 ⇔ 磁盘 png 一一对应,零死引用零孤儿', () {
    final dir = Directory('assets/stickers/fluent3d');
    expect(dir.existsSync(), isTrue, reason: '贴纸资产目录必须存在');
    final diskIds = dir
        .listSync()
        .whereType<File>()
        .map((f) => f.uri.pathSegments.last)
        .where((n) => n.endsWith('.png'))
        .map((n) => n.substring(0, n.length - 4))
        .toSet();
    final listIds = StickerPack.items.map((item) => item.id).toSet();
    // 清单每个 id 都有磁盘 png(渲染不会死引用)
    expect(listIds.difference(diskIds), isEmpty, reason: '清单有 id 缺对应 png');
    // 磁盘每个 png 都在清单(不留孤儿资产打包进 app)
    expect(diskIds.difference(listIds), isEmpty, reason: '磁盘有孤儿 png');
  });

  test('items id 唯一且计数为 48', () {
    final ids = StickerPack.items.map((item) => item.id).toList();
    expect(ids.toSet().length, ids.length, reason: 'id 不得重复');
    expect(ids.length, 48);
  });

  test('assetPath 由 id 稳定推导', () {
    expect(
      StickerPack.assetPath('grinning_face'),
      'assets/stickers/fluent3d/grinning_face.png',
    );
  });

  test('isKnown:已知贴纸真、未知 id 假、错包假', () {
    expect(
      StickerPack.isKnown(packId: 'fluent3d', stickerId: 'grinning_face'),
      isTrue,
    );
    expect(
      StickerPack.isKnown(packId: 'fluent3d', stickerId: 'not_a_sticker'),
      isFalse,
    );
    expect(
      StickerPack.isKnown(packId: 'other_pack', stickerId: 'grinning_face'),
      isFalse,
    );
  });

  test('grouped 覆盖全部且每类计数正确', () {
    final grouped = StickerPack.grouped();
    final total = grouped.values.fold<int>(0, (sum, list) => sum + list.length);
    expect(total, StickerPack.items.length);
    expect(grouped[StickerCategory.smileys]!.length, 16);
    expect(grouped[StickerCategory.gestures]!.length, 12);
    expect(grouped[StickerCategory.hearts]!.length, 10);
    expect(grouped[StickerCategory.celebration]!.length, 10);
  });
}
