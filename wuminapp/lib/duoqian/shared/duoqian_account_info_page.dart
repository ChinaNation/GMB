import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:isar/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/admins_change/services/institution_admin_service.dart';
import 'package:wuminapp_mobile/institution/institution_data.dart';
import 'package:wuminapp_mobile/proposal/shared/internal_vote_service.dart';
import 'package:wuminapp_mobile/proposal/transfer/transfer_proposal_page.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import '../institution/institution_duoqian_close_page.dart';
import '../personal/personal_admin_list_page.dart';
import '../personal/personal_duoqian_close_page.dart';
import '../personal/personal_pending_create_lookup.dart';
import '../personal/personal_proposal_list_section.dart';
import 'duoqian_manage_models.dart';
import 'duoqian_manage_service.dart';

/// 多签机构详情页。
///
/// 展示机构名称、SFID ID、多签地址、状态、管理员列表。
/// 右上角 "..." 提供关闭操作。
class DuoqianAccountInfoPage extends StatefulWidget {
  const DuoqianAccountInfoPage({
    super.key,
    required this.institution,
    this.isPersonal = false,
  });

  final InstitutionInfo institution;

  /// 是否为个人多签（不显示 SFID ID 行）。
  final bool isPersonal;

  @override
  State<DuoqianAccountInfoPage> createState() => _DuoqianAccountInfoPageState();
}

class _DuoqianAccountInfoPageState extends State<DuoqianAccountInfoPage> {
  final DuoqianManageService _manageService = DuoqianManageService();
  final InstitutionAdminService _adminService = InstitutionAdminService();
  final ChainRpc _rpc = ChainRpc();

  bool _loading = true;
  String? _error;

