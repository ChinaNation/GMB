import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';

/// 公民徽章样式：扇贝底色=链上身份档，内符号=会员(勾)/仅身份(小人)。
///
/// 规则（用户定稿）：底色 访客橙 / 投票蓝 / 竞选红；有生效会员→勾，否则→小人。
/// 全体统一显示徽章（含纯访客=橙+小人）。
///
/// 勾色（用户定稿）：默认白；**高档身份买低档会员**（会员档 < 身份档，如竞选公民买
/// 投票会员）时勾染成所买会员档颜色（竞选身份+投票会员=红扇贝+蓝勾）。同档保持白勾。
class IdentityBadgeStyle {
  const IdentityBadgeStyle({
    required this.color,
    required this.checked,
    this.checkColor = Colors.white,
  });

  /// 扇贝底色 = 链上身份档（竞选红 / 投票蓝 / 访客橙）。
  final Color color;

  /// true=有生效会员→显示对勾；false=只有身份/纯访客→显示小人。
  final bool checked;

  /// 对勾颜色：默认白；当**会员档低于身份档**（高档身份买低档会员）时，染成所买
  /// 会员档的颜色以示区分。降档时勾色必与底色异色，一定清晰；同档保持白色。
  final Color checkColor;
}

/// 身份/会员档位序：访客 0 < 投票 1 < 竞选 2。
int _identityTierRank(String? level) => switch (level) {
      'candidate' => 2,
      'voting' => 1,
      _ => 0,
    };

/// 档位对应颜色。
Color _identityTierColor(String? level) => switch (level) {
      'candidate' => AppTheme.identityCandidate,
      'voting' => AppTheme.identityVoting,
      _ => AppTheme.identityVisitor,
    };

/// 计算徽章样式。人人都有徽章，故恒返回非空（返回类型保留可空仅为兼容既有调用点）。
///
/// 底色取身份档；勾色默认白，仅当「降档买会员」（[membershipLevel] 档 < [identityLevel]
/// 档，如竞选公民买投票会员）时染成会员档颜色。
IdentityBadgeStyle? identityBadgeStyle({
  required String? identityLevel,
  required String? membershipLevel,
  required bool membershipActive,
}) {
  final color = _identityTierColor(identityLevel);
  Color checkColor = Colors.white;
  if (membershipActive &&
      _identityTierRank(membershipLevel) < _identityTierRank(identityLevel)) {
    checkColor = _identityTierColor(membershipLevel);
  }
  return IdentityBadgeStyle(
    color: color,
    checked: membershipActive,
    checkColor: checkColor,
  );
}

/// 徽章无障碍/提示文案。
String identityBadgeLabel({
  required String? identityLevel,
  required bool checked,
}) {
  final base = switch (identityLevel) {
    'candidate' => '竞选公民',
    'voting' => '投票公民',
    _ => '访客',
  };
  return checked ? '$base · 会员' : base;
}

/// 推特式扇贝勋章徽章（四处认证展示点共用）：
/// 底为身份色扇贝勋章，中心 checked=白色对勾（有会员）/ 否则=白色小人（仅身份）。
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
      old.color != color || old.checked != checked || old.checkColor != checkColor;
}
