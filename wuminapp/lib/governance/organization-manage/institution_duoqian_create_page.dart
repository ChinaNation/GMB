import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:smoldot/smoldot.dart' show LightClientStatusSnapshot;
import 'package:wuminapp_mobile/qr/envelope.dart';
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/common/institution_info.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/qr/pages/qr_scan_page.dart'
    show QrScanMode, QrScanPage;
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/ui/widgets/chain_progress_banner.dart';
import 'package:wuminapp_mobile/my/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/capabilities/api_client.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import 'duoqian_manage_service.dart';

/// 创建机构多签账户提案页面。
///
/// 用户输入 SFID ID 查询注册状态，填写管理员列表、阈值、初始资金后发起提案。
class InstitutionDuoqianCreatePage extends StatefulWidget {
  const InstitutionDuoqianCreatePage({
    super.key,
    required this.institution,
    required this.adminWallets,
  });

  final InstitutionInfo institution;
  final List<WalletProfile> adminWallets;

  @override
  State<InstitutionDuoqianCreatePage> createState() =>
      _InstitutionDuoqianCreatePageState();
}

const int _defaultInstitutionAdminOrg = 5;

class _InstitutionDuoqianCreatePageState
    extends State<InstitutionDuoqianCreatePage> {
  final _sfidNumberController = TextEditingController();
  final _thresholdController = TextEditingController();
  final Map<String, TextEditingController> _accountAmountControllers = {};

  final _manageService = DuoqianManageService();
  final _apiClient = ApiClient();

  bool _submitting = false;
  String? _sfidError;
  bool _checkingSfid = false;
  LightClientStatusSnapshot? _chainProgress;
  String? _chainProgressError;

  // ── 机构账户列表（从 SFID 后端查询） ──
  String? _institutionName; // 查到的机构名称
  List<InstitutionAccountEntry> _accounts = []; // 机构下所有账户

  // 管理员列表（公钥 hex，不含 0x）
  final List<String> _adminPubkeys = [];
  String? _creatorPubkey; // 创建人公钥（始终占管理员列表第一位，不可移除）

  late WalletProfile _selectedWallet;

  @override
  void initState() {
    super.initState();
    debugPrint(
        '[DuoqianCreate-Diag] initState: adminWallets.length=${widget.adminWallets.length}');
    if (widget.adminWallets.isNotEmpty) {
      final w = widget.adminWallets.first;
      debugPrint('[DuoqianCreate-Diag] first wallet: name=${w.walletName} '
          'pubkeyHex.len=${w.pubkeyHex.length} address.len=${w.address.length} '
          'signMode=${w.signMode}');
    }
    _selectedWallet = widget.adminWallets.first;
    _syncCreatorAdmin(widget.adminWallets.first);
    debugPrint(
        '[DuoqianCreate-Diag] after sync: _adminPubkeys=${_adminPubkeys.length} '
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
    _sfidNumberController.dispose();
    _thresholdController.dispose();
    _disposeAccountAmountControllers();
    super.dispose();
  }

  // ──── SFID 查询（通过 SFID 后端 API） ────

  /// 输入 SFID ID 后点击"查询"：从 SFID 后端获取机构名称 + 账户列表。
  Future<void> _checkSfidRegistration() async {
    final sfidText = _sfidNumberController.text.trim();
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
      _institutionName = null;
      _accounts = [];
    });
    _disposeAccountAmountControllers();

    try {
      final resp = await _apiClient.fetchInstitutionAccounts(sfidText);
      if (!mounted) return;

      if (resp.accounts.isEmpty) {
        setState(() {
          _sfidError = '该机构尚未创建任何账户';
          _checkingSfid = false;
        });
      } else {
        _replaceAccountAmountControllers(resp.accounts);
        setState(() {
          _institutionName = resp.institutionName;
          _accounts = resp.accounts;
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
    if (_accounts.isEmpty || _accountAmountControllers.isEmpty) {
      return '请先查询机构账户';
    }
    final accountNames = _accountAmountControllers.keys.toList();
    if (!accountNames.contains('主账户') || !accountNames.contains('费用账户')) {
      return '机构注册账户必须包含主账户和费用账户';
    }
    final blockedAccounts = _accounts
        .where(
            (a) => a.chainStatus == 'REGISTERED' || a.chainStatus == 'PENDING')
        .map((a) => a.accountName)
        .toList();
    if (blockedAccounts.isNotEmpty) {
      return '账户已在链上或正在注册中：${blockedAccounts.join('、')}';
    }
    if (_adminPubkeys.length < 2) return '管理员至少 2 人';

    final thresholdText = _thresholdController.text.trim();
    final threshold = int.tryParse(thresholdText);
    if (threshold == null) return '请输入有效的阈值';

    final adminCount = _adminPubkeys.length;
    final minThreshold = (adminCount ~/ 2) + 1;
    if (threshold < minThreshold) {
      return '阈值不能小于 $minThreshold（必须过半）';
    }
    if (threshold > adminCount) return '阈值不能超过管理员数量';

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
      final sfidText = _sfidNumberController.text.trim();
      final threshold = int.parse(_thresholdController.text.trim());
      final registrationInfo =
          await _apiClient.fetchInstitutionRegistrationInfo(sfidText);
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
          return await hotWalletManager.signWithWalletNoAuth(
              wallet.walletIndex, payload);
        }
        // 冷钱包 QR 签名
        final qrSigner = QrSigner();
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'create-dq-'),
          address: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          display: SignDisplay(
            action: 'propose_create_institution',
            summary: '发起创建机构多签账户提案',
            fields: [
              SignDisplayField(
                  key: 'sfid_number',
                  label: 'SFID ID',
                  value: registrationInfo.sfidNumber),
              SignDisplayField(
                  key: 'institution_name',
                  label: '机构名称',
                  value: registrationInfo.institutionName),
              const SignDisplayField(
                  key: 'org', label: '管理员组织类型', value: '其他机构账户'),
              SignDisplayField(
                  key: 'admin_count',
                  label: '管理员数量',
                  value: _adminPubkeys.length.toString()),
              SignDisplayField(
                  key: 'threshold',
                  label: '阈值',
                  value: '$threshold/${_adminPubkeys.length}'),
              SignDisplayField(
                  key: 'total_amount_yuan',
                  label: '初始资金合计',
                  value: _formatFenAsGmb(totalAmountFen)),
              ...accounts.map(
                (account) => SignDisplayField(
                  key: 'amount_${account.accountName}',
                  label: '${account.accountName} 初始资金',
                  value: _formatFenAsGmb(account.amountFen),
                ),
              ),
              SignDisplayField(
                  key: 'province',
                  label: '签发省份',
                  value: registrationInfo.credential.province),
              SignDisplayField(
                  key: 'signer_admin_pubkey',
                  label: '签发管理员',
                  value: registrationInfo.credential.signerAdminPubkey),
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

      final result = await _manageService.submitProposeCreateInstitution(
        sfidNumber: registrationInfo.sfidNumber,
        institutionName: registrationInfo.institutionName,
        accounts: accounts,
        adminOrg: _defaultInstitutionAdminOrg,
        adminCount: _adminPubkeys.length,
        adminPubkeys: adminPubkeyBytes,
        threshold: threshold,
        registerNonce: registrationInfo.credential.registerNonce,
        signatureHex: registrationInfo.credential.signature,
        province: registrationInfo.credential.province,
        signerAdminPubkeyHex: registrationInfo.credential.signerAdminPubkey,
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
    debugPrint(
        '[DuoqianCreate-Diag] build START: _adminPubkeys=${_adminPubkeys.length} '
        '_accounts=${_accounts.length} _checkingSfid=$_checkingSfid');
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
            busy: _submitting || _checkingSfid,
            onProgressChanged: _handleChainProgressChanged,
            onErrorChanged: _handleChainProgressErrorChanged,
          ),
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
                  controller: _sfidNumberController,
                  decoration: InputDecoration(
                    hintText: '输入 SFID 机构标识',
                    errorText: _sfidError,
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
          // 机构信息 + 初始账户资金
          if (_institutionName != null) ...[
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
                '机构名称：$_institutionName',
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

  Widget _buildAccountAmountInput(InstitutionAccountEntry account) {
    final controller = _accountAmountControllers[account.accountName];
    if (controller == null) return const SizedBox.shrink();
    final blocked =
        account.chainStatus == 'REGISTERED' || account.chainStatus == 'PENDING';
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
      case 'INACTIVE':
        return '未上链';
      case 'PENDING':
        return '链上注册处理中';
      case 'REGISTERED':
        return '已上链';
      case 'FAILED':
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
