import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/identity_badge.dart';

IdentityBadgeStyle _style(String? identity, String? membership, {bool active = true}) {
  return identityBadgeStyle(
    identityLevel: identity,
    membershipLevel: membership,
    membershipActive: active,
  )!;
}

void main() {
  group('identityBadgeStyle 底色 = 身份档', () {
    test('竞选=红 / 投票=蓝 / 访客(空或 visitor)=橙', () {
      expect(_style('candidate', 'candidate').color, AppTheme.identityCandidate);
      expect(_style('voting', 'voting').color, AppTheme.identityVoting);
      expect(_style('visitor', 'visitor').color, AppTheme.identityVisitor);
      expect(_style(null, 'visitor').color, AppTheme.identityVisitor);
    });
  });

  group('勾色规则', () {
    test('同档买会员 → 勾保持白色', () {
      expect(_style('candidate', 'candidate').checkColor, Colors.white);
      expect(_style('voting', 'voting').checkColor, Colors.white);
      expect(_style('visitor', 'visitor').checkColor, Colors.white);
    });

    test('传入低于身份的会员档：底色仍随身份、勾恒白（精确匹配已无降档）', () {
      // exact-match 禁降档后不存在「身份高于会员」的真实组合；即便被喂入，
      // 底色只取身份档，勾色统一白（不再按会员档着色）。
      final s1 = _style('candidate', 'voting');
      expect(s1.color, AppTheme.identityCandidate);
      expect(s1.checkColor, Colors.white);
      final s2 = _style('voting', 'visitor');
      expect(s2.color, AppTheme.identityVoting);
      expect(s2.checkColor, Colors.white);
    });

    test('无生效会员 → 小人(checked=false)，勾色不参与', () {
      final s = _style('candidate', null, active: false);
      expect(s.checked, isFalse);
      expect(s.color, AppTheme.identityCandidate);
    });
  });
}
