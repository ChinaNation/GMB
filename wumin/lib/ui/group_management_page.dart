import 'package:flutter/material.dart';
import 'package:isar/isar.dart';

import '../isar/wallet_isar.dart';
import 'app_theme.dart';

/// 分组管理页面。
///
/// 可新增、重命名、删除分组。默认分组不能删除。
class GroupManagementPage extends StatefulWidget {
  const GroupManagementPage({super.key});

  @override
  State<GroupManagementPage> createState() => _GroupManagementPageState();
}

class _GroupManagementPageState extends State<GroupManagementPage> {
  // 总分组数上限，包含默认分组"全部 / 分组一 / 分组二"。
  static const int maxGroups = 50;

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

  /// 分组名称最大字符数。
  static const int maxGroupNameLength = 5;

  Future<void> _addGroup() async {
    if (_groups.length >= maxGroups) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('最多只能创建 $maxGroups 个分组')),
      );
      return;
    }

    final name = await _showNameDialog(title: '新建分组');
    if (name == null || name.isEmpty) return;
    if (name.runes.length > maxGroupNameLength) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('分组名称最多5个字')),
      );
      return;
    }

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
    if (name.runes.length > maxGroupNameLength) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('分组名称最多5个字')),
      );
      return;
    }

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

  // ignore: unused_element
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
            icon: Container(
              width: 32,
              height: 32,
              decoration: BoxDecoration(
                color: AppTheme.primary.withAlpha(25),
                borderRadius: BorderRadius.circular(8),
              ),
              child: const Icon(Icons.add,
                  size: 20, color: AppTheme.primaryLight),
            ),
            tooltip: '新建分组',
          ),
        ],
      ),
      body: _groups.isEmpty
          ? Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.folder_outlined,
                      size: 48, color: AppTheme.textTertiary),
                  const SizedBox(height: 12),
                  const Text(
                    '暂无分组',
                    style: TextStyle(color: AppTheme.textTertiary),
                  ),
                ],
              ),
            )
          : ListView.builder(
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
                    margin: const EdgeInsets.symmetric(vertical: 4),
                    decoration: BoxDecoration(
                      color: AppTheme.danger.withAlpha(30),
                      borderRadius:
                          BorderRadius.circular(AppTheme.radiusMd),
                    ),
                    child: const Icon(Icons.delete_outline,
                        color: AppTheme.danger),
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
                  child: Padding(
                    padding: const EdgeInsets.symmetric(vertical: 4),
                    child: Material(
                      color: Colors.transparent,
                      child: InkWell(
                        borderRadius:
                            BorderRadius.circular(AppTheme.radiusMd),
                        onTap: group.name == '全部'
                            ? null
                            : () => _renameGroup(group),
                        child: Container(
                          padding: const EdgeInsets.all(16),
                          decoration: AppTheme.cardDecoration(),
                          child: Row(
                            children: [
                              Container(
                                width: 36,
                                height: 36,
                                decoration: BoxDecoration(
                                  color: group.name == '全部'
                                      ? AppTheme.primary.withAlpha(20)
                                      : AppTheme.surfaceElevated,
                                  borderRadius:
                                      BorderRadius.circular(8),
                                ),
                                child: Icon(
                                  group.name == '全部'
                                      ? Icons.folder_special_rounded
                                      : Icons.folder_outlined,
                                  size: 18,
                                  color: group.name == '全部'
                                      ? AppTheme.primaryLight
                                      : AppTheme.textSecondary,
                                ),
                              ),
                              const SizedBox(width: 14),
                              Text(
                                group.name,
                                style: const TextStyle(
                                  fontWeight: FontWeight.w600,
                                  color: AppTheme.textPrimary,
                                  fontSize: 15,
                                ),
                              ),
                              if (group.name == '全部') ...[
                                const SizedBox(width: 8),
                                Container(
                                  padding: const EdgeInsets.symmetric(
                                      horizontal: 6, vertical: 2),
                                  decoration: BoxDecoration(
                                    color:
                                        AppTheme.primary.withAlpha(20),
                                    borderRadius:
                                        BorderRadius.circular(4),
                                  ),
                                  child: const Text(
                                    '默认',
                                    style: TextStyle(
                                      fontSize: 10,
                                      color: AppTheme.primaryLight,
                                      fontWeight: FontWeight.w500,
                                    ),
                                  ),
                                ),
                              ],
                              const Spacer(),
                              if (group.name != '全部')
                                const Icon(Icons.chevron_right,
                                    size: 20,
                                    color: AppTheme.textTertiary),
                            ],
                          ),
                        ),
                      ),
                    ),
                  ),
                );
              },
            ),
    );
  }
}
