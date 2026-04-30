import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:smoldot/smoldot.dart' show LightClientStatusSnapshot;
import 'package:wuminapp_mobile/citizen/institution/institution_data.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/qr/pages/qr_scan_page.dart'
    show QrScanMode, QrScanPage;
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/ui/widgets/chain_progress_banner.dart';
import 'package:wuminapp_mobile/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import '../shared/duoqian_manage_service.dart';

/// 关闭个人多签账户提案页面。
///
/// 指定受益人地址后发起个人多签账户关闭提案。
class PersonalDuoqianClosePage extends StatefulWidget {
  const PersonalDuoqianClosePage({
    super.key,
    required this.institution,
    required this.adminWallets,
  });

  final InstitutionInfo institution;
  final List<WalletProfile> adminWallets;

  @override
  State<PersonalDuoqianClosePage> createState() =>
      _PersonalDuoqianClosePageState();
}

class _PersonalDuoqianClosePageState extends State<PersonalDuoqianClosePage> {
  final _beneficiaryController = TextEditingController();
  final _manageService = DuoqianManageService();

  bool _submitting = false;
  bool _loadingBalance = true;
  double? _availableBalance;
  String? _addressError;
  LightClientStatusSnapshot? _chainProgress;
  String? _chainProgressError;

  late WalletProfile _selectedWallet;
  late String _duoqianSs58;

  @override
  void initState() {
    super.initState();
    _selectedWallet = widget.adminWallets.first;
    _duoqianSs58 = _hexToSs58(widget.institution.duoqianAddress);
    _fetchBalance();
  }

  @override
  void dispose() {
    _beneficiaryController.dispose();
    super.dispose();
  }

  Future<void> _fetchBalance() async {
    try {
      final balance =
          await ChainRpc().fetchBalance(widget.institution.duoqianAddress);
      if (!mounted) return;
      setState(() {
        _availableBalance = balance;
        _loadingBalance = false;
      });
    } catch (_) {
      if (!mounted) return;
      setState(() => _loadingBalance = false);
    }
  }

  // ──── 校验 ────

  bool _validateAddress(String address) {
    if (address.isEmpty) {
      setState(() => _addressError = '请输入受益人地址');
      return false;
    }
    try {
      Keyring().decodeAddress(address);
    } catch (_) {
      setState(() => _addressError = '地址格式无效');
      return false;
    }

    // 受益人不能是多签地址本身
    if (address == _duoqianSs58) {
      setState(() => _addressError = '受益人不能与个人多签地址相同');
      return false;
    }

    setState(() => _addressError = null);
    return true;
  }

  // ──── 提交 ────

