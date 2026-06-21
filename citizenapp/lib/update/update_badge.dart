import 'package:flutter/material.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 更新提醒红点，只表达“当前仍有可安装更新”。
///
/// 中文注释：红点不承担已读状态；是否显示完全由更新检查结果驱动。
class UpdateDotBadge extends StatelessWidget {
  const UpdateDotBadge({
    super.key,
    required this.show,
    required this.child,
    this.dotKey,
  });

  final bool show;
  final Widget child;
  final Key? dotKey;

  @override
  Widget build(BuildContext context) {
    if (!show) {
      return child;
    }

    return Stack(
      clipBehavior: Clip.none,
      children: [
        child,
        Positioned(
          top: -2,
          right: -2,
          child: Container(
            key: dotKey,
            width: 8,
            height: 8,
            decoration: BoxDecoration(
              color: AppTheme.danger,
              shape: BoxShape.circle,
              border: Border.all(color: AppTheme.surfaceWhite, width: 1.5),
            ),
          ),
        ),
      ],
    );
  }
}
