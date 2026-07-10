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

    test('竞选身份 + 投票会员 → 红扇贝 + 蓝勾', () {
      final s = _style('candidate', 'voting');
      expect(s.color, AppTheme.identityCandidate);
      expect(s.checkColor, AppTheme.identityVoting);
    });

    test('竞选身份 + 访客会员 → 橙勾', () {
      expect(_style('candidate', 'visitor').checkColor, AppTheme.identityVisitor);
    });

    test('投票身份 + 访客会员 → 蓝扇贝 + 橙勾', () {
      final s = _style('voting', 'visitor');
      expect(s.color, AppTheme.identityVoting);
      expect(s.checkColor, AppTheme.identityVisitor);
    });

    test('无生效会员 → 小人(checked=false)，勾色不参与', () {
      final s = _style('candidate', null, active: false);
      expect(s.checked, isFalse);
      expect(s.color, AppTheme.identityCandidate);
    });
  });
}
