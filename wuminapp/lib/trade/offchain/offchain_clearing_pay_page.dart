import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/offchain_clearing.dart';
import 'package:wuminapp_mobile/rpc/sfid_public.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/trade/offchain/payment_intent.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 扫码支付清算体系 Step 2c-i:**扫码付款确认页**(替代已删除的 `offchain_pay_page.dart`)。
///
/// 中文注释:
/// - 入口:`onchain_trade_page.dart::_openOffchainPay` 扫商户码成功后跳转过来
///   (商户码 `UserTransferBody` 的 `bank` 字段是收款方清算行 `shenfen_id`)。
/// - Step 2c-i 范围:**同行**扫码付款(`payer_bank == recipient_bank`)。跨行
///   (Step 3)在校验步弹"暂不支持"提示并拒绝提交。
/// - 冷钱包(Step 2c-iii)本步**不做**;热钱包之外直接拦截。
/// - 流程:
///   1. 连清算行节点 RPC,查 `offchain_queryUserBank(user)` 得付款方清算行
///      `payer_bank` SS58(未绑定 → 结束);
///   2. 通过 SFID `/api/v1/app/clearing-banks/search` 把 QR 里的收款方
///      `shenfen_id` 解析为 `recipient_bank` 主账户 hex;
///   3. 同行校验(`payer_bank` == `recipient_bank` hex → SS58 对比);
///   4. 查 `offchain_queryFeeRate(payer_bank)` 得 `(rate_bp, min_fee_fen)`,本地
///      计算 `fee_fen`(与 runtime 一致的四舍五入);
///   5. 展示确认 UI(金额 / 手续费 / 合计 / 收款方地址 / 备注);
///   6. 用户点确认 → 查 `offchain_queryNextNonce(user)` → 构造
///      `NodePaymentIntent`(随机 tx_id + `expires_at = currentBlock + 100`) →
///      `signingHash()` → 热钱包 `signWithWalletNoAuth` → 提交
///      `offchain_submitPayment(intent_hex, sig_hex)` → 显示结果。
class OffchainClearingPayPage extends StatefulWidget {
  const OffchainClearingPayPage({
    super.key,
    required this.wallet,
    required this.toAddress,
    required this.recipientBankShenfenId,
    required this.clearingNodeWssUrl,
    required this.sfidBaseUrl,
    this.initialAmountYuan,
    this.memo,
  });

  /// 付款方当前钱包(仅支持热钱包,冷钱包流程 Step 2c-iii)。
  final WalletProfile wallet;

  /// 商户 QR `UserTransferBody.address` 收款方地址(SS58 或 0x hex pubkey)。
  final String toAddress;

  /// 商户 QR `UserTransferBody.bank` 收款方清算行 `shenfen_id`。
  final String recipientBankShenfenId;

  /// 付款方绑定的清算行节点 WSS URL(由调用方传入,已知绑定才进得了本页)。
  final String clearingNodeWssUrl;

  /// SFID 后端 baseUrl(用于按 `shenfen_id` 查收款方清算行主账户地址)。
  final String sfidBaseUrl;

  /// 商户 QR 预填金额(元,字符串)。空 → 由用户输入。
  final String? initialAmountYuan;

  final String? memo;

  @override
  State<OffchainClearingPayPage> createState() => _OffchainClearingPayPageState();
}

enum _PageState { loading, ready, submitting, done, error }

class _OffchainClearingPayPageState extends State<OffchainClearingPayPage> {
  static const int _expiresInBlocks = 100; // ≈ 10 分钟(6s/block),签名离提交留足余量

  _PageState _state = _PageState.loading;
  String _errorMessage = '';

  // 预取:付款方绑定的清算行主账户 SS58
  String? _payerBankSs58;
  // 预取:QR 收款方 shenfen_id 解析出的清算行主账户 hex
  String? _recipientBankHex;
  // 预取:费率
  int _rateBp = 0;
  int _minFeeFen = 1;
  // 预取:当前最新块高(用于 expires_at)
  int _currentBlockNumber = 0;

  // 金额输入(元,字符串)
  final TextEditingController _amountCtrl = TextEditingController();

  // 提交结果
  String? _resultTxId;

  late final OffchainClearingNodeRpc _nodeRpc =
      OffchainClearingNodeRpc(widget.clearingNodeWssUrl);

