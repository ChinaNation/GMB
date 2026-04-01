import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:isar/isar.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../isar/wallet_isar.dart';
import '../qr/qr_protocols.dart';
import '../util/screenshot_guard.dart';
import '../wallet/wallet_manager.dart';
import 'app_theme.dart';

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
    if (_screenshotGuardActive) {
      ScreenshotGuard.onSecurityEvent = null;
      ScreenshotGuard.disable();
    }
    super.dispose();
  }

  void _enableScreenshotGuard() {
    if (!_screenshotGuardActive) {
      _screenshotGuardActive = true;
      ScreenshotGuard.onSecurityEvent = _onSecurityEvent;
      ScreenshotGuard.enable();
    }
  }

  /// iOS 截屏/录屏事件：立即隐藏已展示的私钥和助记词。
  void _onSecurityEvent(String event) {
    if (!mounted) return;
    if (event == 'screenshot_taken' || event == 'screen_recording_started') {
      setState(() {
        _seedVisible = false;
        _mnemonicVisible = false;
        _seedHex = null;
        _mnemonic = null;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(event == 'screenshot_taken'
              ? '检测到截屏，密钥信息已隐藏。请勿截屏保存密钥。'
              : '检测到屏幕录制，密钥信息已隐藏'),
          duration: const Duration(seconds: 3),
        ),
      );
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
            style: TextButton.styleFrom(foregroundColor: AppTheme.danger),
            child: const Text('查看'),
          ),
        ],
      ),
    );
  }

  void _copyToClipboard(String text, String label) {
    Clipboard.setData(ClipboardData(text: text));
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
          content: Text('$label已复制'), duration: const Duration(seconds: 1)),
    );
  }

  void _showUserQrCode(WalletProfile wallet) {
    final qrData = jsonEncode({
      'proto': QrProtocols.user,
      'address': wallet.address,
      'pubkey': '0x${wallet.pubkeyHex}',
      'name': wallet.walletName,
    });
    showDialog(
      context: context,
      builder: (context) => Dialog(
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(16),
        ),
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                wallet.walletName,
                style: const TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                ),
              ),
              const SizedBox(height: 4),
              Text(
                '扫码转账或绑定',
                style: TextStyle(
                  fontSize: 13,
                  color: Colors.grey[600],
                ),
              ),
              const SizedBox(height: 16),
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Colors.white,
                  borderRadius: BorderRadius.circular(12),
                  border: Border.all(color: Colors.grey[200]!),
                ),
                child: QrImageView(
                  data: qrData,
                  version: QrVersions.auto,
                  size: 240,
                  eyeStyle: const QrEyeStyle(
                    eyeShape: QrEyeShape.square,
                    color: Color(0xFF134E4A),
                  ),
                  dataModuleStyle: const QrDataModuleStyle(
                    dataModuleShape: QrDataModuleShape.square,
                    color: Color(0xFF134E4A),
                  ),
                ),
              ),
              const SizedBox(height: 12),
              Text(
                wallet.address,
                style: TextStyle(
                  fontSize: 11,
                  color: Colors.grey[500],
                  fontFamily: 'monospace',
                ),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 16),
              SizedBox(
                width: double.infinity,
                child: TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: const Text('关闭'),
                ),
              ),
            ],
          ),
        ),
      ),
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
          // 钱包头部卡片
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              gradient: AppTheme.primaryGradient,
              borderRadius: BorderRadius.circular(AppTheme.radiusLg),
              boxShadow: [
                BoxShadow(
                  color: AppTheme.primary.withAlpha(40),
                  blurRadius: 16,
                  offset: const Offset(0, 6),
                ),
              ],
            ),
            child: Row(
              children: [
                Container(
                  width: 48,
                  height: 48,
                  decoration: BoxDecoration(
                    color: Colors.white.withAlpha(30),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: const Icon(Icons.account_balance_wallet_rounded,
                      color: Colors.white, size: 24),
                ),
                const SizedBox(width: 14),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        wallet.walletName,
                        style: const TextStyle(
                          fontSize: 18,
                          fontWeight: FontWeight.w700,
                          color: Colors.white,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        '${wallet.address.substring(0, 8)}...${wallet.address.substring(wallet.address.length - 6)}',
                        style: TextStyle(
                          fontSize: 13,
                          color: Colors.white.withAlpha(180),
                          fontFamily: 'monospace',
                        ),
                      ),
                    ],
                  ),
                ),
                GestureDetector(
                  onTap: () => _showUserQrCode(wallet),
                  child: Container(
                    width: 36,
                    height: 36,
                    decoration: BoxDecoration(
                      color: Colors.white.withAlpha(30),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: const Icon(Icons.qr_code_rounded,
                        color: Colors.white, size: 20),
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 20),
          // 信息区
          Container(
            decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
            child: Column(
              children: [
                _buildInfoTile('地址', wallet.address),
                const Divider(height: 1, indent: 16, endIndent: 16),
                _buildInfoTile('公钥', '0x${wallet.pubkeyHex}'),
                if (_groups
                    .where((g) => g.name != '全部')
                    .isNotEmpty) ...[
                  const Divider(height: 1, indent: 16, endIndent: 16),
                  _buildGroupSelector(),
                ],
              ],
            ),
          ),
          const SizedBox(height: 16),
          // 敏感信息区
          Container(
            decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
            child: Column(
              children: [
                _buildSecretTile(
                  label: '私钥',
                  value: _seedHex != null ? '0x$_seedHex' : null,
                  visible: _seedVisible,
                  onReveal: _revealSeed,
                  onHide: () => setState(() => _seedVisible = false),
                ),
                const Divider(height: 1, indent: 16, endIndent: 16),
                _buildSecretTile(
                  label: '助记词',
                  value: _mnemonic,
                  visible: _mnemonicVisible,
                  onReveal: _revealMnemonic,
                  onHide: () => setState(() => _mnemonicVisible = false),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildGroupSelector() {
    final selectableGroups = _groups.where((g) => g.name != '全部').toList();
    if (selectableGroups.isEmpty) return const SizedBox.shrink();

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          GestureDetector(
            onTap: () => setState(() => _groupsExpanded = !_groupsExpanded),
            behavior: HitTestBehavior.opaque,
            child: Row(
              children: [
                const Icon(Icons.folder_outlined,
                    size: 16, color: AppTheme.textSecondary),
                const SizedBox(width: 8),
                const Text(
                  '分组',
                  style: TextStyle(
                    fontSize: 13,
                    color: AppTheme.textSecondary,
                    fontWeight: FontWeight.w500,
                  ),
                ),
                const Spacer(),
                Icon(
                  _groupsExpanded
                      ? Icons.keyboard_arrow_down
                      : Icons.keyboard_arrow_right,
                  size: 20,
                  color: AppTheme.textTertiary,
                ),
              ],
            ),
          ),
          if (_groupsExpanded) ...[
            const SizedBox(height: 10),
            Wrap(
              spacing: 8,
              runSpacing: 4,
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

  Widget _buildInfoTile(String label, String value) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
            style: const TextStyle(
              fontSize: 12,
              color: AppTheme.textSecondary,
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 8),
          Row(
            children: [
              Expanded(
                child: SelectableText(
                  value,
                  style: const TextStyle(
                    fontSize: 13,
                    fontFamily: 'monospace',
                    color: AppTheme.textPrimary,
                    letterSpacing: 0.3,
                  ),
                ),
              ),
              Container(
                width: 34,
                height: 34,
                decoration: BoxDecoration(
                  color: AppTheme.surfaceElevated,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: IconButton(
                  padding: EdgeInsets.zero,
                  icon: const Icon(Icons.copy_rounded,
                      size: 16, color: AppTheme.primaryLight),
                  onPressed: () => _copyToClipboard(value, label),
                  tooltip: '复制',
                ),
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
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                label == '私钥' ? Icons.vpn_key_rounded : Icons.key_rounded,
                size: 16,
                color: AppTheme.textSecondary,
              ),
              const SizedBox(width: 8),
              Text(
                label,
                style: const TextStyle(
                  fontSize: 12,
                  color: AppTheme.textSecondary,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ],
          ),
          const SizedBox(height: 10),
          if (!visible)
            Material(
              color: Colors.transparent,
              child: InkWell(
                onTap: onReveal,
                borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                child: Container(
                  width: double.infinity,
                  padding:
                      const EdgeInsets.symmetric(vertical: 18, horizontal: 12),
                  decoration: BoxDecoration(
                    color: AppTheme.surfaceElevated,
                    borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                    border: Border.all(color: AppTheme.border),
                  ),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.visibility_off_rounded,
                          color: AppTheme.textTertiary, size: 18),
                      const SizedBox(width: 8),
                      Text(
                        '点击查看$label',
                        style: const TextStyle(
                          color: AppTheme.textTertiary,
                          fontSize: 13,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            )
          else ...[
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(14),
              decoration: BoxDecoration(
                color: AppTheme.danger.withAlpha(15),
                borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                border: Border.all(color: AppTheme.danger.withAlpha(40)),
              ),
              child: Text(
                value ?? '无数据',
                style: const TextStyle(
                  fontSize: 13,
                  fontFamily: 'monospace',
                  color: AppTheme.textPrimary,
                  letterSpacing: 0.3,
                ),
              ),
            ),
            const SizedBox(height: 10),
            Row(
              children: [
                const Expanded(
                  child: Text(
                    '请手抄备份，不支持复制',
                    style: TextStyle(
                        color: AppTheme.danger,
                        fontSize: 12,
                        fontWeight: FontWeight.w500),
                  ),
                ),
                TextButton.icon(
                  onPressed: onHide,
                  icon: const Icon(Icons.visibility_off_rounded, size: 16),
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
