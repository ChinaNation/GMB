import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:math';
import 'dart:typed_data';

import 'package:path_provider/path_provider.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../8964/services/square_api_client.dart';
import '../wallet/core/device_subkey.dart';
import '../wallet/core/wallet_manager.dart';
import 'crypto/chat_device_binding.dart';
import 'crypto/mls_boundary.dart';
import 'crypto/mls_group_boundary.dart';
import 'crypto/mls_native.dart';
import 'crypto/mls_state_store.dart';
import 'chat_flow.dart';
import 'chat_media_limits.dart';
import 'chat_models.dart';
import 'chat_payload.dart';
import 'chat_push_service.dart';
import 'group/group_flow.dart';
import 'group/group_model.dart';
import 'media/chat_relay_media.dart';
import 'media/media_resend.dart';
import 'proto/chat_envelope.pb.dart';
import 'storage/chat_store.dart';
import 'transport/chat_cloud_transport.dart';
import 'transport/chat_transport.dart';
import 'transport/chat_webrtc_transport.dart';

typedef ChatLoginSigner = Future<String> Function({
  required int walletIndex,
  required String ownerAccount,
  required Uint8List loginMessage,
});

typedef ChatDeviceBindingSigner = Future<String> Function({
  required int walletIndex,
  required String ownerAccount,
  required Uint8List bindingMessage,
});

typedef ChatCloudTransportFactory = ChatCloudTransport Function({
  required String ownerAccount,
  required String ownerDeviceId,
  Uri? serviceBaseUrl,
  String? sessionToken,
});

typedef ChatPushTokenProvider = Future<ChatPushToken> Function();

typedef MlsStateStoreFactory = Future<MlsStateStore> Function(
  String ownerAccount,
  String deviceId,
);

/// 系统推送唤醒后的短时后台收发窗口。
///
/// Cloudflare 不代存消息，因此接收设备被唤醒后必须主动建立瞬时连接。若发送设备
/// 此刻离线，`peer_ready` 会反向唤醒发送设备，由其本机队列继续投递。
@pragma('vm:entry-point')
Future<void> chatRuntimeBackgroundHandler(RemoteMessage message) async {
  final sender = ChatPushService.wakeSenderFromData(message.data);
  if (sender == null) return;
  await ChatPushService.storeWakeSender(sender);

  final push = ChatPushService();
  try {
    await ensureChatFirebaseReady();
    final runtime = ChatRuntime(
      pushService: push,
      pushTokenProvider: () => push.readToken(requestPermission: false),
    );
    final owner = await runtime.readOwnerAccount();
    if (owner == null) return;
    final stop = await runtime.startRealtimeSync(onNotice: () async {});
    if (stop == null) return;
    await Future<void>.delayed(const Duration(seconds: 20));
    await stop();
  } catch (_) {
    // 后台执行时间由系统控制；失败后保留发送方提示，前台恢复时继续重试。
  } finally {
    await push.dispose();
  }
}

class _ChatOwnerContext {
  const _ChatOwnerContext({
    required this.account,
    required this.deviceId,
    required this.devicePublicKeyHex,
    required this.crypto,
    required this.transport,
    required this.webrtc,
    required this.sessionExpiresAt,
  });

  final _ChatOwner account;
  final String deviceId;
  final String devicePublicKeyHex;
  final MlsCrypto crypto;
  final ChatCloudTransport transport;
  final ChatWebrtcTransport webrtc;
  final int sessionExpiresAt;

  bool get isUsable =>
      sessionExpiresAt - ChatRuntime._sessionRefreshSkewMillis >
      DateTime.now().millisecondsSinceEpoch;

  ChatDevice get identity => ChatDevice(
        ownerAccount: account.address,
        deviceId: deviceId,
        devicePublicKeyHex: devicePublicKeyHex,
      );
}

class _ChatOwner {
  const _ChatOwner({
    required this.walletIndex,
    required this.address,
    required this.walletName,
  });

  final int walletIndex;
  final String address;
  final String walletName;
}

/// 公民 Chat 运行态编排服务。
///
/// 页面层不直接操作 OpenMLS、Cloudflare 瞬时转发、近场通道和 Isar。
/// 这个服务负责读取默认用户钱包、建立设备身份，并把聊天发送
/// /同步接到正式 transport。登录和设备绑定只使用硬件 P-256 设备子钥；钱包
/// seed、钱包主私钥和生物识别不得进入任何 Chat 初始化或收发路径。
class ChatRuntime {
  ChatRuntime({
    ChatStore? store,
    WalletManager? walletManager,
    SharedPreferences? preferences,
    SquareApiClient? squareApiClient,
    ChatLoginSigner? loginSigner,
    ChatDeviceBindingSigner? deviceBindingSigner,
    DeviceSubkey? deviceSubkey,
    MlsStateStoreFactory? stateStoreFactory,
    MlsCrypto Function(
      ChatDevice identity,
      MlsStateStore stateStore,
    )? cryptoFactory,
    ChatCloudTransportFactory? cloudTransportFactory,
    ChatPushService? pushService,
    ChatPushTokenProvider? pushTokenProvider,
  })  : _store = store ?? ChatStore(),
        _walletManager = walletManager ?? WalletManager(),
        _preferences = preferences,
        _squareApiClient = squareApiClient ?? SquareApiClient(),
        _loginSigner = loginSigner,
        _deviceBindingSigner = deviceBindingSigner,
        _deviceSubkey = deviceSubkey ?? DeviceSubkey(),
        _stateStoreFactory = stateStoreFactory,
        _cryptoFactory = cryptoFactory,
        _cloudTransportFactory = cloudTransportFactory,
        _pushService = pushService ?? ChatPushService(),
        _pushTokenProvider = pushTokenProvider;