  @override
  void initState() {
    super.initState();
    _amountCtrl.text = widget.initialAmountYuan ?? '';
    _loadPrerequisites();
  }

  @override
  void dispose() {
    _amountCtrl.dispose();
    super.dispose();
  }

  Future<void> _loadPrerequisites() async {
    try {
      // 1. 付款方绑定的清算行
      final payerBank = await _nodeRpc.queryUserBank(widget.wallet.address);
      if (payerBank == null || payerBank.isEmpty) {
        _setError('您尚未绑定清算行,请先返回"选择/绑定清算行"完成绑定');
        return;
      }

      // 2. 收款方清算行(通过 SFID 按 shenfen_id 查)
      final sfid = SfidPublicApi(baseUrl: widget.sfidBaseUrl);
      try {
        final search = await sfid.searchClearingBanks(
          keyword: widget.recipientBankShenfenId,
        );
        final match = search.items.firstWhere(
          (b) => b.sfidId == widget.recipientBankShenfenId,
          orElse: () => throw Exception('收款方清算行 ${widget.recipientBankShenfenId} 未在 SFID 系统查到'),
        );
        final recHex = match.mainAccount;
        if (recHex == null || recHex.isEmpty) {
          _setError('收款方清算行未上链,无法付款');
          return;
        }
        _recipientBankHex = recHex;
      } finally {
        sfid.close();
      }

      // 3. 同行校验
      final payerHex = _ss58ToHex(payerBank);
      if (_normalizeHex(payerHex) != _normalizeHex(_recipientBankHex!)) {
        _setError('收款方清算行与您绑定的清算行不同,Step 1 仅支持同行扫码支付');
        return;
      }
      _payerBankSs58 = payerBank;

      // 4. 费率
      final rate = await _nodeRpc.queryFeeRate(payerBank);
      if (rate.rateBp <= 0) {
        _setError('清算行费率未配置(rate_bp=${rate.rateBp}),请联系清算行运维');
        return;
      }
      _rateBp = rate.rateBp;
      _minFeeFen = rate.minFeeFen;

      // 5. 当前块高
      final latest = await ChainRpc().fetchLatestBlock();
      _currentBlockNumber = latest.blockNumber;

      if (mounted) {
        setState(() => _state = _PageState.ready);
      }
    } catch (e) {
      _setError('加载支付信息失败:$e');
    }
  }

  void _setError(String msg) {
    if (!mounted) return;
    setState(() {
      _state = _PageState.error;
      _errorMessage = msg;
    });
  }

  BigInt? _parseAmountFen() {
    final txt = _amountCtrl.text.trim();
    if (txt.isEmpty) return null;
    final yuan = double.tryParse(txt);
    if (yuan == null || yuan <= 0) return null;
    final fen = BigInt.from((yuan * 100).round());
    return fen > BigInt.zero ? fen : null;
  }

  BigInt? _computeFeeFen(BigInt amountFen) {
    try {
      return NodePaymentIntent.calcFeeFen(
        amountFen: amountFen,
        rateBp: _rateBp,
        minFeeFen: _minFeeFen,
      );
    } catch (_) {
      return null;
    }
  }

  Future<void> _confirmAndSubmit() async {
    final amountFen = _parseAmountFen();
    if (amountFen == null) {
      _showSnack('请输入有效金额');
      return;
    }
    final feeFen = _computeFeeFen(amountFen);
    if (feeFen == null) {
      _showSnack('手续费计算失败');
      return;
    }

    setState(() => _state = _PageState.submitting);
    try {
      // 6. nonce
      final nonce = await _nodeRpc.queryNextNonce(widget.wallet.address);

      // 7. 构造 intent
      final payer = hexToBytes(widget.wallet.pubkeyHex);
      if (payer.length != 32) {
        throw Exception('钱包公钥长度异常:${payer.length}');
      }
      final recipient = _decodeAccount(widget.toAddress);
      final payerBankBytes = _ss58ToBytes(_payerBankSs58!);
      final recipientBankBytes = hexToBytes(_ensure0x(_recipientBankHex!));

      final intent = NodePaymentIntent(
        txId: NodePaymentIntent.randomTxId(),
        payer: payer,
        payerBank: payerBankBytes,
        recipient: recipient,
        recipientBank: recipientBankBytes,
        amount: amountFen,
        fee: feeFen,
        nonce: BigInt.from(nonce),
        expiresAt: _currentBlockNumber + _expiresInBlocks,
      );

      // 8. 签名:热钱包直签 / 冷钱包走 QR 两段握手(Step 2c-iii)
      final sig = await _signSigningHash(
        signingHash: intent.signingHash(),
        amountFen: amountFen,
        feeFen: feeFen,
      );
      if (sig.length != 64) {
        throw Exception('签名长度异常:${sig.length}');
      }

      // 9. 提交
      final resp = await _nodeRpc.submitPayment(
        intentHex: bytesToHex(intent.scaleEncode()),
        payerSigHex: bytesToHex(sig),
      );

      if (!mounted) return;
      setState(() {
        _resultTxId = resp.txId;
        _state = _PageState.done;
      });
    } catch (e) {
      _setError('提交失败:$e');
    }
  }

