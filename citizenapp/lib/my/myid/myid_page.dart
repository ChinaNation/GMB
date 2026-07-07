import 'package:flutter/material.dart';
import 'package:citizenapp/my/myid/myid_service.dart';
import 'package:citizenapp/ui/app_theme.dart';

class MyIdPage extends StatefulWidget {
  const MyIdPage({super.key, this.myIdService});

  final MyIdService? myIdService;

  @override
  State<MyIdPage> createState() => _MyIdPageState();
}

class _MyIdPageState extends State<MyIdPage> {
  late final MyIdService _myIdService;

  MyIdState _state =
      const MyIdState(identityStatus: MyIdIdentityStatus.notOnchain);
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _myIdService = widget.myIdService ?? MyIdService();
    _loadState();
  }

  Future<void> _loadState() async {
    setState(() {
      _loading = true;
    });
    final nextState = await _myIdService.getState();
    if (!mounted) return;
    setState(() {
      _state = nextState;
      _loading = false;
    });
  }

  String _statusLabel() {
    return switch (_state.identityStatus) {
      MyIdIdentityStatus.notOnchain => '未上链',
      MyIdIdentityStatus.normal => '正常',
      MyIdIdentityStatus.notYetValid => '未生效',
      MyIdIdentityStatus.expired => '已过期',
      MyIdIdentityStatus.revoked => '已吊销',
      MyIdIdentityStatus.conflict => '异常',
      MyIdIdentityStatus.queryFailed => '读取失败',
    };
  }

  Color _statusColor() {
    return switch (_state.identityStatus) {
      MyIdIdentityStatus.normal => AppTheme.success,
      MyIdIdentityStatus.notYetValid => AppTheme.warning,
      MyIdIdentityStatus.notOnchain => AppTheme.textTertiary,
      MyIdIdentityStatus.expired ||
      MyIdIdentityStatus.revoked ||
      MyIdIdentityStatus.conflict ||
      MyIdIdentityStatus.queryFailed =>
        AppTheme.danger,
    };
  }

  String _identityIdText() {
    final cidNumber = _state.identityCidNumber?.trim();
    return cidNumber == null || cidNumber.isEmpty ? '未上链' : cidNumber;
  }

  String _walletText() {
    final wallet = _state.identityWalletAccount?.trim();
    return wallet == null || wallet.isEmpty ? '未上链' : wallet;
  }

  String _validityText() {
    final validFrom = _formatDate(_state.passportValidFrom);
    final validUntil = _formatDate(_state.passportValidUntil);
    if (validFrom == null || validUntil == null) {
      return '未上链';
    }
    return '$validFrom-$validUntil';
  }

  Widget _buildStatusBadge() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: _statusColor().withAlpha(25),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Text(
        _statusLabel(),
        style: TextStyle(
          fontSize: 12,
          color: _statusColor(),
          fontWeight: FontWeight.w700,
        ),
      ),
    );
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
        padding: const EdgeInsets.all(16),
        children: [
          if (_loading)
            const LinearProgressIndicator(minHeight: 2)
          else
            const SizedBox(height: 2),
          const SizedBox(height: 14),
          Container(
            padding: const EdgeInsets.all(16),
            decoration: AppTheme.cardDecoration(),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Container(
                      width: 44,
                      height: 44,
                      decoration: BoxDecoration(
                        color: AppTheme.primary.withAlpha(18),
                        borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                      ),
                      child: const Icon(
                        Icons.badge_outlined,
                        color: AppTheme.primary,
                        size: 24,
                      ),
                    ),
                    const SizedBox(width: 12),
                    const Expanded(
                      child: Text(
                        '电子护照',
                        style: TextStyle(
                          fontSize: 20,
                          fontWeight: FontWeight.w700,
                          color: AppTheme.textPrimary,
                        ),
                      ),
                    ),
                    _buildStatusBadge(),
                  ],
                ),
                const SizedBox(height: 18),
                _FieldBlock(label: '投票账户', value: _walletText()),
                const SizedBox(height: 14),
                _FieldBlock(label: '身份 CID 号', value: _identityIdText()),
                const SizedBox(height: 14),
                _FieldBlock(label: '状态', value: _statusLabel()),
                const SizedBox(height: 14),
                _FieldBlock(label: '有效期', value: _validityText()),
              ],
            ),
          ),
          if (_state.errorMessage != null) ...[
            const SizedBox(height: 12),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
              decoration: AppTheme.bannerDecoration(AppTheme.danger),
              child: Text(
                _state.errorMessage!,
                style: const TextStyle(
                  color: AppTheme.danger,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }
}

class _FieldBlock extends StatelessWidget {
  const _FieldBlock({
    required this.label,
    required this.value,
  });

  final String label;
  final String value;

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
          maxLines: 2,
          overflow: TextOverflow.ellipsis,
          style: const TextStyle(
            fontSize: 15,
            color: AppTheme.textPrimary,
            height: 1.4,
          ),
        ),
      ],
    );
  }
}
