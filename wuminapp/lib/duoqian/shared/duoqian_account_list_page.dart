import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:isar/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../../Isar/wallet_isar.dart';
import 'package:wuminapp_mobile/citizen/institution/institution_data.dart';
import '../../qr/bodies/user_duoqian_body.dart';
import '../../qr/envelope.dart';
import '../../qr/pages/qr_scan_page.dart' show QrScanMode, QrScanPage;
import '../../qr/qr_protocols.dart';
import '../../ui/app_theme.dart';
import '../../wallet/core/wallet_manager.dart';
import '../institution/institution_duoqian_create_page.dart';
import '../personal/personal_duoqian_create_page.dart';
import 'duoqian_account_info_page.dart';
import 'duoqian_account_type.dart';
import 'duoqian_manage_models.dart';
import 'duoqian_manage_service.dart';

/// 单类型多签账户列表页。
///
/// 交易页会分别进入机构多签与个人多签，本页只展示一种账户类型；
/// 扫码加入时也按当前入口写入对应本地表，保持手机端心智一致。
class DuoqianAccountListPage extends StatefulWidget {
  const DuoqianAccountListPage({
    super.key,
    required this.accountType,
  });

  final DuoqianAccountType accountType;

  @override
  State<DuoqianAccountListPage> createState() => _DuoqianAccountListPageState();
}

class _DuoqianAccountListPageState extends State<DuoqianAccountListPage> {
  List<_DuoqianListItem> _items = [];
  bool _loading = true;

