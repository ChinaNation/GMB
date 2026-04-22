import 'dart:async';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:qr/qr.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:wuminapp_mobile/qr/bodies/user_transfer_body.dart';
import 'package:wuminapp_mobile/qr/envelope.dart';
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/rpc/offchain_clearing.dart';
import 'package:wuminapp_mobile/trade/offchain/clearing_bank_prefs.dart';
import 'package:wuminapp_mobile/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 扫码支付 Step 2c-ii-a:**清算行收款 QR 页**。
///
/// 中文注释:
/// - 清算行(L2)体系唯一收款 QR 页。`shenfen_id` 来自 `ClearingBankPrefs`
///   (绑定 `call_index 30 bind_clearing_bank` 时落盘;原省储行 `call_index 9
///   bind_clearing_institution` + `OnchainRpc.queryClearingInstitution` 已
///   在 Step 2b-iv-b 随老 pallet 一起下线)。
/// - QR 载荷为 `WUMIN_QR_V1 kind=user_transfer`,字段:
///     - `address`:本人收款地址(SS58)
///     - `bank`:收款方清算行 `shenfen_id`,付款方扫码时用 SFID API 反查主账户
///     - `amount` / `memo`:可选,由商户预填
/// - 余额轮询:若调用方传入 `clearingNodeWssUrl`,每 5 秒调
///   `offchain_queryBalance(user)` 刷一次当前余额,显示在 QR 下方。
///   listener 把 `PaymentSettled` 事件写回本地 ledger 后,轮询自然看到余额增加
///   (无需 WS 订阅,Step 2c-ii-b 再做实时推送)。
/// - 未绑定 / 缓存丢失 → 提示用户先去绑定,不生成 QR。
class OffchainClearingReceivePage extends StatefulWidget {
  const OffchainClearingReceivePage({
    super.key,
    required this.wallet,
    this.clearingNodeWssUrl,
  });

  final WalletProfile wallet;

  /// 可选 WSS。为空时余额不轮询,仅展示 QR。
  final String? clearingNodeWssUrl;

  @override
  State<OffchainClearingReceivePage> createState() =>
      _OffchainClearingReceivePageState();
}

