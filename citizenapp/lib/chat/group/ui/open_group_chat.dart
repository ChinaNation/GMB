import 'package:flutter/material.dart';

import 'package:citizenapp/chat/chat_page.dart';
import 'package:citizenapp/chat/chat_runtime.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 打开某群的群聊详情。
///
/// 群目前支持文本 + emoji(emoji 即文本);媒体/贴纸群发是后续步,故只接
/// `onSendText`。复用 1:1 的 `ChatPage`,发送走 `runtime.sendGroupText`。
Future<void> openGroupChat(
  BuildContext context, {
  required String groupId,
  required String title,
}) async {
  final accountId = (await WalletManager().getDefaultWallet())?.accountId ?? '';
  if (accountId.isEmpty) {
    if (!context.mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('请先在「我的 → 我的钱包」创建热钱包')),
    );
    return;
  }
  final runtime = ChatRuntime();
  if (!context.mounted) return;
  await Navigator.of(context).push<void>(
    MaterialPageRoute(
      builder: (_) => ChatPage(
        conversationId: groupId,
        accountId: accountId,
        peerUserId: groupId,
        title: title,
        isGroup: true,
        onSendText: (text) =>
            runtime.sendGroupText(groupId: groupId, text: text),
        onSendSticker: (packId, stickerId) => runtime.sendGroupSticker(
          groupId: groupId,
          packId: packId,
          stickerId: stickerId,
        ),
        onSendMedia: (media) =>
            runtime.sendGroupMedia(groupId: groupId, media: media),
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
        onDeleteConversation: () => runtime.deleteLocalConversation(groupId),
      ),
    ),
  );
}
