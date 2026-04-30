import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/citizen/governance/all_proposals_view.dart';
import 'package:wuminapp_mobile/citizen/institution/institution_data.dart';
import 'package:wuminapp_mobile/citizen/institution/institution_list_page.dart';
import 'package:wuminapp_mobile/citizen/vote/vote_page.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';

/// 底部“公民”Tab 的总入口。
///
/// 这里仅负责公民域二级导航分发，具体业务分别下沉到 vote/governance/institution/proposal。
class CitizenTabPage extends StatefulWidget {
  const CitizenTabPage({super.key, this.onPendingVoteCountChanged});

  final ValueChanged<int>? onPendingVoteCountChanged;

  @override
  State<CitizenTabPage> createState() => _CitizenTabPageState();
}

class _CitizenTabPageState extends State<CitizenTabPage> {
  int _selectedTab = 1;
  static const List<String> _tabs = ['投票', '治理', '机构'];

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        children: [
          const SizedBox(height: 10),
          _StyledTabs(
            tabs: _tabs,
            selectedIndex: _selectedTab,
            onSelected: (index) {
              setState(() {
                _selectedTab = index;
              });
            },
          ),
          Expanded(child: _buildTabContent()),
        ],
      ),
    );
  }

  Widget _buildTabContent() {
    assert(kProvincialCouncils.length == 43);
    assert(kProvincialBanks.length == 43);

    switch (_selectedTab) {
      case 0:
        return const VotePage();
      case 1:
        return AllProposalsView(
          onPendingVoteCountChanged: widget.onPendingVoteCountChanged,
        );
      case 2:
        return const InstitutionListPage(
          nationalCouncil: kNationalCouncil,
          provincialCouncils: kProvincialCouncils,
          provincialBanks: kProvincialBanks,
        );
      default:
        return const SizedBox.shrink();
    }
  }
}

/// 公民域二级 tab 切换组件。
class _StyledTabs extends StatelessWidget {
  const _StyledTabs({
    required this.tabs,
    required this.selectedIndex,
    required this.onSelected,
  });

  final List<String> tabs;
  final int selectedIndex;
  final ValueChanged<int> onSelected;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 48, vertical: 4),
      padding: const EdgeInsets.all(4),
      decoration: BoxDecoration(
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(AppTheme.radiusMd),
        border: Border.all(color: AppTheme.border),
      ),
      child: Row(
        children: [
          for (int i = 0; i < tabs.length; i++)
            Expanded(
              child: GestureDetector(
                onTap: () => onSelected(i),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 200),
                  curve: Curves.easeInOut,
                  padding: const EdgeInsets.symmetric(vertical: 8),
                  decoration: BoxDecoration(
                    color: i == selectedIndex
                        ? AppTheme.surfaceWhite
                        : Colors.transparent,
                    borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                    boxShadow: i == selectedIndex
                        ? [
                            BoxShadow(
                              color: AppTheme.primary.withAlpha(15),
                              blurRadius: 4,
                              offset: const Offset(0, 1),
                            ),
                          ]
                        : null,
                  ),
                  child: Text(
                    tabs[i],
                    textAlign: TextAlign.center,
                    style: TextStyle(
                      fontSize: 15,
                      fontWeight: i == selectedIndex
                          ? FontWeight.w700
                          : FontWeight.w500,
                      color: i == selectedIndex
                          ? AppTheme.primary
                          : AppTheme.textSecondary,
                    ),
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}