  DuoqianAccountInfo? _accountInfo;
  List<String> _adminPubkeys = const [];

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
        _manageService.fetchDuoqianAccount(widget.institution.duoqianAddress),
        _adminService.fetchAdmins(widget.institution.sfidNumber),
      ]);

      final accountInfo = results[0] as DuoqianAccountInfo?;
      final admins = results[1] as List<String>;

      // 余额取值规则(bug 4):
      // Active → 链上 free_balance(实时)
      // Pending / null → 本机 Isar PersonalDuoqianProposalEntity (action='create')
      //                   snapshot.amount_fen(发起人承诺金额,链上还未到账)
      final balance = await _resolveBalance(accountInfo?.status);

      if (!mounted) return;
      setState(() {
        _accountInfo = accountInfo;
        _adminPubkeys = admins;
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
    // Pending / null 态:从本机 Isar PersonalDuoqianProposalEntity 取
    // (该 multisig 的 create 提案 snapshot 含 amount_fen)。
    if (!widget.isPersonal) return null;
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

  // ──── 关闭 ────

  void _showDeleteMenu() {
    final title = widget.isPersonal ? '关闭个人多签' : '关闭机构多签';
    final content = widget.isPersonal
        ? '关闭个人多签将发起链上关闭提案，需要其他管理员投票通过后才会真正关闭。\n\n确定要发起关闭吗？'
        : '关闭机构多签将发起链上关闭提案，需要其他管理员投票通过后才会真正关闭。\n\n确定要发起关闭吗？';
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(title),
        content: Text(content),
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
          SnackBar(
            content:
                Text(widget.isPersonal ? '请先导入此账户的管理员钱包' : '请先导入此机构的管理员钱包'),
          ),
        );
      }
      return;
    }

    final closed = await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => widget.isPersonal
            ? PersonalDuoqianClosePage(
                institution: widget.institution,
                adminWallets: wallets,
              )
            : InstitutionDuoqianClosePage(
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

  /// 是否展示右上角三点菜单。
  ///
  /// - 个人多签:Active 显"关闭",Pending 显"撤销创建" → 都展示
  /// - 机构多签:仅 Active 显"关闭"; Pending 不展示(SFID 治理流程负责)
  /// - 状态未知(`_accountInfo == null`):不展示
  bool _shouldShowMenu() {
    final status = _accountInfo?.status;
    if (status == null) return false;
    if (status == DuoqianStatus.active) return true;
    return widget.isPersonal;
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
      if (widget.isPersonal) {
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
      } else {
        await isar.duoqianInstitutionEntitys
            .where()
            .duoqianAddressEqualTo(widget.institution.duoqianAddress)
            .deleteAll();
      }
    });
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
  /// 仅个人 Pending 路径调用(机构 Pending 不展示此入口);Active 走 propose_close。
  /// 当前仅支持热钱包:冷钱包用户走"管理员列表" → 投反对票完成同样语义。
  Future<void> _confirmRevokeCreate() async {
    if (!widget.isPersonal) return;
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

  Future<void> _openTransferProposal() async {
    final wallets = await _getAdminWallets();
    if (!mounted || wallets.isEmpty) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('未找到此多签账户的管理员钱包')),
        );
      }
      return;
    }

    final created = await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => TransferProposalPage(
          institution: widget.institution,
          icon: widget.isPersonal ? Icons.person : Icons.business,
          badgeColor: widget.isPersonal ? AppTheme.accent : AppTheme.info,
          adminWallets: wallets,
        ),
      ),
    );
    if (created == true && mounted) {
      await _load();
    }
  }

  // ──── UI ────

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: Text(
          widget.isPersonal ? '个人多签账户' : '机构多签账户',
          style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: AppTheme.primaryDark,
        elevation: 0,
        scrolledUnderElevation: 0.5,
        // 三点菜单按"个人/机构 + Active/Pending"四象限分流(2026-05-03 二改):
        // - 个人 Active   → "关闭个人多签":走链上 propose_close 全员投票
        // - 个人 Pending  → "撤销创建":走链上 internal_vote(approve=false) 早期否决
        //                   全员通过的 threshold 决定一票反对即 REJECTED;链上
        //                   cleanup_pending_personal_create 自动清 admins-change
        // - 机构 Active   → "关闭机构多签":同 propose_close 路径
        // - 机构 Pending  → 不展示菜单:机构创建 SFID 治理流程,wuminapp 不应作为撤销入口
        // null 态(链上未找到对应 institution_id)同样不展示菜单。
        actions: [
          if (_shouldShowMenu())
            PopupMenuButton<String>(
              icon: const Icon(Icons.more_vert),
              onSelected: (value) {
                if (value == 'close') _showDeleteMenu();
                if (value == 'revoke_create') _confirmRevokeCreate();
              },
              itemBuilder: (_) {
                final isActive = _accountInfo?.status == DuoqianStatus.active;
                return [
                  if (isActive)
                    PopupMenuItem(
                      value: 'close',
                      child: Row(
                        children: [
                          const Icon(Icons.delete_outline,
                              size: 20, color: AppTheme.danger),
                          const SizedBox(width: 8),
                          Text(
                            widget.isPersonal ? '关闭个人多签' : '关闭机构多签',
                            style: const TextStyle(color: AppTheme.danger),
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
    final statusLabel = info == null
        ? '未找到'
        : info.status == DuoqianStatus.active
            ? '已激活'
            : '待激活';
    final statusColor = info?.status == DuoqianStatus.active
        ? AppTheme.success
        : AppTheme.warning;

    return RefreshIndicator(
      onRefresh: () async {
        _adminService.clearCache(widget.institution.sfidNumber);
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
                  Text(
                    widget.isPersonal ? '账户信息' : '机构信息',
                    style: const TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.w700,
                      color: AppTheme.primaryDark,
                    ),
                  ),
                  const SizedBox(height: 12),
                  _buildInfoRow('名称', widget.institution.name),
                  if (!widget.isPersonal) ...[
                    const Divider(height: 20),
                    _buildInfoRow(
                      'SFID ID',
                      _extractSfidNumber(widget.institution.sfidNumber),
                    ),
                  ],
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
                  // 账户余额(bug 4):Active 显示链上 free_balance,Pending 显示
                  // 发起人承诺金额(snapshot.amount_fen)+ "不可用"灰色标签。
                  const Divider(height: 20),
                  _buildBalanceRow(info?.status),
                  const Divider(height: 20),
                  _buildInfoRow('状态', statusLabel, valueColor: statusColor),
                  // 管理员数量 / 通过阈值 已删除(bug 4):管理员列表卡片
                  // subtitle 已显示这两项信息,避免重复。
                ],
              ),
            ),
          ),

          const SizedBox(height: 16),
          _buildTransferEntryCard(),

          const SizedBox(height: 16),

          // 管理员列表(折叠成单行,点击进入子页)
          _buildAdminEntryCard(info),

          // 个人多签提案列表(req 5):活跃 + 历史(本机 Isar 永久保留终态记录)
          if (widget.isPersonal) ...[
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
        ],
      ),
    );
  }

  /// 管理员列表入口卡片(req 1):点击进入完整管理员列表页(个人多签
  /// 页内含"激活"按钮的三态交互;机构多签复用同 [DuoqianAccountInfoPage]
  /// 路径,本入口仅个人多签开启)。
  Widget _buildAdminEntryCard(DuoqianAccountInfo? info) {
    final adminCount = _adminPubkeys.length;
    final threshold = info?.threshold;
    final subtitle = threshold == null
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
    if (!widget.isPersonal) {
      // 机构多签暂沿用旧的平铺渲染,此处仅 personal 入口开放新子页;
      // 后续机构页改造会单独引入。
      return;
    }
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

  Widget _buildTransferEntryCard() {
    // 待激活的多签账户(链上提案尚未通过 → DuoqianStatus.pending)不允许发起转账提案,
    // 整张卡片置灰显示但不响应点击,文案提示用户先完成激活。
    final canTransfer = _accountInfo?.status == DuoqianStatus.active;
    final accentColor =
        canTransfer ? AppTheme.primaryDark : AppTheme.textTertiary;
    final subtitle = canTransfer ? '从当前多签账户发起链上转账' : '账户尚未激活,无法发起转账';

    // bug 2(2026-05-03):卡片高度对齐 institution_detail_page._buildAdminEntry,
    // 36×36 icon + Padding(14,12),与管理员卡片一致(原 38×38 + Padding(16,14) 偏高)。
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: accentColor.withValues(alpha: 0.15)),
      ),
      child: InkWell(
        onTap: canTransfer ? _openTransferProposal : null,
        borderRadius: BorderRadius.circular(12),
        child: Opacity(
          opacity: canTransfer ? 1.0 : 0.5,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
            child: Row(
              children: [
                Container(
                  width: 36,
                  height: 36,
                  decoration: BoxDecoration(
                    color: accentColor.withValues(alpha: 0.08),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Icon(
                    Icons.send_outlined,
                    size: 18,
                    color: accentColor,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text(
                        '发起转账',
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
                          fontSize: 12,
                          color: AppTheme.textTertiary,
                        ),
                      ),
                    ],
                  ),
                ),
                const Icon(
                  Icons.chevron_right,
                  size: 20,
                  color: AppTheme.textTertiary,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  /// 账户余额行(bug 4):
  /// - Active:链上 free_balance 实时(无标签)
  /// - Pending / null:发起人承诺金额(snapshot.amount_fen)+ "不可用" 灰色标签
  Widget _buildBalanceRow(DuoqianStatus? status) {
    final balanceStr = _balanceYuan == null
        ? '—'
        : '${AmountFormat.format(_balanceYuan!)} GMB';
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

  String _extractSfidNumber(String sfidNumber) {
    // sfidNumber 格式："duoqian:hex..." → 返回原始 sfidNumber
    // 但我们存储的 sfidNumber 是 UTF-8，sfidNumber 是 "duoqian:" + hex address
    // 这里直接显示 sfidNumber 的地址部分
    if (isRegisteredDuoqianIdentity(sfidNumber)) {
      return registeredDuoqianAddressFromIdentity(sfidNumber) ?? sfidNumber;
    }
    return sfidNumber;
  }

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
