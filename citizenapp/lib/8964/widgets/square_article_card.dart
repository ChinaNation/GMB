import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 文章卡：首图（media_items[0]）+ 标题 + 正文摘要 + 作者/时间。
class SquareArticleCard extends StatelessWidget {
  const SquareArticleCard({super.key, required this.post, this.onTap});

  final SquarePost post;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final cover = post.mediaItems.isNotEmpty ? post.mediaItems.first : null;
    final title = post.title?.trim();
    return Card(
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: onTap,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (cover != null && cover.url.isNotEmpty)
              AspectRatio(
                aspectRatio: 16 / 9,
                child: Image.network(
                  cover.url,
                  fit: BoxFit.cover,
                  errorBuilder: (_, __, ___) => const ColoredBox(
                    color: AppTheme.surfaceElevated,
                    child: Center(
                      child: Icon(Icons.image_rounded,
                          size: 42, color: AppTheme.textTertiary),
                    ),
                  ),
                ),
              ),
            Padding(
              padding: const EdgeInsets.all(14),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (title != null && title.isNotEmpty)
                    Text(
                      title,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        color: AppTheme.textPrimary,
                        fontSize: 17,
                        fontWeight: FontWeight.w700,
                        height: 1.35,
                      ),
                    ),
                  if (post.text.trim().isNotEmpty) ...[
                    const SizedBox(height: 6),
                    Text(
                      post.text.trim(),
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        color: AppTheme.textSecondary,
                        fontSize: 14,
                        height: 1.45,
                      ),
                    ),
                  ],
                  const SizedBox(height: 10),
                  Row(
                    children: [
                      const Icon(Icons.article_outlined,
                          size: 15, color: AppTheme.textTertiary),
                      const SizedBox(width: 6),
                      Expanded(
                        child: Text(
                          post.author.title,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(
                            color: AppTheme.textTertiary,
                            fontSize: 12,
                          ),
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
