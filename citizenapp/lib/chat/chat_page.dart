import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_chat_core/flutter_chat_core.dart';
import 'package:flutter_chat_ui/flutter_chat_ui.dart';
import 'package:file_picker/file_picker.dart';

import '../ui/app_theme.dart';
import 'chat_ui_adapter.dart';
import 'chat_flow.dart';
import 'chat_models.dart';
import 'storage/chat_store.dart';

typedef ChatSendTextCallback = Future<void> Function(String text);
typedef ChatSendAttachmentCallback = Future<void> Function(
  ChatAttachmentDraft attachment,
);
typedef ChatSyncCallback = Future<int> Function();
typedef ChatStartRealtimeCallback = Future<Future<void> Function()?> Function({
  required Future<void> Function() onNotice,
  Future<void> Function()? onDisconnected,
});
typedef ChatDownloadAttachmentCallback = Future<ChatDownloadedAttachment>
    Function(
  String conversationId,
  String controlPlaintext,
);
typedef ChatPickAttachmentCallback = Future<ChatAttachmentDraft?> Function();
typedef ChatDeleteConversationCallback = Future<void> Function();

/// 公民 Chat 聊天详情页。
///
/// 页面只使用现成聊天 UI 展示和输入，消息真源仍是本地
/// [ChatStore]，发送和同步由上层注入的 P2P/MLS 状态机完成。
class ChatPage extends StatefulWidget {
  ChatPage({
    super.key,
    required this.conversationId,
    required this.ownerAccount,
    required this.peerUserId,
    required this.title,
    ChatStore? store,
    this.onSendText,
    this.onSendAttachment,
    this.onDownloadAttachment,
    this.pickAttachment,
    this.onSync,
    this.onStartRealtime,
    this.onDeleteConversation,
  }) : store = store ?? ChatStore();

  final String conversationId;
  final String ownerAccount;
  final String peerUserId;
  final String title;
  final ChatStore store;
  final ChatSendTextCallback? onSendText;
  final ChatSendAttachmentCallback? onSendAttachment;
  final ChatDownloadAttachmentCallback? onDownloadAttachment;
  final ChatPickAttachmentCallback? pickAttachment;
  final ChatSyncCallback? onSync;
  final ChatStartRealtimeCallback? onStartRealtime;
  final ChatDeleteConversationCallback? onDeleteConversation;

  @override
  State<ChatPage> createState() => _ChatPageState();
}

class _ChatPageState extends State<ChatPage> {
  // 聊天页停留时短轮询当前 mailbox；失败后退避，避免弱网下持续压请求。
  static const _normalPollInterval = Duration(seconds: 8);
  static const _backoffPollInterval = Duration(seconds: 30);
  // 实时已连时仍保留的低频心跳兜底：即使 WS 推送静默丢失，也能在此间隔内收到。
  static const _heartbeatPollInterval = Duration(seconds: 20);

  late final InMemoryChatController _chatController;
  late final _ChatLifecycleObserver _lifecycleObserver;
  bool _loading = true;
  bool _syncing = false;
  bool _attachmentBusy = false;
  bool _deleting = false;
  bool _polling = false;
  bool _realtimeConnecting = false;
  bool _appResumed = false;
  String? _error;
  Timer? _pollTimer;
  Future<void> Function()? _stopRealtime;
  Future<void>? _openCoordinatorInFlight;

  @override
  void initState() {
    super.initState();
    _chatController = InMemoryChatController();
    final lifecycleState = WidgetsBinding.instance.lifecycleState;
    _appResumed =
        lifecycleState == null || lifecycleState == AppLifecycleState.resumed;
    _lifecycleObserver = _ChatLifecycleObserver(
      onResume: () {
        _appResumed = true;
        _requestOpenCoordinate();
      },
      onPause: () {
        _appResumed = false;
        _pauseSync();
      },
    );
    WidgetsBinding.instance.addObserver(_lifecycleObserver);
    WidgetsBinding.instance
        .addPostFrameCallback((_) => _requestOpenCoordinate());
  }

  /// 首次打开和 resume 共享同一个同步 future，系统生命周期抖动不得重复建立
  /// WebSocket 或重复拉取 mailbox。
  void _requestOpenCoordinate() {
    if (!mounted || !_appResumed || _openCoordinatorInFlight != null) {
      return;
    }
    late final Future<void> created;
    created = _syncOnOpen().whenComplete(() {
      if (identical(_openCoordinatorInFlight, created)) {
        _openCoordinatorInFlight = null;
      }
    });
    _openCoordinatorInFlight = created;
  }

  @override
  void dispose() {
    _pauseSync();
    WidgetsBinding.instance.removeObserver(_lifecycleObserver);
    _chatController.dispose();
    super.dispose();
  }

