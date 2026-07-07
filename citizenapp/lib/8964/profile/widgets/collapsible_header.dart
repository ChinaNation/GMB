import 'dart:ui';

import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';

/// 可折叠主页头部：随折叠比例把背景图渐进虚化，资料主体淡出，折叠满时浮现居中标题。
///
/// 虚化只作用于背景单图层（[ImageFiltered]），不使用全屏 [BackdropFilter]，
/// 避免每帧重算整屏带来的性能开销。
class CollapsibleHeader extends StatelessWidget {
  const CollapsibleHeader({
    super.key,
    required this.expandedHeight,
    required this.foreground,
    required this.collapsedTitle,
    this.banner,
    this.bottomInset = 0,
  });

  /// SliverAppBar 的 expandedHeight（不含状态栏），用于换算折叠比例。
  final double expandedHeight;

  /// 展开态资料主体（头像/名/计数等），随折叠淡出。
  final Widget foreground;

  /// 折叠满时浮现的居中标题。
  final String collapsedTitle;

  /// 背景图层；为空时用主题渐变占位（阶段 6 接入真实背景图）。
  final Widget? banner;

  /// 底部固定区域高度（分类 TabBar），资料主体在其上方，避免被遮挡。
  final double bottomInset;

  static const double _maxBlurSigma = 18;

  @override
  Widget build(BuildContext context) {
    final topPadding = MediaQuery.of(context).padding.top;
    final minHeight = kToolbarHeight + topPadding;
    final maxHeight = expandedHeight + topPadding;

    return LayoutBuilder(
      builder: (context, constraints) {
        final span = (maxHeight - minHeight).clamp(1.0, double.infinity);
        final collapse =
            ((maxHeight - constraints.maxHeight) / span).clamp(0.0, 1.0);
        return Stack(
          fit: StackFit.expand,
          children: [
            ImageFiltered(
              imageFilter: ImageFilter.blur(
                sigmaX: _maxBlurSigma * collapse,
                sigmaY: _maxBlurSigma * collapse,
                tileMode: TileMode.decal,
              ),
              // 渐变作底：背景图为空或加载失败时透出，避免出现纯色/空洞。
              child: Stack(
                fit: StackFit.expand,
                children: [
                  const _GradientBanner(),
                  if (banner != null) Positioned.fill(child: banner!),
                ],
              ),
            ),
            // 压暗层保持锐利（不进模糊），保证白字可读。
            const DecoratedBox(
              decoration: BoxDecoration(
                gradient: LinearGradient(
                  begin: Alignment.topCenter,
                  end: Alignment.bottomCenter,
                  colors: [Color(0x14000000), Color(0x40000000)],
                ),
              ),
            ),
            Positioned(
              top: topPadding,
              left: 56,
              right: 56,
              height: kToolbarHeight,
              child: Opacity(
                opacity: collapse,
                child: Center(
                  child: Text(
                    collapsedTitle,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                      color: Colors.white,
                      fontSize: 16,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
              ),
            ),
            Positioned(
              left: 0,
              right: 0,
              bottom: bottomInset,
              child: Opacity(
                opacity: (1 - collapse).clamp(0.0, 1.0),
                child: IgnorePointer(
                  ignoring: collapse > 0.5,
                  child: foreground,
                ),
              ),
            ),
          ],
        );
      },
    );
  }
}

class _GradientBanner extends StatelessWidget {
  const _GradientBanner();

  @override
  Widget build(BuildContext context) {
    return const DecoratedBox(
      decoration: BoxDecoration(
        gradient: LinearGradient(
          colors: [
            AppTheme.primaryDark,
            AppTheme.primary,
            AppTheme.primaryLight,
          ],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
      ),
    );
  }
}
