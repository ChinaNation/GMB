import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:isar/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';
import 'package:wuminapp_mobile/citizen/institution/institution_admin_service.dart';
import 'package:wuminapp_mobile/citizen/institution/institution_data.dart';
import 'package:wuminapp_mobile/citizen/proposal/transfer/transfer_proposal_page.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import '../institution/institution_duoqian_close_page.dart';
import '../personal/personal_admin_list_page.dart';
import '../personal/personal_duoqian_close_page.dart';
import '../personal/personal_proposal_list_section.dart';
import 'duoqian_manage_models.dart';
import 'duoqian_manage_service.dart';

/// 多签机构详情页。
///
/// 展示机构名称、SFID ID、多签地址、状态、管理员列表。
/// 右上角 "..." 提供关闭操作。
class DuoqianAccountInfoPage extends StatefulWidget {
  const DuoqianAccountInfoPage({
    super.key,
    required this.institution,
    this.isPersonal = false,
  });

  final InstitutionInfo institution;

  /// 是否为个人多签（不显示 SFID ID 行）。
  final bool isPersonal;

  @override
  State<DuoqianAccountInfoPage> createState() => _DuoqianAccountInfoPageState();
}

class _DuoqianAccountInfoPageState extends State<DuoqianAccountInfoPage> {
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
        _error = SmoldotClientManager.instance.buildUserFacingError(e);
        _loading = false;
      });
    }
  }

  // ──── 关闭 ────

  void _showDeleteMenu() {
    final title = widget.isPersonal ? '关闭个人多签' : '关闭机构多签';
    final content = widget.isPersonal
        ? '关闭个人多签将发起链上关闭提案，需要其他管理员投票通过后才会真正关闭。\n\n确定要发起关闭吗？'
        : '关闭机构多签将发起链上关闭提案，需要其他管理员投票通过后才会真正关闭。\n\n确定要发起关闭吗？';
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(title),
        content: Text(content),
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
            style: TextButton.styleFrom(foregroundColor: AppTheme.danger),
            child: const Text('发起关闭'),
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
          SnackBar(
            content:
                Text(widget.isPersonal ? '请先导入此账户的管理员钱包' : '请先导入此机构的管理员钱包'),
          ),
        );
      }
      return;
    }

    final closed = await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => widget.isPersonal
            ? PersonalDuoqianClosePage(
                institution: widget.institution,
                adminWallets: wallets,
              )
            : InstitutionDuoqianClosePage(
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
      if (widget.isPersonal) {
        await isar.personalDuoqianEntitys
            .where()
            .duoqianAddressEqualTo(widget.institution.duoqianAddress)
            .deleteAll();
      } else {
        await isar.duoqianInstitutionEntitys
            .where()
            .duoqianAddressEqualTo(widget.institution.duoqianAddress)
            .deleteAll();
      }
    });
  }

  Future<void> _openTransferProposal() async {
    final wallets = await _getAdminWallets();
    if (!mounted || wallets.isEmpty) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('未找到此多签账户的管理员钱包')),
        );
      }
      return;
    }

    final created = await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => TransferProposalPage(
          institution: widget.institution,
          icon: widget.isPersonal ? Icons.person : Icons.business,
          badgeColor: widget.isPersonal ? AppTheme.accent : AppTheme.info,
          adminWallets: wallets,
        ),
      ),
    );
    if (created == true && mounted) {
      await _load();
    }
  }

  // ──── UI ────

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: Text(
          widget.isPersonal ? '个人多签账户' : '机构多签账户',
          style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: AppTheme.primaryDark,
        elevation: 0,
        scrolledUnderElevation: 0.5,
        actions: [
          PopupMenuButton<String>(
            icon: const Icon(Icons.more_vert),
            onSelected: (value) {
              if (value == 'delete') _showDeleteMenu();
            },
            itemBuilder: (_) => [
              PopupMenuItem(
                value: 'delete',
                child: Row(
                  children: [
                    const Icon(Icons.delete_outline,
                        size: 20, color: AppTheme.danger),
                    const SizedBox(width: 8),
                    Text(
                      widget.isPersonal ? '关闭个人多签' : '关闭机构多签',
                      style: const TextStyle(color: AppTheme.danger),
                    ),
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
            const Icon(Icons.error_outline, size: 48, color: AppTheme.danger),
            const SizedBox(height: 12),
            const Text('加载失败',
                style: TextStyle(fontSize: 16, color: AppTheme.textSecondary)),
            const SizedBox(height: 6),
            Text(
              _error!,
              style:
                  const TextStyle(fontSize: 12, color: AppTheme.textTertiary),
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
        ? AppTheme.success
        : AppTheme.warning;

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
              side: const BorderSide(color: AppTheme.border),
            ),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    widget.isPersonal ? '账户信息' : '机构信息',
                    style: const TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.w700,
                      color: AppTheme.primaryDark,
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
                    duoqianSs58,
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
                    _buildInfoRow(
                        '通过阈值', '${info.threshold} / ${info.adminCount}'),
                  ],
                ],
              ),
            ),
          ),

          const SizedBox(height: 16),
          _buildTransferEntryCard(),

          const SizedBox(height: 16),

          // 管理员列表(折叠成单行,点击进入子页)
          _buildAdminEntryCard(info),

          // 个人多签提案列表(req 5):活跃 + 历史(本机 Isar 永久保留终态记录)
          if (widget.isPersonal) ...[
            const SizedBox(height: 16),
            FutureBuilder<List<WalletProfile>>(
              future: _getAdminWallets(),
              builder: (context, snapshot) {
                final wallets = snapshot.data ?? const <WalletProfile>[];
                return PersonalProposalListSection(
                  institution: widget.institution,
                  adminWallets: wallets,
                );
              },
            ),
          ],
        ],
      ),
    );
  }

  /// 管理员列表入口卡片(req 1):点击进入完整管理员列表页(个人多签
  /// 页内含"激活"按钮的三态交互;机构多签复用同 [DuoqianAccountInfoPage]
  /// 路径,本入口仅个人多签开启)。
  Widget _buildAdminEntryCard(DuoqianAccountInfo? info) {
    final adminCount = _adminPubkeys.length;
    final threshold = info?.threshold;
    final subtitle = threshold == null
        ? '$adminCount 人'
        : '$adminCount 人 · 阈值 $threshold/$adminCount';

    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: const BorderSide(color: AppTheme.border),
      ),
      child: ListTile(
        leading: const Icon(Icons.group_outlined,
            size: 22, color: AppTheme.primaryDark),
        title: const Text(
          '管理员列表',
          style: TextStyle(
            fontSize: 15,
            fontWeight: FontWeight.w600,
            color: AppTheme.textPrimary,
          ),
        ),
        subtitle: Text(
          subtitle,
          style: const TextStyle(fontSize: 12, color: AppTheme.textTertiary),
        ),
        trailing: const Icon(Icons.chevron_right,
            size: 20, color: AppTheme.textTertiary),
        onTap: () => _openAdminListPage(info),
      ),
    );
  }

  Future<void> _openAdminListPage(DuoqianAccountInfo? info) async {
    if (!widget.isPersonal) {
      // 机构多签暂沿用旧的平铺渲染,此处仅 personal 入口开放新子页;
      // 后续机构页改造会单独引入。
      return;
    }
    final wallets = await _getAdminWallets();
    if (!mounted) return;
    final creator = await _resolvePersonalCreatorPubkeyHex();
    if (!mounted) return;
    await Navigator.push(
      context,
      MaterialPageRoute(
        builder: (_) => PersonalAdminListPage(
          institution: widget.institution,
          duoqianStatus: info?.status ?? DuoqianStatus.pending,
          adminPubkeys: _adminPubkeys,
          adminWallets: wallets,
          creatorPubkeyHex: creator,
        ),
      ),
    );
    // 子页可能完成投票 → 刷新本页状态(可能多签已激活)
    if (mounted) await _load();
  }

  /// 从本机 Isar 读取个人多签创建者公钥 hex。
  /// req 3 未实现时,只有创建者本机有此记录;非创建者打开子页 creatorPubkeyHex 为 null
  /// (届时所有 admin 都按"非创建者"渲染,语义略损但不阻塞主流程)。
  Future<String?> _resolvePersonalCreatorPubkeyHex() async {
    try {
      final isar = await WalletIsar.instance.db();
      final entity = await isar.personalDuoqianEntitys
          .filter()
          .duoqianAddressEqualTo(widget.institution.duoqianAddress)
          .findFirst();
      if (entity == null) return null;
      // creatorAddress 是 SS58,转 pubkey hex(小写,无 0x)。
      final pair =
          Keyring().decodeAddress(entity.creatorAddress);
      return pair
          .map((b) => b.toRadixString(16).padLeft(2, '0'))
          .join()
          .toLowerCase();
    } catch (_) {
      return null;
    }
  }

  Widget _buildTransferEntryCard() {
    // 待激活的多签账户(链上提案尚未通过 → DuoqianStatus.pending)不允许发起转账提案,
    // 整张卡片置灰显示但不响应点击,文案提示用户先完成激活。
    final canTransfer = _accountInfo?.status == DuoqianStatus.active;
    final accentColor =
        canTransfer ? AppTheme.primaryDark : AppTheme.textTertiary;
    final subtitle = canTransfer
        ? '从当前多签账户发起链上转账'
        : '账户尚未激活,无法发起转账';

    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: accentColor.withValues(alpha: 0.15)),
      ),
      child: InkWell(
        onTap: canTransfer ? _openTransferProposal : null,
        borderRadius: BorderRadius.circular(12),
        child: Opacity(
          opacity: canTransfer ? 1.0 : 0.5,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
            child: Row(
              children: [
                Container(
                  width: 38,
                  height: 38,
                  decoration: BoxDecoration(
                    color: accentColor.withValues(alpha: 0.08),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Icon(
                    Icons.send_outlined,
                    size: 19,
                    color: accentColor,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text(
                        '发起转账提案',
                        style: TextStyle(
                          fontSize: 15,
                          fontWeight: FontWeight.w600,
                          color: AppTheme.textPrimary,
                        ),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        subtitle,
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
            style: const TextStyle(fontSize: 13, color: AppTheme.textSecondary),
          ),
        ),
        Expanded(
          child: Text(
            value,
            style: TextStyle(
              fontSize: 13,
              color: valueColor ?? AppTheme.textPrimary,
              fontWeight: valueColor != null ? FontWeight.w600 : null,
            ),
          ),
        ),
        if (onCopy != null)
          GestureDetector(
            onTap: onCopy,
            child:
                const Icon(Icons.copy, size: 16, color: AppTheme.textTertiary),
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

  String _hexToSs58(String hex) {
    final bytes = _hexDecode(hex);
    return Keyring().encodeAddress(Uint8List.fromList(bytes), 2027);
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
