import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:isar/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../Isar/wallet_isar.dart';
import 'duoqian_create_proposal_page.dart';
import 'duoqian_institution_info_page.dart';
import 'institution_data.dart';
import 'personal_duoqian_create_page.dart';
import '../wallet/core/wallet_manager.dart';

/// 多签列表页模式。
enum InstitutionListMode {
  /// 从多签交易页 "+" 进入，选择账户后 pop 返回 InstitutionInfo。
  select,

  /// 从我的页 "多签" 进入，管理多签账户。
  manage,
}

/// 统一多签账户列表页。
///
/// 混合展示机构多签和个人多签，通过标签区分。
/// - [InstitutionListMode.select]：选择账户，无"+"按钮。
/// - [InstitutionListMode.manage]：管理账户，"+"弹出创建选择。
class DuoqianInstitutionListPage extends StatefulWidget {
  const DuoqianInstitutionListPage({
    super.key,
    required this.mode,
  });

  final InstitutionListMode mode;

  @override
  State<DuoqianInstitutionListPage> createState() =>
      _DuoqianInstitutionListPageState();
}

class _DuoqianInstitutionListPageState
    extends State<DuoqianInstitutionListPage> {
  static const Color _inkGreen = Color(0xFF0B3D2E);
  static const Color _institutionColor = Color(0xFF1565C0);
  static const Color _personalColor = Color(0xFF6A1B9A);

  List<_DuoqianListItem> _items = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() => _loading = true);
    try {
      final isar = await WalletIsar.instance.db();

      // 读取机构多签
      final institutions = await isar.duoqianInstitutionEntitys
          .where()
          .findAll();

      // 读取个人多签
      final personals = await isar.personalDuoqianEntitys
          .where()
          .findAll();

      // 合并排序
      final items = <_DuoqianListItem>[
        ...institutions.map((e) => _DuoqianListItem(
              type: _DuoqianType.institution,
              duoqianAddress: e.duoqianAddress,
              name: e.name,
              addedAtMillis: e.addedAtMillis,
              sfidId: e.sfidId,
            )),
        ...personals.map((e) => _DuoqianListItem(
              type: _DuoqianType.personal,
              duoqianAddress: e.duoqianAddress,
              name: e.name,
              addedAtMillis: e.addedAtMillis,
            )),
      ];
      items.sort((a, b) => b.addedAtMillis.compareTo(a.addedAtMillis));

      if (!mounted) return;
      setState(() {
        _items = items;
        _loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => _loading = false);
    }
  }

  // ──── 添加 ────

  void _showCreateMenu() {
    showModalBottomSheet(
      context: context,
      builder: (ctx) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: Icon(Icons.business, color: _institutionColor),
              title: const Text('创建机构多签账户'),
              subtitle: const Text('需要 SFID 机构标识'),
              onTap: () {
                Navigator.pop(ctx);
                _openCreateInstitution();
              },
            ),
            ListTile(
              leading: Icon(Icons.person, color: _personalColor),
              title: const Text('创建个人多签账户'),
              subtitle: const Text('无需 SFID，直接设置管理员'),
              onTap: () {
                Navigator.pop(ctx);
                _openCreatePersonal();
              },
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _openCreateInstitution() async {
    final wallets = await _getColdWallets();
    if (!mounted || wallets.isEmpty) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('请先导入冷钱包')),
        );
      }
      return;
    }

    final created = await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => DuoqianCreateProposalPage(
          institution: InstitutionInfo(
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
      MaterialPageRoute(
        builder: (_) => const PersonalDuoqianCreatePage(),
      ),
    );
    if (created == true) await _load();
  }

  Future<List<WalletProfile>> _getColdWallets() async {
    final wm = WalletManager();
    final wallets = await wm.getWallets();
    return wallets.where((w) => w.signMode == 'external').toList();
  }

  // ──── 卡片点击 ────

  void _onCardTap(_DuoqianListItem item) {
    final institution = _itemToInstitutionInfo(item);

    if (widget.mode == InstitutionListMode.select) {
      Navigator.pop(context, institution);
    } else {
      Navigator.push(
        context,
        MaterialPageRoute(
          builder: (_) => DuoqianInstitutionInfoPage(
            institution: institution,
            isPersonal: item.type == _DuoqianType.personal,
          ),
        ),
      ).then((_) {
        if (mounted) _load();
      });
    }
  }

  InstitutionInfo _itemToInstitutionInfo(_DuoqianListItem item) {
    return InstitutionInfo(
      name: item.name,
      shenfenId: item.type == _DuoqianType.institution
          ? registeredDuoqianIdentity(item.duoqianAddress)
          : 'personal:${item.duoqianAddress}',
      orgType: OrgType.duoqian,
      duoqianAddress: item.duoqianAddress,
    );
  }

  // ──── UI ────

  @override
  Widget build(BuildContext context) {
    final isSelect = widget.mode == InstitutionListMode.select;
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: Text(
          isSelect ? '选择多签账户' : '多签',
          style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: _inkGreen,
        elevation: 0,
        scrolledUnderElevation: 0.5,
        actions: isSelect
            ? []
            : [
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
          Icon(Icons.groups_outlined, size: 64, color: Colors.grey[300]),
          const SizedBox(height: 12),
          Text(
            '暂无多签账户',
            style: TextStyle(fontSize: 16, color: Colors.grey[500]),
          ),
          if (widget.mode == InstitutionListMode.manage) ...[
            const SizedBox(height: 6),
            Text(
              '点击右上角 + 创建',
              style: TextStyle(fontSize: 13, color: Colors.grey[400]),
            ),
          ],
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
    final isInstitution = item.type == _DuoqianType.institution;
    final tagColor = isInstitution ? _institutionColor : _personalColor;
    final tagLabel = isInstitution ? '机构' : '个人';
    final tagIcon = isInstitution ? Icons.business : Icons.person;

    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: tagColor.withValues(alpha: 0.15)),
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
                  color: tagColor.withValues(alpha: 0.08),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(tagIcon, size: 20, color: tagColor),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Flexible(
                          child: Text(
                            item.name,
                            style: const TextStyle(
                              fontSize: 15,
                              fontWeight: FontWeight.w600,
                              color: _inkGreen,
                            ),
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                        const SizedBox(width: 6),
                        Container(
                          padding: const EdgeInsets.symmetric(
                              horizontal: 5, vertical: 1),
                          decoration: BoxDecoration(
                            color: tagColor.withValues(alpha: 0.1),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            tagLabel,
                            style: TextStyle(
                              fontSize: 10,
                              fontWeight: FontWeight.w600,
                              color: tagColor,
                            ),
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 2),
                    Text(
                      _truncateAddress(ss58),
                      style: TextStyle(fontSize: 12, color: Colors.grey[500]),
                    ),
                  ],
                ),
              ),
              Icon(Icons.chevron_right, size: 20, color: Colors.grey[400]),
            ],
          ),
        ),
      ),
    );
  }

  // ──── 工具 ────

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

// ──── 内部数据模型 ────

enum _DuoqianType { institution, personal }

class _DuoqianListItem {
  const _DuoqianListItem({
    required this.type,
    required this.duoqianAddress,
    required this.name,
    required this.addedAtMillis,
    this.sfidId,
  });

  final _DuoqianType type;
  final String duoqianAddress;
  final String name;
  final int addedAtMillis;
  final String? sfidId;
}
