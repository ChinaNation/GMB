import 'dart:async' show unawaited;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:isar/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/transaction/duoqian-transfer/duoqian_transfer_entry.dart';
import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/my/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import 'institution_duoqian_close_page.dart';
import 'institution_manage_models.dart';
import 'institution_manage_service.dart';

/// 机构多签账户详情页。
///
/// 只展示和处理 OrganizationManage 机构多签。个人多签详情已经迁移到
/// `lib/personal-manage/personal_manage_account_info_page.dart`。
class InstitutionAccountInfoPage extends StatefulWidget {
  const InstitutionAccountInfoPage({
    super.key,
    required this.institution,
    this.initialLocalStatus,
    this.initialAdminPubkeys = const [],
  });

  final InstitutionInfo institution;
  final String? initialLocalStatus;
  final List<String> initialAdminPubkeys;

  @override
  State<InstitutionAccountInfoPage> createState() => _InstitutionAccountInfoPageState();
}

class _InstitutionAccountInfoPageState extends State<InstitutionAccountInfoPage> {
  final InstitutionManageService _manageService = InstitutionManageService();
  final ChainRpc _rpc = ChainRpc();

  InstitutionAccountInfo? _accountInfo;
  List<String> _adminPubkeys = const [];
  String _localStatus = InstitutionDuoqianLocalState.statusPending;
  int? _lastDetailRefreshAtMillis;
  int? _lastBalanceRefreshAtMillis;
  double? _balanceYuan;

