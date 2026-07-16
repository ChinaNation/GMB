import 'package:flutter/material.dart';

import 'package:citizenapp/chat/chat_page.dart';
import 'package:citizenapp/chat/chat_runtime.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

typedef DirectChatOpener = Future<void> Function(
  BuildContext context, {
  required String peerAddress,
  required String title,
});

/// 打开与某钱包地址的一对一聊天。
///
/// sender = 默认热钱包地址（空则引导创建热钱包）。广场用户主页「消息」与联系人详情
/// 「消息」共用此入口，复用现有 Chat 运行态，避免重复拼装。
Future<void> openDirectChat(
  BuildContext context, {
  required String peerAddress,
  required String title,
}) async {
  final sender = (await WalletManager().getDefaultWallet())?.address ?? '';
  if (sender.isEmpty) {
    if (!context.mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('请先在「我的 → 我的钱包」创建热钱包')),
    );
    return;
  }
  final runtime = ChatRuntime();
  final conversationId = ChatRuntime.directConversationId(sender, peerAddress);
  if (!context.mounted) return;
  await Navigator.of(context).push<void>(
    MaterialPageRoute(
      builder: (_) => ChatPage(
        conversationId: conversationId,
        ownerAccount: sender,
        peerUserId: peerAddress,
        title: title,
        onSendText: (text) => runtime.sendText(
          peerAccount: peerAddress,
          conversationId: conversationId,
          text: text,
        ),
        onSendMedia: (media) => runtime.sendMedia(
          peerAccount: peerAddress,
          conversationId: conversationId,
          media: media,
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
