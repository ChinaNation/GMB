import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 广场卡片媒体区（横屏单块 / 2 图 / 3 图以上取前两张）。
///
/// 横竖屏由 `mediaItems.first.isPortrait`（来自媒体原始宽高）决定：
/// - 单块：横屏 16:9，竖屏兜底 3:4（竖屏单图/单视频的"左媒体右文字"由卡片组合，正常不走此分支）。
/// - 2 图 / 3 图以上：只出前两张，左右各半，外侧圆角、中缝直角、2px 细缝；
///   3 图以上在第二张右下角显示 `+N`（N = 总数 - 2）。
class SquareMediaGrid extends StatelessWidget {
  const SquareMediaGrid({super.key, required this.mediaItems});

  final List<SquareMediaItem> mediaItems;

  @override
  Widget build(BuildContext context) {
    if (mediaItems.isEmpty) return const SizedBox.shrink();

    final portrait = mediaItems.first.isPortrait;
    const r = AppTheme.radiusMd;

    if (mediaItems.length == 1) {
      return AspectRatio(
        aspectRatio: portrait ? 3 / 4 : 16 / 9,
        child: SquareMediaTile(
          item: mediaItems.first,
          radius: BorderRadius.circular(r),
        ),
      );
    }

    // 两张 tile 各占一半：容器比例保证左右图分别为 1:1（横）或 3:4（竖）。
    final hidden = mediaItems.length - 2;
    return AspectRatio(
      aspectRatio: portrait ? 3 / 2 : 2,
      child: Row(
        children: [
          Expanded(
            child: SquareMediaTile(
              item: mediaItems[0],
              radius: const BorderRadius.only(
                topLeft: Radius.circular(r),
                bottomLeft: Radius.circular(r),
              ),
            ),
          ),
          const SizedBox(width: 2),
          Expanded(
            child: SquareMediaTile(
              item: mediaItems[1],
              radius: const BorderRadius.only(
                topRight: Radius.circular(r),
                bottomRight: Radius.circular(r),
              ),
              overlayCount: hidden > 0 ? hidden : null,
            ),
          ),
        ],
      ),
    );
  }
}

/// 单个媒体块：图片/视频封面 + 视频播放键/冷归档态 + 右下角 `+N` 角标。
/// 由 [SquareMediaGrid] 与卡片竖屏单图/单视频布局共用。
class SquareMediaTile extends StatelessWidget {
  const SquareMediaTile({
    super.key,
    required this.item,
    required this.radius,
    this.overlayCount,
  });

  final SquareMediaItem item;
  final BorderRadius radius;

  /// 非空时在右下角显示 `+N`，表示还有 N 张未展开。
  final int? overlayCount;

  @override
  Widget build(BuildContext context) {
    final isVideo = item.mediaKind == SquareMediaKind.video;
    final imageUrl = isVideo ? (item.coverUrl ?? '') : item.url;
    return ClipRRect(
      borderRadius: radius,
      child: DecoratedBox(
        decoration: const BoxDecoration(color: AppTheme.surfaceElevated),
        child: Stack(
          fit: StackFit.expand,
          children: [
            if (imageUrl.isNotEmpty)
              Image.network(
                imageUrl,
                fit: BoxFit.cover,
                errorBuilder: (_, __, ___) => _fallbackIcon(isVideo),
              )
            else
              _fallbackIcon(isVideo),
            if (isVideo && (item.isArchived || item.isRestoring))
              _archiveOverlay(item.isRestoring)
            else if (isVideo)
              const Center(
                child: Icon(Icons.play_circle_fill_rounded,
                    size: 42, color: Colors.white70),
              ),
            if (overlayCount != null)
              Positioned(
                right: 8,
                bottom: 8,
                child: _CountBadge(count: overlayCount!),
              ),
          ],
        ),
      ),
    );
  }

  Widget _fallbackIcon(bool isVideo) {
    return Icon(
      isVideo ? Icons.play_circle_fill_rounded : Icons.image_rounded,
      size: 42,
      color: AppTheme.textTertiary,
    );
  }

  /// 视频冷归档占位：作者退订满 3 月后视频已移入冷存不可播，重订解冻；显示占位而非坏播放器。
  Widget _archiveOverlay(bool restoring) {
    return ColoredBox(
      color: Colors.black54,
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              restoring
                  ? Icons.hourglass_top_rounded
                  : Icons.inventory_2_outlined,
              size: 34,
              color: Colors.white70,
            ),
            const SizedBox(height: 6),
            Text(
              restoring ? '恢复中' : '已归档',
              style: const TextStyle(
                color: Colors.white70,
                fontSize: 12,
                fontWeight: FontWeight.w600,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _CountBadge extends StatelessWidget {
  const _CountBadge({required this.count});

  final int count;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        color: Colors.black.withAlpha(0x80),
        borderRadius: BorderRadius.circular(20),
      ),
      child: Text(
        '+$count',
        style: const TextStyle(
          color: Colors.white,
          fontSize: 12,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}
