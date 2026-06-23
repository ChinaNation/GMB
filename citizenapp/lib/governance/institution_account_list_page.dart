// 多签统一账户列表页(个人 + 机构 混合视图)。
//
// 设计要点:
// - 后端按 governance/personal-manage 与 governance/organization-manage 分开;
//   本页只是 UI 编排壳子,并行加载两套数据源,合并按时间倒序展示。
// - 首屏只读本地 Isar,链上状态刷新和反向发现均转为后台任务;
//   下拉刷新才强制查链和全量 discovery。
// - 反向校验由各自 discovery service 内部完成,本页不涉及。
// - 右上角 "+" 弹 ActionSheet,2 选项分别进入个人多签/机构多签创建页。

import 'dart:async' show unawaited;
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:isar_community/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:citizenapp/governance/shared/institution_info.dart';
import 'package:citizenapp/isar/wallet_isar.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import 'organization-manage/institution_account_info_page.dart';
import 'organization-manage/institution_manage_models.dart' as org_models;
import 'organization-manage/institution_manage_service.dart';
import 'organization-manage/institution_duoqian_create_page.dart';
import 'personal-manage/personal_account_create_page.dart';
import 'personal-manage/personal_manage_account_info_page.dart';
import 'personal-manage/personal_manage_models.dart';
import 'personal-manage/personal_manage_service.dart';
import 'personal-manage/personal_proposal_history_service.dart';
import 'shared/multisig_discovery_coordinator.dart';

enum _AccountKind { personal, institution }

class _UnifiedItem {
  _UnifiedItem.personal(
    PersonalAccountEntity item, {
    required this.localStatus,
  })  : kind = _AccountKind.personal,
        displayLabel = item.name,
        account = item.account,
        addedAtMillis = item.addedAtMillis,
        discoveredViaAdmin = item.discoveredViaAdmin,
        matchedAdminsLen = item.matchedAdminPubkeys.length,
        personal = item,
        institution = null;

  _UnifiedItem.institution(
    InstitutionEntity item, {
    required this.localStatus,
  })  : kind = _AccountKind.institution,
        displayLabel = item.accountName,
        account = item.account,
        addedAtMillis = item.addedAtMillis,
        discoveredViaAdmin = item.discoveredViaAdmin,
        matchedAdminsLen = item.matchedAdminPubkeys.length,
        personal = null,
        institution = item;

  final _AccountKind kind;
  final String displayLabel;
  final String account;
  final int addedAtMillis;
  final bool discoveredViaAdmin;
  final int matchedAdminsLen;
  final String? localStatus;
  final PersonalAccountEntity? personal;
  final InstitutionEntity? institution;
}

class InstitutionAccountListPage extends StatefulWidget {
  const InstitutionAccountListPage({super.key});

  @override
  State<InstitutionAccountListPage> createState() =>
      _InstitutionAccountListPageState();
}

