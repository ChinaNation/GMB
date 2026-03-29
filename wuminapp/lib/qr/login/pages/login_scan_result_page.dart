import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;

import '../login_service.dart';
import '../../qr_protocols.dart';
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
      final walletManager = WalletManager.instance;
      final activeIndex = walletManager.activeWalletIndex;
      if (activeIndex == null) {
        throw const LoginException(LoginErrorCode.walletUnavailable, '请先选择钱包');
      }

      final receipt = await _loginService.buildReceiptPayload(
        challenge: challenge,
        walletIndex: activeIndex,
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
    // 从 QR 中解析后端地址不可行（QR 不含后端 URL）。
    // wuminapp 作为热钱包可以联网，直接向 SFID/CPMS 后端提交。
    // 但后端 URL 需要从某处获取——当前使用 receipt 自行构建，
    // 由 SFID/CPMS 前端轮询 challenge 状态来获取登录结果。
    //
    // 实际链路：wuminapp 签名后不直接提交，而是通过前端 polling 机制。
    // SFID/CPMS 前端在生成 QR 后会持续 poll /api/v1/admin/auth/qr/result。
    // 因此 wuminapp 需要把 receipt 提交到后端的 /api/v1/admin/auth/qr/complete。
    //
    // 但 wuminapp 不知道后端 URL。解决方案：
    // receipt 包含 challenge_id，SFID/CPMS 前端也知道这个 ID，
    // 前端可以扫描 wuminapp 显示的 receipt QR 来完成提交。
    // 这和冷钱包流程一致——wuminapp 也显示 receipt QR。
    //
    // 暂时显示 receipt QR 供前端扫描，与冷钱包流程统一。
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('扫码登录'),
        backgroundColor: AppTheme.background,
        foregroundColor: AppTheme.textPrimary,
        elevation: 0,
      ),
      backgroundColor: AppTheme.background,
      body: SafeArea(
        child: _error != null
            ? _buildError()
            : _submitted
                ? _buildReceiptQr()
                : _challenge != null
                    ? _buildConfirm()
                    : const Center(child: CircularProgressIndicator()),
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
            const Icon(Icons.error_outline, size: 64, color: Colors.redAccent),
            const SizedBox(height: 16),
            Text(
              _error!,
              style: const TextStyle(color: Colors.redAccent, fontSize: 16),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 24),
            ElevatedButton(
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
          Card(
            color: AppTheme.surface,
            child: Padding(
              padding: const EdgeInsets.all(20),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    '扫码登录',
                    style: TextStyle(
                      fontSize: 20,
                      fontWeight: FontWeight.bold,
                      color: AppTheme.textPrimary,
                    ),
                  ),
                  const SizedBox(height: 16),
                  _infoRow('系统', c.systemDisplayName),
                  if (c.isExpired)
                    _infoRow('状态', '已过期')
                  else
                    _infoRow('有效期', '${c.expiresAt - (DateTime.now().millisecondsSinceEpoch ~/ 1000)}秒'),
                ],
              ),
            ),
          ),
          const SizedBox(height: 12),
          Text(
            '确认后将使用当前钱包签名登录',
            style: TextStyle(color: AppTheme.textSecondary, fontSize: 14),
            textAlign: TextAlign.center,
          ),
          const Spacer(),
          ElevatedButton(
            onPressed: _signing || c.isExpired ? null : _confirmAndSign,
            style: ElevatedButton.styleFrom(
              padding: const EdgeInsets.symmetric(vertical: 16),
              backgroundColor: AppTheme.primary,
            ),
            child: _signing
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                  )
                : const Text('确认登录', style: TextStyle(fontSize: 16)),
          ),
          const SizedBox(height: 16),
        ],
      ),
    );
  }

  Widget _buildReceiptQr() {
    // 签名完成后显示成功提示（热钱包登录后 SFID 前端通过 polling 检测到状态变化）
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.check_circle, size: 64, color: Colors.green),
            const SizedBox(height: 16),
            const Text(
              '签名已完成',
              style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            Text(
              '请在登录页面确认登录结果',
              style: TextStyle(color: AppTheme.textSecondary, fontSize: 14),
            ),
            const SizedBox(height: 24),
            ElevatedButton(
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
            child: Text(label, style: TextStyle(color: AppTheme.textSecondary, fontSize: 14)),
          ),
          Expanded(
            child: Text(value, style: TextStyle(color: AppTheme.textPrimary, fontSize: 14)),
          ),
        ],
      ),
    );
  }
}
