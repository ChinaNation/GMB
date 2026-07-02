// 个人多签账户列表页。
//
// 设计边界：
// - 本页只读取、发现、刷新和展示个人多签账户。
// - 机构账户由 OnChina 注册局登记，CitizenApp 这里不再发现或展示机构多签。
// - 入口放在交易 tab，避免把个人自助多签与机构登记流程混在一起。

import 'dart:async' show unawaited;
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:isar_community/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:citizenapp/citizen/shared/admin_accounts_scan_service.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/isar/wallet_isar.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import 'personal_account_create_page.dart';
import 'personal_manage_account_info_page.dart';
import 'personal_manage_discovery_service.dart';
import 'personal_manage_models.dart';
import 'personal_manage_service.dart';
import 'personal_proposal_history_service.dart';

class PersonalAccountListPage extends StatefulWidget {
  const PersonalAccountListPage({super.key});

  @override
  State<PersonalAccountListPage> createState() =>
      _PersonalAccountListPageState();
}

class _PersonalAccountListPageState extends State<PersonalAccountListPage> {
  final PersonalManageService _personalManageService = PersonalManageService();
  final PersonalProposalHistoryService _personalProposalHistoryService =
      PersonalProposalHistoryService();
  final AdminAccountsScanService _scanService = AdminAccountsScanService();
  final PersonalManageDiscoveryService _discoveryService =
      PersonalManageDiscoveryService();

  List<PersonalAccountEntity> _items = [];
  Map<String, String?> _statuses = const {};
  bool _loading = true;
  bool _scanning = false;
  String? _scanProgress;

  static const _activeStatusTtl = Duration(minutes: 60);
  static const _inactiveStatusTtl = Duration(minutes: 10);

