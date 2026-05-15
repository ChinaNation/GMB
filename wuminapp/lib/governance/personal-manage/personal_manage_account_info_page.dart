import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:isar/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/institution_admin_service.dart';
import 'package:wuminapp_mobile/transaction/duoqian-transfer/duoqian_transfer_entry.dart';
import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/votingengine/internal-vote/internal_vote_service.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/my/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import 'personal_admin_list_page.dart';
import 'personal_duoqian_close_page.dart';
import 'personal_pending_create_lookup.dart';
import 'personal_proposal_list_section.dart';
import 'personal_manage_models.dart';
import 'personal_manage_service.dart';

/// 个人多签账户详情页。
///
/// 展示个人多签名称、地址、余额、状态、管理员列表和个人提案历史。
class PersonalManageAccountInfoPage extends StatefulWidget {
  const PersonalManageAccountInfoPage({
    super.key,
    required this.institution,
  });

  final InstitutionInfo institution;

  @override
  State<PersonalManageAccountInfoPage> createState() =>
      _PersonalManageAccountInfoPageState();
}

class _PersonalManageAccountInfoPageState
    extends State<PersonalManageAccountInfoPage> {
  final PersonalManageService _personalManageService = PersonalManageService();
  final InstitutionAdminService _adminService = InstitutionAdminService();
  final ChainRpc _rpc = ChainRpc();

  AdminSubjectIdentity get _subjectIdentity =>
      AdminSubjectIdentity.fromInstitution(widget.institution);

  bool _loading = true;
  String? _error;

  DuoqianAccountInfo? _accountInfo;
  List<String> _adminPubkeys = const [];
  bool _isClosed = false;

  /// 账户余额(元):Active 来自链上 free_balance,Pending 来自本机 Isar
  /// PersonalDuoqianProposalEntity.snapshotJson.amount_fen(发起人承诺入金)。
  double? _balanceYuan;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });

    try {
      final results = await Future.wait([
        _personalManageService.fetchPersonalAccount(
          widget.institution.duoqianAddress,
        ),
        _adminService.fetchAdmins(_subjectIdentity),
      ]);

      final accountInfo = results[0] as DuoqianAccountInfo?;
      final admins = results[1] as List<String>;
      final isClosed = accountInfo == null;
      final accountStatus = accountInfo?.status;

      // 余额取值规则：
      // Active → 链上 free_balance(实时)
      // Pending → 本机创建快照金额(链上还未到账)
      // Closed → 不显示金额,避免注销账户继续显示旧创建金额。
      if (isClosed) {
        await _markLocalStatus(PersonalDuoqianLocalState.statusClosed);
      } else {
        await _markLocalStatus(
          accountStatus == DuoqianStatus.active
              ? PersonalDuoqianLocalState.statusActive
              : PersonalDuoqianLocalState.statusPending,
        );
      }
      final balance = isClosed ? null : await _resolveBalance(accountStatus);

      if (!mounted) return;
      setState(() {
        _accountInfo = accountInfo;
        _adminPubkeys = admins;
        _isClosed = isClosed;
        _balanceYuan = balance;
        _loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = SmoldotClientManager.instance.buildUserFacingError(e);
        _loading = false;
      });
    }
  }

  Future<double?> _resolveBalance(DuoqianStatus? status) async {
    if (status == DuoqianStatus.active) {
      try {
        return await _rpc.fetchBalance(widget.institution.duoqianAddress);
      } catch (_) {
        return null;
      }
    }
    // Pending 态:从本机 Isar PersonalDuoqianProposalEntity 取
    // (该 multisig 的 create 提案 snapshot 含 amount_fen)。
    try {
      final isar = await WalletIsar.instance.db();
      final entity = await isar.personalDuoqianProposalEntitys
          .filter()
          .personalAddressEqualTo(widget.institution.duoqianAddress)
          .actionEqualTo('create')
          .findFirst();
      if (entity?.snapshotJson == null || entity!.snapshotJson!.isEmpty) {
        return null;
      }
      final snapshot = jsonDecode(entity.snapshotJson!) as Map<String, dynamic>;
      final amountFenStr = snapshot['amount_fen']?.toString();
      if (amountFenStr == null) return null;
      final fen = BigInt.tryParse(amountFenStr);
      if (fen == null) return null;
      return fen.toDouble() / 100.0;
    } catch (_) {
      return null;
    }
  }

  Future<void> _markLocalStatus(String status) async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      await PersonalDuoqianLocalState.putStatusInTxn(
        isar,
        widget.institution.duoqianAddress,
        status,
      );
    });
  }

  // ──── 关闭 ────

  void _showDeleteMenu() {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('关闭个人多签'),
        content: const Text(
          '关闭个人多签将发起链上关闭提案，需要其他管理员投票通过后才会真正关闭。\n\n确定要发起关闭吗？',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () {
              Navigator.pop(ctx);
              _openClosePage();
            },
            style: TextButton.styleFrom(foregroundColor: AppTheme.danger),
            child: const Text('发起关闭'),
          ),
        ],
      ),
    );
  }

  Future<void> _openClosePage() async {
    final wallets = await _getAdminWallets();
    if (!mounted || wallets.isEmpty) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('请先导入此账户的管理员钱包')),
        );
      }
      return;
    }

    final closed = await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => PersonalDuoqianClosePage(
          institution: widget.institution,
          adminWallets: wallets,
        ),
      ),
    );
    if (closed == true && mounted) {
      // 关闭提案已提交,但**链上 close 还没真正执行**(要等其他管理员投票通过)。
      // 此时 admins-change Subjects 仍存,反向索引下次扫还会拉回 → **不能立即删本地**。
      // 等链上 close execute 自动清掉 admins-change 后,反向索引下次扫不到再清孤立 entity。
      Navigator.pop(context);
    }
  }

  /// 是否展示右上角三点菜单；Active 显关闭，Pending 显撤销创建，Closed 显删除。
  bool _shouldShowMenu() {
    if (_isClosed) return true;
    final status = _accountInfo?.status;
    if (status == null) return false;
    if (status == DuoqianStatus.active) return true;
    return true;
  }

  Future<List<WalletProfile>> _getAdminWallets() async {
    final wm = WalletManager();
    final wallets = await wm.getWallets();
    final adminSet = _adminPubkeys.toSet();
    return wallets.where((w) {
      var pk = w.pubkeyHex.toLowerCase();
      if (pk.startsWith('0x')) pk = pk.substring(2);
      return adminSet.contains(pk);
    }).toList();
  }

  Future<void> _removeFromLocal() async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      await isar.personalDuoqianEntitys
          .where()
          .duoqianAddressEqualTo(widget.institution.duoqianAddress)
          .deleteAll();
      // 个人多签 create/transfer/close 提案 snapshot 一并清掉,否则
      // [PersonalProposalHistoryService] 下次会把它们再拉回详情页。
      await isar.personalDuoqianProposalEntitys
          .filter()
          .personalAddressEqualTo(widget.institution.duoqianAddress)
          .deleteAll();
      await PersonalDuoqianLocalState.deleteStatusInTxn(
        isar,
        widget.institution.duoqianAddress,
      );
    });
  }

  Future<void> _confirmDeleteLocal() async {
    if (!_isClosed) return;
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('删除'),
        content: const Text('确认删除该已注销个人多签账户在本机的所有数据？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, true),
            style: TextButton.styleFrom(foregroundColor: AppTheme.danger),
            child: const Text('删除'),
          ),
        ],
      ),
    );
    if (ok != true || !mounted) return;
    await _removeFromLocal();
    if (!mounted) return;
    Navigator.pop(context);
  }

  /// 撤销 Pending 阶段的个人多签创建提案(向链上发起反对投票)。
  ///
  /// 链上侧:个人多签 propose_create 的 threshold = 全员通过,任意一票反对都让
  /// `tally.yes + remaining < threshold` 立即满足,提案直接进入 STATUS_REJECTED。
  /// `cleanup_pending_personal_create` 自动执行:unreserve 创建者锁仓 + 删
  /// `PersonalManage::PersonalDuoqians` /
  /// `PendingPersonalCreate` / `admins-change::Subjects`。其他管理员设备的反向索引下次扫不到该
  /// institution_id,自动清理孤立 Isar entity。
  ///
  /// 仅个人 Pending 路径调用；Active 走 propose_close。
  /// 当前仅支持热钱包:冷钱包用户走"管理员列表" → 投反对票完成同样语义。
  Future<void> _confirmRevokeCreate() async {
    if (_accountInfo?.status == DuoqianStatus.active) return;

    final adminWallets = await _getAdminWallets();
    if (!mounted) return;
    if (adminWallets.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请先导入此多签的管理员钱包')),
      );
      return;
    }
    final hot = adminWallets.firstWhere(
      (w) => w.isHotWallet,
      orElse: () => adminWallets.first,
    );
    if (!hot.isHotWallet) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('当前管理员钱包均为冷钱包,请到"管理员列表"扫码投反对票')),
      );
      return;
    }

    final pid = await PersonalPendingCreateLookup()
        .findActiveCreate(widget.institution.duoqianAddress);
    if (!mounted) return;
    if (pid == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('未找到活跃的创建提案,可能已被处理')),
      );
      return;
    }

    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('撤销创建'),
        content: const Text(
          '将向链上发起反对投票。提案被否决后,链上自动清理该多签,'
          '所有管理员设备上的本地记录会随之消失。\n\n'
          '创建者锁定的资金将原路返还。',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, true),
            style: TextButton.styleFrom(foregroundColor: AppTheme.danger),
            child: const Text('撤销'),
          ),
        ],
      ),
    );
    if (ok != true || !mounted) return;

    setState(() => _loading = true);
    try {
      final wm = WalletManager();
      await wm.authenticateForSigning();
      final pubkeyBytes = _hexDecode(hot.pubkeyHex);
      await InternalVoteService().submit(
        proposalId: pid,
        approve: false,
        fromAddress: hot.address,
        signerPubkey: Uint8List.fromList(pubkeyBytes),
        sign: (payload) => wm.signWithWalletNoAuth(hot.walletIndex, payload),
      );
      // 链上 reject 触发 cleanup 是异步的(下个出块周期),但 admins-change
      // 一旦清空,反向索引就扫不到 → 兜底机制完整。本地立即清,避免用户再看到。
      await _removeFromLocal();
      if (!mounted) return;
      Navigator.pop(context);
    } catch (e) {
      if (!mounted) return;
      setState(() => _loading = false);
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('撤销失败:$e')),
      );
    }
  }

  // ──── UI ────

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '个人多签账户',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: AppTheme.primaryDark,
        elevation: 0,
        scrolledUnderElevation: 0.5,
        // 个人多签菜单:
        // - Active  → 关闭个人多签，走 PersonalManage::propose_close。
        // - Pending → 撤销创建，走 InternalVote approve=false 早期否决。
        actions: [
          if (_shouldShowMenu())
            PopupMenuButton<String>(
              icon: const Icon(Icons.more_vert),
              onSelected: (value) {
                if (value == 'delete') _confirmDeleteLocal();
                if (value == 'close') _showDeleteMenu();
                if (value == 'revoke_create') _confirmRevokeCreate();
              },
              itemBuilder: (_) {
                final isActive = _accountInfo?.status == DuoqianStatus.active;
                return [
                  if (_isClosed)
                    const PopupMenuItem(
                      value: 'delete',
                      child: Row(
                        children: [
                          Icon(Icons.delete_outline,
                              size: 20, color: AppTheme.danger),
                          SizedBox(width: 8),
                          Text(
                            '删除',
                            style: TextStyle(color: AppTheme.danger),
                          ),
                        ],
                      ),
                    )
                  else if (isActive)
                    const PopupMenuItem(
                      value: 'close',
                      child: Row(
                        children: [
                          Icon(Icons.delete_outline,
                              size: 20, color: AppTheme.danger),
                          SizedBox(width: 8),
                          Text(
                            '关闭个人多签',
                            style: TextStyle(color: AppTheme.danger),
                          ),
                        ],
                      ),
                    )
                  else
                    const PopupMenuItem(
                      value: 'revoke_create',
                      child: Row(
                        children: [
                          Icon(Icons.cancel_outlined,
                              size: 20, color: AppTheme.danger),
                          SizedBox(width: 8),
                          Text('撤销创建',
                              style: TextStyle(color: AppTheme.danger)),
                        ],
                      ),
                    ),
                ];
              },
            ),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : _error != null
              ? _buildError()
              : _buildContent(),
    );
  }

  Widget _buildError() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.error_outline, size: 48, color: AppTheme.danger),
            const SizedBox(height: 12),
            const Text('加载失败',
                style: TextStyle(fontSize: 16, color: AppTheme.textSecondary)),
            const SizedBox(height: 6),
            Text(
              _error!,
              style:
                  const TextStyle(fontSize: 12, color: AppTheme.textTertiary),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 16),
            OutlinedButton(onPressed: _load, child: const Text('重试')),
          ],
        ),
      ),
    );
  }

  Widget _buildContent() {
    final duoqianSs58 = _hexToSs58(widget.institution.duoqianAddress);
    final info = _accountInfo;
    final statusLabel = _isClosed
        ? '已注销'
        : info == null
            ? '已注销'
            : info.status == DuoqianStatus.active
                ? '已激活'
                : '待激活';
    final statusColor = _isClosed
        ? AppTheme.textTertiary
        : info?.status == DuoqianStatus.active
            ? AppTheme.success
            : AppTheme.warning;

    return RefreshIndicator(
      onRefresh: () async {
        _adminService.clearCache(_subjectIdentity);
        await _load();
      },
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          // 基本信息卡片
          Card(
            elevation: 0,
            margin: EdgeInsets.zero,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
              side: const BorderSide(color: AppTheme.border),
            ),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    '账户信息',
                    style: TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.w700,
                      color: AppTheme.primaryDark,
                    ),
                  ),
                  const SizedBox(height: 12),
                  _buildInfoRow('名称', widget.institution.name),
                  const Divider(height: 20),
                  _buildInfoRow(
                    '多签地址',
                    duoqianSs58,
                    onCopy: () {
                      Clipboard.setData(ClipboardData(text: duoqianSs58));
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(
                          content: Text('地址已复制'),
                          duration: Duration(seconds: 1),
                        ),
                      );
                    },
                  ),
                  if (!_isClosed) ...[
                    // 账户余额：Active 显示链上 free_balance，Pending 显示
                    // 发起人承诺金额；注销账户不再显示旧金额。
                    const Divider(height: 20),
                    _buildBalanceRow(info?.status),
                  ],
                  const Divider(height: 20),
                  _buildInfoRow('状态', statusLabel, valueColor: statusColor),
                  // 管理员数量 / 通过阈值 已删除(bug 4):管理员列表卡片
                  // subtitle 已显示这两项信息,避免重复。
                ],
              ),
            ),
          ),

          if (!_isClosed) ...[
            const SizedBox(height: 16),
            DuoqianTransferEntryCard(
              institution: widget.institution,
              isPersonal: true,
              enabled: _accountInfo?.status == DuoqianStatus.active,
              loadAdminWallets: _getAdminWallets,
              onCreated: _load,
            ),
            const SizedBox(height: 16),
          ] else
            const SizedBox(height: 16),

          // 管理员列表(折叠成单行,点击进入子页)
          _buildAdminEntryCard(info),

          // 个人多签提案列表(req 5):活跃 + 历史(本机 Isar 永久保留终态记录)
          const SizedBox(height: 16),
          FutureBuilder<List<WalletProfile>>(
            future: _getAdminWallets(),
            builder: (context, snapshot) {
              final wallets = snapshot.data ?? const <WalletProfile>[];
              return PersonalProposalListSection(
                institution: widget.institution,
                adminWallets: wallets,
              );
            },
          ),
        ],
      ),
    );
  }

  /// 管理员列表入口卡片(req 1):点击进入完整管理员列表页。
  Widget _buildAdminEntryCard(DuoqianAccountInfo? info) {
    final adminCount = _adminPubkeys.length;
    final threshold = info?.threshold;
    final subtitle = _isClosed
        ? '已注销'
        : threshold == null
            ? '$adminCount 人'
            : '$adminCount 人 · 阈值 $threshold/$adminCount';

    // bug 2(2026-05-03):卡片高度对齐 institution_detail_page._buildAdminEntry,
    // 用 InkWell + Padding(14,12) + Row(36×36 icon)替代 ListTile 减少视觉高度。
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: const BorderSide(color: AppTheme.border),
      ),
      child: InkWell(
        onTap: () => _openAdminListPage(info),
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
          child: Row(
            children: [
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: AppTheme.primaryDark.withValues(alpha: 0.08),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: const Icon(Icons.group_outlined,
                    size: 18, color: AppTheme.primaryDark),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      '管理员列表',
                      style: TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.primaryDark,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      subtitle,
                      style: const TextStyle(
                          fontSize: 12, color: AppTheme.textTertiary),
                    ),
                  ],
                ),
              ),
              const Icon(Icons.chevron_right,
                  size: 20, color: AppTheme.textTertiary),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _openAdminListPage(DuoqianAccountInfo? info) async {
    final wallets = await _getAdminWallets();
    if (!mounted) return;
    final creator = await _resolvePersonalCreatorPubkeyHex();
    if (!mounted) return;
    await Navigator.push(
      context,
      MaterialPageRoute(
        builder: (_) => PersonalAdminListPage(
          institution: widget.institution,
          duoqianStatus: info?.status ?? DuoqianStatus.pending,
          adminPubkeys: _adminPubkeys,
          adminWallets: wallets,
          creatorPubkeyHex: creator,
        ),
      ),
    );
    // 子页可能完成投票 → 刷新本页状态(可能多签已激活)
    if (mounted) await _load();
  }

  /// 从本机 Isar 读取个人多签创建者公钥 hex。
  /// req 3 未实现时,只有创建者本机有此记录;非创建者打开子页 creatorPubkeyHex 为 null
  /// (届时所有 admin 都按"非创建者"渲染,语义略损但不阻塞主流程)。
  Future<String?> _resolvePersonalCreatorPubkeyHex() async {
    try {
      final isar = await WalletIsar.instance.db();
      final entity = await isar.personalDuoqianEntitys
          .filter()
          .duoqianAddressEqualTo(widget.institution.duoqianAddress)
          .findFirst();
      if (entity == null) return null;
      // creatorAddress 是 SS58,转 pubkey hex(小写,无 0x)。
      final pair = Keyring().decodeAddress(entity.creatorAddress);
      return pair
          .map((b) => b.toRadixString(16).padLeft(2, '0'))
          .join()
          .toLowerCase();
    } catch (_) {
      return null;
    }
  }

  /// 账户余额行(bug 4):
  /// - Active:链上 free_balance 实时(无标签)
  /// - Pending:发起人承诺金额(snapshot.amount_fen)+ "不可用" 灰色标签
  Widget _buildBalanceRow(DuoqianStatus? status) {
    final balanceStr =
        _balanceYuan == null ? '—' : AmountFormat.format(_balanceYuan!);
    final isPending = status != DuoqianStatus.active;
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const SizedBox(
          width: 80,
          child: Text(
            '账户余额',
            style: TextStyle(fontSize: 13, color: AppTheme.textSecondary),
          ),
        ),
        Expanded(
          child: Wrap(
            spacing: 8,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              Text(
                balanceStr,
                style: const TextStyle(
                  fontSize: 13,
                  color: AppTheme.textPrimary,
                  fontWeight: FontWeight.w600,
                ),
              ),
              if (isPending && _balanceYuan != null)
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: AppTheme.textTertiary.withValues(alpha: 0.1),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: const Text(
                    '不可用',
                    style: TextStyle(
                      fontSize: 11,
                      color: AppTheme.textTertiary,
                    ),
                  ),
                ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildInfoRow(String label, String value,
      {VoidCallback? onCopy, Color? valueColor}) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 80,
          child: Text(
            label,
            style: const TextStyle(fontSize: 13, color: AppTheme.textSecondary),
          ),
        ),
        Expanded(
          child: Text(
            value,
            style: TextStyle(
              fontSize: 13,
              color: valueColor ?? AppTheme.textPrimary,
              fontWeight: valueColor != null ? FontWeight.w600 : null,
            ),
          ),
        ),
        if (onCopy != null)
          GestureDetector(
            onTap: onCopy,
            child:
                const Icon(Icons.copy, size: 16, color: AppTheme.textTertiary),
          ),
      ],
    );
  }

  // ──── 工具 ────

  String _hexToSs58(String hex) {
    final bytes = _hexDecode(hex);
    return Keyring().encodeAddress(Uint8List.fromList(bytes), 2027);
  }

  Uint8List _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = Uint8List(h.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }
}
