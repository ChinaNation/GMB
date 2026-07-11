import 'package:flutter/material.dart';

import 'package:citizenapp/my/myid/myid_service.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/identity_badge.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 电子护照页。
///
/// 身份只认**默认用户**(最靠前热钱包);默认用户切换(增删/拖拽排序)后,
/// 监听 [WalletManager.walletsRevision] 自动重读跟随。按三档渲染同一张卡:
/// 访客(完全匿名)/ 投票公民 / 竞选公民,字段数递增。卡片宽度适应屏幕
/// (窄屏填满、宽屏限宽居中),高度随内容多少自适应。
class MyIdPage extends StatefulWidget {
  const MyIdPage({super.key, this.myIdService});

  final MyIdService? myIdService;

  @override
  State<MyIdPage> createState() => _MyIdPageState();
}

class _MyIdPageState extends State<MyIdPage> {
  late final MyIdService _myIdService;

  MyIdState _state = const MyIdState(tier: MyIdTier.visitor);
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _myIdService = widget.myIdService ?? MyIdService();
    // 切换默认用户即跟随(与聊天/广场同一身份版本号)。
    WalletManager.walletsRevision.addListener(_loadState);
    _loadState();
  }

  @override
  void dispose() {
    WalletManager.walletsRevision.removeListener(_loadState);
    super.dispose();
  }

  Future<void> _loadState() async {
    setState(() => _loading = true);
    final nextState = await _myIdService.getState();
    if (!mounted) return;
    setState(() {
      _state = nextState;
      _loading = false;
    });
  }

  // ── 分档色/文案 ──

  Color _tierColor() => switch (_state.tier) {
        MyIdTier.candidate => AppTheme.identityCandidate,
        MyIdTier.voting => AppTheme.identityVoting,
        MyIdTier.visitor => AppTheme.identityVisitor,
      };

  String _tierLabel() => switch (_state.tier) {
        MyIdTier.candidate => '竞选公民',
        MyIdTier.voting => '投票公民',
        MyIdTier.visitor => '访客',
      };

  bool get _isQueryFailed => _state.status == MyIdStatus.queryFailed;

  ({String text, Color color})? _statusPill() {
    if (_isQueryFailed) return (text: '读取失败', color: AppTheme.danger);
    if (!_state.isCitizen) return (text: '访客', color: AppTheme.textTertiary);
    return switch (_state.status) {
      MyIdStatus.normal => (text: '正常', color: AppTheme.success),
      MyIdStatus.notYetValid => (text: '未生效', color: AppTheme.warning),
      MyIdStatus.expired => (text: '已过期', color: AppTheme.danger),
      MyIdStatus.revoked => (text: '已吊销', color: AppTheme.danger),
      _ => null,
    };
  }

  String _validityText() {
    final from = _formatDate(_state.passportValidFrom);
    final until = _formatDate(_state.passportValidUntil);
    if (from == null || until == null) return '—';
    return '$from 至 $until';
  }

  String _shortAddress(String address) {
    final value = address.trim();
    if (value.length <= 16) return value;
    return '${value.substring(0, 8)}…${value.substring(value.length - 8)}';
  }

  String? _formatDate(String? raw) {
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

  @override
  Widget build(BuildContext context) {
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
          const SizedBox(height: 14),
          // 宽度适应屏幕:窄屏填满,宽屏限宽居中,避免平板上过分拉伸。
          Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 560),
              child: _buildCard(),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCard() {
    // 高度随内容自适应:Column mainAxisSize.min,访客最矮、竞选最高。
    return Container(
      padding: const EdgeInsets.all(18),
      decoration: AppTheme.cardDecoration(),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          _buildHeader(),
          ..._buildTierContent(),
        ],
      ),
    );
  }

  Widget _buildHeader() {
    final style = identityBadgeStyle(
      identityLevel: _state.identityLevel,
      membershipLevel: null,
      membershipActive: false,
    )!;
    final pill = _statusPill();
    return Row(
      children: [
        IdentityBadge(style: style, size: 44, tooltip: _tierLabel()),
        const SizedBox(width: 12),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                '电子护照',
                style: TextStyle(
                  fontSize: 19,
                  fontWeight: FontWeight.w700,
                  color: AppTheme.textPrimary,
                ),
              ),
              const SizedBox(height: 4),
              _tierChip(),
            ],
          ),
        ),
        if (pill != null) _pill(pill.text, pill.color),
      ],
    );
  }

  Widget _tierChip() {
    final color = _tierColor();
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: color.withAlpha(20),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Text(
        _tierLabel(),
        style: TextStyle(
          fontSize: 12,
          color: color,
          fontWeight: FontWeight.w700,
        ),
      ),
    );
  }

  Widget _pill(String text, Color color) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withAlpha(25),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Text(
        text,
        style: TextStyle(
          fontSize: 12,
          color: color,
          fontWeight: FontWeight.w700,
        ),
      ),
    );
  }

  List<Widget> _buildTierContent() {
    if (_isQueryFailed) return _errorContent();
    return switch (_state.tier) {
      MyIdTier.visitor => _visitorContent(),
      MyIdTier.voting => _citizenContent(candidate: false),
      MyIdTier.candidate => _citizenContent(candidate: true),
    };
  }

  List<Widget> _visitorContent() {
    final hint = _state.errorMessage;
    return [
      Padding(
        padding: const EdgeInsets.symmetric(vertical: 26),
        child: Center(
          child: Column(
            children: [
              const Text(
                '完全匿名',
                style: TextStyle(
                  fontSize: 22,
                  fontWeight: FontWeight.w700,
                  letterSpacing: 2,
                  color: AppTheme.identityVisitor,
                ),
              ),
              const SizedBox(height: 8),
              const Text(
                '无公民身份',
                style: TextStyle(fontSize: 13, color: AppTheme.textTertiary),
              ),
              if (hint != null && hint.isNotEmpty) ...[
                const SizedBox(height: 10),
                Text(
                  hint,
                  style:
                      const TextStyle(fontSize: 12, color: AppTheme.textTertiary),
                ),
              ],
            ],
          ),
        ),
      ),
    ];
  }

  List<Widget> _citizenContent({required bool candidate}) {
    return [
      const SizedBox(height: 16),
      _FieldBlock(
        label: '投票账户',
        value: _shortAddress(_state.votingAccount ?? '—'),
        mono: true,
      ),
      _sectionHeader(Icons.link, '链上身份信息', AppTheme.textSecondary),
      const SizedBox(height: 12),
      _FieldBlock(
        label: '公民身份 CID 号',
        value: _state.cidNumber ?? '—',
        mono: true,
      ),
      const SizedBox(height: 14),
      _FieldBlock(
        label: '居住选区',
        value: (_state.residenceDistrict?.isNotEmpty ?? false)
            ? _state.residenceDistrict!
            : '—',
      ),
      const SizedBox(height: 14),
      _FieldBlock(label: '投票身份有效期', value: _validityText()),
      if (candidate) ..._candidateExtra(),
    ];
  }

  List<Widget> _candidateExtra() {
    final sex = _state.citizenSexLabel;
    return [
      const SizedBox(height: 16),
      const Divider(height: 1, color: AppTheme.border),
      const SizedBox(height: 12),
      _sectionHeader(
        Icons.how_to_vote_outlined,
        '竞选公开信息',
        AppTheme.identityCandidate,
      ),
      const SizedBox(height: 12),
      Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: _FieldBlock(
              label: '姓名',
              value: (_state.citizenFullName?.isNotEmpty ?? false)
                  ? _state.citizenFullName!
                  : '—',
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: _FieldBlock(
              label: '性别',
              value: (sex?.isNotEmpty ?? false) ? sex! : '—',
            ),
          ),
        ],
      ),
      const SizedBox(height: 14),
      _FieldBlock(
        label: '出生地',
        value: (_state.birthDistrict?.isNotEmpty ?? false)
            ? _state.birthDistrict!
            : '—',
      ),
    ];
  }

  List<Widget> _errorContent() {
    return [
      Padding(
        padding: const EdgeInsets.symmetric(vertical: 22),
        child: Center(
          child: Column(
            children: [
              const Icon(Icons.error_outline, color: AppTheme.danger, size: 28),
              const SizedBox(height: 10),
              Text(
                _state.errorMessage ?? '读取失败',
                textAlign: TextAlign.center,
                style: const TextStyle(
                  fontSize: 14,
                  color: AppTheme.danger,
                  fontWeight: FontWeight.w600,
                ),
              ),
              const SizedBox(height: 14),
              OutlinedButton.icon(
                onPressed: _loading ? null : _loadState,
                icon: const Icon(Icons.refresh, size: 18),
                label: const Text('重试'),
              ),
            ],
          ),
        ),
      ),
    ];
  }

  Widget _sectionHeader(IconData icon, String text, Color color) {
    return Padding(
      padding: const EdgeInsets.only(top: 16),
      child: Row(
        children: [
          Icon(icon, size: 16, color: color),
          const SizedBox(width: 6),
          Text(
            text,
            style: TextStyle(
              fontSize: 12,
              color: color,
              fontWeight: FontWeight.w700,
            ),
          ),
        ],
      ),
    );
  }
}

class _FieldBlock extends StatelessWidget {
  const _FieldBlock({
    required this.label,
    required this.value,
    this.mono = false,
  });

  final String label;
  final String value;
  final bool mono;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: const TextStyle(
            fontSize: 13,
            color: AppTheme.textSecondary,
            fontWeight: FontWeight.w600,
          ),
        ),
        const SizedBox(height: 6),
        Text(
          value,
          style: TextStyle(
            fontSize: mono ? 13 : 15,
            fontFamily: mono ? 'monospace' : null,
            color: AppTheme.textPrimary,
            height: 1.4,
          ),
        ),
      ],
    );
  }
}
