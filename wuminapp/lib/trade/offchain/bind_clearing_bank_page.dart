import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/rpc/onchain_clearing_bank.dart';
import 'package:wuminapp_mobile/rpc/sfid_public.dart';
import 'package:wuminapp_mobile/trade/offchain/clearing_bank_prefs.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 扫码支付清算体系 Step 1 新增:绑定**清算行**(L2)确认页。
///
/// 中文注释:
/// - 清算行(L2)体系唯一绑定页。数据源:SFID 搜索结果传入的 `ClearingBankInfo`;
///   链上调用 `bind_clearing_bank(bank_main_address)`(call_index 30)。
///   原省储行绑定页 + `bind_clearing_institution` extrinsic 已在 Step 2b-iv-b
///   随老 pallet 一起删除。
/// - 绑定即开户,**无预存、无业务开户费**;链上仅产生付费调用 1 元/次。
/// - 本步**仅支持热钱包**(冷钱包扫码签名 Step 2 接入,与旧页面保持一致风格)。
/// - 2026-04-23:原来的清算行入口页 / 清算行列表页 / 收款码页已整体下线,本页目
///   前无活跃入口,等「设置清算行」真实交互落地时再复用。
class BindClearingBankPage extends StatefulWidget {
  const BindClearingBankPage({
    super.key,
    required this.wallet,
    required this.bank,
  });

  final WalletProfile wallet;
  final ClearingBankInfo bank;

  @override
  State<BindClearingBankPage> createState() => _BindClearingBankPageState();
}

class _BindClearingBankPageState extends State<BindClearingBankPage> {
  bool _submitting = false;

  @override
  Widget build(BuildContext context) {
    final b = widget.bank;
    final name = b.institutionName.isEmpty ? '(未命名机构)' : b.institutionName;
    return Scaffold(
      appBar: AppBar(title: const Text('绑定清算行')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          ListTile(
            title: const Text('清算行'),
            subtitle: Text(name),
          ),
          ListTile(
            title: const Text('所在地'),
            subtitle: Text('${b.province} ${b.city}'),
          ),
          ListTile(
            title: const Text('SFID'),
            subtitle: SelectableText(b.sfidId),
          ),
          ListTile(
            title: const Text('主账户地址'),
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
                : const Text('确认绑定'),
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
      // Step 2 增加冷钱包 QR 签名路径(参照 lib/wallet/ui/bind_clearing_page.dart)
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Step 1 暂仅支持热钱包绑定;冷钱包路径 Step 2 接入')),
      );
      return;
    }

    setState(() => _submitting = true);
    try {
      final mainAccountBytes = _hexToBytes(mainAccountHex);
      if (mainAccountBytes.length != 32) {
        throw Exception('主账户地址必须是 32 字节,实际 ${mainAccountBytes.length}');
      }
      final pubkeyBytes = _hexToBytes(wallet.pubkeyHex);
      if (pubkeyBytes.length != 32) {
        throw Exception('钱包公钥必须是 32 字节');
      }

      final walletManager = WalletManager();
      await walletManager.authenticateForSigning();

      final rpc = OnchainClearingBankRpc();
      final result = await rpc.bindClearingBank(
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(pubkeyBytes),
        bankMainAccount: Uint8List.fromList(mainAccountBytes),
        sign: (payload) =>
            walletManager.signWithWalletNoAuth(wallet.walletIndex, payload),
      );

      // 扫码支付 Step 2c-ii-a:绑定成功同步持久化 `shenfen_id`,供后续收款码页面
      // `bank` 字段回填。链上 `UserBank` 只存主账户,没 `shenfen_id`,不在这里落盘
      // 将无法在本地重建收款码。
      await ClearingBankPrefs.save(wallet.walletIndex, widget.bank.sfidId);

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('绑定已提交,tx=${_short(result.txHash)},等待链上确认')),
      );
      Navigator.pop(context, true);
    } on WalletAuthException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(e.message)));
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('绑定失败:$e')));
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

  static String _short(String h) {
    if (h.length <= 14) return h;
    return '${h.substring(0, 8)}…${h.substring(h.length - 4)}';
  }
}