class _InstitutionAccountListPageState
    extends State<InstitutionAccountListPage> {
  List<_UnifiedItem> _items = [];
  bool _loading = true;
  bool _scanning = false;
  String? _scanProgress;
  final PersonalManageService _personalManageService = PersonalManageService();
  final PersonalProposalHistoryService _personalProposalHistoryService =
      PersonalProposalHistoryService();
  final InstitutionManageService _duoqianManageService =
      InstitutionManageService();

  static const _activeStatusTtl = Duration(minutes: 60);
  static const _inactiveStatusTtl = Duration(minutes: 10);
  static const _discoveryWalletFingerprintKey =
      'duoqian_discovery_wallet_fingerprint';

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load({bool runDiscovery = true}) async {
    setState(() => _loading = true);
    try {
      await _readFromIsar();
    } catch (_) {
      // 本地库异常不阻塞页面进入,空态可继续触发手动刷新。
    }
    if (!mounted) return;
    setState(() => _loading = false);

    if (runDiscovery) {
      unawaited(_runBackgroundRefresh());
    }
  }

  Future<void> _readFromIsar() async {
    final snapshot = await WalletIsar.instance.read((isar) async {
      final personals = await isar.personalAccountEntitys.where().findAll();
      final institutions = await isar.institutionEntitys.where().findAll();
      final personalStatuses = await PersonalAccountLocalState.readStatuses(
        isar,
        personals.map((p) => p.account),
      );
      final institutionStatuses =
          await InstitutionDuoqianLocalState.readStatuses(
        isar,
        institutions.map((p) => p.account),
      );
      return (
        personals: personals,
        institutions: institutions,
        personalStatuses: personalStatuses,
        institutionStatuses: institutionStatuses,
      );
    });
    final merged = <_UnifiedItem>[
      ...snapshot.personals.map(
        (p) => _UnifiedItem.personal(
          p,
          localStatus: snapshot.personalStatuses[_normalizeHex(p.account)],
        ),
      ),
      ...snapshot.institutions.map(
        (p) => _UnifiedItem.institution(
          p,
          localStatus: snapshot.institutionStatuses[_normalizeHex(p.account)],
        ),
      ),
    ]..sort((a, b) => b.addedAtMillis.compareTo(a.addedAtMillis));
    if (!mounted) return;
    setState(() => _items = merged);
  }

  Future<void> _runBackgroundRefresh() async {
    await _refreshKnownStatuses();
    await _runDiscoveryIfWalletsChanged();
  }

  Future<void> _refreshKnownStatuses({
    bool force = false,
    Set<String>? personalAccounts,
    Set<String>? institutionAccounts,
  }) async {
    final snapshot = await WalletIsar.instance.read((isar) async {
      final personals = await isar.personalAccountEntitys.where().findAll();
      final institutions = await isar.institutionEntitys.where().findAll();
      final personalStatuses =
          await PersonalAccountLocalState.readStatusSnapshots(
        isar,
        personals.map((p) => p.account),
      );
      final institutionStatuses =
          await InstitutionDuoqianLocalState.readStatusSnapshots(
        isar,
        institutions.map((p) => p.account),
      );
      return (
        personals: personals,
        institutions: institutions,
        personalStatuses: personalStatuses,
        institutionStatuses: institutionStatuses,
      );
    });

    final personalFilter = personalAccounts?.map(_normalizeHex).toSet();
    final institutionFilter = institutionAccounts?.map(_normalizeHex).toSet();
    final personalTargets = snapshot.personals.where((item) {
      final address = _normalizeHex(item.account);
      if (personalFilter != null && !personalFilter.contains(address)) {
        return false;
      }
      return force || _shouldRefreshStatus(snapshot.personalStatuses[address]);
    }).toList(growable: false);
    final institutionTargets = snapshot.institutions.where((item) {
      final address = _normalizeHex(item.account);
      if (institutionFilter != null && !institutionFilter.contains(address)) {
        return false;
      }
      return force ||
          _shouldRefreshStatus(snapshot.institutionStatuses[address]);
    }).toList(growable: false);

    if (personalTargets.isEmpty && institutionTargets.isEmpty) return;
    await Future.wait([
      _syncPersonalStatuses(personalTargets),
      _syncInstitutionStatuses(institutionTargets),
    ]);
    await _readFromIsar();
  }

  bool _shouldRefreshStatus(DuoqianLocalStatusSnapshot? snapshot) {
    if (snapshot?.lastSyncAtMillis == null) return true;
    final lastSyncAt = DateTime.fromMillisecondsSinceEpoch(
      snapshot!.lastSyncAtMillis!,
    );
    final ttl = snapshot.status == PersonalAccountLocalState.statusActive ||
            snapshot.status == InstitutionDuoqianLocalState.statusActive
        ? _activeStatusTtl
        : _inactiveStatusTtl;
    return DateTime.now().difference(lastSyncAt) >= ttl;
  }

  Future<void> _syncPersonalStatuses(
      List<PersonalAccountEntity> personals) async {
    if (personals.isEmpty) return;
    Map<String, AccountInfo?> infos;
    try {
      infos = await _personalManageService.fetchPersonalAccountsBatch(
        personals.map((p) => p.account),
      );
    } catch (_) {
      // 中文注释：批量查链失败时保留本地旧状态，不能把网络失败写成已注销。
      return;
    }
    for (final personal in personals) {
      try {
        final info = infos[_normalizeHex(personal.account)];
        if (info == null &&
            await _personalProposalHistoryService
                .hasUnchainedVotingCreateProposal(personal.account)) {
          await _deletePersonalGhost(personal.account);
          continue;
        }
        final status = info == null
            ? PersonalAccountLocalState.statusClosed
            : info.status == DuoqianStatus.active
                ? PersonalAccountLocalState.statusActive
                : PersonalAccountLocalState.statusPending;
        await WalletIsar.instance.writeTxn((isar) async {
          await PersonalAccountLocalState.putStatusInTxn(
            isar,
            personal.account,
            status,
          );
          if (info == null) {
            await PersonalAccountLocalState.deleteDetailInTxn(
              isar,
              personal.account,
            );
          } else {
            final previousDetail = await PersonalAccountLocalState.readDetail(
              isar,
              personal.account,
            );
            await PersonalAccountLocalState.putDetailInTxn(
              isar,
              personal.account,
              DuoqianLocalDetailSnapshot(
                status: status,
                admins: info.admins,
                threshold: info.threshold,
                balanceYuan: previousDetail?.balanceYuan,
                lastBalanceRefreshAtMillis:
                    previousDetail?.lastBalanceRefreshAtMillis,
                updatedAtMillis: DateTime.now().millisecondsSinceEpoch,
                lastChainRefreshAtMillis: DateTime.now().millisecondsSinceEpoch,
              ),
            );
          }
        });
      } catch (_) {
        // 中文注释：链路异常时不改本地状态，避免把网络失败误判成注销。
      }
    }
  }

  Future<void> _deletePersonalGhost(String personalAccountHex) async {
    await WalletIsar.instance.writeTxn((isar) async {
      // 中文注释：旧版本曾在 txHash 返回后提前写入本地多签；若链上没有账户
      // 且创建提案也不存在，说明它从未上链，不能展示为“已注销”。
      await isar.personalAccountEntitys
          .where()
          .accountEqualTo(personalAccountHex)
          .deleteAll();
      await isar.personalAccountProposalEntitys
          .filter()
          .personalAccountEqualTo(personalAccountHex)
          .deleteAll();
      await PersonalAccountLocalState.deleteStatusInTxn(
        isar,
        personalAccountHex,
      );
      await PersonalAccountLocalState.deleteDetailInTxn(
        isar,
        personalAccountHex,
      );
    });
  }

  Future<void> _syncInstitutionStatuses(
    List<InstitutionEntity> institutions,
  ) async {
    if (institutions.isEmpty) return;
    Map<String, org_models.InstitutionAccountInfo?> infos;
    try {
      infos = await _duoqianManageService.fetchAccountsBatch(
        institutions.map((p) => p.account),
      );
    } catch (_) {
      // 中文注释：批量查链失败时保留本地旧状态，不能把网络失败写成已注销。
      return;
    }
    for (final institution in institutions) {
      try {
        final info = infos[_normalizeHex(institution.account)];
        final status = info == null
            ? InstitutionDuoqianLocalState.statusClosed
            : info.status == org_models.InstitutionStatus.active
                ? InstitutionDuoqianLocalState.statusActive
                : InstitutionDuoqianLocalState.statusPending;
        await WalletIsar.instance.writeTxn((isar) async {
          await InstitutionDuoqianLocalState.putStatusInTxn(
            isar,
            institution.account,
            status,
          );
          if (info == null) {
            await InstitutionDuoqianLocalState.deleteDetailInTxn(
              isar,
              institution.account,
            );
          } else {
            final previousDetail =
                await InstitutionDuoqianLocalState.readDetail(
              isar,
              institution.account,
            );
            await InstitutionDuoqianLocalState.putDetailInTxn(
              isar,
              institution.account,
              DuoqianLocalDetailSnapshot(
                status: status,
                admins: info.admins,
                threshold: info.threshold,
                balanceYuan: previousDetail?.balanceYuan,
                lastBalanceRefreshAtMillis:
                    previousDetail?.lastBalanceRefreshAtMillis,
                updatedAtMillis: DateTime.now().millisecondsSinceEpoch,
                lastChainRefreshAtMillis: DateTime.now().millisecondsSinceEpoch,
              ),
            );
          }
        });
      } catch (_) {
        // 中文注释：网络/轻节点异常不改本地状态，避免把查询失败误判为注销。
      }
    }
  }

  Future<({bool anyChanged, bool completed})> _runBackgroundDiscovery({
    bool force = false,
  }) async {
    if (_scanning) return (anyChanged: false, completed: false);
    setState(() {
      _scanning = true;
      _scanProgress = '扫描中...';
    });
    var anyChanged = false;
    var completed = false;
    try {
      // 机构多签 + 个人多签共用一次 AdminAccounts 扫描(ADR-018 §九),
      // 由协调器分发到两类后处理,避免同一张表各扫一遍。
      final result = await MultisigDiscoveryCoordinator().discoverAll(
        force: force,
        onProgress: (s, t, m) {
          if (mounted) {
            setState(() {
              _scanProgress = '多签扫描 $s${t == null ? '' : '/$t'} · 已解码 $m';
            });
          }
        },
      );
      anyChanged = result.anyChanged;
      completed = result.completed;
    } catch (e) {
      debugPrint('[DuoqianListPage] discovery 失败: $e');
    } finally {
      if (anyChanged) {
        await _readFromIsar();
      }
      if (mounted) {
        setState(() {
          _scanning = false;
          _scanProgress = null;
        });
      }
    }
    return (anyChanged: anyChanged, completed: completed);
  }

  Future<void> _runDiscoveryIfWalletsChanged() async {
    final fingerprint = await _currentWalletFingerprint();
    final lastFingerprint = await _readDiscoveryWalletFingerprint();
    if (lastFingerprint == fingerprint) return;
    final result = await _runBackgroundDiscovery(force: true);
    if (result.completed) {
      await _writeDiscoveryWalletFingerprint(fingerprint);
    }
  }

  Future<void> _onPullRefresh() async {
    await _refreshKnownStatuses(force: true);
    final result = await _runBackgroundDiscovery(force: true);
    if (result.completed) {
      await _writeDiscoveryWalletFingerprint(await _currentWalletFingerprint());
    }
    await _readFromIsar();
  }

  Future<void> _openCreateMenu() async {
    final choice = await showModalBottomSheet<_AccountKind>(
      context: context,
      backgroundColor: Colors.white,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(16)),
      ),
      builder: (sheetCtx) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.person_outline, color: AppTheme.accent),
              title: const Text('新增个人多签'),
              subtitle: const Text('无需身份ID'),
              onTap: () => Navigator.pop(sheetCtx, _AccountKind.personal),
            ),
            const Divider(height: 1),
            ListTile(
              leading: const Icon(Icons.account_balance_outlined,
                  color: AppTheme.info),
              title: const Text('新增机构多签'),
              subtitle: const Text('需要身份ID'),
              onTap: () => Navigator.pop(sheetCtx, _AccountKind.institution),
            ),
          ],
        ),
      ),
    );
    if (!mounted || choice == null) return;
    switch (choice) {
      case _AccountKind.personal:
        await _openCreatePersonal();
      case _AccountKind.institution:
        await _openCreateInstitution();
    }
  }

  Future<void> _openCreatePersonal() async {
    final createdAddress = await Navigator.push<String>(
      context,
      MaterialPageRoute(builder: (_) => const PersonalAccountCreatePage()),
    );
    if (createdAddress != null) {
      await _refreshKnownStatuses(
        force: true,
        personalAccounts: {createdAddress},
      );
    }
  }

  Future<void> _openCreateInstitution() async {
    final wallets = await WalletManager().getWallets();
    if (!mounted) return;
    if (wallets.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请先导入钱包')),
      );
      return;
    }
    final createdAddress = await Navigator.push<String>(
      context,
      MaterialPageRoute(
        builder: (_) => InstitutionDuoqianCreatePage(
          institution: const InstitutionInfo(
            cidFullName: '新建多签机构',
            cidShortName: '新建多签机构',
            cidNumber:
                'institution-account:0000000000000000000000000000000000000000000000000000000000000000',
            orgType: OrgType.account,
            // 设计缺口: 新建机构时尚无 CID 码，占位用空字符串；实际 CID 码在创建完成后
// 由 institution_manage_service 从链上读取并写入 Isar。
            adminAccountCode: '',
            account:
                '0000000000000000000000000000000000000000000000000000000000000000',
          ),
          adminWallets: wallets,
        ),
      ),
    );
    if (createdAddress != null) {
      await _refreshKnownStatuses(
        force: true,
        institutionAccounts: {createdAddress},
      );
    }
  }

  void _onCardTap(_UnifiedItem item) {
    final route = switch (item.kind) {
      _AccountKind.personal => MaterialPageRoute(
          builder: (_) => PersonalManageAccountInfoPage(
            institution: InstitutionInfo(
              cidFullName: item.displayLabel,
              cidShortName: item.displayLabel,
              cidNumber: 'personal-account:${item.account}',
              orgType: OrgType.account,
              account: item.account,
            ),
            initialLocalStatus: item.localStatus,
            initialAdminPubkeys:
                item.personal?.matchedAdminPubkeys ?? const <String>[],
          ),
        ),
      _AccountKind.institution => MaterialPageRoute(
          builder: (_) => InstitutionAccountInfoPage(
            institution: InstitutionInfo(
              cidFullName: item.displayLabel,
              cidShortName: item.displayLabel,
              cidNumber: registeredAccountIdentity(item.account),
              orgType: OrgType.account,
              adminAccountCode: item.institution?.adminAccountCode,
              account: item.account,
            ),
            initialLocalStatus: item.localStatus,
            initialAdminPubkeys:
                item.institution?.matchedAdminPubkeys ?? const <String>[],
          ),
        ),
    };
    Navigator.push(context, route).then((_) {
      if (!mounted) return;
      switch (item.kind) {
        case _AccountKind.personal:
          unawaited(_refreshKnownStatuses(
            force: true,
            personalAccounts: {item.account},
          ));
        case _AccountKind.institution:
          unawaited(_refreshKnownStatuses(
            force: true,
            institutionAccounts: {item.account},
          ));
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '多签',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: AppTheme.primaryDark,
        elevation: 0,
        scrolledUnderElevation: 0.5,
        actions: [
          IconButton(
            icon: const Icon(Icons.add),
            tooltip: '新增多签',
            onPressed: _openCreateMenu,
          ),
        ],
      ),
      body: Column(
        children: [
          if (_scanning && _scanProgress != null) _buildScanBanner(),
          Expanded(
            child: _loading
                ? const Center(child: CircularProgressIndicator())
                : _items.isEmpty
                    ? _buildEmpty()
                    : _buildList(),
          ),
        ],
      ),
    );
  }

  Widget _buildScanBanner() {
    return Container(
      width: double.infinity,
      color: AppTheme.primaryDark.withValues(alpha: 0.06),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Row(
        children: [
          const SizedBox(
            width: 12,
            height: 12,
            child: CircularProgressIndicator(strokeWidth: 1.5),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              _scanProgress!,
              style: const TextStyle(
                fontSize: 12,
                color: AppTheme.textSecondary,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEmpty() {
    return RefreshIndicator(
      onRefresh: _onPullRefresh,
      child: ListView(
        children: const [
          SizedBox(height: 80),
          Icon(Icons.account_tree_outlined, size: 64, color: AppTheme.border),
          SizedBox(height: 12),
          Center(
            child: Text(
              '暂无多签账户',
              style: TextStyle(fontSize: 16, color: AppTheme.textTertiary),
            ),
          ),
          SizedBox(height: 6),
          Center(
            child: Text(
              '点击右上角 + 新增个人多签或机构多签;\n你作为管理员参与的多签会自动出现在此',
              textAlign: TextAlign.center,
              style: TextStyle(fontSize: 13, color: AppTheme.textTertiary),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildList() {
    return RefreshIndicator(
      onRefresh: _onPullRefresh,
      child: ListView.separated(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        itemCount: _items.length,
        separatorBuilder: (_, __) => const SizedBox(height: 8),
        itemBuilder: (_, index) => _buildCard(_items[index]),
      ),
    );
  }

  Widget _buildCard(_UnifiedItem item) {
    final ss58 = _hexToSs58(item.account);
    final isPersonal = item.kind == _AccountKind.personal;
    final color = isPersonal ? AppTheme.accent : AppTheme.info;
    final iconData = isPersonal ? Icons.person : Icons.business;
    final tag = isPersonal ? '个人' : '机构';
    final isClosed =
        item.localStatus == PersonalAccountLocalState.statusClosed ||
            item.localStatus == InstitutionDuoqianLocalState.statusClosed;
    final subtitleParts = <String>[
      _truncateAddress(ss58),
      if (item.discoveredViaAdmin) '我作为 ${item.matchedAdminsLen} 位管理员之一参与',
    ];
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: color.withValues(alpha: 0.15)),
      ),
      child: InkWell(
        onTap: () => _onCardTap(item),
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
          child: Row(
            children: [
              Container(
                width: 40,
                height: 40,
                decoration: BoxDecoration(
                  color: color.withValues(alpha: 0.08),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(iconData, size: 20, color: color),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Container(
                          padding: const EdgeInsets.symmetric(
                              horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: color.withValues(alpha: 0.12),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            tag,
                            style: TextStyle(
                              fontSize: 11,
                              fontWeight: FontWeight.w600,
                              color: color,
                            ),
                          ),
                        ),
                        if (isClosed) ...[
                          const SizedBox(width: 6),
                          Container(
                            padding: const EdgeInsets.symmetric(
                                horizontal: 6, vertical: 2),
                            decoration: BoxDecoration(
                              color:
                                  AppTheme.textTertiary.withValues(alpha: 0.12),
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: const Text(
                              '已注销',
                              style: TextStyle(
                                fontSize: 11,
                                fontWeight: FontWeight.w600,
                                color: AppTheme.textTertiary,
                              ),
                            ),
                          ),
                        ],
                        const SizedBox(width: 6),
                        Expanded(
                          child: Text(
                            item.displayLabel,
                            style: const TextStyle(
                              fontSize: 15,
                              fontWeight: FontWeight.w600,
                              color: AppTheme.primaryDark,
                            ),
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 2),
                    Text(
                      subtitleParts.join(' · '),
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
    );
  }

  String _hexToSs58(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final bytes = Uint8List(h.length ~/ 2);
    for (var i = 0; i < bytes.length; i++) {
      bytes[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return Keyring().encodeAddress(bytes, 2027);
  }

  String _truncateAddress(String address) {
    if (address.length <= 14) return address;
    return '${address.substring(0, 6)}...${address.substring(address.length - 6)}';
  }

  String _normalizeHex(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    return h.toLowerCase();
  }

  Future<String> _currentWalletFingerprint() async {
    final wallets = await WalletManager().getWallets();
    final pubkeys = wallets
        .map((wallet) => _normalizeHex(wallet.pubkeyHex))
        .toList()
      ..sort();
    return pubkeys.join('|');
  }

  Future<String?> _readDiscoveryWalletFingerprint() {
    return WalletIsar.instance.read((isar) async {
      return (await isar.appKvEntitys.getByKey(_discoveryWalletFingerprintKey))
          ?.stringValue;
    });
  }

  Future<void> _writeDiscoveryWalletFingerprint(String fingerprint) {
    return WalletIsar.instance.writeTxn((isar) async {
      final entity = await isar.appKvEntitys.getByKey(
            _discoveryWalletFingerprintKey,
          ) ??
          AppKvEntity();
      entity
        ..key = _discoveryWalletFingerprintKey
        ..stringValue = fingerprint
        ..intValue = DateTime.now().millisecondsSinceEpoch;
      await isar.appKvEntitys.putByKey(entity);
    });
  }
}
