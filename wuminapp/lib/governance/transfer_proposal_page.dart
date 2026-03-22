import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'institution_data.dart';
import 'transfer_proposal_service.dart';
import '../qr/pages/qr_scan_page.dart';
import '../qr/pages/qr_sign_session_page.dart';
import '../rpc/onchain.dart' show OnchainRpc;
import '../signer/qr_signer.dart';
import '../wallet/core/wallet_manager.dart';

/// 机构转账提案创建页面。
class TransferProposalPage extends StatefulWidget {
  const TransferProposalPage({
    super.key,
    required this.institution,
    required this.icon,
    required this.badgeColor,
    required this.adminWallets,
  });

  final InstitutionInfo institution;
  final IconData icon;
  final Color badgeColor;

  /// 当前用户导入的、属于此机构的管理员钱包列表。
  final List<WalletProfile> adminWallets;

  @override
  State<TransferProposalPage> createState() => _TransferProposalPageState();
}

class _TransferProposalPageState extends State<TransferProposalPage> {
  static const Color _inkGreen = Color(0xFF0B3D2E);
  static const Color _inputFieldColor = Color(0xFFF7F7F7);
  static const Color _inputBorderColor = Color(0xFFD0D0D0);

  final _beneficiaryController = TextEditingController();
  final _amountController = TextEditingController();
  final _remarkController = TextEditingController();

  bool _loadingBalance = true;
  bool _submitting = false;
  double? _availableBalance;
  double _estimatedFee = 0.0;
  String? _addressError;
  String? _amountError;

  late final String _fromSs58;
  late WalletProfile _selectedWallet;

  @override
  void initState() {
    super.initState();
    _selectedWallet = widget.adminWallets.first;
    _fromSs58 = _duoqianHexToSs58(widget.institution.duoqianAddress);
    _fetchBalance();
    _amountController.addListener(_onAmountChanged);
  }

  @override
  void dispose() {
    _beneficiaryController.dispose();
    _amountController.dispose();
    _remarkController.dispose();
    super.dispose();
  }

  String _duoqianHexToSs58(String hex) {
    final bytes = _hexToBytes(hex);
    return Keyring().encodeAddress(Uint8List.fromList(bytes), 2027);
  }

  Future<void> _fetchBalance() async {
    try {
      final service = TransferProposalService();
      final balance = await service.fetchInstitutionBalance(widget.institution);
      if (!mounted) return;
      setState(() {
        _availableBalance = balance;
        _loadingBalance = false;
      });
    } catch (_) {
      if (!mounted) return;
      setState(() {
        _availableBalance = null;
        _loadingBalance = false;
      });
    }
  }

  void _onAmountChanged() {
    final text = _amountController.text.trim();
    final amount = double.tryParse(text);
    setState(() {
      if (amount != null && amount > 0) {
        _estimatedFee = OnchainRpc.estimateTransferFeeYuan(amount);
      } else {
        _estimatedFee = 0.0;
      }
    });
  }

  Future<void> _scanToAddress() async {
    final result = await Navigator.of(context).push<QrScanTransferResult>(
      MaterialPageRoute(
        builder: (_) => const QrScanPage(mode: QrScanMode.transfer),
      ),
    );
    if (result == null || !mounted) return;
    setState(() {
      _beneficiaryController.text = result.toAddress;
    });
  }

  bool _validateAddress() {
    final address = _beneficiaryController.text.trim();
    if (address.isEmpty) {
      setState(() => _addressError = '请输入收款地址');
      return false;
    }
    try {
      Keyring().decodeAddress(address);
    } catch (_) {
      setState(() => _addressError = '地址格式无效');
      return false;
    }
    // 检查是否与机构地址相同
    final beneficiaryBytes = Keyring().decodeAddress(address);
    final institutionBytes =
        Uint8List.fromList(_hexToBytes(widget.institution.duoqianAddress));
    if (_bytesEqual(beneficiaryBytes, institutionBytes)) {
      setState(() => _addressError = '收款地址不能与机构地址相同');
      return false;
    }
    setState(() => _addressError = null);
    return true;
  }

  bool _validateAmount() {
    final text = _amountController.text.trim();
    final amount = double.tryParse(text);
    if (amount == null || amount < 1.11) {
      setState(() => _amountError = '最低转账金额为 1.11 元（存在性保证金）');
      return false;
    }
    if (_availableBalance != null) {
      final fee = OnchainRpc.estimateTransferFeeYuan(amount);
      const ed = 1.11;
      if (amount + fee + ed > _availableBalance!) {
        setState(() => _amountError =
            '余额不足（需保留 ${ed.toStringAsFixed(2)} 元 ED + ${fee.toStringAsFixed(2)} 元手续费）');
        return false;
      }
    }
    setState(() => _amountError = null);
    return true;
  }