  // 中文注释：使用新的 fingerprint key，避免旧“个人+机构”扫描记录让个人列表跳过首轮发现。
  static const _discoveryWalletFingerprintKey =
      'personal_multisig_discovery_wallet_fingerprint';

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
      // 中文注释：本地库异常不阻塞页面进入，用户仍可通过下拉刷新重试。
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
      final statuses = await PersonalAccountLocalState.readStatuses(
        isar,
        personals.map((p) => p.account),
      );
      return (personals: personals, statuses: statuses);
    });

    final sorted = [...snapshot.personals]
      ..sort((a, b) => b.addedAtMillis.compareTo(a.addedAtMillis));
    if (!mounted) return;
    setState(() {
      _items = sorted;
      _statuses = snapshot.statuses;
    });
  }

  Future<void> _runBackgroundRefresh() async {
    await _refreshKnownStatuses();
    await _runDiscoveryIfWalletsChanged();
  }

  Future<void> _refreshKnownStatuses({
    bool force = false,
    Set<String>? personalAccounts,
  }) async {
    final snapshot = await WalletIsar.instance.read((isar) async {
      final personals = await isar.personalAccountEntitys.where().findAll();
      final statuses = await PersonalAccountLocalState.readStatusSnapshots(
        isar,
        personals.map((p) => p.account),
      );
      return (personals: personals, statuses: statuses);
    });

    final filter = personalAccounts?.map(_normalizeHex).toSet();
    final targets = snapshot.personals.where((item) {
      final address = _normalizeHex(item.account);
      if (filter != null && !filter.contains(address)) return false;
      return force || _shouldRefreshStatus(snapshot.statuses[address]);
    }).toList(growable: false);

    if (targets.isEmpty) return;
    await _syncPersonalStatuses(targets);
    await _readFromIsar();
  }

  bool _shouldRefreshStatus(MultisigLocalStatusSnapshot? snapshot) {
    if (snapshot?.lastSyncAtMillis == null) return true;
    final lastSyncAt = DateTime.fromMillisecondsSinceEpoch(
      snapshot!.lastSyncAtMillis!,
    );
    final ttl = snapshot.status == PersonalAccountLocalState.statusActive
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
            : info.status == MultisigStatus.active
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
              MultisigLocalDetailSnapshot(
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
        // 中文注释：单个账户刷新失败只跳过该账户，避免影响整页列表。
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

  Future<({bool anyChanged, bool completed})> _runBackgroundDiscovery() async {
    if (_scanning) return (anyChanged: false, completed: false);

    final myPubkeys = await _currentWalletPubkeys();
    if (myPubkeys.isEmpty) return (anyChanged: false, completed: true);

    setState(() {
      _scanning = true;
      _scanProgress = '扫描个人多签中...';
    });

    var anyChanged = false;
    var completed = false;
    try {
      // 中文注释：这里只做个人多签发现，扫描结果直接交给个人多签服务处理。
      final scan = await _scanService.scanAll(
        onProgress: (scanned, total, decoded) {
          if (!mounted) return;
          setState(() {
            _scanProgress =
                '个人多签扫描 $scanned${total == null ? '' : '/$total'} · 已解码 $decoded';
          });
        },
      );
      final stats = await _discoveryService.processScanned(
        scan,
        myPubkeys: myPubkeys,
      );
      anyChanged = stats.newlyAdded > 0 || stats.orphansRemoved > 0;
      completed = !stats.partialFailure;
    } catch (e) {
      debugPrint('[PersonalAccountListPage] discovery 失败: $e');
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
    if (fingerprint.isEmpty) return;
    final lastFingerprint = await _readDiscoveryWalletFingerprint();
    if (lastFingerprint == fingerprint) return;
    final result = await _runBackgroundDiscovery();
    if (result.completed) {
      await _writeDiscoveryWalletFingerprint(fingerprint);
    }
  }

  Future<void> _onPullRefresh() async {
    await _refreshKnownStatuses(force: true);
    final result = await _runBackgroundDiscovery();
    if (result.completed) {
      await _writeDiscoveryWalletFingerprint(await _currentWalletFingerprint());
    }
    await _readFromIsar();
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

  void _onCardTap(PersonalAccountEntity item) {
    final localStatus = _statuses[_normalizeHex(item.account)];
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (_) => PersonalManageAccountInfoPage(
          institution: InstitutionInfo(
            cidFullName: item.accountName,
            cidShortName: item.accountName,
            cidFullNameEn: 'Personal Multisig ${item.account.substring(0, 8)}',
            cidShortNameEn: 'Personal Multisig ${item.account.substring(0, 8)}',
            cidNumber: 'personal-account:${item.account}',
            orgType: OrgType.account,
            account: item.account,
          ),
          initialLocalStatus: localStatus,
          initialAdminPubkeys: item.matchedAdminPubkeys,
        ),
      ),
    ).then((_) {
      if (!mounted) return;
      unawaited(_refreshKnownStatuses(
        force: true,
        personalAccounts: {item.account},
      ));
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '多签账户',
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
            tooltip: '新增个人多签',
            onPressed: _openCreatePersonal,
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
              '点击右上角 + 新增个人多签;\n你作为管理员参与的个人多签会自动出现在此',
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

  Widget _buildCard(PersonalAccountEntity item) {
    final ss58 = _accountAddressLabel(item.account);
    final localStatus = _statuses[_normalizeHex(item.account)];
    final isClosed = localStatus == PersonalAccountLocalState.statusClosed;
    final subtitleParts = <String>[
      _truncateAddress(ss58),
      if (item.discoveredViaAdmin)
        '我作为 ${item.matchedAdminPubkeys.length} 位管理员之一参与',
    ];
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: AppTheme.accent.withValues(alpha: 0.15)),
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
                  color: AppTheme.accent.withValues(alpha: 0.08),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: const Icon(
                  Icons.person,
                  size: 20,
                  color: AppTheme.accent,
                ),
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
                            color: AppTheme.accent.withValues(alpha: 0.12),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: const Text(
                            '个人',
                            style: TextStyle(
                              fontSize: 11,
                              fontWeight: FontWeight.w600,
                              color: AppTheme.accent,
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
                            item.accountName,
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

  String _accountAddressLabel(String hex) {
    try {
      return _hexToSs58(hex);
    } catch (_) {
      return hex;
    }
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

  Future<Set<String>> _currentWalletPubkeys() async {
    final wallets = await WalletManager().getWallets();
    return wallets.map((wallet) => _normalizeHex(wallet.pubkeyHex)).toSet();
  }

  Future<String> _currentWalletFingerprint() async {
    final pubkeys = (await _currentWalletPubkeys()).toList()..sort();
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
