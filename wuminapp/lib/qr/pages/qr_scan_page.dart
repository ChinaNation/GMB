import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:wuminapp_mobile/qr/contact/contact_qr_models.dart';
import 'package:wuminapp_mobile/qr/login/login_models.dart';
import 'package:wuminapp_mobile/qr/login/login_service.dart';
import 'package:wuminapp_mobile/qr/qr_router.dart';
import 'package:wuminapp_mobile/qr/transfer/transfer_qr_models.dart';
import 'package:wuminapp_mobile/user/user_service.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 扫码结果：收款码预填数据。
class QrScanTransferResult {
  const QrScanTransferResult({
    required this.toAddress,
    this.amount,
    this.symbol,
    this.memo,
    this.bank,
  });

  final String toAddress;
  final String? amount;
  final String? symbol;
  final String? memo;
  final String? bank;
}

/// 统一扫码页。
///
/// 根据 [QrRouter] 分析扫码内容，分发到不同业务流程：
/// - 登录码 → 签名并展示回执
/// - 收款码 → 返回预填数据给交易页
/// - 用户码 → 添加到通讯录
/// - 裸地址 → 兼容旧格式，等同收款码
class QrScanPage extends StatefulWidget {
  const QrScanPage({
    super.key,
    this.walletIndex,
    this.selfAccountPubkeyHex,
    this.enableContact = true,
  });

  /// 指定钱包索引（登录签名用）。
  final int? walletIndex;

  /// 当前用户公钥（通讯录防自加用）。
  final String? selfAccountPubkeyHex;

  /// 是否启用用户码扫描。钱包页面入口设为 false。
  final bool enableContact;

  @override
  State<QrScanPage> createState() => _QrScanPageState();
}