  Future<void> _submit() async {
    final blockedReason = _submitBlockedReason;
    if (blockedReason != null) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(blockedReason)),
      );
      return;
    }

    final beneficiary = _beneficiaryController.text.trim();
    if (!_validateAddress(beneficiary)) return;

    if (_availableBalance != null && (_availableBalance! * 100).round() < 111) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('账户余额不足（最低 1.11 元）')),
      );
      return;
    }

    setState(() => _submitting = true);

    try {
      final wallet = _selectedWallet;
      final pubkeyBytes = _hexDecode(wallet.pubkeyHex);

      Future<Uint8List> signCallback(Uint8List payload) async {
        final qrSigner = QrSigner();
        final rv = await ChainRpc().fetchRuntimeVersion();
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'close-dq-'),
          address: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          specVersion: rv.specVersion,
          display: SignDisplay(
            action: 'propose_close_personal',
            summary: '发起关闭个人多签账户提案',
            fields: [
              // propose_close_personal 当前链端仍复用 propose_close 的 Registry =
              // (duoqian_address, beneficiary)。"当前余额" 属辅助展示,
              // 页面已独立显示,不塞 display.fields 避免对齐失败
              // (2026-04-22 两色识别整改)。
              SignDisplayField(
                  key: 'duoqian_address', label: '个人多签地址', value: _duoqianSs58),
              SignDisplayField(
                  key: 'beneficiary', label: '受益人', value: beneficiary),
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
                expectedPubkey: '0x${wallet.pubkeyHex}'),
          ),
        );
        if (response == null) throw Exception('签名已取消');
        return Uint8List.fromList(_hexDecode(response.body.signature));
      }

      final result = await _manageService.submitProposeClosePersonal(
        duoqianAddress: widget.institution.duoqianAddress,
        beneficiaryAddress: beneficiary,
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(pubkeyBytes),
        sign: signCallback,
      );

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('提案已提交：${_truncateAddress(result.txHash)}'),
          backgroundColor: AppTheme.primaryDark,
        ),
      );
      Navigator.of(context).pop(true);
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('提交失败：$e'), backgroundColor: AppTheme.danger),
      );
    } finally {
      if (mounted) setState(() => _submitting = false);
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

  /// 中文注释：关闭个人多签会直接动到账户资金，链不同步时不允许继续发起。
  String? get _submitBlockedReason {
    final progress = _chainProgress;
    if (progress == null) {
      return _chainProgressError ?? '正在读取区块链状态，请稍后再试';
    }
    if (!progress.hasPeers) {
      return '轻节点尚未连接到区块链网络，暂不能发起关闭个人多签提案';
    }
    if (progress.isSyncing) {
      return '轻节点仍在同步区块头，完成后才能发起关闭个人多签提案';
    }
    if (!progress.isUsable) {
      return _chainProgressError ?? '区块链状态尚未就绪，暂不能发起关闭个人多签提案';
    }
    return null;
  }

  // ──── UI ────

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '关闭个人多签',
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
          // 个人多签地址（只读）
          _buildSectionTitle('个人多签地址'),
          const SizedBox(height: 8),
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: AppTheme.surfaceMuted,
              borderRadius: BorderRadius.circular(10),
            ),
            child: Text(
              _duoqianSs58,
              style: const TextStyle(
                fontSize: 13,
                fontFamily: 'monospace',
                color: AppTheme.textSecondary,
              ),
            ),
          ),

          const SizedBox(height: 16),

          // 当前余额
          _buildSectionTitle('个人多签余额'),
          const SizedBox(height: 8),
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: AppTheme.surfaceMuted,
              borderRadius: BorderRadius.circular(10),
            ),
            child: _loadingBalance
                ? const SizedBox(
                    height: 16,
                    width: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : Text(
                    _availableBalance != null
                        ? '${AmountFormat.format(_availableBalance!, symbol: '')} 元'
                        : '查询失败',
                    style: TextStyle(
                      fontSize: 15,
                      fontWeight: FontWeight.w600,
                      color: _availableBalance != null
                          ? AppTheme.primaryDark
                          : AppTheme.danger,
                    ),
                  ),
          ),

          const SizedBox(height: 20),

          // 受益人地址
          _buildSectionTitle('受益人地址'),
          const SizedBox(height: 8),
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _beneficiaryController,
                  decoration: InputDecoration(
                    hintText: '输入或扫码',
                    errorText: _addressError,
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                    contentPadding: const EdgeInsets.symmetric(
                        horizontal: 12, vertical: 10),
                  ),
                ),
              ),
              const SizedBox(width: 8),
              IconButton(
                icon: const Icon(Icons.qr_code_scanner,
                    color: AppTheme.primaryDark),
                onPressed: () async {
                  final result = await Navigator.push<String>(
                    context,
                    MaterialPageRoute(
                        builder: (_) =>
                            const QrScanPage(mode: QrScanMode.transfer)),
                  );
                  if (result != null && mounted) {
                    _beneficiaryController.text = result.trim();
                  }
                },
              ),
            ],
          ),

          if (widget.adminWallets.length > 1) ...[
            const SizedBox(height: 20),
            _buildSectionTitle('签名钱包'),
            const SizedBox(height: 8),
            DropdownButtonFormField<WalletProfile>(
              initialValue: _selectedWallet,
              items: widget.adminWallets.map((w) {
                return DropdownMenuItem(
                  value: w,
                  child: Text(
                    '${w.walletName} (${_truncateAddress(w.address)})',
                    style: const TextStyle(fontSize: 13),
                  ),
                );
              }).toList(),
              onChanged: (w) {
                if (w != null) setState(() => _selectedWallet = w);
              },
              decoration: InputDecoration(
                border:
                    OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
                contentPadding:
                    const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
              ),
            ),
          ],

          const SizedBox(height: 28),

          // 提交按钮
          SizedBox(
            width: double.infinity,
            child: ElevatedButton(
              onPressed: _canSubmit ? _submit : null,
              style: ElevatedButton.styleFrom(
                backgroundColor: AppTheme.danger,
                foregroundColor: Colors.white,
                padding: const EdgeInsets.symmetric(vertical: 14),
                shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(12)),
              ),
              child: _submitting
                  ? const SizedBox(
                      width: 18,
                      height: 18,
                      child: CircularProgressIndicator(
                          strokeWidth: 2, color: Colors.white),
                    )
                  : const Text('发起关闭个人多签提案',
                      style:
                          TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
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

  Widget _buildSectionTitle(String title) {
    return Text(
      title,
      style: const TextStyle(
        fontSize: 14,
        fontWeight: FontWeight.w600,
        color: AppTheme.primaryDark,
      ),
    );
  }

  // ──── 工具 ────

  String _truncateAddress(String address) {
    if (address.length <= 14) return address;
    return '${address.substring(0, 6)}...${address.substring(address.length - 6)}';
  }

  String _hexToSs58(String hex) {
    final bytes = _hexDecode(hex);
    return Keyring().encodeAddress(Uint8List.fromList(bytes), 2027);
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

  Uint8List _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = Uint8List(h.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }
}
