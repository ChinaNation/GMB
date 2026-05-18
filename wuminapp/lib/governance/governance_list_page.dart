import 'dart:async';

import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/governance/organization-manage/institution_detail_page.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_context.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/ui/page_transitions.dart';
import 'package:wuminapp_mobile/ui/widgets/pressable_card.dart';

const String governanceProvincialCouncilOrderPrefsKey =
    'governance.institution_order.prc.v1';
const String governanceProvincialBankOrderPrefsKey =
    'governance.institution_order.prb.v1';

@visibleForTesting
List<InstitutionInfo> applyGovernanceInstitutionOrder(
  List<InstitutionInfo> source,
  List<String>? savedOrder,
) {
  final byId = <String, InstitutionInfo>{
    for (final institution in source) institution.sfidNumber: institution,
  };
  final ordered = <InstitutionInfo>[];
  final used = <String>{};

  if (savedOrder != null) {
    for (final sfidNumber in savedOrder) {
      final institution = byId[sfidNumber];
      if (institution != null && used.add(sfidNumber)) {
        ordered.add(institution);
      }
    }
  }

  // 中文注释：静态注册表未来若有新增机构，本机旧顺序里没有的项必须补回末尾。
  for (final institution in source) {
    if (used.add(institution.sfidNumber)) {
      ordered.add(institution);
    }
  }
  return ordered;
}

@visibleForTesting
List<InstitutionInfo> reorderGovernanceInstitutions(
  List<InstitutionInfo> source,
  int fromIndex,
  int toIndex,
) {
  if (fromIndex < 0 ||
      fromIndex >= source.length ||
      toIndex < 0 ||
      toIndex >= source.length ||
      fromIndex == toIndex) {
    return List<InstitutionInfo>.of(source);
  }
  final next = List<InstitutionInfo>.of(source);
  final item = next.removeAt(fromIndex);
  next.insert(toIndex.clamp(0, next.length), item);
  return next;
}

enum _GovernanceSectionKind {
  nationalCouncil,
  provincialCouncil,
  provincialBank,
}

class _GovernanceDragData {
  const _GovernanceDragData({
    required this.sectionKind,
    required this.index,
  });

  final _GovernanceSectionKind sectionKind;
  final int index;
}

