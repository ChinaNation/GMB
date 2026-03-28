import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../util/amount_format.dart';
import 'duoqian_manage_service.dart';
import 'institution_data.dart';
import '../qr/pages/qr_scan_page.dart' show QrScanPage, QrScanMode;
import '../qr/pages/qr_sign_session_page.dart';
import '../rpc/chain_rpc.dart';
import '../signer/qr_signer.dart';
import '../wallet/core/wallet_manager.dart';

/// 创建多签账户提案页面。
///
/// 用户输入 SFID ID 查询注册状态，填写管理员列表、阈值、初始资金后发起提案。
class DuoqianCreateProposalPage extends StatefulWidget {
  const DuoqianCreateProposalPage({
    super.key,
    required this.institution,
    required this.adminWallets,
  });

  final InstitutionInfo institution;
  final List<WalletProfile> adminWallets;

  @override
  State<DuoqianCreateProposalPage> createState() =>
      _DuoqianCreateProposalPageState();
}

class _DuoqianCreateProposalPageState
    extends State<DuoqianCreateProposalPage> {
  static const Color _inkGreen = Color(0xFF0B3D2E);

  final _sfidIdController = TextEditingController();
  final _amountController = TextEditingController();
  final _thresholdController = TextEditingController();

  final _manageService = DuoqianManageService();

  bool _submitting = false;
  String? _sfidError;
  String? _registeredAddress; // 查链获得的派生多签地址 hex
  bool _checkingSfid = false;

  // 管理员列表（公钥 hex，不含 0x）
  final List<String> _adminPubkeys = [];

  late WalletProfile _selectedWallet;

  @override
  void initState() {
    super.initState();
    _selectedWallet = widget.adminWallets.first;
  }

  @override
  void dispose() {
    _sfidIdController.dispose();
    _amountController.dispose();
    _thresholdController.dispose();
    super.dispose();
  }

  // ──── SFID 查询 ────

  Future<void> _checkSfidRegistration() async {
    final sfidText = _sfidIdController.text.trim();
    if (sfidText.isEmpty) {
      setState(() => _sfidError = 'SFID ID 不能为空');
      return;
    }

    final sfidBytes = Uint8List.fromList(utf8.encode(sfidText));
    if (sfidBytes.length > 96) {
      setState(() => _sfidError = 'SFID ID 超过最大长度（96 字节）');
      return;
    }

    setState(() {
      _checkingSfid = true;
      _sfidError = null;
      _registeredAddress = null;
    });

    try {
      final address = await _manageService.fetchSfidRegisteredAddress(sfidBytes);
      if (!mounted) return;

      if (address == null) {
        setState(() {
          _sfidError = '此 SFID ID 尚未在链上注册';
          _checkingSfid = false;
        });
      } else {
        setState(() {
          _registeredAddress = address;
          _checkingSfid = false;
        });
      }
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _sfidError = '查询失败：$e';
        _checkingSfid = false;
      });
    }
  }

  // ──── 管理员管理 ────

  Future<void> _addAdminByQr() async {
    final result = await Navigator.push<String>(
      context,
      MaterialPageRoute(builder: (_) => const QrScanPage(mode: QrScanMode.transfer)),
    );
    if (result == null || !mounted) return;

    // 尝试解码为 SS58 地址
    try {
      final pubkey = Keyring().decodeAddress(result.trim());
      final hex = _toHex(pubkey);
      _addAdminPubkey(hex);
    } catch (_) {
      // 如果不是地址，尝试作为 hex 公钥
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
    if (_registeredAddress == null) return '请先查询 SFID 注册状态';
    if (_adminPubkeys.length < 2) return '管理员至少 2 人';

    final thresholdText = _thresholdController.text.trim();
    final threshold = int.tryParse(thresholdText);
    if (threshold == null) return '请输入有效的阈值';

    final adminCount = _adminPubkeys.length;
    final minThreshold = (adminCount + 1) ~/ 2;
    if (minThreshold < 2) {
      if (threshold < 2) return '阈值不能小于 2';
    } else if (threshold < minThreshold) {
      return '阈值不能小于 $minThreshold（管理员数的一半）';
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
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(error)),
      );
      return;
    }

    setState(() => _submitting = true);

    try {
      final sfidBytes =
          Uint8List.fromList(utf8.encode(_sfidIdController.text.trim()));
      final threshold = int.parse(_thresholdController.text.trim());
      final amountYuan = double.parse(_amountController.text.trim());
      final amountFen = BigInt.from((amountYuan * 100).round());

      final adminPubkeyBytes = _adminPubkeys
          .map((hex) => Uint8List.fromList(_hexDecode(hex)))
          .toList();

      final wallet = _selectedWallet;
      final pubkeyBytes = _hexDecode(wallet.pubkeyHex);

      Future<Uint8List> signCallback(Uint8List payload) async {
        final qrSigner = QrSigner();
        final rv = await ChainRpc().fetchRuntimeVersion();
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'create-dq-'),
          account: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          specVersion: rv.specVersion,
          display: {
            'action': 'propose_create',
            'action_label': '创建多签提案',
            'summary': '发起创建多签账户提案',
            'fields': [
              {'key': 'sfid_id', 'label': 'SFID ID', 'value': _sfidIdController.text.trim()},
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

      final result = await _manageService.submitProposeCreate(
        sfidId: sfidBytes,
        adminCount: _adminPubkeys.length,
        adminPubkeys: adminPubkeyBytes,
        threshold: threshold,
        amountFen: amountFen,
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(pubkeyBytes),
        sign: signCallback,
      );

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
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '创建多签账户',
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
          // SFID ID 输入
          _buildSectionTitle('SFID ID'),
          const SizedBox(height: 8),
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _sfidIdController,
                  decoration: InputDecoration(
                    hintText: '输入 SFID 机构标识',
                    errorText: _sfidError,
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                    contentPadding:
                        const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                  ),
                ),
              ),
              const SizedBox(width: 8),
              ElevatedButton(
                onPressed: _checkingSfid ? null : _checkSfidRegistration,
                style: ElevatedButton.styleFrom(
                  backgroundColor: _inkGreen,
                  foregroundColor: Colors.white,
                  padding:
                      const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                ),
                child: _checkingSfid
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(
                            strokeWidth: 2, color: Colors.white),
                      )
                    : const Text('查询'),
              ),
            ],
          ),
          if (_registeredAddress != null) ...[
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: Colors.green.withValues(alpha: 0.08),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Colors.green.withValues(alpha: 0.3)),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    '已注册，派生多签地址：',
                    style: TextStyle(fontSize: 12, color: Colors.green),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    _hexToSs58(_registeredAddress!),
                    style: const TextStyle(fontSize: 12, fontFamily: 'monospace'),
                  ),
                ],
              ),
            ),
          ],

          const SizedBox(height: 20),

          // 管理员列表
          _buildSectionTitle('管理员列表（${_adminPubkeys.length}/64）'),
          const SizedBox(height: 8),
          ..._adminPubkeys.asMap().entries.map((entry) {
            final index = entry.key;
            final pubkey = entry.value;
            final ss58 = _hexToSs58(pubkey);
            return ListTile(
              dense: true,
              contentPadding: EdgeInsets.zero,
              leading: CircleAvatar(
                radius: 14,
                backgroundColor: _inkGreen.withValues(alpha: 0.08),
                child: Text(
                  '${index + 1}',
                  style: const TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    color: _inkGreen,
                  ),
                ),
              ),
              title: Text(
                _truncateAddress(ss58),
                style: const TextStyle(fontSize: 13),
              ),
              trailing: IconButton(
                icon: Icon(Icons.close, size: 18, color: Colors.red[300]),
                onPressed: () => _removeAdmin(index),
              ),
            );
          }),
          OutlinedButton.icon(
            onPressed: _addAdminByQr,
            icon: const Icon(Icons.qr_code_scanner, size: 18),
            label: const Text('扫码添加管理员'),
            style: OutlinedButton.styleFrom(
              foregroundColor: _inkGreen,
              side: BorderSide(color: _inkGreen.withValues(alpha: 0.3)),
            ),
          ),

          const SizedBox(height: 20),

          // 阈值
          _buildSectionTitle('通过阈值'),
          const SizedBox(height: 8),
          TextField(
            controller: _thresholdController,
            keyboardType: TextInputType.number,
            decoration: InputDecoration(
              hintText: _adminPubkeys.length >= 2
                  ? '范围：${(_adminPubkeys.length + 1) ~/ 2} ~ ${_adminPubkeys.length}'
                  : '请先添加管理员',
              border:
                  OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
              contentPadding:
                  const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            ),
          ),

          const SizedBox(height: 20),

          // 初始资金
          _buildSectionTitle('初始资金（元）'),
          const SizedBox(height: 8),
          TextField(
            controller: _amountController,
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            decoration: InputDecoration(
              hintText: '最低 1.11 元',
              border:
                  OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
              contentPadding:
                  const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            ),
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
                border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(10)),
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
                backgroundColor: _inkGreen,
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
                  : const Text('发起创建提案',
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
        color: _inkGreen,
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

  List<int> _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = List<int>.filled(h.length ~/ 2, 0);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }
}
