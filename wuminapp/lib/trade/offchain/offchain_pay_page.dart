
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/rpc/offchain.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/trade/offchain/clearing_banks.dart';
import 'package:wuminapp_mobile/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/trade/local_tx_store.dart';
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';

/// 链下快捷支付确认页面。
///
/// 顾客扫描商户收款码后跳转到此页面，确认金额并签名支付。
/// 签名后提交到商户绑定的省储行节点，由省储行即时确认。
class OffchainPayPage extends StatefulWidget {
  const OffchainPayPage({
    super.key,
    required this.toAddress,
    this.amount,
    required this.bank,
    this.memo,
  });

  /// 收款方地址（商户）。
  final String toAddress;

  /// 金额（可选，商户设置了则直接显示，否则用户输入）。
  final String? amount;

  /// 清算省储行 shenfen_id。
  final String bank;

  /// 备注。
  final String? memo;

  @override
  State<OffchainPayPage> createState() => _OffchainPayPageState();
}

class _OffchainPayPageState extends State<OffchainPayPage> {
  static const double _edYuan = 1.11;

  final TextEditingController _amountController = TextEditingController();
  WalletProfile? _currentWallet;
  bool _loadingWallet = true;
  bool _submitting = false;

  /// 省储行真实费率（bp），从省储行节点查询。
  int _rateBp = 1;
  bool _loadingRate = true;
  bool _rateFailed = false; // 费率查询失败，禁止交易

  /// 后台轮询链上状态，确认上链后更新本地交易记录。
  ///
  /// fire-and-forget，不阻塞 UI。
  /// 省储行打包周期约 60 分钟，加缓冲共轮询 90 分钟（180 × 30s）。
  /// 连续查询失败 5 次后停止轮询并标记 pending_check。
  static Future<void> _pollOnchainStatus(String txId, String bank) async {
    const maxAttempts = 180; // 180 × 30s = 90 分钟
    const maxConsecutiveErrors = 5;
    int consecutiveErrors = 0;

    for (var i = 0; i < maxAttempts; i++) {
      await Future.delayed(const Duration(seconds: 30));
      try {
        final receipt = await OffchainRpc.queryTxStatus(bank, txId);
        consecutiveErrors = 0; // 成功一次就重置
        if (receipt.status == OffchainTxStatus.onchain) {
          await LocalTxStore.updateStatus(txId, 'onchain');
          debugPrint('[OffchainPoll] tx=$txId 已上链 (第${i + 1}次查询)');
          return;
        }
      } catch (e) {
        consecutiveErrors++;
        debugPrint('[OffchainPoll] tx=$txId 查询失败($consecutiveErrors/$maxConsecutiveErrors): $e');
        if (consecutiveErrors >= maxConsecutiveErrors) {
          debugPrint('[OffchainPoll] tx=$txId 连续失败$maxConsecutiveErrors次，标记 pending_check');
          await LocalTxStore.updateStatus(txId, 'pending_check');
          return;
        }
      }
    }
    // 90 分钟仍未上链，标记待人工检查
    debugPrint('[OffchainPoll] tx=$txId 轮询超时($maxAttempts次)，标记 pending_check');
    await LocalTxStore.updateStatus(txId, 'pending_check');
  }

  /// 商户是否预设了金额。
  bool get _amountPreset =>
      widget.amount != null && widget.amount!.isNotEmpty;

  @override
  void initState() {
    super.initState();
    if (_amountPreset) {
      _amountController.text = widget.amount!;
    }
    _loadWallet();
    _loadRate();
  }

  Future<void> _loadRate() async {
    try {
      final rate = await OffchainRpc.queryInstitutionRate(widget.bank);
      if (mounted) {
        setState(() {
          _rateBp = rate;
          _loadingRate = false;
          _rateFailed = false;
        });
      }
    } catch (e) {
      debugPrint('[OffchainPay] 费率查询失败: $e');
      if (mounted) {
        setState(() {
          _loadingRate = false;
          _rateFailed = true;
        });
      }
    }
  }

