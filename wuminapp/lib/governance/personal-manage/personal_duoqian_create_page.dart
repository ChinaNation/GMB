import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_query_service.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/qr/envelope.dart';
import 'package:wuminapp_mobile/qr/pages/qr_scan_page.dart'
    show QrScanMode, QrScanPage;
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/my/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import 'personal_manage_service.dart';
import 'personal_proposal_history_service.dart';

/// 个人多签账户创建页面（无需 SFID）。
class PersonalDuoqianCreatePage extends StatefulWidget {
  const PersonalDuoqianCreatePage({super.key});

  @override
  State<PersonalDuoqianCreatePage> createState() =>
      _PersonalDuoqianCreatePageState();
}

class _PersonalDuoqianCreatePageState extends State<PersonalDuoqianCreatePage> {
  static const int _ss58Prefix = 2027;

  final _nameController = TextEditingController();
  final _amountController = TextEditingController();
  final _thresholdController = TextEditingController();

  final _manageService = PersonalManageService();

  bool _submitting = false;
  final List<String> _adminPubkeys = [];
  WalletProfile? _selectedWallet;
  List<WalletProfile> _wallets = [];
  String? _creatorPubkey; // 创建人公钥（始终占管理员列表第一位，不可移除）

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
    if (!mounted) return;
    setState(() {
      _wallets = wallets;
      if (wallets.isNotEmpty) {
        _selectedWallet = wallets.first;
        _syncCreatorAdmin(wallets.first);
        _syncThresholdInput();
      }
    });
  }

  /// 钱包切换时同步更新创建人在管理员列表中的位置。
  void _syncCreatorAdmin(WalletProfile wallet) {
    var pubkey = wallet.pubkeyHex.toLowerCase();
    if (pubkey.startsWith('0x')) pubkey = pubkey.substring(2);
    // 移除旧创建人
    if (_creatorPubkey != null) {
      _adminPubkeys.remove(_creatorPubkey);
    }
    _creatorPubkey = pubkey;
    _adminPubkeys.remove(pubkey); // 防重复
    _adminPubkeys.insert(0, pubkey);
  }

  // ──── 地址预览 ────

  String? _previewAddress() {
    final wallet = _selectedWallet;
    final name = _nameController.text.trim();
    if (wallet == null || name.isEmpty) return null;

    try {
      final creatorBytes = _hexDecode(wallet.pubkeyHex);
      final nameBytes = utf8.encode(name);
      // 与 citizenchain primitives::core_const::{DUOQIAN_DOMAIN, OP_PERSONAL} 严格对齐
      // preimage = b"DUOQIAN_V1" (10B) || 0x04 || ss58_prefix_le (2B) || creator (32B) || account_name
      final input = <int>[
        ...utf8.encode('DUOQIAN_V1'),
        0x04, // OP_PERSONAL
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
          builder: (_) => const QrScanPage(
                mode: QrScanMode.raw,
                customTitle: '扫码添加管理员',
              )),
    );
    if (result == null || !mounted) return;

    // 解析 WUMIN_QR_V1 user_contact(user_duoqian 已于 2026-05-03 下线 → 多签发现走反向索引)
    try {
      final env = QrEnvelope.parse(result.trim());
      if (env.kind == QrKind.userContact) {
        final address = (env.body as dynamic).address?.toString() ?? '';
        if (address.isEmpty) throw const FormatException('缺少 address 字段');
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
    setState(() {
      _adminPubkeys.add(hex);
      _syncThresholdInput();
    });
  }

  void _removeAdmin(int index) {
    // 创建人不可移除
    if (_adminPubkeys[index] == _creatorPubkey) return;
    setState(() {
      _adminPubkeys.removeAt(index);
      _syncThresholdInput();
    });
  }

  int? get _minimumRegularThreshold {
    final count = _adminPubkeys.length;
    if (count < 2) return null;
    return PersonalManageService.minimumRegularThreshold(count);
  }

  int? get _regularThreshold {
    if (_adminPubkeys.length < 2) return null;
    return int.tryParse(_thresholdController.text.trim());
  }

  /// 中文注释：管理员数量变化时只把普通阈值拉回合法范围，
  /// 不把阈值固定死；用户仍可在最低过半到全员之间自由输入。
  void _syncThresholdInput() {
    final min = _minimumRegularThreshold;
    if (min == null) {
      _thresholdController.clear();
      return;
    }
    final max = _adminPubkeys.length;
    final current = int.tryParse(_thresholdController.text.trim());
    final next = current == null
        ? min
        : current < min
            ? min
            : current > max
                ? max
                : current;
    _thresholdController.text = next.toString();
  }

  // ──── 提交 ────

  String? _validate() {
    final name = _nameController.text.trim();
    if (name.isEmpty) return '请输入多签账户名称';
    if (utf8.encode(name).length > 128) return '名称超过最大长度（128 字节）';
    if (_adminPubkeys.length < 2) return '管理员至少 2 人';
    if (_adminPubkeys.length > 64) return '管理员最多 64 人';
    if (_selectedWallet == null) return '请先导入钱包';
    final minThreshold = _minimumRegularThreshold;
    final regularThreshold = _regularThreshold;
    if (minThreshold == null || regularThreshold == null) {
      return '请输入有效的普通阈值';
    }
    if (regularThreshold < minThreshold) {
      return '普通阈值不能小于 $minThreshold（必须过半）';
    }
    if (regularThreshold > _adminPubkeys.length) {
      return '普通阈值不能超过管理员数量';
    }

    final amount = AmountFormat.tryParse(_amountController.text);
    if (amount == null || amount <= 0) return '请输入有效金额';
    if ((amount * 100).round() < 111) return '初始资金不能低于 1.11 元';

    return null;
  }

  Future<void> _submit() async {
    final error = _validate();
    if (error != null) {
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text(error)));
      return;
    }

    setState(() => _submitting = true);

    try {
      final wallet = _selectedWallet!;
      final nameText = _nameController.text.trim();
      final nameBytes = Uint8List.fromList(utf8.encode(nameText));
      final regularThreshold = _regularThreshold!;
      final createThreshold = _adminPubkeys.length;
      final amountYuan = AmountFormat.tryParse(_amountController.text) ?? 0;
      final amountFen = BigInt.from((amountYuan * 100).round());

      final adminPubkeyBytes = _adminPubkeys
          .map((hex) => Uint8List.fromList(_hexDecode(hex)))
          .toList();
      final pubkeyBytes = _hexDecode(wallet.pubkeyHex);

      // 热钱包：先认证，后续用本地签名；冷钱包：走 QR 签名。
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
        // 冷钱包 QR 签名
        final qrSigner = QrSigner();
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'personal-dq-'),
          address: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          display: SignDisplay(
            action: 'propose_create_personal',
            summary: '发起创建个人多签账户提案',
            fields: [
              // propose_create_personal 链端 fields 与 wumin decoder 对齐：
              // 普通阈值由用户输入，注册提案阈值固定全员通过。
              SignDisplayField(
                  key: 'account_name', label: '名称', value: nameText),
              SignDisplayField(
                  key: 'admin_count',
                  label: '管理员数量',
                  value: _adminPubkeys.length.toString()),
              SignDisplayField(
                  key: 'regular_threshold',
                  label: '普通阈值',
                  value: '$regularThreshold/${_adminPubkeys.length}'),
              SignDisplayField(
                  key: 'create_threshold',
                  label: '注册阈值',
                  value: '$createThreshold/${_adminPubkeys.length}'),
              SignDisplayField(
                  key: 'amount_yuan',
                  label: '初始资金',
                  value: AmountFormat.format(amountYuan)),
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

      // 提前查链上 NextProposalId,作为本次创建提案的预测 ID。
      // 写入 Isar `PersonalDuoqianProposalEntity` 时使用,后续详情页打开
      // 通过 PersonalProposalHistoryService 同步链上状态时按此 ID 校准。
      final predictedProposalId =
          await ProposalQueryService().fetchNextProposalId();

      final result = await _manageService.submitProposeCreatePersonal(
        accountName: nameBytes,
        adminPubkeys: adminPubkeyBytes,
        regularThreshold: regularThreshold,
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
          await PersonalDuoqianLocalState.putStatusInTxn(
            isar,
            addrHex,
            PersonalDuoqianLocalState.statusPending,
          );
        });

        // 同步记录创建提案到 PersonalDuoqianProposalEntity(req 5 历史保留)
        await PersonalProposalHistoryService().recordOrUpdate(
          personalAddressHex: addrHex,
          proposalId: predictedProposalId,
          action: PersonalProposalAction.create,
          status: PersonalProposalStatus.voting,
          // 中文注释：runtime 投票引擎创建提案后会在同一事务自动给发起人记一票赞成。
          yesVotes: 1,
          noVotes: 0,
          snapshot: {
            'name': nameText,
            'admin_count': _adminPubkeys.length,
            'regular_threshold': regularThreshold,
            'create_threshold': createThreshold,
            'amount_fen': amountFen.toString(),
          },
        );
      }

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
    final preview = _previewAddress();
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text('创建个人多签',
            style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700)),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: AppTheme.primaryDark,
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
              border:
                  OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
              contentPadding:
                  const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            ),
          ),
          if (preview != null) ...[
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: AppTheme.primaryDark.withValues(alpha: 0.06),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('派生多签地址：',
                      style:
                          TextStyle(fontSize: 12, color: AppTheme.primaryDark)),
                  const SizedBox(height: 4),
                  Text(preview,
                      style: const TextStyle(
                          fontSize: 12, fontFamily: 'monospace')),
                ],
              ),
            ),
          ],
          const SizedBox(height: 20),
          _buildSectionTitle('管理员列表（${_adminPubkeys.length}/64）'),
          const SizedBox(height: 8),
          ..._adminPubkeys.asMap().entries.map((entry) {
            final ss58 = _hexToSs58(entry.value);
            final isCreator = entry.value == _creatorPubkey;
            return ListTile(
              dense: true,
              contentPadding: EdgeInsets.zero,
              leading: CircleAvatar(
                radius: 14,
                backgroundColor: AppTheme.primaryDark.withValues(alpha: 0.08),
                child: Text('${entry.key + 1}',
                    style: const TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.primaryDark)),
              ),
              title: Row(
                children: [
                  Flexible(
                      child: Text(_truncateAddress(ss58),
                          style: const TextStyle(fontSize: 13))),
                  if (isCreator) ...[
                    const SizedBox(width: 6),
                    Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 5, vertical: 1),
                      decoration: BoxDecoration(
                        color: AppTheme.success.withValues(alpha: 0.1),
                        borderRadius: BorderRadius.circular(6),
                      ),
                      child: const Text('创建人',
                          style: TextStyle(
                              fontSize: 10,
                              fontWeight: FontWeight.w600,
                              color: AppTheme.success)),
                    ),
                  ],
                ],
              ),
              trailing: isCreator
                  ? null
                  : IconButton(
                      icon: const Icon(Icons.close,
                          size: 18, color: AppTheme.danger),
                      onPressed: () => _removeAdmin(entry.key),
                    ),
            );
          }),
          OutlinedButton.icon(
            onPressed: _addAdminByQr,
            icon: SvgPicture.asset(
              'assets/icons/scan-line.svg',
              width: 18,
              height: 18,
              colorFilter: const ColorFilter.mode(
                AppTheme.primaryDark,
                BlendMode.srcIn,
              ),
            ),
            label: const Text('扫码添加管理员'),
            style: OutlinedButton.styleFrom(
                foregroundColor: AppTheme.primaryDark,
                side: BorderSide(
                    color: AppTheme.primaryDark.withValues(alpha: 0.3))),
          ),
          const SizedBox(height: 20),
          _buildSectionTitle('阈值规则', note: '注册须全员同意'),
          const SizedBox(height: 8),
          _buildThresholdPreview(),
          const SizedBox(height: 20),
          _buildSectionTitle('初始资金（元）'),
          const SizedBox(height: 8),
          TextField(
            controller: _amountController,
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            inputFormatters: [ThousandSeparatorFormatter()],
            decoration: InputDecoration(
              hintText: '最低 1.11 元',
              border:
                  OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
              contentPadding:
                  const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            ),
          ),
          if (_wallets.length > 1) ...[
            const SizedBox(height: 20),
            _buildSectionTitle('签名钱包'),
            const SizedBox(height: 8),
            DropdownButtonFormField<WalletProfile>(
              initialValue: _selectedWallet,
              items: _wallets
                  .map((w) => DropdownMenuItem(
                      value: w,
                      child: Text(
                          '${w.walletName} (${_truncateAddress(w.address)})',
                          style: const TextStyle(fontSize: 13))))
                  .toList(),
              onChanged: (w) {
                if (w != null) {
                  setState(() {
                    _selectedWallet = w;
                    _syncCreatorAdmin(w);
                    _syncThresholdInput();
                  });
                }
              },
              decoration: InputDecoration(
                  border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(10)),
                  contentPadding:
                      const EdgeInsets.symmetric(horizontal: 12, vertical: 10)),
            ),
          ],
          const SizedBox(height: 28),
          SizedBox(
            width: double.infinity,
            child: ElevatedButton(
              onPressed: _submitting ? null : _submit,
              style: ElevatedButton.styleFrom(
                  backgroundColor: AppTheme.primaryDark,
                  foregroundColor: Colors.white,
                  padding: const EdgeInsets.symmetric(vertical: 14),
                  shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(12))),
              child: _submitting
                  ? const SizedBox(
                      width: 18,
                      height: 18,
                      child: CircularProgressIndicator(
                          strokeWidth: 2, color: Colors.white))
                  : const Text('发起创建提案',
                      style:
                          TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSectionTitle(String title, {String? note}) => Row(
        children: [
          Text(title,
              style: const TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.w600,
                  color: AppTheme.primaryDark)),
          if (note != null) ...[
            const SizedBox(width: 8),
            Text(
              note,
              style: const TextStyle(
                fontSize: 12,
                color: AppTheme.textTertiary,
              ),
            ),
          ],
        ],
      );

  Widget _buildThresholdPreview() {
    final count = _adminPubkeys.length;
    final min = _minimumRegularThreshold;
    final createText = count < 2 ? '至少添加 2 名管理员' : '$count/$count';
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: AppTheme.primaryDark.withValues(alpha: 0.04),
        border: Border.all(color: AppTheme.primaryDark.withValues(alpha: 0.12)),
        borderRadius: BorderRadius.circular(10),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          TextField(
            controller: _thresholdController,
            enabled: count >= 2,
            keyboardType: TextInputType.number,
            inputFormatters: [FilteringTextInputFormatter.digitsOnly],
            onChanged: (_) => setState(() {}),
            decoration: InputDecoration(
              labelText: '普通提案阈值',
              hintText: min == null ? '至少添加 2 名管理员' : '$min 到 $count',
              helperText: min == null
                  ? '至少添加 2 名管理员'
                  : '可输入 $min/$count 到 $count/$count',
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(10),
              ),
              contentPadding:
                  const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            ),
          ),
          const SizedBox(height: 8),
          _buildThresholdRow('注册提案阈值', createText),
        ],
      ),
    );
  }

  Widget _buildThresholdRow(String label, String value) => Row(
        children: [
          Expanded(
            child: Text(label,
                style: const TextStyle(
                    fontSize: 13, color: AppTheme.textSecondary)),
          ),
          Text(value,
              style: const TextStyle(
                  fontSize: 13,
                  fontWeight: FontWeight.w700,
                  color: AppTheme.primaryDark)),
        ],
      );

  String _truncateAddress(String a) => a.length <= 14
      ? a
      : '${a.substring(0, 6)}...${a.substring(a.length - 6)}';
  String _hexToSs58(String hex) =>
      Keyring().encodeAddress(Uint8List.fromList(_hexDecode(hex)), _ss58Prefix);
  String _toHex(List<int> b) {
    final s = StringBuffer();
    for (final v in b) {
      s.write(v.toRadixString(16).padLeft(2, '0'));
    }
    return s.toString();
  }

  List<int> _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    return List.generate(h.length ~/ 2,
        (i) => int.parse(h.substring(i * 2, i * 2 + 2), radix: 16));
  }
}
