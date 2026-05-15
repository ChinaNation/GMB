import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/governance/admins-change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/governance/admins-change/pages/admin_set_change_confirm_page.dart';
import 'package:wuminapp_mobile/governance/admins-change/admin_set_change_qr_adapter.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/admin_set_change_service.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/admin_set_validation.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/admin_subject_service.dart';
import 'package:wuminapp_mobile/governance/admins-change/widgets/admin_set_change_action_bar.dart';
import 'package:wuminapp_mobile/governance/admins-change/widgets/admin_set_diff_card.dart';
import 'package:wuminapp_mobile/governance/admins-change/widgets/admin_set_editor.dart';
import 'package:wuminapp_mobile/governance/admins-change/widgets/admin_subject_card.dart';
import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

class AdminSetChangePage extends StatefulWidget {
  const AdminSetChangePage({
    super.key,
    required this.institution,
    required this.subjectIdentity,
    required this.adminWallets,
  });

  final InstitutionInfo institution;
  final AdminSubjectIdentity subjectIdentity;
  final List<WalletProfile> adminWallets;

  @override
  State<AdminSetChangePage> createState() => _AdminSetChangePageState();
}

class _AdminSetChangePageState extends State<AdminSetChangePage> {
  final _subjectService = AdminSubjectService();
  final _changeService = AdminSetChangeService();
  final _thresholdController = TextEditingController();
  AdminSubjectState? _subject;
  List<String> _newAdmins = const [];
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
      final subject = await _subjectService.fetchByIdentity(
        widget.subjectIdentity,
      );
      if (!mounted) return;
      setState(() {
        _subject = subject;
        _newAdmins = subject?.admins ?? const [];
        if (subject != null) _syncThresholdInput(subject, _newAdmins.length);
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
    final subject = _subject;
    return Scaffold(
      appBar: AppBar(title: const Text('更换管理员')),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : subject == null
              ? Center(child: Text(_error ?? '未查询到管理员主体'))
              : ListView(
                  padding: const EdgeInsets.all(16),
                  children: [
                    AdminSubjectCard(subject: subject),
                    const SizedBox(height: 12),
                    _buildWalletSelector(),
                    const SizedBox(height: 12),
                    AdminSetEditor(
                        admins: _newAdmins,
                        onChanged: (value) => _setNewAdmins(subject, value)),
                    const SizedBox(height: 12),
                    _buildThresholdCard(subject),
                    const SizedBox(height: 12),
                    AdminSetDiffCard(
                        currentAdmins: subject.admins, newAdmins: _newAdmins),
                    if (_error != null) ...[
                      const SizedBox(height: 12),
                      Text(_error!, style: const TextStyle(color: Colors.red)),
                    ],
                  ],
                ),
      bottomNavigationBar: subject == null
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

  void _setNewAdmins(AdminSubjectState subject, List<String> value) {
    setState(() {
      _newAdmins = value;
      _syncThresholdInput(subject, value.length);
    });
  }

  Widget _buildThresholdCard(AdminSubjectState subject) {
    final fixed = AdminSetValidation.fixedGovernanceThreshold(subject.org);
    if (subject.kind == 0 && fixed != null) {
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
                '固定阈值 $fixed/${_newAdmins.length}，不允许修改',
                style: const TextStyle(color: Colors.grey),
              ),
            ],
          ),
        ),
      );
    }
    final min = _newAdmins.isEmpty
        ? 0
        : AdminSetValidation.minimumDynamicThreshold(_newAdmins.length);
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
            helperText: _newAdmins.isEmpty
                ? '请先添加管理员'
                : '范围：$min ~ ${_newAdmins.length}',
          ),
        ),
      ),
    );
  }

  void _syncThresholdInput(AdminSubjectState subject, int adminCount) {
    final fixed = AdminSetValidation.fixedGovernanceThreshold(subject.org);
    if (subject.kind == 0 && fixed != null) {
      _thresholdController.text = fixed.toString();
      return;
    }
    if (adminCount <= 0) {
      _thresholdController.clear();
      return;
    }
    final min = AdminSetValidation.minimumDynamicThreshold(adminCount);
    final current = int.tryParse(_thresholdController.text.trim());
    if (current == null || current < min || current > adminCount) {
      _thresholdController.text = min.toString();
    }
  }

  int _readNewThreshold(AdminSubjectState subject) {
    final fixed = AdminSetValidation.fixedGovernanceThreshold(subject.org);
    if (subject.kind == 0 && fixed != null) return fixed;
    final value = int.tryParse(_thresholdController.text.trim());
    if (value == null) throw StateError('请输入有效阈值');
    return value;
  }

  Future<void> _submit() async {
    final subject = _subject;
    final wallet = _selectedWallet;
    if (subject == null || wallet == null) return;
    setState(() {
      _submitting = true;
      _error = null;
    });
    try {
      final newThreshold = _readNewThreshold(subject);
      final validated = AdminSetValidation.validate(
        subject: subject,
        proposerPubkeyHex: wallet.pubkeyHex,
        newAdmins: _newAdmins,
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
          address: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${AdminSubjectIdCodec.hexEncode(payload)}',
          display: AdminSetChangeQrAdapter.buildDisplay(
            subject: subject,
            newAdmins: validated.admins,
            newThreshold: validated.threshold,
          ),
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
        return AdminSubjectIdCodec.hexDecode(response.body.signature);
      }

      final result = await _changeService.submit(
        subject: subject,
        newAdmins: validated.admins,
        newThreshold: validated.threshold,
        fromAddress: wallet.address,
        signerPubkey: AdminSubjectIdCodec.hexDecode(wallet.pubkeyHex),
        sign: signCallback,
      );
      _subjectService.clearSubjectCache(subject.subjectIdHex);
      _subjectService.clearIdentityCache(widget.subjectIdentity);
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