  Future<void> _reloadMessages() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final messages = await widget.store.readMessages(widget.conversationId);
      await _chatController.setMessages(
        storedMessagesToChatMessages(
          messages,
          ownerAccount: widget.ownerAccount,
        ),
        animated: false,
      );
    } catch (error) {
      _error = error.toString();
    } finally {
      if (mounted) {
        setState(() {
          _loading = false;
        });
      }
    }
  }

  Future<void> _syncOnOpen() async {
    final sync = widget.onSync;
    if (sync == null) {
      await _reloadMessages();
      return;
    }
    await _syncAndReload(silent: true);
    if (mounted && widget.onSync != null) {
      final realtimeReady = await _startRealtime();
      if (!realtimeReady && mounted && widget.onSync != null) {
        _schedulePoll(_normalPollInterval);
      }
    }
  }

  Future<bool> _startRealtime() async {
    final starter = widget.onStartRealtime;
    if (!_appResumed || starter == null) {
      return false;
    }
    if (_stopRealtime != null || _realtimeConnecting) {
      return _stopRealtime != null;
    }
    _realtimeConnecting = true;
    try {
      final stop = await starter(
        onNotice: () => _syncAndReload(silent: true),
        onDisconnected: () async {
          _stopRealtime = null;
          if (_appResumed && mounted && widget.onSync != null) {
            _schedulePoll(_backoffPollInterval);
          }
        },
      );
      if (!mounted || !_appResumed) {
        await stop?.call();
        return false;
      }
      _stopRealtime = stop;
      if (stop != null) {
        // 实时已连也保留低频心跳兜底，防止推送静默丢失导致收不到新消息。
        _schedulePoll(_heartbeatPollInterval);
      }
      return stop != null;
    } catch (_) {
      return false;
    } finally {
      _realtimeConnecting = false;
    }
  }

  Future<bool> _syncAndReload({required bool silent}) async {
    final sync = widget.onSync;
    if (sync == null) {
      if (!silent && mounted) {
        setState(() {
          _error = '当前会话尚未绑定同步链路';
        });
      }
      return false;
    }
    try {
      await sync();
      await _reloadMessages();
      return true;
    } catch (error) {
      if (!silent && mounted) {
        setState(() {
          _error = error.toString();
        });
      }
      return false;
    }
  }

  void _schedulePoll(Duration delay) {
    if (!_appResumed) {
      return;
    }
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
    if (stop != null) {
      unawaited(stop());
    }
  }

  Future<void> _runPoll() async {
    if (!mounted || !_appResumed || widget.onSync == null) {
      return;
    }
    if (_polling) {
      _schedulePoll(_backoffPollInterval);
      return;
    }
    _polling = true;
    final ok = await _syncAndReload(silent: true);
    _polling = false;
    if (!mounted || !_appResumed || widget.onSync == null) {
      return;
    }
    // 实时在线：保留低频心跳兜底，按心跳间隔继续复查。
    if (_stopRealtime != null) {
      _schedulePoll(_heartbeatPollInterval);
      return;
    }
    // 实时离线：尝试重连；重连成功由 _startRealtime 起心跳，否则常规/退避轮询。
    if (ok && await _startRealtime()) {
      return;
    }
    _schedulePoll(ok ? _normalPollInterval : _backoffPollInterval);
  }

  Future<void> _handleSend(String text) async {
    final normalized = text.trim();
    if (normalized.isEmpty) {
      return;
    }
    final sender = widget.onSendText;
    if (sender == null) {
      setState(() {
        _error = '当前会话尚未绑定发送链路';
      });
      return;
    }
    try {
      await sender(normalized);
      await _reloadMessages();
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error.toString();
        });
      }
    }
  }

  Future<void> _handleAttachmentTap() async {
    final sender = widget.onSendAttachment;
    if (sender == null) {
      setState(() {
        _error = '当前会话尚未绑定附件发送链路';
      });
      return;
    }
    setState(() {
      _attachmentBusy = true;
      _error = null;
    });
    try {
      final draft = await (widget.pickAttachment?.call() ?? _pickAttachment());
      if (draft == null) {
        return;
      }
      await sender(draft);
      await _reloadMessages();
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error.toString();
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          _attachmentBusy = false;
        });
      }
    }
  }

  Future<void> _handleMessageTap(
    BuildContext context,
    Message message, {
    required int index,
    required TapUpDetails details,
  }) async {
    final metadata = message.metadata ?? const <String, dynamic>{};
    if (metadata['message_kind'] != ChatMessageKind.attachment.name) {
      return;
    }
    final controlPlaintext =
        metadata['attachment_control_plaintext']?.toString() ?? '';
    if (controlPlaintext.isEmpty) {
      setState(() {
        _error = '附件控制消息为空，无法下载';
      });
      return;
    }
    final downloader = widget.onDownloadAttachment;
    if (downloader == null) {
      setState(() {
        _error = '当前会话尚未绑定附件下载链路';
      });
      return;
    }
    setState(() {
      _attachmentBusy = true;
      _error = null;
    });
    try {
      final downloaded = await downloader(
        widget.conversationId,
        controlPlaintext,
      );
      if (mounted) {
        ScaffoldMessenger.of(this.context).showSnackBar(
          SnackBar(content: Text('附件已保存：${downloaded.fileName}')),
        );
      }
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error.toString();
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          _attachmentBusy = false;
        });
      }
    }
  }

  Future<void> _handleSync() async {
    final sync = widget.onSync;
    if (sync == null) {
      setState(() {
        _error = '当前会话尚未绑定同步链路';
      });
      return;
    }
    setState(() {
      _syncing = true;
      _error = null;
    });
    try {
      await _syncAndReload(silent: false);
    } finally {
      if (mounted) {
        setState(() {
          _syncing = false;
        });
      }
    }
  }

  Future<void> _handleDeleteConversation() async {
    final confirmed = await _confirmDeleteConversation(context);
    if (!confirmed || !mounted) {
      return;
    }
    setState(() {
      _deleting = true;
      _error = null;
    });
    try {
      _pauseSync();
      final deleter = widget.onDeleteConversation ??
          () => widget.store.deleteConversation(widget.conversationId);
      await deleter();
      if (!mounted) {
        return;
      }
      if (Navigator.of(context).canPop()) {
        Navigator.of(context).pop(true);
      } else {
        await _chatController.setMessages(const [], animated: false);
        setState(() {
          _deleting = false;
        });
      }
    } catch (error) {
      if (mounted) {
        setState(() {
          _deleting = false;
          _error = error.toString();
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        backgroundColor: AppTheme.surfaceCard,
        foregroundColor: AppTheme.textPrimary,
        elevation: 0,
        titleSpacing: 0,
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              widget.title,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
            ),
            Text(
              _shortAccount(widget.peerUserId),
              style: const TextStyle(
                fontSize: 12,
                color: AppTheme.textSecondary,
              ),
            ),
          ],
        ),
        actions: [
          IconButton(
            tooltip: '同步',
            onPressed: _syncing || _deleting ? null : _handleSync,
            icon: _syncing
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.sync_rounded),
          ),
          PopupMenuButton<_ChatMenuAction>(
            tooltip: '更多',
            icon: const Icon(Icons.more_vert_rounded),
            enabled: !_deleting,
            onSelected: (action) {
              switch (action) {
                case _ChatMenuAction.deleteConversation:
                  unawaited(_handleDeleteConversation());
              }
            },
            itemBuilder: (context) => const [
              PopupMenuItem(
                value: _ChatMenuAction.deleteConversation,
                child: Row(
                  children: [
                    Icon(Icons.delete_outline_rounded, size: 18),
                    SizedBox(width: 10),
                    Text('删除聊天记录'),
                  ],
                ),
              ),
            ],
          ),
        ],
      ),
      body: Column(
        children: [
          if (_attachmentBusy || _deleting)
            const LinearProgressIndicator(minHeight: 2),
          if (_error != null)
            Container(
              width: double.infinity,
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
              color: Colors.red.withAlpha(20),
              child: Text(
                _error!,
                style: const TextStyle(color: Colors.red, fontSize: 12),
              ),
            ),
          Expanded(
            child: _loading
                ? const Center(child: CircularProgressIndicator())
                : Chat(
                    currentUserId: widget.ownerAccount,
                    chatController: _chatController,
                    onMessageSend: _handleSend,
                    onAttachmentTap: _handleAttachmentTap,
                    onMessageTap: _handleMessageTap,
                    backgroundColor: AppTheme.scaffoldBg,
                    resolveUser: (id) async {
                      final isMe = id == widget.ownerAccount;
                      return User(
                        id: id,
                        name: isMe ? '我' : widget.title,
                      );
                    },
                  ),
          ),
        ],
      ),
    );
  }
}

