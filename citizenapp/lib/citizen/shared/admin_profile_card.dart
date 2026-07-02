import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/citizen/shared/admin_profile.dart';
import 'package:citizenapp/my/util/amount_format.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 统一管理员资料卡片。
///
/// 中文注释：字段标签固定显示；链上资料缺少某个字段值时只留空值，不隐藏
/// “姓名 / 职务 / 任期 / 来源 / 身份CID / 账户 / 余额”这些 UI 标签。
class AdminProfileCard extends StatelessWidget {
  const AdminProfileCard({
    super.key,
    required this.profile,
    this.index,
    this.trailing,
    this.balanceYuan,
    this.backgroundColor,
    this.borderColor,
    this.compact = false,
  });

  static const double actionHeight = 28;

  final AdminProfile profile;
  final int? index;
  final Widget? trailing;
  final double? balanceYuan;
  final Color? backgroundColor;
  final Color? borderColor;
  final bool compact;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: EdgeInsets.all(compact ? 10 : 12),
      decoration: BoxDecoration(
        color: backgroundColor ?? AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: borderColor ?? AppTheme.border),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              if (index != null) _IndexBadge(index: index!),
              const Spacer(),
              if (trailing != null)
                SizedBox(
                  height: actionHeight,
                  child: Align(
                    alignment: Alignment.centerRight,
                    child: trailing!,
                  ),
                ),
            ],
          ),
          const SizedBox(height: 8),
          _ProfilePairRow(
            left: _ProfileInlineField(label: '姓名', value: profile.name),
            right: _ProfileInlineField(label: '职务', value: profile.adminRole),
          ),
          _ProfilePairRow(
            left: _ProfileInlineField(label: '任期', value: profile.termLabel),
            right: _ProfileInlineField(
              label: '来源',
              value: profile.source == AdminProfileSource.unknown
                  ? ''
                  : profile.source.label,
            ),
          ),
          _ProfileRow(label: '身份CID', value: profile.cidNumber),
          _ProfileRow(
            label: '账户',
            value: _formatAddress(profile.account),
            monospace: true,
          ),
          _ProfileRow(label: '余额', value: _formatBalance(balanceYuan)),
        ],
      ),
    );
  }

  static String _formatBalance(double? value) {
    if (value == null || value.isNaN || value.isInfinite) return '';
    return '${AmountFormat.formatThousands(value)} 元';
  }

  static String _formatAddress(String pubkeyHex) {
    final clean =
        pubkeyHex.startsWith('0x') ? pubkeyHex.substring(2) : pubkeyHex;
    if (clean.isEmpty || clean.length.isOdd) return pubkeyHex;
    try {
      final bytes = Uint8List(clean.length ~/ 2);
      for (var i = 0; i < bytes.length; i++) {
        bytes[i] = int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16);
      }
      return ss58FromAccountId(bytes);
    } on FormatException {
      return pubkeyHex;
    }
  }
}

class _IndexBadge extends StatelessWidget {
  const _IndexBadge({required this.index});

  final int index;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 28,
      height: AdminProfileCard.actionHeight,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        color: AppTheme.primary.withValues(alpha: 0.10),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Text(
        '$index',
        style: const TextStyle(
          fontSize: 12,
          fontWeight: FontWeight.w700,
          color: AppTheme.primary,
        ),
      ),
    );
  }
}

class _ProfilePairRow extends StatelessWidget {
  const _ProfilePairRow({required this.left, required this.right});

  final _ProfileInlineField left;
  final _ProfileInlineField right;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(child: left),
          const SizedBox(width: 10),
          Expanded(child: right),
        ],
      ),
    );
  }
}

class _ProfileInlineField extends StatelessWidget {
  const _ProfileInlineField({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          '$label:',
          softWrap: false,
          style: const TextStyle(
            fontSize: 11,
            color: AppTheme.textTertiary,
            height: 1.35,
          ),
        ),
        const SizedBox(width: 4),
        Expanded(
          child: Text(
            value,
            style: const TextStyle(
              fontSize: 12,
              color: AppTheme.textPrimary,
              height: 1.35,
            ),
            softWrap: true,
          ),
        ),
      ],
    );
  }
}

class _ProfileRow extends StatelessWidget {
  const _ProfileRow({
    required this.label,
    required this.value,
    this.monospace = false,
  });

  final String label;
  final String value;
  final bool monospace;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '$label:',
            softWrap: false,
            style: const TextStyle(
              fontSize: 11,
              color: AppTheme.textTertiary,
              height: 1.35,
            ),
          ),
          const SizedBox(width: 4),
          Expanded(
            child: Text(
              value,
              style: TextStyle(
                fontSize: monospace ? 11 : 12,
                fontFamily: monospace ? 'monospace' : null,
                color: AppTheme.textPrimary,
                height: 1.35,
              ),
              softWrap: true,
            ),
          ),
        ],
      ),
    );
  }
}
