import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/my/myid/myid_service.dart';
import 'package:wuminapp_mobile/my/myid/myid_sign_page.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/pages/wallet_page.dart';

class MyIdPage extends StatefulWidget {
  const MyIdPage({super.key, this.myIdService});

  final MyIdService? myIdService;

  @override
  State<MyIdPage> createState() => _MyIdPageState();
}

class _MyIdPageState extends State<MyIdPage> {
  late final MyIdService _myIdService;

  MyIdState _state = const MyIdState(status: MyIdStatus.unset);
  bool _submitting = false;

  @override
  void initState() {
    super.initState();
    _myIdService = widget.myIdService ?? MyIdService();
    _loadState();
  }

  Future<void> _loadState() async {
    final localState = await _myIdService.getState();
    if (!mounted) return;
    setState(() {
      _state = localState;
    });
    final synced = await _myIdService.syncFromBackend();
    if (!mounted) return;
    setState(() {
      _state = synced;
    });
  }

  Future<void> _selectWallet() async {
    if (_submitting) return;
    final wallet = await Navigator.of(context).push<WalletProfile>(
      MaterialPageRoute(
        builder: (_) => const MyWalletPage(
          selectForBind: true,
          bindPurposeLabel: '电子护照',
        ),
      ),
    );
    if (!mounted || wallet == null) return;
    setState(() {
      _submitting = true;
    });
    try {
      // 中文注释：后端当前验签协议仍使用旧挑战前缀；用户侧入口已统一为电子护照。
      final timestamp = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final signMessage = 'CITIZEN_VOTE_REGISTER|${wallet.address}|$timestamp';
      final messageBytes = Uint8List.fromList(utf8.encode(signMessage));

      if (!wallet.isHotWallet) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('冷钱包暂不支持电子护照注册，请使用热钱包')),
        );
        setState(() => _submitting = false);
        return;
      }
      final walletManager = WalletManager();
      final signatureBytes = await walletManager.signWithWallet(
        wallet.walletIndex,
        messageBytes,
      );

      final sigHex =
          '0x${signatureBytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final nextState = await _myIdService.registerMyId(
        walletAddress: wallet.address,
        walletPubkeyHex: wallet.pubkeyHex,
        isColdWallet: wallet.isColdWallet,
        signatureHex: sigHex,
        signMessage: signMessage,
      );
      if (!mounted) return;
      setState(() {
        _state = nextState;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('电子护照已注册，等待现场绑定')),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('电子护照注册失败：$e')),
      );
    } finally {
      if (mounted) {
        setState(() {
          _submitting = false;
        });
      }
    }
  }

  Future<void> _openSignPage() async {
    if (_state.walletPubkeyHex == null) return;
    final walletManager = WalletManager();
    final wallets = await walletManager.getWallets();
    final wallet = wallets.cast<WalletProfile?>().firstWhere(
          (w) =>
              w!.pubkeyHex.toLowerCase() ==
              _state.walletPubkeyHex!.toLowerCase(),
          orElse: () => null,
        );
    if (!mounted) return;
    if (wallet == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('未找到匹配的钱包')),
      );
      return;
    }
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => MyIdSignPage(wallet: wallet),
      ),
    );
  }

  String _statusLabel() {
    return switch (_state.status) {
      MyIdStatus.unset => '未设置',
      MyIdStatus.pending => '待绑定',
      MyIdStatus.bound => '已绑定',
    };
  }

  Color _statusColor() {
    return switch (_state.status) {
      MyIdStatus.unset => AppTheme.textTertiary,
      MyIdStatus.pending => AppTheme.warning,
      MyIdStatus.bound => AppTheme.success,
    };
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

  String _identityIdText() {
    final code = _state.sfidCode?.trim();
    return code == null || code.isEmpty ? '未绑定' : code;
  }

  String _identityStatusText() {
    // 中文注释：identityStatus 是身份ID状态，不是绑定状态；
    // 只有 SFID 明确返回 NORMAL 才显示正常，其他状态统一按异常展示。
    return _state.identityStatus?.trim().toUpperCase() == 'NORMAL'
        ? '状态：正常'
        : '状态：异常';
  }

  String _validityText() {
    final validFrom = _formatDate(_state.validFrom);
    final validUntil = _formatDate(_state.validUntil);
    if (validFrom == null || validUntil == null) {
      return '有效期：未绑定';
    }
    return '有效期：$validFrom-$validUntil';
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
    // 中文注释：后端返回 YYYY-MM-DD 日期，不按本地时区转换，避免护照日期跨天。
    return '${year.toString().padLeft(4, '0')}年'
        '${month.toString().padLeft(2, '0')}月'
        '${day.toString().padLeft(2, '0')}日';
  }

  @override
  Widget build(BuildContext context) {
    final canSign = _state.status == MyIdStatus.pending && !_state.isColdWallet;
    return Scaffold(
      appBar: AppBar(
        title: const Text('电子护照'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
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
                const Text(
                  '身份ID',
                  style: TextStyle(
                    fontSize: 13,
                    color: AppTheme.textSecondary,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  _identityIdText(),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    fontSize: 15,
                    color: AppTheme.textPrimary,
                    height: 1.4,
                  ),
                ),
                const SizedBox(height: 14),
                const Text(
                  '投票账户',
                  style: TextStyle(
                    fontSize: 13,
                    color: AppTheme.textSecondary,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  _state.walletAddress ?? '未设置',
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    fontSize: 15,
                    color: AppTheme.textPrimary,
                    height: 1.4,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  _identityStatusText(),
                  style: TextStyle(
                    fontSize: 13,
                    color:
                        _state.identityStatus?.trim().toUpperCase() == 'NORMAL'
                            ? AppTheme.success
                            : AppTheme.danger,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  _validityText(),
                  style: const TextStyle(
                    fontSize: 13,
                    color: AppTheme.textSecondary,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 16),
          SizedBox(
            width: double.infinity,
            child: FilledButton.icon(
              onPressed: _submitting ? null : _selectWallet,
              icon: _submitting
                  ? const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.account_balance_wallet_outlined),
              label: Text(_submitting ? '正在注册...' : '选择钱包'),
            ),
          ),
          if (canSign) ...[
            const SizedBox(height: 12),
            SizedBox(
              width: double.infinity,
              child: OutlinedButton.icon(
                onPressed: _openSignPage,
                icon: const Icon(Icons.qr_code_scanner_outlined),
                label: const Text('现场签名'),
              ),
            ),
          ],
        ],
      ),
    );
  }
}