  static const _kDeviceId = 'chat.device.id';
  static const _kDevicePublicKeyHex = 'chat.device.public_key_hex';
  static const _kDeviceBindingPrefix = 'chat.cloudflare.device_binding';
  static const _kPushTokenPrefix = 'chat.push.token';
  static const _kKeyPackagePublishedPrefix =
      'chat.cloudflare.key_package_until';
  static const _deviceBindingTtl = Duration(days: 90);
  static const _keyPackageRefreshSkewMillis = 24 * 60 * 60 * 1000;
  static const _sessionRefreshSkewMillis = 60 * 1000;

  final ChatStore _store;
  final WalletManager _walletManager;
  final SharedPreferences? _preferences;
  final SquareApiClient _squareApiClient;
  final ChatLoginSigner? _loginSigner;
  final ChatDeviceBindingSigner? _deviceBindingSigner;
  final DeviceSubkey _deviceSubkey;
  final MlsStateStoreFactory? _stateStoreFactory;
  final MlsCrypto Function(
    ChatDevice identity,
    MlsStateStore stateStore,
  )? _cryptoFactory;
  final ChatCloudTransportFactory? _cloudTransportFactory;
  final ChatPushService _pushService;
  final ChatPushTokenProvider? _pushTokenProvider;

  /// 正在经 WebRTC 传输字节的媒体 attachmentId(初始发送或补发中),用于去重:
  /// peer_ready 触发的补发不得对在途媒体再整块重传。
  final Set<String> _mediaBytesInFlight = {};

  /// 同一账户/设备只允许一条初始化链。成功上下文复用到 session 临近过期；
  /// 失败只释放命中的 future，不得误删后来创建的新初始化。
  final Map<String, Future<_ChatOwnerContext>> _readyFlights = {};
  final Map<String, _ChatOwnerContext> _readyContexts = {};
  final Map<String, String> _ownerContextKeys = {};
  final Map<String, int> _ownerGenerations = {};

  Future<SharedPreferences> get _prefs async {
    final provided = _preferences;
    if (provided != null) {
      return provided;
    }
    return SharedPreferences.getInstance();
  }

  Future<ChatInboxOverview> readOverview({
    String? ownerAccount,
    required int pendingOutgoing,
    required int unreadCount,
  }) async {
    final account = ownerAccount ?? await readOwnerAccount();
    return ChatInboxOverview(
      ownerAccount: account,
      pendingOutgoing: pendingOutgoing,
      unreadCount: unreadCount,
    );
  }

  Future<String?> readOwnerAccount() async {
    final wallet = await _walletManager.getDefaultWallet();
    return wallet?.address;
  }

  /// 页面、轮询、WebSocket 和发送入口共享的唯一就绪入口。
  Future<void> ensureReady(String ownerAccount) async {
    final account = await _readOwner(expectedOwnerAccount: ownerAccount);
    await _readyContext(account);
  }

  /// 默认账户切换或本机 Chat 数据清理时精确失效该账户上下文。
  void invalidateOwner(String ownerAccount) {
    _ownerGenerations[ownerAccount] =
        (_ownerGenerations[ownerAccount] ?? 0) + 1;
    _readyFlights.remove(ownerAccount);
    final key = _ownerContextKeys.remove(ownerAccount);
    if (key != null) {
      _readyContexts.remove(key);
    }
  }

  static String directConversationId(
    String senderWalletAddress,
    String peerAccount,
  ) {
    return 'dm:$senderWalletAddress:$peerAccount';
  }

  Future<List<ChatDeliveryResult>> sendText({
    required String peerAccount,
    required String conversationId,
    required String text,
  }) async {
    final context = await _readyContext(await _readOwner());
    final flow = _messageFlow(context);
    try {
      return await flow.sendText(
        conversationId: conversationId,
        senderAccount: context.account.address,
        recipientAccount: peerAccount,
        senderDeviceId: context.deviceId,
        text: text,
      );
    } catch (error) {
      if (!_needsFirstKeyPackage(error)) {
        rethrow;
      }
      final packages = await context.transport.fetchKeyPackages(
        ownerAccount: peerAccount,
        requesterAccount: context.account.address,
      );
      if (packages.isEmpty) {
        throw StateError('对方没有可用 Chat KeyPackage');
      }
      final consumed = await context.transport.consumeKeyPackage(
        ownerAccount: peerAccount,
        keyPackageId: packages.first.keyPackageId,
        requesterAccount: context.account.address,
      );
      return flow.sendText(
        conversationId: conversationId,
        senderAccount: context.account.address,
        recipientAccount: peerAccount,
        senderDeviceId: context.deviceId,
        recipientKeyPackage: consumed,
        text: text,
      );
    }
  }