  bool get _isPersonal => widget.accountType == DuoqianAccountType.personal;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() => _loading = true);
    try {
      final isar = await WalletIsar.instance.db();
      final items = <_DuoqianListItem>[];

      if (_isPersonal) {
        final personals = await isar.personalDuoqianEntitys.where().findAll();
        items.addAll(personals.map((e) => _DuoqianListItem(
              duoqianAddress: e.duoqianAddress,
              name: e.name,
              addedAtMillis: e.addedAtMillis,
            )));
      } else {
        final institutions =
            await isar.duoqianInstitutionEntitys.where().findAll();
        items.addAll(institutions.map((e) => _DuoqianListItem(
              duoqianAddress: e.duoqianAddress,
              name: e.name,
              addedAtMillis: e.addedAtMillis,
            )));
      }

      items.sort((a, b) => b.addedAtMillis.compareTo(a.addedAtMillis));
      if (!mounted) return;
      setState(() {
        _items = items;
        _loading = false;
      });
    } catch (_) {
      if (!mounted) return;
      setState(() => _loading = false);
    }
  }

  void _showCreateMenu() {
    showModalBottomSheet(
      context: context,
      builder: (ctx) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: Icon(
                _isPersonal ? Icons.person : Icons.business,
                color: _isPersonal ? AppTheme.accent : AppTheme.info,
              ),
              title: Text(widget.accountType.createTitle),
              subtitle: Text(_isPersonal ? '无需 SFID，直接设置管理员' : '需要 SFID 机构标识'),
              onTap: () {
                Navigator.pop(ctx);
                if (_isPersonal) {
                  _openCreatePersonal();
                } else {
                  _openCreateInstitution();
                }
              },
            ),
            const Divider(height: 1),
            ListTile(
              leading: const Icon(
                Icons.qr_code_scanner,
                color: AppTheme.primaryDark,
              ),
              title: const Text('扫码加入多签账户'),
              subtitle: const Text('扫描多签账户二维码加入当前列表'),
              onTap: () {
                Navigator.pop(ctx);
                _scanJoinDuoqian();
              },
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _openCreateInstitution() async {
    final wallets = await _getWallets();
    if (!mounted || wallets.isEmpty) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('请先导入钱包')),
        );
      }
      return;
    }

    final created = await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => InstitutionDuoqianCreatePage(
          institution: const InstitutionInfo(
            name: '新建多签机构',
            shenfenId:
                'duoqian:0000000000000000000000000000000000000000000000000000000000000000',
            orgType: OrgType.duoqian,
            duoqianAddress:
                '0000000000000000000000000000000000000000000000000000000000000000',
          ),
          adminWallets: wallets,
        ),
      ),
    );
    if (created == true) await _load();
  }

  Future<void> _openCreatePersonal() async {
    final created = await Navigator.push<bool>(
      context,
      MaterialPageRoute(builder: (_) => const PersonalDuoqianCreatePage()),
    );
    if (created == true) await _load();
  }

  Future<void> _scanJoinDuoqian() async {
    final result = await Navigator.push<String>(
      context,
      MaterialPageRoute(
        builder: (_) => QrScanPage(
          mode: QrScanMode.raw,
          customTitle: '扫码加入${widget.accountType.title}',
        ),
      ),
    );
    if (result == null || !mounted) return;

    UserDuoqianBody qrBody;
    try {
      final env = QrEnvelope.parse(result.trim());
      if (env.kind != QrKind.userDuoqian) {
        throw const FormatException('不是多签账户码');
      }
      qrBody = env.body as UserDuoqianBody;
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('请扫描多签账户二维码：$e'),
          backgroundColor: AppTheme.danger,
        ),
      );
      return;
    }

    String hexAddress;
    try {
      final pubkey = Keyring().decodeAddress(qrBody.address);
      hexAddress = _toHex(pubkey);
    } catch (_) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
            content: Text('多签地址无效'), backgroundColor: AppTheme.danger),
      );
      return;
    }

    final isar = await WalletIsar.instance.db();
    final existsPersonal = await isar.personalDuoqianEntitys
        .filter()
        .duoqianAddressEqualTo(hexAddress)
        .findFirst();
    final existsInstitution = await isar.duoqianInstitutionEntitys
        .filter()
        .duoqianAddressEqualTo(hexAddress)
        .findFirst();
    if (existsPersonal != null || existsInstitution != null) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('该多签账户已在列表中')),
      );
      return;
    }

    final manageService = DuoqianManageService();
    final accountInfo = await manageService.fetchDuoqianAccount(hexAddress);
    if (accountInfo == null) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('链上未找到该多签账户'),
          backgroundColor: AppTheme.danger,
        ),
      );
      return;
    }

    final wallets = await _getWallets();
    WalletProfile? matchedWallet;
    for (final w in wallets) {
      var pk = w.pubkeyHex.toLowerCase();
      if (pk.startsWith('0x')) pk = pk.substring(2);
      if (accountInfo.adminPubkeys.contains(pk)) {
        matchedWallet = w;
        break;
      }
    }
    if (matchedWallet == null) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('你不是该多签账户的管理员'),
          backgroundColor: AppTheme.danger,
        ),
      );
      return;
    }

    await _saveDuoqianToLocal(hexAddress, qrBody.name);
    if (!mounted) return;
    final statusText = accountInfo.status == DuoqianStatus.active
        ? '已加入${widget.accountType.title}「${qrBody.name}」'
        : '已加入「${qrBody.name}」，该账户待投票激活';
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(statusText),
        backgroundColor: accountInfo.status == DuoqianStatus.active
            ? AppTheme.success
            : AppTheme.warning,
      ),
    );
    await _load();
  }

  Future<void> _saveDuoqianToLocal(String hexAddress, String name) async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      if (_isPersonal) {
        final entity = PersonalDuoqianEntity()
          ..duoqianAddress = hexAddress
          ..name = name
          ..creatorAddress = ''
          ..addedAtMillis = DateTime.now().millisecondsSinceEpoch;
        await isar.personalDuoqianEntitys.put(entity);
        return;
      }

      // 中文注释：多签账户 QR 本身不携带机构/个人类型，机构入口扫码加入时
      // 按当前入口归入机构多签，本地 sfidId 用注册多签 identity 占位。
      final entity = DuoqianInstitutionEntity()
        ..duoqianAddress = hexAddress
        ..sfidId = registeredDuoqianIdentity(hexAddress)
        ..name = name
        ..addedAtMillis = DateTime.now().millisecondsSinceEpoch;
      await isar.duoqianInstitutionEntitys.put(entity);
    });
  }

  Future<List<WalletProfile>> _getWallets() async {
    final wm = WalletManager();
    return wm.getWallets();
  }

  void _onCardTap(_DuoqianListItem item) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (_) => DuoqianAccountInfoPage(
          institution: _itemToInstitutionInfo(item),
          isPersonal: _isPersonal,
        ),
      ),
    ).then((_) {
      if (mounted) _load();
    });
  }

  InstitutionInfo _itemToInstitutionInfo(_DuoqianListItem item) {
    return InstitutionInfo(
      name: item.name,
      shenfenId: _isPersonal
          ? 'personal:${item.duoqianAddress}'
          : registeredDuoqianIdentity(item.duoqianAddress),
      orgType: OrgType.duoqian,
      duoqianAddress: item.duoqianAddress,
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: Text(
          widget.accountType.title,
          style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: AppTheme.primaryDark,
        elevation: 0,
        scrolledUnderElevation: 0.5,
        actions: [
          IconButton(
            icon: const Icon(Icons.add),
            onPressed: _showCreateMenu,
          ),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : _items.isEmpty
              ? _buildEmpty()
              : _buildList(),
    );
  }

  Widget _buildEmpty() {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            _isPersonal ? Icons.person_outline : Icons.business_outlined,
            size: 64,
            color: AppTheme.border,
          ),
          const SizedBox(height: 12),
          Text(
            widget.accountType.emptyTitle,
            style: const TextStyle(fontSize: 16, color: AppTheme.textTertiary),
          ),
          const SizedBox(height: 6),
          const Text(
            '点击右上角 + 创建或扫码加入',
            style: TextStyle(fontSize: 13, color: AppTheme.textTertiary),
          ),
        ],
      ),
    );
  }

  Widget _buildList() {
    return RefreshIndicator(
      onRefresh: _load,
      child: ListView.separated(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        itemCount: _items.length,
        separatorBuilder: (_, __) => const SizedBox(height: 8),
        itemBuilder: (_, index) => _buildCard(_items[index]),
      ),
    );
  }

  Widget _buildCard(_DuoqianListItem item) {
    final ss58 = _hexToSs58(item.duoqianAddress);
    final color = _isPersonal ? AppTheme.accent : AppTheme.info;
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
                child: Icon(
                  _isPersonal ? Icons.person : Icons.business,
                  size: 20,
                  color: color,
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
                      _truncateAddress(ss58),
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

  String _toHex(List<int> bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}

class _DuoqianListItem {
  const _DuoqianListItem({
    required this.duoqianAddress,
    required this.name,
    required this.addedAtMillis,
  });

  final String duoqianAddress;
  final String name;
  final int addedAtMillis;
}
