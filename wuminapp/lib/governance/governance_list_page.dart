import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/common/institution_info.dart';
import 'package:wuminapp_mobile/governance/organization-manage/institution_detail_page.dart';
import 'package:wuminapp_mobile/common/proposal/proposal_context.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/ui/page_transitions.dart';
import 'package:wuminapp_mobile/ui/widgets/pressable_card.dart';

/// 治理 tab 二级页：展示治理类机构（国储会/省储会/省储行）分类、排序与详情入口。
///
/// 提案发起与投票事件仍由机构详情页承接。
class GovernanceListPage extends StatefulWidget {
  const GovernanceListPage({
    super.key,
    required this.nationalCouncil,
    required this.provincialCouncils,
    required this.provincialBanks,
  });

  final List<InstitutionInfo> nationalCouncil;
  final List<InstitutionInfo> provincialCouncils;
  final List<InstitutionInfo> provincialBanks;

  @override
  State<GovernanceListPage> createState() => _GovernanceListPageState();
}

class _GovernanceListPageState extends State<GovernanceListPage> {
  /// 对列表按“管理员机构优先”排序。
  List<InstitutionInfo> _sorted(List<InstitutionInfo> list) {
    final sorted = List<InstitutionInfo>.from(list);
    sorted.sort((a, b) {
      final aAdmin = ProposalContextResolver.isAdminInstitution(a.sfidNumber);
      final bAdmin = ProposalContextResolver.isAdminInstitution(b.sfidNumber);
      if (aAdmin && !bAdmin) return -1;
      if (!aAdmin && bAdmin) return 1;
      return 0;
    });
    return sorted;
  }

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 24),
      children: [
        const Text(
          '治理机构',
          style: TextStyle(
            fontSize: 22,
            fontWeight: FontWeight.w700,
            color: AppTheme.textPrimary,
          ),
        ),
        const SizedBox(height: 20),
        _GovernanceSection(
          title: '国储会',
          icon: Icons.account_balance,
          badgeColor: AppTheme.primaryDark,
          institutions: widget.nationalCouncil,
          onReturnFromDetail: () => setState(() {}),
        ),
        _GovernanceSection(
          title: '省储会',
          icon: Icons.groups_2_outlined,
          badgeColor: AppTheme.primary,
          institutions: _sorted(widget.provincialCouncils),
          onReturnFromDetail: () => setState(() {}),
        ),
        _GovernanceSection(
          title: '省储行',
          icon: Icons.account_balance_wallet_outlined,
          badgeColor: AppTheme.accent,
          institutions: _sorted(widget.provincialBanks),
          onReturnFromDetail: () => setState(() {}),
        ),
      ],
    );
  }
}

class _GovernanceSection extends StatelessWidget {
  const _GovernanceSection({
    required this.title,
    required this.icon,
    required this.badgeColor,
    required this.institutions,
    this.onReturnFromDetail,
  });

  final String title;
  final IconData icon;
  final Color badgeColor;
  final List<InstitutionInfo> institutions;
  final VoidCallback? onReturnFromDetail;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Container(
              width: 28,
              height: 28,
              decoration: BoxDecoration(
                color: badgeColor.withAlpha(20),
                borderRadius: BorderRadius.circular(7),
              ),
              child: Icon(icon, size: 16, color: badgeColor),
            ),
            const SizedBox(width: 10),
            Text(
              '$title（${institutions.length}）',
              style: const TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w700,
                color: AppTheme.textPrimary,
              ),
            ),
          ],
        ),
        const SizedBox(height: 10),
        LayoutBuilder(
          builder: (context, constraints) {
            if (constraints.maxWidth <= 0) {
              return const SizedBox.shrink();
            }
            // 机构列表固定一行两列，避免不同 Android 机型出现列数漂移。
            const crossAxisCount = 2;
            final childAspectRatio = constraints.maxWidth < 360 ? 2.6 : 2.9;
            return GridView.builder(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              itemCount: institutions.length,
              gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                crossAxisCount: crossAxisCount,
                mainAxisSpacing: 8,
                crossAxisSpacing: 8,
                childAspectRatio: childAspectRatio,
              ),
              itemBuilder: (context, index) {
                final inst = institutions[index];
                final isAdmin = ProposalContextResolver.isAdminInstitution(
                  inst.sfidNumber,
                );
                return _GovernanceCard(
                  institution: inst,
                  icon: icon,
                  badgeColor: badgeColor,
                  isAdmin: isAdmin,
                  onReturnFromDetail: onReturnFromDetail,
                );
              },
            );
          },
        ),
        const SizedBox(height: 16),
      ],
    );
  }
}

class _GovernanceCard extends StatelessWidget {
  const _GovernanceCard({
    required this.institution,
    required this.icon,
    required this.badgeColor,
    this.isAdmin = false,
    this.onReturnFromDetail,
  });

  final InstitutionInfo institution;
  final IconData icon;
  final Color badgeColor;
  final bool isAdmin;
  final VoidCallback? onReturnFromDetail;

  @override
  Widget build(BuildContext context) {
    final effectiveColor = isAdmin ? AppTheme.success : badgeColor;
    return PressableCard(
      child: Container(
        decoration: AppTheme.cardDecoration(selected: isAdmin),
        child: Material(
          color: Colors.transparent,
          child: InkWell(
            onTap: () async {
              await Navigator.of(context).push(
                FadeSlideRoute(
                  page: InstitutionDetailPage(
                    institution: institution,
                    icon: icon,
                    badgeColor: effectiveColor,
                  ),
                ),
              );
              onReturnFromDetail?.call();
            },
            borderRadius: BorderRadius.circular(AppTheme.radiusMd),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
              child: Row(
                children: [
                  Container(
                    width: 28,
                    height: 28,
                    decoration: BoxDecoration(
                      color: effectiveColor.withAlpha(20),
                      borderRadius: BorderRadius.circular(7),
                    ),
                    child: Icon(icon, size: 14, color: effectiveColor),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      institution.name,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.w500,
                        color: AppTheme.textPrimary,
                      ),
                    ),
                  ),
                  const Icon(
                    Icons.chevron_right,
                    size: 16,
                    color: AppTheme.textTertiary,
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
