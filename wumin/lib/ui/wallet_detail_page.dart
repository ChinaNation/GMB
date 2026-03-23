import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:isar/isar.dart';

import '../isar/wallet_isar.dart';
import '../util/screenshot_guard.dart';
import '../wallet/wallet_manager.dart';

/// 钱包详情页：名称、地址、公钥、私钥（遮挡）、助记词（遮挡）。
class WalletDetailPage extends StatefulWidget {
  const WalletDetailPage({super.key, required this.wallet});

  final WalletProfile wallet;

  @override
  State<WalletDetailPage> createState() => _WalletDetailPageState();
}

class _WalletDetailPageState extends State<WalletDetailPage> {
  final WalletManager _walletManager = WalletManager();

  String? _seedHex;
  String? _mnemonic;
  bool _seedVisible = false;
  bool _mnemonicVisible = false;
  bool _screenshotGuardActive = false;
  List<WalletGroupEntity> _groups = [];
  late Set<String> _selectedGroups;
  bool _groupsExpanded = false;

  @override
  void initState() {
    super.initState();
    _selectedGroups = widget.wallet.groupNames.toSet();
    _loadGroups();
  }

  @override
  void dispose() {
    if (_screenshotGuardActive) ScreenshotGuard.disable();
    super.dispose();
  }

  void _enableScreenshotGuard() {
    if (!_screenshotGuardActive) {
      _screenshotGuardActive = true;
      ScreenshotGuard.enable();
    }
  }

  Future<void> _loadGroups() async {
    final isar = await WalletIsar.instance.db();
    final groups = await isar.walletGroupEntitys
        .where()
        .sortBySortOrder()
        .findAll();
    if (!mounted) return;
    setState(() => _groups = groups);
  }

  Future<void> _toggleGroup(String groupName, bool selected) async {
    final updated = Set<String>.from(_selectedGroups);
    if (selected) {
      updated.add(groupName);
    } else {
      updated.remove(groupName);
    }

    final isar = await WalletIsar.instance.db();
    final entity = await isar.walletProfileEntitys
        .filter()
        .walletIndexEqualTo(widget.wallet.walletIndex)
        .findFirst();
    if (entity == null) return;
    await isar.writeTxn(() async {
      entity.groupNames = updated.join(',');
      await isar.walletProfileEntitys.put(entity);
    });
    if (!mounted) return;
    setState(() => _selectedGroups = updated);
  }

