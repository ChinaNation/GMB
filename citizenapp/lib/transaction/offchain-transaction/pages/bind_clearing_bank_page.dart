import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:citizenapp/transaction/offchain-transaction/services/clearing_bank_directory.dart';
import 'package:citizenapp/transaction/offchain-transaction/rpc/onchain_clearing_bank_rpc.dart';
import 'package:citizenapp/rpc/cid_public.dart';
import 'package:citizenapp/transaction/offchain-transaction/services/clearing_bank_prefs.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 绑定**清算行**(L2)确认页。
///
///
/// - 清算行(L2)体系唯一绑定页。数据源:CID 搜索结果传入的 `ClearingBankInfo`;
///   链上调用 `bind_clearing_bank(bank_main_account)`(call_index 30)。
/// - 绑定即开户,**无预存、无业务开户费**;链上仅产生付费调用 1 元/次。
/// - 本步仅支持热钱包;冷钱包必须等绑定 payload 可独立展示和验证后再接入。
/// - 本页目前无活跃入口,等「设置清算行」真实交互落地时再复用。
class BindClearingBankPage extends StatefulWidget {
  const BindClearingBankPage({
    super.key,
    required this.wallet,
    required this.bank,
    this.endpoint,
    this.switchMode = false,
  });

  final WalletProfile wallet;
  final ClearingBankInfo bank;
  final ClearingBankNodeEndpoint? endpoint;
  final bool switchMode;

  @override
  State<BindClearingBankPage> createState() => _BindClearingBankPageState();
}

class _BindClearingBankPageState extends State<BindClearingBankPage> {
  bool _submitting = false;

  @override
  Widget build(BuildContext context) {
    final b = widget.bank;
    final title = b.displayTitle.isEmpty ? '(未设置全称)' : b.displayTitle;
    return Scaffold(
      appBar: AppBar(title: Text(widget.switchMode ? '切换清算行' : '绑定清算行')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          ListTile(
            title: const Text('清算行'),
            subtitle: Text(title),
          ),
          ListTile(
            title: const Text('所在地'),
            subtitle: Text('${b.provinceName} ${b.cityName}'),
          ),
          ListTile(
            title: const Text('CID'),
            subtitle: SelectableText(b.cidNumber),
          ),
          ListTile(
            title: const Text('主账户'),
            subtitle: SelectableText('0x${b.mainAccount ?? ''}'),
          ),
          const SizedBox(height: 12),
          const Card(
            child: Padding(
              padding: EdgeInsets.all(12),
              child: Text(
                '说明:\n'
                '· 绑定即开户,无需预存\n'
                '· 链上手续费 1 元/次(付费调用)\n'
                '· 同一时间只能绑定一家清算行\n'
                '· 切换前需把当前清算行存款全部提现',
                style: TextStyle(fontSize: 13, color: Colors.grey),
              ),
            ),
          ),
          const SizedBox(height: 24),
          FilledButton(
            onPressed: _submitting ? null : _confirmBind,
            child: _submitting
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : Text(widget.switchMode ? '确认切换' : '确认绑定'),
          ),
        ],
      ),
    );
  }

  Future<void> _confirmBind() async {
    final mainAccountHex = widget.bank.mainAccount;
    if (mainAccountHex == null || mainAccountHex.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('该清算行主账户尚未上链,无法绑定')),
      );
      return;
    }

    final wallet = widget.wallet;
    if (!wallet.isHotWallet) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('当前仅支持热钱包绑定；冷钱包绑定需可独立验证的签名协议')),
      );
      return;
    }

    setState(() => _submitting = true);
    try {
      final mainAccountBytes = _hexToBytes(mainAccountHex);
      if (mainAccountBytes.length != 32) {
        throw Exception('主账户必须是 32 字节,实际 ${mainAccountBytes.length}');
      }
      final pubkeyBytes = _hexToBytes(wallet.pubkeyHex);
      if (pubkeyBytes.length != 32) {
        throw Exception('钱包公钥必须是 32 字节');
      }

      final walletManager = WalletManager();

      final rpc = OnchainClearingBankRpc();
      final result = widget.switchMode
          ? await rpc.switchBank(
              fromAddress: wallet.address,
              signerPubkey: Uint8List.fromList(pubkeyBytes),
              newBankMainAccount: Uint8List.fromList(mainAccountBytes),
              sign: (payload) =>
                  walletManager.signWithWallet(wallet.walletIndex, payload),
            )
          : await rpc.bindClearingBank(
              fromAddress: wallet.address,
              signerPubkey: Uint8List.fromList(pubkeyBytes),
              bankMainAccount: Uint8List.fromList(mainAccountBytes),
              sign: (payload) =>
                  walletManager.signWithWallet(wallet.walletIndex, payload),
            );

      // 绑定成功后写入完整清算行快照。链上仍是最终权威,本地快照只用于
      // 手机端页面展示、充值提现和扫码付款时快速定位清算行节点端点。
      final endpoint = widget.endpoint;
      if (endpoint != null) {
        final now = DateTime.now().millisecondsSinceEpoch;
        await ClearingBankPrefs.saveSnapshot(
          wallet.walletIndex,
          ClearingBankBindingSnapshot(
            cidNumber: widget.bank.cidNumber,
            cidFullName: widget.bank.cidFullName,
            cidShortName: widget.bank.cidShortName,
            mainAccount: _normalizeHex(widget.bank.mainAccount ?? ''),
            feeAccount: widget.bank.feeAccount == null
                ? null
                : _normalizeHex(widget.bank.feeAccount!),
            peerId: endpoint.peerId,
            rpcDomain: endpoint.rpcDomain,
            rpcPort: endpoint.rpcPort,
            boundAtMs: now,
            lastVerifiedAtMs: now,
          ),
        );
      } else {
        await ClearingBankPrefs.save(wallet.walletIndex, widget.bank.cidNumber);
      }

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            '${widget.switchMode ? '切换' : '绑定'}已提交,tx=${_short(result.txHash)},等待链上确认',
          ),
        ),
      );
      Navigator.pop(context, true);
    } on WalletAuthException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text(e.message)));
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text('绑定失败:$e')));
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
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

  static String _normalizeHex(String input) {
    final text = input.startsWith('0x') ? input.substring(2) : input;
    return text.toLowerCase();
  }

  static String _short(String h) {
    if (h.length <= 14) return h;
    return '${h.substring(0, 8)}…${h.substring(h.length - 4)}';
  }
}
