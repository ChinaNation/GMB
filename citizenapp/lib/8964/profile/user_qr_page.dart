import 'dart:convert';
import 'dart:math';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:saver_gallery/saver_gallery.dart';

import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/qr/bodies/user_contact_body.dart';
import 'package:citizenapp/qr/bodies/user_transfer_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 当前账户二维码入口。
///
/// 只有链上 CID↔AccountId 闭环命中的身份账户才能生成 `k=3 user_contact`；
/// 未注册账户或其它钱包子账户只生成 `k=4 user_transfer`，避免把本机钱包标签伪装成
/// 公开昵称或给无 CID 账户伪造用户身份。
Future<void> openAccountQrPage(
  BuildContext context, {
  required String accountId,
  required String paymentDisplayName,
  IdentityAccountResolver? identityResolver,
  CitizenProfileCache profileCache = const CitizenProfileCache(),
}) async {
  final normalizedAccountId = accountId.trim();
  if (!isAccountIdText(normalizedAccountId)) {
    _showQrMessage(context, '账户标识无效，无法生成二维码');
    return;
  }
  try {
    final identity =
        await (identityResolver ?? IdentityAccountResolver()).resolve();
    if (!context.mounted) return;
    final cidNumber = identity?.snapshot?.cidNumber.trim() ?? '';
    if (identity != null &&
        identity.accountId == normalizedAccountId &&
        cidNumber.isNotEmpty) {
      final cached = await profileCache.read(cidNumber);
      if (!context.mounted) return;
      final displayName =
          ProfilePresentation.forAccountId(cidNumber).resolveDisplayName(
        publicName: cached?.displayName,
      );
      await Navigator.of(context).push<void>(
        MaterialPageRoute<void>(
          builder: (_) => UserQrPage.userContact(
            cidNumber: cidNumber,
            displayName: displayName,
            accountId: normalizedAccountId,
          ),
        ),
      );
      return;
    }
    await Navigator.of(context).push<void>(
      MaterialPageRoute<void>(
        builder: (_) => UserQrPage.userTransfer(
          displayName: paymentDisplayName,
          accountId: normalizedAccountId,
        ),
      ),
    );
  } on Exception {
    if (context.mounted) {
      _showQrMessage(context, '身份读取失败，请稍后重试');
    }
  }
}

void _showQrMessage(BuildContext context, String message) {
  ScaffoldMessenger.of(context).showSnackBar(
    SnackBar(content: Text(message)),
  );
}

/// 二维码展示页：身份账户使用固定 `k=3`，普通收款账户使用五分钟 `k=4`。
class UserQrPage extends StatefulWidget {
  const UserQrPage.userContact({
    super.key,
    required this.cidNumber,
    required this.displayName,
    required this.accountId,
  }) : userContact = true;

  const UserQrPage.userTransfer({
    super.key,
    required this.displayName,
    required this.accountId,
  })  : cidNumber = null,
        userContact = false;

  final String? cidNumber;
  final String displayName;
  final String accountId;
  final bool userContact;

  @override
  State<UserQrPage> createState() => _UserQrPageState();
}

class _UserQrPageState extends State<UserQrPage> {
  static const int _transferTtlSeconds = 300;

  final GlobalKey _qrKey = GlobalKey();
  bool _saving = false;
  late final String _qrData = _buildQrData();

  /// 展示态 SS58 地址（accountId 为授权真源，ss58 仅用于展示与二维码载荷）。
  String get _ss58Address => ss58FromAccountIdText(widget.accountId);

  String _buildQrData() {
    if (widget.userContact) {
      return QrEnvelope<UserContactBody>(
        kind: QrKind.userContact,
        id: null,
        issuedAt: null,
        expiresAt: null,
        body: UserContactBody(
          cidNumber: widget.cidNumber!,
          ss58Address: _ss58Address,
          displayName: widget.displayName,
        ),
      ).toRawJson();
    }
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    return QrEnvelope<UserTransferBody>(
      kind: QrKind.userTransfer,
      id: _newTransferRequestId(),
      issuedAt: now,
      expiresAt: now + _transferTtlSeconds,
      body: UserTransferBody(
        ss58Address: _ss58Address,
        recipientName: widget.displayName,
        amount: '',
        symbol: 'GMB',
        memo: '',
        bank: '',
      ),
    ).toRawJson();
  }

  String _newTransferRequestId() {
    final random = Random.secure();
    final bytes = List<int>.generate(16, (_) => random.nextInt(256));
    return 'pay_${base64UrlEncode(bytes).replaceAll('=', '')}';
  }

