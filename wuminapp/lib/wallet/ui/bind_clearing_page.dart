import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/rpc/onchain.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/trade/offchain/clearing_banks.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';

/// 绑定清算省储行选择页面。
///
/// 用户从 43 个省储行中选择一个作为清算行。
/// 选择后调用链上 bind_clearing_institution extrinsic 提交绑定。
class BindClearingPage extends StatefulWidget {
  const BindClearingPage({
    super.key,
    this.currentShenfenId,
    required this.wallet,
  });

  /// 当前已绑定的省储行 shenfen_id（高亮显示）。
  final String? currentShenfenId;

  /// 当前钱包（用于签名提交）。
  final WalletProfile wallet;

  @override
  State<BindClearingPage> createState() => _BindClearingPageState();
}

class _BindClearingPageState extends State<BindClearingPage> {
  String _searchText = '';
  bool _submitting = false;

  List<ClearingBank> get _filteredBanks {
    if (_searchText.isEmpty) return clearingBanks;
    return clearingBanks
        .where((b) => b.shenfenName.contains(_searchText))
        .toList();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('选择清算省储行'),
        centerTitle: true,
      ),
      body: Column(
        children: [
          // 搜索框
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 8),
            child: TextField(
              onChanged: (v) => setState(() => _searchText = v.trim()),
              decoration: const InputDecoration(
                hintText: '搜索省储行名称',
                prefixIcon: Icon(Icons.search, size: 20),
                isDense: true,
              ),
            ),
          ),
          if (_submitting)
            const Padding(
              padding: EdgeInsets.all(16),
              child: Center(child: CircularProgressIndicator()),
            ),
          // 省储行列表
          Expanded(
            child: ListView.separated(
              itemCount: _filteredBanks.length,
              separatorBuilder: (_, __) =>
                  const Divider(height: 1, indent: 16, endIndent: 16),
              itemBuilder: (context, index) {
                final bank = _filteredBanks[index];
                final isCurrent = bank.shenfenId == widget.currentShenfenId;
                final isEnabled = bank.enabled;
                return ListTile(
                  enabled: isEnabled && !_submitting,
                  title: Text(
                    bank.shenfenName,
                    style: TextStyle(
                      fontSize: 15,
                      fontWeight:
                          isCurrent ? FontWeight.w700 : FontWeight.normal,
                      color: isCurrent
                          ? AppTheme.primary
                          : isEnabled
                              ? AppTheme.textPrimary
                              : AppTheme.textTertiary,
                    ),
                  ),
                  trailing: isCurrent
                      ? const Icon(Icons.check, color: AppTheme.primary, size: 20)
                      : !isEnabled
                          ? const Text(
                              '未开通',
                              style: TextStyle(
                                fontSize: 12,
                                color: AppTheme.textTertiary,
                              ),
                            )
                          : null,
                  onTap: () {
                    if (!isEnabled) return;
                    if (isCurrent) {
                      Navigator.pop(context);
                      return;
                    }
                    _confirmBind(bank);
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _confirmBind(ClearingBank bank) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('确认绑定'),
        content: Text(
          '确认将清算省储行绑定为「${bank.shenfenName}」？\n\n'
          '每次绑定或更换收取 0.1 元手续费。',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext, false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(dialogContext, true),
            child: const Text('确认绑定'),
          ),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;

    setState(() => _submitting = true);

    // 余额预检查：需要 0.1 元手续费 + 1.11 元 ED = 1.21 元
    try {
      final balance = await ChainRpc().fetchBalance(widget.wallet.pubkeyHex);
      if (balance < 1.21) {
        if (!mounted) return;
        setState(() => _submitting = false);
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('余额不足，绑定需至少 1.21 元（手续费 0.1 元 + 最低余额 1.11 元）')),
        );
        return;
      }
    } catch (e) {
      if (!mounted) return;
      setState(() => _submitting = false);
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('查询余额失败：$e')),
      );
      return;
    }

    try {
      final wallet = widget.wallet;
      final Future<Uint8List> Function(Uint8List payload) signCallback;

      if (wallet.isHotWallet) {
        // 热钱包：先验证设备密码/生物识别
        final walletManager = WalletManager();
        await walletManager.authenticateForSigning();
        signCallback = (payload) =>
            walletManager.signWithWalletNoAuth(wallet.walletIndex, payload);
      } else {
        // 冷钱包：使用签名协议 WUMIN_SIGN_V1.0.0
        signCallback = (Uint8List payload) async {
          final qrSigner = QrSigner();
          final requestId = QrSigner.generateRequestId(prefix: 'bind-');
          final rv = await ChainRpc().fetchRuntimeVersion();
          final request = qrSigner.buildRequest(
            requestId: requestId,
            account: wallet.address,
            pubkey: '0x${wallet.pubkeyHex}',
            payloadHex: '0x${_toHex(payload)}',
            specVersion: rv.specVersion,
            display: {
              'action': 'bind_clearing',
              'action_label': '绑定清算行',
              'summary': '绑定清算行：${bank.shenfenName}',
              'fields': [
                {
                  'key': 'institution',
                  'label': '清算省储行',
                  'value': bank.shenfenName,
                },
              ],
            },
          );
          final requestJson = qrSigner.encodeRequest(request);

          final response = await Navigator.push<QrSignResponse>(
            context,
            MaterialPageRoute(
              builder: (navContext) => QrSignSessionPage(
                request: request,
                requestJson: requestJson,
                expectedPubkey: '0x${wallet.pubkeyHex}',
              ),
            ),
          );

          if (response == null) {
            throw Exception('签名已取消');
          }
          return Uint8List.fromList(_hexToBytes(response.signature));
        };
      }

      final onchainRpc = OnchainRpc();
      final pubkeyBytes = _hexToBytes(wallet.pubkeyHex);
      await onchainRpc.bindClearingInstitution(
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(pubkeyBytes),
        shenfenId: bank.shenfenId,
        sign: signCallback,
      );

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('已提交绑定「${bank.shenfenName}」，等待链上确认')),
      );
      Navigator.pop(context, bank);
    } on WalletAuthException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(e.message)),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('绑定失败：$e')),
      );
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
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