/// 治理 tab 二级页：展示治理类机构（国储会/省储会/省储行）分类与详情入口。
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
  late List<InstitutionInfo> _provincialCouncils;
  late List<InstitutionInfo> _provincialBanks;
  bool _provincialCouncilsExpanded = false;
  bool _provincialBanksExpanded = false;

  @override
  void initState() {
    super.initState();
    _resetInstitutionLists();
    unawaited(_loadLocalInstitutionOrder());
  }

  @override
  void didUpdateWidget(covariant GovernanceListPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.provincialCouncils != widget.provincialCouncils ||
        oldWidget.provincialBanks != widget.provincialBanks) {
      _resetInstitutionLists();
      unawaited(_loadLocalInstitutionOrder());
    }
  }

  void _resetInstitutionLists() {
    _provincialCouncils = List<InstitutionInfo>.of(widget.provincialCouncils);
    _provincialBanks = List<InstitutionInfo>.of(widget.provincialBanks);
  }

  Future<void> _loadLocalInstitutionOrder() async {
    final prefs = await SharedPreferences.getInstance();
    final councils = applyGovernanceInstitutionOrder(
      widget.provincialCouncils,
      prefs.getStringList(governanceProvincialCouncilOrderPrefsKey),
    );
    final banks = applyGovernanceInstitutionOrder(
      widget.provincialBanks,
      prefs.getStringList(governanceProvincialBankOrderPrefsKey),
    );
    if (!mounted) return;
    setState(() {
      _provincialCouncils = councils;
      _provincialBanks = banks;
    });
  }

  Future<void> _reorderInstitution(
    _GovernanceSectionKind sectionKind,
    int fromIndex,
    int toIndex,
  ) async {
    final prefsKey = switch (sectionKind) {
      _GovernanceSectionKind.provincialCouncil =>
        governanceProvincialCouncilOrderPrefsKey,
      _GovernanceSectionKind.provincialBank =>
        governanceProvincialBankOrderPrefsKey,
      _GovernanceSectionKind.nationalCouncil => null,
    };
    if (prefsKey == null) return;

    late final List<InstitutionInfo> next;
    setState(() {
      if (sectionKind == _GovernanceSectionKind.provincialCouncil) {
        next = reorderGovernanceInstitutions(
          _provincialCouncils,
          fromIndex,
          toIndex,
        );
        _provincialCouncils = next;
      } else {
        next = reorderGovernanceInstitutions(
          _provincialBanks,
          fromIndex,
          toIndex,
        );
        _provincialBanks = next;
      }
    });

    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.setStringList(
        prefsKey,
        next.map((institution) => institution.sfidNumber).toList(),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('保存治理机构顺序失败：$e')),
      );
    }
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
          sectionKind: _GovernanceSectionKind.nationalCouncil,
          title: '国储会',
          icon: Icons.account_balance,
          badgeColor: AppTheme.primaryDark,
          institutions: widget.nationalCouncil,
          onReturnFromDetail: () => setState(() {}),
        ),
        _GovernanceSection(
          sectionKind: _GovernanceSectionKind.provincialCouncil,
          title: '省储会',
          icon: Icons.groups_2_outlined,
          badgeColor: AppTheme.primary,
          institutions: _provincialCouncils,
          collapsible: true,
          expanded: _provincialCouncilsExpanded,
          onToggleExpanded: () {
            setState(() {
              _provincialCouncilsExpanded = !_provincialCouncilsExpanded;
            });
          },
          onReorder: (fromIndex, toIndex) => _reorderInstitution(
            _GovernanceSectionKind.provincialCouncil,
            fromIndex,
            toIndex,
          ),
          onReturnFromDetail: () => setState(() {}),
        ),
        _GovernanceSection(
          sectionKind: _GovernanceSectionKind.provincialBank,
          title: '省储行',
          icon: Icons.account_balance_wallet_outlined,
          badgeColor: AppTheme.accent,
          institutions: _provincialBanks,
          collapsible: true,
          expanded: _provincialBanksExpanded,
          onToggleExpanded: () {
            setState(() {
              _provincialBanksExpanded = !_provincialBanksExpanded;
            });
          },
          onReorder: (fromIndex, toIndex) => _reorderInstitution(
            _GovernanceSectionKind.provincialBank,
            fromIndex,
            toIndex,
          ),
          onReturnFromDetail: () => setState(() {}),
        ),
      ],
    );
  }
}

class _GovernanceSection extends StatelessWidget {
  const _GovernanceSection({
    required this.sectionKind,
    required this.title,
    required this.icon,
    required this.badgeColor,
    required this.institutions,
    this.collapsible = false,
    this.expanded = true,
    this.onToggleExpanded,
    this.onReorder,
    this.onReturnFromDetail,
  });

  final _GovernanceSectionKind sectionKind;
  final String title;
  final IconData icon;
  final Color badgeColor;
  final List<InstitutionInfo> institutions;
  final bool collapsible;
  final bool expanded;
  final VoidCallback? onToggleExpanded;
  final Future<void> Function(int fromIndex, int toIndex)? onReorder;
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
            if (collapsible) ...[
              const Spacer(),
              IconButton(
                key: ValueKey(
                  'governance_section_toggle_${sectionKind.name}',
                ),
                tooltip: expanded ? '折叠$title' : '展开$title',
                visualDensity: VisualDensity.compact,
                constraints: const BoxConstraints.tightFor(
                  width: 32,
                  height: 32,
                ),
                onPressed: onToggleExpanded,
                icon: Icon(
                  expanded ? Icons.keyboard_arrow_down : Icons.chevron_right,
                  size: 24,
                  color: AppTheme.textSecondary,
                ),
              ),
            ],
          ],
        ),
        if (collapsible && !expanded) const SizedBox(height: 16),
        if (!collapsible || expanded) ...[
          const SizedBox(height: 10),
          LayoutBuilder(
            builder: (context, constraints) {
              if (constraints.maxWidth <= 0) {
                return const SizedBox.shrink();
              }
              if (sectionKind == _GovernanceSectionKind.nationalCouncil) {
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    for (final inst in institutions)
                      SizedBox(
                        key: ValueKey(
                          'governance_national_card_${inst.sfidNumber}',
                        ),
                        height: 76,
                        child: _GovernanceCard(
                          institution: inst,
                          icon: icon,
                          badgeColor: badgeColor,
                          isAdmin: ProposalContextResolver.isAdminInstitution(
                            inst.sfidNumber,
                          ),
                          onReturnFromDetail: onReturnFromDetail,
                        ),
                      ),
                  ],
                );
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
                  final reorder = onReorder;
                  if (reorder == null) {
                    return _GovernanceCard(
                      institution: inst,
                      icon: icon,
                      badgeColor: badgeColor,
                      isAdmin: isAdmin,
                      onReturnFromDetail: onReturnFromDetail,
                    );
                  }
                  final card = _GovernanceCard(
                    institution: inst,
                    icon: icon,
                    badgeColor: badgeColor,
                    isAdmin: isAdmin,
                    pressAnimationEnabled: false,
                    onReturnFromDetail: onReturnFromDetail,
                  );
                  return _GovernanceReorderableCard(
                    sectionKind: sectionKind,
                    index: index,
                    institution: inst,
                    icon: icon,
                    badgeColor: badgeColor,
                    isAdmin: isAdmin,
                    onReturnFromDetail: onReturnFromDetail,
                    onReorder: reorder,
                    child: card,
                  );
                },
              );
            },
          ),
          const SizedBox(height: 16),
        ],
      ],
    );
  }
}