  Future<List<ChatDeliveryResult>> sendMedia({
    required String peerAccount,
    required String conversationId,
    required ChatMediaDraft media,
  }) async {
    final context = await _readyContext(await _readOwner());
    final flow = _messageFlow(context);
    // 登记/清除"待设备投递":对方离线时字节发不出,留 pending 由上线补发。缓存路径
    // 不持久化(补发时按当前 Documents 重算),只存 conversationId/attachmentId/fileName。
    Future<void> recordPending(String attachmentId) =>
        _store.recordOutgoingMedia(
          attachmentId: attachmentId,
          recipientAccount: peerAccount,
          conversationId: conversationId,
          fileName: media.fileName,
          contentType: media.contentType,
          byteSize: media.byteSize,
        );
    Future<void> markDelivered(String attachmentId) =>
        _store.deleteOutgoingMedia(attachmentId, peerAccount);
    try {
      return await flow.sendMedia(
        conversationId: conversationId,
        senderAccount: context.account.address,
        recipientAccount: peerAccount,
        senderDeviceId: context.deviceId,
        media: media,
        sendDeviceAttachment: _guardedDeviceSender(context),
        saveLocalAttachment: _copySentAttachmentToCache,
        recordPendingMedia: recordPending,
        onDeviceDelivered: markDelivered,
        uploadRelayMedia: _relayUploader(context),
      );
    } catch (error) {
      if (!_needsFirstKeyPackage(error)) {
        rethrow;
      }
      final packages = await context.transport.fetchKeyPackages(
        ownerAccount: peerAccount,
        requesterAccount: context.account.address,
      );
      if (packages.isEmpty) {
        throw StateError('对方没有可用 Chat KeyPackage');
      }
      final consumed = await context.transport.consumeKeyPackage(
        ownerAccount: peerAccount,
        keyPackageId: packages.first.keyPackageId,
        requesterAccount: context.account.address,
      );
      return flow.sendMedia(
        conversationId: conversationId,
        senderAccount: context.account.address,
        recipientAccount: peerAccount,
        senderDeviceId: context.deviceId,
        recipientKeyPackage: consumed,
        media: media,
        sendDeviceAttachment: _guardedDeviceSender(context),
        saveLocalAttachment: _copySentAttachmentToCache,
        recordPendingMedia: recordPending,
        onDeviceDelivered: markDelivered,
        uploadRelayMedia: _relayUploader(context),
      );
    }
  }

  /// 包住 WebRTC 字节发送,把 attachmentId 计入在途集合(去重防双传),结束即移除。
  ChatAttachmentDeviceSender _guardedDeviceSender(_ChatOwnerContext context) {
    return ({
      required recipientAccount,
      required conversationId,
      required attachmentId,
      required fileName,
      required contentType,
      required sourcePath,
      required byteSize,
    }) async {
      final inFlightKey =
          MediaResend.inFlightKey(attachmentId, recipientAccount);
      _mediaBytesInFlight.add(inFlightKey);
      try {
        await context.webrtc.sendAttachment(
          recipientAccount: recipientAccount,
          conversationId: conversationId,
          attachmentId: attachmentId,
          fileName: fileName,
          contentType: contentType,
          sourcePath: sourcePath,
          byteSize: byteSize,
        );
      } finally {
        _mediaBytesInFlight.remove(inFlightKey);
      }
    };
  }

  /// 大媒体(>100MB)中转上传 seam:加密源文件 → 上传密文到 R2 → 返回描述子。
  ChatRelayUploader _relayUploader(_ChatOwnerContext context) {
    return ({
      required conversationId,
      required attachmentId,
      required media,
      int recipientCount = 1,
    }) async {
      final dir = await getApplicationDocumentsDirectory();
      return ChatRelayMedia.upload(
        transport: context.transport,
        sourcePath: media.sourcePath,
        byteSize: media.byteSize,
        recipientCount: recipientCount,
        tempDirectory: Directory('${dir.path}/chat/attachments/.tmp'),
      );
    };
  }

  /// 发送内置贴纸:只走控制信封,不经 WebRTC。首次会话缺 KeyPackage 时同样
  /// 领取后重试。
  Future<List<ChatDeliveryResult>> sendSticker({
    required String peerAccount,
    required String conversationId,
    required String packId,
    required String stickerId,
  }) async {
    final context = await _readyContext(await _readOwner());
    final flow = _messageFlow(context);
    try {
      return await flow.sendSticker(
        conversationId: conversationId,
        senderAccount: context.account.address,
        recipientAccount: peerAccount,
        senderDeviceId: context.deviceId,
        packId: packId,
        stickerId: stickerId,
      );
    } catch (error) {
      if (!_needsFirstKeyPackage(error)) {
        rethrow;
      }
      final packages = await context.transport.fetchKeyPackages(
        ownerAccount: peerAccount,
        requesterAccount: context.account.address,
      );
      if (packages.isEmpty) {
        throw StateError('对方没有可用 Chat KeyPackage');
      }
      final consumed = await context.transport.consumeKeyPackage(
        ownerAccount: peerAccount,
        keyPackageId: packages.first.keyPackageId,
        requesterAccount: context.account.address,
      );
      return flow.sendSticker(
        conversationId: conversationId,
        senderAccount: context.account.address,
        recipientAccount: peerAccount,
        senderDeviceId: context.deviceId,
        recipientKeyPackage: consumed,
        packId: packId,
        stickerId: stickerId,
      );
    }
  }

  // ==== 私密小群 ====

  /// 建群:选联系人账户,领其 KeyPackage 批量加入,创建者为 admin。
  Future<ChatGroup> createGroup({
    required String name,
    List<String> inviteeAccounts = const [],
  }) async {
    final context = await _readyContext(await _readOwner());
    final invitees = await _fetchInviteeKeyPackages(context, inviteeAccounts);
    final groupId = newGroupId(context.account.address);
    return _groupFlow(context).createGroup(
      groupId: groupId,
      name: name,
      ownerAccount: context.account.address,
      ownerDeviceId: context.deviceId,
      invitees: invitees,
    );
  }

  /// 加人(仅 admin)。
  Future<void> addGroupMembers({
    required String groupId,
    required List<String> inviteeAccounts,
  }) async {
    final context = await _readyContext(await _readOwner());
    final invitees = await _fetchInviteeKeyPackages(context, inviteeAccounts);
    await _groupFlow(context).addMembers(
      groupId: groupId,
      actorAccount: context.account.address,
      actorDeviceId: context.deviceId,
      invitees: invitees,
    );
  }

  /// 删人(仅 admin,按账户)。
  Future<void> removeGroupMembers({
    required String groupId,
    required List<String> targetAccounts,
  }) async {
    final context = await _readyContext(await _readOwner());
    await _groupFlow(context).removeMembers(
      groupId: groupId,
      actorAccount: context.account.address,
      actorDeviceId: context.deviceId,
      targetAccounts: targetAccounts,
    );
  }

