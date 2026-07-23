import 'package:flutter/material.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/creator/creator_money.dart'
    show fenToYuanMoneyLabel;
import 'package:citizenapp/ui/app_theme.dart';

/// 会员详情页：展开某一档（自由/民主/薪火）的**完整权益**（聊天 / 动态 / 文章 / 每月额度）。
///
/// 数据全部取自 [SquareMembershipPlan]（后端下发或本地兜底，单源），价格取自链上（[priceFen]）。
/// 订阅动作不在本页实现：点击底部按钮返回会员页并触发原订阅流程（[onSubscribe]），
/// 避免把签名/上链逻辑复制到详情页。
class MembershipDetailPage extends StatelessWidget {
  const MembershipDetailPage({
    super.key,
    required this.plan,
    required this.priceFen,
    required this.actionLabel,
    required this.subscribeEnabled,
    required this.onSubscribe,
  });

  final SquareMembershipPlan plan;

  /// 本档链上月价（分）；null=链上未设该档价，显示占位「—」。
  final int? priceFen;
  final String actionLabel;
  final bool subscribeEnabled;
  final VoidCallback onSubscribe;

  @override
  Widget build(BuildContext context) {
    final level = plan.membershipLevel;
    final tierColor = _tierColor(level);
    final onTier = _onTierColor(level);
    return Scaffold(
      appBar: AppBar(title: const Text('会员详情'), centerTitle: true),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 16, 16, 24),
        children: [
          _header(tierColor, onTier),
          const SizedBox(height: 16),
          _section(tierColor, Icons.chat_bubble_outline, '聊天', [
            _row('单文件上限', '每个 ${plan.chatFileSizeLabel}'),
            _row('大文件中转（>100MB）', plan.supportsLargeFileRelay ? '支持' : '不支持'),
          ]),
          const SizedBox(height: 12),
          _section(tierColor, Icons.dynamic_feed_outlined, '动态', [
            _row('文字', '最多 ${plan.dynamicTextMaxChars} 字'),
            _row('图片',
                '${plan.dynamicMaxImages} 张 · ${plan.dynamicImageQualityLabel}'),
            _row('视频',
                '${plan.dynamicMaxVideos} 个 · ${plan.dynamicVideoDurationLabel} · ${plan.dynamicVideoQualityLabel}'),
            _row('单视频体积', '最大 ${plan.dynamicVideoBytesLabel}'),
          ]),
          const SizedBox(height: 12),
          _section(tierColor, Icons.article_outlined, '文章', [
            _row('正文', '最多 ${_wan(plan.articleBodyMaxChars)}'),
            _row('配图',
                '${plan.articleMaxImages} 张 · ${plan.articleImageQualityLabel}'),
            _row('首图 · 标题',
                '${plan.articleCoverQualityLabel} · ${plan.articleTitleMinChars}–${plan.articleTitleMaxChars} 字'),
          ]),
          const SizedBox(height: 12),
          _section(tierColor, Icons.calendar_today_outlined, '每月额度', [
            _row('图片上传', '${_thousands(plan.monthlyImages)} 张'),
            _row('视频上传', plan.monthlyVideoDurationLabel),
            _row('并发上传', '${plan.activeUploads} 个'),
          ]),
          const SizedBox(height: 20),
          _subscribeButton(context, tierColor, onTier),
        ],
      ),
    );
  }

  Widget _header(Color tierColor, Color onTier) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: tierColor,
        borderRadius: BorderRadius.circular(14),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  '会员订阅',
                  style: TextStyle(
                    color: onTier.withValues(alpha: 0.82),
                    fontSize: 12,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  plan.displayName,
                  style: TextStyle(
                    color: onTier,
                    fontSize: 24,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(width: 12),
          Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                priceFen == null ? '—' : fenToYuanMoneyLabel(priceFen!),
                style: TextStyle(
                  color: onTier,
                  fontSize: 18,
                  fontWeight: FontWeight.w800,
                ),
              ),
              if (priceFen != null)
                Text(
                  '/月',
                  style: TextStyle(
                    color: onTier.withValues(alpha: 0.82),
                    fontSize: 11,
                  ),
                ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _section(
    Color tierColor,
    IconData icon,
    String title,
    List<Widget> rows,
  ) {
    return Container(
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: AppTheme.border),
      ),
      padding: const EdgeInsets.fromLTRB(14, 10, 14, 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 6),
            child: Row(
              children: [
                Icon(icon, size: 16, color: tierColor),
                const SizedBox(width: 6),
                Text(
                  title,
                  style: TextStyle(
                    color: tierColor,
                    fontSize: 13,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ],
            ),
          ),
          for (final row in rows) ...[
            const Divider(height: 1, color: AppTheme.border),
            row,
          ],
        ],
      ),
    );
  }

  Widget _row(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 9),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
            style: const TextStyle(
              color: AppTheme.textSecondary,
              fontSize: 14,
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              value,
              textAlign: TextAlign.right,
              style: const TextStyle(
                color: AppTheme.textPrimary,
                fontSize: 14,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _subscribeButton(BuildContext context, Color tierColor, Color onTier) {
    final label = priceFen == null
        ? actionLabel
        : '$actionLabel · ${fenToYuanMoneyLabel(priceFen!)}/月';
    return Material(
      color: subscribeEnabled ? tierColor : AppTheme.border,
      borderRadius: BorderRadius.circular(12),
      child: InkWell(
        borderRadius: BorderRadius.circular(12),
        onTap: subscribeEnabled
            ? () {
                // 订阅流程在会员页：先返回,再触发原 _handleAction。
                Navigator.of(context).pop();
                onSubscribe();
              }
            : null,
        child: Padding(
          padding: const EdgeInsets.symmetric(vertical: 14),
          child: Center(
            child: Text(
              label,
              style: TextStyle(
                color: subscribeEnabled ? onTier : AppTheme.textTertiary,
                fontSize: 15,
                fontWeight: FontWeight.w700,
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// 会员档色（自由金 / 民主蓝 / 薪火红），与会员卡片一致。
Color _tierColor(String level) => switch (level) {
      'spark' => AppTheme.identityCandidate,
      'democracy' => AppTheme.identityVoting,
      _ => AppTheme.identityVisitor,
    };

/// 顶带前景色：自由金底用深棕，其余用白字。
Color _onTierColor(String level) =>
    level == 'freedom' ? const Color(0xFF4A3000) : Colors.white;

/// 整数千分号：5000 → "5,000"。
String _thousands(int n) =>
    n.toString().replaceAllMapped(RegExp(r'\B(?=(\d{3})+$)'), (_) => ',');

/// 字数「万」简写：30000 → "3 万字"；非整万回退千分号原值。
String _wan(int chars) => chars >= 10000 && chars % 10000 == 0
    ? '${chars ~/ 10000} 万字'
    : '${_thousands(chars)} 字';