class _GovernanceReorderableCard extends StatelessWidget {
  const _GovernanceReorderableCard({
    required this.sectionKind,
    required this.index,
    required this.institution,
    required this.icon,
    required this.badgeColor,
    required this.isAdmin,
    required this.onReorder,
    required this.child,
    this.onReturnFromDetail,
  });

  final _GovernanceSectionKind sectionKind;
  final int index;
  final InstitutionInfo institution;
  final IconData icon;
  final Color badgeColor;
  final bool isAdmin;
  final Future<void> Function(int fromIndex, int toIndex) onReorder;
  final Widget child;
  final VoidCallback? onReturnFromDetail;

  @override
  Widget build(BuildContext context) {
    return DragTarget<_GovernanceDragData>(
      onWillAcceptWithDetails: (details) {
        final data = details.data;
        return data.sectionKind == sectionKind && data.index != index;
      },
      onAcceptWithDetails: (details) {
        final data = details.data;
        unawaited(onReorder(data.index, index));
      },
      builder: (context, candidateData, rejectedData) {
        final highlighted = candidateData.isNotEmpty;
        return AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          decoration: highlighted
              ? BoxDecoration(
                  borderRadius: BorderRadius.circular(AppTheme.radiusMd),
                  border: Border.all(color: badgeColor, width: 1.5),
                )
              : null,
          child: LongPressDraggable<_GovernanceDragData>(
            data: _GovernanceDragData(
              sectionKind: sectionKind,
              index: index,
            ),
            feedback: Material(
              color: Colors.transparent,
              child: SizedBox(
                width: 190,
                height: 64,
                child: Opacity(
                  opacity: 0.92,
                  child: _GovernanceCard(
                    institution: institution,
                    icon: icon,
                    badgeColor: badgeColor,
                    isAdmin: isAdmin,
                    navigationEnabled: false,
                    pressAnimationEnabled: false,
                    onReturnFromDetail: onReturnFromDetail,
                  ),
                ),
              ),
            ),
            childWhenDragging: Opacity(opacity: 0.35, child: child),
            child: child,
          ),
        );
      },
    );
  }
}

class _GovernanceCard extends StatelessWidget {
  const _GovernanceCard({
    required this.institution,
    required this.icon,
    required this.badgeColor,
    this.isAdmin = false,
    this.navigationEnabled = true,
    this.pressAnimationEnabled = true,
    this.onReturnFromDetail,
  });

  final InstitutionInfo institution;
  final IconData icon;
  final Color badgeColor;
  final bool isAdmin;
  final bool navigationEnabled;
  final bool pressAnimationEnabled;
  final VoidCallback? onReturnFromDetail;

  @override
  Widget build(BuildContext context) {
    final effectiveColor = isAdmin ? AppTheme.success : badgeColor;
    final card = Container(
      decoration: AppTheme.cardDecoration(selected: isAdmin),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: navigationEnabled
              ? () async {
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
                }
              : null,
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
    );
    if (!pressAnimationEnabled) {
      return card;
    }
    return PressableCard(child: card);
  }
}
