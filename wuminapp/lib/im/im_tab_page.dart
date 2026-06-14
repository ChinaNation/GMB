import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/im/im_session_models.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';

/// 公民“信息”Tab。
///
/// 本页先承载 IM 入口和边界状态；真实 P2P 投递、OpenMLS 加密、Isar 持久化
/// 会在后续任务接入到 `lib/im/` 子模块中。
class ImTabPage extends StatelessWidget {
  const ImTabPage({super.key});

  @override
  Widget build(BuildContext context) {
    const overview = ImInboxOverview.empty;

    return const SafeArea(
      child: ColoredBox(
        color: AppTheme.scaffoldBg,
        child: CustomScrollView(
          slivers: [
            SliverToBoxAdapter(child: _ImHeader()),
            SliverToBoxAdapter(child: _NodeStatusPanel(overview: overview)),
            SliverToBoxAdapter(child: _QuickActions()),
            SliverFillRemaining(
              hasScrollBody: false,
              child: _EmptyConversationList(),
            ),
          ],
        ),
      ),
    );
  }
}

class _ImHeader extends StatelessWidget {
  const _ImHeader();

  @override
  Widget build(BuildContext context) {
    return const Padding(
      padding: EdgeInsets.fromLTRB(20, 18, 20, 12),
      child: Row(
        children: [
          Expanded(
            child: Text(
              '信息',
              style: TextStyle(
                fontSize: 24,
                fontWeight: FontWeight.w700,
                color: AppTheme.textPrimary,
              ),
            ),
          ),
          Icon(Icons.search_rounded, color: AppTheme.textSecondary),
          SizedBox(width: 18),
          Icon(Icons.add_comment_outlined, color: AppTheme.textSecondary),
        ],
      ),
    );
  }
}

class _NodeStatusPanel extends StatelessWidget {
  const _NodeStatusPanel({required this.overview});

  final ImInboxOverview overview;

  @override
  Widget build(BuildContext context) {
    final statusText = switch (overview.nodeStatus) {
      ImNodeBindingStatus.unbound => '未绑定',
      ImNodeBindingStatus.offline => '离线',
      ImNodeBindingStatus.online => '在线',
      ImNodeBindingStatus.syncing => '同步中',
    };

    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
      child: Container(
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
          color: AppTheme.surfaceWhite,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: AppTheme.border),
        ),
        child: Row(
          children: [
            Container(
              width: 42,
              height: 42,
              decoration: BoxDecoration(
                color: AppTheme.primary.withAlpha(22),
                borderRadius: BorderRadius.circular(8),
              ),
              child: const Icon(
                Icons.dns_outlined,
                color: AppTheme.primary,
                size: 22,
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    '私人通信全节点',
                    style: TextStyle(
                      fontSize: 15,
                      fontWeight: FontWeight.w700,
                      color: AppTheme.textPrimary,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    statusText,
                    style: const TextStyle(
                      fontSize: 13,
                      color: AppTheme.textSecondary,
                    ),
                  ),
                ],
              ),
            ),
            Text(
              '${overview.pendingOutgoing}',
              style: const TextStyle(
                color: AppTheme.textSecondary,
                fontWeight: FontWeight.w600,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _QuickActions extends StatelessWidget {
  const _QuickActions();

  @override
  Widget build(BuildContext context) {
    return const Padding(
      padding: EdgeInsets.fromLTRB(16, 0, 16, 10),
      child: Row(
        children: [
          Expanded(
            child: _ActionButton(
              icon: Icons.qr_code_scanner_rounded,
              label: '扫码',
            ),
          ),
          SizedBox(width: 10),
          Expanded(
            child: _ActionButton(
              icon: Icons.sensors_rounded,
              label: '附近',
            ),
          ),
          SizedBox(width: 10),
          Expanded(
            child: _ActionButton(
              icon: Icons.payments_outlined,
              label: '公民币',
            ),
          ),
        ],
      ),
    );
  }
}

class _ActionButton extends StatelessWidget {
  const _ActionButton({
    required this.icon,
    required this.label,
  });

  final IconData icon;
  final String label;

  @override
  Widget build(BuildContext context) {
    return Container(
      height: 48,
      decoration: BoxDecoration(
        color: AppTheme.surfaceWhite,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: AppTheme.border),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(icon, size: 19, color: AppTheme.textSecondary),
          const SizedBox(width: 6),
          Text(
            label,
            style: const TextStyle(
              fontSize: 13,
              fontWeight: FontWeight.w600,
              color: AppTheme.textPrimary,
            ),
          ),
        ],
      ),
    );
  }
}

class _EmptyConversationList extends StatelessWidget {
  const _EmptyConversationList();

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: Padding(
        padding: EdgeInsets.fromLTRB(32, 32, 32, 80),
        child: Text(
          '暂无会话',
          style: TextStyle(
            color: AppTheme.textSecondary,
            fontSize: 15,
            fontWeight: FontWeight.w500,
          ),
        ),
      ),
    );
  }
}
