import 'package:flutter/material.dart';

import 'package:citizenapp/im/im_chat_page.dart';
import 'package:citizenapp/im/im_runtime.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

typedef DirectChatOpener = Future<void> Function(
  BuildContext context, {
  required String peerAddress,
  required String title,
});

/// 打开与某钱包地址的一对一聊天。
///
/// sender = 默认热钱包地址（空则引导创建热钱包）。广场用户主页「消息」与联系人详情
/// 「消息」共用此入口，复用现有 IM 运行态，避免重复拼装。
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
  final runtime = ImRuntime();
  final conversationId = ImRuntime.directConversationId(sender, peerAddress);
  if (!context.mounted) return;
  await Navigator.of(context).push<void>(
    MaterialPageRoute(
      builder: (_) => ImChatPage(
        conversationId: conversationId,
        currentUserId: sender,
        peerUserId: peerAddress,
        title: title,
        onSendText: (text) => runtime.sendText(
          peerWalletAddress: peerAddress,
          conversationId: conversationId,
          text: text,
        ),
        onSendAttachment: (attachment) => runtime.sendAttachment(
          peerWalletAddress: peerAddress,
          conversationId: conversationId,
          attachment: attachment,
        ),
        onDownloadAttachment: (conversationId, controlPlaintext) =>
            runtime.downloadAttachment(
          conversationId: conversationId,
          controlPlaintext: controlPlaintext,
        ),
        onSync: runtime.syncPending,
        onStartRealtime: runtime.startRealtimeSync,
        onDeleteConversation: () =>
            runtime.deleteLocalConversation(conversationId),
      ),
    ),
  );
}
