import '../chat_models.dart';

/// Chat 本地存储的临时内存实现。
///
/// 本类只用于聊天 Tab 基础壳展示和单元测试。真实消息库必须进入 Isar schema，
/// 且落 schema 前需要按 GMB 规则再次确认。
class ChatMemoryStore {
  ChatMemoryStore({
    ChatInboxOverview overview = ChatInboxOverview.empty,
    List<ChatConversationPreview> conversations = const [],
  })  : _overview = overview,
        _conversations = List.of(conversations);

  ChatInboxOverview _overview;
  final List<ChatConversationPreview> _conversations;

  /// 当前收件箱概览。
  ChatInboxOverview get overview => _overview;

  /// 当前会话快照。
  List<ChatConversationPreview> get conversations =>
      List.unmodifiable(_conversations);

  /// 更新本机聊天概览。
  void updateOverview(ChatInboxOverview overview) {
    _overview = overview;
  }

  /// 写入会话预览。
  void upsertConversation(ChatConversationPreview preview) {
    final index = _conversations.indexWhere(
      (item) => item.conversationId == preview.conversationId,
    );
    if (index >= 0) {
      _conversations[index] = preview;
    } else {
      _conversations.add(preview);
    }
    _conversations.sort((a, b) => b.lastUpdatedAt.compareTo(a.lastUpdatedAt));
  }
}
