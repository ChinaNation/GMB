import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/citizen/institution/institution.dart';
import 'package:citizenapp/citizen/institution/institution_classification.dart';
import 'package:citizenapp/citizen/institution/institution_detail_page.dart';
import 'package:citizenapp/citizen/institution/institution_repository.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart' show OrgType;
import 'package:citizenapp/citizen/shared/proposal/proposal_context.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/page_transitions.dart';
import 'package:citizenapp/ui/widgets/pressable_card.dart';

const String governanceProvincialCouncilOrderPrefsKey =
    'governance.institution_order.prc.v1';
const String governanceProvincialBankOrderPrefsKey =
    'governance.institution_order.prb.v1';
const String _governanceProvincialCouncilIconAsset =
    'assets/icons/government-line.svg';
const String _governanceProvincialBankIconAsset = 'assets/icons/bank.svg';

@visibleForTesting
List<Institution> applyGovernanceInstitutionOrder(
  List<Institution> source,
  List<String>? savedOrder,
) {
  final byId = <String, Institution>{
    for (final institution in source) institution.cidNumber: institution,
  };
  final ordered = <Institution>[];
  final used = <String>{};

  if (savedOrder != null) {
    for (final cidNumber in savedOrder) {
      final institution = byId[cidNumber];
      if (institution != null && used.add(cidNumber)) {
        ordered.add(institution);
      }
    }
  }

  // 中文注释：静态注册表未来若有新增机构，本机旧顺序里没有的项必须补回末尾。
  for (final institution in source) {
    if (used.add(institution.cidNumber)) {
      ordered.add(institution);
    }
  }
  return ordered;
}

