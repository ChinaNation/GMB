import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:citizenapp/governance/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/governance/admins-change/models/admin_account.dart';
import 'package:citizenapp/governance/admins-change/pages/admin_set_change_confirm_page.dart';
import 'package:citizenapp/governance/admins-change/services/admin_set_change_service.dart';
import 'package:citizenapp/governance/admins-change/services/admin_set_validation.dart';
import 'package:citizenapp/governance/admins-change/services/admin_account_service.dart';
import 'package:citizenapp/governance/admins-change/widgets/admin_set_change_action_bar.dart';
import 'package:citizenapp/governance/admins-change/widgets/admin_set_diff_card.dart';
import 'package:citizenapp/governance/admins-change/widgets/admin_set_editor.dart';
import 'package:citizenapp/governance/admins-change/widgets/admin_account_card.dart';
import 'package:citizenapp/governance/shared/institution_info.dart';
import 'package:citizenapp/qr/pages/qr_sign_session_page.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

class AdminSetChangePage extends StatefulWidget {
  const AdminSetChangePage({
    super.key,
    required this.institution,
    required this.accountIdentity,
    required this.adminWallets,
  });

  final InstitutionInfo institution;
  final AdminAccountIdentity accountIdentity;
  final List<WalletProfile> adminWallets;

  @override
  State<AdminSetChangePage> createState() => _AdminSetChangePageState();
}

