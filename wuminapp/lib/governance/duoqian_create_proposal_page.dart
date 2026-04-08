import 'dart:convert';
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

  final _sfidIdController = TextEditingController();
  final _nameController = TextEditingController();
  final _amountController = TextEditingController();
  final _thresholdController = TextEditingController();

  final _manageService = DuoqianManageService();

  bool _submitting = false;
  String? _sfidError;
  String? _registeredAddress; // 查链获得的派生多签地址 hex
  bool _checkingSfid = false;

  // 管理员列表（公钥 hex，不含 0x）
  final List<String> _adminPubkeys = [];
  String? _creatorPubkey; // 创建人公钥（始终占管理员列表第一位，不可移除）

  late WalletProfile _selectedWallet;

  @override
  void initState() {
    super.initState();
    debugPrint('[DuoqianCreate-Diag] initState: adminWallets.length=${widget.adminWallets.length}');
    if (widget.adminWallets.isNotEmpty) {
      final w = widget.adminWallets.first;
      debugPrint('[DuoqianCreate-Diag] first wallet: name=${w.walletName} '
          'pubkeyHex.len=${w.pubkeyHex.length} address.len=${w.address.length} '
          'signMode=${w.signMode}');
    }
    _selectedWallet = widget.adminWallets.first;
    _syncCreatorAdmin(widget.adminWallets.first);
    debugPrint('[DuoqianCreate-Diag] after sync: _adminPubkeys=${_adminPubkeys.length} '
        'creator=${_creatorPubkey?.substring(0, 8)}...');
  }

  /// 钱包切换时同步更新创建人在管理员列表中的位置。
  void _syncCreatorAdmin(WalletProfile wallet) {
    var pubkey = wallet.pubkeyHex.toLowerCase();
    if (pubkey.startsWith('0x')) pubkey = pubkey.substring(2);
    if (_creatorPubkey != null) {
      _adminPubkeys.remove(_creatorPubkey);
    }
    _creatorPubkey = pubkey;
    _adminPubkeys.remove(pubkey);
    _adminPubkeys.insert(0, pubkey);
  }

  @override
  void dispose() {
    _sfidIdController.dispose();
    _nameController.dispose();
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
    final nameText = _nameController.text.trim();
    if (nameText.isEmpty) {
      setState(() => _sfidError = '请输入账户名称');
      return;
    }

    final sfidBytes = Uint8List.fromList(utf8.encode(sfidText));
    if (sfidBytes.length > 96) {
      setState(() => _sfidError = 'SFID ID 超过最大长度（96 字节）');
      return;
    }
    final nameBytes = Uint8List.fromList(utf8.encode(nameText));

    setState(() {
      _checkingSfid = true;
      _sfidError = null;
      _registeredAddress = null;
    });

    try {
      final address = await _manageService.fetchSfidRegisteredAddress(sfidBytes, nameBytes);
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
      MaterialPageRoute(builder: (_) => const QrScanPage(
        mode: QrScanMode.raw,
        customTitle: '扫码添加管理员',
      )),
    );
    if (result == null || !mounted) return;

    // 解析 WUMIN_USER_V1.0.0 协议
    try {
      final json = jsonDecode(result.trim());
      if (json is Map && json['proto'] == 'WUMIN_USER_V1.0.0') {
        final address = json['address']?.toString();
        if (address == null || address.isEmpty) {
          throw FormatException('缺少 address 字段');
        }
        final pubkey = Keyring().decodeAddress(address);
        _addAdminPubkey(_toHex(pubkey));
        return;
      }
    } catch (e) {
      if (e is FormatException) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('二维码格式错误：$e')),
        );
        return;
      }
      // 非 JSON，继续下方处理
    }

    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('请扫描有效的用户二维码')),
    );
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
    if (_adminPubkeys[index] == _creatorPubkey) return;
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

      // 热钱包：先认证，后续用本地签名；冷钱包：走 QR 签名。
      WalletManager? hotWalletManager;
      if (wallet.isHotWallet) {
        hotWalletManager = WalletManager();
        await hotWalletManager.authenticateForSigning();
      }

      Future<Uint8List> signCallback(Uint8List payload) async {
        if (hotWalletManager != null) {
          return await hotWalletManager.signWithWalletNoAuth(wallet.walletIndex, payload);
        }
        // 冷钱包 QR 签名
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

      final nameBytes =
          Uint8List.fromList(utf8.encode(_nameController.text.trim()));
      final result = await _manageService.submitProposeCreate(
        sfidId: sfidBytes,
        name: nameBytes,
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
    debugPrint('[DuoqianCreate-Diag] build START: _adminPubkeys=${_adminPubkeys.length} '
        '_registeredAddress=${_registeredAddress != null} _checkingSfid=$_checkingSfid');
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '创建多签账户',
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
          // SFID ID 输入
          _buildSectionTitle('SFID ID'),
          const SizedBox(height: 8),
          // 中文注释：ElevatedButton 不能直接放在水平 unbounded 约束下，否则
          // _RenderInputPadding 会抛 "BoxConstraints forces an infinite width"，
          // 进而把整个 ListView 拖成 NEEDS-LAYOUT 状态，渲染出空白页（白屏）。
          // 用 IntrinsicHeight + Row(MainAxisSize.max) + 固定宽度按钮包裹。
          Row(
            mainAxisSize: MainAxisSize.max,
            crossAxisAlignment: CrossAxisAlignment.start,
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
              SizedBox(
                width: 84,
                height: 48,
                child: ElevatedButton(
                  onPressed: _checkingSfid ? null : _checkSfidRegistration,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppTheme.primaryDark,
                    foregroundColor: Colors.white,
                    padding: EdgeInsets.zero,
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
              ),
            ],
          ),
          if (_registeredAddress != null) ...[
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: AppTheme.success.withValues(alpha: 0.08),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: AppTheme.success.withValues(alpha: 0.3)),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    '已注册，派生多签地址：',
                    style: TextStyle(fontSize: 12, color: AppTheme.success),
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
          _buildSectionTitle('账户名称'),
          const SizedBox(height: 8),
          TextField(
            controller: _nameController,
            decoration: InputDecoration(
              hintText: '输入多签账户名称（如：运营账户）',
              border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
              contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            ),
          ),

          const SizedBox(height: 20),

          // 管理员列表
          _buildSectionTitle('管理员列表（${_adminPubkeys.length}/64）'),
          const SizedBox(height: 8),
          ..._adminPubkeys.asMap().entries.map((entry) {
            final index = entry.key;
            final pubkey = entry.value;
            final ss58 = _hexToSs58(pubkey);
            final isCreator = pubkey == _creatorPubkey;
            return ListTile(
              dense: true,
              contentPadding: EdgeInsets.zero,
              leading: CircleAvatar(
                radius: 14,
                backgroundColor: AppTheme.primaryDark.withValues(alpha: 0.08),
                child: Text(
                  '${index + 1}',
                  style: const TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    color: AppTheme.primaryDark,
                  ),
                ),
              ),
              title: Row(
                children: [
                  Flexible(child: Text(_truncateAddress(ss58), style: const TextStyle(fontSize: 13))),
                  if (isCreator) ...[
                    const SizedBox(width: 6),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                      decoration: BoxDecoration(
                        color: AppTheme.success.withValues(alpha: 0.1),
                        borderRadius: BorderRadius.circular(6),
                      ),
                      child: const Text('创建人', style: TextStyle(fontSize: 10, fontWeight: FontWeight.w600, color: AppTheme.success)),
                    ),
                  ],
                ],
              ),
              trailing: isCreator
                  ? null
                  : IconButton(
                      icon: Icon(Icons.close, size: 18, color: AppTheme.danger),
                      onPressed: () => _removeAdmin(index),
                    ),
            );
          }),
          OutlinedButton.icon(
            onPressed: _addAdminByQr,
            icon: const Icon(Icons.qr_code_scanner, size: 18),
            label: const Text('扫码添加管理员'),
            style: OutlinedButton.styleFrom(
              foregroundColor: AppTheme.primaryDark,
              side: BorderSide(color: AppTheme.primaryDark.withValues(alpha: 0.3)),
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
                if (w != null) setState(() { _selectedWallet = w; _syncCreatorAdmin(w); });
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
                backgroundColor: AppTheme.primaryDark,
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

  List<int> _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = List<int>.filled(h.length ~/ 2, 0);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }
}