  /// 退群(本机标记已退,并发退群请求让 admin 重钥)。
  Future<void> leaveGroup(String groupId) async {
    final context = await _readyContext(await _readOwner());
    await _groupFlow(context).leaveGroup(groupId);
  }

  /// 改群名(仅 admin)。
  Future<void> renameGroup({
    required String groupId,
    required String name,
  }) async {
    final context = await _readyContext(await _readOwner());
    await _groupFlow(context).renameGroup(groupId, name);
  }

  /// 群发文本。
  Future<List<ChatDeliveryResult>> sendGroupText({
    required String groupId,
    required String text,
  }) async {
    final context = await _readyContext(await _readOwner());
    return _groupFlow(context).sendGroupText(
      groupId: groupId,
      senderAccount: context.account.address,
      senderDeviceId: context.deviceId,
      text: text,
    );
  }

  /// 群发内置贴纸(零字节,收端本地渲染)。
  Future<List<ChatDeliveryResult>> sendGroupSticker({
    required String groupId,
    required String packId,
    required String stickerId,
  }) async {
    final context = await _readyContext(await _readOwner());
    return _groupFlow(context).sendGroupSticker(
      groupId: groupId,
      senderAccount: context.account.address,
      senderDeviceId: context.deviceId,
      packId: packId,
      stickerId: stickerId,
    );
  }

  /// 群发媒体:≤100MB 对每个成员 WebRTC 直传(离线按成员补发);>100MB 走已部署中转
  /// (一次上传 + K 扇 N,仅薪火可发/可收)。四门按己档强制。
  Future<List<ChatDeliveryResult>> sendGroupMedia({
    required String groupId,
    required ChatMediaDraft media,
  }) async {
    final context = await _readyContext(await _readOwner());
    return _groupFlow(context).sendGroupMedia(
      groupId: groupId,
      senderAccount: context.account.address,
      senderDeviceId: context.deviceId,
      media: media,
      sendMemberAttachment: _guardedDeviceSender(context),
      uploadRelayMedia: _relayUploader(context),
      saveLocalAttachment: _copySentAttachmentToCache,
      recordPendingMember: (attachmentId, member) =>
          _store.recordOutgoingMedia(
        attachmentId: attachmentId,
        recipientAccount: member,
        conversationId: groupId,
        fileName: media.fileName,
        contentType: media.contentType,
        byteSize: media.byteSize,
      ),
      markMemberDelivered: (attachmentId, member) =>
          _store.deleteOutgoingMedia(attachmentId, member),
    );
  }

  /// 逐个被邀请账户领取一枚 KeyPackage(复用 1:1 fetch/consume),
  /// 兜底补齐 ownerAccount 供群扇出定位新人。
  Future<List<MlsKeyPackage>> _fetchInviteeKeyPackages(
    _ChatOwnerContext context,
    List<String> inviteeAccounts,
  ) async {
    final packages = <MlsKeyPackage>[];
    for (final account in inviteeAccounts) {
      final available = await context.transport.fetchKeyPackages(
        ownerAccount: account,
        requesterAccount: context.account.address,
      );
      if (available.isEmpty) {
        throw StateError('对方 $account 没有可用 Chat KeyPackage');
      }
      final consumed = await context.transport.consumeKeyPackage(
        ownerAccount: account,
        keyPackageId: available.first.keyPackageId,
        requesterAccount: context.account.address,
      );
      packages.add(
        consumed.ownerAccount.isNotEmpty
            ? consumed
            : MlsKeyPackage(
                ownerAccount: account,
                deviceId: consumed.deviceId,
                keyPackageId: consumed.keyPackageId,
                keyPackageBytes: consumed.keyPackageBytes,
                cipherSuite: consumed.cipherSuite,
                createdAtMillis: consumed.createdAtMillis,
                expiresAtMillis: consumed.expiresAtMillis,
                devicePublicKeyHex: consumed.devicePublicKeyHex,
              ),
      );
    }
    return packages;
  }

  ChatGroupFlow _groupFlow(_ChatOwnerContext context) {
    return ChatGroupFlow(
      crypto: context.crypto as MlsGroupCrypto,
      store: _store,
      ownerAccount: context.account.address,
      ownerDeviceId: context.deviceId,
      deliverer: (envelope, _) {
        return ChatFlow.deliverWithTransport(
          transport: context.transport,
          envelope: envelope,
        );
      },
    );
  }

  /// 解析媒体在本机缓存中的绝对路径,供聊天页内联渲染。字节未到达(对方离线
  /// 或仍在 WebRTC 传输中)时返回 null,由 UI 显示占位。永不抛错。
  Future<String?> resolveCachedMediaPath({
    required String conversationId,
    required String attachmentId,
    required String fileName,
    required String contentType,
    required int clearByteSize,
  }) async {
    try {
      final dir = await getApplicationDocumentsDirectory();
      final cached = await ChatFlow.readCachedAttachment(
        conversationId: conversationId,
        attachmentId: attachmentId,
        fileName: fileName,
        contentType: contentType,
        clearByteSize: clearByteSize,
        cacheDirectory: Directory('${dir.path}/chat/attachments'),
      );
      return cached?.filePath;
    } catch (_) {
      return null;
    }
  }

  Future<ChatDownloadedAttachment> downloadAttachment({
    required String conversationId,
    required String controlPlaintext,
  }) async {
    final dir = await getApplicationDocumentsDirectory();
    final cacheDirectory = Directory('${dir.path}/chat/attachments');
    final content = ChatPayloadCodec.decode(controlPlaintext);
    if (content.isRelayMedia) {
      return _downloadRelayAttachment(conversationId, content, cacheDirectory);
    }
    return ChatFlow.downloadAttachment(
      conversationId: conversationId,
      controlPlaintext: controlPlaintext,
      cacheDirectory: cacheDirectory,
    );
  }

