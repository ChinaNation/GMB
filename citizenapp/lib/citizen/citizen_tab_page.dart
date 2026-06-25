import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/election/election_tab.dart';
import 'package:citizenapp/citizen/governance/governance_tab.dart';
import 'package:citizenapp/citizen/legislation/legislation_tab.dart';
import 'package:citizenapp/citizen/public/public_page.dart';
import 'package:citizenapp/citizen/vote/vote_view.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 底部“公民”Tab 的总入口。
///
/// 仅负责公民域二级导航分发；具体业务分别下沉到 public/vote/governance/institution/proposal。
class CitizenTabPage extends StatefulWidget {
  const CitizenTabPage({super.key, this.onPendingVoteCountChanged});

  final ValueChanged<int>? onPendingVoteCountChanged;

  @override
  State<CitizenTabPage> createState() => _CitizenTabPageState();
}

class _CitizenTabPageState extends State<CitizenTabPage> {
  int _selectedTab = 0;
  static const List<String> _tabs = ['广场', '立法', '选举', '治理', '公权'];

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
    switch (_selectedTab) {
      case 0: // 广场:订阅/本地区/我是管理员 机构动态(P7 改造;现为全局提案流)
        return VoteView(
          onPendingVoteCountChanged: widget.onPendingVoteCountChanged,
        );
      case 1: // 立法(P3 接法律浏览)
        return const LegislationTab();
      case 2: // 选举(P8 接选举活动视图)
        return const ElectionTab();
      case 3: // 治理:国储会/省储会/省储行(统一目录按机构码过滤)
        return const GovernanceTab();
      case 4: // 公权:全部机构地理浏览
        return const PublicPage();
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
