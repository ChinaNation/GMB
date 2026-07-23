import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/profile/widgets/profile_avatar.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 广场卡片统一作者头部：方形圆角头像 + 右下角扇贝身份勋章、昵称、
/// 竞选药丸（仅竞选公民）、竞选岗位/时间、右上角更多按钮。
///
/// 图片/视频/文章及其竞选变体共用同一头部，保证身份表达一致。
class SquarePostHeader extends StatelessWidget {
  const SquarePostHeader({
    super.key,
    required this.post,
    this.onAuthorTap,
    this.onMore,
    this.avatarUrl,
    this.avatarHeaders,
  });

  final SquarePost post;

  /// 点击头像/昵称区进入作者主页。
  final VoidCallback? onAuthorTap;

  /// 右上角更多菜单；为 null 时按钮仍展示但不响应。
  final VoidCallback? onMore;

  /// 作者头像已解析的可读地址（由页面据 avatarObjectKey + session 生成）；
  /// 缺失或读取失败时使用统一的本地默认照片。
  final String? avatarUrl;

  /// 头像 `Image.network` 的鉴权头（钱包 session Bearer）。
  final Map<String, String>? avatarHeaders;

  bool get _isCampaign => post.postCategory == SquarePostCategory.campaign;

  @override
  Widget build(BuildContext context) {
    final author = post.author;
    return Row(
      // start 对齐让更多按钮贴近卡片上边缘。
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          child: GestureDetector(
            onTap: onAuthorTap,
            behavior: HitTestBehavior.opaque,
            child: Row(
              children: [
                ProfileAvatar(
                  seed: author.accountId,
                  size: 40,
                  imageUrl: avatarUrl,
                  imageHeaders: avatarHeaders,
                  identityLevel: author.identityLevel,
                  membershipLevel: author.membershipLevel,
                  membershipActive: author.membershipActive,
                  borderRadius: 12,
                ),
                const SizedBox(width: 11),
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
                                fontWeight: FontWeight.w600,
                              ),
                            ),
                          ),
                          // 药丸只有竞选公民显示；投票/访客靠头像勋章表达。
                          if (_isCampaign) ...[
                            const SizedBox(width: 6),
                            const _CampaignPill(),
                          ],
                        ],
                      ),
                      const SizedBox(height: 2),
                      Text(
                        _subtitle(),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          color: AppTheme.textTertiary,
                          fontSize: 12,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
        IconButton(
          onPressed: onMore,
          icon: const Icon(Icons.more_horiz,
              size: 20, color: AppTheme.textTertiary),
          padding: EdgeInsets.zero,
          constraints: const BoxConstraints(minWidth: 28, minHeight: 20),
          visualDensity: VisualDensity.compact,
          tooltip: '更多',
        ),
      ],
    );
  }

  /// 竞选岗位只有竞选公民且有值时展示，否则只显示时间。
  String _subtitle() {
    final time = _formatCreatedAt(post.createdAt);
    final position = post.campaignPosition?.trim();
    if (_isCampaign && position != null && position.isNotEmpty) {
      return '$position · $time';
    }
    return time;
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

class _CampaignPill extends StatelessWidget {
  const _CampaignPill();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 1),
      decoration: BoxDecoration(
        color: AppTheme.danger.withAlpha(0x1F),
        borderRadius: BorderRadius.circular(20),
      ),
      child: const Text(
        '竞选',
        style: TextStyle(
          color: AppTheme.danger,
          fontSize: 11,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}