  /// >100MB 中转媒体的接收:门②(超本机会员档则拒收,非薪火收 >100MB 一律拒)→
  /// 命中缓存直接返回 → 否则换 URL 流式下载密文、解密落缓存。
  Future<ChatDownloadedAttachment> _downloadRelayAttachment(
    String conversationId,
    ChatContent content,
    Directory cacheDirectory,
  ) async {
    final byteSize = content.byteSize ?? 0;
    if (ChatMediaLimits.exceedsForKind(content.kind, byteSize)) {
      throw ChatMediaTooLargeException(
        byteSize: byteSize,
        limitBytes: ChatMediaLimits.forKind(content.kind),
        kind: content.kind,
      );
    }
    final attachmentId = content.attachmentId ?? '';
    final fileName = content.fileName ?? '';
    final contentType = content.mime ?? 'application/octet-stream';
    final cached = await ChatFlow.readCachedAttachment(
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: contentType,
      clearByteSize: byteSize,
      cacheDirectory: cacheDirectory,
    );
    if (cached != null) return cached;

    final context = await _readyContext(await _readOwner());
    final destPath = ChatFlow.attachmentCachePath(
      cacheDirectory: cacheDirectory,
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
    );
    await File(destPath).parent.create(recursive: true);
    await ChatRelayMedia.download(
      transport: context.transport,
      relayObjectKey: content.relayObjectKey ?? '',
      contentKeyB64: content.contentKeyB64 ?? '',
      destPath: destPath,
      tempDirectory: Directory('${cacheDirectory.path}/.tmp'),
    );
    return ChatDownloadedAttachment(
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: contentType,
      clearByteSize: byteSize,
      filePath: destPath,
    );
  }

  Future<void> deleteLocalConversation(String conversationId) async {
    await _store.deleteConversation(conversationId);
    final dir = await getApplicationDocumentsDirectory();
    final attachmentDir = Directory(
      '${dir.path}/chat/attachments/${_safePath(conversationId)}',
    );
    if (await attachmentDir.exists()) {
      await attachmentDir.delete(recursive: true);
    }
  }

  /// 重试发送设备本机队列中的密文,并补发待设备投递的媒体字节。
  /// 密文成功转交在线接收设备后立即删队列项;媒体字节收到 WebRTC ack 后删待投递行。
  Future<int> retryOutgoing({String? recipientAccount}) async {
    final context = await _readyContext(await _readOwner());
    final queued = await _store.readQueuedEnvelopes(
      recipientAccount: recipientAccount,
    );
    var sent = 0;
    for (final item in queued) {
      final result = await context.transport.sendEncryptedEnvelope(
        envelopeId: item.envelopeId,
        envelopeBytes: item.envelopeBytes,
      );
      await _store.markOutgoingDelivery(
        envelopeId: item.envelopeId,
        state: result.state,
        errorMessage: result.errorMessage,
      );
      if (result.state == ChatMessageDeliveryState.sent) sent += 1;
    }
    // 媒体字节补发**只在明确对端时(peer_ready 确知在线)**触发,且**不阻塞**:
    // 绝不在无差别的轮询/实时启动(recipientAccount==null)路径对离线对端反复整块
    // 重连重发(每条阻塞 45 秒、无退避),那会拖垮轮询与后台唤醒窗口。
    if (recipientAccount != null) {
      unawaited(
          _resendPendingMedia(context, recipientAccount: recipientAccount));
    }
    return sent;
  }

  /// 上线补发:遍历待设备投递的媒体,从本机缓存副本重发 WebRTC 字节。核心去重/清孤
  /// 儿/删行逻辑在可测的 [MediaResend.run];缓存路径按**当前 Documents 目录重算**
  /// (不用持久化的绝对路径,避免容器 UUID 变更后误判丢失)。
  Future<void> _resendPendingMedia(
    _ChatOwnerContext context, {
    required String recipientAccount,
  }) async {
    final pending = await _store.readPendingOutgoingMedia(
      recipientAccount: recipientAccount,
    );
    if (pending.isEmpty) return;
    final dir = await getApplicationDocumentsDirectory();
    final cacheDir = Directory('${dir.path}/chat/attachments');
    await MediaResend.run(
      pending: pending,
      inFlight: _mediaBytesInFlight,
      resolveCachePath: (media) => ChatFlow.attachmentCachePath(
        cacheDirectory: cacheDir,
        conversationId: media.conversationId,
        attachmentId: media.attachmentId,
        fileName: media.fileName,
      ),
      cacheFileExists: (path) => File(path).exists(),
      sendBytes: (media, path) => context.webrtc.sendAttachment(
        recipientAccount: media.recipientAccount,
        conversationId: media.conversationId,
        attachmentId: media.attachmentId,
        fileName: media.fileName,
        contentType: media.contentType,
        sourcePath: path,
        byteSize: media.byteSize,
      ),
      deletePending: (media) =>
          _store.deleteOutgoingMedia(media.attachmentId, media.recipientAccount),
    );
  }

  /// 门③:接收端落盘二次门控。委托给可单测的 [ChatFlow.acceptReceivedMediaToCache]
  /// (超限删临时不入缓存;否则移入缓存)。
  Future<void> _saveReceivedAttachmentToCache({
    required String senderAccount,
    required String conversationId,
    required String attachmentId,
    required String fileName,
    required String contentType,
    required String filePath,
    required int byteSize,
  }) async {
    final dir = await getApplicationDocumentsDirectory();
    await ChatFlow.acceptReceivedMediaToCache(
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: contentType,
      tempFilePath: filePath,
      byteSize: byteSize,
      cacheDirectory: Directory('${dir.path}/chat/attachments'),
    );
  }

