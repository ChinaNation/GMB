import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:smoldot/smoldot.dart' show LightClientStatusSnapshot;
import 'package:citizenapp/governance/shared/multisig_create_amount_rules.dart';
import 'package:citizenapp/governance/shared/reserved_account_names.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/governance/shared/institution_info.dart';
import 'package:citizenapp/isar/wallet_isar.dart';
import 'package:citizenapp/qr/pages/qr_scan_page.dart'
    show QrScanMode, QrScanPage;
import 'package:citizenapp/qr/pages/qr_sign_session_page.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/widgets/chain_progress_banner.dart';
import 'package:citizenapp/my/util/amount_format.dart';
import 'package:citizenapp/wallet/capabilities/api_client.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import 'institution_manage_service.dart';

/// 创建机构多签账户提案页面。
///
/// 用户输入 CID ID 查询注册状态，填写管理员列表、阈值、初始资金后发起提案。
class InstitutionMultisigCreatePage extends StatefulWidget {
  const InstitutionMultisigCreatePage({
    super.key,
    required this.institution,
    required this.adminWallets,
  });

  final InstitutionInfo institution;
  final List<WalletProfile> adminWallets;

  @override
  State<InstitutionMultisigCreatePage> createState() =>
      _InstitutionMultisigCreatePageState();
}

