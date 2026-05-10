// 多签交易统一账户列表页(个人 + 机构 混合视图)。
//
// 设计要点:
// - 后端按 governance/personal-manage 与 governance/organization-manage 分开;
//   本页只是 UI 编排壳子,并行加载两套数据源,合并按时间倒序展示。
// - 启动期/进入页面时并行触发两个 discovery 服务,30 分钟内重复进入不再扫;
//   下拉刷新强制全扫。
// - 反向校验由各自 discovery service 内部完成,本页不涉及。
// - 右上角 "+" 弹 ActionSheet,2 选项分别进入个人多签/机构多签创建页。

import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:isar/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:wuminapp_mobile/common/institution_info.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import 'organization-manage/duoqian_account_info_page.dart';
import 'organization-manage/duoqian_discovery_service.dart';
import 'organization-manage/institution_duoqian_create_page.dart';
import 'personal-manage/personal_duoqian_create_page.dart';
import 'personal-manage/personal_manage_account_info_page.dart';
import 'personal-manage/personal_manage_discovery_service.dart';

enum _DuoqianKind { personal, institution }

class _UnifiedItem {
  _UnifiedItem.personal(PersonalDuoqianEntity item)
      : kind = _DuoqianKind.personal,
        name = item.name,
        duoqianAddress = item.duoqianAddress,
        addedAtMillis = item.addedAtMillis,
        discoveredViaAdmin = item.discoveredViaAdmin,
        matchedAdminCount = item.matchedAdminPubkeys.length,
        personal = item,
        institution = null;

  _UnifiedItem.institution(DuoqianInstitutionEntity item)
      : kind = _DuoqianKind.institution,
        name = item.name,
        duoqianAddress = item.duoqianAddress,
        addedAtMillis = item.addedAtMillis,
        discoveredViaAdmin = item.discoveredViaAdmin,
        matchedAdminCount = item.matchedAdminPubkeys.length,
        personal = null,
        institution = item;

  final _DuoqianKind kind;
  final String name;
  final String duoqianAddress;
  final int addedAtMillis;
  final bool discoveredViaAdmin;
  final int matchedAdminCount;
  final PersonalDuoqianEntity? personal;
  final DuoqianInstitutionEntity? institution;
}

class DuoqianAccountListPage extends StatefulWidget {
  const DuoqianAccountListPage({super.key});

  @override
  State<DuoqianAccountListPage> createState() => _DuoqianAccountListPageState();
}

class _DuoqianAccountListPageState extends State<DuoqianAccountListPage> {
  List<_UnifiedItem> _items = [];
  bool _loading = true;
  bool _scanning = false;
  String? _scanProgress;

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
      // ignore: discarded_futures
      _runBackgroundDiscovery();
    }
  }

  Future<void> _readFromIsar() async {
    final isar = await WalletIsar.instance.db();
    final personals = await isar.personalDuoqianEntitys.where().findAll();
    final institutions = await isar.duoqianInstitutionEntitys.where().findAll();
    final merged = <_UnifiedItem>[
      ...personals.map(_UnifiedItem.personal),
      ...institutions.map(_UnifiedItem.institution),
    ]..sort((a, b) => b.addedAtMillis.compareTo(a.addedAtMillis));
    if (!mounted) return;
    setState(() => _items = merged);
  }

  Future<void> _runBackgroundDiscovery({bool force = false}) async {
    if (_scanning) return;
    setState(() {
      _scanning = true;
      _scanProgress = '扫描中...';
    });
    var anyChanged = false;
    try {
      final personalFuture =
          PersonalManageDiscoveryService().discoverByMyWallets(
        force: force,
        onProgress: (s, t, m) {
          if (mounted) {
            setState(() {
              _scanProgress = '个人多签扫描 $s${t == null ? '' : '/$t'} · 已发现 $m';
            });
          }
        },
      );
      final institutionFuture = DuoqianDiscoveryService().discoverByMyWallets(
        force: force,
        onProgress: (s, t, m) {
          if (mounted) {
            setState(() {
              _scanProgress = '机构多签扫描 $s${t == null ? '' : '/$t'} · 已发现 $m';
            });
          }
        },
      );
      final personalStats = await personalFuture;
      final institutionStats = await institutionFuture;
      anyChanged = personalStats.newlyAdded > 0 ||
          personalStats.orphansRemoved > 0 ||
          institutionStats.newlyAdded > 0 ||
          institutionStats.orphansRemoved > 0;
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
  }

  Future<void> _onPullRefresh() async {
    await _readFromIsar();
    await _runBackgroundDiscovery(force: true);
  }

  Future<void> _openCreateMenu() async {
    final choice = await showModalBottomSheet<_DuoqianKind>(
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
              onTap: () => Navigator.pop(sheetCtx, _DuoqianKind.personal),
            ),
            const Divider(height: 1),
            ListTile(
              leading:
                  const Icon(Icons.account_tree_outlined, color: AppTheme.info),
              title: const Text('新增机构多签'),
              onTap: () => Navigator.pop(sheetCtx, _DuoqianKind.institution),
            ),
          ],
        ),
      ),
    );
    if (!mounted || choice == null) return;
    switch (choice) {
      case _DuoqianKind.personal:
        await _openCreatePersonal();
      case _DuoqianKind.institution:
        await _openCreateInstitution();
    }
  }

  Future<void> _openCreatePersonal() async {
    final created = await Navigator.push<bool>(
      context,
      MaterialPageRoute(builder: (_) => const PersonalDuoqianCreatePage()),
    );
    if (created == true) await _load();
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
    final created = await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => InstitutionDuoqianCreatePage(
          institution: const InstitutionInfo(
            name: '新建多签机构',
            sfidNumber:
                'duoqian:0000000000000000000000000000000000000000000000000000000000000000',
            orgType: OrgType.duoqian,
            adminSubjectOrg: 5,
            duoqianAddress:
                '0000000000000000000000000000000000000000000000000000000000000000',
          ),
          adminWallets: wallets,
        ),
      ),
    );
    if (created == true) await _load();
  }

  void _onCardTap(_UnifiedItem item) {
    final route = switch (item.kind) {
      _DuoqianKind.personal => MaterialPageRoute(
          builder: (_) => PersonalManageAccountInfoPage(
            institution: InstitutionInfo(
              name: item.name,
              sfidNumber: 'personal:${item.duoqianAddress}',
              orgType: OrgType.duoqian,
              duoqianAddress: item.duoqianAddress,
            ),
          ),
        ),
      _DuoqianKind.institution => MaterialPageRoute(
          builder: (_) => DuoqianAccountInfoPage(
            institution: InstitutionInfo(
              name: item.name,
              sfidNumber: registeredDuoqianIdentity(item.duoqianAddress),
              orgType: OrgType.duoqian,
              adminSubjectOrg: item.institution?.adminSubjectOrg,
              duoqianAddress: item.duoqianAddress,
            ),
          ),
        ),
    };
    Navigator.push(context, route).then((_) {
      if (mounted) _load(runDiscovery: false);
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '多签交易',
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
    final ss58 = _hexToSs58(item.duoqianAddress);
    final isPersonal = item.kind == _DuoqianKind.personal;
    final color = isPersonal ? AppTheme.accent : AppTheme.info;
    final iconData = isPersonal ? Icons.person : Icons.business;
    final tag = isPersonal ? '个人' : '机构';
    final subtitleParts = <String>[
      _truncateAddress(ss58),
      if (item.discoveredViaAdmin) '我作为 ${item.matchedAdminCount} 位管理员之一参与',
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
                        const SizedBox(width: 6),
                        Expanded(
                          child: Text(
                            item.name,
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
}