  /// 发送端把自己发出的媒体**复制**一份进缓存(保留源),以便在会话里看到并支持
  /// 上线补发。
  Future<void> _copySentAttachmentToCache({
    required String conversationId,
    required String attachmentId,
    required String fileName,
    required String contentType,
    required String sourcePath,
    required int byteSize,
  }) async {
    final dir = await getApplicationDocumentsDirectory();
    await ChatFlow.importAttachmentFileToCache(
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: contentType,
      sourcePath: sourcePath,
      byteSize: byteSize,
      moveSource: false,
      cacheDirectory: Directory('${dir.path}/chat/attachments'),
    );
  }

  Future<Future<void> Function()?> startRealtimeSync({
    required Future<void> Function() onNotice,
    Future<void> Function()? onDisconnected,
  }) async {
    final context = await _readyContext(await _readOwner());
    final stopSocket = await context.transport.connectRealtime(
      onMessage: (message) async {
        final type = message['type'];
        if (type == 'gmb_chat_envelope_v2') {
          final encoded = message['envelope'];
          if (encoded is! String || encoded.isEmpty) return;
          final bytes = _base64UrlDecode(encoded);
          final conversationId = _peekConversationId(bytes);
          if (conversationId != null && conversationId.startsWith('grp:')) {
            await _groupFlow(context).processIncomingGroupEnvelope(bytes);
          } else {
            await _messageFlow(context).processIncomingEnvelopeBytes(bytes);
          }
          await onNotice();
          return;
        }
        if (type == 'gmb_chat_signal_v1') {
          final sender = message['sender_account'];
          final signal = message['signal'];
          if (sender is! String || signal is! Map<String, dynamic>) return;
          if (signal['kind'] == 'peer_ready') {
            await retryOutgoing(recipientAccount: sender);
          } else {
            await context.webrtc.handleSignal(sender, signal);
          }
        }
      },
      onDisconnected: onDisconnected,
    );
    if (stopSocket == null) return null;

    Future<void> notifySenderReady(String senderAccount) async {
      if (senderAccount.isEmpty) return;
      await context.transport.sendSignal(
        recipientAccount: senderAccount,
        signal: const {'kind': 'peer_ready'},
      );
    }

    final pushSubscription = _pushService.wakeSenders.listen(
      (sender) => unawaited(notifySenderReady(sender)),
    );
    final pendingSenders = await _pushService.takePendingWakeSenders();
    for (final sender in pendingSenders) {
      await notifySenderReady(sender);
    }
    final tokenSubscription = _pushService.tokenChanges.listen(
      (_) => unawaited(_refreshPushRegistration(context)),
    );
    await retryOutgoing();
    return () async {
      await pushSubscription.cancel();
      await tokenSubscription.cancel();
      await stopSocket();
    };
  }

  Future<void> _refreshPushRegistration(_ChatOwnerContext context) async {
    try {
      await _ensureDeviceRegistered(
        account: context.account,
        identity: context.identity,
        prefs: await _prefs,
        transport: context.transport,
      );
    } catch (_) {
      // Token 刷新失败不会删除旧登记；下一次平台回调或 Chat 初始化继续重试。
    }
  }

  Future<_ChatOwnerContext> _readyContext(_ChatOwner account) {
    final knownKey = _ownerContextKeys[account.address];
    final cached = knownKey == null ? null : _readyContexts[knownKey];
    if (cached != null && cached.isUsable) {
      return Future.value(cached);
    }
    if (knownKey != null) {
      _readyContexts.remove(knownKey);
    }

    final flightKey = account.address;
    final existing = _readyFlights[flightKey];
    if (existing != null) {
      return existing;
    }

    final generation = _ownerGenerations[account.address] ?? 0;
    late final Future<_ChatOwnerContext> created;
    created = _buildOwnerContext(account).then((context) {
      if ((_ownerGenerations[account.address] ?? 0) != generation) {
        throw StateError('聊天账户已切换，本次旧初始化结果已丢弃');
      }
      final contextKey = _contextKey(context.identity);
      final previousKey = _ownerContextKeys[account.address];
      if (previousKey != null && previousKey != contextKey) {
        _readyContexts.remove(previousKey);
      }
      _ownerContextKeys[account.address] = contextKey;
      _readyContexts[contextKey] = context;
      return context;
    }).whenComplete(() {
      if (identical(_readyFlights[flightKey], created)) {
        _readyFlights.remove(flightKey);
      }
    });
    _readyFlights[flightKey] = created;
    return created;
  }

