import 'dart:async';

import 'package:flutter/material.dart';
import 'package:qr_flutter/qr_flutter.dart';

import 'app_theme.dart';
import '../signer/field_labels.dart';
import '../signer/offline_sign_service.dart';
import '../signer/qr_signer.dart';
import '../util/screenshot_guard.dart';
import '../wallet/wallet_manager.dart';

/// 离线签名页面。
///
/// 扫码与账户定位由 [ScanPage] 全局完成并传入 [account]+[raw];本页只解析该
/// 签名请求、展示中文摘要、在本机完成签名并展示响应二维码(不再自带扫码)。
class OfflineSignPage extends StatefulWidget {
  const OfflineSignPage({
    super.key,
    required this.account,
    required this.walletName,
    required this.raw,
  });

  /// 签名主体账户（由 ScanPage 按 QR 的 signerPublicKey 定位后传入）。
  final Account account;
  final String walletName;
  final String raw;

  @override
  State<OfflineSignPage> createState() => _OfflineSignPageState();
}

class _OfflineSignPageState extends State<OfflineSignPage> {
  final OfflineSignService _offlineSignService = OfflineSignService();
  final QrSigner _qrSigner = QrSigner();

  Timer? _timer;
  bool _signing = false;
  SignRequestEnvelope? _request;
  SignResponseEnvelope? _response;
  OfflineSignVerification? _verification;
  String? _parseError;
  int _remainingSeconds = 0;

  @override
  void initState() {
    super.initState();
    ScreenshotGuard.enable();
    WidgetsBinding.instance
        .addPostFrameCallback((_) => _parseRequest(widget.raw));
  }

  @override
  void dispose() {
    _timer?.cancel();
    ScreenshotGuard.disable();
    super.dispose();
  }

  int _secondsLeft(SignRequestEnvelope request) {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final left = (request.expiresAt ?? 0) - now;
    return left > 0 ? left : 0;
  }

