import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/citizen/institution/institution_data.dart';
import 'package:wuminapp_mobile/citizen/institution/institution_detail_page.dart';
import 'package:wuminapp_mobile/citizen/shared/proposal_context.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/ui/page_transitions.dart';
import 'package:wuminapp_mobile/ui/widgets/pressable_card.dart';

/// 公民 Tab 下的机构二级页。
///
/// 只负责机构分类、机构排序和进入机构详情；提案发起与投票事件仍由机构详情页承接。
class InstitutionListPage extends StatefulWidget {
  const InstitutionListPage({
    super.key,
    required this.nationalCouncil,
    required this.provincialCouncils,
    required this.provincialBanks,
  });

  final List<InstitutionInfo> nationalCouncil;
  final List<InstitutionInfo> provincialCouncils;
  final List<InstitutionInfo> provincialBanks;

  @override
  State<InstitutionListPage> createState() => _InstitutionListPageState();
}

class _InstitutionListPageState extends State<InstitutionListPage> {
  /// 对列表按“管理员机构优先”排序。
  List<InstitutionInfo> _sorted(List<InstitutionInfo> list) {
    final sorted = List<InstitutionInfo>.from(list);
    sorted.sort((a, b) {
      final aAdmin = ProposalContextResolver.isAdminInstitution(a.shenfenId);
      final bAdmin = ProposalContextResolver.isAdminInstitution(b.shenfenId);
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
          '机构分类',
          style: TextStyle(
            fontSize: 22,
            fontWeight: FontWeight.w700,
            color: AppTheme.textPrimary,
          ),
        ),
        const SizedBox(height: 4),
        const Text(
          '查看各级机构信息与治理提案',
          style: TextStyle(
            fontSize: 13,
            color: AppTheme.textSecondary,
          ),
        ),
        const SizedBox(height: 20),
        _InstitutionSection(
          title: '国储会',
          icon: Icons.account_balance,
          badgeColor: AppTheme.primaryDark,
          institutions: widget.nationalCouncil,
          onReturnFromDetail: () => setState(() {}),
        ),
        _InstitutionSection(
          title: '省储会',
          icon: Icons.groups_2_outlined,
          badgeColor: AppTheme.primary,
          institutions: _sorted(widget.provincialCouncils),
          onReturnFromDetail: () => setState(() {}),
        ),
        _InstitutionSection(
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

class _InstitutionSection extends StatelessWidget {
  const _InstitutionSection({
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
                  inst.shenfenId,
                );
                return _InstitutionCard(
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

class _InstitutionCard extends StatelessWidget {
  const _InstitutionCard({
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