  Future<_ChatOwnerContext> _buildOwnerContext(_ChatOwner account) async {
    final prefs = await _prefs;
    var deviceId = prefs.getString(_kDeviceId);
    if (deviceId == null || deviceId.isEmpty) {
      deviceId = 'chat-${_newNonce()}';
      await prefs.setString(_kDeviceId, deviceId);
    }

    var devicePublicKeyHex = prefs.getString(_kDevicePublicKeyHex) ?? '';
    final stateStore = await _stateStore(account.address, deviceId);
    var identity = ChatDevice(
      ownerAccount: account.address,
      deviceId: deviceId,
      devicePublicKeyHex:
          devicePublicKeyHex.isEmpty ? '00' : devicePublicKeyHex,
    );
    final crypto = _cryptoFactory?.call(identity, stateStore) ??
        NativeMlsCrypto(identity: identity, stateStore: stateStore);
    MlsKeyPackage? freshKeyPackage;
    if (devicePublicKeyHex.isEmpty) {
      freshKeyPackage = await crypto.createKeyPackage(identity);
      final keyPackage = freshKeyPackage;
      if (keyPackage.devicePublicKeyHex.isEmpty) {
        throw StateError('OpenMLS native 未返回 Chat 设备公钥，请先重编 native 库');
      }
      devicePublicKeyHex = keyPackage.devicePublicKeyHex;
      await prefs.setString(_kDevicePublicKeyHex, devicePublicKeyHex);
      identity = ChatDevice(
        ownerAccount: account.address,
        deviceId: deviceId,
        devicePublicKeyHex: devicePublicKeyHex,
      );
    }
    final finalCrypto = _cryptoFactory?.call(identity, stateStore) ??
        NativeMlsCrypto(identity: identity, stateStore: stateStore);
    final service = await _ensureServiceReady(
      account: account,
      identity: identity,
      crypto: finalCrypto,
      prefs: prefs,
      initialKeyPackage: freshKeyPackage,
    );
    final transport = _cloudTransportFactory?.call(
          ownerAccount: account.address,
          ownerDeviceId: deviceId,
          serviceBaseUrl: service.baseUri,
          sessionToken: service.session.sessionToken,
        ) ??
        ChatCloudTransport(
          ownerAccount: account.address,
          ownerDeviceId: deviceId,
          serviceBaseUrl: service.baseUri,
          sessionToken: service.session.sessionToken,
          requestSigner: service.session.signRequest,
        );
    final docsDir = await getApplicationDocumentsDirectory();
    final tempDirectory = '${docsDir.path}/chat/attachments/.tmp';
    // 回收被永久放弃的续传残档(对端删会话/待投递后不会再续写的 .part)。
    unawaited(ChatAttachmentReceiveBuffer.sweepStalePartials(tempDirectory));
    final webrtc = ChatWebrtcTransport(
      ownerAccount: account.address,
      cloud: transport,
      tempDirectory: tempDirectory,
      onAttachment: _saveReceivedAttachmentToCache,
    );
    return _ChatOwnerContext(
      account: account,
      deviceId: deviceId,
      devicePublicKeyHex: identity.devicePublicKeyHex,
      crypto: finalCrypto,
      transport: transport,
      webrtc: webrtc,
      sessionExpiresAt: service.session.expiresAt,
    );
  }

  Future<_ChatServiceContext> _ensureServiceReady({
    required _ChatOwner account,
    required ChatDevice identity,
    required MlsCrypto crypto,
    required SharedPreferences prefs,
    MlsKeyPackage? initialKeyPackage,
  }) async {
    // 后台会话握手绝不读 seed / 不弹窗 / 不懒注册：子钥只在钱包创建时静默注册。
    // 未注册设备（旧格式钱包等）在此直接会话失败，按不可用降级处理，绝不在合并主线程弹 Turnstile。
    final session = await _squareApiClient.ensureSession(
      ownerAccount: account.address,
      signLoginPayload: (payload) => _signSquareLoginPayload(account, payload),
    );
    final transport = _cloudTransportFactory?.call(
          ownerAccount: account.address,
          ownerDeviceId: identity.deviceId,
          serviceBaseUrl: _squareApiClient.baseUri,
          sessionToken: session.sessionToken,
        ) ??
        ChatCloudTransport(
          ownerAccount: account.address,
          ownerDeviceId: identity.deviceId,
          serviceBaseUrl: _squareApiClient.baseUri,
          sessionToken: session.sessionToken,
          requestSigner: session.signRequest,
        );

    await _ensureDeviceRegistered(
      account: account,
      identity: identity,
      prefs: prefs,
      transport: transport,
    );
    await _ensureOwnKeyPackagePublished(
      identity: identity,
      crypto: crypto,
      prefs: prefs,
      transport: transport,
      initialKeyPackage: initialKeyPackage,
    );
    return _ChatServiceContext(
      baseUri: _squareApiClient.baseUri,
      session: session,
    );
  }

  Future<void> _ensureDeviceRegistered({
    required _ChatOwner account,
    required ChatDevice identity,
    required SharedPreferences prefs,
    required ChatCloudTransport transport,
  }) async {
    final cacheKey = _deviceBindingCacheKey(identity);
    final cachedExpiresAt = prefs.getInt(cacheKey) ?? 0;
    final now = DateTime.now().millisecondsSinceEpoch;
    final pushToken = await _readPushToken();
    final pushCacheKey = _pushTokenCacheKey(identity);
    if (cachedExpiresAt - _keyPackageRefreshSkewMillis > now &&
        prefs.getString(pushCacheKey) == pushToken.token) {
      return;
    }

    final expiresAt = DateTime.now().toUtc().add(_deviceBindingTtl);
    final binding = ChatDeviceBinding(
      ownerAccount: account.address,
      deviceId: identity.deviceId,
      devicePublicKeyHex: identity.devicePublicKeyHex,
      expiresAt: expiresAt,
      nonce: _newNonce(),
    );
    final signatureHex = await _signDeviceBinding(
      account: account,
      bindingMessage: binding.signingMessage(),
    );
    await transport.registerDevice(
      devicePublicKeyHex: identity.devicePublicKeyHex,
      pushProvider: pushToken.provider,
      pushToken: pushToken.token,
      bindingSignature: signatureHex,
      expiresAtMillis: expiresAt.millisecondsSinceEpoch,
      nonce: binding.nonce,
    );
    await prefs.setInt(cacheKey, expiresAt.millisecondsSinceEpoch);
    await prefs.setString(pushCacheKey, pushToken.token);
  }

  Future<ChatPushToken> _readPushToken() {
    return _pushTokenProvider?.call() ?? _pushService.initialize();
  }

