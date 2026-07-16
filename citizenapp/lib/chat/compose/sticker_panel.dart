import 'package:flutter/material.dart';

import '../stickers/sticker_pack.dart';

/// 贴纸选择面板:Fluent 3D 内置包按分类分组的网格,点选回调 `(packId, stickerId)`。
///
/// 纯展示 + 回调,不含发送/加密逻辑(发送在 `chat_page`)。清单只读 [StickerPack]
/// 单源;缺图/解码失败由 `errorBuilder` 降级为占位图标,绝不崩。
class StickerPanel extends StatelessWidget {
  const StickerPanel({
    super.key,
    required this.onPick,
    this.height = 264,
  });

  /// 点选一个贴纸时回调,参数为 `(packId, stickerId)`。
  final void Function(String packId, String stickerId) onPick;

  /// 面板高度(含分类 Tab)。
  final double height;

  static const Map<StickerCategory, String> _categoryLabels = {
    StickerCategory.smileys: '表情',
    StickerCategory.gestures: '手势',
    StickerCategory.hearts: '爱心',
    StickerCategory.celebration: '庆祝',
  };

  @override
  Widget build(BuildContext context) {
    final grouped = StickerPack.grouped();
    final categories = grouped.keys.toList();
    return SizedBox(
      height: height,
      child: DefaultTabController(
        length: categories.length,
        child: Column(
          children: [
            TabBar(
              isScrollable: true,
              tabAlignment: TabAlignment.start,
              tabs: [
                for (final category in categories)
                  Tab(
                    height: 36,
                    text: _categoryLabels[category] ?? category.name,
                  ),
              ],
            ),
            Expanded(
              child: TabBarView(
                children: [
                  for (final category in categories)
                    _StickerGrid(items: grouped[category]!, onPick: onPick),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _StickerGrid extends StatelessWidget {
  const _StickerGrid({required this.items, required this.onPick});

  final List<StickerItem> items;
  final void Function(String packId, String stickerId) onPick;

  @override
  Widget build(BuildContext context) {
    return GridView.builder(
      padding: const EdgeInsets.all(8),
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 5,
        mainAxisSpacing: 4,
        crossAxisSpacing: 4,
      ),
      itemCount: items.length,
      itemBuilder: (context, index) {
        final item = items[index];
        return InkWell(
          key: ValueKey('sticker-${item.id}'),
          borderRadius: BorderRadius.circular(10),
          onTap: () => onPick(StickerPack.packId, item.id),
          child: Padding(
            padding: const EdgeInsets.all(6),
            child: Image.asset(
              StickerPack.assetPath(item.id),
              fit: BoxFit.contain,
              errorBuilder: (_, __, ___) => const Icon(
                Icons.broken_image_rounded,
                color: Colors.grey,
              ),
            ),
          ),
        );
      },
    );
  }
}
