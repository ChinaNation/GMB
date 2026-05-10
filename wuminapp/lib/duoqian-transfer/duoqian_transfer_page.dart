import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:smoldot/smoldot.dart' show LightClientStatusSnapshot;
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/ui/widgets/chain_progress_banner.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:isar/isar.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/personal-manage/personal_proposal_history_service.dart';
import 'package:wuminapp_mobile/util/amount_format.dart';
import 'package:wuminapp_mobile/common/institution_info.dart';
import 'package:wuminapp_mobile/duoqian-transfer/duoqian_transfer_balance_guard.dart';
import 'package:wuminapp_mobile/duoqian-transfer/duoqian_transfer_service.dart';
import 'package:wuminapp_mobile/qr/pages/qr_scan_page.dart';
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/rpc/onchain.dart' show OnchainRpc;
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 机构转账提案创建页面。
class DuoqianTransferPage extends StatefulWidget {
  const DuoqianTransferPage({
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
  State<DuoqianTransferPage> createState() => _DuoqianTransferPageState();
}

class _DuoqianTransferPageState extends State<DuoqianTransferPage> {
  final _beneficiaryController = TextEditingController();
  final _amountController = TextEditingController();
  final _remarkController = TextEditingController();

  bool _loadingBalance = true;
  bool _submitting = false;
  double? _availableBalance;
  double _estimatedFee = 0.0;
  String? _addressError;
  String? _amountError;
  LightClientStatusSnapshot? _chainProgress;
  String? _chainProgressError;

  late final String _fromSs58;
  late WalletProfile _selectedWallet;

  @override
  void initState() {
    super.initState();
    _selectedWallet = widget.adminWallets.first;
    _fromSs58 = _accountHexToSs58(widget.institution.mainAddress);
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

  String _accountHexToSs58(String hex) {
    final bytes = _hexToBytes(hex);
    return Keyring().encodeAddress(Uint8List.fromList(bytes), 2027);
  }

  Future<void> _fetchBalance() async {
    try {
      final service = DuoqianTransferService();
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
    final amount = AmountFormat.tryParse(_amountController.text);
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
        Uint8List.fromList(_hexToBytes(widget.institution.mainAddress));
    if (_bytesEqual(beneficiaryBytes, institutionBytes)) {
      setState(() => _addressError = '收款地址不能与机构地址相同');
      return false;
    }
    setState(() => _addressError = null);
    return true;
  }

  bool _validateAmount() {
    final amount = AmountFormat.tryParse(_amountController.text);
    if (amount == null || amount < 1.11) {
      setState(() => _amountError = '最低转账金额为 1.11 元（存在性保证金）');
      return false;
    }
    if (_availableBalance != null) {
      final fee = OnchainRpc.estimateTransferFeeYuan(amount);
      const ed = 1.11;
      if (amount + fee + ed > _availableBalance!) {
        setState(() => _amountError =
            '余额不足（需保留 ${AmountFormat.format(ed, symbol: '')} 元 ED + ${AmountFormat.format(fee, symbol: '')} 元手续费）');
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
    final blockedReason = _submitBlockedReason;
    if (blockedReason != null) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(blockedReason)),
      );
      return;
    }

    if (!_validateAddress() || !_validateAmount() || !_validateRemark()) {
      return;
    }

    final wallet = _selectedWallet;
    final amountYuan = AmountFormat.tryParse(_amountController.text) ?? 0;
    final requiredAdminFee = OnchainRpc.estimateTransferFeeYuan(amountYuan);
    final balanceBlockedReason =
        await DuoqianTransferBalanceGuard.checkAdminWalletBalance(
      wallet: wallet,
      requiredFeeYuan: requiredAdminFee,
      actionLabel: '发起多签转账提案',
    );
    if (balanceBlockedReason != null) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(balanceBlockedReason)),
      );
      return;
    }

    setState(() => _submitting = true);

    try {
      // 多签管理员的转账提案签名(2026-05-03 整改):
      // 多签管理员(个人 + 机构)支持冷热钱包双路径,与 personal_duoqian_create_page 对齐;
      // 治理机构(NRC/PRC/PRB)和区块链软件端管理员才只支持冷钱包(QR)。
      // 这里多签提案 → 热钱包优先 → 冷钱包 fallback 走 QR。
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
        final beneficiary = _beneficiaryController.text.trim();
        final institutionLabel = _coldWalletInstitutionLabel();
        // 千分位格式化，与 PayloadDecoder._fenToYuan 对齐
        final amountFormatted = AmountFormat.format(
                AmountFormat.tryParse(_amountController.text) ?? 0,
                symbol: '')
            .trim();
        final remarkText = _remarkController.text;
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'propose-'),
          address: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          display: SignDisplay(
            action: 'propose_transfer',
            summary:
                '$institutionLabel 提案转账 $amountFormatted GMB 给 $beneficiary',
            fields: [
              // propose_transfer 冷钱包按账户级 SubjectId 展示真实支出主体;
              // org 只是链端路由,不进入 display.fields,避免个人多签被显示成机构。
              SignDisplayField(
                  key: 'institution', label: '转出账户', value: institutionLabel),
              SignDisplayField(
                  key: 'beneficiary', label: '收款账户', value: beneficiary),
              SignDisplayField(
                  key: 'amount_yuan',
                  label: '金额',
                  value: '$amountFormatted GMB'),
              SignDisplayField(key: 'remark', label: '备注', value: remarkText),
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
        return Uint8List.fromList(_hexToBytes(response.body.signature));
      }

      final signerPubkey = Uint8List.fromList(_hexToBytes(wallet.pubkeyHex));

      final service = DuoqianTransferService();
      // 提前查链上 NextProposalId,作为本次转账提案的预测 ID;
      // 若该多签是个人多签,后续写入 Isar 历史(req 5)。
      final predictedProposalId = await service.fetchNextProposalId();

      await service.submitProposeTransfer(
        institution: widget.institution,
        beneficiaryAddress: _beneficiaryController.text.trim(),
        amountYuan: amountYuan,
        remark: _remarkController.text,
        fromAddress: wallet.address,
        signerPubkey: signerPubkey,
        sign: signCallback,
      );

      // 若是个人多签,写入提案历史 entity(转账提案在多签详情页提案列表展示)。
      // 本页 institution.orgType 个人/机构都是 OrgType.duoqian,通过 Isar 查
      // PersonalDuoqianEntity 命中即视作个人多签。
      await _maybeRecordPersonalProposal(
        proposalId: predictedProposalId,
        beneficiary: _beneficiaryController.text.trim(),
        amountYuan: amountYuan,
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

  String _coldWalletInstitutionLabel() {
    final identity = widget.institution.sfidNumber;
    final registered = registeredDuoqianAddressFromIdentity(identity);
    if (registered != null) {
      return '机构账户 ${_shortHex(registered)}';
    }
    final personal = personalDuoqianAddressFromIdentity(identity);
    if (personal != null) {
      return '个人多签 ${_shortHex(personal)}';
    }
    return widget.institution.name;
  }

  String _shortHex(String hex) {
    final value = hex.startsWith('0x') ? hex.substring(2) : hex;
    if (value.length <= 14) return value;
    return '${value.substring(0, 8)}...${value.substring(value.length - 6)}';
  }

  /// 仅当 [widget.institution] 是个人多签时,把转账提案写入 Isar 历史
  /// (`PersonalDuoqianProposalEntity`),让详情页"提案列表"区域立即看到。
  /// 机构多签的提案历史由其他模块负责,这里 silent skip。
  Future<void> _maybeRecordPersonalProposal({
    required int proposalId,
    required String beneficiary,
    required double amountYuan,
  }) async {
    try {
      final isar = await WalletIsar.instance.db();
      final personal = await isar.personalDuoqianEntitys
          .filter()
          .duoqianAddressEqualTo(widget.institution.mainAddress)
          .findFirst();
      if (personal == null) return; // 非个人多签,跳过

      await PersonalProposalHistoryService().recordOrUpdate(
        personalAddressHex: widget.institution.mainAddress,
        proposalId: proposalId,
        action: PersonalProposalAction.transfer,
        status: PersonalProposalStatus.voting,
        yesVotes: 0,
        noVotes: 0,
        snapshot: {
          'beneficiary': beneficiary,
          'amount_yuan': amountYuan,
        },
      );
    } catch (_) {
      // 写入失败不阻断主流程(链端已成功)
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

  /// 中文注释：提案页允许用户先填写表单，但链未连上或仍在同步时禁止真正提交。
  String? get _submitBlockedReason {
    final progress = _chainProgress;
    if (progress == null) {
      return _chainProgressError ?? '正在读取区块链状态，请稍后再试';
    }
    if (!progress.hasPeers) {
      return '轻节点尚未连接到区块链网络，暂不能提交转账提案';
    }
    if (progress.isSyncing) {
      return '轻节点仍在同步区块头，完成后才能提交转账提案';
    }
    if (!progress.isUsable) {
      return _chainProgressError ?? '区块链状态尚未就绪，暂不能提交转账提案';
    }
    return null;
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
        foregroundColor: AppTheme.primaryDark,
        elevation: 0,
        scrolledUnderElevation: 0.5,
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          ChainProgressBanner(
            busy: _submitting || _loadingBalance,
            onProgressChanged: _handleChainProgressChanged,
            onErrorChanged: _handleChainProgressErrorChanged,
          ),
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
              hintStyle:
                  const TextStyle(color: AppTheme.textTertiary, fontSize: 14),
              filled: true,
              fillColor: AppTheme.surfaceMuted,
              enabledBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.border),
              ),
              focusedBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.primaryDark),
              ),
              errorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.danger),
              ),
              focusedErrorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.danger),
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
            inputFormatters: [ThousandSeparatorFormatter()],
            decoration: InputDecoration(
              hintText: '最低 1.11 元',
              hintStyle:
                  const TextStyle(color: AppTheme.textTertiary, fontSize: 14),
              filled: true,
              fillColor: AppTheme.surfaceMuted,
              enabledBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.border),
              ),
              focusedBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.primaryDark),
              ),
              errorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.danger),
              ),
              focusedErrorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.danger),
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
            _estimatedFee > 0
                ? '${AmountFormat.format(_estimatedFee, symbol: '')} 元'
                : '--',
          ),
          const SizedBox(height: 8),

          // ──── 可用余额 ────
          _buildInfoRow(
            '可用余额',
            _loadingBalance
                ? '查询中...'
                : _availableBalance != null
                    ? '${AmountFormat.format(_availableBalance!, symbol: '')} 元'
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
              hintStyle:
                  const TextStyle(color: AppTheme.textTertiary, fontSize: 14),
              filled: true,
              fillColor: AppTheme.surfaceMuted,
              enabledBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.border),
              ),
              focusedBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.primaryDark),
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
                backgroundColor: AppTheme.primaryDark,
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(10),
                ),
              ),
              onPressed: _canSubmit ? _submit : null,
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
          color: AppTheme.success.withValues(alpha: 0.06),
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: AppTheme.success.withValues(alpha: 0.2)),
        ),
        child: Row(
          children: [
            const Icon(Icons.verified_user, size: 16, color: AppTheme.success),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                _truncateAddress(wallets.first.address),
                style: const TextStyle(
                  fontSize: 13,
                  fontFamily: 'monospace',
                  color: AppTheme.success,
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
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: AppTheme.success.withValues(alpha: 0.3)),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<int>(
          value: _selectedWallet.walletIndex,
          isExpanded: true,
          icon: const Icon(Icons.arrow_drop_down, color: AppTheme.primaryDark),
          items: wallets.map((w) {
            return DropdownMenuItem<int>(
              value: w.walletIndex,
              child: Row(
                children: [
                  const Icon(Icons.verified_user,
                      size: 14, color: AppTheme.success),
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
              color: AppTheme.primaryDark,
            ),
          ),
        ),
        // 原"注册多签机构"badge 已删除(2026-05-03):
        // 个人多签和机构多签 orgType 都是 OrgType.duoqian → label 输出"注册多签机构",
        // 但用户进入页面是个人多签时显示这个标签具有误导性。
        // 直接不显示标签,只显示多签账户名(已足够标识)。
      ],
    );
  }

  Widget _buildLabel(String text) {
    return Text(
      text,
      style: const TextStyle(
        fontSize: 13,
        fontWeight: FontWeight.w600,
        color: AppTheme.primaryDark,
      ),
    );
  }

  Widget _buildReadOnlyField(String value) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 14),
      decoration: BoxDecoration(
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: AppTheme.border),
      ),
      child: SelectableText(
        value,
        style: const TextStyle(
          fontSize: 13,
          color: AppTheme.textSecondary,
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
          style: const TextStyle(fontSize: 13, color: AppTheme.textSecondary),
        ),
        Text(
          value,
          style: const TextStyle(
            fontSize: 13,
            fontWeight: FontWeight.w600,
            color: AppTheme.primaryDark,
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
