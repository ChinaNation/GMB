import 'dart:async';

import 'package:flutter/material.dart';

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
typedef ImSendAttachmentFactory = ImSendAttachmentCallback? Function(
  String peerWalletAddress,
  String conversationId,
);
typedef ImDownloadAttachmentFactory = ImDownloadAttachmentCallback? Function(
  String peerWalletAddress,
);

/// 公民“信息”Tab。
///
/// 信息页只展示会话列表。联系人添加、联系人详情和转账入口统一归属
/// “我的通讯录”；互联网 mailbox 和近场传输由 IM 运行态自动处理。
class ImTabPage extends StatefulWidget {
  ImTabPage({
    super.key,
    ImIsarStore? store,
    WalletManager? walletManager,
    this.currentUserId,
    this.sendTextFactory,
    this.sendAttachmentFactory,
    this.downloadAttachmentFactory,
    this.syncFactory,
    this.runtime,
  })  : store = store ?? ImIsarStore(),
        walletManager = walletManager ?? WalletManager();

  final ImIsarStore store;
  final WalletManager walletManager;
  final String? currentUserId;
  final ImSendTextFactory? sendTextFactory;
  final ImSendAttachmentFactory? sendAttachmentFactory;
  final ImDownloadAttachmentFactory? downloadAttachmentFactory;
  final ImSyncFactory? syncFactory;
  final ImRuntime? runtime;

  @override
  State<ImTabPage> createState() => _ImTabPageState();
}

class _ImTabPageState extends State<ImTabPage> {
  // 信息页只做前台轻量轮询；离开页面或 App 退后台即停止，不做后台常驻扫描。
  static const _normalPollInterval = Duration(seconds: 15);
  static const _backoffPollInterval = Duration(seconds: 30);

  List<ImConversationPreview> _conversations = const [];
  String _currentUserId = '';
  bool _loading = true;
  bool _polling = false;
  bool _realtimeConnecting = false;
  String? _error;
  Timer? _pollTimer;
  String? _realtimeWallet;
  Future<void> Function()? _stopRealtime;
  late final _LifecycleObserver _lifecycleObserver;

  @override
  void initState() {
    super.initState();
    _lifecycleObserver = _LifecycleObserver(
      onResume: () => _reload(syncFirst: true),
      onPause: _pauseSync,
    );
    WidgetsBinding.instance.addObserver(_lifecycleObserver);
    _reload(syncFirst: true);
  }

  @override
  void dispose() {
    _pauseSync();
    WidgetsBinding.instance.removeObserver(_lifecycleObserver);
    super.dispose();
  }

  Future<void> _reload({bool syncFirst = false}) async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final activeWallet = widget.currentUserId ?? await _readCommunicationId();
      if (syncFirst && activeWallet.isNotEmpty) {
        await _syncPendingSilently();
      }
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
      _configurePolling(activeWallet);
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

  Future<bool> _syncPendingSilently() async {
    final runtime = widget.runtime;
    if (runtime == null) {
      return true;
    }
    try {
      await runtime.syncPending();
      return true;
    } catch (_) {
      return false;
    }
  }

  void _configurePolling(String activeWallet) {
    if (activeWallet.isEmpty || widget.runtime == null) {
      _pauseSync();
      return;
    }
    if (_realtimeWallet != null && _realtimeWallet != activeWallet) {
      _pauseSync();
    }
    if (_stopRealtime != null) {
      return;
    }
    _schedulePoll(_normalPollInterval);
    unawaited(_startRealtime(activeWallet));
  }

  Future<bool> _startRealtime(String activeWallet) async {
    final runtime = widget.runtime;
    if (runtime == null || activeWallet.isEmpty) {
      return false;
    }
    if (_stopRealtime != null || _realtimeConnecting) {
      return _stopRealtime != null;
    }
    _realtimeConnecting = true;
    try {
      final stop = await runtime.startRealtimeSync(
        onNotice: () => _syncAndRefresh(activeWallet),
        onDisconnected: () async {
          _stopRealtime = null;
          _realtimeWallet = null;
          if (mounted && widget.runtime != null && _currentUserId.isNotEmpty) {
            _schedulePoll(_backoffPollInterval);
          }
        },
      );
      if (!mounted || _currentUserId != activeWallet) {
        await stop?.call();
        return false;
      }
      _stopRealtime = stop;
      _realtimeWallet = activeWallet;
      if (stop != null) {
        _stopPolling();
      }
      return stop != null;
    } catch (_) {
      return false;
    } finally {
      _realtimeConnecting = false;
    }
  }

  Future<void> _syncAndRefresh(String ownerChatAccount) async {
    await _syncPendingSilently();
    final conversations = await widget.store.readConversationPreviews(
      ownerChatAccount: ownerChatAccount,
    );
    if (mounted && _currentUserId == ownerChatAccount) {
      setState(() {
        _conversations = conversations;
      });
    }
  }

  void _schedulePoll(Duration delay) {
    _pollTimer?.cancel();
    _pollTimer = Timer(delay, _runPoll);
  }

  void _stopPolling() {
    _pollTimer?.cancel();
    _pollTimer = null;
  }

  void _pauseSync() {
    _stopPolling();
    final stop = _stopRealtime;
    _stopRealtime = null;
    _realtimeWallet = null;
    if (stop != null) {
      unawaited(stop());
    }
  }

