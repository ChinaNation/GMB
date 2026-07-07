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
            if (isVideo)
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
}
