import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:smoldot/smoldot.dart' show LightClientStatusSnapshot;

import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/qr/pages/qr_sign_session_page.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/onchain.dart' show OnchainRpc;
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/transaction/multisig-transfer/multisig_transfer_balance_guard.dart';
import 'package:citizenapp/transaction/multisig-transfer/multisig_transfer_service.dart';
import 'package:citizenapp/transaction/shared/account_balance_snapshot_store.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/widgets/chain_progress_banner.dart';
import 'package:citizenapp/my/util/amount_format.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 治理机构手续费划转提案创建页面。
///
/// source 锁定为机构 `feeAccount`,destination 固定为机构 `mainAccount`,
/// 链端调用 `propose_sweep_to_main (call_index=2)`,无 beneficiary/remark 入参。
class SweepToMainPage extends StatefulWidget {
  const SweepToMainPage({
    super.key,
    required this.institution,
    required this.icon,
    required this.badgeColor,
    required this.adminWallets,
  });

  final InstitutionInfo institution;
  final IconData icon;
  final Color badgeColor;

  final List<WalletProfile> adminWallets;

  @override
  State<SweepToMainPage> createState() => _SweepToMainPageState();
}

class _SweepToMainPageState extends State<SweepToMainPage> {
  final _amountController = TextEditingController();

  bool _loadingBalance = true;
  bool _submitting = false;
  double? _availableBalance;
  double _estimatedFee = 0.0;
  String? _amountError;
  LightClientStatusSnapshot? _chainProgress;
  String? _chainProgressError;

  late final String _feeAccountHex;
  late final String _mainAccountHex;
  late final String _fromSs58;
  late final String _toSs58;
  late WalletProfile _selectedWallet;

  @override
  void initState() {
    super.initState();
    _selectedWallet = widget.adminWallets.first;
    final feeHex = widget.institution.accounts?.feeAccount;
    if (feeHex == null) {
      throw StateError('治理机构 InstitutionAccounts.feeAccount 为空,无法发起手续费划转');
    }
    _feeAccountHex = feeHex;
    _mainAccountHex = widget.institution.mainAccount;
    _fromSs58 = _accountHexToSs58(_feeAccountHex);
    _toSs58 = _accountHexToSs58(_mainAccountHex);
    _fetchBalance();
    _amountController.addListener(_onAmountChanged);
  }

  @override
  void dispose() {
    _amountController.dispose();
    super.dispose();
  }

  String _accountHexToSs58(String hex) {
    final bytes = _hexToBytes(hex);
    return Keyring().encodeAddress(Uint8List.fromList(bytes), 2027);
  }

  Future<void> _fetchBalance() async {
    final store = AccountBalanceSnapshotStore.instance;
    final local = await store.read(_feeAccountHex);
    if (local != null && mounted) {
      setState(() {
        _availableBalance = local.balanceYuan;
        _loadingBalance = false;
      });
      if (local.isFresh(AccountBalanceSnapshotStore.displayTtl)) return;
    }
    try {
      final balance = await ChainRpc().fetchFinalizedBalance(_feeAccountHex);
      try {
        await store.put(
          accountHex: _feeAccountHex,
          balanceYuan: balance,
        );
      } catch (_) {
        // 余额快照写入失败不影响当前链上余额展示。
      }
      if (!mounted) return;
      setState(() {
        _availableBalance = balance;
        _loadingBalance = false;
      });
    } catch (_) {
      if (!mounted) return;
      if (local == null) {
        setState(() {
          _availableBalance = null;
          _loadingBalance = false;
        });
      }
    }
  }

  void _onAmountChanged() {
    final amount = AmountFormat.tryParse(_amountController.text);
    setState(() {
      if (amount != null && amount > 0) {
        _estimatedFee = OnchainRpc.estimateTransferFeeYuan(amount);
      } else {
        _estimatedFee = 0.0;
      }
    });
  }

  bool _validateAmount() {
    final amount = AmountFormat.tryParse(_amountController.text);
    if (amount == null || amount < 1.11) {
      setState(() => _amountError = '最低划转金额为 1.11 元（存在性保证金）');
      return false;
    }
    if (_availableBalance != null) {
      final fee = OnchainRpc.estimateTransferFeeYuan(amount);
      const ed = 1.11;
      if (amount + fee + ed > _availableBalance!) {
        setState(() => _amountError =
            '余额不足（需保留 ${AmountFormat.format(ed, symbol: '')} 元 ED + ${AmountFormat.format(fee, symbol: '')} 元手续费）');
        return false;
      }
    }
    setState(() => _amountError = null);
    return true;
  }