  Future<void> _revealSeed() async {
    final confirmed = await _confirmReveal('私钥');
    if (confirmed != true) return;
    try {
      final seed = await _walletManager.getSeedHex(widget.wallet.walletIndex);
      if (!mounted) return;
      _enableScreenshotGuard();
      setState(() {
        _seedHex = seed;
        _seedVisible = true;
      });
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('验证失败：$e')),
      );
    }
  }

  Future<void> _revealMnemonic() async {
    final confirmed = await _confirmReveal('助记词');
    if (confirmed != true) return;
    try {
      final mnemonic =
          await _walletManager.getMnemonic(widget.wallet.walletIndex);
      if (!mounted) return;
      _enableScreenshotGuard();
      setState(() {
        _mnemonic = mnemonic;
        _mnemonicVisible = true;
      });
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('验证失败：$e')),
      );
    }
  }

  Future<bool?> _confirmReveal(String label) {
    return showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('查看$label'),
        content: Text('$label是核心机密信息，泄露将导致资产被盗。\n\n确认要查看吗？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            style: TextButton.styleFrom(foregroundColor: Colors.red),
            child: const Text('查看'),
          ),
        ],
      ),
    );
  }

  void _copyToClipboard(String text, String label) {
    Clipboard.setData(ClipboardData(text: text));
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('$label已复制'), duration: const Duration(seconds: 1)),
    );
  }

  @override
  Widget build(BuildContext context) {
    final wallet = widget.wallet;
    return Scaffold(
      appBar: AppBar(
        title: const Text('钱包详情'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          _buildInfoTile('名称', wallet.walletName, copyable: false),
          const Divider(),
          _buildGroupSelector(),
          const Divider(),
          _buildInfoTile('地址', wallet.address),
          const Divider(),
          _buildInfoTile('公钥', '0x${wallet.pubkeyHex}'),
          const Divider(),
          _buildSecretTile(
            label: '私钥',
            value: _seedHex != null ? '0x$_seedHex' : null,
            visible: _seedVisible,
            onReveal: _revealSeed,
            onHide: () => setState(() => _seedVisible = false),
          ),
          const Divider(),
          _buildSecretTile(
            label: '助记词',
            value: _mnemonic,
            visible: _mnemonicVisible,
            onReveal: _revealMnemonic,
            onHide: () => setState(() => _mnemonicVisible = false),
          ),
        ],
      ),
    );
  }

  Widget _buildGroupSelector() {
    if (_groups.isEmpty) {
      return const SizedBox.shrink();
    }
    // 排除"全部"，它是虚拟分组
    final selectableGroups = _groups.where((g) => g.name != '全部').toList();
    if (selectableGroups.isEmpty) {
      return const SizedBox.shrink();
    }

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 「分组」+ 箭头，点击展开/收起
          GestureDetector(
            onTap: () => setState(() => _groupsExpanded = !_groupsExpanded),
            behavior: HitTestBehavior.opaque,
            child: Row(
              children: [
                Text(
                  '分组',
                  style: TextStyle(
                    fontSize: 13,
                    color: Colors.grey.shade600,
                    fontWeight: FontWeight.w500,
                  ),
                ),
                const Spacer(),
                Icon(
                  _groupsExpanded
                      ? Icons.keyboard_arrow_down
                      : Icons.keyboard_arrow_right,
                  size: 20,
                  color: Colors.grey.shade500,
                ),
              ],
            ),
          ),
          // 展开后显示全部分组 chip
          if (_groupsExpanded) ...[
            const SizedBox(height: 4),
            Wrap(
              spacing: 8,
              runSpacing: 0,
              children: selectableGroups.map((g) {
                final checked = _selectedGroups.contains(g.name);
                return FilterChip(
                  label: Text(g.name),
                  selected: checked,
                  onSelected: (val) => _toggleGroup(g.name, val),
                );
              }).toList(),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildInfoTile(String label, String value, {bool copyable = true}) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
            style: TextStyle(
              fontSize: 13,
              color: Colors.grey.shade600,
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 6),
          Row(
            children: [
              Expanded(
                child: SelectableText(
                  value,
                  style: const TextStyle(
                    fontSize: 14,
                    fontFamily: 'monospace',
                  ),
                ),
              ),
              if (copyable)
                IconButton(
                  icon: const Icon(Icons.copy, size: 18),
                  onPressed: () => _copyToClipboard(value, label),
                  tooltip: '复制',
                ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildSecretTile({
    required String label,
    required String? value,
    required bool visible,
    required VoidCallback onReveal,
    required VoidCallback onHide,
  }) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
            style: TextStyle(
              fontSize: 13,
              color: Colors.grey.shade600,
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 6),
          if (!visible)
            InkWell(
              onTap: onReveal,
              borderRadius: BorderRadius.circular(8),
              child: Container(
                width: double.infinity,
                padding: const EdgeInsets.symmetric(vertical: 16, horizontal: 12),
                decoration: BoxDecoration(
                  color: Colors.grey.shade100,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Icon(Icons.visibility_off, color: Colors.grey.shade500),
                    const SizedBox(width: 8),
                    Text(
                      '点击查看$label',
                      style: TextStyle(color: Colors.grey.shade600),
                    ),
                  ],
                ),
              ),
            )
          else ...[
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.red.shade50,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Colors.red.shade200),
              ),
              child: Text(
                value ?? '无数据',
                style: const TextStyle(
                  fontSize: 14,
                  fontFamily: 'monospace',
                ),
              ),
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                const Expanded(
                  child: Text(
                    '请手抄备份，不支持复制',
                    style: TextStyle(color: Colors.red, fontSize: 12),
                  ),
                ),
                TextButton.icon(
                  onPressed: onHide,
                  icon: const Icon(Icons.visibility_off, size: 16),
                  label: const Text('隐藏'),
                ),
              ],
            ),
          ],
        ],
      ),
    );
  }
}
