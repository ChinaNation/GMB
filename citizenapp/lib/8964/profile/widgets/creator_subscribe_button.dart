import 'package:flutter/material.dart';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/subscribe/creator_subscribe_service.dart';
import 'package:citizenapp/my/creator/creator_api.dart';
import 'package:citizenapp/my/creator/creator_money.dart';
import 'package:citizenapp/my/creator/models/creator_plan.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 广场他人主页「订阅 TA / 取消」按钮（订阅者侧）。
///
/// 有档才显示；双态：未订阅→「订阅 TA」（选档/周期→上链热签+生物识别）、已订阅→「已订阅·取消」
/// （二次确认→上链热签+生物识别）。订阅=授权按月扣款、取消=撤销。
class CreatorSubscribeButton extends StatefulWidget {
  const CreatorSubscribeButton({
    super.key,
    required this.creatorAccount,
    CreatorApi? api,
    CreatorSubscribeService? service,
    SquareSessionProvider? sessionProvider,
  })  : _api = api,
        _service = service,
        _sessionProvider = sessionProvider;

  final String creatorAccount;
  final CreatorApi? _api;
  final CreatorSubscribeService? _service;
  final SquareSessionProvider? _sessionProvider;

  @override
  State<CreatorSubscribeButton> createState() => _CreatorSubscribeButtonState();
}

class _CreatorSubscribeButtonState extends State<CreatorSubscribeButton> {
  late final CreatorApi _api = widget._api ?? CreatorApiHttp();
  late final CreatorSubscribeService _service =
      widget._service ?? CreatorSubscribeService();
  late final SquareSessionProvider _session =
      widget._sessionProvider ?? SquareSessionProvider.instance;

  bool _loading = true;
  bool _busy = false;
  CreatorPlan? _plan;
  String? _status;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() => _loading = true);
    try {
      final session = await _session.ensureSession();
      if (session == null) {
        if (mounted) setState(() => _loading = false);
        return;
      }
      final plan = await _api.fetchPlanOf(session, widget.creatorAccount);
      final status =
          await _api.fetchMySubscriptionTo(session, widget.creatorAccount);
      if (!mounted) return;
      setState(() {
        _plan = plan;
        _status = status;
        _loading = false;
      });
    } on Exception {
      if (mounted) setState(() => _loading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    // 未开档 / 加载中不显示，避免空按钮。
    if (_loading || _plan == null || _plan!.tiers.isEmpty) {
      return const SizedBox.shrink();
    }
    final subscribed = _status == 'active';
    if (subscribed) {
      return OutlinedButton.icon(
        onPressed: _busy ? null : _cancel,
        icon: const Icon(Icons.check_circle_outline, size: 18),
        label: const Text('已订阅 · 取消'),
      );
    }
    return FilledButton.icon(
      onPressed: _busy ? null : _openPicker,
      icon: const Icon(Icons.workspace_premium_outlined, size: 18),
      label: const Text('订阅 TA'),
    );
  }

  Future<void> _openPicker() async {
    final selection = await showModalBottomSheet<_TierPeriodSelection>(
      context: context,
      isScrollControlled: true,
      builder: (_) => _TierPeriodPicker(plan: _plan!),
    );
    if (selection == null || !mounted) return;
    await _run(() => _service.subscribe(
          creatorAddress: widget.creatorAccount,
          tierId: selection.tierId,
          period: selection.period.key,
          priceFen: selection.priceFen,
        ));
  }

  Future<void> _cancel() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('取消订阅'),
        content: const Text('取消后区块链不再按月从你的钱包扣款，确定取消？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(false),
            child: const Text('再想想'),
          ),
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(true),
            style: TextButton.styleFrom(foregroundColor: AppTheme.danger),
            child: const Text('取消订阅'),
          ),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;
    await _run(() => _service.cancel(creatorAddress: widget.creatorAccount));
  }

  Future<void> _run(Future<void> Function() action) async {
    setState(() => _busy = true);
    try {
      await action();
      if (!mounted) return;
      await _load();
    } on CreatorSubscribeException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text(e.message)));
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }
}

class _TierPeriodSelection {
  const _TierPeriodSelection(this.tierId, this.period, this.priceFen);
  final String tierId;
  final BillingPeriod period;
  final int priceFen;
}

/// 选档 + 周期底部弹窗：列出每档可用的月/季/年选项，点选即返回。
class _TierPeriodPicker extends StatelessWidget {
  const _TierPeriodPicker({required this.plan});

  final CreatorPlan plan;

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      top: false,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(16, 12, 16, 18),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Center(
              child: Container(
                width: 38,
                height: 4,
                decoration: BoxDecoration(
                  color: AppTheme.border,
                  borderRadius: BorderRadius.circular(4),
                ),
              ),
            ),
            const SizedBox(height: 14),
            const Text(
              '选择会员档与周期',
              style: TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w700,
                color: AppTheme.textPrimary,
              ),
            ),
            const SizedBox(height: 4),
            const Text(
              '订阅后区块链按所选周期自动扣公民币；款项全额进创作者钱包。',
              style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
            ),
            const SizedBox(height: 14),
            for (final tier in plan.tiers) _tierBlock(context, tier),
          ],
        ),
      ),
    );
  }

  Widget _tierBlock(BuildContext context, CreatorTier tier) {
    final periods =
        BillingPeriod.values.where((period) => tier.hasPeriod(period)).toList();
    return Padding(
      padding: const EdgeInsets.only(bottom: 14),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            tier.name.isEmpty ? '未命名档位' : tier.name,
            style: const TextStyle(
              fontSize: 15,
              fontWeight: FontWeight.w600,
              color: AppTheme.textPrimary,
            ),
          ),
          const SizedBox(height: 8),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: periods.map((period) {
              final fen = tier.priceFenOf(period)!;
              return OutlinedButton(
                onPressed: () => Navigator.of(context).pop(
                  _TierPeriodSelection(tier.tierId, period, fen),
                ),
                style: OutlinedButton.styleFrom(
                  minimumSize: const Size(0, 40),
                  padding: const EdgeInsets.symmetric(horizontal: 14),
                ),
                child: Text('${period.label} ${fenToYuanLabel(fen)} 元'),
              );
            }).toList(),
          ),
        ],
      ),
    );
  }
}
