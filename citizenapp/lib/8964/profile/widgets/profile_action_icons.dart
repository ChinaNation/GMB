import 'package:flutter/material.dart';

/// 头像行右上角的三图标（决策 5：是图标不是按钮）。
///
/// 本人：通知 / 聊天 / 关注（我的关注列表）。
/// 他人：关注(toggle) / 消息。
class ProfileActionIcons extends StatelessWidget {
  const ProfileActionIcons({
    super.key,
    required this.isSelf,
    required this.isFollowing,
    this.onNotifications,
    this.onChat,
    this.onFollowingList,
    this.onToggleFollow,
  });

  final bool isSelf;
  final bool isFollowing;
  final VoidCallback? onNotifications;
  final VoidCallback? onChat;
  final VoidCallback? onFollowingList;
  final VoidCallback? onToggleFollow;

  @override
  Widget build(BuildContext context) {
    final buttons = isSelf
        ? <Widget>[
            _CircleIcon(
              icon: Icons.notifications_outlined,
              tooltip: '通知',
              onTap: onNotifications,
            ),
            _CircleIcon(
              icon: Icons.chat_bubble_outline,
              tooltip: '聊天',
              onTap: onChat,
            ),
            _CircleIcon(
              icon: Icons.people_outline,
              tooltip: '关注',
              onTap: onFollowingList,
            ),
          ]
        : <Widget>[
            _CircleIcon(
              icon: isFollowing ? Icons.how_to_reg : Icons.person_add_alt,
              tooltip: isFollowing ? '已关注' : '关注',
              onTap: onToggleFollow,
            ),
            _CircleIcon(
              icon: Icons.chat_bubble_outline,
              tooltip: '消息',
              onTap: onChat,
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
    this.onTap,
  });

  final IconData icon;
  final String tooltip;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Colors.black.withValues(alpha: 0.22),
      shape: const CircleBorder(),
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.all(7),
          child: Icon(
            icon,
            color: Colors.white,
            size: 20,
            semanticLabel: tooltip,
          ),
        ),
      ),
    );
  }
}