  bool _validateRemark() {
    final remark = _remarkController.text;
    final bytes = utf8.encode(remark);
    if (bytes.length > 256) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('备注超过 256 字节限制（当前 ${bytes.length} 字节）')),
      );
      return false;
    }
    return true;
  }

  Future<void> _submit() async {
    if (!_validateAddress() || !_validateAmount() || !_validateRemark()) {
      return;
    }

    final wallet = _selectedWallet;

    setState(() => _submitting = true);

    try {
      Future<Uint8List> signCallback(Uint8List payload) async {
        // 管理员操作统一通过 QR 码签名（wumin 冷钱包）
        final qrSigner = QrSigner();
        final beneficiary = _beneficiaryController.text.trim();
        final amountText = _amountController.text.trim();
        // 统一格式化为两位小数，与 PayloadDecoder._fenToYuan 对齐
        final amountFormatted =
            (double.tryParse(amountText) ?? 0).toStringAsFixed(2);
        final remarkText = _remarkController.text;
        final request = qrSigner.buildRequest(
          requestId: 'propose-${DateTime.now().millisecondsSinceEpoch}',
          account: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          display: {
            'action': 'propose_transfer',
            'summary': '提案转账 $amountFormatted GMB 给 $beneficiary',
            'fields': {
              'beneficiary': beneficiary,
              'amount_yuan': amountFormatted,
              'remark': remarkText,
            },
          },
        );
        final requestJson = qrSigner.encodeRequest(request);
        final response = await Navigator.push<QrSignResponse>(
          context,
          MaterialPageRoute(
            builder: (_) => QrSignSessionPage(
                request: request,
                requestJson: requestJson,
                expectedPubkey: '0x${wallet.pubkeyHex}'),
          ),
        );
        if (response == null) throw Exception('签名已取消');
        return Uint8List.fromList(_hexToBytes(response.signature));
      }

      final signerPubkey = Uint8List.fromList(_hexToBytes(wallet.pubkeyHex));

      final service = TransferProposalService();
      await service.submitProposeTransfer(
        institution: widget.institution,
        beneficiaryAddress: _beneficiaryController.text.trim(),
        amountYuan: double.parse(_amountController.text.trim()),
        remark: _remarkController.text,
        fromAddress: wallet.address,
        signerPubkey: signerPubkey,
        sign: signCallback,
      );

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('提交成功')),
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

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '发起转账提案',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: _inkGreen,
        elevation: 0,
        scrolledUnderElevation: 0.5,
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          // ──── 机构信息 ────
          _buildInstitutionHeader(),
          const SizedBox(height: 16),

          // ──── 发起管理员 ────
          _buildLabel('发起管理员'),
          const SizedBox(height: 6),
          _buildAdminSelector(),
          const SizedBox(height: 16),

          // ──── 转出地址（只读） ────
          _buildLabel('转出地址'),
          const SizedBox(height: 6),
          _buildReadOnlyField(_fromSs58),
          const SizedBox(height: 16),

          // ──── 收款地址 ────
          _buildLabel('收款地址'),
          const SizedBox(height: 6),
          TextField(
            controller: _beneficiaryController,
            decoration: InputDecoration(
              hintText: '输入 SS58 格式地址',
              hintStyle: TextStyle(color: Colors.grey[400], fontSize: 14),
              filled: true,
              fillColor: _inputFieldColor,
              enabledBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: _inputBorderColor),
              ),
              focusedBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: _inkGreen),
              ),
              errorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: Colors.red),
              ),
              focusedErrorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: Colors.red),
              ),
              errorText: _addressError,
              suffixIcon: IconButton(
                tooltip: '扫码填入收款地址',
                onPressed: _scanToAddress,
                icon: SvgPicture.asset(
                  'assets/icons/scan-line.svg',
                  width: 18,
                  height: 18,
                ),
              ),
            ),
            style: const TextStyle(fontSize: 14),
          ),
          const SizedBox(height: 16),

          // ──── 转账金额 ────
          _buildLabel('转账金额（元）'),
          const SizedBox(height: 6),
          TextField(
            controller: _amountController,
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            inputFormatters: [
              FilteringTextInputFormatter.allow(RegExp(r'^\d*\.?\d{0,2}')),
            ],
            decoration: InputDecoration(
              hintText: '最低 1.11 元',
              hintStyle: TextStyle(color: Colors.grey[400], fontSize: 14),
              filled: true,
              fillColor: _inputFieldColor,
              enabledBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: _inputBorderColor),
              ),
              focusedBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: _inkGreen),
              ),
              errorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: Colors.red),
              ),
              focusedErrorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: Colors.red),
              ),
              errorText: _amountError,
              suffixText: '元',
            ),
            style: const TextStyle(fontSize: 14),
          ),
          const SizedBox(height: 12),

          // ──── 预估手续费 ────
          _buildInfoRow(
            '预估手续费',
            _estimatedFee > 0 ? '${_estimatedFee.toStringAsFixed(2)} 元' : '--',
          ),
          const SizedBox(height: 8),

          // ──── 可用余额 ────
          _buildInfoRow(
            '可用余额',
            _loadingBalance
                ? '查询中...'
                : _availableBalance != null
                    ? '${_availableBalance!.toStringAsFixed(2)} 元'
                    : '查询失败',
          ),
          const SizedBox(height: 16),

          // ──── 备注 ────
          _buildLabel('备注（可选）'),
          const SizedBox(height: 6),
          TextField(
            controller: _remarkController,
            maxLines: 3,
            decoration: InputDecoration(
              hintText: '最多 256 字节',
              hintStyle: TextStyle(color: Colors.grey[400], fontSize: 14),
              filled: true,
              fillColor: _inputFieldColor,
              enabledBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: _inputBorderColor),
              ),
              focusedBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: _inkGreen),
              ),
            ),
            style: const TextStyle(fontSize: 14),
          ),
          const SizedBox(height: 24),

          // ──── 提交按钮 ────
          SizedBox(
            width: double.infinity,
            height: 48,
            child: FilledButton(
              style: FilledButton.styleFrom(
                backgroundColor: _inkGreen,
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(10),
                ),
              ),
              onPressed: _submitting ? null : _submit,
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
                      '提交转账提案',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: Colors.white,
                      ),
                    ),
            ),
          ),
        ],
      ),
    );
  }

  // ──── 子组件 ────

  String _truncateAddress(String address) {
    if (address.length <= 16) return address;
    return '${address.substring(0, 8)}...${address.substring(address.length - 8)}';
  }

  Widget _buildAdminSelector() {
    final wallets = widget.adminWallets;
    if (wallets.length == 1) {
      // 只有一个管理员钱包，直接展示
      return Container(
        width: double.infinity,
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
        decoration: BoxDecoration(
          color: Colors.green.withValues(alpha: 0.06),
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: Colors.green.withValues(alpha: 0.2)),
        ),
        child: Row(
          children: [
            const Icon(Icons.verified_user, size: 16, color: Colors.green),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                _truncateAddress(wallets.first.address),
                style: const TextStyle(
                  fontSize: 13,
                  fontFamily: 'monospace',
                  color: Colors.green,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ),
          ],
        ),
      );
    }
    // 多个管理员钱包，下拉选择
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        color: _inputFieldColor,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.green.withValues(alpha: 0.3)),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<int>(
          value: _selectedWallet.walletIndex,
          isExpanded: true,
          icon: const Icon(Icons.arrow_drop_down, color: _inkGreen),
          items: wallets.map((w) {
            return DropdownMenuItem<int>(
              value: w.walletIndex,
              child: Row(
                children: [
                  const Icon(Icons.verified_user,
                      size: 14, color: Colors.green),
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
            widget.institution.name,
            style: const TextStyle(
              fontSize: 15,
              fontWeight: FontWeight.w600,
              color: _inkGreen,
            ),
          ),
        ),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
          decoration: BoxDecoration(
            color: widget.badgeColor.withValues(alpha: 0.10),
            borderRadius: BorderRadius.circular(10),
          ),
          child: Text(
            OrgType.label(widget.institution.orgType),
            style: TextStyle(
              fontSize: 11,
              color: widget.badgeColor,
              fontWeight: FontWeight.w600,
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
        color: _inkGreen,
      ),
    );
  }

  Widget _buildReadOnlyField(String value) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 14),
      decoration: BoxDecoration(
        color: _inputFieldColor,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: _inputBorderColor),
      ),
      child: SelectableText(
        value,
        style: TextStyle(
          fontSize: 13,
          color: Colors.grey[600],
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
          style: TextStyle(fontSize: 13, color: Colors.grey[600]),
        ),
        Text(
          value,
          style: const TextStyle(
            fontSize: 13,
            fontWeight: FontWeight.w600,
            color: _inkGreen,
          ),
        ),
      ],
    );
  }
}

// ──── 工具函数 ────

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

bool _bytesEqual(List<int> a, List<int> b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}
