import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/widgets/square_media_grid.dart';
import 'package:citizenapp/8964/widgets/square_post_actions.dart';
import 'package:citizenapp/8964/widgets/square_post_header.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 广场图片/视频动态卡（含竞选变体）。
///
/// 版式：作者头部 → 正文 + 媒体 → 互动栏。媒体按横竖屏 + 数量出图：
/// - 竖屏单图/单视频：左媒体（约 40%，3:4）+ 右正文；
/// - 其余：正文在上、下面走 [SquareMediaGrid]（横屏 16:9 单块 / 2 图 / 3 图以上前两张）。
class SquarePostCard extends StatelessWidget {
  const SquarePostCard({
    super.key,
    required this.post,
    this.onTap,
    this.onAuthorTap,
    this.avatarUrl,
    this.avatarHeaders,
  });

  final SquarePost post;
  final VoidCallback? onTap;

  /// 点击作者头像/名进入其用户主页。
  final VoidCallback? onAuthorTap;

  /// 作者真头像地址与鉴权头（由页面据 avatarObjectKey + session 生成）。
  final String? avatarUrl;
  final Map<String, String>? avatarHeaders;

  /// 竖屏单图/单视频走"左媒体右文字"布局。
  bool get _isPortraitSingle =>
      post.mediaItems.length == 1 && post.mediaItems.first.isPortrait;

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
              SquarePostHeader(
                post: post,
                onAuthorTap: onAuthorTap,
                avatarUrl: avatarUrl,
                avatarHeaders: avatarHeaders,
              ),
              _buildBody(),
              const SizedBox(height: 12),
              const SquarePostActions(),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildBody() {
    final media = post.mediaItems;
    final caption = post.text;

    if (media.isEmpty) {
      if (caption.isEmpty) return const SizedBox.shrink();
      return Padding(
        padding: const EdgeInsets.only(top: 12),
        child: _caption(caption),
      );
    }

    if (_isPortraitSingle) {
      return Padding(
        padding: const EdgeInsets.only(top: 12),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              flex: 2,
              child: AspectRatio(
                aspectRatio: 3 / 4,
                child: SquareMediaTile(
                  item: media.first,
                  radius: BorderRadius.circular(AppTheme.radiusMd),
                ),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              flex: 3,
              child: caption.isEmpty
                  ? const SizedBox.shrink()
                  : _caption(caption),
            ),
          ],
        ),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (caption.isNotEmpty) ...[
          const SizedBox(height: 12),
          _caption(caption),
        ],
        const SizedBox(height: 12),
        SquareMediaGrid(mediaItems: media),
      ],
    );
  }

  Widget _caption(String text) => Text(
        text,
        style: const TextStyle(
          color: AppTheme.textPrimary,
          fontSize: 15,
          height: 1.45,
        ),
      );
}
