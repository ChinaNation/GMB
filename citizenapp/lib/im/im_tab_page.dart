import 'package:flutter/material.dart';

import '../my/user/user_service.dart';
import '../ui/app_theme.dart';
import '../wallet/core/wallet_manager.dart';
import 'im_chat_page.dart';
import 'im_runtime.dart';
import 'im_session_models.dart';
import 'storage/im_isar_store.dart';

typedef ImSendTextFactory = ImSendTextCallback? Function(
  String peerWalletAddress,
  String conversationId,
);
typedef ImSyncFactory = ImSyncCallback? Function(String peerWalletAddress);

/// 公民“信息”Tab。
///
/// 中文注释：信息页只展示会话列表。联系人添加、联系人详情和转账入口统一归属
/// “我的通讯录”；通信节点配对归属设置/用户资料流程，不能在信息页暴露工程按钮。
class ImTabPage extends StatefulWidget {
  ImTabPage({
    super.key,
    ImIsarStore? store,
    WalletManager? walletManager,
    UserProfileService? profileService,
    this.currentUserId,
    this.sendTextFactory,
    this.syncFactory,
    this.runtime,
  })  : store = store ?? ImIsarStore(),
        walletManager = walletManager ?? WalletManager(),
        profileService = profileService ?? UserProfileService();

  final ImIsarStore store;
  final WalletManager walletManager;
  final UserProfileService profileService;
  final String? currentUserId;
  final ImSendTextFactory? sendTextFactory;
  final ImSyncFactory? syncFactory;
  final ImRuntime? runtime;

  @override
  State<ImTabPage> createState() => _ImTabPageState();
}

class _ImTabPageState extends State<ImTabPage> {
  List<ImConversationPreview> _conversations = const [];
  String _currentUserId = '';
  bool _loading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _reload();
  }

  Future<void> _reload() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final activeWallet = widget.currentUserId ?? await _readCommunicationId();
      final conversations = await widget.store.readConversationPreviews(
        ownerChatAccount: activeWallet.isEmpty ? null : activeWallet,
      );
      if (!mounted) {
        return;
      }
      setState(() {
        _conversations = conversations;
        _currentUserId = activeWallet;
      });
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error.toString();
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          _loading = false;
        });
      }
    }
  }

  Future<String> _readCommunicationId() async {
    final runtimeAddress = await widget.runtime?.readCommunicationAddress();
    if (runtimeAddress != null && runtimeAddress.isNotEmpty) {
      return runtimeAddress;
    }
    final profile = await widget.profileService.getState();
    return profile.communicationAddress?.trim() ?? '';
  }

  void _openConversation(ImConversationPreview preview) {
    if (_currentUserId.isEmpty) {
      setState(() {
        _error = '请先在用户资料中设置通信账户';
      });
      return;
    }
    Navigator.of(context)
        .push(
          MaterialPageRoute<void>(
            builder: (context) => ImChatPage(
              conversationId: preview.conversationId,
              currentUserId: _currentUserId,
              peerUserId: preview.walletAddress,
              title: preview.title,
              store: widget.store,
              onSendText: widget.sendTextFactory?.call(
                    preview.walletAddress,
                    preview.conversationId,
                  ) ??
                  (widget.runtime == null
                      ? null
                      : (text) => widget.runtime!.sendText(
                            peerWalletAddress: preview.walletAddress,
                            conversationId: preview.conversationId,
                            text: text,
                          )),
              onSync: widget.syncFactory?.call(preview.walletAddress) ??
                  (widget.runtime == null
                      ? null
                      : () => widget.runtime!.syncPending()),
            ),
          ),
        )
        .then((_) => _reload());
  }

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: ColoredBox(
        color: AppTheme.scaffoldBg,
        child: RefreshIndicator(
          onRefresh: _reload,
          child: CustomScrollView(
            slivers: [
              const SliverToBoxAdapter(child: _ImHeader()),
              if (_error != null)
                SliverToBoxAdapter(child: _ErrorBanner(message: _error!)),
              if (_loading)
                const SliverFillRemaining(
                  hasScrollBody: false,
                  child: Center(child: CircularProgressIndicator()),
                )
              else if (_currentUserId.isEmpty)
                const SliverFillRemaining(
                  hasScrollBody: false,
                  child: _NoCommunicationAccount(),
                )
              else if (_conversations.isNotEmpty)
                SliverList.builder(
                  itemCount: _conversations.length,
                  itemBuilder: (context, index) {
                    final preview = _conversations[index];
                    return _ConversationTile(
                      preview: preview,
                      onTap: () => _openConversation(preview),
                    );
                  },
                )
              else
                const SliverFillRemaining(
                  hasScrollBody: false,
                  child: _EmptyConversationList(),
                ),
            ],
          ),
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
        ],
      ),
    );
  }
}

class _ConversationTile extends StatelessWidget {
  const _ConversationTile({
    required this.preview,
    required this.onTap,
  });

  final ImConversationPreview preview;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final subtitle = preview.lastMessage.trim().isEmpty
        ? preview.walletAddress
        : preview.lastMessage.trim();
    return _ListTileShell(
      title: preview.title,
      subtitle: subtitle,
      trailing: _statusText(preview.deliveryState),
      unreadCount: preview.unreadCount,
      onTap: onTap,
    );
  }
}

class _ListTileShell extends StatelessWidget {
  const _ListTileShell({
    required this.title,
    required this.subtitle,
    required this.trailing,
    required this.unreadCount,
    required this.onTap,
  });

  final String title;
  final String subtitle;
  final String trailing;
  final int unreadCount;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 10),
      child: Material(
        color: AppTheme.surfaceWhite,
        borderRadius: BorderRadius.circular(8),
        child: InkWell(
          borderRadius: BorderRadius.circular(8),
          onTap: onTap,
          child: Padding(
            padding: const EdgeInsets.all(14),
            child: Row(
              children: [
                CircleAvatar(
                  backgroundColor: AppTheme.primary.withAlpha(24),
                  child: const Icon(
                    Icons.person_outline,
                    color: AppTheme.primary,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        title,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          fontWeight: FontWeight.w700,
                          color: AppTheme.textPrimary,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        subtitle,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          fontSize: 13,
                          color: AppTheme.textSecondary,
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 10),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    Text(
                      trailing,
                      style: const TextStyle(
                        fontSize: 12,
                        color: AppTheme.textSecondary,
                      ),
                    ),
                    if (unreadCount > 0) ...[
                      const SizedBox(height: 6),
                      CircleAvatar(
                        radius: 10,
                        backgroundColor: AppTheme.primary,
                        child: Text(
                          '$unreadCount',
                          style: const TextStyle(
                            color: Colors.white,
                            fontSize: 11,
                          ),
                        ),
                      ),
                    ],
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _ErrorBanner extends StatelessWidget {
  const _ErrorBanner({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 10),
      child: Text(
        message,
        style: const TextStyle(color: Colors.red, fontSize: 12),
      ),
    );
  }
}

class _NoCommunicationAccount extends StatelessWidget {
  const _NoCommunicationAccount();

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: Padding(
        padding: EdgeInsets.fromLTRB(32, 32, 32, 80),
        child: Text(
          '请先在用户资料中设置通信账户',
          textAlign: TextAlign.center,
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

String _statusText(ImMessageDeliveryState state) {
  return switch (state) {
    ImMessageDeliveryState.queued => '排队',
    ImMessageDeliveryState.sending => '发送中',
    ImMessageDeliveryState.sent => '已发送',
    ImMessageDeliveryState.receivedByDevice => '已接收',
    ImMessageDeliveryState.failed => '失败',
  };
}
