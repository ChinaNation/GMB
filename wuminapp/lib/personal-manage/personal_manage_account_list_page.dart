// 个人多签账户列表页。
//
// 只读取 PersonalDuoqianEntity，只触发 PersonalManage 个人多签发现和创建；
// 机构多签列表继续留在 duoqian 目录，二者不再共用主业务入口。

import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:isar/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/institution/institution_data.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';

import 'personal_duoqian_create_page.dart';
import 'personal_manage_account_info_page.dart';
import 'personal_manage_discovery_service.dart';

class PersonalManageAccountListPage extends StatefulWidget {
  const PersonalManageAccountListPage({super.key});

  @override
  State<PersonalManageAccountListPage> createState() =>
      _PersonalManageAccountListPageState();
}

class _PersonalManageAccountListPageState
    extends State<PersonalManageAccountListPage> {
  List<PersonalDuoqianEntity> _items = [];
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
      // 中文注释：本地库异常不阻塞页面进入，空态可继续触发手动刷新。
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
    final items = await isar.personalDuoqianEntitys.where().findAll()
      ..sort((a, b) => b.addedAtMillis.compareTo(a.addedAtMillis));
    if (!mounted) return;
    setState(() => _items = items);
  }

  Future<void> _runBackgroundDiscovery({bool force = false}) async {
    if (_scanning) return;
    setState(() {
      _scanning = true;
      _scanProgress = '扫描中...';
    });
    try {
      final stats = await PersonalManageDiscoveryService().discoverByMyWallets(
        force: force,
        onProgress: (s, t, m) {
          if (mounted) {
            setState(() {
              _scanProgress = '扫描中 $s${t == null ? '' : '/$t'} · 已发现 $m';
            });
          }
        },
      );
      if (stats.newlyAdded > 0 || stats.orphansRemoved > 0) {
        await _readFromIsar();
      }
    } catch (e) {
      debugPrint('[PersonalManageListPage] discovery 失败: $e');
    } finally {
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

  Future<void> _openCreatePersonal() async {
    final created = await Navigator.push<bool>(
      context,
      MaterialPageRoute(builder: (_) => const PersonalDuoqianCreatePage()),
    );
    if (created == true) await _load();
  }

  void _onCardTap(PersonalDuoqianEntity item) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (_) => PersonalManageAccountInfoPage(
          institution: InstitutionInfo(
            name: item.name,
            sfidNumber: 'personal:${item.duoqianAddress}',
            orgType: OrgType.duoqian,
            duoqianAddress: item.duoqianAddress,
          ),
        ),
      ),
    ).then((_) {
      if (mounted) _load(runDiscovery: false);
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '个人多签',
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
            tooltip: '创建个人多签',
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
          Icon(Icons.person_outline, size: 64, color: AppTheme.border),
          SizedBox(height: 12),
          Center(
            child: Text(
              '暂无个人多签',
              style: TextStyle(fontSize: 16, color: AppTheme.textTertiary),
            ),
          ),
          SizedBox(height: 6),
          Center(
            child: Text(
              '点击右上角 + 创建个人多签;\n你作为管理员参与的个人多签会自动出现在此',
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

  Widget _buildCard(PersonalDuoqianEntity item) {
    final ss58 = _hexToSs58(item.duoqianAddress);
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
                    Text(
                      item.name,
                      style: const TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.primaryDark,
                      ),
                      overflow: TextOverflow.ellipsis,
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
