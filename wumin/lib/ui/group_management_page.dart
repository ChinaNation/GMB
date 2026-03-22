import 'package:flutter/material.dart';
import 'package:isar/isar.dart';

import '../isar/wallet_isar.dart';

/// 分组管理页面。
///
/// 可新增、重命名、删除分组。默认分组不能删除。
class GroupManagementPage extends StatefulWidget {
  const GroupManagementPage({super.key});

  @override
  State<GroupManagementPage> createState() => _GroupManagementPageState();
}

class _GroupManagementPageState extends State<GroupManagementPage> {
  static const int maxGroups = 20;

  List<WalletGroupEntity> _groups = [];

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final isar = await WalletIsar.instance.db();
    final groups = await isar.walletGroupEntitys
        .where()
        .sortBySortOrder()
        .findAll();
    if (!mounted) return;
    setState(() {
      _groups = groups;
    });
  }

  Future<void> _addGroup() async {
    if (_groups.length >= maxGroups) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('最多只能创建 $maxGroups 个分组')),
      );
      return;
    }

    final name = await _showNameDialog(title: '新建分组');
    if (name == null || name.isEmpty) return;

    // 检查重名
    if (_groups.any((g) => g.name == name)) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('分组"$name"已存在')),
      );
      return;
    }

    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      await isar.walletGroupEntitys.put(
        WalletGroupEntity()
          ..name = name
          ..sortOrder = _groups.length
          ..isDefault = false,
      );
    });
    await _load();
  }

  Future<void> _renameGroup(WalletGroupEntity group) async {
    final name = await _showNameDialog(
      title: '重命名分组',
      initialValue: group.name,
    );
    if (name == null || name.isEmpty || name == group.name) return;

    if (_groups.any((g) => g.name == name)) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('分组"$name"已存在')),
      );
      return;
    }

    final isar = await WalletIsar.instance.db();

    // 更新所有属于该分组的钱包
    final oldName = group.name;
    await isar.writeTxn(() async {
      group.name = name;
      await isar.walletGroupEntitys.put(group);

      final wallets = await isar.walletProfileEntitys
          .filter()
          .groupNamesContains(oldName)
          .findAll();
      for (final w in wallets) {
        final names = w.groupNames
            .split(',')
            .map((n) => n == oldName ? name : n)
            .join(',');
        w.groupNames = names;
        await isar.walletProfileEntitys.put(w);
      }
    });
    await _load();
  }

  Future<void> _deleteGroup(WalletGroupEntity group) async {
    if (group.isDefault) return;

    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('删除分组'),
        content: Text('确定删除分组"${group.name}"？\n该分组下的钱包将移至"全部"。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('删除'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;

    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      final wallets = await isar.walletProfileEntitys
          .filter()
          .groupNamesContains(group.name)
          .findAll();
      for (final w in wallets) {
        final names = w.groupNames
            .split(',')
            .where((n) => n != group.name)
            .join(',');
        w.groupNames = names;
        await isar.walletProfileEntitys.put(w);
      }
      await isar.walletGroupEntitys.delete(group.id);
    });
    await _load();
  }

  Future<String?> _showNameDialog({
    required String title,
    String initialValue = '',
  }) async {
    final controller = TextEditingController(text: initialValue);
    return showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(title),
        content: TextField(
          controller: controller,
          maxLength: 5,
          autofocus: true,
          decoration: const InputDecoration(
            hintText: '最多5个字',
            counterText: '',
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () {
              final text = controller.text.trim();
              Navigator.pop(ctx, text);
            },
            child: const Text('确定'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('分组管理'),
        centerTitle: true,
        actions: [
          IconButton(
            onPressed: _addGroup,
            icon: const Icon(Icons.add),
            tooltip: '新建分组',
          ),
        ],
      ),
      body: ListView.builder(
        padding: const EdgeInsets.all(16),
        itemCount: _groups.length,
        itemBuilder: (context, index) {
          final group = _groups[index];
          return Dismissible(
            key: ValueKey(group.id),
            direction: group.isDefault
                ? DismissDirection.none
                : DismissDirection.endToStart,
            background: Container(
              alignment: Alignment.centerRight,
              padding: const EdgeInsets.only(right: 20),
              color: Colors.red,
              child: const Icon(Icons.delete, color: Colors.white),
            ),
            confirmDismiss: (_) async {
              if (group.isDefault) return false;
              final confirmed = await showDialog<bool>(
                context: context,
                builder: (ctx) => AlertDialog(
                  title: const Text('删除分组'),
                  content: Text(
                      '确定删除分组"${group.name}"？\n该分组下的钱包将移至"全部"。'),
                  actions: [
                    TextButton(
                      onPressed: () => Navigator.pop(ctx, false),
                      child: const Text('取消'),
                    ),
                    FilledButton(
                      onPressed: () => Navigator.pop(ctx, true),
                      child: const Text('删除'),
                    ),
                  ],
                ),
              );
              if (confirmed != true) return false;
              final isar = await WalletIsar.instance.db();
              await isar.writeTxn(() async {
                // 从所有钱包的 groupNames 中移除该分组
                final wallets = await isar.walletProfileEntitys
                    .filter()
                    .groupNamesContains(group.name)
                    .findAll();
                for (final w in wallets) {
                  final names = w.groupNames
                      .split(',')
                      .where((n) => n != group.name)
                      .join(',');
                  w.groupNames = names;
                  await isar.walletProfileEntitys.put(w);
                }
                await isar.walletGroupEntitys.delete(group.id);
              });
              await _load();
              return false;
            },
            child: Card(
              margin: const EdgeInsets.symmetric(vertical: 4),
              child: ListTile(
                contentPadding:
                    const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
                title: Text(
                  group.name,
                  style: const TextStyle(fontWeight: FontWeight.w600),
                ),
                trailing: group.name == '全部'
                    ? null
                    : IconButton(
                        icon: const Icon(Icons.chevron_right),
                        onPressed: () => _renameGroup(group),
                      ),
              ),
            ),
          );
        },
      ),
    );
  }
}