  void _startCountdown(SignRequestEnvelope request) {
    _timer?.cancel();
    _remainingSeconds = _secondsLeft(request);
    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (!mounted) return;
      setState(() {
        _remainingSeconds = _secondsLeft(request);
      });
    });
  }

  void _parseRequest(String raw) {
    try {
      final request = _offlineSignService.parseRequest(raw);
      final verification = _offlineSignService.verifyPayload(request);
      if (!mounted) return;
      setState(() {
        _request = request;
        _verification = verification;
      });
      _startCountdown(request);
    } on QrSignException catch (e) {
      if (!mounted) return;
      setState(() => _parseError = e.message);
    }
  }

  Future<void> _signRequest() async {
    final request = _request;
    // 同一个已扫描请求只允许进入一次密钥签名：签名进行中或
    // 已生成响应二维码时直接返回，不叠加任何确认签名。
    if (request == null || _signing || _response != null) return;
    if (_remainingSeconds <= 0) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('签名请求已过期，请重新扫描')),
      );
      return;
    }

    setState(() {
      _signing = true;
    });
    try {
      final response = await _offlineSignService.signParsedRequest(
        accountId: widget.account.accountId,
        request: request,
      );
      if (!mounted) return;
      setState(() {
        _response = response;
      });
    } on OfflineSignException catch (e) {
      if (!mounted) return;
      _showError('离线签名失败', e.message);
    } on WalletAuthException catch (e) {
      if (!mounted) return;
      _showError('身份验证', e.message);
    } catch (e) {
      if (!mounted) return;
      _showError('离线签名失败', '$e');
    } finally {
      if (mounted) {
        setState(() {
          _signing = false;
        });
      }
    }
  }

  Future<void> _showError(String title, String message) async {
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: Text(message),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('确定'),
          ),
        ],
      ),
    );
  }

  String _truncate(String text, {int head = 12, int tail = 8}) {
    if (text.length <= head + tail + 3) return text;
    return '${text.substring(0, head)}...${text.substring(text.length - tail)}';
  }

  // 扫码确认页字段名必须显示中文,翻译单源在 signer/field_labels.dart。

  Widget _buildTransactionDetails(
      SignRequestEnvelope request, OfflineSignVerification verification) {
    final decoded = verification.decoded;
    final rejected = verification.status == SignDecisionStatus.reject;

    final Widget statusBanner;
    switch (verification.status) {
      case SignDecisionStatus.normal:
        statusBanner = _buildBanner(
          color: AppTheme.success,
          icon: Icons.verified_rounded,
          text: '签名状态正常，内容已完整中文解释，可以签名',
        );
      case SignDecisionStatus.reject:
        statusBanner = _buildBanner(
          color: AppTheme.danger,
          icon: Icons.dangerous_rounded,
          text: verification.rejectReason ?? '签名请求已拒绝',
        );
    }

    final actionLabel = verification.actionLabel ?? '未登记签名动作';

    final List<Widget> detailRows;
    if (!rejected && decoded != null) {
      detailRows = [
        _detailRow('交易类型', actionLabel),
        ...decoded.reviewFields.entries.map((e) {
          final label = fieldLabelTextOrNull(e.key);
          return _detailRow(
            label ?? '未翻译字段',
            fieldValueText(e.key, e.value),
          );
        }),
      ];
    } else {
      detailRows = [
        _detailRow('交易类型', actionLabel),
        _detailRow('状态', '拒绝签名'),
        _detailRow('原因', verification.rejectReason ?? '签名请求已拒绝'),
      ];
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        statusBanner,
        const SizedBox(height: 12),
        Container(
          padding: const EdgeInsets.all(16),
          decoration: AppTheme.cardDecoration(),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: detailRows,
          ),
        ),
      ],
    );
  }

  Widget _buildBanner({
    required Color color,
    required IconData icon,
    required String text,
  }) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
      decoration: AppTheme.bannerDecoration(color),
      child: Row(
        children: [
          Icon(icon, color: color, size: 20),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              text,
              style: TextStyle(
                color: color,
                fontWeight: FontWeight.w600,
                fontSize: 13,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _detailRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 80,
            child: Text(
              label,
              style: const TextStyle(
                color: AppTheme.textSecondary,
                fontWeight: FontWeight.w500,
                fontSize: 13,
              ),
            ),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(
                fontWeight: FontWeight.w600,
                color: AppTheme.textPrimary,
                fontSize: 14,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRequestSummary(SignRequestEnvelope request) {
    final expired = _remainingSeconds <= 0;
    final verification = _verification;
    final isRejected = verification?.status == SignDecisionStatus.reject;

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        // 有效期横幅
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
          decoration: AppTheme.bannerDecoration(
              expired ? AppTheme.danger : AppTheme.success),
          child: Row(
            children: [
              Icon(
                expired ? Icons.timer_off_rounded : Icons.timer_rounded,
                color: expired ? AppTheme.danger : AppTheme.success,
                size: 18,
              ),
              const SizedBox(width: 8),
              Text(
                expired ? '签名请求已过期，请重新扫描' : '签名请求有效期剩余：${_remainingSeconds}s',
                style: TextStyle(
                  color: expired ? AppTheme.danger : AppTheme.success,
                  fontWeight: FontWeight.w600,
                  fontSize: 13,
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 12),
        // 请求基本信息
        Container(
          padding: const EdgeInsets.all(16),
          decoration: AppTheme.cardDecoration(),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _detailRow('请求 ID', request.id ?? ''),
              _detailRow(
                '签名公钥',
                _truncate(request.body.signerPublicKeyHex),
              ),
            ],
          ),
        ),
        const SizedBox(height: 12),
        if (verification != null)
          _buildTransactionDetails(request, verification),
        const SizedBox(height: 16),
        if (isRejected) ...[
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(14),
            decoration: AppTheme.bannerDecoration(AppTheme.danger),
            child: Text(
              verification?.rejectReason ?? '签名请求已拒绝',
              style: const TextStyle(
                color: AppTheme.danger,
                fontWeight: FontWeight.w600,
                fontSize: 13,
              ),
            ),
          ),
          const SizedBox(height: 8),
        ],
        Row(
          children: [
            Expanded(
              child: OutlinedButton(
                onPressed: () => Navigator.of(context).pop(),
                child: const Text('返回'),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: FilledButton(
                onPressed:
                    (_signing || expired || isRejected) ? null : _signRequest,
                child: _signing
                    ? const SizedBox(
                        width: 20,
                        height: 20,
                        child: CircularProgressIndicator(
                          strokeWidth: 2,
                          color: Colors.white,
                        ),
                      )
                    : const Text('确认签名'),
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildResponseView(SignResponseEnvelope response) {
    final responseJson = _qrSigner.encodeResponse(response);
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        // 成功横幅
        _buildBanner(
          color: AppTheme.success,
          icon: Icons.check_circle_rounded,
          text: '签名已完成，请用在线手机扫描下方签名响应二维码',
        ),
        const SizedBox(height: 24),
        // QR 码容器
        Center(
          child: Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              color: Colors.white,
              borderRadius: BorderRadius.circular(AppTheme.radiusLg),
              boxShadow: [
                BoxShadow(
                  color: AppTheme.primary.withAlpha(20),
                  blurRadius: 20,
                  offset: const Offset(0, 8),
                ),
              ],
            ),
            child: QrImageView(
              data: responseJson,
              version: QrVersions.auto,
              size: 360,
              errorCorrectionLevel: QrErrorCorrectLevel.M,
              eyeStyle: const QrEyeStyle(
                eyeShape: QrEyeShape.square,
                color: AppTheme.primaryDark,
              ),
              dataModuleStyle: const QrDataModuleStyle(
                dataModuleShape: QrDataModuleShape.square,
                color: AppTheme.primaryDark,
              ),
            ),
          ),
        ),
        const SizedBox(height: 20),
        // 信息
        Container(
          padding: const EdgeInsets.all(16),
          decoration: AppTheme.cardDecoration(),
          child: Column(
            children: [
              _detailRow('请求 ID', response.id ?? ''),
              _detailRow(
                '签名公钥',
                _truncate(response.body.signerPublicKeyHex),
              ),
            ],
          ),
        ),
        const SizedBox(height: 24),
        FilledButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('完成'),
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final request = _request;
    final response = _response;
    final parseError = _parseError;
    return Scaffold(
      appBar: AppBar(
        title: const Text('扫码签名'),
        centerTitle: true,
      ),
      body: parseError != null
          ? _buildParseError(parseError)
          : response != null
              ? _buildResponseView(response)
              : request != null
                  ? _buildRequestSummary(request)
                  : const Center(child: CircularProgressIndicator()),
    );
  }

  Widget _buildParseError(String message) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.error_outline, size: 56, color: AppTheme.danger),
            const SizedBox(height: 16),
            Text(message,
                textAlign: TextAlign.center,
                style: const TextStyle(color: AppTheme.danger, fontSize: 15)),
            const SizedBox(height: 24),
            FilledButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('返回'),
            ),
          ],
        ),
      ),
    );
  }
}
