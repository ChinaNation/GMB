import 'package:flutter/material.dart';

import 'package:citizenapp/chat/chat_page.dart';
import 'package:citizenapp/chat/chat_runtime.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';

typedef DirectChatOpener = Future<void> Function(
  BuildContext context, {
  required String peerCidNumber,
  required String title,
});

/// 打开与目标 CID 的一对一聊天。
///
/// 发起方使用身份账户（CID 绑定账户）的 AccountId（空则引导创建热钱包）。广场用户主页「消息」与联系人详情
/// 「消息」共用此入口，复用现有 Chat 运行态，避免重复拼装。
Future<void> openDirectChat(
  BuildContext context, {
  required String peerCidNumber,
  required String title,
}) async {
  final identity = await IdentityAccountCache.instance.resolve();
  final sender = identity?.accountId ?? '';
  final ownerCidNumber = identity?.snapshot?.cidNumber ?? '';
  if (sender.isEmpty || ownerCidNumber.isEmpty) {
    if (!context.mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('请先在「我的 → 我的钱包」创建热钱包')),
    );
    return;
  }
  // 不能和自己发起聊天：所有私信入口的最后一道防线（广场主页/通讯录都走此收口）。
  if (peerCidNumber.trim() == ownerCidNumber) {
    if (!context.mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('不能和自己发起聊天')),
    );
    return;
  }
  final runtime = ChatRuntime();
  final conversationId =
      ChatRuntime.directConversationId(ownerCidNumber, peerCidNumber);
  if (!context.mounted) return;
  await Navigator.of(context).push<void>(
    MaterialPageRoute(
      builder: (_) => ChatPage(
        conversationId: conversationId,
        ownerCidNumber: ownerCidNumber,
        accountId: sender,
        peerUserId: peerCidNumber,
        title: title,
        onSendText: (text) => runtime.sendText(
          peerCidNumber: peerCidNumber,
          conversationId: conversationId,
          text: text,
        ),
        onSendMedia: (media) => runtime.sendMedia(
          peerCidNumber: peerCidNumber,
          conversationId: conversationId,
          media: media,
        ),
        onSendSticker: (packId, stickerId) => runtime.sendSticker(
          peerCidNumber: peerCidNumber,
          conversationId: conversationId,
          packId: packId,
          stickerId: stickerId,
        ),
        onResolveMediaPath: (
          conversationId,
          attachmentId,
          fileName,
          contentType,
          clearByteSize,
        ) =>
            runtime.resolveCachedMediaPath(
          conversationId: conversationId,
          attachmentId: attachmentId,
          fileName: fileName,
          contentType: contentType,
          clearByteSize: clearByteSize,
        ),
        onDownloadAttachment: (conversationId, controlPlaintext) =>
            runtime.downloadAttachment(
          conversationId: conversationId,
          controlPlaintext: controlPlaintext,
        ),
        onSync: runtime.retryOutgoing,
        onStartRealtime: runtime.startRealtimeSync,
        onDeleteConversation: () =>
            runtime.deleteLocalConversation(conversationId),
      ),
    ),
  );
}
