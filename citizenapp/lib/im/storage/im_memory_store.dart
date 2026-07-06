import '../im_session_models.dart';

/// IM 本地存储的临时内存实现。
///
/// 本类只用于信息 Tab 基础壳展示和单元测试。真实消息库必须进入 Isar schema，
/// 且落 schema 前需要按 GMB 规则再次确认。
class ImMemoryStore {
  ImMemoryStore({
    ImInboxOverview overview = ImInboxOverview.empty,
    List<ImConversationPreview> conversations = const [],
  })  : _overview = overview,
        _conversations = List.of(conversations);

  ImInboxOverview _overview;
  final List<ImConversationPreview> _conversations;

  /// 当前收件箱概览。
  ImInboxOverview get overview => _overview;

  /// 当前会话快照。
  List<ImConversationPreview> get conversations =>
      List.unmodifiable(_conversations);

  /// 更新互联网 mailbox / 近场聊天概览。
  void updateOverview(ImInboxOverview overview) {
    _overview = overview;
  }

  /// 写入会话预览。
  void upsertConversation(ImConversationPreview preview) {
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
