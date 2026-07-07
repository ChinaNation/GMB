import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 文章详情：首图 + 标题 + 作者 + 正文全文 + 正文图（media_items[1..]）。
class SquareArticleDetailPage extends StatelessWidget {
  const SquareArticleDetailPage({super.key, required this.post});

  final SquarePost post;

  @override
  Widget build(BuildContext context) {
    final media = post.mediaItems;
    final cover = media.isNotEmpty ? media.first : null;
    final bodyImages = media.length > 1 ? media.sublist(1) : const [];
    final title = post.title?.trim();

    return Scaffold(
      appBar: AppBar(title: const Text('文章'), centerTitle: true),
      body: ListView(
        children: [
          if (cover != null && cover.url.isNotEmpty)
            Image.network(
              cover.url,
              fit: BoxFit.cover,
              errorBuilder: (_, __, ___) => const SizedBox.shrink(),
            ),
          Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (title != null && title.isNotEmpty)
                  Text(
                    title,
                    style: const TextStyle(
                      color: AppTheme.textPrimary,
                      fontSize: 22,
                      fontWeight: FontWeight.w700,
                      height: 1.35,
                    ),
                  ),
                const SizedBox(height: 8),
                Text(
                  post.author.title,
                  style: const TextStyle(
                    color: AppTheme.textTertiary,
                    fontSize: 13,
                  ),
                ),
                const SizedBox(height: 16),
                if (post.text.trim().isNotEmpty)
                  Text(
                    post.text.trim(),
                    style: const TextStyle(
                      color: AppTheme.textPrimary,
                      fontSize: 16,
                      height: 1.7,
                    ),
                  ),
                for (final image in bodyImages)
                  if (image.url.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.only(top: 12),
                      child: ClipRRect(
                        borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                        child: Image.network(
                          image.url,
                          fit: BoxFit.cover,
                          errorBuilder: (_, __, ___) => const SizedBox.shrink(),
                        ),
                      ),
                    ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
