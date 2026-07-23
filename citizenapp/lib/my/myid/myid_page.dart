import 'package:flutter/material.dart';

import 'package:citizenapp/my/myid/myid_service.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/identity_badge.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 电子护照页。
///
/// 页面始终展示访客轻节点、投票身份、竞选身份三张卡。只有默认用户当前身份对应的
/// 卡片排在首位、标记“当前身份”并展示真实链上值；非当前公民卡只展示该身份涉及
/// 的字段名称，不能重复泄露当前用户数据。链读取失败时不静默降级成访客轻节点。
class MyIdPage extends StatefulWidget {
  const MyIdPage({super.key, this.myIdService});

  final MyIdService? myIdService;

  @override
  State<MyIdPage> createState() => _MyIdPageState();
}

class _MyIdPageState extends State<MyIdPage> {
  static const List<MyIdTier> _baseTierOrder = <MyIdTier>[
    MyIdTier.visitor,
    MyIdTier.voting,
    MyIdTier.candidate,
  ];

  late final MyIdService _myIdService;
  MyIdState _state = const MyIdState(tier: MyIdTier.visitor);
  bool _loading = true;

  bool get _isQueryFailed => _state.status == MyIdStatus.queryFailed;

  @override
  void initState() {
    super.initState();
    _myIdService = widget.myIdService ?? MyIdService();
    // 默认用户切换必须让电子护照立即重排，和广场、聊天共用同一身份版本号。
    WalletManager.walletsRevision.addListener(_loadState);
    _loadState();
  }

  @override
  void dispose() {
    WalletManager.walletsRevision.removeListener(_loadState);
    super.dispose();
  }

  Future<void> _loadState() async {
    if (mounted) setState(() => _loading = true);
    MyIdState nextState;
    try {
      nextState = await _myIdService.getState();
    } on Exception catch (error) {
      // Service 正常会把链错误收口为 queryFailed；这里兜住依赖异常，仍不能把
      // 未知错误误认成访客轻节点。
      nextState = MyIdState(
        tier: MyIdTier.visitor,
        status: MyIdStatus.queryFailed,
        errorMessage: '电子护照读取失败：$error',
      );
    }
    if (!mounted) return;
    setState(() {
      _state = nextState;
      _loading = false;
    });
  }

  List<MyIdTier> _orderedTiers() {
    if (_isQueryFailed) return _baseTierOrder;
    return <MyIdTier>[
      _state.tier,
      ..._baseTierOrder.where((tier) => tier != _state.tier),
    ];
  }

  bool _isCurrent(MyIdTier tier) => !_isQueryFailed && tier == _state.tier;

  bool _showActualValues(MyIdTier tier) =>
      _isCurrent(tier) && tier != MyIdTier.visitor;

