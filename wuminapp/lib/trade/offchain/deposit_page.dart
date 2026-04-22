import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/rpc/onchain_clearing_bank.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 扫码支付清算体系 Step 1 新增:**充值** L3 自持账户 → 清算行主账户。
///
/// 中文注释:
/// - 调链上 `deposit(amount)`(call_index 31)。
/// - 链上费按金额 0.1% 最低 0.1 元(链上资金交易,由 `PowTxAmountExtractor` 处理)。
/// - 本步仅支持热钱包,冷钱包 QR 签名留 Step 2。
class DepositPage extends StatefulWidget {
  const DepositPage({super.key, required this.wallet});

  final WalletProfile wallet;

  @override
  State<DepositPage> createState() => _DepositPageState();
}

class _DepositPageState extends State<DepositPage> {
  final TextEditingController _amountCtrl = TextEditingController();
  bool _submitting = false;

  @override
  void dispose() {
    _amountCtrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('充值到清算行')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Text('从自持账户转入绑定的清算行存款。',
                style: TextStyle(color: Colors.grey)),
            const SizedBox(height: 16),
            TextField(
              controller: _amountCtrl,
              keyboardType: const TextInputType.numberWithOptions(decimal: true),
              decoration: const InputDecoration(
                labelText: '充值金额(元)',
                hintText: '例如 100.00',
              ),
            ),
            const SizedBox(height: 24),
            FilledButton(
              onPressed: _submitting ? null : _submit,
              child: _submitting
                  ? const SizedBox(
                      width: 20,
                      height: 20,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Text('确认充值'),
            ),
            const SizedBox(height: 12),
            const Text(
              '链上费:金额 × 0.1%(最低 0.1 元)',
              style: TextStyle(fontSize: 12, color: Colors.grey),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _submit() async {
    final amountFen = _parseAmountToFen(_amountCtrl.text);
    if (amountFen == null || amountFen <= BigInt.zero) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请输入有效的充值金额(元)')),
      );
      return;
    }

    final wallet = widget.wallet;
    if (!wallet.isHotWallet) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Step 1 暂仅支持热钱包充值;冷钱包路径 Step 2 接入')),
      );
      return;
    }

    setState(() => _submitting = true);
    try {
      final pubkeyBytes = _hexToBytes(wallet.pubkeyHex);
      if (pubkeyBytes.length != 32) {
        throw Exception('钱包公钥必须是 32 字节');
      }
      final walletManager = WalletManager();
      await walletManager.authenticateForSigning();

      final rpc = OnchainClearingBankRpc();
      final result = await rpc.deposit(
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(pubkeyBytes),
        amountFen: amountFen,
        sign: (payload) =>
            walletManager.signWithWalletNoAuth(wallet.walletIndex, payload),
      );

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('充值已提交,tx=${_short(result.txHash)}')),
      );
      Navigator.pop(context, true);
    } on WalletAuthException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(e.message)));
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('充值失败:$e')));
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
  }

  /// 把"元"字符串转为 BigInt 分。`100.5` → 10050。
  static BigInt? _parseAmountToFen(String input) {
    final s = input.trim();
    if (s.isEmpty) return null;
    final dotIdx = s.indexOf('.');
    String intPart;
    String fracPart;
    if (dotIdx < 0) {
      intPart = s;
      fracPart = '00';
    } else {
      intPart = s.substring(0, dotIdx);
      final raw = s.substring(dotIdx + 1);
      if (raw.isEmpty) {
        fracPart = '00';
      } else if (raw.length == 1) {
        fracPart = '${raw}0';
      } else if (raw.length == 2) {
        fracPart = raw;
      } else {
        // 超过 2 位小数:截断(不进行四舍五入,避免与链上 round_div 冲突)
        fracPart = raw.substring(0, 2);
      }
    }
    if (intPart.isEmpty) intPart = '0';
    if (!RegExp(r'^\d+$').hasMatch(intPart) ||
        !RegExp(r'^\d{2}$').hasMatch(fracPart)) {
      return null;
    }
    return BigInt.parse('$intPart$fracPart');
  }

  static List<int> _hexToBytes(String input) {
    final text = input.startsWith('0x') ? input.substring(2) : input;
    if (text.isEmpty || text.length.isOdd) return const <int>[];
    final out = <int>[];
    for (var i = 0; i < text.length; i += 2) {
      out.add(int.parse(text.substring(i, i + 2), radix: 16));
    }
    return out;
  }

  static String _short(String h) {
    if (h.length <= 14) return h;
    return '${h.substring(0, 8)}…${h.substring(h.length - 4)}';
  }
}