class _OffchainClearingReceivePageState
    extends State<OffchainClearingReceivePage> {
  static const Duration _balancePollInterval = Duration(seconds: 5);

  final TextEditingController _amountCtrl = TextEditingController();
  final TextEditingController _memoCtrl = TextEditingController();
  final GlobalKey _qrKey = GlobalKey();

  String? _shenfenId;
  bool _loadingBank = true;
  bool _savingQr = false;

  BigInt? _balanceFen;
  String? _balanceError;
  Timer? _balanceTimer;
  bool _balanceInFlight = false;

  OffchainClearingNodeRpc? get _nodeRpc =>
      widget.clearingNodeWssUrl == null || widget.clearingNodeWssUrl!.isEmpty
          ? null
          : OffchainClearingNodeRpc(widget.clearingNodeWssUrl!);

  @override
  void initState() {
    super.initState();
    _loadBoundBank();
  }

  @override
  void dispose() {
    _amountCtrl.dispose();
    _memoCtrl.dispose();
    _balanceTimer?.cancel();
    super.dispose();
  }

  Future<void> _loadBoundBank() async {
    final sfid = await ClearingBankPrefs.load(widget.wallet.walletIndex);
    if (!mounted) return;
    setState(() {
      _shenfenId = sfid;
      _loadingBank = false;
    });
    if (sfid != null && _nodeRpc != null) {
      _startBalancePolling();
    }
  }

  void _startBalancePolling() {
    _balanceTimer?.cancel();
    unawaited(_refreshBalance());
    _balanceTimer = Timer.periodic(_balancePollInterval, (_) {
      unawaited(_refreshBalance());
    });
  }

  Future<void> _refreshBalance() async {
    if (_balanceInFlight) return; // 防止上一次 RTT 未回就重入
    final rpc = _nodeRpc;
    if (rpc == null) return;
    _balanceInFlight = true;
    try {
      final fen = await rpc.queryBalance(widget.wallet.address);
      if (!mounted) return;
      setState(() {
        _balanceFen = BigInt.from(fen);
        _balanceError = null;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => _balanceError = '余额查询失败:$e');
    } finally {
      _balanceInFlight = false;
    }
  }

  String? _buildQrData() {
    final sfid = _shenfenId;
    if (sfid == null) return null;
    final amountText = AmountFormat.stripCommas(_amountCtrl.text);
    final memo = _memoCtrl.text.trim();
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final id = 'rcv_${DateTime.now().microsecondsSinceEpoch}';
    return QrEnvelope<UserTransferBody>(
      kind: QrKind.userTransfer,
      id: id,
      issuedAt: now,
      expiresAt: now + 600,
      body: UserTransferBody(
        address: widget.wallet.address,
        name: widget.wallet.walletName,
        amount: amountText,
        symbol: 'GMB',
        memo: memo,
        bank: sfid,
      ),
    ).toRawJson();
  }

  Future<void> _saveQrToGallery() async {
    if (_savingQr) return;
    setState(() => _savingQr = true);
    try {
      final boundary =
          _qrKey.currentContext?.findRenderObject() as RenderRepaintBoundary?;
      if (boundary == null) return;
      final image = await boundary.toImage(pixelRatio: 3.0);
      final byteData = await image.toByteData(format: ui.ImageByteFormat.png);
      if (byteData == null) return;
      final pngBytes = Uint8List.fromList(byteData.buffer.asUint8List());
      await Clipboard.setData(
          ClipboardData(text: '已生成收款码(${pngBytes.length} 字节 PNG)'));
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('QR 已生成,可使用系统截屏保存')),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('保存失败:$e')),
      );
    } finally {
      if (mounted) setState(() => _savingQr = false);
    }
  }

  String _fenToYuan(BigInt fen) {
    final neg = fen.isNegative;
    final abs = fen.abs();
    final yuan = abs ~/ BigInt.from(100);
    final cents = (abs % BigInt.from(100)).toInt();
    final s = '$yuan.${cents.toString().padLeft(2, '0')}';
    return neg ? '-$s' : s;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('收款(清算行)')),
      body: _body(),
    );
  }

  Widget _body() {
    if (_loadingBank) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_shenfenId == null) {
      return Padding(
        padding: const EdgeInsets.all(24),
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.info_outline, size: 48, color: Colors.grey),
              const SizedBox(height: 16),
              const Text(
                '尚未绑定清算行,请先从"选择/绑定清算行"完成绑定再回来收款',
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('返回'),
              ),
            ],
          ),
        ),
      );
    }
    return _readyView();
  }

  Widget _readyView() {
    final qrData = _buildQrData();
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        _kv('收款地址', widget.wallet.address),
        _kv('清算行', _shenfenId!),
        const SizedBox(height: 16),
        TextField(
          controller: _amountCtrl,
          keyboardType:
              const TextInputType.numberWithOptions(decimal: true),
          inputFormatters: [ThousandSeparatorFormatter()],
          decoration: const InputDecoration(
            labelText: '金额(元,可选)',
            hintText: '不填则由付款方输入',
            suffixText: 'GMB',
            border: OutlineInputBorder(),
          ),
          onChanged: (_) => setState(() {}),
        ),
        const SizedBox(height: 12),
        TextField(
          controller: _memoCtrl,
          decoration: const InputDecoration(
            labelText: '备注(可选)',
            border: OutlineInputBorder(),
          ),
          onChanged: (_) => setState(() {}),
        ),
        const SizedBox(height: 24),
        if (qrData != null)
          Center(
            child: RepaintBoundary(
              key: _qrKey,
              child: Container(
                padding: const EdgeInsets.all(12),
                color: Colors.white,
                child: CustomPaint(
                  size: const Size(240, 240),
                  painter: _QrPainter(qrData),
                ),
              ),
            ),
          ),
        const SizedBox(height: 12),
        Center(
          child: TextButton.icon(
            onPressed: _savingQr ? null : _saveQrToGallery,
            icon: const Icon(Icons.image_outlined),
            label: Text(_savingQr ? '生成中...' : '生成 QR 图片'),
          ),
        ),
        const Divider(height: 32),
        _balanceSection(),
      ],
    );
  }

  Widget _balanceSection() {
    if (_nodeRpc == null) {
      return const Text(
        '未配置清算行节点 WSS,余额不轮询;付款到账后请返回上一页重进刷新',
        style: TextStyle(color: Colors.grey),
      );
    }
    if (_balanceError != null) {
      return Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Icon(Icons.warning_amber_outlined,
              color: Colors.orange, size: 20),
          const SizedBox(width: 8),
          Expanded(child: Text(_balanceError!)),
        ],
      );
    }
    final txt = _balanceFen == null
        ? '加载中...'
        : '${_fenToYuan(_balanceFen!)} 元';
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          '可用余额(清算行,每 5s 刷新)',
          style: TextStyle(fontSize: 13, color: Colors.grey),
        ),
        const SizedBox(height: 4),
        Text(
          txt,
          style: const TextStyle(fontSize: 22, fontWeight: FontWeight.w700),
        ),
      ],
    );
  }

  Widget _kv(String key, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 72,
            child: Text(key, style: const TextStyle(color: Colors.grey)),
          ),
          Expanded(
            child: SelectableText(
              value,
              style: const TextStyle(fontFamily: 'monospace', fontSize: 13),
            ),
          ),
        ],
      ),
    );
  }
}

class _QrPainter extends CustomPainter {
  _QrPainter(this.data);

  final String data;

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

    for (var row = 0; row < moduleCount; row++) {
      for (var col = 0; col < moduleCount; col++) {
        if (qrImage.isDark(row, col)) {
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
  bool shouldRepaint(covariant _QrPainter oldDelegate) =>
      oldDelegate.data != data;
}
