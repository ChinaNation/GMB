import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/identity_badge.dart';

IdentityBadgeStyle _style(String? identity, String? membership,
    {bool active = true}) {
  return identityBadgeStyle(
    identityLevel: identity,
    membershipLevel: membership,
    membershipActive: active,
  )!;
}

void main() {
  group('identityBadgeStyle 会员档位徽章', () {
    test('有效会员按自由金 / 民主蓝 / 薪火红显示并带白色对勾', () {
      final freedom = _style('candidate', 'freedom');
      expect(freedom.color, AppTheme.identityVisitor);
      expect(freedom.checked, isTrue);
      expect(freedom.checkColor, Colors.white);

      final democracy = _style('visitor', 'democracy');
      expect(democracy.color, AppTheme.identityVoting);
      expect(democracy.checked, isTrue);

      final spark = _style('voting', 'spark');
      expect(spark.color, AppTheme.identityCandidate);
      expect(spark.checked, isTrue);
    });

    test('无有效会员时按竞选红 / 投票蓝 / 访客金显示身份小人', () {
      final candidate = _style('candidate', 'spark', active: false);
      expect(candidate.color, AppTheme.identityCandidate);
      expect(candidate.checked, isFalse);

      final voting = _style('voting', null, active: false);
      expect(voting.color, AppTheme.identityVoting);
      expect(voting.checked, isFalse);

      final visitor = _style(null, null, active: false);
      expect(visitor.color, AppTheme.identityVisitor);
      expect(visitor.checked, isFalse);
    });

    test('会员有效态缺少合法档位时失败关闭并回落身份徽章', () {
      final s = _style('candidate', 'unknown');
      expect(s.checked, isFalse);
      expect(s.color, AppTheme.identityCandidate);
    });

    test('提示文案同时说明身份和具体会员档位', () {
      expect(
        identityBadgeLabel(
          identityLevel: 'candidate',
          membershipLevel: 'spark',
          checked: true,
        ),
        '竞选公民 · 薪火会员',
      );
      expect(
        identityBadgeLabel(
          identityLevel: 'voting',
          membershipLevel: null,
          checked: false,
        ),
        '投票公民',
      );
    });
  });
}
