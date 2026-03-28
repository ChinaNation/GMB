import 'package:flutter/material.dart';

import 'duoqian_close_proposal_page.dart';
import 'duoqian_create_proposal_page.dart';
import 'institution_data.dart';
import 'runtime_upgrade_page.dart';
import 'transfer_proposal_page.dart';
import 'transfer_proposal_service.dart';
import '../wallet/core/wallet_manager.dart';

/// 提案类型选择页。
///
/// 根据机构类型条件显示可发起的提案类型：
/// - 所有机构：转账、换管理员、决议销毁
/// - 仅国储会（NRC）：决议发行、验证密钥、状态升级
class ProposalTypesPage extends StatelessWidget {
  const ProposalTypesPage({
    super.key,
    required this.institution,
    required this.icon,
    required this.badgeColor,
    required this.adminWallets,
  });

  final InstitutionInfo institution;
  final IconData icon;
  final Color badgeColor;

  /// 当前用户导入的、属于此机构的管理员钱包列表。
  final List<WalletProfile> adminWallets;

  static const Color _inkGreen = Color(0xFF0B3D2E);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '发起提案',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: _inkGreen,
        elevation: 0,
        scrolledUnderElevation: 0.5,
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          // 机构信息
          Padding(
            padding: const EdgeInsets.only(bottom: 16),
            child: Row(
              children: [
                Container(
                  width: 36,
                  height: 36,
                  decoration: BoxDecoration(
                    color: badgeColor.withValues(alpha: 0.12),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Icon(icon, size: 18, color: badgeColor),
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Text(
                    institution.name,
                    style: const TextStyle(
                      fontSize: 15,
                      fontWeight: FontWeight.w600,
                      color: _inkGreen,
                    ),
                  ),
                ),
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                  decoration: BoxDecoration(
                    color: badgeColor.withValues(alpha: 0.10),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Text(
                    OrgType.label(institution.orgType),
                    style: TextStyle(
                      fontSize: 11,
                      color: badgeColor,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
              ],
            ),
          ),

          // ──── 通用提案类型（所有机构） ────
          _buildSectionTitle('通用提案'),
          const SizedBox(height: 8),
          _ProposalTypeCard(
            icon: Icons.send_outlined,
            title: '转账',
            subtitle: '从机构多签账户发起转账',
            color: const Color(0xFF176650),
            onTap: () => _checkAndOpenProposal(
                context,
                () => TransferProposalPage(
                      institution: institution,
                      icon: icon,
                      badgeColor: badgeColor,
                      adminWallets: adminWallets,
                    )),
          ),
          const SizedBox(height: 8),
          _ProposalTypeCard(
            icon: Icons.swap_horiz,
            title: '换管理员',
            subtitle: '提议更换本机构管理员',
            color: const Color(0xFF2E7D5B),
            onTap: () => _checkAndOpenProposal(context, null, name: '换管理员'),
          ),
          const SizedBox(height: 8),
          _ProposalTypeCard(
            icon: Icons.delete_outline,
            title: '决议销毁',
            subtitle: '提议销毁机构持有的资产',
            color: const Color(0xFFB71C1C),
            onTap: () => _checkAndOpenProposal(context, null, name: '决议销毁'),
          ),

          // ──── 注册多签机构专属提案类型 ────
          if (institution.orgType == OrgType.duoqian) ...[
            const SizedBox(height: 20),
            _buildSectionTitle('多签管理'),
            const SizedBox(height: 8),
            _ProposalTypeCard(
              icon: Icons.group_add,
              title: '创建多签',
              subtitle: '发起创建多签账户提案',
              color: const Color(0xFF1565C0),
              onTap: () => _checkAndOpenProposal(
                context,
                () => DuoqianCreateProposalPage(
                  institution: institution,
                  adminWallets: adminWallets,
                ),
              ),
            ),
            const SizedBox(height: 8),
            _ProposalTypeCard(
              icon: Icons.group_remove,
              title: '关闭多签',
              subtitle: '发起关闭多签账户提案，资金转入指定受益人',
              color: const Color(0xFFB71C1C),
              onTap: () => _checkAndOpenProposal(
                context,
                () => DuoqianCloseProposalPage(
                  institution: institution,
                  adminWallets: adminWallets,
                ),
              ),
            ),
          ],

          // ──── 国储会专属提案类型 ────
          if (institution.orgType == OrgType.nrc) ...[
            const SizedBox(height: 20),
            _buildSectionTitle('国储会专属提案'),
            const SizedBox(height: 8),
            _ProposalTypeCard(
              icon: Icons.account_balance,
              title: '决议发行',
              subtitle: '发起公民币发行决议，需联合投票+公民投票',
              color: _inkGreen,
              onTap: () => _checkAndOpenProposal(context, null, name: '决议发行'),
            ),
            const SizedBox(height: 8),
            _ProposalTypeCard(
              icon: Icons.vpn_key_outlined,
              title: '验证密钥',
              subtitle: '更换 GRANDPA 共识验证密钥',
              color: const Color(0xFF4527A0),
              onTap: () => _checkAndOpenProposal(context, null, name: '验证密钥'),
            ),
            const SizedBox(height: 8),
            _ProposalTypeCard(
              icon: Icons.arrow_upward,
              title: '状态升级',
              subtitle: 'Runtime 升级，需联合投票+公民投票',
              color: const Color(0xFF1565C0),
              onTap: () => _checkAndOpenProposal(
                context,
                () => RuntimeUpgradePage(adminWallets: adminWallets),
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildSectionTitle(String title) {
    return Text(
      title,
      style: const TextStyle(
        fontSize: 14,
        fontWeight: FontWeight.w600,
        color: Colors.grey,
      ),
    );
  }

  /// 检查活跃提案数，未达上限则打开页面，达上限则弹窗提示。
  /// [pageBuilder] 为 null 时表示该功能开发中。
  Future<void> _checkAndOpenProposal(
    BuildContext context,
    Widget Function()? pageBuilder, {
    String? name,
  }) async {
    try {
      final service = TransferProposalService();
      final activeIds =
          await service.fetchActiveProposalIds(institution.shenfenId);
      if (!context.mounted) return;

      if (activeIds.length >=
          TransferProposalService.maxActiveProposalsPerInstitution) {
        showDialog(
          context: context,
          builder: (ctx) => AlertDialog(
            title: const Text('提案数量已达上限'),
            content: Text(
              '本机构当前有 ${activeIds.length} 个活跃提案，'
              '已达上限 ${TransferProposalService.maxActiveProposalsPerInstitution} 个。'
              '请等待现有提案完成后再发起新提案。',
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(ctx),
                child: const Text('知道了'),
              ),
            ],
          ),
        );
        return;
      }

      if (pageBuilder != null) {
        final created = await Navigator.push<bool>(
          context,
          MaterialPageRoute(builder: (_) => pageBuilder()),
        );
        if (created == true && context.mounted) {
          Navigator.of(context).pop(true);
        }
      } else {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('${name ?? "该"}功能开发中')),
        );
      }
    } catch (e) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('查询失败：$e')),
      );
    }
  }
}

class _ProposalTypeCard extends StatelessWidget {
  const _ProposalTypeCard({
    required this.icon,
    required this.title,
    required this.subtitle,
    required this.color,
    required this.onTap,
  });

  final IconData icon;
  final String title;
  final String subtitle;
  final Color color;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: color.withValues(alpha: 0.15)),
      ),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
          child: Row(
            children: [
              Container(
                width: 40,
                height: 40,
                decoration: BoxDecoration(
                  color: color.withValues(alpha: 0.10),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(icon, size: 20, color: color),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      title,
                      style: TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: color,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      subtitle,
                      style: TextStyle(fontSize: 12, color: Colors.grey[500]),
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
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
}
