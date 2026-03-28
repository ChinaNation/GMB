import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:isar/isar.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../Isar/wallet_isar.dart';
import '../util/amount_format.dart';
import 'duoqian_manage_service.dart';
import '../qr/pages/qr_scan_page.dart' show QrScanPage, QrScanMode;
import '../qr/pages/qr_sign_session_page.dart';
import '../rpc/chain_rpc.dart';
import '../signer/qr_signer.dart';
import '../wallet/core/wallet_manager.dart';

/// 个人多签账户创建页面（无需 SFID）。
class PersonalDuoqianCreatePage extends StatefulWidget {
  const PersonalDuoqianCreatePage({super.key});

  @override
  State<PersonalDuoqianCreatePage> createState() =>
      _PersonalDuoqianCreatePageState();
}

class _PersonalDuoqianCreatePageState
    extends State<PersonalDuoqianCreatePage> {
  static const Color _inkGreen = Color(0xFF0B3D2E);
  static const int _ss58Prefix = 2027;

  final _nameController = TextEditingController();
  final _amountController = TextEditingController();
  final _thresholdController = TextEditingController();

  final _manageService = DuoqianManageService();

  bool _submitting = false;
  final List<String> _adminPubkeys = [];
  WalletProfile? _selectedWallet;
  List<WalletProfile> _coldWallets = [];

  @override
  void initState() {
    super.initState();
    _loadWallets();
  }

  @override
  void dispose() {
    _nameController.dispose();
    _amountController.dispose();
    _thresholdController.dispose();
    super.dispose();
  }

  Future<void> _loadWallets() async {
    final wm = WalletManager();
    final wallets = await wm.getWallets();
    final cold = wallets.where((w) => w.signMode == 'external').toList();
    if (!mounted) return;
    setState(() {
      _coldWallets = cold;
      _selectedWallet = cold.isNotEmpty ? cold.first : null;
    });
  }

  // ──── 地址预览 ────

  String? _previewAddress() {
    final wallet = _selectedWallet;
    final name = _nameController.text.trim();
    if (wallet == null || name.isEmpty) return null;

    try {
      final creatorBytes = _hexDecode(wallet.pubkeyHex);
      final nameBytes = utf8.encode(name);
      final input = <int>[
        ...utf8.encode('DUOQIAN_PERSONAL_V1'),
        ..._u16LeBytes(_ss58Prefix),
        ...creatorBytes,
        ...nameBytes,
      ];
      final digest = Hasher.blake2b256.hash(Uint8List.fromList(input));
      return Keyring().encodeAddress(digest, _ss58Prefix);
    } catch (_) {
      return null;
    }
  }

  List<int> _u16LeBytes(int value) => [value & 0xFF, (value >> 8) & 0xFF];

  // ──── 管理员管理 ────

  Future<void> _addAdminByQr() async {
    final result = await Navigator.push<String>(
      context,
      MaterialPageRoute(
          builder: (_) => const QrScanPage(mode: QrScanMode.transfer)),
    );
    if (result == null || !mounted) return;
    try {
      final pubkey = Keyring().decodeAddress(result.trim());
      _addAdminPubkey(_toHex(pubkey));
    } catch (_) {
      final clean = result.trim().startsWith('0x')
          ? result.trim().substring(2)
          : result.trim();
      if (clean.length == 64) {
        _addAdminPubkey(clean.toLowerCase());
      } else {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('无法识别为有效地址或公钥')),
        );
      }
    }
  }

  void _addAdminPubkey(String hex) {
    if (_adminPubkeys.contains(hex)) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('该管理员已在列表中')),
      );
      return;
    }
    if (_adminPubkeys.length >= 64) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('管理员数量已达上限（64）')),
      );
      return;
    }
    setState(() => _adminPubkeys.add(hex));
  }

  void _removeAdmin(int index) {
    setState(() => _adminPubkeys.removeAt(index));
  }

  // ──── 提交 ────

  String? _validate() {
    final name = _nameController.text.trim();
    if (name.isEmpty) return '请输入多签账户名称';
    if (utf8.encode(name).length > 128) return '名称超过最大长度（128 字节）';
    if (_adminPubkeys.length < 2) return '管理员至少 2 人';
    if (_selectedWallet == null) return '请先导入冷钱包';

    final thresholdText = _thresholdController.text.trim();
    final threshold = int.tryParse(thresholdText);
    if (threshold == null) return '请输入有效的阈值';
    final adminCount = _adminPubkeys.length;
    final minThreshold = (adminCount + 1) ~/ 2;
    if (minThreshold < 2) {
      if (threshold < 2) return '阈值不能小于 2';
    } else if (threshold < minThreshold) {
      return '阈值不能小于 $minThreshold';
    }
    if (threshold > adminCount) return '阈值不能超过管理员数量';

    final amountText = _amountController.text.trim();
    final amount = double.tryParse(amountText);
    if (amount == null || amount <= 0) return '请输入有效金额';
    if ((amount * 100).round() < 111) return '初始资金不能低于 1.11 元';

    return null;
  }

  Future<void> _submit() async {
    final error = _validate();
    if (error != null) {
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(error)));
      return;
    }

    setState(() => _submitting = true);

    try {
      final wallet = _selectedWallet!;
      final nameText = _nameController.text.trim();
      final nameBytes = Uint8List.fromList(utf8.encode(nameText));
      final threshold = int.parse(_thresholdController.text.trim());
      final amountYuan = double.parse(_amountController.text.trim());
      final amountFen = BigInt.from((amountYuan * 100).round());

      final adminPubkeyBytes = _adminPubkeys
          .map((hex) => Uint8List.fromList(_hexDecode(hex)))
          .toList();
      final pubkeyBytes = _hexDecode(wallet.pubkeyHex);

      Future<Uint8List> signCallback(Uint8List payload) async {
        final qrSigner = QrSigner();
        final rv = await ChainRpc().fetchRuntimeVersion();
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'personal-dq-'),
          account: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          specVersion: rv.specVersion,
          display: {
            'action': 'propose_create_personal',
            'action_label': '创建个人多签',
            'summary': '发起创建个人多签账户提案',
            'fields': [
              {'key': 'name', 'label': '名称', 'value': nameText},
              {'key': 'admin_count', 'label': '管理员数量', 'value': _adminPubkeys.length.toString()},
              {'key': 'threshold', 'label': '阈值', 'value': '$threshold/${_adminPubkeys.length}'},
              {'key': 'amount_yuan', 'label': '初始资金', 'value': AmountFormat.format(amountYuan, symbol: ''), 'format': 'currency'},
            ],
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
        return Uint8List.fromList(_hexDecode(response.signature));
      }

      final result = await _manageService.submitProposeCreatePersonal(
        name: nameBytes,
        adminCount: _adminPubkeys.length,
        adminPubkeys: adminPubkeyBytes,
        threshold: threshold,
        amountFen: amountFen,
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(pubkeyBytes),
        sign: signCallback,
      );

      // 存入本地 Isar
      final previewAddr = _previewAddress();
      if (previewAddr != null) {
        final addrHex = _toHex(Keyring().decodeAddress(previewAddr));
        final isar = await WalletIsar.instance.db();
        await isar.writeTxn(() async {
          final entity = PersonalDuoqianEntity()
            ..duoqianAddress = addrHex
            ..name = nameText
            ..creatorAddress = wallet.address
            ..addedAtMillis = DateTime.now().millisecondsSinceEpoch;
          await isar.personalDuoqianEntitys.put(entity);
        });
      }

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('提案已提交：${_truncateAddress(result.txHash)}'),
          backgroundColor: _inkGreen,
        ),
      );
      Navigator.of(context).pop(true);
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('提交失败：$e'), backgroundColor: Colors.red),
      );
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
  }

  // ──── UI ────

  @override
  Widget build(BuildContext context) {
    final preview = _previewAddress();
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text('创建个人多签', style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700)),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: _inkGreen,
        elevation: 0,
        scrolledUnderElevation: 0.5,
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          _buildSectionTitle('多签账户名称'),
          const SizedBox(height: 8),
          TextField(
            controller: _nameController,
            onChanged: (_) => setState(() {}),
            decoration: InputDecoration(
              hintText: '输入名称（如：家庭基金）',
              border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
              contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            ),
          ),
          if (preview != null) ...[
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: _inkGreen.withValues(alpha: 0.06),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('派生多签地址：', style: TextStyle(fontSize: 12, color: _inkGreen)),
                  const SizedBox(height: 4),
                  Text(preview, style: const TextStyle(fontSize: 12, fontFamily: 'monospace')),
                ],
              ),
            ),
          ],

          const SizedBox(height: 20),
          _buildSectionTitle('管理员列表（${_adminPubkeys.length}/64）'),
          const SizedBox(height: 8),
          ..._adminPubkeys.asMap().entries.map((entry) {
            final ss58 = _hexToSs58(entry.value);
            return ListTile(
              dense: true,
              contentPadding: EdgeInsets.zero,
              leading: CircleAvatar(
                radius: 14,
                backgroundColor: _inkGreen.withValues(alpha: 0.08),
                child: Text('${entry.key + 1}', style: const TextStyle(fontSize: 11, fontWeight: FontWeight.w600, color: _inkGreen)),
              ),
              title: Text(_truncateAddress(ss58), style: const TextStyle(fontSize: 13)),
              trailing: IconButton(
                icon: Icon(Icons.close, size: 18, color: Colors.red[300]),
                onPressed: () => _removeAdmin(entry.key),
              ),
            );
          }),
          OutlinedButton.icon(
            onPressed: _addAdminByQr,
            icon: const Icon(Icons.qr_code_scanner, size: 18),
            label: const Text('扫码添加管理员'),
            style: OutlinedButton.styleFrom(foregroundColor: _inkGreen, side: BorderSide(color: _inkGreen.withValues(alpha: 0.3))),
          ),

          const SizedBox(height: 20),
          _buildSectionTitle('通过阈值'),
          const SizedBox(height: 8),
          TextField(
            controller: _thresholdController,
            keyboardType: TextInputType.number,
            decoration: InputDecoration(
              hintText: _adminPubkeys.length >= 2 ? '范围：${(_adminPubkeys.length + 1) ~/ 2} ~ ${_adminPubkeys.length}' : '请先添加管理员',
              border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
              contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            ),
          ),

          const SizedBox(height: 20),
          _buildSectionTitle('初始资金（元）'),
          const SizedBox(height: 8),
          TextField(
            controller: _amountController,
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            decoration: InputDecoration(
              hintText: '最低 1.11 元',
              border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
              contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            ),
          ),

          if (_coldWallets.length > 1) ...[
            const SizedBox(height: 20),
            _buildSectionTitle('签名钱包'),
            const SizedBox(height: 8),
            DropdownButtonFormField<WalletProfile>(
              value: _selectedWallet,
              items: _coldWallets.map((w) => DropdownMenuItem(value: w, child: Text('${w.walletName} (${_truncateAddress(w.address)})', style: const TextStyle(fontSize: 13)))).toList(),
              onChanged: (w) { if (w != null) setState(() => _selectedWallet = w); },
              decoration: InputDecoration(border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)), contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10)),
            ),
          ],

          const SizedBox(height: 28),
          SizedBox(
            width: double.infinity,
            child: ElevatedButton(
              onPressed: _submitting ? null : _submit,
              style: ElevatedButton.styleFrom(backgroundColor: _inkGreen, foregroundColor: Colors.white, padding: const EdgeInsets.symmetric(vertical: 14), shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12))),
              child: _submitting
                  ? const SizedBox(width: 18, height: 18, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                  : const Text('发起创建提案', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSectionTitle(String title) => Text(title, style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600, color: _inkGreen));

  String _truncateAddress(String a) => a.length <= 14 ? a : '${a.substring(0, 6)}...${a.substring(a.length - 6)}';
  String _hexToSs58(String hex) => Keyring().encodeAddress(Uint8List.fromList(_hexDecode(hex)), _ss58Prefix);
  String _toHex(List<int> b) { final s = StringBuffer(); for (final v in b) { s.write(v.toRadixString(16).padLeft(2, '0')); } return s.toString(); }
  List<int> _hexDecode(String hex) { final h = hex.startsWith('0x') ? hex.substring(2) : hex; return List.generate(h.length ~/ 2, (i) => int.parse(h.substring(i * 2, i * 2 + 2), radix: 16)); }
}
