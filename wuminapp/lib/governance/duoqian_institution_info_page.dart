import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:isar/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../Isar/wallet_isar.dart';
import 'duoqian_close_proposal_page.dart';
import 'duoqian_manage_models.dart';
import 'duoqian_manage_service.dart';
import 'institution_admin_service.dart';
import 'institution_data.dart';
import '../wallet/core/wallet_manager.dart';

/// 多签机构详情页。
///
/// 展示机构名称、SFID ID、多签地址、状态、管理员列表。
/// 右上角 "..." 提供注销（关闭）操作。
class DuoqianInstitutionInfoPage extends StatefulWidget {
  const DuoqianInstitutionInfoPage({
    super.key,
    required this.institution,
    this.isPersonal = false,
  });

  final InstitutionInfo institution;

  /// 是否为个人多签（不显示 SFID ID 行）。
  final bool isPersonal;

  @override
  State<DuoqianInstitutionInfoPage> createState() =>
      _DuoqianInstitutionInfoPageState();
}

class _DuoqianInstitutionInfoPageState
    extends State<DuoqianInstitutionInfoPage> {
  static const Color _inkGreen = Color(0xFF0B3D2E);

  final DuoqianManageService _manageService = DuoqianManageService();
  final InstitutionAdminService _adminService = InstitutionAdminService();

  bool _loading = true;
  String? _error;

  DuoqianAccountInfo? _accountInfo;
  List<String> _adminPubkeys = const [];

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });

    try {
      final results = await Future.wait([
        _manageService.fetchDuoqianAccount(widget.institution.duoqianAddress),
        _adminService.fetchAdmins(widget.institution.shenfenId),
      ]);

      final accountInfo = results[0] as DuoqianAccountInfo?;
      final admins = results[1] as List<String>;

      if (!mounted) return;
      setState(() {
        _accountInfo = accountInfo;
        _adminPubkeys = admins;
        _loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.toString();
        _loading = false;
      });
    }
  }

  // ──── 注销 ────

  void _showDeleteMenu() {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('注销多签机构'),
        content: const Text(
          '注销将发起链上关闭提案，需要其他管理员投票通过后才会真正关闭。\n\n确定要发起注销吗？',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () {
              Navigator.pop(ctx);
              _openClosePage();
            },
            style: TextButton.styleFrom(foregroundColor: Colors.red),
            child: const Text('发起注销'),
          ),
        ],
      ),
    );
  }

  Future<void> _openClosePage() async {
    final wallets = await _getAdminWallets();
    if (!mounted || wallets.isEmpty) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('请先导入此机构的管理员冷钱包')),
        );
      }
      return;
    }

    final closed = await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => DuoqianCloseProposalPage(
          institution: widget.institution,
          adminWallets: wallets,
        ),
      ),
    );
    if (closed == true && mounted) {
      // 提案提交成功，从本地移除
      await _removeFromLocal();
      if (mounted) Navigator.pop(context);
    }
  }

  Future<List<WalletProfile>> _getAdminWallets() async {
    final wm = WalletManager();
    final wallets = await wm.getWallets();
    final adminSet = _adminPubkeys.toSet();
    return wallets.where((w) {
      var pk = w.pubkeyHex.toLowerCase();
      if (pk.startsWith('0x')) pk = pk.substring(2);
      return adminSet.contains(pk);
    }).toList();
  }

  Future<void> _removeFromLocal() async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      await isar.duoqianInstitutionEntitys
          .where()
          .duoqianAddressEqualTo(widget.institution.duoqianAddress)
          .deleteAll();
    });
  }

  // ──── UI ────

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '机构详情',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: _inkGreen,
        elevation: 0,
        scrolledUnderElevation: 0.5,
        actions: [
          PopupMenuButton<String>(
            icon: const Icon(Icons.more_vert),
            onSelected: (value) {
              if (value == 'delete') _showDeleteMenu();
            },
            itemBuilder: (_) => [
              const PopupMenuItem(
                value: 'delete',
                child: Row(
                  children: [
                    Icon(Icons.delete_outline, size: 20, color: Colors.red),
                    SizedBox(width: 8),
                    Text('注销机构', style: TextStyle(color: Colors.red)),
                  ],
                ),
              ),
            ],
          ),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : _error != null
              ? _buildError()
              : _buildContent(),
    );
  }

  Widget _buildError() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.error_outline, size: 48, color: Colors.red),
            const SizedBox(height: 12),
            Text('加载失败',
                style: TextStyle(fontSize: 16, color: Colors.grey[700])),
            const SizedBox(height: 6),
            Text(
              _error!,
              style: TextStyle(fontSize: 12, color: Colors.grey[500]),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 16),
            OutlinedButton(onPressed: _load, child: const Text('重试')),
          ],
        ),
      ),
    );
  }

  Widget _buildContent() {
    final duoqianSs58 = _hexToSs58(widget.institution.duoqianAddress);
    final info = _accountInfo;
    final statusLabel = info == null
        ? '未找到'
        : info.status == DuoqianStatus.active
            ? '已激活'
            : '待激活';
    final statusColor = info?.status == DuoqianStatus.active
        ? Colors.green
        : Colors.orange;

    return RefreshIndicator(
      onRefresh: () async {
        _adminService.clearCache(widget.institution.shenfenId);
        await _load();
      },
      child: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          // 基本信息卡片
          Card(
            elevation: 0,
            margin: EdgeInsets.zero,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
              side: BorderSide(color: Colors.grey[200]!),
            ),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    '机构信息',
                    style: TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.w700,
                      color: _inkGreen,
                    ),
                  ),
                  const SizedBox(height: 12),
                  _buildInfoRow('名称', widget.institution.name),
                  if (!widget.isPersonal) ...[
                    const Divider(height: 20),
                    _buildInfoRow(
                      'SFID ID',
                      _extractSfidId(widget.institution.shenfenId),
                    ),
                  ],
                  const Divider(height: 20),
                  _buildInfoRow(
                    '多签地址',
                    _truncateAddress(duoqianSs58),
                    onCopy: () {
                      Clipboard.setData(ClipboardData(text: duoqianSs58));
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(
                          content: Text('地址已复制'),
                          duration: Duration(seconds: 1),
                        ),
                      );
                    },
                  ),
                  const Divider(height: 20),
                  _buildInfoRow('状态', statusLabel, valueColor: statusColor),
                  if (info != null) ...[
                    const Divider(height: 20),
                    _buildInfoRow('管理员数量', '${info.adminCount}'),
                    const Divider(height: 20),
                    _buildInfoRow('通过阈值', '${info.threshold} / ${info.adminCount}'),
                  ],
                ],
              ),
            ),
          ),

          const SizedBox(height: 16),

          // 管理员列表
          Card(
            elevation: 0,
            margin: EdgeInsets.zero,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
              side: BorderSide(color: Colors.grey[200]!),
            ),
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 8),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Padding(
                    padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
                    child: Text(
                      '管理员列表（${_adminPubkeys.length} 人）',
                      style: const TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w700,
                        color: _inkGreen,
                      ),
                    ),
                  ),
                  const Divider(),
                  if (_adminPubkeys.isEmpty)
                    Padding(
                      padding: const EdgeInsets.all(16),
                      child: Text(
                        '暂无管理员信息',
                        style: TextStyle(color: Colors.grey[500]),
                      ),
                    )
                  else
                    ...List.generate(_adminPubkeys.length, (index) {
                      final pubkey = _adminPubkeys[index];
                      final ss58 = _pubkeyToSS58(pubkey);
                      return ListTile(
                        dense: true,
                        leading: CircleAvatar(
                          radius: 16,
                          backgroundColor: _inkGreen.withValues(alpha: 0.08),
                          child: Text(
                            '${index + 1}',
                            style: const TextStyle(
                              fontSize: 12,
                              fontWeight: FontWeight.w600,
                              color: _inkGreen,
                            ),
                          ),
                        ),
                        title: Text(
                          _truncateAddress(ss58),
                          style: const TextStyle(fontSize: 13),
                        ),
                        trailing: IconButton(
                          icon: Icon(Icons.copy,
                              size: 16, color: Colors.grey[400]),
                          onPressed: () {
                            Clipboard.setData(ClipboardData(text: ss58));
                            ScaffoldMessenger.of(context).showSnackBar(
                              const SnackBar(
                                content: Text('地址已复制'),
                                duration: Duration(seconds: 1),
                              ),
                            );
                          },
                        ),
                      );
                    }),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildInfoRow(String label, String value,
      {VoidCallback? onCopy, Color? valueColor}) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 80,
          child: Text(
            label,
            style: TextStyle(fontSize: 13, color: Colors.grey[600]),
          ),
        ),
        Expanded(
          child: Text(
            value,
            style: TextStyle(
              fontSize: 13,
              color: valueColor ?? const Color(0xFF333333),
              fontWeight: valueColor != null ? FontWeight.w600 : null,
            ),
          ),
        ),
        if (onCopy != null)
          GestureDetector(
            onTap: onCopy,
            child: Icon(Icons.copy, size: 16, color: Colors.grey[400]),
          ),
      ],
    );
  }

  // ──── 工具 ────

  String _extractSfidId(String shenfenId) {
    // shenfenId 格式："duoqian:hex..." → 返回原始 sfidId
    // 但我们存储的 sfidId 是 UTF-8，shenfenId 是 "duoqian:" + hex address
    // 这里直接显示 shenfenId 的地址部分
    if (isRegisteredDuoqianIdentity(shenfenId)) {
      return registeredDuoqianAddressFromIdentity(shenfenId) ?? shenfenId;
    }
    return shenfenId;
  }

  String _pubkeyToSS58(String pubkeyHex) {
    final hex =
        pubkeyHex.startsWith('0x') ? pubkeyHex.substring(2) : pubkeyHex;
    final bytes = _hexDecode(hex);
    return Keyring().encodeAddress(bytes, 2027);
  }

  String _hexToSs58(String hex) {
    final bytes = _hexDecode(hex);
    return Keyring().encodeAddress(Uint8List.fromList(bytes), 2027);
  }

  String _truncateAddress(String address) {
    if (address.length <= 14) return address;
    return '${address.substring(0, 6)}...${address.substring(address.length - 6)}';
  }

  Uint8List _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = Uint8List(h.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }
}
