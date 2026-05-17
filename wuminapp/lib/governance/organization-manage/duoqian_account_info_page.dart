import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/institution_admin_service.dart';
import 'package:wuminapp_mobile/transaction/duoqian-transfer/duoqian_transfer_entry.dart';
import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/my/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import 'institution_duoqian_close_page.dart';
import 'duoqian_manage_models.dart';
import 'duoqian_manage_service.dart';

/// 机构多签账户详情页。
///
/// 只展示和处理 OrganizationManage 机构多签。个人多签详情已经迁移到
/// `lib/personal-manage/personal_manage_account_info_page.dart`。
class DuoqianAccountInfoPage extends StatefulWidget {
  const DuoqianAccountInfoPage({
    super.key,
    required this.institution,
  });

  final InstitutionInfo institution;

  @override
  State<DuoqianAccountInfoPage> createState() => _DuoqianAccountInfoPageState();
}

class _DuoqianAccountInfoPageState extends State<DuoqianAccountInfoPage> {
  final DuoqianManageService _manageService = DuoqianManageService();
  final InstitutionAdminService _adminService = InstitutionAdminService();
  final ChainRpc _rpc = ChainRpc();

  AdminSubjectIdentity get _subjectIdentity =>
      AdminSubjectIdentity.fromInstitution(widget.institution);

  bool _loading = true;
  String? _error;

  DuoqianAccountInfo? _accountInfo;
  List<String> _adminPubkeys = const [];
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
        _adminService.fetchAdmins(_subjectIdentity),
      ]);

      final accountInfo = results[0] as DuoqianAccountInfo?;
      final admins = results[1] as List<String>;
      final balance = await _resolveBalance(accountInfo?.status);
      await _writeLocalStatus(accountInfo);

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
    if (status != DuoqianStatus.active) return null;
    try {
      return await _rpc.fetchBalance(widget.institution.duoqianAddress);
    } catch (_) {
      return null;
    }
  }

  Future<void> _writeLocalStatus(DuoqianAccountInfo? accountInfo) async {
    final status = accountInfo == null
        ? InstitutionDuoqianLocalState.statusClosed
        : accountInfo.status == DuoqianStatus.active
            ? InstitutionDuoqianLocalState.statusActive
            : InstitutionDuoqianLocalState.statusPending;
    await WalletIsar.instance.writeTxn((isar) async {
      await InstitutionDuoqianLocalState.putStatusInTxn(
        isar,
        widget.institution.duoqianAddress,
        status,
      );
    });
  }

  void _showDeleteMenu() {
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
    final wallets = await _getAdminWallets();
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

  bool get _isClosed => _accountInfo == null;

  bool _shouldShowMenu() =>
      _accountInfo?.status == DuoqianStatus.active || _isClosed;

  Future<void> _removeFromLocal() async {
    await WalletIsar.instance.writeTxn((isar) async {
      await isar.duoqianInstitutionEntitys
          .deleteByDuoqianAddress(widget.institution.duoqianAddress);
      await InstitutionDuoqianLocalState.deleteStatusInTxn(
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
                if (value == 'close') _showDeleteMenu();
                if (value == 'delete') _confirmDeleteLocal();
              },
              itemBuilder: (_) => [
                if (_accountInfo?.status == DuoqianStatus.active)
                  const PopupMenuItem(
                    value: 'close',
                    child: Row(
                      children: [
                        Icon(
                          Icons.delete_outline,
                          size: 20,
                          color: AppTheme.danger,
                        ),
                        SizedBox(width: 8),
                        Text(
                          '关闭机构多签',
                          style: TextStyle(color: AppTheme.danger),
                        ),
                      ],
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
            const Text(
              '加载失败',
              style: TextStyle(fontSize: 16, color: AppTheme.textSecondary),
            ),
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
        ? '已注销'
        : info.status == DuoqianStatus.active
            ? '已激活'
            : '待激活';
    final statusColor = info == null
        ? AppTheme.textTertiary
        : info.status == DuoqianStatus.active
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
              enabled: _accountInfo?.status == DuoqianStatus.active,
              loadAdminWallets: _getAdminWallets,
              onCreated: _load,
            ),
            const SizedBox(height: 16),
          ],
          _buildAdminEntryCard(info),
        ],
      ),
    );
  }

  Widget _buildAdminEntryCard(DuoqianAccountInfo? info) {
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
