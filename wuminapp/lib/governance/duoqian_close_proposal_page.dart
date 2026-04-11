import 'dart:typed_data';

import 'package:flutter/material.dart';
import '../ui/app_theme.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../util/amount_format.dart';
import 'duoqian_manage_service.dart';
import 'institution_data.dart';
import '../qr/pages/qr_scan_page.dart' show QrScanPage, QrScanMode;
import '../qr/pages/qr_sign_session_page.dart';
import '../rpc/chain_rpc.dart';
import '../qr/bodies/sign_request_body.dart';
import '../signer/qr_signer.dart';
import '../wallet/core/wallet_manager.dart';

/// 关闭多签账户提案页面。
///
/// 指定受益人地址后发起关闭多签账户提案。
class DuoqianCloseProposalPage extends StatefulWidget {
  const DuoqianCloseProposalPage({
    super.key,
    required this.institution,
    required this.adminWallets,
  });

  final InstitutionInfo institution;
  final List<WalletProfile> adminWallets;

  @override
  State<DuoqianCloseProposalPage> createState() =>
      _DuoqianCloseProposalPageState();
}

class _DuoqianCloseProposalPageState extends State<DuoqianCloseProposalPage> {

  final _beneficiaryController = TextEditingController();
  final _manageService = DuoqianManageService();

  bool _submitting = false;
  bool _loadingBalance = true;
  double? _availableBalance;
  String? _addressError;

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
      final balance = await ChainRpc().fetchBalance(widget.institution.duoqianAddress);
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
      setState(() => _addressError = '受益人不能与多签地址相同');
      return false;
    }

    setState(() => _addressError = null);
    return true;
  }

  // ──── 提交 ────

  Future<void> _submit() async {
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
            action: 'propose_close',
            summary: '发起关闭多签账户提案',
            fields: [
              SignDisplayField(label: '多签地址', value: _duoqianSs58),
              SignDisplayField(label: '受益人', value: beneficiary),
              if (_availableBalance != null)
                SignDisplayField(label: '当前余额', value: AmountFormat.format(_availableBalance!, symbol: '')),
            ],
          ),
        );
        final requestJson = qrSigner.encodeRequest(request);
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

      final result = await _manageService.submitProposeClose(
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

  // ──── UI ────

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '关闭多签账户',
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
          // 多签地址（只读）
          _buildSectionTitle('多签地址'),
          const SizedBox(height: 8),
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: AppTheme.surfaceMuted,
              borderRadius: BorderRadius.circular(10),
            ),
            child: Text(
              _duoqianSs58,
              style: TextStyle(
                fontSize: 13,
                fontFamily: 'monospace',
                color: AppTheme.textSecondary,
              ),
            ),
          ),

          const SizedBox(height: 16),

          // 当前余额
          _buildSectionTitle('当前余额'),
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
                icon: const Icon(Icons.qr_code_scanner, color: AppTheme.primaryDark),
                onPressed: () async {
                  final result = await Navigator.push<String>(
                    context,
                    MaterialPageRoute(builder: (_) => const QrScanPage(mode: QrScanMode.transfer)),
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
              value: _selectedWallet,
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
              onPressed: _submitting ? null : _submit,
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
                  : const Text('发起关闭提案',
                      style:
                          TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
            ),
          ),
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
