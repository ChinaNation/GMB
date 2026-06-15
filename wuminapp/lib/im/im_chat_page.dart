import 'package:flutter/material.dart';
import 'package:flutter_chat_core/flutter_chat_core.dart';
import 'package:flutter_chat_ui/flutter_chat_ui.dart';

import '../ui/app_theme.dart';
import 'im_chat_ui_adapter.dart';
import 'storage/im_isar_store.dart';

typedef ImSendTextCallback = Future<void> Function(String text);
typedef ImSyncCallback = Future<int> Function();

/// 公民 IM 聊天详情页。
///
/// 中文注释：页面只使用现成聊天 UI 展示和输入，消息真源仍是本地
/// [ImIsarStore]，发送和同步由上层注入的 P2P/MLS 状态机完成。
class ImChatPage extends StatefulWidget {
  ImChatPage({
    super.key,
    required this.conversationId,
    required this.currentUserId,
    required this.peerUserId,
    required this.title,
    ImIsarStore? store,
    this.onSendText,
    this.onSync,
  }) : store = store ?? ImIsarStore();

  final String conversationId;
  final String currentUserId;
  final String peerUserId;
  final String title;
  final ImIsarStore store;
  final ImSendTextCallback? onSendText;
  final ImSyncCallback? onSync;

  @override
  State<ImChatPage> createState() => _ImChatPageState();
}

class _ImChatPageState extends State<ImChatPage> {
  late final InMemoryChatController _chatController;
  bool _loading = true;
  bool _syncing = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _chatController = InMemoryChatController();
    _reloadMessages();
  }

  @override
  void dispose() {
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
        imStoredMessagesToChatMessages(
          messages,
          currentUserId: widget.currentUserId,
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
      await sync();
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
          _syncing = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        backgroundColor: AppTheme.surfaceWhite,
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
            onPressed: _syncing ? null : _handleSync,
            icon: _syncing
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.sync_rounded),
          ),
        ],
      ),
      body: Column(
        children: [
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
                    currentUserId: widget.currentUserId,
                    chatController: _chatController,
                    onMessageSend: _handleSend,
                    backgroundColor: AppTheme.scaffoldBg,
                    resolveUser: (id) async {
                      final isMe = id == widget.currentUserId;
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

String _shortAccount(String value) {
  if (value.length <= 16) {
    return value;
  }
  return '${value.substring(0, 8)}...${value.substring(value.length - 6)}';
}