class _QrScanPageState extends State<QrScanPage> {
  final MobileScannerController _controller = MobileScannerController();
  final QrRouter _router = QrRouter();
  final LoginService _loginService = LoginService();
  final UserContactService _contactService = UserContactService();
  bool _handled = false;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _handleCode(String raw) async {
    if (_handled) {
      return;
    }
    _handled = true;
    await _controller.stop();

    try {
      final result = _router.route(raw);
      switch (result.type) {
        case QrRouteType.login:
          await _handleLogin(raw);
        case QrRouteType.transfer:
          _handleTransfer(result);
        case QrRouteType.contact:
          if (widget.enableContact) {
            await _handleContact(raw);
          } else {
            await _showUnrecognized();
          }
        case QrRouteType.legacyAddress:
          _handleLegacyAddress(result.extractedAddress!);
        case QrRouteType.qrSign:
          await _showQrSignHint();
        case QrRouteType.unknown:
          await _showUnrecognized();
      }
    } catch (e) {
      if (!mounted) {
        return;
      }
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('扫码处理异常'),
          content: Text('$e'),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('确定'),
            ),
          ],
        ),
      );
    } finally {
      if (mounted) {
        _handled = false;
        await _controller.start();
      }
    }
  }

  // ---------------------------------------------------------------------------
  // 登录码
  // ---------------------------------------------------------------------------

  Future<void> _handleLogin(String raw) async {
    LoginChallenge challenge;
    try {
      challenge = _loginService.parseChallenge(raw);
      await _loginService.validateSystemSignature(challenge);
    } catch (e) {
      if (!mounted) {
        return;
      }
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('无法识别登录二维码'),
          content: Text('$e'),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('确定'),
            ),
          ],
        ),
      );
      return;
    }

    if (!mounted) {
      return;
    }

    final shouldSign = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('登录 ${_displaySystemName(challenge.system)}系统'),
        content: const Text('请确认后生成登录签名二维码。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('签名并生成二维码'),
          ),
        ],
      ),
    );

    if (shouldSign == true && mounted) {
      await _signAndShowReceipt(challenge);
    }
  }

  Future<void> _signAndShowReceipt(LoginChallenge challenge) async {
    try {
      if (challenge.isExpired) {
        throw Exception('登录挑战已过期，请重新扫码');
      }
      final result = await _loginService.buildReceiptPayload(
        challenge,
        walletIndex: widget.walletIndex,
      );

      if (!mounted) {
        return;
      }

      final compact = jsonEncode(result);

      final goBack = await Navigator.of(context).push<bool>(
        MaterialPageRoute(
          builder: (_) => _LoginReceiptPage(
            compactPayload: compact,
            expiresAt: challenge.expiresAt,
          ),
        ),
      );
      if (goBack == true && mounted) {
        Navigator.of(context).pop();
      }
    } on WalletAuthException catch (e) {
      if (!mounted) {
        return;
      }
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('身份验证'),
          content: Text(e.message),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('确定'),
            ),
          ],
        ),
      );
    } catch (e) {
      if (!mounted) {
        return;
      }
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('登录回执生成失败'),
          content: Text('$e'),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('确定'),
            ),
          ],
        ),
      );
    }
  }

  // ---------------------------------------------------------------------------
  // 收款码
  // ---------------------------------------------------------------------------

  void _handleTransfer(QrRouteResult result) {
    if (!mounted) {
      return;
    }
    try {
      final payload = TransferQrPayload.fromJson(result.jsonData!);
      Navigator.of(context).pop(QrScanTransferResult(
        toAddress: payload.to,
        amount: payload.amount,
        symbol: payload.symbol,
        memo: payload.memo,
        bank: payload.bank,
      ));
    } catch (e) {
      showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('收款码解析失败'),
          content: Text('$e'),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('确定'),
            ),
          ],
        ),
      );
    }
  }

  // ---------------------------------------------------------------------------
  // 裸地址（向后兼容）
  // ---------------------------------------------------------------------------

  void _handleLegacyAddress(String address) {
    if (!mounted) {
      return;
    }
    Navigator.of(context).pop(QrScanTransferResult(toAddress: address));
  }

  // ---------------------------------------------------------------------------
  // 用户码
  // ---------------------------------------------------------------------------

  Future<void> _handleContact(String raw) async {
    if (!mounted) {
      return;
    }
    try {
      final result = await _contactService.addFromQrPayload(
        raw,
        selfAccountPubkeyHex: widget.selfAccountPubkeyHex,
      );
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            result.created
                ? '已加入通讯录：${result.contact.displayNickname}'
                : '已更新通讯录：${result.contact.displayNickname}',
          ),
        ),
      );
    } catch (e) {
      if (!mounted) {
        return;
      }
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('无法识别用户二维码'),
          content: Text('$e'),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('继续扫描'),
            ),
          ],
        ),
      );
    }
  }

  // ---------------------------------------------------------------------------
  // 扫码签名提示
  // ---------------------------------------------------------------------------

  Future<void> _showQrSignHint() async {
    if (!mounted) return;
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('签名请求'),
        content: const Text('这是一个冷钱包签名请求二维码。\n请在转账页面发起交易后，通过签名会话页面扫描回执。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('确定'),
          ),
        ],
      ),
    );
  }

  // ---------------------------------------------------------------------------
  // 未识别
  // ---------------------------------------------------------------------------

  Future<void> _showUnrecognized() async {
    if (!mounted) {
      return;
    }
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('无法识别二维码'),
        content: Text(widget.enableContact
            ? '请扫描登录码、收款码或用户码。'
            : '请扫描登录码或收款码。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('确定'),
          ),
        ],
      ),
    );
  }

  String _displaySystemName(String system) {
    if (system.toLowerCase() == 'sfid') {
      return 'SFID';
    }
    return system.toUpperCase();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('扫码'),
        centerTitle: true,
      ),
      body: Stack(
        fit: StackFit.expand,
        children: [
          MobileScanner(
            controller: _controller,
            onDetect: (capture) async {
              final code = capture.barcodes.first.rawValue;
              if (code == null || code.isEmpty) {
                return;
              }
              await _handleCode(code);
            },
          ),
          Align(
            alignment: Alignment.topCenter,
            child: Container(
              margin: const EdgeInsets.all(16),
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                color: Colors.black54,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Text(
                widget.enableContact
                    ? '扫描登录码、收款码或用户码'
                    : '扫描登录码或收款码',
                style: const TextStyle(color: Colors.white),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

// -----------------------------------------------------------------------------
// 登录回执展示页（私有，仅 QrScanPage 内部使用）
// -----------------------------------------------------------------------------

class _LoginReceiptPage extends StatefulWidget {
  const _LoginReceiptPage({
    required this.compactPayload,
    required this.expiresAt,
  });

  final String compactPayload;
  final int expiresAt;

  @override
  State<_LoginReceiptPage> createState() => _LoginReceiptPageState();
}

class _LoginReceiptPageState extends State<_LoginReceiptPage> {
  Timer? _timer;
  late int _remainingSeconds;

  @override
  void initState() {
    super.initState();
    _remainingSeconds = _secondsLeft();
    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (!mounted) {
        return;
      }
      setState(() {
        _remainingSeconds = _secondsLeft();
      });
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  int _secondsLeft() {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final left = widget.expiresAt - now;
    return left > 0 ? left : 0;
  }

  @override
  Widget build(BuildContext context) {
    final expired = _remainingSeconds <= 0;
    return Scaffold(
      appBar: AppBar(
        title: const Text('登录回执'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: BoxDecoration(
              color: expired ? Colors.red.shade50 : Colors.green.shade50,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Text(
              expired ? '该回执已过期，请重新扫码' : '回执有效期剩余：${_remainingSeconds}s',
              style: TextStyle(
                color: expired ? Colors.red.shade700 : Colors.green.shade700,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
          const SizedBox(height: 12),
          Center(
            child: QrImageView(
              data: widget.compactPayload,
              version: QrVersions.auto,
              size: 220,
              errorStateBuilder: (cxt, err) {
                return Container(
                  width: 220,
                  height: 220,
                  padding: const EdgeInsets.all(10),
                  decoration: BoxDecoration(
                    color: Colors.red.shade50,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Center(
                    child: Text(
                      '回执二维码渲染失败：$err',
                      style: TextStyle(color: Colors.red.shade700),
                    ),
                  ),
                );
              },
            ),
          ),
          const SizedBox(height: 12),
          const SizedBox(height: 20),
          Row(
            children: [
              Expanded(
                child: OutlinedButton(
                  onPressed: () => Navigator.of(context).pop(false),
                  child: const Text('重新扫码'),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: FilledButton(
                  onPressed: () => Navigator.of(context).pop(true),
                  child: const Text('完成'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
