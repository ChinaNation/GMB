import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/ui/app_theme.dart';

class SquareMediaGrid extends StatelessWidget {
  const SquareMediaGrid({
    super.key,
    required this.mediaItems,
  });

  final List<SquareMediaItem> mediaItems;

  @override
  Widget build(BuildContext context) {
    if (mediaItems.isEmpty) {
      return const SizedBox.shrink();
    }

    final itemCount = mediaItems.length.clamp(1, 4);
    return GridView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: itemCount,
      gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: itemCount == 1 ? 1 : 2,
        crossAxisSpacing: 6,
        mainAxisSpacing: 6,
        childAspectRatio: itemCount == 1 ? 1.7 : 1,
      ),
      itemBuilder: (context, index) {
        final item = mediaItems[index];
        final hiddenCount = mediaItems.length - itemCount;
        return _MediaTile(
          item: item,
          overlayText: index == itemCount - 1 && hiddenCount > 0
              ? '+$hiddenCount'
              : null,
        );
      },
    );
  }
}

class _MediaTile extends StatelessWidget {
  const _MediaTile({
    required this.item,
    required this.overlayText,
  });

  final SquareMediaItem item;
  final String? overlayText;

  @override
  Widget build(BuildContext context) {
    final isVideo = item.mediaKind == SquareMediaKind.video;
    final imageUrl = isVideo ? (item.coverUrl ?? '') : item.url;
    return ClipRRect(
      borderRadius: BorderRadius.circular(AppTheme.radiusSm),
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
            if (overlayText != null)
              ColoredBox(
                color: Colors.black38,
                child: Center(
                  child: Text(
                    overlayText!,
                    style: const TextStyle(
                      color: Colors.white,
                      fontSize: 22,
                      fontWeight: FontWeight.w800,
                    ),
                  ),
                ),
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
              restoring ? Icons.hourglass_top_rounded : Icons.inventory_2_outlined,
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