  @override
  void initState() {
    super.initState();
    _localStatus =
        widget.initialLocalStatus ?? InstitutionDuoqianLocalState.statusPending;
    _adminPubkeys = _normalizeAdminPubkeys(widget.initialAdminPubkeys);
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
        final entity = await isar.institutionEntitys
            .filter()
            .duoqianAddressEqualTo(widget.institution.duoqianAddress)
            .findFirst();
        final statuses = await InstitutionDuoqianLocalState.readStatusSnapshots(
          isar,
          [widget.institution.duoqianAddress],
        );
        final detail = await InstitutionDuoqianLocalState.readDetail(
          isar,
          widget.institution.duoqianAddress,
        );
        return (
          entity: entity,
          status: statuses[_normalizeHex(widget.institution.duoqianAddress)],
          detail: detail,
        );
      });

      final status = local.status?.status ??
          local.detail?.status ??
          widget.initialLocalStatus ??
          InstitutionDuoqianLocalState.statusPending;
      final isClosed = status == InstitutionDuoqianLocalState.statusClosed;
      final admins = local.detail?.adminPubkeys.isNotEmpty == true
          ? local.detail!.adminPubkeys
          : local.entity?.matchedAdminPubkeys.isNotEmpty == true
              ? local.entity!.matchedAdminPubkeys
              : widget.initialAdminPubkeys;
      final normalizedAdmins = _normalizeAdminPubkeys(admins);
      final accountInfo = isClosed
          ? null
          : InstitutionAccountInfo(
              adminCount: normalizedAdmins.length,
              threshold: local.detail?.threshold,
              adminPubkeys: normalizedAdmins,
              status: _statusEnumFromLocal(status),
            );

      if (!mounted) return;
      setState(() {
        _localStatus = status;
        _accountInfo = accountInfo;
        _adminPubkeys = normalizedAdmins;
        _balanceYuan = isClosed ? null : local.detail?.balanceYuan;
        _lastDetailRefreshAtMillis = local.detail?.lastChainRefreshAtMillis ??
            local.status?.lastSyncAtMillis;
        _lastBalanceRefreshAtMillis = local.detail?.lastBalanceRefreshAtMillis;
      });
    } catch (_) {
      // 中文注释：本地读取失败不阻塞详情页；页面仍展示入口传入的名称和地址。
    }
  }

  bool _shouldRefreshDetail() {
    if (_lastDetailRefreshAtMillis == null) return true;
    final lastSyncAt = DateTime.fromMillisecondsSinceEpoch(
      _lastDetailRefreshAtMillis!,
    );
    final ttl = _localStatus == InstitutionDuoqianLocalState.statusActive
        ? const Duration(minutes: 60)
        : const Duration(minutes: 10);
    return DateTime.now().difference(lastSyncAt) >= ttl;
  }

  bool _shouldRefreshBalance() {
    if (_localStatus != InstitutionDuoqianLocalState.statusActive) return false;
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
          await _rpc.fetchFinalizedBalance(widget.institution.duoqianAddress);
      final now = DateTime.now().millisecondsSinceEpoch;
      await WalletIsar.instance.writeTxn((isar) async {
        final previous = await InstitutionDuoqianLocalState.readDetail(
          isar,
          widget.institution.duoqianAddress,
        );
        await InstitutionDuoqianLocalState.putDetailInTxn(
          isar,
          widget.institution.duoqianAddress,
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
      final infos = await _manageService.fetchDuoqianAccountsBatch(
        [widget.institution.duoqianAddress],
      );
      final info = infos[_normalizeHex(widget.institution.duoqianAddress)];
      final status = info == null
          ? InstitutionDuoqianLocalState.statusClosed
          : _localStatusFromInfo(info.status);
      final balance = info == null ? null : await _resolveBalance(info.status);
      final now = DateTime.now().millisecondsSinceEpoch;

      await WalletIsar.instance.writeTxn((isar) async {
        await InstitutionDuoqianLocalState.putStatusInTxn(
          isar,
          widget.institution.duoqianAddress,
          status,
        );
        if (info == null) {
          await InstitutionDuoqianLocalState.deleteDetailInTxn(
            isar,
            widget.institution.duoqianAddress,
          );
        } else {
          final previous = await InstitutionDuoqianLocalState.readDetail(
            isar,
            widget.institution.duoqianAddress,
          );
          await InstitutionDuoqianLocalState.putDetailInTxn(
            isar,
            widget.institution.duoqianAddress,
            DuoqianLocalDetailSnapshot(
              status: status,
              adminPubkeys: info.adminPubkeys,
              threshold: info.threshold,
              balanceYuan: balance ?? previous?.balanceYuan,
              lastChainRefreshAtMillis: now,
              lastBalanceRefreshAtMillis:
                  info.status == InstitutionStatus.active && balance != null
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
        _accountInfo = info;
        _adminPubkeys = _normalizeAdminPubkeys(info?.adminPubkeys);
        _balanceYuan = status == InstitutionDuoqianLocalState.statusClosed
            ? null
            : balance ?? _balanceYuan;
        _lastDetailRefreshAtMillis = now;
        if (status == InstitutionDuoqianLocalState.statusClosed) {
          _lastBalanceRefreshAtMillis = null;
        } else if (balance != null) {
          _lastBalanceRefreshAtMillis = now;
        }
      });
    } catch (_) {
      // 中文注释：链上刷新失败只保留本地详情，不弹同步提示也不清空页面。
    }
  }

  Future<double?> _resolveBalance(InstitutionStatus? status) async {
    if (status != InstitutionStatus.active) return null;
    try {
      return await _rpc
          .fetchFinalizedBalance(widget.institution.duoqianAddress);
    } catch (_) {
      return null;
    }
  }

  String _localStatusFromInfo(InstitutionStatus status) {
    return status == InstitutionStatus.active
        ? InstitutionDuoqianLocalState.statusActive
        : InstitutionDuoqianLocalState.statusPending;
  }

  InstitutionStatus _statusEnumFromLocal(String status) {
    return status == InstitutionDuoqianLocalState.statusActive
        ? InstitutionStatus.active
        : InstitutionStatus.pending;
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

  void _confirmClose() {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('关闭机构多签'),
        content: const Text(
          '关闭机构多签将发起链上关闭提案，需要其他管理员投票通过后才会真正关闭。\n\n确定要发起关闭吗？',
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
          const SnackBar(content: Text('请先导入此机构的管理员钱包')),
        );
      }
      return;
    }

    final closed = await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => InstitutionDuoqianClosePage(
          institution: widget.institution,
          adminWallets: wallets,
        ),
      ),
    );
    if (closed == true && mounted) {
      // 中文注释：关闭提案提交后仍需链上投票通过，不能立即删除本地机构记录。
      Navigator.pop(context);
    }
  }

  bool get _isClosed =>
      _localStatus == InstitutionDuoqianLocalState.statusClosed;

  bool _shouldShowMenu() =>
      _localStatus == InstitutionDuoqianLocalState.statusActive || _isClosed;

  Future<void> _removeFromLocal() async {
    await WalletIsar.instance.writeTxn((isar) async {
      await isar.institutionEntitys
          .deleteByDuoqianAddress(widget.institution.duoqianAddress);
      await InstitutionDuoqianLocalState.deleteStatusInTxn(
        isar,
        widget.institution.duoqianAddress,
      );
      await InstitutionDuoqianLocalState.deleteDetailInTxn(
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
        content: const Text('确认删除该已注销机构多签账户在本机的所有数据？'),
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
    if (ok != true) return;
    await _removeFromLocal();
    if (!mounted) return;
    Navigator.of(context).pop(true);
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

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '机构多签账户',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: AppTheme.primaryDark,
        elevation: 0,
        scrolledUnderElevation: 0.5,
        actions: [
          if (_shouldShowMenu())
            PopupMenuButton<String>(
              icon: const Icon(Icons.more_vert),
              onSelected: (value) {
                if (value == 'close') _confirmClose();
                if (value == 'delete') _confirmDeleteLocal();
              },
              itemBuilder: (_) => [
                if (_localStatus == InstitutionDuoqianLocalState.statusActive)
                  const PopupMenuItem(
                    value: 'close',
                    child: Text(
                      '关闭机构多签',
                      style: TextStyle(color: AppTheme.danger),
                    ),
                  ),
                if (_isClosed)
                  const PopupMenuItem(
                    value: 'delete',
                    child: Row(
                      children: [
                        Icon(
                          Icons.delete_outline,
                          size: 20,
                          color: AppTheme.danger,
                        ),
                        SizedBox(width: 8),
                        Text(
                          '删除',
                          style: TextStyle(color: AppTheme.danger),
                        ),
                      ],
                    ),
                  ),
              ],
            ),
        ],
      ),
      body: _buildContent(),
    );
  }

  Widget _buildContent() {
    final duoqianSs58 = _hexToSs58(widget.institution.duoqianAddress);
    final info = _accountInfo;
    final statusLabel = _isClosed
        ? '已注销'
        : _localStatus == InstitutionDuoqianLocalState.statusActive
            ? '已激活'
            : '待激活';
    final statusColor = _isClosed
        ? AppTheme.textTertiary
        : _localStatus == InstitutionDuoqianLocalState.statusActive
            ? AppTheme.success
            : AppTheme.warning;

    return RefreshIndicator(
      onRefresh: () async {
        await _refreshChainDetail(force: true);
      },
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
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
                    '机构信息',
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
                    'SFID ID',
                    _extractSfidNumber(widget.institution.sfidNumber),
                  ),
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
                    const Divider(height: 20),
                    _buildBalanceRow(),
                  ],
                  const Divider(height: 20),
                  _buildInfoRow('状态', statusLabel, valueColor: statusColor),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          if (!_isClosed) ...[
            DuoqianTransferEntryCard(
              institution: widget.institution,
              isPersonal: false,
              enabled:
                  _localStatus == InstitutionDuoqianLocalState.statusActive,
              loadAdminWallets: _getAdminWallets,
              onCreated: () => _refreshChainDetail(force: true),
            ),
            const SizedBox(height: 16),
          ],
          _buildAdminEntryCard(info),
        ],
      ),
    );
  }

  Widget _buildAdminEntryCard(InstitutionAccountInfo? info) {
    final adminCount = _adminPubkeys.length;
    final threshold = info?.threshold;
    final subtitle = threshold == null
        ? '$adminCount 人'
        : '$adminCount 人 · 阈值 $threshold/$adminCount';

    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: const BorderSide(color: AppTheme.border),
      ),
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
              child: const Icon(
                Icons.group_outlined,
                size: 18,
                color: AppTheme.primaryDark,
              ),
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
                      fontSize: 12,
                      color: AppTheme.textTertiary,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBalanceRow() {
    final balanceStr = _balanceYuan == null
        ? '-'
        : '${AmountFormat.format(_balanceYuan!)} GMB';
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
          child: Text(
            balanceStr,
            style: const TextStyle(
              fontSize: 13,
              color: AppTheme.textPrimary,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildInfoRow(
    String label,
    String value, {
    VoidCallback? onCopy,
    Color? valueColor,
  }) {
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
            child: const Icon(
              Icons.copy,
              size: 16,
              color: AppTheme.textTertiary,
            ),
          ),
      ],
    );
  }

  String _extractSfidNumber(String sfidNumber) {
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