  @override
  Widget build(BuildContext context) {
    final tiers = _orderedTiers();
    return Scaffold(
      appBar: AppBar(
        title: const Text('电子护照'),
        centerTitle: true,
        actions: [
          IconButton(
            tooltip: '刷新',
            onPressed: _loading ? null : _loadState,
            icon: const Icon(Icons.refresh),
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 14),
        children: [
          if (_loading)
            const LinearProgressIndicator(minHeight: 2)
          else
            const SizedBox(height: 2),
          if (_isQueryFailed) ...[
            const SizedBox(height: 12),
            _PassportMessageBanner(
              message: _state.errorMessage ?? '链上身份读取失败',
              isError: true,
              onRetry: _loading ? null : _loadState,
            ),
          ] else if ((_state.errorMessage ?? '').trim().isNotEmpty) ...[
            const SizedBox(height: 12),
            _PassportMessageBanner(message: _state.errorMessage!),
          ],
          const SizedBox(height: 14),
          Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 560),
              child: Column(
                children: [
                  for (var index = 0; index < tiers.length; index++) ...[
                    _PassportIdentityCard(
                      key: ValueKey<String>(
                          'passport-card-${tiers[index].name}'),
                      tier: tiers[index],
                      current: _isCurrent(tiers[index]),
                      showActualValues: _showActualValues(tiers[index]),
                      fields: _fieldsFor(tiers[index]),
                    ),
                    if (index != tiers.length - 1) const SizedBox(height: 14),
                  ],
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  List<_PassportField> _fieldsFor(MyIdTier tier) {
    if (tier == MyIdTier.visitor) return const <_PassportField>[];
    final showValues = _showActualValues(tier);
    final fields = <_PassportField>[
      _PassportField(
        label: '投票账户',
        value: showValues ? _shortAddress(_state.votingAccountId) : null,
        mono: true,
      ),
      _PassportField(
        label: '公民CID号',
        value: showValues ? _displayValue(_state.cidNumber) : null,
        mono: true,
      ),
      _PassportField(
        label: '居住选区',
        value: showValues ? _displayValue(_state.residenceDistrict) : null,
      ),
      _PassportField(
        label: '身份状态',
        value: showValues ? _statusText(_state.status) : null,
      ),
      _PassportField(
        label: '身份有效期',
        value: showValues ? _validityText() : null,
      ),
    ];
    if (tier == MyIdTier.candidate) {
      fields.addAll(<_PassportField>[
        _PassportField(
          label: '公民姓名',
          value: showValues
              ? _displayValue(
                  '${_state.familyName ?? ''}${_state.givenName ?? ''}')
              : null,
        ),
        _PassportField(
          label: '性别',
          value: showValues ? _displayValue(_state.citizenSexLabel) : null,
        ),
        _PassportField(
          label: '出生日期',
          value:
              showValues ? (_formatDate(_state.citizenBirthDate) ?? '—') : null,
        ),
        _PassportField(
          label: '出生地',
          value: showValues ? _displayValue(_state.birthDistrict) : null,
        ),
      ]);
    }
    return fields;
  }

  String _validityText() {
    final from = _formatDate(_state.passportValidFrom);
    final until = _formatDate(_state.passportValidUntil);
    if (from == null || until == null) return '—';
    return '$from 至 $until';
  }

  static String _displayValue(String? input) {
    final value = input?.trim() ?? '';
    return value.isEmpty ? '—' : value;
  }

  static String _shortAddress(String? input) {
    final value = input?.trim() ?? '';
    if (value.isEmpty) return '—';
    if (value.length <= 18) return value;
    return '${value.substring(0, 8)}…${value.substring(value.length - 8)}';
  }

  static String _statusText(MyIdStatus? status) => switch (status) {
        MyIdStatus.normal => '正常',
        MyIdStatus.notYetValid => '未生效',
        MyIdStatus.expired => '已过期',
        MyIdStatus.revoked => '已吊销',
        _ => '—',
      };

  static String? _formatDate(String? raw) {
    final value = raw?.trim();
    if (value == null || value.isEmpty) return null;
    final parts = value.split('-');
    if (parts.length != 3) return null;
    final year = int.tryParse(parts[0]);
    final month = int.tryParse(parts[1]);
    final day = int.tryParse(parts[2]);
    if (year == null || month == null || day == null) return null;
    return '${year.toString().padLeft(4, '0')}年'
        '${month.toString().padLeft(2, '0')}月'
        '${day.toString().padLeft(2, '0')}日';
  }
}

class _PassportIdentityCard extends StatelessWidget {
  const _PassportIdentityCard({
    super.key,
    required this.tier,
    required this.current,
    required this.showActualValues,
    required this.fields,
  });

  final MyIdTier tier;
  final bool current;
  final bool showActualValues;
  final List<_PassportField> fields;

  String get _title => switch (tier) {
        MyIdTier.visitor => '访客轻节点',
        MyIdTier.voting => '公民身份 · 投票',
        MyIdTier.candidate => '公民身份 · 竞选',
      };

  Color get _color => switch (tier) {
        MyIdTier.visitor => AppTheme.identityVisitor,
        MyIdTier.voting => AppTheme.identityVoting,
        MyIdTier.candidate => AppTheme.identityCandidate,
      };

  String get _identityLevel => switch (tier) {
        MyIdTier.visitor => 'visitor',
        MyIdTier.voting => 'voting',
        MyIdTier.candidate => 'candidate',
      };

  @override
  Widget build(BuildContext context) {
    final badgeStyle = identityBadgeStyle(
      identityLevel: _identityLevel,
      membershipLevel: null,
      membershipActive: false,
    )!;
    return AnimatedContainer(
      duration: const Duration(milliseconds: 220),
      width: double.infinity,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Color.alphaBlend(_color.withAlpha(8), AppTheme.surfaceCard),
        borderRadius: BorderRadius.circular(AppTheme.radiusLg),
        border: Border.all(color: _color, width: current ? 2 : 1),
        boxShadow: [
          BoxShadow(
            color: _color.withAlpha(current ? 38 : 13),
            blurRadius: current ? 18 : 8,
            offset: Offset(0, current ? 8 : 3),
          ),
        ],
      ),
      child: Stack(
        children: [
          Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            mainAxisSize: MainAxisSize.min,
            children: [
              Padding(
                padding: EdgeInsets.only(right: current ? 88 : 0),
                child: Row(
                  children: [
                    IdentityBadge(
                      style: badgeStyle,
                      size: 44,
                      tooltip: _title,
                    ),
                    const SizedBox(width: 12),
                    // Flexible（非 Expanded）让标题只占内容宽度，匿名标签紧贴“访客轻节点”右侧，
                    // 而不是被撑到卡片最右端；标题过长时仍走省略号避免溢出。
                    Flexible(
                      child: Text(
                        _title,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(
                          fontSize: 18,
                          height: 1.2,
                          fontWeight: FontWeight.w800,
                          color: _color,
                        ),
                      ),
                    ),
                    // 访客轻节点默认匿名，用一枚小标签直接点明，替代原“没有公民身份信息”整段空态。
                    if (tier == MyIdTier.visitor) ...[
                      const SizedBox(width: 8),
                      _AnonymousTag(color: _color),
                    ],
                  ],
                ),
              ),
              // 访客轻节点无上链字段：删去空态整块后卡片只保留标题行，高度自然收缩。
              if (tier != MyIdTier.visitor) ...[
                const SizedBox(height: 14),
                for (var index = 0; index < fields.length; index++) ...[
                  _PassportFieldRow(
                    field: fields[index],
                    color: _color,
                    showValue: showActualValues,
                  ),
                  if (index != fields.length - 1) const SizedBox(height: 6),
                ],
              ],
            ],
          ),
          if (current)
            Positioned(
              top: 0,
              right: 0,
              child: Container(
                key: ValueKey<String>('current-identity-${tier.name}'),
                padding: const EdgeInsets.symmetric(horizontal: 9, vertical: 5),
                decoration: BoxDecoration(
                  color: _color,
                  borderRadius: BorderRadius.circular(7),
                ),
                child: const Text(
                  '当前身份',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 11,
                    fontWeight: FontWeight.w800,
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}

/// 访客轻节点的“匿名”提示标签：小圆角药丸 + 隐私图标，沿用所在卡片的身份色。
class _AnonymousTag extends StatelessWidget {
  const _AnonymousTag({required this.color});

  final Color color;

  @override
  Widget build(BuildContext context) {
    return Container(
      key: const ValueKey<String>('passport-anonymous-tag'),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: color.withAlpha(20),
        borderRadius: BorderRadius.circular(999),
        border: Border.all(color: color.withAlpha(64)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.visibility_off_outlined, size: 12, color: color),
          const SizedBox(width: 3),
          Text(
            '匿名',
            style: TextStyle(
              fontSize: 11,
              height: 1.2,
              fontWeight: FontWeight.w700,
              color: color,
            ),
          ),
        ],
      ),
    );
  }
}

class _PassportField {
  const _PassportField({required this.label, this.value, this.mono = false});

  final String label;
  final String? value;
  final bool mono;
}

class _PassportFieldRow extends StatelessWidget {
  const _PassportFieldRow({
    required this.field,
    required this.color,
    required this.showValue,
  });

  final _PassportField field;
  final Color color;
  final bool showValue;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
      decoration: BoxDecoration(
        color: color.withAlpha(showValue ? 10 : 7),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withAlpha(24)),
      ),
      child: showValue
          ? Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                SizedBox(
                  width: 126,
                  child: Text(
                    field.label,
                    style: const TextStyle(
                      fontSize: 12,
                      height: 1.4,
                      color: AppTheme.textSecondary,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    field.value ?? '—',
                    textAlign: TextAlign.right,
                    style: TextStyle(
                      fontSize: field.mono ? 12 : 13,
                      height: 1.4,
                      fontFamily: field.mono ? 'monospace' : null,
                      color: AppTheme.textPrimary,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
              ],
            )
          : Text(
              field.label,
              style: const TextStyle(
                fontSize: 12,
                height: 1.35,
                color: AppTheme.textPrimary,
                fontWeight: FontWeight.w600,
              ),
            ),
    );
  }
}

class _PassportMessageBanner extends StatelessWidget {
  const _PassportMessageBanner({
    required this.message,
    this.isError = false,
    this.onRetry,
  });

  final String message;
  final bool isError;
  final VoidCallback? onRetry;

  @override
  Widget build(BuildContext context) {
    final color = isError ? AppTheme.danger : AppTheme.warning;
    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 560),
        child: Container(
          width: double.infinity,
          padding: const EdgeInsets.fromLTRB(12, 10, 8, 10),
          decoration: AppTheme.bannerDecoration(color),
          child: Row(
            children: [
              Icon(
                isError ? Icons.error_outline : Icons.info_outline,
                size: 18,
                color: color,
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  message,
                  style: const TextStyle(
                    fontSize: 12,
                    height: 1.4,
                    color: AppTheme.textSecondary,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ),
              if (onRetry != null)
                TextButton(onPressed: onRetry, child: const Text('重试')),
            ],
          ),
        ),
      ),
    );
  }
}