@visibleForTesting
List<Institution> reorderGovernanceInstitutions(
  List<Institution> source,
  int fromIndex,
  int toIndex,
) {
  if (fromIndex < 0 ||
      fromIndex >= source.length ||
      toIndex < 0 ||
      toIndex >= source.length ||
      fromIndex == toIndex) {
    return List<Institution>.of(source);
  }
  final next = List<Institution>.of(source);
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

/// 治理 tab 视图(ADR-028 P2)：从统一目录按机构码过滤治理类机构(国储会/省储会/
/// 省储行),分类展示 + 拖拽排序;详情入口走统一机构详情页。替代旧 GovernanceListPage
/// 静态烘焙注册表「列表」承载。
///
/// 提案发起与投票事件仍由机构详情页承接。
class GovernanceTab extends StatefulWidget {
  const GovernanceTab({super.key, this.repository});

  /// 统一机构仓库门面(测试注入;默认 [InstitutionRepository])。
  final InstitutionRepository? repository;

  @override
  State<GovernanceTab> createState() => _GovernanceTabState();
}

class _GovernanceTabState extends State<GovernanceTab> {
  late final InstitutionRepository _repo =
      widget.repository ?? InstitutionRepository();

  /// 治理 tab 机构码集合(储备治理三档)。
  static const Set<String> _governanceCodes = kGovernanceCodes;

  List<Institution> _national = const [];
  List<Institution> _provincialCouncils = const [];
  List<Institution> _provincialBanks = const [];
  bool _loading = true;
  bool _provincialCouncilsExpanded = false;
  bool _provincialBanksExpanded = false;

  @override
  void initState() {
    super.initState();
    unawaited(_loadInstitutions());
  }

  /// 从统一目录按机构码取治理机构,按 orgType 分三组,再叠加本机保存的拖拽顺序。
  Future<void> _loadInstitutions() async {
    final all = await _repo.listByCodes(_governanceCodes);
    final national =
        all.where((i) => i.orgType == OrgType.nrc).toList(growable: false);
    final councilsRaw =
        all.where((i) => i.orgType == OrgType.prc).toList(growable: false);
    final banksRaw =
        all.where((i) => i.orgType == OrgType.prb).toList(growable: false);
    final prefs = await SharedPreferences.getInstance();
    final councils = applyGovernanceInstitutionOrder(
      councilsRaw,
      prefs.getStringList(governanceProvincialCouncilOrderPrefsKey),
    );
    final banks = applyGovernanceInstitutionOrder(
      banksRaw,
      prefs.getStringList(governanceProvincialBankOrderPrefsKey),
    );
    if (!mounted) return;
    setState(() {
      _national = national;
      _provincialCouncils = councils;
      _provincialBanks = banks;
      _loading = false;
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

    late final List<Institution> next;
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
        next.map((institution) => institution.cidNumber).toList(),
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
    if (_loading) {
      return const Center(child: CircularProgressIndicator(strokeWidth: 2));
    }
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
          institutions: _national,
          onReturnFromDetail: () => setState(() {}),
        ),
        _GovernanceSection(
          sectionKind: _GovernanceSectionKind.provincialCouncil,
          title: '省储会',
          icon: Icons.groups_2_outlined,
          iconAsset: _governanceProvincialCouncilIconAsset,
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
          iconAsset: _governanceProvincialBankIconAsset,
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
    this.iconAsset,
    this.collapsible = false,
    this.expanded = true,
    this.onToggleExpanded,
    this.onReorder,
    this.onReturnFromDetail,
  });

  final _GovernanceSectionKind sectionKind;
  final String title;
  final IconData icon;
  final String? iconAsset;
  final Color badgeColor;
  final List<Institution> institutions;
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
            _GovernanceIconBadge(
              icon: icon,
              iconAsset: iconAsset,
              color: badgeColor,
              boxSize: 28,
              iconSize: 16,
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
              // 中文注释：国储会虽横跨整行，但高度必须和省储会/省储行网格卡片保持一致。
              const crossAxisCount = 2;
              const crossAxisSpacing = 8.0;
              final childAspectRatio = constraints.maxWidth < 360 ? 2.6 : 2.9;
              final gridCardWidth =
                  (constraints.maxWidth - crossAxisSpacing) / crossAxisCount;
              final governanceCardHeight = gridCardWidth / childAspectRatio;
              if (sectionKind == _GovernanceSectionKind.nationalCouncil) {
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    for (final inst in institutions)
                      SizedBox(
                        key: ValueKey(
                          'governance_national_card_${inst.cidNumber}',
                        ),
                        height: governanceCardHeight,
                        child: _GovernanceCard(
                          institution: inst,
                          icon: icon,
                          badgeColor: badgeColor,
                          isAdmin: ProposalContextResolver.isInstitutionAdmin(
                            inst.cidNumber,
                          ),
                          onReturnFromDetail: onReturnFromDetail,
                        ),
                      ),
                  ],
                );
              }
              // 机构列表固定一行两列，避免不同 Android 机型出现列数漂移。
              return GridView.builder(
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemCount: institutions.length,
                gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                  crossAxisCount: crossAxisCount,
                  mainAxisSpacing: 8,
                  crossAxisSpacing: crossAxisSpacing,
                  childAspectRatio: childAspectRatio,
                ),
                itemBuilder: (context, index) {
                  final inst = institutions[index];
                  final isAdmin = ProposalContextResolver.isInstitutionAdmin(
                    inst.cidNumber,
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
  final Institution institution;
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

  final Institution institution;
  final IconData icon;
  final Color badgeColor;
  final bool isAdmin;
  final bool navigationEnabled;
  final bool pressAnimationEnabled;
  final VoidCallback? onReturnFromDetail;

  @override
  Widget build(BuildContext context) {
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
                        cidNumber: institution.cidNumber,
                        repository: InstitutionRepository(),
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
                Expanded(
                  // 中文注释：机构卡片不再显示名称左侧图标，只保留名称和右箭头。
                  child: Text(
                    institution.displayName,
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

class _GovernanceIconBadge extends StatelessWidget {
  const _GovernanceIconBadge({
    required this.icon,
    required this.color,
    required this.boxSize,
    required this.iconSize,
    this.iconAsset,
  });

  final IconData icon;
  final String? iconAsset;
  final Color color;
  final double boxSize;
  final double iconSize;

  @override
  Widget build(BuildContext context) {
    final asset = iconAsset;
    return Container(
      width: boxSize,
      height: boxSize,
      decoration: BoxDecoration(
        color: color.withAlpha(20),
        borderRadius: BorderRadius.circular(7),
      ),
      child: Center(
        // 中文注释：省储会/省储行使用指定 SVG 图案；国储会使用 Material 图标。
        child: asset == null
            ? Icon(icon, size: iconSize, color: color)
            : SvgPicture.asset(
                asset,
                width: iconSize,
                height: iconSize,
                colorFilter: ColorFilter.mode(color, BlendMode.srcIn),
              ),
      ),
    );
  }
}