  Future<void> _runPoll() async {
    if (!mounted || widget.runtime == null || _currentUserId.isEmpty) {
      return;
    }
    if (_stopRealtime != null) {
      return;
    }
    if (_polling) {
      _schedulePoll(_backoffPollInterval);
      return;
    }
    _polling = true;
    var ok = true;
    try {
      ok = await _syncPendingSilently();
      final conversations = await widget.store.readConversationPreviews(
        ownerChatAccount: _currentUserId,
      );
      if (mounted) {
        setState(() {
          _conversations = conversations;
        });
      }
    } catch (_) {
      ok = false;
    }
    _polling = false;
    if (mounted && widget.runtime != null && _currentUserId.isNotEmpty) {
      if (ok && await _startRealtime(_currentUserId)) {
        return;
      }
      _schedulePoll(ok ? _normalPollInterval : _backoffPollInterval);
    }
  }

  Future<String> _readCommunicationId() async {
    final runtimeAddress = await widget.runtime?.readCommunicationAddress();
    if (runtimeAddress != null && runtimeAddress.isNotEmpty) {
      return runtimeAddress;
    }
    // 身份统一取默认用户钱包（列表中最靠前的热钱包）。
    final wallet = await widget.walletManager.getDefaultWallet();
    return wallet?.address ?? '';
  }

  Future<void> _deleteLocalConversation(String conversationId) {
    final runtime = widget.runtime;
    if (runtime != null) {
      return runtime.deleteLocalConversation(conversationId);
    }
    return widget.store.deleteConversation(conversationId);
  }

  Future<void> _confirmAndDeleteConversation(
    ImConversationPreview preview,
  ) async {
    final confirmed = await _confirmDeleteConversation(context);
    if (!confirmed || !mounted) {
      return;
    }
    try {
      await _deleteLocalConversation(preview.conversationId);
      if (!mounted) {
        return;
      }
      setState(() {
        _conversations = _conversations
            .where(
              (item) => item.conversationId != preview.conversationId,
            )
            .toList(growable: false);
        _error = null;
      });
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error.toString();
        });
      }
    }
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
              onSendAttachment: widget.sendAttachmentFactory?.call(
                    preview.walletAddress,
                    preview.conversationId,
                  ) ??
                  (widget.runtime == null
                      ? null
                      : (attachment) => widget.runtime!.sendAttachment(
                            peerWalletAddress: preview.walletAddress,
                            conversationId: preview.conversationId,
                            attachment: attachment,
                          )),
              onDownloadAttachment: widget.downloadAttachmentFactory?.call(
                    preview.walletAddress,
                  ) ??
                  (widget.runtime == null
                      ? null
                      : (
                          String conversationId,
                          String controlPlaintext,
                        ) =>
                          widget.runtime!.downloadAttachment(
                            conversationId: conversationId,
                            controlPlaintext: controlPlaintext,
                          )),
              onSync: widget.syncFactory?.call(preview.walletAddress) ??
                  (widget.runtime == null
                      ? null
                      : () => widget.runtime!.syncPending()),
              onStartRealtime: widget.runtime?.startRealtimeSync,
              onDeleteConversation: () => _deleteLocalConversation(
                preview.conversationId,
              ),
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
          onRefresh: () => _reload(syncFirst: true),
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
                      onDelete: () => _confirmAndDeleteConversation(preview),
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

class _LifecycleObserver extends WidgetsBindingObserver {
  _LifecycleObserver({
    required this.onResume,
    required this.onPause,
  });

  final VoidCallback onResume;
  final VoidCallback onPause;

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      onResume();
    } else {
      onPause();
    }
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
    required this.onDelete,
  });

  final ImConversationPreview preview;
  final VoidCallback onTap;
  final Future<void> Function() onDelete;

  @override
  Widget build(BuildContext context) {
    final subtitle = preview.lastMessage.trim().isEmpty
        ? preview.walletAddress
        : preview.lastMessage.trim();
    return Dismissible(
      key: ValueKey('im-conversation-${preview.conversationId}'),
      direction: DismissDirection.endToStart,
      background: const _DeleteDismissBackground(),
      confirmDismiss: (_) async {
        await onDelete();
        return false;
      },
      child: _ListTileShell(
        title: preview.title,
        subtitle: subtitle,
        trailing: _statusText(preview.deliveryState),
        unreadCount: preview.unreadCount,
        onTap: onTap,
      ),
    );
  }
}

class _DeleteDismissBackground extends StatelessWidget {
  const _DeleteDismissBackground();

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 10),
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: Colors.red.shade600,
          borderRadius: BorderRadius.circular(8),
        ),
        child: const Align(
          alignment: Alignment.centerRight,
          child: Padding(
            padding: EdgeInsets.only(right: 20),
            child: Icon(Icons.delete_outline_rounded, color: Colors.white),
          ),
        ),
      ),
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

Future<bool> _confirmDeleteConversation(BuildContext context) async {
  final confirmed = await showDialog<bool>(
    context: context,
    builder: (context) => AlertDialog(
      title: const Text('删除聊天记录'),
      content: const Text('确定删除这台设备上的聊天记录？'),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(false),
          child: const Text('取消'),
        ),
        TextButton(
          onPressed: () => Navigator.of(context).pop(true),
          child: const Text('删除'),
        ),
      ],
    ),
  );
  return confirmed ?? false;
}