  @override
  void dispose() {
    _amountController.dispose();
    super.dispose();
  }

  Future<void> _loadWallet() async {
    final walletManager = WalletManager();
    final wallet = await walletManager.getWallet();
    if (!mounted) return;
    setState(() {
      _currentWallet = wallet;
      _loadingWallet = false;
    });
  }

  Future<void> _submit() async {
    if (_loadingWallet || _currentWallet == null) return;
    if (_loadingRate) return;
    if (_rateFailed) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('无法获取清算行费率，请稍后重试')),
      );
      return;
    }

    final amount = AmountFormat.tryParse(_amountController.text);
    if (amount == null || amount <= 0) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请输入有效金额')),
      );
      return;
    }

    // 预估手续费
    final fee = OffchainRpc.calculateOffchainFeeYuan(amount, _rateBp);
    final availableBalance = _currentWallet!.balance - _edYuan;
    if (amount + fee > availableBalance) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            '余额不足，可用余额：${AmountFormat.format(availableBalance, symbol: '')} 元',
          ),
        ),
      );
      return;
    }

    // 确认对话框
    final bankName = clearingBankName(widget.bank) ?? widget.bank;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('确认支付'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('收款方：${_formatAddress(widget.toAddress)}'),
            const SizedBox(height: 4),
            Text('清算行：$bankName'),
            const SizedBox(height: 4),
            Text('支付金额：${AmountFormat.format(amount, symbol: 'GMB')}'),
            const SizedBox(height: 4),
            Text('预估手续费：${AmountFormat.format(fee, symbol: 'GMB')}'),
            const Divider(height: 16),
            Text(
              '合计：${AmountFormat.format(amount + fee, symbol: 'GMB')}',
              style: const TextStyle(fontWeight: FontWeight.w700),
            ),
            if (widget.memo != null && widget.memo!.isNotEmpty) ...[
              const SizedBox(height: 4),
              Text('备注：${widget.memo}',
                  style: const TextStyle(color: AppTheme.textSecondary)),
            ],
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext, false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(dialogContext, true),
            child: const Text('确认支付'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;

    setState(() => _submitting = true);

    try {
      final wallet = _currentWallet!;
      final txId = QrSigner.generateRequestId(prefix: 'offchain-');

      // 构造签名
      final Future<Uint8List> Function(Uint8List payload) signCallback;

      if (wallet.isHotWallet) {
        final walletManager = WalletManager();
        await walletManager.authenticateForSigning();
        signCallback = (payload) =>
            walletManager.signWithWalletNoAuth(wallet.walletIndex, payload);
      } else {
        // 冷钱包:使用统一协议 WUMIN_QR_V1 kind=sign_request
        signCallback = (Uint8List payload) async {
          final qrSigner = QrSigner();
          final requestId = QrSigner.generateRequestId(prefix: 'offpay-');
          final amountFormatted = AmountFormat.format(
              AmountFormat.tryParse(_amountController.text) ?? 0,
              symbol: '')
              .trim();
          final rv = await ChainRpc().fetchRuntimeVersion();
          final request = qrSigner.buildRequest(
            requestId: requestId,
            address: wallet.address,
            pubkey: '0x${wallet.pubkeyHex}',
            payloadHex: '0x${_toHex(payload)}',
            specVersion: rv.specVersion,
            display: SignDisplay(
              action: 'offchain_pay',
              summary: '扫码支付 $amountFormatted GMB 给 ${widget.toAddress}',
              fields: [
                SignDisplayField(label: '收款方', value: widget.toAddress),
                SignDisplayField(
                    label: '金额', value: '$amountFormatted GMB'),
                SignDisplayField(label: '清算行', value: bankName),
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

          return Uint8List.fromList(_hexToBytes(response.body.signature));
        };
      }

      // 构造链下支付 payload（pallet=21, call=99 统一格式）
      final amountFen = (amount * 100).round();
      final feeFen = (fee * 100).round();
      final payerPubkey = Uint8List.fromList(_hexToBytes(wallet.pubkeyHex));
      final recipientPubkey = Uint8List.fromList(
        Keyring().decodeAddress(widget.toAddress),
      );
      final txIdBytes = Uint8List(32);
      final txIdRaw = txId.codeUnits;
      for (var i = 0; i < txIdRaw.length && i < 32; i++) {
        txIdBytes[i] = txIdRaw[i];
      }

      final payloadBytes = OffchainRpc.buildPayload(
        payerPubkey: payerPubkey,
        recipientPubkey: recipientPubkey,
        amountFen: amountFen,
        feeFen: feeFen,
        txIdBytes: txIdBytes,
        bankShenfenId: widget.bank,
      );
      final signature = await signCallback(payloadBytes);

      // 提交到省储行
      final receipt = await OffchainRpc.submitSignedTx(
        bankShenfenId: widget.bank,
        payerAddress: wallet.address,
        recipientAddress: widget.toAddress,
        amountFen: amountFen,
        feeFen: feeFen,
        signature: _toHex(signature),
        txId: txId,
      );

      if (!mounted) return;

      if (receipt.status == OffchainTxStatus.confirmed) {
        // 写入本地交易记录
        final localEntity = LocalTxEntity()
          ..txId = txId
          ..walletAddress = wallet.address
          ..txType = 'offchain_pay'
          ..direction = 'out'
          ..fromAddress = wallet.address
          ..toAddress = widget.toAddress
          ..amountYuan = amount
          ..feeYuan = fee
          ..bankShenfenId = widget.bank
          ..status = 'confirmed'
          ..createdAtMillis = DateTime.now().millisecondsSinceEpoch
          ..confirmedAtMillis = DateTime.now().millisecondsSinceEpoch;
        await LocalTxStore.insert(localEntity);

        // 后台轮询链上状态，确认上链后更新本地记录
        _pollOnchainStatus(txId, widget.bank);

        if (!mounted) return;
        await showDialog<void>(
          context: context,
          builder: (successContext) => AlertDialog(
            title: const Row(
              children: [
                Icon(Icons.check_circle, color: AppTheme.success, size: 24),
                SizedBox(width: 8),
                Text('支付成功'),
              ],
            ),
            content: Text('交易已由 $bankName 确认\n交易编号：$txId'),
            actions: [
              FilledButton(
                onPressed: () {
                  Navigator.pop(successContext);
                  Navigator.pop(context);
                },
                child: const Text('完成'),
              ),
            ],
          ),
        );
      } else {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('支付失败：${receipt.message ?? "未知错误"}')),
        );
      }
    } on WalletAuthException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(e.message)),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('支付失败：$e')),
      );
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
  }

  String _formatAddress(String address) {
    if (address.length <= 16) return address;
    return '${address.substring(0, 8)}...${address.substring(address.length - 8)}';
  }

  @override
  Widget build(BuildContext context) {
    final bankName = clearingBankName(widget.bank) ?? widget.bank;

    return Scaffold(
      appBar: AppBar(
        title: const Text('扫码支付'),
        centerTitle: true,
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            children: [
              // 支付信息卡片
              Container(
                width: double.infinity,
                decoration: AppTheme.cardDecoration(),
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      // 余额
                      if (_currentWallet != null)
                        Padding(
                          padding: const EdgeInsets.only(bottom: 16),
                          child: Row(
                            children: [
                              Text(
                                '可用余额：${AmountFormat.format(_currentWallet!.balance, symbol: '')} 元',
                                style: const TextStyle(
                                  fontSize: 13,
                                  color: AppTheme.textSecondary,
                                ),
                              ),
                              const Spacer(),
                              Container(
                                width: 40,
                                height: 18,
                                decoration: BoxDecoration(
                                  color: AppTheme.success,
                                  borderRadius: BorderRadius.circular(100),
                                ),
                                child: const Center(
                                  child: Text(
                                    '链下',
                                    style: TextStyle(
                                      fontSize: 8,
                                      color: Colors.white,
                                      fontWeight: FontWeight.w600,
                                      height: 1.0,
                                    ),
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),

                      // 收款方
                      _buildInfoRow('收款方', _formatAddress(widget.toAddress)),
                      const SizedBox(height: 12),

                      // 清算行
                      _buildInfoRow('清算行', bankName),
                      if (_rateFailed) ...[
                        const SizedBox(height: 8),
                        Container(
                          padding: const EdgeInsets.all(10),
                          decoration: BoxDecoration(
                            color: AppTheme.danger.withValues(alpha: 0.08),
                            borderRadius: BorderRadius.circular(8),
                            border: Border.all(color: AppTheme.danger.withValues(alpha: 0.3)),
                          ),
                          child: const Row(
                            children: [
                              Icon(Icons.error_outline, size: 16, color: AppTheme.danger),
                              SizedBox(width: 6),
                              Expanded(child: Text(
                                '无法获取清算行费率，暂时无法支付',
                                style: TextStyle(fontSize: 12, color: AppTheme.danger),
                              )),
                            ],
                          ),
                        ),
                      ],
                      const SizedBox(height: 12),

                      // 备注
                      if (widget.memo != null && widget.memo!.isNotEmpty) ...[
                        _buildInfoRow('备注', widget.memo!),
                        const SizedBox(height: 12),
                      ],

                      // 金额输入
                      if (_amountPreset)
                        _buildInfoRow(
                            '金额', AmountFormat.formatString(widget.amount!),
                            bold: true)
                      else
                        TextField(
                          controller: _amountController,
                          keyboardType: const TextInputType.numberWithOptions(decimal: true),
                          inputFormatters: [ThousandSeparatorFormatter()],
                          style: const TextStyle(color: AppTheme.textPrimary),
                          decoration: const InputDecoration(
                            labelText: '支付金额',
                            suffixText: 'GMB',
                          ),
                        ),
                    ],
                  ),
                ),
              ),
              const Spacer(),

              // 支付按钮
              SizedBox(
                width: double.infinity,
                child: FilledButton(
                  onPressed: (_submitting || _loadingWallet || _loadingRate || _rateFailed || _currentWallet == null)
                      ? null
                      : _submit,
                  child: Padding(
                    padding: const EdgeInsets.symmetric(vertical: 4),
                    child: Text(
                      _submitting ? '支付中...' : '确认支付',
                      style: const TextStyle(fontSize: 16),
                    ),
                  ),
                ),
              ),
              const SizedBox(height: 16),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildInfoRow(String label, String value, {bool bold = false}) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 60,
          child: Text(
            label,
            style: const TextStyle(
              fontSize: 14,
              color: AppTheme.textSecondary,
            ),
          ),
        ),
        Expanded(
          child: Text(
            value,
            style: TextStyle(
              fontSize: 14,
              color: AppTheme.textPrimary,
              fontWeight: bold ? FontWeight.w700 : FontWeight.normal,
            ),
          ),
        ),
      ],
    );
  }
}

String _toHex(List<int> bytes) {
  const chars = '0123456789abcdef';
  final buf = StringBuffer();
  for (final b in bytes) {
    buf
      ..write(chars[(b >> 4) & 0x0f])
      ..write(chars[b & 0x0f]);
  }
  return buf.toString();
}

List<int> _hexToBytes(String input) {
  final text = input.startsWith('0x') ? input.substring(2) : input;
  if (text.isEmpty || text.length.isOdd) return const <int>[];
  final out = <int>[];
  for (var i = 0; i < text.length; i += 2) {
    out.add(int.parse(text.substring(i, i + 2), radix: 16));
  }
  return out;
}