  void _showSnack(String msg) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(msg)));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('扫码付款(清算行)')),
      body: _body(),
    );
  }

  Widget _body() {
    switch (_state) {
      case _PageState.loading:
        return const Center(child: CircularProgressIndicator());
      case _PageState.error:
        return _errorView();
      case _PageState.submitting:
        return const Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              CircularProgressIndicator(),
              SizedBox(height: 12),
              Text('正在签名并提交到清算行...'),
            ],
          ),
        );
      case _PageState.done:
        return _doneView();
      case _PageState.ready:
        return _confirmView();
    }
  }

  Widget _errorView() {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.error_outline, size: 48, color: Colors.red),
            const SizedBox(height: 16),
            Text(_errorMessage, textAlign: TextAlign.center),
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

  Widget _doneView() {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.check_circle_outline, size: 48, color: Colors.green),
            const SizedBox(height: 16),
            const Text('支付已受理,清算行会在下一批次上链'),
            const SizedBox(height: 8),
            SelectableText('tx_id: ${_resultTxId ?? ''}',
                style: const TextStyle(fontFamily: 'monospace', fontSize: 12)),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('完成'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _confirmView() {
    final amountFen = _parseAmountFen();
    final feeFen = (amountFen != null) ? _computeFeeFen(amountFen) : null;
    final totalFen = (amountFen != null && feeFen != null)
        ? amountFen + feeFen
        : null;
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _kv('收款方地址', widget.toAddress),
          _kv('收款方清算行', widget.recipientBankShenfenId),
          if (widget.memo != null && widget.memo!.isNotEmpty)
            _kv('备注', widget.memo!),
          const Divider(height: 32),
          TextField(
            controller: _amountCtrl,
            enabled: widget.initialAmountYuan == null ||
                widget.initialAmountYuan!.isEmpty,
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            decoration: const InputDecoration(
              labelText: '金额(元)',
              border: OutlineInputBorder(),
            ),
            onChanged: (_) => setState(() {}),
          ),
          const SizedBox(height: 16),
          _kv('费率', '$_rateBp bp (万分之 $_rateBp)'),
          _kv('手续费', feeFen == null ? '—' : '${_fenToYuan(feeFen)} 元'),
          _kv('合计扣款', totalFen == null ? '—' : '${_fenToYuan(totalFen)} 元'),
          const SizedBox(height: 24),
          ElevatedButton(
            onPressed: (amountFen != null && feeFen != null)
                ? _confirmAndSubmit
                : null,
            child: const Text('确认并签名付款'),
          ),
        ],
      ),
    );
  }

  Widget _kv(String key, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 96,
            child: Text(key, style: const TextStyle(color: Colors.grey)),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(fontFamily: 'monospace'),
            ),
          ),
        ],
      ),
    );
  }

  // ─── 地址编解码工具 ───

  /// 把 SS58 地址转成 `0x` 前缀的 hex(小写)。
  String _ss58ToHex(String ss58) {
    final bytes = Keyring().decodeAddress(ss58);
    return bytesToHex(Uint8List.fromList(bytes));
  }

  /// 把 SS58 地址转成 32 字节。
  Uint8List _ss58ToBytes(String ss58) {
    final bytes = Keyring().decodeAddress(ss58);
    if (bytes.length != 32) {
      throw Exception('SS58 解码长度异常:${bytes.length}');
    }
    return Uint8List.fromList(bytes);
  }

  /// 归一 hex(去 0x 前缀 + 小写)用于比较。
  String _normalizeHex(String hex) {
    final t = hex.startsWith('0x') ? hex.substring(2) : hex;
    return t.toLowerCase();
  }

  String _ensure0x(String hex) =>
      hex.startsWith('0x') ? hex : '0x$hex';

  /// 热钱包 / 冷钱包统一签名入口。
  ///
  /// - 热钱包:先走 `WalletManager.authenticateForSigning`(生物/密码) +
  ///   `signWithWalletNoAuth(walletIndex, signingHash)` 直接返 64 字节签名。
  /// - 冷钱包:用 `QrSigner` 构造 `sign_request` envelope(`payload_hex` = 32
  ///   字节 signing_hash),通过 `QrSignSessionPage` 两段 QR 握手:热钱包展示
  ///   `sign_request` → wumin 冷钱包扫码 → 显示交易详情(action=`offchain_clearing_pay`)
  ///   → 冷钱包 sr25519 签名 → 生成 `sign_response` QR → 热钱包扫码取签名。
  /// - 冷钱包对 `payload_hex` **盲签**(只把 32 字节当字节流处理),与
  ///   `NodePaymentIntent` 的 SCALE 布局解耦,无需修改 wumin 冷钱包 app。
  Future<Uint8List> _signSigningHash({
    required Uint8List signingHash,
    required BigInt amountFen,
    required BigInt feeFen,
  }) async {
    final wallet = widget.wallet;
    if (wallet.isHotWallet) {
      final manager = WalletManager();
      await manager.authenticateForSigning();
      return manager.signWithWalletNoAuth(wallet.walletIndex, signingHash);
    }

    // 冷钱包路径
    final qrSigner = QrSigner();
    final requestId = QrSigner.generateRequestId(prefix: 'offchain-pay-');
    final rv = await ChainRpc().fetchRuntimeVersion();
    final request = qrSigner.buildRequest(
      requestId: requestId,
      address: wallet.address,
      pubkey: '0x${wallet.pubkeyHex}',
      payloadHex: bytesToHex(signingHash),
      specVersion: rv.specVersion,
      display: SignDisplay(
        action: 'offchain_clearing_pay',
        summary: '清算行扫码付款 ${_fenToYuan(amountFen)} 元 → ${widget.toAddress}',
        fields: [
          SignDisplayField(label: '金额', value: '${_fenToYuan(amountFen)} 元'),
          SignDisplayField(label: '手续费', value: '${_fenToYuan(feeFen)} 元'),
          SignDisplayField(
            label: '合计扣款',
            value: '${_fenToYuan(amountFen + feeFen)} 元',
          ),
          SignDisplayField(label: '收款方', value: widget.toAddress),
          SignDisplayField(
            label: '收款清算行',
            value: widget.recipientBankShenfenId,
          ),
        ],
      ),
    );
    final requestJson = qrSigner.encodeRequest(request);

    if (!mounted) throw Exception('页面已关闭');
    final response = await Navigator.push<SignResponseEnvelope>(
      context,
      MaterialPageRoute(
        builder: (_) => QrSignSessionPage(
          request: request,
          requestJson: requestJson,
          expectedPubkey: '0x${wallet.pubkeyHex}',
        ),
      ),
    );
    if (response == null) {
      throw Exception('签名已取消');
    }
    return hexToBytes(response.body.signature);
  }

  /// 分转元(两位小数)。本地化展示,不参与任何链上金额计算。
  String _fenToYuan(BigInt fen) {
    // 避免 BigInt → double 丢精度(最大 9e18 分 ≈ 2^63,double 精度 2^53)。
    // 这里走字符串分隔:整数部分 / 小数部分。
    final neg = fen.isNegative;
    final abs = fen.abs();
    final yuan = abs ~/ BigInt.from(100);
    final cents = (abs % BigInt.from(100)).toInt();
    final s = '$yuan.${cents.toString().padLeft(2, '0')}';
    return neg ? '-$s' : s;
  }

  /// QR 里的 `toAddress` 既可能是 SS58,也可能是 `0x` hex pubkey,两种都兼容。
  Uint8List _decodeAccount(String address) {
    final t = address.trim();
    if (t.startsWith('0x')) {
      final bytes = hexToBytes(t);
      if (bytes.length != 32) {
        throw Exception('hex 地址长度必须 32 字节,实际 ${bytes.length}');
      }
      return bytes;
    }
    return _ss58ToBytes(t);
  }
}