  Future<void> _ensureOwnKeyPackagePublished({
    required ChatDevice identity,
    required MlsCrypto crypto,
    required SharedPreferences prefs,
    required ChatCloudTransport transport,
    MlsKeyPackage? initialKeyPackage,
  }) async {
    final cacheKey = _keyPackageCacheKey(identity);
    final cachedUntil = prefs.getInt(cacheKey) ?? 0;
    final now = DateTime.now().millisecondsSinceEpoch;
    if (cachedUntil - _keyPackageRefreshSkewMillis > now) {
      return;
    }

    final keyPackage =
        initialKeyPackage ?? await crypto.createKeyPackage(identity);
    if (keyPackage.devicePublicKeyHex.isNotEmpty &&
        keyPackage.devicePublicKeyHex.toLowerCase() !=
            identity.devicePublicKeyHex.toLowerCase()) {
      throw StateError('OpenMLS native 返回的 Chat 设备公钥与本机身份不一致');
    }
    await transport.publishKeyPackage(keyPackage);
    await prefs.setInt(cacheKey, keyPackage.expiresAtMillis);
  }

  Future<String> _signSquareLoginPayload(
    _ChatOwner account,
    Uint8List loginMessage,
  ) async {
    final signer = _loginSigner;
    if (signer != null) {
      return signer(
        walletIndex: account.walletIndex,
        ownerAccount: account.address,
        loginMessage: loginMessage,
      );
    }
    // 会话握手 = 非用户动权 → P-256 硬件子钥静默签名 signing_message 摘要（不读 seed、不弹生物识别）。
    final raw =
        await _deviceSubkey.signRawHex(account.walletIndex, loginMessage);
    return '0x$raw';
  }

  Future<String> _signDeviceBinding({
    required _ChatOwner account,
    required Uint8List bindingMessage,
  }) async {
    final signer = _deviceBindingSigner;
    if (signer != null) {
      return signer(
        walletIndex: account.walletIndex,
        ownerAccount: account.address,
        bindingMessage: bindingMessage,
      );
    }
    // 与 Worker 使用同一硬件 P-256 子钥；该原生 key 无 user-auth 门禁。
    final raw = await _deviceSubkey.signRawHex(
      account.walletIndex,
      bindingMessage,
    );
    return '0x$raw';
  }

  Future<_ChatOwner> _readOwner({String? expectedOwnerAccount}) async {
    // 身份统一取默认用户钱包（钱包列表中最靠前的热钱包）。
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) {
      throw StateError('请先在「我的 → 我的钱包」创建热钱包，第一个热钱包即默认用户');
    }
    if (!wallet.isHotWallet) {
      throw StateError('默认用户必须是热钱包');
    }
    if (expectedOwnerAccount != null &&
        wallet.address != expectedOwnerAccount) {
      throw StateError('默认用户已切换，请重新进入聊天');
    }
    return _ChatOwner(
      walletIndex: wallet.walletIndex,
      address: wallet.address,
      walletName: wallet.walletName,
    );
  }

  ChatFlow _messageFlow(_ChatOwnerContext context) {
    return ChatFlow(
      crypto: context.crypto,
      store: _store,
      deliverer: (envelope, _) {
        return ChatFlow.deliverWithTransport(
          transport: context.transport,
          envelope: envelope,
        );
      },
    );
  }

  Future<MlsStateStore> _stateStore(
    String ownerAccount,
    String deviceId,
  ) async {
    final factory = _stateStoreFactory;
    if (factory != null) {
      return factory(ownerAccount, deviceId);
    }
    final dir = await getApplicationDocumentsDirectory();
    final safeWallet = _safePath(ownerAccount);
    final safeDevice = _safePath(deviceId);
    return MlsStateStore(
      Directory('${dir.path}/chat/mls/$safeWallet/$safeDevice'),
    );
  }
}

class _ChatServiceContext {
  const _ChatServiceContext({
    required this.baseUri,
    required this.session,
  });

  final Uri baseUri;
  final SquareSession session;
}

bool _needsFirstKeyPackage(Object error) {
  return error.toString().contains('首次 MLS 会话必须提供');
}

String _newNonce() {
  final random = Random.secure();
  final bytes = List<int>.generate(16, (_) => random.nextInt(256));
  return bytes.map((item) => item.toRadixString(16).padLeft(2, '0')).join();
}

String _safePath(String value) {
  return value.replaceAll(RegExp(r'[^a-zA-Z0-9_.-]'), '_');
}

String _contextKey(ChatDevice identity) {
  return '${identity.ownerAccount}|${identity.deviceId}|'
      '${identity.devicePublicKeyHex.toLowerCase()}';
}

String _deviceBindingCacheKey(ChatDevice identity) {
  return '${ChatRuntime._kDeviceBindingPrefix}.'
      '${_safePath(identity.ownerAccount)}.'
      '${_safePath(identity.deviceId)}.${identity.devicePublicKeyHex}';
}

String _keyPackageCacheKey(ChatDevice identity) {
  return '${ChatRuntime._kKeyPackagePublishedPrefix}.'
      '${_safePath(identity.ownerAccount)}.'
      '${_safePath(identity.deviceId)}.${identity.devicePublicKeyHex}';
}

String _pushTokenCacheKey(ChatDevice identity) {
  return '${ChatRuntime._kPushTokenPrefix}.'
      '${_safePath(identity.ownerAccount)}.${_safePath(identity.deviceId)}';
}

List<int> _base64UrlDecode(String value) {
  final normalized = value.padRight((value.length + 3) ~/ 4 * 4, '=');
  return base64Url.decode(normalized);
}

/// 只读取 envelope 的 conversation_id 以决定路由(群 `grp:` vs 私聊 `dm:`);
/// 解析失败返回 null,交由原私聊路径兜底。
String? _peekConversationId(List<int> envelopeBytes) {
  try {
    return ChatEnvelope.fromBuffer(envelopeBytes).conversationId;
  } catch (_) {
    return null;
  }
}