  Future<void> _submit() async {
    final blockedReason = _submitBlockedReason;
    if (blockedReason != null) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(blockedReason)),
      );
      return;
    }

    if (!_validateAmount()) return;

    final wallet = _selectedWallet;
    final amountYuan = AmountFormat.tryParse(_amountController.text) ?? 0;
    final requiredAdminFee = OnchainRpc.estimateTransferFeeYuan(amountYuan);
    final balanceBlockedReason =
        await MultisigTransferBalanceGuard.checkAdminWalletBalance(
      wallet: wallet,
      requiredFeeYuan: requiredAdminFee,
      actionLabel: '发起手续费划转提案',
    );
    if (balanceBlockedReason != null) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(balanceBlockedReason)),
      );
      return;
    }

    setState(() => _submitting = true);

    try {
      WalletManager? hotWalletManager;
      if (wallet.isHotWallet) {
        hotWalletManager = WalletManager();
        await hotWalletManager.authenticateForSigning();
      }

      Future<Uint8List> signCallback(Uint8List payload) async {
        if (hotWalletManager != null) {
          return await hotWalletManager.signWithWalletNoAuth(
              wallet.walletIndex, payload);
        }
        final qrSigner = QrSigner();
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'propose-sweep-'),
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          action: QrActions.sweepToMain,
        );
        final requestJson = qrSigner.encodeRequest(request);
        if (!mounted) throw Exception('页面已关闭');
        final response = await Navigator.push<SignResponseEnvelope>(
          context,
          MaterialPageRoute(
            builder: (_) => QrSignSessionPage(
                request: request,
                requestJson: requestJson,
                expectedPubkey: '0x${wallet.pubkeyHex}'),
          ),
        );
        if (response == null) throw Exception('签名已取消');
        return Uint8List.fromList(_hexToBytes(response.body.signatureHex));
      }

      final signerPubkey = Uint8List.fromList(_hexToBytes(wallet.pubkeyHex));

      final service = MultisigTransferService();
      // 提案类交易等真正入块并核对事件后才返回，proposalId 来自
      // 链上 SweepToMainProposed 事件，是业务成功的唯一凭据。
      final result = await service.submitProposeSweep(
        institution: widget.institution,
        amountYuan: amountYuan,
        fromAddress: wallet.address,
        signerPubkey: signerPubkey,
        sign: signCallback,
      );

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('提案已创建（#${result.proposalId}），等待管理员投票')),
      );
      Navigator.of(context).pop(true);
    } on WalletAuthException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(e.message)),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('提交失败：$e')),
      );
    } finally {
      if (mounted) {
        setState(() => _submitting = false);
      }
    }
  }

  void _handleChainProgressChanged(LightClientStatusSnapshot? progress) {
    if (!mounted) return;
    setState(() {
      _chainProgress = progress;
    });
  }

  void _handleChainProgressErrorChanged(String? error) {
    if (!mounted) return;
    setState(() {
      _chainProgressError = error;
    });
  }

  bool get _canSubmit => !_submitting && _submitBlockedReason == null;

  String? get _submitBlockedReason {
    final progress = _chainProgress;
    if (progress == null) {
      return _chainProgressError ?? '正在读取区块链状态，请稍后再试';
    }
    if (!progress.hasPeers) {
      return '轻节点尚未连接到区块链网络，暂不能提交手续费划转提案';
    }
    if (progress.isSyncing) {
      return '轻节点仍在同步区块头，完成后才能提交手续费划转提案';
    }
    if (!progress.isUsable) {
      return _chainProgressError ?? '区块链状态尚未就绪，暂不能提交手续费划转提案';
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '发起手续费划转提案',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: AppTheme.primaryDark,
        elevation: 0,
        scrolledUnderElevation: 0.5,
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          ChainProgressBanner(
            busy: _submitting || _loadingBalance,
            onProgressChanged: _handleChainProgressChanged,
            onErrorChanged: _handleChainProgressErrorChanged,
          ),
          _buildInstitutionHeader(),
          const SizedBox(height: 16),
          _buildLabel('发起管理员'),
          const SizedBox(height: 6),
          _buildAdminSelector(),
          const SizedBox(height: 16),
          _buildLabel('转出账户（费用账户）'),
          const SizedBox(height: 6),
          _buildReadOnlyField(_fromSs58),
          const SizedBox(height: 16),
          _buildLabel('划入账户（本机构主账户）'),
          const SizedBox(height: 6),
          _buildReadOnlyField(_toSs58),
          const SizedBox(height: 16),
          _buildLabel('划转金额（元）'),
          const SizedBox(height: 6),
          TextField(
            controller: _amountController,
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            inputFormatters: [ThousandSeparatorFormatter()],
            decoration: InputDecoration(
              hintText: '最低 1.11 元',
              hintStyle:
                  const TextStyle(color: AppTheme.textTertiary, fontSize: 14),
              filled: true,
              fillColor: AppTheme.surfaceMuted,
              enabledBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.border),
              ),
              focusedBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.primaryDark),
              ),
              errorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.danger),
              ),
              focusedErrorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.danger),
              ),
              errorText: _amountError,
              suffixText: '元',
            ),
            style: const TextStyle(fontSize: 14),
          ),
          const SizedBox(height: 12),
          _buildInfoRow(
            '预估手续费',
            _estimatedFee > 0
                ? '${AmountFormat.format(_estimatedFee, symbol: '')} 元'
                : '--',
          ),
          const SizedBox(height: 8),
          _buildInfoRow(
            '费用账户可用余额',
            _loadingBalance
                ? '查询中...'
                : _availableBalance != null
                    ? '${AmountFormat.format(_availableBalance!, symbol: '')} 元'
                    : '查询失败',
          ),
          const SizedBox(height: 24),
          SizedBox(
            width: double.infinity,
            height: 48,
            child: FilledButton(
              style: FilledButton.styleFrom(
                backgroundColor: AppTheme.primaryDark,
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(10),
                ),
              ),
              onPressed: _canSubmit ? _submit : null,
              child: _submitting
                  ? const SizedBox(
                      width: 20,
                      height: 20,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        color: Colors.white,
                      ),
                    )
                  : const Text(
                      '提交手续费划转提案',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: Colors.white,
                      ),
                    ),
            ),
          ),
          if (_submitBlockedReason != null) ...[
            const SizedBox(height: 10),
            Text(
              _submitBlockedReason!,
              style: const TextStyle(
                fontSize: 12,
                height: 1.4,
                color: AppTheme.textSecondary,
              ),
            ),
          ],
        ],
      ),
    );
  }

  String _truncateAddress(String address) {
    if (address.length <= 16) return address;
    return '${address.substring(0, 8)}...${address.substring(address.length - 8)}';
  }

  Widget _buildAdminSelector() {
    final wallets = widget.adminWallets;
    if (wallets.length == 1) {
      return Container(
        width: double.infinity,
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
        decoration: BoxDecoration(
          color: AppTheme.success.withValues(alpha: 0.06),
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: AppTheme.success.withValues(alpha: 0.2)),
        ),
        child: Row(
          children: [
            const Icon(Icons.verified_user, size: 16, color: AppTheme.success),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                _truncateAddress(wallets.first.address),
                style: const TextStyle(
                  fontSize: 13,
                  fontFamily: 'monospace',
                  color: AppTheme.success,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ),
          ],
        ),
      );
    }
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: AppTheme.success.withValues(alpha: 0.3)),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<int>(
          value: _selectedWallet.walletIndex,
          isExpanded: true,
          icon: const Icon(Icons.arrow_drop_down, color: AppTheme.primaryDark),
          items: wallets.map((w) {
            return DropdownMenuItem<int>(
              value: w.walletIndex,
              child: Row(
                children: [
                  const Icon(Icons.verified_user,
                      size: 14, color: AppTheme.success),
                  const SizedBox(width: 6),
                  Expanded(
                    child: Text(
                      _truncateAddress(w.address),
                      style: const TextStyle(
                        fontSize: 13,
                        fontFamily: 'monospace',
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ],
              ),
            );
          }).toList(),
          onChanged: (index) {
            if (index == null) return;
            setState(() {
              _selectedWallet =
                  wallets.firstWhere((w) => w.walletIndex == index);
            });
          },
        ),
      ),
    );
  }

  Widget _buildInstitutionHeader() {
    return Row(
      children: [
        Container(
          width: 36,
          height: 36,
          decoration: BoxDecoration(
            color: widget.badgeColor.withValues(alpha: 0.12),
            borderRadius: BorderRadius.circular(10),
          ),
          child: Icon(widget.icon, size: 18, color: widget.badgeColor),
        ),
        const SizedBox(width: 10),
        Expanded(
          child: Text(
            '${widget.institution.cidShortName}（手续费划转）',
            style: const TextStyle(
              fontSize: 15,
              fontWeight: FontWeight.w600,
              color: AppTheme.primaryDark,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildLabel(String text) {
    return Text(
      text,
      style: const TextStyle(
        fontSize: 13,
        fontWeight: FontWeight.w600,
        color: AppTheme.primaryDark,
      ),
    );
  }

  Widget _buildReadOnlyField(String value) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 14),
      decoration: BoxDecoration(
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: AppTheme.border),
      ),
      child: SelectableText(
        value,
        style: const TextStyle(
          fontSize: 13,
          color: AppTheme.textSecondary,
          fontFamily: 'monospace',
        ),
      ),
    );
  }

  Widget _buildInfoRow(String label, String value) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(
          label,
          style: const TextStyle(fontSize: 13, color: AppTheme.textSecondary),
        ),
        Text(
          value,
          style: const TextStyle(
            fontSize: 13,
            fontWeight: FontWeight.w600,
            color: AppTheme.primaryDark,
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
