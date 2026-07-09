import 'dart:ui';

import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';

/// 推特式折叠头部：顶部短头图（折叠时渐进虚化）+ 头图下方白底资料区。
///
/// 展开态呈「短头图在上、白底资料在下、头像跨压头图下缘」的推特结构；上滑时
/// 头图渐进虚化、资料区淡出，折叠满时浮现居中标题（分类标签由外层 SliverAppBar
/// 的 bottom 固定吸顶）。虚化只作用于头图单图层（[ImageFiltered]），不使用全屏
/// [BackdropFilter]，避免每帧重算整屏。
class CollapsibleHeader extends StatelessWidget {
  const CollapsibleHeader({
    super.key,
    required this.expandedHeight,
    required this.bannerHeight,
    required this.foreground,
    required this.collapsedTitle,
    this.banner,
  });

  /// SliverAppBar 的 expandedHeight（不含状态栏），用于换算折叠比例。
  final double expandedHeight;

  /// 顶部头图高度（不含状态栏）；资料区从头图下缘开始。
  final double bannerHeight;

  /// 头图下方的白底资料主体（头像/名/计数等），随折叠淡出。
  final Widget foreground;

  /// 折叠满时浮现的居中标题。
  final String collapsedTitle;

  /// 头图图层；为空时纯品牌色平铺。
  final Widget? banner;

  static const double _maxBlurSigma = 18;

  @override
  Widget build(BuildContext context) {
    final topPadding = MediaQuery.of(context).padding.top;
    final minHeight = kToolbarHeight + topPadding;
    final maxHeight = expandedHeight + topPadding;
    final bannerBottom = topPadding + bannerHeight;

    return LayoutBuilder(
      builder: (context, constraints) {
        final span = (maxHeight - minHeight).clamp(1.0, double.infinity);
        final collapse =
            ((maxHeight - constraints.maxHeight) / span).clamp(0.0, 1.0);
        return Stack(
          fit: StackFit.expand,
          children: [
            // 白底铺满，头图下方即为白色资料区。
            const Positioned.fill(
              child: ColoredBox(color: AppTheme.surfaceWhite),
            ),
            // 顶部短头图：折叠时渐进虚化。
            Positioned(
              top: 0,
              left: 0,
              right: 0,
              height: bannerBottom,
              child: ImageFiltered(
                imageFilter: ImageFilter.blur(
                  sigmaX: _maxBlurSigma * collapse,
                  sigmaY: _maxBlurSigma * collapse,
                  tileMode: TileMode.decal,
                ),
                child: Stack(
                  fit: StackFit.expand,
                  children: [
                    // 品牌色平铺兜底：无背景图或加载失败时透出。
                    const ColoredBox(color: AppTheme.primaryDark),
                    if (banner != null) Positioned.fill(child: banner!),
                    // 压暗层保证返回/菜单/标题白字在头图上可读。
                    const DecoratedBox(
                      decoration: BoxDecoration(
                        gradient: LinearGradient(
                          begin: Alignment.topCenter,
                          end: Alignment.bottomCenter,
                          colors: [Color(0x22000000), Color(0x38000000)],
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
            // 折叠满时浮现的居中标题。
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
            // 资料区：头图下方白底，头像跨压头图下缘；随折叠淡出。
            Positioned(
              top: bannerBottom,
              left: 0,
              right: 0,
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
