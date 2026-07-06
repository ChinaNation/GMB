import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/widgets/square_media_grid.dart';
import 'package:citizenapp/ui/app_theme.dart';

class SquarePostCard extends StatelessWidget {
  const SquarePostCard({
    super.key,
    required this.post,
    this.onTap,
  });

  final SquarePost post;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    return Card(
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.all(14),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _AuthorRow(post: post),
              if (post.text.isNotEmpty) ...[
                const SizedBox(height: 12),
                Text(
                  post.text,
                  style: const TextStyle(
                    color: AppTheme.textPrimary,
                    fontSize: 15,
                    height: 1.45,
                  ),
                ),
              ],
              if (post.mediaItems.isNotEmpty) ...[
                const SizedBox(height: 12),
                SquareMediaGrid(mediaItems: post.mediaItems),
              ],
              const SizedBox(height: 12),
              const Row(
                children: [
                  Icon(Icons.thumb_up_alt_outlined,
                      size: 18, color: AppTheme.textTertiary),
                  SizedBox(width: 18),
                  Icon(Icons.mode_comment_outlined,
                      size: 18, color: AppTheme.textTertiary),
                  SizedBox(width: 18),
                  Icon(Icons.bookmark_border_rounded,
                      size: 18, color: AppTheme.textTertiary),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _AuthorRow extends StatelessWidget {
  const _AuthorRow({required this.post});

  final SquarePost post;

  @override
  Widget build(BuildContext context) {
    final author = post.author;
    return Row(
      children: [
        CircleAvatar(
          radius: 18,
          backgroundColor: AppTheme.primary.withAlpha(20),
          child: Icon(
            author.isCertified
                ? Icons.verified_user_rounded
                : Icons.account_circle_rounded,
            color: AppTheme.primary,
            size: 20,
          ),
        ),
        const SizedBox(width: 10),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Flexible(
                    child: Text(
                      author.title,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        color: AppTheme.textPrimary,
                        fontSize: 14,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                  ),
                  if (author.isCertified) ...[
                    const SizedBox(width: 6),
                    const Icon(Icons.verified_rounded,
                        size: 15, color: AppTheme.primary),
                  ],
                ],
              ),
              const SizedBox(height: 2),
              Text(
                _formatCreatedAt(post.createdAt),
                style: const TextStyle(
                  color: AppTheme.textTertiary,
                  fontSize: 12,
                ),
              ),
            ],
          ),
        ),
        if (post.postCategory == SquarePostCategory.campaign)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: AppTheme.gold.withAlpha(24),
              borderRadius: BorderRadius.circular(AppTheme.radiusSm),
            ),
            child: const Text(
              '竞选',
              style: TextStyle(
                color: AppTheme.gold,
                fontSize: 12,
                fontWeight: FontWeight.w700,
              ),
            ),
          ),
      ],
    );
  }

  String _formatCreatedAt(DateTime createdAt) {
    final now = DateTime.now();
    final diff = now.difference(createdAt);
    if (diff.inMinutes < 1) return '刚刚';
    if (diff.inHours < 1) return '${diff.inMinutes} 分钟前';
    if (diff.inDays < 1) return '${diff.inHours} 小时前';
    return '${createdAt.year}-${createdAt.month.toString().padLeft(2, '0')}-${createdAt.day.toString().padLeft(2, '0')}';
  }
}