  /// 复制地址到剪贴板（并入原钱包收款弹窗的能力）。
  void _copyAddress() {
    Clipboard.setData(ClipboardData(text: _ss58Address));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('SS58 地址已复制')),
    );
  }

  Future<void> _saveQr() async {
    if (_saving) return;
    setState(() => _saving = true);
    try {
      final boundary =
          _qrKey.currentContext?.findRenderObject() as RenderRepaintBoundary?;
      if (boundary == null) return;
      final image = await boundary.toImage(pixelRatio: 3.0);
      final byteData = await image.toByteData(format: ui.ImageByteFormat.png);
      if (byteData == null || !mounted) return;
      final result = await SaverGallery.saveImage(
        byteData.buffer.asUint8List(),
        fileName: 'my_qr_${DateTime.now().millisecondsSinceEpoch}.png',
        androidRelativePath: 'Pictures/CitizenApp',
        skipIfExists: false,
      );
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(result.isSuccess ? '已保存到相册' : '保存失败')),
      );
    } on Exception catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('保存失败：$e')),
      );
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('二维码'),
        centerTitle: true,
      ),
      body: Column(
        children: [
          const Spacer(),
          Text(
            widget.displayName,
            style: const TextStyle(
              fontSize: 20,
              fontWeight: FontWeight.w700,
            ),
          ),
          const SizedBox(height: 24),
          Stack(
            alignment: Alignment.center,
            children: [
              RepaintBoundary(
                key: _qrKey,
                child: Container(
                  color: Colors.white,
                  padding: const EdgeInsets.all(12),
                  child: CustomPaint(
                    size: const Size(240, 240),
                    painter: _HollowQrPainter(
                      data: _qrData,
                      hollowSize: 48,
                    ),
                  ),
                ),
              ),
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: Colors.white,
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(
                    color: AppTheme.border,
                    width: 1,
                  ),
                ),
                child: IconButton(
                  constraints: const BoxConstraints(),
                  padding: EdgeInsets.zero,
                  onPressed: _saving ? null : _saveQr,
                  icon: _saving
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.download,
                          size: 20, color: AppTheme.textSecondary),
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          // 地址居中显示，复制图标浮右不抢中心。
          Stack(
            alignment: Alignment.center,
            children: [
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 48),
                child: GestureDetector(
                  onTap: _copyAddress,
                  child: Text(
                    _ss58Address,
                    textAlign: TextAlign.center,
                    style: const TextStyle(
                      fontSize: 13,
                      color: AppTheme.textTertiary,
                      height: 1.5,
                    ),
                  ),
                ),
              ),
              Positioned(
                right: 16,
                child: IconButton(
                  icon: const Icon(Icons.copy, size: 16),
                  color: AppTheme.textTertiary,
                  tooltip: '复制地址',
                  padding: EdgeInsets.zero,
                  constraints:
                      const BoxConstraints(minWidth: 24, minHeight: 24),
                  onPressed: _copyAddress,
                ),
              ),
            ],
          ),
          const Spacer(),
          Padding(
            padding: const EdgeInsets.only(bottom: 32),
            child: Text(
              widget.userContact ? '扫描此二维码可加为联系人，或向其转账' : '临时收款码，5 分钟内有效',
              style: const TextStyle(
                color: AppTheme.textTertiary,
                fontSize: 12,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// 自绘二维码，中央 [hollowSize] 像素区域不绘制任何模块（真正留白）。
class _HollowQrPainter extends CustomPainter {
  _HollowQrPainter({required this.data, required this.hollowSize});

  final String data;
  final double hollowSize;

  @override
  void paint(Canvas canvas, Size size) {
    final qrCode = QrCode.fromData(
      data: data,
      errorCorrectLevel: QrErrorCorrectLevel.H,
    );
    final qrImage = QrImage(qrCode);
    final moduleCount = qrImage.moduleCount;
    final moduleSize = size.width / moduleCount;
    final paint = Paint()..color = const Color(0xFF000000);

    final hollowModules = (hollowSize / moduleSize).ceil();
    final hollowStart = (moduleCount - hollowModules) ~/ 2;
    final hollowEnd = hollowStart + hollowModules;

    for (var row = 0; row < moduleCount; row++) {
      for (var col = 0; col < moduleCount; col++) {
        if (qrImage.isDark(row, col)) {
          if (row >= hollowStart &&
              row < hollowEnd &&
              col >= hollowStart &&
              col < hollowEnd) {
            continue;
          }
          canvas.drawRect(
            Rect.fromLTWH(
              col * moduleSize,
              row * moduleSize,
              moduleSize,
              moduleSize,
            ),
            paint,
          );
        }
      }
    }
  }

  @override
  bool shouldRepaint(_HollowQrPainter oldDelegate) {
    return oldDelegate.data != data || oldDelegate.hollowSize != hollowSize;
  }
}
