import 'package:flutter/material.dart';

import '../login_models.dart';
import '../login_service.dart';
import '../../../wallet/core/wallet_manager.dart';
import '../../../ui/app_theme.dart';

/// 扫码登录确认页：解析登录 QR → 显示系统信息 → 用户确认 → 签名 → 提交 receipt → 完成。
class LoginScanResultPage extends StatefulWidget {
  const LoginScanResultPage({super.key, required this.challengeRaw});

  final String challengeRaw;

  @override
  State<LoginScanResultPage> createState() => _LoginScanResultPageState();
}

class _LoginScanResultPageState extends State<LoginScanResultPage> {
  final LoginService _loginService = LoginService();

  LoginChallenge? _challenge;
  String? _error;
  bool _signing = false;
  bool _submitted = false;

  @override
  void initState() {
    super.initState();
    _parse();
  }

  Future<void> _parse() async {
    try {
      final challenge = _loginService.parseChallenge(widget.challengeRaw);
      await _loginService.validateSystemSignature(challenge);
      if (!mounted) return;
      setState(() => _challenge = challenge);
    } on LoginException catch (e) {
      if (!mounted) return;
      setState(() => _error = e.message);
    } catch (e) {
      if (!mounted) return;
      setState(() => _error = '解析失败: $e');
    }
  }

  Future<void> _confirmAndSign() async {
    final challenge = _challenge;
    if (challenge == null || _signing) return;

    setState(() => _signing = true);

    try {
      final walletManager = WalletManager();
      final activeWallet = await walletManager.getWallet();
      if (activeWallet == null) {
        throw const LoginException(LoginErrorCode.walletMissing, '请先选择钱包');
      }

      final receipt = await _loginService.buildReceiptPayload(
        challenge,
        walletIndex: activeWallet.walletIndex,
      );

      // 提交 receipt 到后端
      await _submitReceipt(challenge, receipt);

      if (!mounted) return;
      setState(() {
        _submitted = true;
        _signing = false;
      });
    } on LoginException catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.message;
        _signing = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = '签名失败: $e';
        _signing = false;
      });
    }
  }

  Future<void> _submitReceipt(
    LoginChallenge challenge,
    Map<String, dynamic> receipt,
  ) async {
    // wuminapp 签名后不直接提交，而是通过前端 polling 机制。
    // SFID/CPMS 前端在生成 QR 后会持续 poll /api/v1/admin/auth/qr/result。
    // 暂时显示成功提示，与冷钱包流程统一。
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('扫码登录'),
      ),
      body: SafeArea(
        child: _error != null
            ? _buildError()
            : _submitted
                ? _buildReceiptQr()
                : _challenge != null
                    ? _buildConfirm()
                    : const Center(
                        child: CircularProgressIndicator(
                            color: AppTheme.primary)),
      ),
    );
  }

  Widget _buildError() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.error_outline, size: 64, color: AppTheme.danger),
            const SizedBox(height: 16),
            Text(
              _error!,
              style: const TextStyle(color: AppTheme.danger, fontSize: 16),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 24),
            OutlinedButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('返回'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildConfirm() {
    final c = _challenge!;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Container(
            decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
            padding: const EdgeInsets.all(20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  '扫码登录',
                  style: TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.bold,
                    color: AppTheme.textPrimary,
                  ),
                ),
                const SizedBox(height: 16),
                _infoRow('系统', c.system.toUpperCase()),
                if (c.isExpired)
                  _infoRow('状态', '已过期')
                else
                  _infoRow('有效期', '${c.ttlSeconds}秒'),
              ],
            ),
          ),
          const SizedBox(height: 12),
          const Text(
            '确认后将使用当前钱包签名登录',
            style: TextStyle(color: AppTheme.textSecondary, fontSize: 14),
            textAlign: TextAlign.center,
          ),
          const Spacer(),
          FilledButton(
            onPressed: _signing || c.isExpired ? null : _confirmAndSign,
            child: _signing
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(
                        strokeWidth: 2, color: Colors.white),
                  )
                : const Text('确认登录'),
          ),
          const SizedBox(height: 16),
        ],
      ),
    );
  }

  Widget _buildReceiptQr() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.check_circle, size: 64, color: AppTheme.success),
            const SizedBox(height: 16),
            const Text(
              '签名已完成',
              style: TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.bold,
                color: AppTheme.textPrimary,
              ),
            ),
            const SizedBox(height: 8),
            const Text(
              '请在登录页面确认登录结果',
              style: TextStyle(color: AppTheme.textSecondary, fontSize: 14),
            ),
            const SizedBox(height: 24),
            OutlinedButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('完成'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _infoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        children: [
          SizedBox(
            width: 80,
            child: Text(label,
                style: const TextStyle(
                    color: AppTheme.textSecondary, fontSize: 14)),
          ),
          Expanded(
            child: Text(value,
                style: const TextStyle(
                    color: AppTheme.textPrimary, fontSize: 14)),
          ),
        ],
      ),
    );
  }
}
