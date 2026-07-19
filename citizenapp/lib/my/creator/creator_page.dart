import 'package:flutter/material.dart';

import 'package:citizenapp/my/creator/creator_plan_edit_sheet.dart';
import 'package:citizenapp/my/creator/creator_service.dart';
import 'package:citizenapp/my/creator/models/creator_overview.dart';
import 'package:citizenapp/my/creator/models/creator_plan.dart';
import 'package:citizenapp/my/creator/widgets/creator_gate_view.dart';
import 'package:citizenapp/my/creator/widgets/creator_overview_card.dart';
import 'package:citizenapp/my/creator/widgets/creator_tier_card.dart';
import 'package:citizenapp/my/membership/membership_page.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 「我的 → 创作者」：管理自己的创作者会员（档位 / 收入概览）。
///
/// 三态：加载中 / 无当前有效平台会员 / 已开通。档位价格链上保存，名称由 Cloudflare 保存；
/// 整次保存只产生一次 `set_creator_plans` 账户签名。
class CreatorPage extends StatefulWidget {
  const CreatorPage({super.key, CreatorService? service}) : _service = service;

  final CreatorService? _service;

  @override
  State<CreatorPage> createState() => _CreatorPageState();
}

class _CreatorPageState extends State<CreatorPage> {
  late final CreatorService _service = widget._service ?? CreatorService();
  CreatorPageData? _data;
  bool _loading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final data = await _service.load();
      if (!mounted) return;
      setState(() {
        _data = data;
        _loading = false;
      });
    } on CreatorException catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.message;
        _loading = false;
      });
    } on Exception catch (e) {
      if (!mounted) return;
      setState(() {
        _error = '加载失败：$e';
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('创作者')),
      body: _body(),
    );
  }

  Widget _body() {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_error != null) {
      return _errorView(_error!);
    }
    final data = _data!;
    if (data.gated) {
      return CreatorGateView(onOpenMembership: _openMembership);
    }
    return _activeView(data.plan!, data.overview!);
  }

  Widget _activeView(CreatorPlan plan, CreatorOverview overview) {
    final atMax = plan.tiers.length >= CreatorPlan.maxTiers;
    return RefreshIndicator(
      onRefresh: _load,
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 16, 16, 24),
        children: [
          CreatorOverviewCard(overview: overview),
          const SizedBox(height: 16),
          if (plan.tiers.isEmpty)
            _emptyTiers()
          else ...[
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 2),
              child: Row(
                children: [
                  const Text(
                    '我的会员档',
                    style: TextStyle(
                      fontSize: 14,
                      fontWeight: FontWeight.w600,
                      color: AppTheme.textPrimary,
                    ),
                  ),
                  const Spacer(),
                  Text(
                    '${plan.tiers.length} / ${CreatorPlan.maxTiers}',
                    style: const TextStyle(
                        fontSize: 12, color: AppTheme.textTertiary),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 12),
            for (final tier in plan.tiers) ...[
              CreatorTierCard(tier: tier, onEdit: () => _openEdit(tier)),
              const SizedBox(height: 12),
            ],
            _addTierButton(atMax),
          ],
          const SizedBox(height: 16),
          _subscribersEntry(overview.subscriberCount),
          const SizedBox(height: 14),
          const Center(
            child: Text(
              '价格以公民币结算 · 订阅款全额进你的钱包 · 保存只签名一次',
              textAlign: TextAlign.center,
              style: TextStyle(fontSize: 11, color: AppTheme.textTertiary),
            ),
          ),
        ],
      ),
    );
  }

  Widget _emptyTiers() {
    return Container(
      decoration: AppTheme.cardDecoration(),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        children: [
          Container(
            width: 52,
            height: 52,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: AppTheme.primary.withAlpha(24),
              borderRadius: BorderRadius.circular(AppTheme.radiusMd),
            ),
            child: const Icon(Icons.storefront_outlined,
                size: 26, color: AppTheme.primary),
          ),
          const SizedBox(height: 12),
          const Text('还没有会员档',
              style: TextStyle(
                  fontSize: 15,
                  fontWeight: FontWeight.w600,
                  color: AppTheme.textPrimary)),
          const SizedBox(height: 6),
          const Text('创建第一个会员档，粉丝就能用公民币订阅你。',
              textAlign: TextAlign.center,
              style: TextStyle(
                  fontSize: 13, height: 1.5, color: AppTheme.textSecondary)),
          const SizedBox(height: 16),
          FilledButton.icon(
            onPressed: () => _openEdit(null),
            icon: const Icon(Icons.add, size: 19),
            label: const Text('创建会员档'),
          ),
        ],
      ),
    );
  }

  Widget _addTierButton(bool atMax) {
    return InkWell(
      onTap: atMax ? null : () => _openEdit(null),
      borderRadius: BorderRadius.circular(AppTheme.radiusMd),
      child: Container(
        padding: const EdgeInsets.symmetric(vertical: 12),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(AppTheme.radiusMd),
          border: Border.all(
            color: atMax ? AppTheme.border : AppTheme.primaryLight,
            width: 1.5,
            style: BorderStyle.solid,
          ),
        ),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.add,
                size: 18,
                color: atMax ? AppTheme.textTertiary : AppTheme.primary),
            const SizedBox(width: 6),
            Text(
              atMax ? '已达 ${CreatorPlan.maxTiers} 档上限' : '新增会员档',
              style: TextStyle(
                fontSize: 14,
                fontWeight: FontWeight.w600,
                color: atMax ? AppTheme.textTertiary : AppTheme.primary,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _subscribersEntry(int count) {
    return InkWell(
      onTap: () => ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('订阅者明细即将上线')),
      ),
      borderRadius: BorderRadius.circular(AppTheme.radiusMd),
      child: Container(
        decoration: AppTheme.cardDecoration(),
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 13),
        child: Row(
          children: [
            Container(
              width: 30,
              height: 30,
              alignment: Alignment.center,
              decoration: BoxDecoration(
                color: AppTheme.info.withAlpha(24),
                borderRadius: BorderRadius.circular(AppTheme.radiusSm),
              ),
              child: const Icon(Icons.group_outlined,
                  size: 17, color: AppTheme.info),
            ),
            const SizedBox(width: 10),
            const Expanded(
              child: Text('谁订阅了我',
                  style: TextStyle(
                      fontSize: 14,
                      fontWeight: FontWeight.w600,
                      color: AppTheme.textPrimary)),
            ),
            Text('$count 位',
                style: const TextStyle(
                    fontSize: 13, color: AppTheme.textSecondary)),
            const SizedBox(width: 4),
            const Icon(Icons.chevron_right,
                size: 20, color: AppTheme.textTertiary),
          ],
        ),
      ),
    );
  }

  Widget _errorView(String message) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.error_outline,
                size: 40, color: AppTheme.textTertiary),
            const SizedBox(height: 12),
            Text(message,
                textAlign: TextAlign.center,
                style: const TextStyle(
                    fontSize: 14, color: AppTheme.textSecondary)),
            const SizedBox(height: 16),
            OutlinedButton(onPressed: _load, child: const Text('重试')),
          ],
        ),
      ),
    );
  }

  Future<void> _openEdit(CreatorTier? tier) async {
    final plan = await showModalBottomSheet<CreatorPlan>(
      context: context,
      isScrollControlled: true,
      builder: (_) => CreatorPlanEditSheet(
        service: _service,
        currentTiers: _data?.plan?.tiers ?? const [],
        editing: tier,
      ),
    );
    if (plan != null && mounted) {
      await _load();
    }
  }

  void _openMembership() {
    Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => const MembershipPage()),
    );
  }
}