class _AdminSetChangePageState extends State<AdminSetChangePage> {
  final _accountService = AdminAccountService();
  final _changeService = AdminSetChangeService();
  final _thresholdController = TextEditingController();
  AdminAccountState? _subject;
  List<String> _admins = const [];
  WalletProfile? _selectedWallet;
  bool _loading = true;
  bool _submitting = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _selectedWallet =
        widget.adminWallets.isNotEmpty ? widget.adminWallets.first : null;
    _load();
  }

  @override
  void dispose() {
    _thresholdController.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final account = await _accountService.fetchByIdentity(
        widget.accountIdentity,
      );
      if (!mounted) return;
      setState(() {
        _subject = account;
        _admins = account?.admins ?? const [];
        if (account != null) _syncThresholdInput(account, _admins.length);
        _loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = '$e';
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final account = _subject;
    return Scaffold(
      appBar: AppBar(title: const Text('更换管理员')),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : account == null
              ? Center(child: Text(_error ?? '未查询到管理员账户'))
              : ListView(
                  padding: const EdgeInsets.all(16),
                  children: [
                    AdminAccountCard(account: account),
                    const SizedBox(height: 12),
                    _buildWalletSelector(),
                    const SizedBox(height: 12),
                    AdminSetEditor(
                        admins: _admins,
                        onChanged: (value) => _setNewAdmins(account, value)),
                    const SizedBox(height: 12),
                    _buildThresholdCard(account),
                    const SizedBox(height: 12),
                    AdminSetDiffCard(
                        currentAdmins: account.admins, admins: _admins),
                    if (_error != null) ...[
                      const SizedBox(height: 12),
                      Text(_error!, style: const TextStyle(color: Colors.red)),
                    ],
                  ],
                ),
      bottomNavigationBar: account == null
          ? null
          : AdminSetChangeActionBar(
              busy: _submitting,
              enabled: _selectedWallet != null,
              onSubmit: _submit,
            ),
    );
  }

  Widget _buildWalletSelector() {
    return DropdownButtonFormField<WalletProfile>(
      initialValue: _selectedWallet,
      decoration: const InputDecoration(labelText: '发起管理员钱包'),
      items: widget.adminWallets
          .map((wallet) =>
              DropdownMenuItem(value: wallet, child: Text(wallet.walletName)))
          .toList(),
      onChanged: _submitting
          ? null
          : (wallet) => setState(() => _selectedWallet = wallet),
    );
  }

  void _setNewAdmins(AdminAccountState account, List<String> value) {
    setState(() {
      _admins = value;
      _syncThresholdInput(account, value.length);
    });
  }

  Widget _buildThresholdCard(AdminAccountState account) {
    final fixed =
        AdminSetValidation.fixedGovernanceThreshold(account.institutionCode);
    if (account.kind == 0 && fixed != null) {
      return Card(
        elevation: 0,
        margin: EdgeInsets.zero,
        child: Padding(
          padding: const EdgeInsets.all(14),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text('阈值规则',
                  style: TextStyle(fontSize: 15, fontWeight: FontWeight.w700)),
              const SizedBox(height: 6),
              Text(
                '固定阈值 $fixed/${_admins.length}，不允许修改',
                style: const TextStyle(color: Colors.grey),
              ),
            ],
          ),
        ),
      );
    }
    final min = _admins.isEmpty
        ? 0
        : AdminSetValidation.minimumDynamicThreshold(_admins.length);
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: TextField(
          controller: _thresholdController,
          keyboardType: TextInputType.number,
          decoration: InputDecoration(
            labelText: '通过阈值',
            helperText:
                _admins.isEmpty ? '请先添加管理员' : '范围：$min ~ ${_admins.length}',
          ),
        ),
      ),
    );
  }

  void _syncThresholdInput(AdminAccountState account, int adminsLen) {
    final fixed =
        AdminSetValidation.fixedGovernanceThreshold(account.institutionCode);
    if (account.kind == 0 && fixed != null) {
      _thresholdController.text = fixed.toString();
      return;
    }
    if (adminsLen <= 0) {
      _thresholdController.clear();
      return;
    }
    final min = AdminSetValidation.minimumDynamicThreshold(adminsLen);
    final current = int.tryParse(_thresholdController.text.trim());
    if (current == null || current < min || current > adminsLen) {
      _thresholdController.text = min.toString();
    }
  }

  int _readNewThreshold(AdminAccountState account) {
    final fixed =
        AdminSetValidation.fixedGovernanceThreshold(account.institutionCode);
    if (account.kind == 0 && fixed != null) return fixed;
    final value = int.tryParse(_thresholdController.text.trim());
    if (value == null) throw StateError('请输入有效阈值');
    return value;
  }

  Future<void> _submit() async {
    final account = _subject;
    final wallet = _selectedWallet;
    if (account == null || wallet == null) return;
    setState(() {
      _submitting = true;
      _error = null;
    });
    try {
      final newThreshold = _readNewThreshold(account);
      final validated = AdminSetValidation.validate(
        account: account,
        proposerPubkeyHex: wallet.pubkeyHex,
        admins: _admins,
        newThreshold: newThreshold,
      );
      WalletManager? hotWalletManager;
      if (wallet.isHotWallet) {
        hotWalletManager = WalletManager();
        await hotWalletManager.authenticateForSigning();
      }
      Future<Uint8List> signCallback(Uint8List payload) async {
        if (hotWalletManager != null) {
          return hotWalletManager.signWithWalletNoAuth(
              wallet.walletIndex, payload);
        }
        final qrSigner = QrSigner();
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'admin-change-'),
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${AdminAccountIdCodec.hexEncode(payload)}',
          action: QrActions.adminsChange,
        );
        final response = await Navigator.of(context).push(
          MaterialPageRoute(
            builder: (_) => QrSignSessionPage(
              request: request,
              requestJson: qrSigner.encodeRequest(request),
              expectedPubkey: '0x${wallet.pubkeyHex}',
            ),
          ),
        );
        if (response == null) throw Exception('签名已取消');
        return AdminAccountIdCodec.hexDecode(response.body.signatureHex);
      }

      final result = await _changeService.submit(
        account: account,
        admins: validated.admins,
        newThreshold: validated.threshold,
        fromAddress: wallet.address,
        signerPubkey: AdminAccountIdCodec.hexDecode(wallet.pubkeyHex),
        sign: signCallback,
      );
      _accountService.clearAccountCache(account.accountHex);
      _accountService.clearIdentityCache(widget.accountIdentity);
      if (!mounted) return;
      await Navigator.of(context).push(
        MaterialPageRoute(
            builder: (_) => AdminSetChangeConfirmPage(txHash: result.txHash)),
      );
      if (mounted) Navigator.of(context).pop(true);
    } catch (e) {
      if (!mounted) return;
      setState(() => _error = '$e');
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
  }
}
