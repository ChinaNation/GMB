import 'dart:async';

import 'package:flutter/material.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../login/login_qr_handler.dart';
import '../wallet/wallet_manager.dart';
import 'app_theme.dart';

/// 登录签名页面：显示登录挑战详情 → 用户确认 → 签名 → 展示 receipt QR。
class LoginSignPage extends StatefulWidget {
  const LoginSignPage({
    super.key,
    required this.wallet,
    required this.challengeRaw,
  });

  final WalletProfile wallet;
  final String challengeRaw;

  @override
  State<LoginSignPage> createState() => _LoginSignPageState();
}

class _LoginSignPageState extends State<LoginSignPage> {
  LoginChallengeEnvelope? _challenge;
  LoginReceiptEnvelope? _receipt;
  String? _error;
  bool _signing = false;
  Timer? _timer;
  int _remainingSeconds = 0;

  @override
  void initState() {
    super.initState();
    _parseChallenge();
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  void _parseChallenge() {
    try {
      final challenge = parseLoginChallenge(widget.challengeRaw);
      if (!verifySystemSignature(challenge)) {
        setState(() => _error = '系统签名验证失败,二维码可能被篡改');
        return;
      }
      if (isLoginChallengeExpired(challenge)) {
        setState(() => _error = '登录二维码已过期');
        return;
      }
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      setState(() {
        _challenge = challenge;
        _remainingSeconds = (challenge.expiresAt ?? 0) - now;
      });
      _startCountdown();
    } on LoginQrException catch (e) {
      setState(() => _error = e.message);
    } catch (e) {
      setState(() => _error = '解析失败: $e');
    }
  }

  void _startCountdown() {
    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (!mounted) return;
      setState(() {
        _remainingSeconds--;
        if (_remainingSeconds <= 0) {
          _timer?.cancel();
          if (_receipt == null) {
            _error = '登录二维码已过期';
          }
        }
      });
    });
  }

  Future<void> _confirmAndSign() async {
    final challenge = _challenge;
    if (challenge == null || _signing) return;

    setState(() => _signing = true);

    try {
      final walletManager = WalletManager();
      // 以当前钱包公钥为 principal 构造签名原文
      final signMessage = buildSignMessage(challenge, widget.wallet.pubkeyHex);
      final result = await walletManager.signUtf8WithWallet(
        widget.wallet.walletIndex,
        signMessage,
      );

      final receipt = buildLoginReceipt(
        challenge: challenge,
        pubkeyHex: result.pubkeyHex,
        signatureHex: result.signatureHex,
      );

      if (!mounted) return;
      setState(() {
        _receipt = receipt;
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

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.surfaceDark,
      appBar: AppBar(
        title: const Text('登录确认'),
        backgroundColor: AppTheme.surfaceDark,
        foregroundColor: AppTheme.textPrimary,
        elevation: 0,
      ),
      body: SafeArea(
        child: _error != null
            ? _buildError()
            : _receipt != null
                ? _buildReceipt()
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
            color: AppTheme.surfaceCard,
            child: Padding(
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
                  _infoRow('系统', loginSystemDisplayName(c)),
                  _infoRow('钱包', _shortenAddress(widget.wallet.address)),
                  _infoRow(
                    '剩余时间',
                    _remainingSeconds > 0
                        ? '$_remainingSeconds秒'
                        : '已过期',
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 12),
          Text(
            '确认后将使用当前钱包签名登录 ${loginSystemDisplayName(c)}',
            style: const TextStyle(color: AppTheme.textSecondary, fontSize: 14),
            textAlign: TextAlign.center,
          ),
          const Spacer(),
          ElevatedButton(
            onPressed:
                _signing || _remainingSeconds <= 0 ? null : _confirmAndSign,
            style: ElevatedButton.styleFrom(
              padding: const EdgeInsets.symmetric(vertical: 16),
              backgroundColor: AppTheme.primary,
            ),
            child: _signing
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      color: Colors.white,
                    ),
                  )
                : const Text('确认登录', style: TextStyle(fontSize: 16)),
          ),
          const SizedBox(height: 16),
        ],
      ),
    );
  }

  Widget _buildReceipt() {
    final json = _receipt!.toRawJson();
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
      child: Column(
        children: [
          const SizedBox(height: 16),
          const Text(
            '请用登录页面扫描此二维码',
            style: TextStyle(
              fontSize: 18,
              fontWeight: FontWeight.bold,
              color: AppTheme.textPrimary,
            ),
          ),
          const SizedBox(height: 8),
          Text(
            loginSystemDisplayName(_challenge!),
            style: const TextStyle(color: AppTheme.textSecondary, fontSize: 14),
          ),
          const SizedBox(height: 24),
          Expanded(
            child: Center(
              child: Container(
                padding: const EdgeInsets.all(16),
                decoration: const BoxDecoration(
                  color: Colors.white,
                  borderRadius: BorderRadius.all(Radius.circular(12)),
                ),
                child: QrImageView(
                  data: json,
                  version: QrVersions.auto,
                  size: 280,
                  errorCorrectionLevel: QrErrorCorrectLevel.M,
                ),
              ),
            ),
          ),
          const SizedBox(height: 16),
          ElevatedButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('完成'),
          ),
          const SizedBox(height: 16),
        ],
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
            child: Text(
              label,
              style: const TextStyle(color: AppTheme.textSecondary, fontSize: 14),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(color: AppTheme.textPrimary, fontSize: 14),
            ),
          ),
        ],
      ),
    );
  }

  String _shortenAddress(String address) {
    if (address.length <= 16) return address;
    return '${address.substring(0, 8)}...${address.substring(address.length - 8)}';
  }
}
