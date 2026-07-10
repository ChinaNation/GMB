import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';

/// 头像行右上角的动作图标——**对主页主人的操作**，只在看**别人**主页时出现。
///
/// 通知 = 订阅该用户动态、聊天 = 给该用户发消息、关注 = 关注/取关该用户。
/// 看自己主页时不显示任何图标（给自己发消息/关注自己无意义；编辑资料在右上 ⋮ 菜单）。
class ProfileActionIcons extends StatelessWidget {
  const ProfileActionIcons({
    super.key,
    required this.isSelf,
    required this.isFollowing,
    this.onSubscribe,
    this.onChat,
    this.onToggleFollow,
  });

  final bool isSelf;
  final bool isFollowing;

  /// 通知：订阅该用户动态。
  final VoidCallback? onSubscribe;

  /// 聊天：给该用户发消息。
  final VoidCallback? onChat;

  /// 关注：关注/取关该用户。
  final VoidCallback? onToggleFollow;

  @override
  Widget build(BuildContext context) {
    // 自己的主页不显示这些操作。
    if (isSelf) return const SizedBox.shrink();

    final buttons = <Widget>[
      _CircleIcon(
        icon: Icons.notifications_outlined,
        tooltip: '订阅动态',
        onTap: onSubscribe,
      ),
      _CircleIcon(
        icon: Icons.chat_bubble_outline,
        tooltip: '发消息',
        onTap: onChat,
      ),
      _CircleIcon(
        icon: isFollowing ? Icons.how_to_reg : Icons.person_add_alt,
        tooltip: isFollowing ? '已关注' : '关注',
        active: isFollowing,
        onTap: onToggleFollow,
      ),
    ];
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final button in buttons)
          Padding(
            padding: const EdgeInsets.only(left: 8),
            child: button,
          ),
      ],
    );
  }
}

class _CircleIcon extends StatelessWidget {
  const _CircleIcon({
    required this.icon,
    required this.tooltip,
    this.active = false,
    this.onTap,
  });

  final IconData icon;
  final String tooltip;

  /// 激活态（如已关注）用品牌色描边与图标高亮。
  final bool active;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final color = active ? AppTheme.primary : AppTheme.textSecondary;
    return Material(
      color: AppTheme.surfaceWhite,
      shape: CircleBorder(
        side: BorderSide(
          color: active ? AppTheme.primary : AppTheme.border,
          width: active ? 1 : 0.5,
        ),
      ),
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.all(7),
          child: Icon(
            icon,
            color: color,
            size: 20,
            semanticLabel: tooltip,
          ),
        ),
      ),
    );
  }
}