Future<ChatAttachmentDraft?> _pickAttachment() async {
  final result = await FilePicker.platform.pickFiles(
    allowMultiple: false,
    withData: true,
  );
  if (result == null || result.files.isEmpty) {
    return null;
  }
  final file = result.files.single;
  final bytes = file.bytes ??
      (file.path == null ? null : await File(file.path!).readAsBytes());
  if (bytes == null) {
    throw StateError('无法读取所选附件');
  }
  return ChatAttachmentDraft(
    fileName: file.name,
    contentType: _guessContentType(file.name),
    bytes: bytes,
  );
}

String _guessContentType(String fileName) {
  final lower = fileName.toLowerCase();
  if (lower.endsWith('.jpg') || lower.endsWith('.jpeg')) {
    return 'image/jpeg';
  }
  if (lower.endsWith('.png')) {
    return 'image/png';
  }
  if (lower.endsWith('.webp')) {
    return 'image/webp';
  }
  if (lower.endsWith('.gif')) {
    return 'image/gif';
  }
  if (lower.endsWith('.mp4')) {
    return 'video/mp4';
  }
  if (lower.endsWith('.mov')) {
    return 'video/quicktime';
  }
  if (lower.endsWith('.pdf')) {
    return 'application/pdf';
  }
  if (lower.endsWith('.txt')) {
    return 'text/plain';
  }
  return 'application/octet-stream';
}

enum _ChatMenuAction { deleteConversation }

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

class _ChatLifecycleObserver extends WidgetsBindingObserver {
  _ChatLifecycleObserver({
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

String _shortAccount(String value) {
  if (value.length <= 16) {
    return value;
  }
  return '${value.substring(0, 8)}...${value.substring(value.length - 6)}';
}
