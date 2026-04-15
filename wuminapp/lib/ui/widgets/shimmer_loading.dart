import 'package:flutter/material.dart';

import '../app_theme.dart';

// ---------------------------------------------------------------------------
// ShimmerEffect - 为子组件添加从左到右的光泽扫过动画
// ---------------------------------------------------------------------------

/// 通用 shimmer 包装器。
///
/// 在 [child] 上叠加一个 [LinearGradient] 动画遮罩，模拟骨架屏加载效果。
/// 不依赖任何第三方包，仅使用 Flutter 内置动画 API。
class ShimmerEffect extends StatefulWidget {
  const ShimmerEffect({super.key, required this.child});

  final Widget child;

  @override
  State<ShimmerEffect> createState() => _ShimmerEffectState();
}

class _ShimmerEffectState extends State<ShimmerEffect>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1500),
    )..repeat();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        final value = _controller.value;
        // 渐变从 -1.0 滑到 2.0，产生扫过效果
        final begin = Alignment(-1.0 + 3.0 * value, -0.3);
        final end = Alignment(-1.0 + 3.0 * value + 1.0, 0.3);

        return ShaderMask(
          blendMode: BlendMode.srcATop,
          shaderCallback: (bounds) {
            return LinearGradient(
              begin: begin,
              end: end,
              colors: const [
                AppTheme.surfaceMuted,
                AppTheme.surfaceWhite,
                AppTheme.surfaceMuted,
              ],
              stops: const [0.0, 0.5, 1.0],
            ).createShader(bounds);
          },
          child: child,
        );
      },
      child: widget.child,
    );
  }
}

// ---------------------------------------------------------------------------
// ShimmerBox - 圆角矩形占位块
// ---------------------------------------------------------------------------

/// 可配置宽高和圆角的灰色占位块，配合 [ShimmerEffect] 使用。
class ShimmerBox extends StatelessWidget {
  const ShimmerBox({
    super.key,
    this.width,
    this.height = 16,
    this.radius = 6,
  });

  final double? width;
  final double height;
  final double radius;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: width,
      height: height,
      decoration: BoxDecoration(
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(radius),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// ProposalCardSkeleton - 模拟提案卡片布局的骨架
// ---------------------------------------------------------------------------

/// 提案卡片骨架屏：左侧图标占位 + 两行文本 + 右侧状态徽章。
class ProposalCardSkeleton extends StatelessWidget {
  const ProposalCardSkeleton({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.border),
      ),
      child: const Row(
        children: [
          // 图标占位
          ShimmerBox(width: 36, height: 36, radius: 10),
          SizedBox(width: 12),
          // 文本行
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                ShimmerBox(width: 120, height: 14),
                SizedBox(height: 6),
                ShimmerBox(width: 180, height: 12),
              ],
            ),
          ),
          SizedBox(width: 8),
          // 状态徽章占位
          ShimmerBox(width: 52, height: 20, radius: 10),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// WalletCardSkeleton - 模拟钱包列表卡片的骨架
// ---------------------------------------------------------------------------

/// 钱包卡片骨架屏：左侧头像占位 + 名称行 + 地址行。
class WalletCardSkeleton extends StatelessWidget {
  const WalletCardSkeleton({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.border),
      ),
      child: const Row(
        children: [
          // 头像占位
          ShimmerBox(width: 40, height: 40, radius: 20),
          SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                ShimmerBox(width: 100, height: 14),
                SizedBox(height: 6),
                ShimmerBox(height: 12),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// ListSkeleton - 批量骨架列表
// ---------------------------------------------------------------------------

/// 将 [itemCount] 个骨架项包裹在 [ShimmerEffect] 中。
///
/// [builder] 用于自定义每个骨架项的 widget（默认无需提供）。
class ListSkeleton extends StatelessWidget {
  const ListSkeleton({
    super.key,
    required this.itemCount,
    required this.itemBuilder,
  });

  final int itemCount;
  final IndexedWidgetBuilder itemBuilder;

  @override
  Widget build(BuildContext context) {
    return ShimmerEffect(
      child: ListView.separated(
        physics: const NeverScrollableScrollPhysics(),
        shrinkWrap: true,
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
        itemCount: itemCount,
        separatorBuilder: (_, __) => const SizedBox(height: 8),
        itemBuilder: itemBuilder,
      ),
    );
  }
}
