import 'dart:async';

import 'package:flutter/material.dart';

import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/chat_payload.dart';
import 'package:citizenapp/chat/group/ui/open_group_chat.dart';
import 'package:citizenapp/chat/open_direct_chat.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';
import 'package:citizenapp/my/user/contact_service.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 群聊打开器；测试可注入替身，正式运行走 [openGroupChat]。
typedef GroupChatOpener = Future<void> Function(
  BuildContext context, {
  required String groupId,
  required String title,
});

/// 聊天搜索页：一个输入框，三段结果 —— 会话 / 联系人 / 聊天记录。
///
/// - 会话与联系人在内存里过滤（进页时一次性载入，数据量小）。
/// - 聊天记录走 [ChatStore.searchMessages] 跨会话检索本机已解密消息。
/// - 点任一结果都复用既有打开收口：群聊 [openGroupChat]、单聊 [openDirectChat]，
///   不在本页复刻 ChatPage 装配。
/// - 聊天记录命中当前**只打开所在会话**，不定位到具体消息（消息级锚点需
///   ChatPage 支持滚动定位，单列后续任务）。
class ChatSearchPage extends StatefulWidget {
  const ChatSearchPage({
    super.key,
    this.store,
    this.contactService,
    this.accountId,
    this.directChatOpener,
    this.groupChatOpener,
  });

  final ChatStore? store;
  final UserContactService? contactService;

  /// 当前默认热钱包账户；不传则页面自行读取。
  final String? accountId;
  final DirectChatOpener? directChatOpener;
  final GroupChatOpener? groupChatOpener;

  @override
  State<ChatSearchPage> createState() => _ChatSearchPageState();
}

class _ChatSearchPageState extends State<ChatSearchPage> {
  late final ChatStore _store = widget.store ?? ChatStore();
  late final UserContactService _contactService =
      widget.contactService ?? UserContactService();
  final TextEditingController _controller = TextEditingController();

  String _accountId = '';
  List<ChatConversationPreview> _conversations =
      const <ChatConversationPreview>[];
  List<UserContact> _contacts = const <UserContact>[];
  List<ChatStoredMessage> _messageHits = const <ChatStoredMessage>[];
  String _query = '';
  bool _loading = true;

  /// 消息检索是异步的：用递增序号丢弃过期结果，
  /// 避免快速输入时旧关键词的结果覆盖新关键词的结果。
  int _searchSeq = 0;

  @override
  void initState() {
    super.initState();
    unawaited(_load());
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    final accountId = widget.accountId ??
        (await WalletManager().getDefaultWallet())?.accountId ??
        '';
    final conversations =
        await _store.readConversationPreviews(accountId: accountId);
    List<UserContact> contacts;
    try {
      contacts = await _contactService.getContacts();
    } on Exception {
      // 通讯录读失败只让「联系人」段为空，不阻塞会话与聊天记录搜索。
      contacts = const <UserContact>[];
    }
    if (!mounted) return;
    setState(() {
      _accountId = accountId;
      _conversations = conversations;
      _contacts = contacts;
      _loading = false;
    });
  }

  Future<void> _onQueryChanged(String value) async {
    final query = value.trim();
    setState(() => _query = query);
    if (query.isEmpty) {
      setState(() => _messageHits = const <ChatStoredMessage>[]);
      return;
    }
    final seq = ++_searchSeq;
    final hits = await _store.searchMessages(
      accountId: _accountId,
      keyword: query,
    );
    if (!mounted || seq != _searchSeq) return;
    setState(() => _messageHits = hits);
  }

  List<ChatConversationPreview> get _conversationHits {
    if (_query.isEmpty) return const <ChatConversationPreview>[];
    final needle = _query.toLowerCase();
    return _conversations
        .where((item) =>
            item.title.toLowerCase().contains(needle) ||
            item.lastMessage.toLowerCase().contains(needle))
        .toList(growable: false);
  }

  List<UserContact> get _contactHits {
    if (_query.isEmpty) return const <UserContact>[];
    final needle = _query.toLowerCase();
    // 只匹配本人备注名与账户：公开昵称要联网拉取，搜索页不引入网络依赖。
    return _contacts
        .where((item) =>
            item.contactName.toLowerCase().contains(needle) ||
            item.accountId.toLowerCase().contains(needle))
        .toList(growable: false);
  }

  Future<void> _openConversation(ChatConversationPreview preview) async {
    if (preview.isGroup) {
      final opener = widget.groupChatOpener ?? openGroupChat;
      await opener(
        context,
        groupId: preview.conversationId,
        title: preview.title,
      );
      return;
    }
    final opener = widget.directChatOpener ?? openDirectChat;
    await opener(
      context,
      peerAccountId: preview.peerAccountId,
      title: preview.title,
    );
  }

