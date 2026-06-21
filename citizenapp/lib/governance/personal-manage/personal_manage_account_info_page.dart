import 'dart:async' show unawaited;
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:isar_community/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:citizenapp/isar/wallet_isar.dart';
import 'package:citizenapp/transaction/duoqian-transfer/duoqian_transfer_entry.dart';
import 'package:citizenapp/governance/shared/institution_info.dart';
import 'package:citizenapp/votingengine/internal-vote/internal_vote_service.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/my/util/amount_format.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

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
    this.initialLocalStatus,
    this.initialAdminPubkeys = const [],
  });

  final InstitutionInfo institution;
  final String? initialLocalStatus;
  final List<String> initialAdminPubkeys;

  @override
  State<PersonalManageAccountInfoPage> createState() =>
      _PersonalManageAccountInfoPageState();
}

class _PersonalManageAccountInfoPageState
    extends State<PersonalManageAccountInfoPage> {
  final PersonalManageService _personalManageService = PersonalManageService();
  final ChainRpc _rpc = ChainRpc();

  DuoqianAccountInfo? _accountInfo;
  List<String> _adminPubkeys = const [];
  String _localStatus = PersonalDuoqianLocalState.statusPending;
  int? _lastDetailRefreshAtMillis;
  int? _lastBalanceRefreshAtMillis;
  bool _isClosed = false;

  /// 账户余额(元):Active 来自链上 free_balance,Pending 来自本机 Isar
  /// PersonalDuoqianProposalEntity.snapshotJson.amount_fen(发起人承诺入金)。
  double? _balanceYuan;

  @override
  void initState() {
    super.initState();
    _localStatus =
        widget.initialLocalStatus ?? PersonalDuoqianLocalState.statusPending;
    _adminPubkeys = _normalizeAdminPubkeys(widget.initialAdminPubkeys);
    _isClosed = _localStatus == PersonalDuoqianLocalState.statusClosed;
    _load();
  }

  Future<void> _load() async {
    await _loadFromLocal();
    if (_shouldRefreshDetail()) {
      unawaited(_refreshChainDetail());
    } else {
      unawaited(_refreshBalanceIfNeeded());
    }
  }

  Future<void> _loadFromLocal() async {
    try {
      final local = await WalletIsar.instance.read((isar) async {
        final entity = await isar.personalDuoqianEntitys
            .filter()
            .duoqianAccountEqualTo(widget.institution.duoqianAccount)
            .findFirst();
        final statuses = await PersonalDuoqianLocalState.readStatusSnapshots(
          isar,
          [widget.institution.duoqianAccount],
        );
        final detail = await PersonalDuoqianLocalState.readDetail(
          isar,
          widget.institution.duoqianAccount,
        );
        final pendingBalance = await _readPendingBalanceFromIsar(isar);
        return (
          entity: entity,
          status: statuses[_normalizeHex(widget.institution.duoqianAccount)],
          detail: detail,
          pendingBalance: pendingBalance,
        );
      });

      final status = local.status?.status ??
          local.detail?.status ??
          widget.initialLocalStatus ??
          PersonalDuoqianLocalState.statusPending;
      final isClosed = status == PersonalDuoqianLocalState.statusClosed;
      final admins = local.detail?.adminPubkeys.isNotEmpty == true
          ? local.detail!.adminPubkeys
          : local.entity?.matchedAdminPubkeys.isNotEmpty == true
              ? local.entity!.matchedAdminPubkeys
              : widget.initialAdminPubkeys;
      final normalizedAdmins = _normalizeAdminPubkeys(admins);
      final statusEnum = _statusEnumFromLocal(status);
      final accountInfo = isClosed
          ? null
          : DuoqianAccountInfo(
              adminsLen: normalizedAdmins.length,
              threshold: local.detail?.threshold,
              adminPubkeys: normalizedAdmins,
              status: statusEnum,
            );
      final balance = isClosed
          ? null
          : statusEnum == DuoqianStatus.active
              ? local.detail?.balanceYuan
              : local.pendingBalance ?? local.detail?.balanceYuan;

      if (!mounted) return;
      setState(() {
        _localStatus = status;
        _accountInfo = accountInfo;
        _adminPubkeys = normalizedAdmins;
        _isClosed = isClosed;
        _balanceYuan = balance;
        _lastDetailRefreshAtMillis = local.detail?.lastChainRefreshAtMillis ??
            local.status?.lastSyncAtMillis;
        _lastBalanceRefreshAtMillis = local.detail?.lastBalanceRefreshAtMillis;
      });
    } catch (_) {
      // 中文注释：本地读取失败也不能让详情页进入全屏错误；保留入口传入的
      // 名称、地址和状态，用户仍可下拉触发链上强制刷新。
    }
  }

  bool _shouldRefreshDetail() {
    if (_lastDetailRefreshAtMillis == null) return true;
    final lastSyncAt = DateTime.fromMillisecondsSinceEpoch(
      _lastDetailRefreshAtMillis!,
    );
    final ttl = _localStatus == PersonalDuoqianLocalState.statusActive
        ? const Duration(minutes: 60)
        : const Duration(minutes: 10);
    return DateTime.now().difference(lastSyncAt) >= ttl;
  }

  bool _shouldRefreshBalance() {
    if (_localStatus != PersonalDuoqianLocalState.statusActive) return false;
    if (_balanceYuan == null) return true;
    if (_lastBalanceRefreshAtMillis == null) return true;
    final lastSyncAt = DateTime.fromMillisecondsSinceEpoch(
      _lastBalanceRefreshAtMillis!,
    );
    return DateTime.now().difference(lastSyncAt) >= const Duration(minutes: 10);
  }

  Future<void> _refreshBalanceIfNeeded({bool force = false}) async {
    if (!force && !_shouldRefreshBalance()) return;
    try {
      final balance =
          await _rpc.fetchFinalizedBalance(widget.institution.duoqianAccount);
      final now = DateTime.now().millisecondsSinceEpoch;
      await WalletIsar.instance.writeTxn((isar) async {
        final previous = await PersonalDuoqianLocalState.readDetail(
          isar,
          widget.institution.duoqianAccount,
        );
        await PersonalDuoqianLocalState.putDetailInTxn(
          isar,
          widget.institution.duoqianAccount,
          DuoqianLocalDetailSnapshot(
            status: previous?.status ?? _localStatus,
            adminPubkeys: previous?.adminPubkeys ?? _adminPubkeys,
            threshold: previous?.threshold ?? _accountInfo?.threshold,
            balanceYuan: balance,
            lastChainRefreshAtMillis: previous?.lastChainRefreshAtMillis ??
                _lastDetailRefreshAtMillis,
            lastBalanceRefreshAtMillis: now,
            updatedAtMillis: now,
          ),
        );
      });
      if (!mounted) return;
      setState(() {
        _balanceYuan = balance;
        _lastBalanceRefreshAtMillis = now;
      });
    } catch (_) {
      // 中文注释：余额失败只保留本地旧余额；不要影响详情页其他信息。
    }
  }

  Future<void> _refreshChainDetail({bool force = false}) async {
    if (!force && !_shouldRefreshDetail()) return;
    try {
      final infos = await _personalManageService.fetchPersonalAccountsBatch(
        [widget.institution.duoqianAccount],
      );
      final info = infos[_normalizeHex(widget.institution.duoqianAccount)];
      final status = info == null
          ? PersonalDuoqianLocalState.statusClosed
          : _localStatusFromInfo(info.status);
      final balance = info == null ? null : await _resolveBalance(info.status);
      final now = DateTime.now().millisecondsSinceEpoch;

      await WalletIsar.instance.writeTxn((isar) async {
        await PersonalDuoqianLocalState.putStatusInTxn(
          isar,
          widget.institution.duoqianAccount,
          status,
        );
        if (info == null) {
          await PersonalDuoqianLocalState.deleteDetailInTxn(
            isar,
            widget.institution.duoqianAccount,
          );
        } else {
          final previous = await PersonalDuoqianLocalState.readDetail(
            isar,
            widget.institution.duoqianAccount,
          );
          await PersonalDuoqianLocalState.putDetailInTxn(
            isar,
            widget.institution.duoqianAccount,
            DuoqianLocalDetailSnapshot(
              status: status,
              adminPubkeys: info.adminPubkeys,
              threshold: info.threshold,
              balanceYuan: balance ?? previous?.balanceYuan,
              lastChainRefreshAtMillis: now,
              lastBalanceRefreshAtMillis:
                  info.status == DuoqianStatus.active && balance != null
                      ? now
                      : previous?.lastBalanceRefreshAtMillis,
              updatedAtMillis: now,
            ),
          );
        }
      });

      if (!mounted) return;
      setState(() {
        _localStatus = status;
        _isClosed = status == PersonalDuoqianLocalState.statusClosed;
        _accountInfo = info;
        _adminPubkeys = _normalizeAdminPubkeys(info?.adminPubkeys);
        _balanceYuan = _isClosed ? null : balance ?? _balanceYuan;
        _lastDetailRefreshAtMillis = now;
        if (_isClosed) {
          _lastBalanceRefreshAtMillis = null;
        } else if (balance != null) {
          _lastBalanceRefreshAtMillis = now;
        }
      });
    } catch (_) {
      // 中文注释：链上刷新失败只保留本地详情，不弹进度提示或全屏失败。
    }
  }

  Future<double?> _resolveBalance(DuoqianStatus? status) async {
    if (status == DuoqianStatus.active) {
      try {
        return await _rpc
            .fetchFinalizedBalance(widget.institution.duoqianAccount);
      } catch (_) {
        return null;
      }
    }
    return WalletIsar.instance.read(_readPendingBalanceFromIsar);
  }

  Future<double?> _readPendingBalanceFromIsar(Isar isar) async {
    // Pending 态:从本机 Isar PersonalDuoqianProposalEntity 取
    // (该 multisig 的 create 提案 snapshot 含 amount_fen)。
    try {
      final entity = await isar.personalDuoqianProposalEntitys
          .filter()
          .personalAddressEqualTo(widget.institution.duoqianAccount)
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

  String _localStatusFromInfo(DuoqianStatus status) {
    return status == DuoqianStatus.active
        ? PersonalDuoqianLocalState.statusActive
        : PersonalDuoqianLocalState.statusPending;
  }

  DuoqianStatus _statusEnumFromLocal(String status) {
    return status == PersonalDuoqianLocalState.statusActive
        ? DuoqianStatus.active
        : DuoqianStatus.pending;
  }

  List<String> _normalizeAdminPubkeys(List<String>? admins) {
    if (admins == null) return const [];
    return admins
        .map(_normalizeHex)
        .where((item) => item.isNotEmpty)
        .toList(growable: false);
  }

  String _normalizeHex(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    return h.toLowerCase();
  }

  // ──── 关闭 ────

  void _confirmClose() {
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
    var wallets = await _getAdminWallets();
    if (wallets.isEmpty) {
      await _refreshChainDetail(force: true);
      wallets = await _getAdminWallets();
    }
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
      // 此时 admins-change AdminAccounts 仍存,反向索引下次扫还会拉回 → **不能立即删本地**。
      // 等链上 close execute 自动清掉 AdminAccounts 后,反向索引下次扫不到再清孤立 entity。
      Navigator.pop(context);
    }
  }

  /// 是否展示右上角三点菜单；Active 显关闭，Pending 显撤销创建，Closed 显删除。
  bool _shouldShowMenu() {
    if (_isClosed) return true;
    return _localStatus == PersonalDuoqianLocalState.statusActive ||
        _localStatus == PersonalDuoqianLocalState.statusPending;
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
    await WalletIsar.instance.writeTxn((isar) async {
      await isar.personalDuoqianEntitys
          .where()
          .duoqianAccountEqualTo(widget.institution.duoqianAccount)
          .deleteAll();
      // 个人多签 create/transfer/close 提案 snapshot 一并清掉,否则
      // [PersonalProposalHistoryService] 下次会把它们再拉回详情页。
      await isar.personalDuoqianProposalEntitys
          .filter()
          .personalAddressEqualTo(widget.institution.duoqianAccount)
          .deleteAll();
      await PersonalDuoqianLocalState.deleteStatusInTxn(
        isar,
        widget.institution.duoqianAccount,
      );
      await PersonalDuoqianLocalState.deleteDetailInTxn(
        isar,
        widget.institution.duoqianAccount,
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
  /// `PendingPersonalCreate` / `admins-change::AdminAccounts`。其他管理员设备的反向索引下次扫不到该
  /// AccountId,自动清理孤立 Isar entity。
  ///
  /// 仅个人 Pending 路径调用；Active 走 propose_close。
  /// 当前仅支持热钱包:冷钱包用户走"管理员列表" → 投反对票完成同样语义。
  Future<void> _confirmRevokeCreate() async {
    if (_localStatus == PersonalDuoqianLocalState.statusActive) return;

    var adminWallets = await _getAdminWallets();
    if (adminWallets.isEmpty) {
      await _refreshChainDetail(force: true);
      adminWallets = await _getAdminWallets();
    }
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
        .findActiveCreate(widget.institution.duoqianAccount);
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
        // - Active  → 关闭个人多签，走 PersonalManage::propose_close，不显示删除图标。
        // - Pending → 撤销创建，走 InternalVote approve=false 早期否决。
        actions: [
          if (_shouldShowMenu())
            PopupMenuButton<String>(
              icon: const Icon(Icons.more_vert),
              onSelected: (value) {
                if (value == 'delete') _confirmDeleteLocal();
                if (value == 'close') _confirmClose();
                if (value == 'revoke_create') _confirmRevokeCreate();
              },
              itemBuilder: (_) {
                final isActive =
                    _localStatus == PersonalDuoqianLocalState.statusActive;
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
                      child: Text(
                        '关闭个人多签',
                        style: TextStyle(color: AppTheme.danger),
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
      body: _buildContent(),
    );
  }

  Widget _buildContent() {
    final duoqianSs58 = _hexToSs58(widget.institution.duoqianAccount);
    final info = _accountInfo;
    final statusLabel = _isClosed
        ? '已注销'
        : _localStatus == PersonalDuoqianLocalState.statusActive
            ? '已激活'
            : '待激活';
    final statusColor = _isClosed
        ? AppTheme.textTertiary
        : _localStatus == PersonalDuoqianLocalState.statusActive
            ? AppTheme.success
            : AppTheme.warning;

    return RefreshIndicator(
      onRefresh: () async {
        await _refreshChainDetail(force: true);
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
                    _buildBalanceRow(_statusEnumFromLocal(_localStatus)),
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
              enabled: _localStatus == PersonalDuoqianLocalState.statusActive,
              loadAdminWallets: _getAdminWallets,
              onCreated: () => _refreshChainDetail(force: true),
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
    final adminsLen = _adminPubkeys.length;
    final threshold = info?.threshold;
    final subtitle = _isClosed
        ? '已注销'
        : threshold == null
            ? '$adminsLen 人'
            : '$adminsLen 人 · 阈值 $threshold/$adminsLen';

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
    if (_adminPubkeys.isEmpty) {
      await _refreshChainDetail(force: true);
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
          duoqianStatus: _statusEnumFromLocal(_localStatus),
          adminPubkeys: _adminPubkeys,
          adminWallets: wallets,
          creatorPubkeyHex: creator,
        ),
      ),
    );
    // 子页可能完成投票 → 精准刷新当前多签状态(可能已激活)。
    if (mounted) await _refreshChainDetail(force: true);
  }

  /// 从本机 Isar 读取个人多签创建者公钥 hex。
  /// req 3 未实现时,只有创建者本机有此记录;非创建者打开子页 creatorPubkeyHex 为 null
  /// (届时所有 admin 都按"非创建者"渲染,语义略损但不阻塞主流程)。
  Future<String?> _resolvePersonalCreatorPubkeyHex() async {
    try {
      final entity = await WalletIsar.instance.read((isar) {
        return isar.personalDuoqianEntitys
            .filter()
            .duoqianAccountEqualTo(widget.institution.duoqianAccount)
            .findFirst();
      });
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
