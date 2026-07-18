import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'package:citizenapp/my/creator/creator_money.dart';
import 'package:citizenapp/my/creator/creator_service.dart';
import 'package:citizenapp/my/creator/models/creator_plan.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 编辑/新增会员档底部弹窗。
///
/// 档名（元展示）+ 月/季/年三周期（开关 + 元价输入，可只开部分）。保存=覆盖式写全部档位，
/// 经 [CreatorService.saveTiers] 走统一 0x1D 主钥签名（★弹一次生物识别）。返回最新 [CreatorPlan]。
class CreatorPlanEditSheet extends StatefulWidget {
  const CreatorPlanEditSheet({
    super.key,
    required this.service,
    required this.currentTiers,
    this.editing,
  });

  final CreatorService service;

  /// 当前全部档位（覆盖式保存需要）。
  final List<CreatorTier> currentTiers;

  /// 正在编辑的档位；null = 新增。
  final CreatorTier? editing;

  @override
  State<CreatorPlanEditSheet> createState() => _CreatorPlanEditSheetState();
}

class _CreatorPlanEditSheetState extends State<CreatorPlanEditSheet> {
  late final TextEditingController _nameController;
  final Map<BillingPeriod, TextEditingController> _priceControllers = {};
  final Map<BillingPeriod, bool> _enabled = {};
  bool _saving = false;

  bool get _isNew => widget.editing == null;

  @override
  void initState() {
    super.initState();
    final editing = widget.editing;
    _nameController = TextEditingController(text: editing?.name ?? '');
    for (final period in BillingPeriod.values) {
      final fen = editing?.priceFenOf(period);
      _enabled[period] = fen != null;
      _priceControllers[period] =
          TextEditingController(text: fen != null ? fenToYuanLabel(fen) : '');
    }
  }

  @override
  void dispose() {
    _nameController.dispose();
    for (final controller in _priceControllers.values) {
      controller.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final bottomInset = MediaQuery.of(context).viewInsets.bottom;
    return Padding(
      padding: EdgeInsets.only(bottom: bottomInset),
      child: SafeArea(
        top: false,
        child: Padding(
          padding: const EdgeInsets.fromLTRB(16, 10, 16, 18),
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
              Text(
                _isNew ? '新增会员档' : '编辑会员档',
                style: const TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w700,
                  color: AppTheme.textPrimary,
                ),
              ),
              const SizedBox(height: 14),
              const Text('会员档名',
                  style:
                      TextStyle(fontSize: 12, color: AppTheme.textSecondary)),
              const SizedBox(height: 6),
              TextField(
                controller: _nameController,
                maxLength: 20,
                decoration: const InputDecoration(
                  hintText: '如「铁杆粉丝」',
                  counterText: '',
                ),
              ),
              const SizedBox(height: 14),
              const Row(
                children: [
                  Text('订阅周期与价格',
                      style: TextStyle(
                          fontSize: 12, color: AppTheme.textSecondary)),
                  Text('（可只开部分周期，单位：元）',
                      style: TextStyle(
                          fontSize: 12, color: AppTheme.textTertiary)),
                ],
              ),
              const SizedBox(height: 8),
              for (final period in BillingPeriod.values) _periodRow(period),
              const SizedBox(height: 18),
              FilledButton(
                onPressed: _saving ? null : _save,
                child: _saving
                    ? const SizedBox(
                        width: 20,
                        height: 20,
                        child: CircularProgressIndicator(
                            strokeWidth: 2, color: Colors.white),
                      )
                    : const Text('保存'),
              ),
              if (!_isNew) ...[
                const SizedBox(height: 6),
                Center(
                  child: TextButton(
                    onPressed: _saving ? null : _confirmDelete,
                    style:
                        TextButton.styleFrom(foregroundColor: AppTheme.danger),
                    child: const Text('删除此档'),
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  Widget _periodRow(BillingPeriod period) {
    final enabled = _enabled[period] ?? false;
    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: Row(
        children: [
          Switch(
            value: enabled,
            onChanged: _saving
                ? null
                : (value) => setState(() => _enabled[period] = value),
          ),
          const SizedBox(width: 4),
          SizedBox(
            width: 34,
            child: Text(period.label,
                style:
                    const TextStyle(fontSize: 14, color: AppTheme.textPrimary)),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: TextField(
              controller: _priceControllers[period],
              enabled: enabled && !_saving,
              keyboardType:
                  const TextInputType.numberWithOptions(decimal: true),
              inputFormatters: [
                FilteringTextInputFormatter.allow(RegExp(r'[0-9.]')),
              ],
              decoration: const InputDecoration(
                suffixText: '元',
                hintText: '0.00',
                isDense: true,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _save() async {
    final name = _nameController.text.trim();
    if (name.isEmpty) {
      _toast('请填写会员档名');
      return;
    }
    final prices = <BillingPeriod, int>{};
    for (final period in BillingPeriod.values) {
      if (_enabled[period] != true) continue;
      final fen = yuanTextToFen(_priceControllers[period]!.text);
      if (fen == null) {
        _toast('${period.label}价格无效');
        return;
      }
      prices[period] = fen;
    }
    if (prices.isEmpty) {
      _toast('至少开启一个周期并填写价格');
      return;
    }

    final tier = CreatorTier(
      tierId: widget.editing?.tierId ?? _newTierId(),
      name: name,
      pricesFen: prices,
    );
    await _commit(_mergeTiers(tier));
  }

  Future<void> _confirmDelete() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('删除此档'),
        content: const Text('删除后已订阅该档的用户不再续费，确定删除？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(false),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(true),
            style: TextButton.styleFrom(foregroundColor: AppTheme.danger),
            child: const Text('删除'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    final remaining = widget.currentTiers
        .where((t) => t.tierId != widget.editing!.tierId)
        .toList();
    await _commit(remaining);
  }

  /// 覆盖式保存全部档位（触发生物识别）。成功回传最新计划并关闭。
  Future<void> _commit(List<CreatorTier> tiers) async {
    setState(() => _saving = true);
    try {
      final plan = await widget.service.saveTiers(tiers);
      if (!mounted) return;
      Navigator.of(context).pop(plan);
    } on CreatorException catch (e) {
      if (!mounted) return;
      setState(() => _saving = false);
      _toast(e.message);
    }
  }

  List<CreatorTier> _mergeTiers(CreatorTier tier) {
    final tiers = List<CreatorTier>.from(widget.currentTiers);
    final index = tiers.indexWhere((t) => t.tierId == tier.tierId);
    if (index >= 0) {
      tiers[index] = tier;
    } else {
      tiers.add(tier);
    }
    return tiers;
  }

  String _newTierId() => 't_${DateTime.now().microsecondsSinceEpoch}';

  void _toast(String message) {
    ScaffoldMessenger.of(context)
        .showSnackBar(SnackBar(content: Text(message)));
  }
}