  Future<void> _openContact(UserContact contact) async {
    final opener = widget.directChatOpener ?? openDirectChat;
    final title = contact.contactName.trim().isEmpty
        ? ProfilePresentation.forAccount(contact.accountId).fallbackName
        : contact.contactName;
    await opener(context, peerAccountId: contact.accountId, title: title);
  }

  /// 聊天记录命中：只打开消息所在会话，不定位到具体消息。
  Future<void> _openMessageHit(ChatStoredMessage message) async {
    ChatConversationPreview? preview;
    for (final item in _conversations) {
      if (item.conversationId == message.conversationId) {
        preview = item;
        break;
      }
    }
    if (preview == null) return;
    await _openConversation(preview);
  }

  @override
  Widget build(BuildContext context) {
    final conversationHits = _conversationHits;
    final contactHits = _contactHits;
    final hasQuery = _query.isNotEmpty;
    final noHit = hasQuery &&
        conversationHits.isEmpty &&
        contactHits.isEmpty &&
        _messageHits.isEmpty;
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        titleSpacing: 0,
        title: TextField(
          key: const ValueKey('chat-search-input'),
          controller: _controller,
          autofocus: true,
          onChanged: (value) => unawaited(_onQueryChanged(value)),
          decoration: const InputDecoration(
            hintText: '搜索会话、联系人、聊天记录',
            border: InputBorder.none,
          ),
        ),
        actions: [
          if (hasQuery)
            IconButton(
              tooltip: '清空',
              onPressed: () {
                _controller.clear();
                unawaited(_onQueryChanged(''));
              },
              icon: const Icon(Icons.close_rounded),
            ),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : !hasQuery
              ? const _SearchHint()
              : noHit
                  ? const Center(child: Text('没有找到相关内容'))
                  : ListView(
                      padding: const EdgeInsets.only(bottom: 24),
                      children: [
                        if (conversationHits.isNotEmpty) ...[
                          const _SectionHeader(title: '会话'),
                          for (final item in conversationHits)
                            ListTile(
                              key: ValueKey(
                                'search-conversation-${item.conversationId}',
                              ),
                              leading: Icon(
                                item.isGroup
                                    ? Icons.groups_rounded
                                    : Icons.person_rounded,
                                color: AppTheme.textSecondary,
                              ),
                              title: Text(item.title),
                              subtitle: Text(
                                item.lastMessage,
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                              ),
                              onTap: () => unawaited(_openConversation(item)),
                            ),
                        ],
                        if (contactHits.isNotEmpty) ...[
                          const _SectionHeader(title: '联系人'),
                          for (final item in contactHits)
                            ListTile(
                              key: ValueKey(
                                'search-contact-${item.accountId}',
                              ),
                              leading: const Icon(
                                Icons.account_circle_rounded,
                                color: AppTheme.textSecondary,
                              ),
                              title: Text(item.contactName),
                              subtitle: Text(
                                item.ss58Address,
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                              ),
                              onTap: () => unawaited(_openContact(item)),
                            ),
                        ],
                        if (_messageHits.isNotEmpty) ...[
                          const _SectionHeader(title: '聊天记录'),
                          for (final item in _messageHits)
                            ListTile(
                              key: ValueKey(
                                'search-message-${item.envelopeId}',
                              ),
                              leading: const Icon(
                                Icons.chat_bubble_outline_rounded,
                                color: AppTheme.textSecondary,
                              ),
                              // 载荷需解码成摘要：媒体/贴纸显示类型化占位。
                              title: Text(
                                ChatPayloadCodec.decode(item.plaintext ?? '')
                                    .summary,
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                              ),
                              onTap: () => unawaited(_openMessageHit(item)),
                            ),
                        ],
                      ],
                    ),
    );
  }
}

class _SectionHeader extends StatelessWidget {
  const _SectionHeader({required this.title});

  final String title;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 6),
      child: Text(
        title,
        style: const TextStyle(
          color: AppTheme.textTertiary,
          fontSize: 12,
          fontWeight: FontWeight.w700,
        ),
      ),
    );
  }
}

class _SearchHint extends StatelessWidget {
  const _SearchHint();

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: Padding(
        padding: EdgeInsets.symmetric(horizontal: 32),
        child: Text(
          '输入关键词，搜索会话、联系人与聊天记录',
          textAlign: TextAlign.center,
          style: TextStyle(color: AppTheme.textTertiary),
        ),
      ),
    );
  }
}
