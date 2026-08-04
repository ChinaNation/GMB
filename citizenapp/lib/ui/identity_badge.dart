import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';

/// 公民徽章样式：无有效会员时展示身份档，有有效会员时展示会员档位。
///
/// 身份档配色：访客金 / 投票蓝 / 竞选红，中心为小人；会员档位配色：自由金 / 民主蓝 /
/// 薪火红，中心为白色对勾。徽章只负责展示，身份和会员的业务资格仍按 ADR-036 独立校验。
class IdentityBadgeStyle {
  const IdentityBadgeStyle({
    required this.color,
    required this.checked,
    this.checkColor = Colors.white,
  });

  /// 扇贝底色：无有效会员取身份档，有有效会员取会员档位。
  final Color color;

  /// true=有生效会员→显示对勾；false=只有身份/纯访客→显示小人。
  final bool checked;

  /// 对勾颜色固定白色；字段保留给绘制器统一使用。
  final Color checkColor;
}

/// 链上身份档位对应颜色。
Color _identityTierColor(String? level) => switch (level) {
      'candidate' => AppTheme.identityCandidate,
      'voting' => AppTheme.identityVoting,
      _ => AppTheme.identityVisitor,
    };

/// 有效会员档位对应颜色；未知档位返回 null，避免把异常数据误展示为有效会员。
Color? _membershipTierColor(String? level) => switch (level) {
      'freedom' => AppTheme.identityVisitor,
      'democracy' => AppTheme.identityVoting,
      'spark' => AppTheme.identityCandidate,
      _ => null,
    };

/// 计算徽章样式。会员必须同时满足有效态和合法档位才显示会员徽章，否则回落身份徽章。
IdentityBadgeStyle? identityBadgeStyle({
  required String? identityLevel,
  required String? membershipLevel,
  required bool membershipActive,
}) {
  final membershipColor =
      membershipActive ? _membershipTierColor(membershipLevel) : null;
  return IdentityBadgeStyle(
    color: membershipColor ?? _identityTierColor(identityLevel),
    checked: membershipColor != null,
    checkColor: Colors.white,
  );
}

/// 徽章无障碍/提示文案。
String identityBadgeLabel({
  required String? identityLevel,
  required String? membershipLevel,
  required bool checked,
}) {
  final base = switch (identityLevel) {
    'candidate' => '竞选公民',
    'voting' => '投票公民',
    _ => '访客',
  };
  final membership = switch (membershipLevel) {
    'freedom' => '自由会员',
    'democracy' => '民主会员',
    'spark' => '薪火会员',
    _ => null,
  };
  return checked && membership != null ? '$base · $membership' : base;
}

/// 推特式扇贝勋章徽章（四处认证展示点共用）：
/// 底为身份档或会员档位色，中心 checked=白色对勾（有效会员）/ 否则=白色小人（身份）。
class IdentityBadge extends StatelessWidget {
  const IdentityBadge({
    super.key,
    required this.style,
    this.size = 24,
    this.tooltip = '',
  });

  final IdentityBadgeStyle style;
  final double size;
  final String tooltip;

  @override
  Widget build(BuildContext context) {
    final badge = SizedBox(
      width: size,
      height: size,
      child: CustomPaint(
        painter: _RosetteBadgePainter(
          color: style.color,
          checked: style.checked,
          checkColor: style.checkColor,
        ),
      ),
    );
    if (tooltip.isEmpty) return badge;
    return Tooltip(message: tooltip, child: badge);
  }
}

class _RosetteBadgePainter extends CustomPainter {
  _RosetteBadgePainter({
    required this.color,
    required this.checked,
    required this.checkColor,
  });

  final Color color;
  final bool checked;
  final Color checkColor;

  // 8 个花瓣圆心（24 网格坐标），围绕中心圆构成扇贝勋章。
  static const List<Offset> _bumps = [
    Offset(18, 12),
    Offset(16.24, 16.24),
    Offset(12, 18),
    Offset(7.76, 16.24),
    Offset(6, 12),
    Offset(7.76, 7.76),
    Offset(12, 6),
    Offset(16.24, 7.76),
  ];

  @override
  void paint(Canvas canvas, Size size) {
    final scale = size.width / 24.0;
    Offset p(double x, double y) => Offset(x * scale, y * scale);
    final center = p(12, 12);

    final fill = Paint()
      ..color = color
      ..isAntiAlias = true;
    for (final bump in _bumps) {
      canvas.drawCircle(p(bump.dx, bump.dy), 4.3 * scale, fill);
    }
    canvas.drawCircle(center, 7.6 * scale, fill);

    if (checked) {
      final stroke = Paint()
        ..color = checkColor
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.2 * scale
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round
        ..isAntiAlias = true;
      final path = Path()
        ..moveTo(8.3 * scale, 12.2 * scale)
        ..lineTo(10.9 * scale, 14.8 * scale)
        ..lineTo(15.8 * scale, 9.4 * scale);
      canvas.drawPath(path, stroke);
    } else {
      final white = Paint()
        ..color = Colors.white
        ..isAntiAlias = true;
      // 小人：头 + 肩。
      canvas.drawCircle(p(12, 9.7), 2.3 * scale, white);
      final shoulders = Path()
        ..moveTo(7.7 * scale, 16.4 * scale)
        ..cubicTo(7.7 * scale, 14.0 * scale, 9.6 * scale, 12.7 * scale,
            12 * scale, 12.7 * scale)
        ..cubicTo(14.4 * scale, 12.7 * scale, 16.3 * scale, 14.0 * scale,
            16.3 * scale, 16.4 * scale)
        ..close();
      canvas.drawPath(shoulders, white);
    }
  }

  @override
  bool shouldRepaint(covariant _RosetteBadgePainter old) =>
      old.color != color ||
      old.checked != checked ||
      old.checkColor != checkColor;
}