class _InstitutionMultisigCreatePageState
    extends State<InstitutionMultisigCreatePage> {
  final _cidNumberController = TextEditingController();
  final _thresholdController = TextEditingController();
  final Map<String, TextEditingController> _accountAmountControllers = {};

  final _manageService = InstitutionManageService();
  final _apiClient = ApiClient();

  bool _submitting = false;
  String? _cidError;
  bool _checkingCid = false;
  LightClientStatusSnapshot? _chainProgress;
  String? _chainProgressError;

  // ── 机构账户列表（从 CID 后端查询） ──
  String? _institutionCidFullName; // 查到的机构全称
  String? _institutionCidShortName; // 查到的机构简称
  List<InstitutionAccountEntry> _accounts = []; // 机构下所有账户

  // 管理员列表（公钥 hex，不含 0x）
  final List<String> _admins = [];
  String? _creatorPubkey; // 创建人公钥（始终占管理员列表第一位，不可移除）

  late WalletProfile _selectedWallet;

  @override
  void initState() {
    super.initState();
    debugPrint(
        '[MultisigCreate-Diag] initState: adminWallets.length=${widget.adminWallets.length}');
    if (widget.adminWallets.isNotEmpty) {
      final w = widget.adminWallets.first;
      debugPrint('[MultisigCreate-Diag] first wallet: name=${w.walletName} '
          'pubkeyHex.len=${w.pubkeyHex.length} address.len=${w.address.length} '
          'signMode=${w.signMode}');
    }
    _selectedWallet = widget.adminWallets.first;
    _syncCreatorAdmin(widget.adminWallets.first);
    debugPrint('[MultisigCreate-Diag] after sync: _admins=${_admins.length} '
        'creator=${_creatorPubkey?.substring(0, 8)}...');
  }

  /// 钱包切换时同步更新创建人在管理员列表中的位置。
  void _syncCreatorAdmin(WalletProfile wallet) {
    var pubkey = wallet.pubkeyHex.toLowerCase();
    if (pubkey.startsWith('0x')) pubkey = pubkey.substring(2);
    if (_creatorPubkey != null) {
      _admins.remove(_creatorPubkey);
    }
    _creatorPubkey = pubkey;
    _admins.remove(pubkey);
    _admins.insert(0, pubkey);
    _syncThresholdInput();
  }

  @override
  void dispose() {
    _cidNumberController.dispose();
    _thresholdController.dispose();
    _disposeAccountAmountControllers();
    super.dispose();
  }

  // ──── CID 查询（通过 CID 后端 API） ────

  /// 输入 CID ID 后点击"查询"：从 CID 后端获取机构全称/简称 + 账户列表。
  Future<void> _checkCidRegistration() async {
    final cidText = _cidNumberController.text.trim();
    if (cidText.isEmpty) {
      setState(() => _cidError = 'CID ID 不能为空');
      return;
    }

    final cidBytes = Uint8List.fromList(utf8.encode(cidText));
    if (cidBytes.length > 96) {
      setState(() => _cidError = 'CID ID 超过最大长度（96 字节）');
      return;
    }

    setState(() {
      _checkingCid = true;
      _cidError = null;
      _institutionCidFullName = null;
      _institutionCidShortName = null;
      _accounts = [];
    });
    _disposeAccountAmountControllers();

    try {
      final resp = await _apiClient.fetchInstitutionAccounts(cidText);
      if (!mounted) return;

      if (resp.accounts.isEmpty) {
        setState(() {
          _cidError = '该机构尚未创建任何账户';
          _checkingCid = false;
        });
      } else {
        _replaceAccountAmountControllers(resp.accounts);
        setState(() {
          _institutionCidFullName = resp.cidFullName;
          _institutionCidShortName = resp.cidShortName;
          _accounts = resp.accounts;
          _checkingCid = false;
        });
      }
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _cidError = '查询失败：$e';
        _checkingCid = false;
      });
    }
  }

  void _disposeAccountAmountControllers() {
    for (final controller in _accountAmountControllers.values) {
      controller.dispose();
    }
    _accountAmountControllers.clear();
  }

  void _replaceAccountAmountControllers(
      List<InstitutionAccountEntry> accounts) {
    _disposeAccountAmountControllers();
    for (final account in accounts) {
      final name = account.accountName.trim();
      if (name.isEmpty) continue;
      _accountAmountControllers[name] = TextEditingController();
    }
  }

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

    // 解析 QR_V1 k=3 user_contact(多签发现走反向索引)
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
      // 非 JSON，继续下方处理
    }

    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('请扫描有效的用户二维码')),
    );
  }

  void _addAdminPubkey(String hex) {
    if (_admins.contains(hex)) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('该管理员已在列表中')),
      );
      return;
    }
    if (_admins.length >= 64) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('管理员数量已达上限（64）')),
      );
      return;
    }
    setState(() {
      _admins.add(hex);
      _syncThresholdInput();
    });
  }

  void _removeAdmin(int index) {
    if (_admins[index] == _creatorPubkey) return;
    setState(() {
      _admins.removeAt(index);
      _syncThresholdInput();
    });
  }

  void _syncThresholdInput() {
    if (_admins.isEmpty) {
      _thresholdController.clear();
      return;
    }
    final minThreshold = (_admins.length ~/ 2) + 1;
    final current = int.tryParse(_thresholdController.text.trim());
    if (current == null || current < minThreshold || current > _admins.length) {
      _thresholdController.text = minThreshold.toString();
    }
  }

  // ──── 提交 ────

  String? _validate() {
    if (_accounts.isEmpty || _accountAmountControllers.isEmpty) {
      return '请先查询机构账户';
    }
    final accountNames = _accountAmountControllers.keys.toList();
    if (!accountNames.contains(kReservedNameMain) ||
        !accountNames.contains(kReservedNameFee)) {
      return '机构注册账户必须包含$kReservedNameMain和$kReservedNameFee';
    }
    // 自定义账户名命中制度专属保留名（永久质押/安全基金/两和基金）即拒；
    // 主账户/费用账户为强制默认账户，不在拒绝之列。
    const protectedNames = [
      kReservedNameStake,
      kReservedNameSafetyFund,
      kReservedNameHe,
    ];
    final forbidden = accountNames
        .where((name) => protectedNames.contains(name.trim()))
        .toList();
    if (forbidden.isNotEmpty) {
      return '账户名不能使用制度专属保留名：${forbidden.join('、')}';
    }
    final blockedAccounts = _accounts
        .where((a) => a.chainStatus == 'Active' || a.chainStatus == 'Pending')
        .map((a) => a.accountName)
        .toList();
    if (blockedAccounts.isNotEmpty) {
      return '账户已在链上或正在注册中：${blockedAccounts.join('、')}';
    }
    if (_admins.length < 2) return '管理员至少 2 人';

    final thresholdText = _thresholdController.text.trim();
    final threshold = int.tryParse(thresholdText);
    if (threshold == null) return '请输入有效的阈值';

    final adminsLen = _admins.length;
    final minThreshold = (adminsLen ~/ 2) + 1;
    if (threshold < minThreshold) {
      return '阈值不能小于 $minThreshold（必须过半）';
    }
    if (threshold > adminsLen) return '阈值不能超过管理员数量';

    for (final entry in _accountAmountControllers.entries) {
      final amount = AmountFormat.tryParse(entry.value.text);
      if (amount == null || amount <= 0) {
        return '请输入 ${entry.key} 的有效金额';
      }
      if ((amount * 100).round() < 111) {
        return '${entry.key} 初始资金不能低于 1.11 元';
      }
    }

    return null;
  }

  Future<String?> _checkCreatorBalance({
    required WalletProfile wallet,
    required BigInt initialTotalFen,
  }) async {
    final balanceYuan =
        await ChainRpc().fetchFinalizedBalance(wallet.pubkeyHex);
    final balanceFen = MultisigCreateAmountRules.yuanToFen(balanceYuan);
    final requiredFen =
        MultisigCreateAmountRules.requiredBalanceFen(initialTotalFen);
    if (balanceFen >= requiredFen) return null;
    return MultisigCreateAmountRules.insufficientBalanceMessage(
      actionLabel: '创建机构多签',
      balanceYuan: balanceYuan,
      initialAmountFen: initialTotalFen,
    );
  }

  Future<void> _submit() async {
    final blockedReason = _submitBlockedReason;
    if (blockedReason != null) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(blockedReason)),
      );
      return;
    }

    final error = _validate();
    if (error != null) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(error)),
      );
      return;
    }

    setState(() => _submitting = true);

    try {
      final cidText = _cidNumberController.text.trim();
      final threshold = int.parse(_thresholdController.text.trim());
      final registrationInfo =
          await _apiClient.fetchInstitutionRegistrationInfo(cidText);
      final accounts = <InstitutionInitialAccountInput>[];
      var totalAmountFen = BigInt.zero;
      for (final accountName in registrationInfo.accountNames) {
        final controller = _accountAmountControllers[accountName];
        if (controller == null) {
          throw Exception('缺少 $accountName 初始资金输入');
        }
        final amountYuan = AmountFormat.tryParse(controller.text);
        if (amountYuan == null || amountYuan <= 0) {
          throw Exception('$accountName 初始资金无效');
        }
        final amountFen = BigInt.from((amountYuan * 100).round());
        accounts.add(
          InstitutionInitialAccountInput(
            accountName: accountName,
            amountFen: amountFen,
          ),
        );
        totalAmountFen += amountFen;
      }

      final wallet = _selectedWallet;
      final balanceError = await _checkCreatorBalance(
        wallet: wallet,
        initialTotalFen: totalAmountFen,
      );
      if (balanceError != null) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
              content: Text(balanceError), backgroundColor: AppTheme.danger),
        );
        return;
      }

      final adminsBytes =
          _admins.map((hex) => Uint8List.fromList(_hexDecode(hex))).toList();

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
          requestId: QrSigner.generateRequestId(prefix: 'create-dq-'),
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          action: QrActions.organizationCreate,
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
        return Uint8List.fromList(_hexDecode(response.body.signatureHex));
      }

      final result = await _manageService.submitProposeCreateInstitution(
        cidNumber: registrationInfo.cidNumber,
        cidFullName: registrationInfo.cidFullName,
        accounts: accounts,
        institutionCode: registrationInfo.institutionCode,
        adminsLen: _admins.length,
        admins: adminsBytes,
        threshold: threshold,
        registerNonce: registrationInfo.credential.registerNonce,
        signatureHex: registrationInfo.credential.signature,
        issuerCidNumber: registrationInfo.credential.issuerCidNumber,
        issuerMainAccountHex: registrationInfo.credential.issuerMainAccount,
        signerPubkeyHex: registrationInfo.credential.signerPubkey,
        scopeProvinceName: registrationInfo.credential.scopeProvinceName,
        scopeCityName: registrationInfo.credential.scopeCityName,
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(pubkeyBytes),
        sign: signCallback,
      );

      await WalletIsar.instance.writeTxn((isar) async {
        // 中文注释：创建交易已入块并确认事件后，直接写入当前账户的本地 pending
        // 快照；列表返回时只精准刷新该账户，不再依赖全量 discovery 扫描。
        final entity = InstitutionEntity()
          ..account = result.mainAccountHex
          ..cidNumber = registrationInfo.cidNumber
          ..adminAccountCode = registrationInfo.institutionCode
          ..accountName = accounts.isEmpty
              ? registrationInfo.cidShortName
              : accounts.first.accountName
          ..addedAtMillis = DateTime.now().millisecondsSinceEpoch
          ..discoveredViaAdmin = false
          ..matchedAdminPubkeys = const [];
        await isar.institutionEntitys.put(entity);
        await InstitutionMultisigLocalState.putStatusInTxn(
          isar,
          result.mainAccountHex,
          InstitutionMultisigLocalState.statusPending,
        );
      });

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
              '提案已确认 #${result.proposalId}：${_truncateAddress(result.txHash)}'),
          backgroundColor: AppTheme.primaryDark,
        ),
      );
      Navigator.of(context).pop(result.mainAccountHex);
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

  /// 中文注释：创建多签依赖多次链上读取与最终 extrinsic 提交，未就绪时直接拦截。
  String? get _submitBlockedReason {
    final progress = _chainProgress;
    if (progress == null) {
      return _chainProgressError ?? '正在读取区块链状态，请稍后再试';
    }
    if (!progress.hasPeers) {
      return '轻节点尚未连接到区块链网络，暂不能发起创建提案';
    }
    if (progress.isSyncing) {
      return '轻节点仍在同步区块头，完成后才能发起创建提案';
    }
    if (!progress.isUsable) {
      return _chainProgressError ?? '区块链状态尚未就绪，暂不能发起创建提案';
    }
    return null;
  }

  // ──── UI ────

  @override
  Widget build(BuildContext context) {
    debugPrint('[MultisigCreate-Diag] build START: _admins=${_admins.length} '
        '_accounts=${_accounts.length} _checkingCid=$_checkingCid');
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '创建机构多签',
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
            busy: _submitting || _checkingCid,
            onProgressChanged: _handleChainProgressChanged,
            onErrorChanged: _handleChainProgressErrorChanged,
          ),
          // CID ID 输入
          _buildSectionTitle('CID ID'),
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
                  controller: _cidNumberController,
                  decoration: InputDecoration(
                    hintText: '输入 CID 机构标识',
                    errorText: _cidError,
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                    contentPadding: const EdgeInsets.symmetric(
                        horizontal: 12, vertical: 10),
                  ),
                ),
              ),
              const SizedBox(width: 8),
              SizedBox(
                width: 84,
                height: 48,
                child: ElevatedButton(
                  onPressed: _checkingCid ? null : _checkCidRegistration,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppTheme.primaryDark,
                    foregroundColor: Colors.white,
                    padding: EdgeInsets.zero,
                  ),
                  child: _checkingCid
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
          // 机构全称/简称 + 初始账户资金
          if (_institutionCidFullName != null) ...[
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: AppTheme.primaryDark.withValues(alpha: 0.05),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                    color: AppTheme.primaryDark.withValues(alpha: 0.2)),
              ),
              child: Text(
                '机构全称：$_institutionCidFullName\n机构简称：${_institutionCidShortName ?? ''}',
                style:
                    const TextStyle(fontSize: 13, fontWeight: FontWeight.w500),
              ),
            ),
            const SizedBox(height: 12),
            _buildSectionTitle('账户初始资金（元）'),
            const SizedBox(height: 8),
            ..._accounts.map(_buildAccountAmountInput),
          ],

          const SizedBox(height: 20),

          // 管理员列表
          _buildSectionTitle('管理员列表（${_admins.length}/64）'),
          const SizedBox(height: 8),
          ..._admins.asMap().entries.map((entry) {
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
                      onPressed: () => _removeAdmin(index),
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
                  color: AppTheme.primaryDark.withValues(alpha: 0.3)),
            ),
          ),

          const SizedBox(height: 20),

          // 阈值
          _buildSectionTitle('阈值规则', note: '注册须全员同意'),
          const SizedBox(height: 8),
          TextField(
            controller: _thresholdController,
            keyboardType: TextInputType.number,
            decoration: InputDecoration(
              hintText: _admins.length >= 2
                  ? '范围：${(_admins.length + 1) ~/ 2} ~ ${_admins.length}'
                  : '请先添加管理员',
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
                if (w != null) {
                  setState(() {
                    _selectedWallet = w;
                    _syncCreatorAdmin(w);
                  });
                }
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

  Widget _buildSectionTitle(String title, {String? note}) {
    return Row(
      children: [
        Text(
          title,
          style: const TextStyle(
            fontSize: 14,
            fontWeight: FontWeight.w600,
            color: AppTheme.primaryDark,
          ),
        ),
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
  }

  Widget _buildAccountAmountInput(InstitutionAccountEntry account) {
    final controller = _accountAmountControllers[account.accountName];
    if (controller == null) return const SizedBox.shrink();
    final blocked =
        account.chainStatus == 'Active' || account.chainStatus == 'Pending';
    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: TextField(
        controller: controller,
        enabled: !blocked,
        keyboardType: const TextInputType.numberWithOptions(decimal: true),
        inputFormatters: [ThousandSeparatorFormatter()],
        decoration: InputDecoration(
          labelText: account.accountName,
          helperText: _chainStatusLabel(account.chainStatus),
          hintText: blocked ? '不可重复创建' : '最低 1.11 元',
          border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
          contentPadding:
              const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
        ),
      ),
    );
  }

  String _chainStatusLabel(String status) {
    switch (status) {
      case 'Pending':
        return '链上注册处理中';
      case 'Active':
        return '已上链';
      case 'Closed':
        return '已注销';
      case 'Failed':
        return '可重试';
      default:
        return status;
    }
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

  String _formatFenAsGmb(BigInt fen) {
    final yuan = fen ~/ BigInt.from(100);
    final remainder = (fen % BigInt.from(100)).toInt().abs();
    final intPart = yuan.toString().replaceAllMapped(
          RegExp(r'(\d)(?=(\d{3})+(?!\d))'),
          (m) => '${m[1]},',
        );
    return '$intPart.${remainder.toString().padLeft(2, '0')} GMB';
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
