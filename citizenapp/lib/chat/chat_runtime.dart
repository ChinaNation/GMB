import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:math';
import 'dart:typed_data';

import 'package:path_provider/path_provider.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../8964/services/square_api_client.dart';
import '../my/myid/identity_account_cache.dart';
import '../wallet/core/device_subkey.dart';
import '../security/local_data_key.dart';
import 'media/attachment_vault.dart';
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
  required String accountId,
  required Uint8List loginMessage,
});

typedef ChatDeviceBindingSigner = Future<String> Function({
  required int walletIndex,
  required String accountId,
  required Uint8List bindingMessage,
});

typedef ChatCloudTransportFactory = ChatCloudTransport Function({
  required String accountId,
  required String localDeviceId,
  Uri? serviceBaseUrl,
  String? sessionToken,
});

typedef ChatPushTokenProvider = Future<ChatPushToken> Function();

typedef MlsStateStoreFactory = Future<MlsStateStore> Function(
  String ownerCidNumber,
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
    final accountId = await runtime.readAccountId();
    if (accountId == null) return;
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

class _ChatAccountContext {
  const _ChatAccountContext({
    required this.account,
    required this.deviceId,
    required this.devicePublicKey,
    required this.crypto,
    required this.transport,
    required this.webrtc,
    required this.sessionExpiresAt,
  });

  final _ChatAccount account;
  final String deviceId;
  final String devicePublicKey;
  final MlsCrypto crypto;
  final ChatCloudTransport transport;
  final ChatWebrtcTransport webrtc;
  final int sessionExpiresAt;

  bool get isUsable =>
      sessionExpiresAt - ChatRuntime._sessionRefreshSkewMillis >
      DateTime.now().millisecondsSinceEpoch;

  /// 当前绑定失效后主动清零 MLS 状态钥并关闭网络上下文。
  Future<void> dispose() async {
    final currentCrypto = crypto;
    if (currentCrypto is NativeMlsCrypto) {
      currentCrypto.dispose();
    }
    transport.dispose();
    await webrtc.dispose();
  }

  ChatDevice get identity => ChatDevice(
        cidNumber: account.cidNumber,
        deviceId: deviceId,
        devicePublicKey: devicePublicKey,
      );
}

class _ChatAccount {
  const _ChatAccount({
    required this.walletIndex,
    required this.cidNumber,
    required this.bindingRevision,
    required this.accountId,
    required this.walletName,
  });

  final int walletIndex;
  final String cidNumber;
  final int bindingRevision;
  final String accountId;
  final String walletName;
}

/// 公民 Chat 运行态编排服务。
///
/// 页面层不直接操作 OpenMLS、Cloudflare 瞬时转发、近场通道和 Isar。
/// 这个服务负责读取身份账户所在钱包、建立设备身份，并把聊天发送
/// /同步接到正式 transport。已有 P-256 设备子钥和数据用途钥均静默使用；实际登录由
/// Worker 确认 P-256 未登记后才登记，实际数据访问确认数据钥缺失后才生成，两条流程
/// 独立且页面门禁均不参与。
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
    IdentityAccountCache? identityAccountCache,
    Future<Directory> Function()? documentsDirectoryProvider,
  })  : _store = store ?? ChatStore(),
        _walletManager = walletManager ?? WalletManager(),
        _identityAccountCache = identityAccountCache,
        _preferences = preferences,
        _squareApiClient = squareApiClient ?? SquareApiClient(),
        _loginSigner = loginSigner,
        _deviceBindingSigner = deviceBindingSigner,
        _deviceSubkey = deviceSubkey ?? DeviceSubkey(),
        _stateStoreFactory = stateStoreFactory,
        _cryptoFactory = cryptoFactory,
        _cloudTransportFactory = cloudTransportFactory,
        _pushService = pushService ?? ChatPushService(),
        _pushTokenProvider = pushTokenProvider,
        _documentsDirectoryProvider =
            documentsDirectoryProvider ?? getApplicationDocumentsDirectory;

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
  final IdentityAccountCache? _identityAccountCache;

  /// 身份账户单源(CID 绑定账户);chat 自身 accountId 一律取此,walletIndex 保持钱包级。
  IdentityAccountCache get _identityCache =>
      _identityAccountCache ?? IdentityAccountCache.instance;

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
  final Future<Directory> Function() _documentsDirectoryProvider;

  /// 正在经 WebRTC 传输字节的媒体 attachmentId(初始发送或补发中),用于去重:
  /// peer_ready 触发的补发不得对在途媒体再整块重传。
  final Set<String> _mediaBytesInFlight = {};

  /// 同一账户/设备只允许一条初始化链。成功上下文复用到 session 临近过期；
  /// 失败只释放命中的 future，不得误删后来创建的新初始化。
  final Map<String, Future<_ChatAccountContext>> _readyFlights = {};
  final Map<String, _ChatAccountContext> _readyContexts = {};
  final Map<String, String> _accountContextKeys = {};
  final Map<String, int> _accountGenerations = {};

  Future<SharedPreferences> get _prefs async {
    final provided = _preferences;
    if (provided != null) {
      return provided;
    }
    return SharedPreferences.getInstance();
  }

  Future<ChatInboxOverview> readOverview({
    String? cidNumber,
    required int pendingOutgoing,
    required int unreadCount,
  }) async {
    final resolvedCidNumber = cidNumber ?? await readCidNumber();
    return ChatInboxOverview(
      cidNumber: resolvedCidNumber,
      pendingOutgoing: pendingOutgoing,
      unreadCount: unreadCount,
    );
  }

  Future<String?> readAccountId() async {
    return _identityCache.accountId();
  }

  Future<String?> readCidNumber() async {
    return (await _identityCache.resolve())?.snapshot?.cidNumber;
  }

  /// 当前绑定钱包账户派生的附件本地静止态密钥。
  Future<List<int>> _attachmentKey() async {
    final accountId = await readAccountId();
    if (accountId == null || accountId.isEmpty) {
      throw StateError('无身份账户，无法读取附件加密密钥');
    }
    return _walletManager.readDataKeyForCurrentBinding(
      accountId,
      LocalKeyPurpose.attachment,
    );
  }

  /// 短命明文目录：解密出来的附件只落这里，与密文缓存物理分开。
  Future<Directory> _plainDirectory() async {
    final attachmentDirectory = await _attachmentDirectory();
    return Directory(
      '${attachmentDirectory.path}/${AttachmentVault.plainDirName}',
    );
  }

  /// 附件密文缓存以 CID 为属主，并按 finalized 绑定版本与账户隔离加密上下文。
  Future<Directory> _attachmentDirectory() async {
    final account = await _readAccount();
    return _attachmentDirectoryForBinding(
      cidNumber: account.cidNumber,
      bindingRevision: account.bindingRevision,
      accountId: account.accountId,
    );
  }

  Future<Directory> _attachmentDirectoryForBinding({
    required String cidNumber,
    required int bindingRevision,
    required String accountId,
  }) async {
    final bindingDirectory = await _bindingDirectory(
      cidNumber: cidNumber,
      bindingRevision: bindingRevision,
      accountId: accountId,
    );
    return Directory('${bindingDirectory.path}/attachments');
  }

  /// 清空短命明文附件。
  ///
  /// 明文按「**只在前台存活**」管理：App 启动、退到后台、删会话/退出账户三处
  /// 各 purge 一次。不做逐处所有权交接——UI 侧预览/播放/打开/转发路径太多，
  /// 漏一处这份明文就永久留在盘上。
  Future<void> purgePlainAttachments() async {
    await AttachmentVault.purgePlainDirectory(await _plainDirectory());
  }

  /// 在 CID 钱包换绑签名前预演全部 Chat 私有数据交接。
  ///
  /// 聊天正文暂存在 Isar 的目标密文清单；附件逐块重加密；MLS 状态由 Rust 原生
  /// 加密边界重封。三者都保留正式的此前密文，且不会生成任何明文文件。
  Future<void> stageAccountHandover({
    required AccountDataBinding source,
    required AccountDataBinding target,
  }) async {
    _validateHandover(source, target);
    final sourceKeys = await _walletManager.deriveDataKeysForBindingHandover(
      source,
      const <({LocalKeyPurpose purpose, String? context})>[
        (purpose: LocalKeyPurpose.attachment, context: null),
        (purpose: LocalKeyPurpose.mls, context: null),
      ],
    );
    final targetKeys = await _walletManager.deriveDataKeysForBindingHandover(
      target,
      const <({LocalKeyPurpose purpose, String? context})>[
        (purpose: LocalKeyPurpose.attachment, context: null),
        (purpose: LocalKeyPurpose.mls, context: null),
      ],
    );
    try {
      await _store.stageAccountHandover(source: source, target: target);
      await AttachmentVault.stageAccountHandover(
        attachmentDirectory: await _attachmentDirectoryForBinding(
          cidNumber: source.cidNumber,
          bindingRevision: source.bindingRevision,
          accountId: source.accountId,
        ),
        handoverId: _handoverId(target),
        currentKey: sourceKeys[0],
        newKey: targetKeys[0],
      );
      final mlsDirs = await _mlsDeviceDirectories(source);
      final bindings = mlsDirs.isEmpty ? null : MlsNativeBindings.load();
      for (final deviceDir in mlsDirs) {
        bindings!.runStateRekey(
          stateStoreDir: deviceDir.path,
          action: 'stage',
          currentStateKeyHex: _hexKey(sourceKeys[1]),
          newStateKeyHex: _hexKey(targetKeys[1]),
        );
        await MlsStateStore(
          deviceDir,
          ownerCidNumber: source.cidNumber,
          stateKey: Uint8List.fromList(sourceKeys[1]),
        ).stageAccountHandover(Uint8List.fromList(targetKeys[1]));
      }
    } finally {
      for (final key in <Uint8List>[...sourceKeys, ...targetKeys]) {
        key.fillRange(0, key.length, 0);
      }
    }
  }

  /// finalized 后提交全部 Chat 目标密文；每个子步骤均可幂等重试。
  Future<void> commitAccountHandover({
    required AccountDataBinding source,
    required AccountDataBinding target,
  }) async {
    _validateHandover(source, target);
    await _store.commitAccountHandover(source: source, target: target);
    await AttachmentVault.commitAccountHandover(
      attachmentDirectory: await _attachmentDirectoryForBinding(
        cidNumber: source.cidNumber,
        bindingRevision: source.bindingRevision,
        accountId: source.accountId,
      ),
      handoverId: _handoverId(target),
    );
    final mlsDirs = await _mlsDeviceDirectories(source);
    final bindings = mlsDirs.isEmpty ? null : MlsNativeBindings.load();
    for (final deviceDir in mlsDirs) {
      bindings!.runStateRekey(
        stateStoreDir: deviceDir.path,
        action: 'commit',
      );
      await MlsStateStore.commitAccountHandoverFiles(deviceDir);
    }
    await _moveBindingDirectory(source, target);
  }

  Future<void> discardAccountHandover({
    required AccountDataBinding source,
    required AccountDataBinding target,
  }) async {
    _validateHandover(source, target);
    await _store.discardAccountHandover(target);
    await AttachmentVault.discardAccountHandover(
      attachmentDirectory: await _attachmentDirectoryForBinding(
        cidNumber: source.cidNumber,
        bindingRevision: source.bindingRevision,
        accountId: source.accountId,
      ),
      handoverId: _handoverId(target),
    );
    final mlsDirs = await _mlsDeviceDirectories(source);
    final bindings = mlsDirs.isEmpty ? null : MlsNativeBindings.load();
    for (final deviceDir in mlsDirs) {
      bindings!.runStateRekey(
        stateStoreDir: deviceDir.path,
        action: 'discard',
      );
      await MlsStateStore.discardAccountHandoverFiles(deviceDir);
    }
  }

  /// 没有当前账户签名的换绑完成后隔离不可继承的 Chat 状态。
  ///
  /// 绑定分区目录天然让新账户使用全新的附件与 MLS 状态；本方法只清理不能跨 MLS
  /// 上下文续用的本地队列和派生镜像，历史正文密文仍留在 Isar 且对新账户不可见。
  Future<void> isolateInaccessibleBinding(AccountDataBinding previous) =>
      _store.isolateInaccessibleBinding(previous.cidNumber);

  /// finalized 当前绑定完成端内交接后，关闭非当前账户上下文并建立当前 Chat 设备。
  Future<void> convergeFinalizedBinding(AccountDataBinding current) async {
    for (final accountId in _accountContextKeys.keys.toList(growable: false)) {
      if (accountId != current.accountId) {
        await _invalidateAccountContext(accountId);
      }
    }
    await _invalidateAccountContext(current.accountId);
  }

  Future<List<Directory>> _mlsDeviceDirectories(
    AccountDataBinding binding,
  ) async {
    final bindingDirectory = await _bindingDirectory(
      cidNumber: binding.cidNumber,
      bindingRevision: binding.bindingRevision,
      accountId: binding.accountId,
    );
    final mlsRoot = Directory('${bindingDirectory.path}/mls');
    if (!await mlsRoot.exists()) return const <Directory>[];
    final out = <Directory>[];
    await for (final entity in mlsRoot.list()) {
      if (entity is Directory) out.add(entity);
    }
    return out;
  }

  Future<Directory> _bindingDirectory({
    required String cidNumber,
    required int bindingRevision,
    required String accountId,
  }) async {
    final root = await _documentsDirectoryProvider();
    return Directory(
      '${root.path}/chat/by_cid/${_safePath(cidNumber)}/by_binding/'
      '$bindingRevision/${_safePath(accountId)}',
    );
  }

  /// 交接密文全部提交后，原子把文件树切换到新绑定目录；重试时目标已存在即视为完成。
  Future<void> _moveBindingDirectory(
    AccountDataBinding source,
    AccountDataBinding target,
  ) async {
    final sourceDirectory = await _bindingDirectory(
      cidNumber: source.cidNumber,
      bindingRevision: source.bindingRevision,
      accountId: source.accountId,
    );
    final targetDirectory = await _bindingDirectory(
      cidNumber: target.cidNumber,
      bindingRevision: target.bindingRevision,
      accountId: target.accountId,
    );
    if (!await sourceDirectory.exists()) {
      return;
    }
    if (await targetDirectory.exists()) {
      throw StateError('Chat 新绑定目录已存在，禁止覆盖');
    }
    await targetDirectory.parent.create(recursive: true);
    await sourceDirectory.rename(targetDirectory.path);
  }

  static void _validateHandover(
    AccountDataBinding source,
    AccountDataBinding target,
  ) {
    source.validate();
    target.validate();
    if (source.genesisHash != target.genesisHash ||
        source.cidNumber != target.cidNumber ||
        target.bindingRevision != source.bindingRevision + 1 ||
        source.accountId == target.accountId) {
      throw const FormatException('Chat 换绑交接上下文不合法');
    }
  }

  static String _handoverId(AccountDataBinding target) =>
      '${target.bindingRevision}-${target.accountId.substring(2)}';

  static String _hexKey(List<int> key) =>
      key.map((value) => value.toRadixString(16).padLeft(2, '0')).join();

  /// 点击「广场发帖」推送时发信号（转发自设备推送服务），供 AppShell 切到广场 tab。
  Stream<void> get squarePostOpens => _pushService.squarePostOpens;

  /// 页面、轮询、WebSocket 和发送入口共享的唯一就绪入口。
  Future<void> ensureReady(String accountId) async {
    final account = await _readAccount(expectedAccountId: accountId);
    await _readyContext(account);
  }

  /// 首帧后台仅补推送登记，不创建 MLS 状态、不解密聊天数据，也不读取钱包账户 child。
  ///
  /// 尚未由用户进入 Chat 建立过 device_id/device_public_key 时直接跳过；后续进入 Chat
  /// 会在已有设备用途钥时静默初始化；真实缺钥由前台实际业务处理。
  Future<void> prewarmPushRegistrationSilently(String accountId) async {
    final account = await _readAccount(expectedAccountId: accountId);
    final prefs = await _prefs;
    final deviceId = prefs.getString(_kDeviceId) ?? '';
    final devicePublicKey = prefs.getString(_kDevicePublicKeyHex) ?? '';
    if (deviceId.isEmpty || devicePublicKey.isEmpty) return;

    var session = await _squareApiClient.ensureSession(
      accountId: account.accountId,
      signLoginPayload: (payload) => _signSquareLoginPayload(account, payload),
    );
    if (session.cidNumber != account.cidNumber ||
        session.bindingRevision != account.bindingRevision ||
        session.accountId != account.accountId) {
      _squareApiClient.clearSession(account.accountId);
      session = await _squareApiClient.ensureSession(
        accountId: account.accountId,
        signLoginPayload: (payload) =>
            _signSquareLoginPayload(account, payload),
      );
    }
    final identity = ChatDevice(
      cidNumber: account.cidNumber,
      deviceId: deviceId,
      devicePublicKey: devicePublicKey,
    );
    final transport = _cloudTransportFactory?.call(
          accountId: account.accountId,
          localDeviceId: identity.deviceId,
          serviceBaseUrl: _squareApiClient.baseUri,
          sessionToken: session.sessionToken,
        ) ??
        ChatCloudTransport(
          accountId: account.accountId,
          localDeviceId: identity.deviceId,
          serviceBaseUrl: _squareApiClient.baseUri,
          sessionToken: session.sessionToken,
          requestSigner: session.signRequest,
        );
    try {
      await _ensureDeviceRegistered(
        account: account,
        identity: identity,
        prefs: prefs,
        transport: transport,
      );
    } finally {
      transport.dispose();
    }
  }

  /// 默认账户切换或本机 Chat 数据清理时精确失效该账户上下文。
  void invalidateAccount(String accountId) {
    unawaited(_invalidateAccountContext(accountId));
  }

  /// finalized 接管路径必须等此前网络与 MLS 上下文全部关闭后再建立新上下文。
  Future<void> _invalidateAccountContext(String accountId) async {
    _accountGenerations[accountId] = (_accountGenerations[accountId] ?? 0) + 1;
    _readyFlights.removeWhere((key, _) => key.endsWith('|$accountId'));
    final key = _accountContextKeys.remove(accountId);
    if (key != null) {
      final context = _readyContexts.remove(key);
      if (context != null) await context.dispose();
    }
    _squareApiClient.clearSession(accountId);
  }

  static String directConversationId(
    String senderCidNumber,
    String peerCidNumber,
  ) {
    final members = [senderCidNumber, peerCidNumber]..sort();
    return 'dm:${members[0]}:${members[1]}';
  }

  Future<List<ChatDeliveryResult>> sendText({
    required String peerCidNumber,
    required String conversationId,
    required String text,
  }) async {
    final context = await _readyContext(await _readAccount());
    final flow = _messageFlow(context);
    try {
      return await flow.sendText(
        conversationId: conversationId,
        senderCidNumber: context.account.cidNumber,
        recipientCidNumber: peerCidNumber,
        senderDeviceId: context.deviceId,
        text: text,
      );
    } catch (error) {
      if (!_needsFirstKeyPackage(error)) {
        rethrow;
      }
      final packages = await context.transport.fetchKeyPackages(
        targetCidNumber: peerCidNumber,
      );
      if (packages.isEmpty) {
        throw StateError('对方没有可用 Chat KeyPackage');
      }
      final consumed = await context.transport.consumeKeyPackage(
        targetCidNumber: peerCidNumber,
        keyPackageId: packages.first.keyPackageId,
      );
      return flow.sendText(
        conversationId: conversationId,
        senderCidNumber: context.account.cidNumber,
        recipientCidNumber: peerCidNumber,
        senderDeviceId: context.deviceId,
        recipientKeyPackage: consumed,
        text: text,
      );
    }
  }

  Future<List<ChatDeliveryResult>> sendMedia({
    required String peerCidNumber,
    required String conversationId,
    required ChatMediaDraft media,
  }) async {
    final context = await _readyContext(await _readAccount());
    final flow = _messageFlow(context);
    // 登记/清除"待设备投递":对方离线时字节发不出,留 pending 由上线补发。缓存路径
    // 不持久化(补发时按当前 Documents 重算),只存 conversationId/attachmentId/fileName。
    Future<void> recordPending(String attachmentId) =>
        _store.recordOutgoingMedia(
          ownerCidNumber: context.account.cidNumber,
          attachmentId: attachmentId,
          recipientCidNumber: peerCidNumber,
          conversationId: conversationId,
          fileName: media.fileName,
          contentType: media.contentType,
          byteSize: media.byteSize,
        );
    Future<void> markDelivered(String attachmentId) =>
        _store.deleteOutgoingMedia(
          context.account.cidNumber,
          attachmentId,
          peerCidNumber,
        );
    try {
      return await flow.sendMedia(
        conversationId: conversationId,
        senderCidNumber: context.account.cidNumber,
        recipientCidNumber: peerCidNumber,
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
        targetCidNumber: peerCidNumber,
      );
      if (packages.isEmpty) {
        throw StateError('对方没有可用 Chat KeyPackage');
      }
      final consumed = await context.transport.consumeKeyPackage(
        targetCidNumber: peerCidNumber,
        keyPackageId: packages.first.keyPackageId,
      );
      return flow.sendMedia(
        conversationId: conversationId,
        senderCidNumber: context.account.cidNumber,
        recipientCidNumber: peerCidNumber,
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
  ChatAttachmentDeviceSender _guardedDeviceSender(_ChatAccountContext context) {
    return ({
      required recipientCidNumber,
      required conversationId,
      required attachmentId,
      required fileName,
      required contentType,
      required sourcePath,
      required byteSize,
    }) async {
      final inFlightKey =
          MediaResend.inFlightKey(attachmentId, recipientCidNumber);
      _mediaBytesInFlight.add(inFlightKey);
      try {
        await context.webrtc.sendAttachment(
          recipientCidNumber: recipientCidNumber,
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
  ChatRelayUploader _relayUploader(_ChatAccountContext context) {
    return ({
      required conversationId,
      required attachmentId,
      required media,
      int recipientCount = 1,
    }) async {
      return ChatRelayMedia.upload(
        transport: context.transport,
        sourcePath: media.sourcePath,
        byteSize: media.byteSize,
        recipientCount: recipientCount,
        tempDirectory: Directory('${(await _attachmentDirectory()).path}/.tmp'),
      );
    };
  }

  /// 发送内置贴纸:只走控制信封,不经 WebRTC。首次会话缺 KeyPackage 时同样
  /// 领取后重试。
  Future<List<ChatDeliveryResult>> sendSticker({
    required String peerCidNumber,
    required String conversationId,
    required String packId,
    required String stickerId,
  }) async {
    final context = await _readyContext(await _readAccount());
    final flow = _messageFlow(context);
    try {
      return await flow.sendSticker(
        conversationId: conversationId,
        senderCidNumber: context.account.cidNumber,
        recipientCidNumber: peerCidNumber,
        senderDeviceId: context.deviceId,
        packId: packId,
        stickerId: stickerId,
      );
    } catch (error) {
      if (!_needsFirstKeyPackage(error)) {
        rethrow;
      }
      final packages = await context.transport.fetchKeyPackages(
        targetCidNumber: peerCidNumber,
      );
      if (packages.isEmpty) {
        throw StateError('对方没有可用 Chat KeyPackage');
      }
      final consumed = await context.transport.consumeKeyPackage(
        targetCidNumber: peerCidNumber,
        keyPackageId: packages.first.keyPackageId,
      );
      return flow.sendSticker(
        conversationId: conversationId,
        senderCidNumber: context.account.cidNumber,
        recipientCidNumber: peerCidNumber,
        senderDeviceId: context.deviceId,
        recipientKeyPackage: consumed,
        packId: packId,
        stickerId: stickerId,
      );
    }
  }

  // ==== 私密小群 ====

  /// 建群：选联系人 CID，领其 KeyPackage 批量加入，创建者为 admin。
  Future<ChatGroup> createGroup({
    required String name,
    List<String> inviteeCidNumbers = const [],
  }) async {
    final context = await _readyContext(await _readAccount());
    final invitees = await _fetchInviteeKeyPackages(context, inviteeCidNumbers);
    final groupId = newGroupId(context.account.cidNumber);
    return _groupFlow(context).createGroup(
      groupId: groupId,
      name: name,
      cidNumber: context.account.cidNumber,
      localDeviceId: context.deviceId,
      invitees: invitees,
    );
  }

  /// 加人(仅 admin)。
  Future<void> addGroupMembers({
    required String groupId,
    required List<String> inviteeCidNumbers,
  }) async {
    final context = await _readyContext(await _readAccount());
    final invitees = await _fetchInviteeKeyPackages(context, inviteeCidNumbers);
    await _groupFlow(context).addMembers(
      groupId: groupId,
      actorCidNumber: context.account.cidNumber,
      actorDeviceId: context.deviceId,
      invitees: invitees,
    );
  }

  /// 删人（仅 admin，按 CID）。
  Future<void> removeGroupMembers({
    required String groupId,
    required List<String> targetCidNumbers,
  }) async {
    final context = await _readyContext(await _readAccount());
    await _groupFlow(context).removeMembers(
      groupId: groupId,
      actorCidNumber: context.account.cidNumber,
      actorDeviceId: context.deviceId,
      targetCidNumbers: targetCidNumbers,
    );
  }

  /// 退群(本机标记已退,并发退群请求让 admin 重钥)。
  Future<void> leaveGroup(String groupId) async {
    final context = await _readyContext(await _readAccount());
    await _groupFlow(context).leaveGroup(groupId);
  }

  /// 改群名(仅 admin)。
  Future<void> renameGroup({
    required String groupId,
    required String name,
  }) async {
    final context = await _readyContext(await _readAccount());
    await _groupFlow(context).renameGroup(groupId, name);
  }

  /// 群发文本。
  Future<List<ChatDeliveryResult>> sendGroupText({
    required String groupId,
    required String text,
  }) async {
    final context = await _readyContext(await _readAccount());
    return _groupFlow(context).sendGroupText(
      groupId: groupId,
      senderCidNumber: context.account.cidNumber,
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
    final context = await _readyContext(await _readAccount());
    return _groupFlow(context).sendGroupSticker(
      groupId: groupId,
      senderCidNumber: context.account.cidNumber,
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
    final context = await _readyContext(await _readAccount());
    return _groupFlow(context).sendGroupMedia(
      groupId: groupId,
      senderCidNumber: context.account.cidNumber,
      senderDeviceId: context.deviceId,
      media: media,
      sendMemberAttachment: _guardedDeviceSender(context),
      uploadRelayMedia: _relayUploader(context),
      saveLocalAttachment: _copySentAttachmentToCache,
      recordPendingMember: (attachmentId, memberCidNumber) =>
          _store.recordOutgoingMedia(
        ownerCidNumber: context.account.cidNumber,
        attachmentId: attachmentId,
        recipientCidNumber: memberCidNumber,
        conversationId: groupId,
        fileName: media.fileName,
        contentType: media.contentType,
        byteSize: media.byteSize,
      ),
      markMemberDelivered: (attachmentId, memberCidNumber) =>
          _store.deleteOutgoingMedia(
        context.account.cidNumber,
        attachmentId,
        memberCidNumber,
      ),
    );
  }

  /// 逐个被邀请 CID 领取一枚 KeyPackage（复用 1:1 fetch/consume）。
  Future<List<MlsKeyPackage>> _fetchInviteeKeyPackages(
    _ChatAccountContext context,
    List<String> inviteeCidNumbers,
  ) async {
    final packages = <MlsKeyPackage>[];
    for (final cidNumber in inviteeCidNumbers) {
      final available = await context.transport.fetchKeyPackages(
        targetCidNumber: cidNumber,
      );
      if (available.isEmpty) {
        throw StateError('对方 $cidNumber 没有可用 Chat KeyPackage');
      }
      final consumed = await context.transport.consumeKeyPackage(
        targetCidNumber: cidNumber,
        keyPackageId: available.first.keyPackageId,
      );
      if (consumed.cidNumber != cidNumber) {
        throw StateError('Worker 返回的 KeyPackage CID 与请求目标不一致');
      }
      packages.add(consumed);
    }
    return packages;
  }

  ChatGroupFlow _groupFlow(_ChatAccountContext context) {
    return ChatGroupFlow(
      crypto: context.crypto as MlsGroupCrypto,
      store: _store,
      ownerCidNumber: context.account.cidNumber,
      cidNumber: context.account.cidNumber,
      currentAccountId: context.account.accountId,
      localDeviceId: context.deviceId,
      deliverer: (envelope, _, recipientCidNumber) {
        return ChatFlow.deliverWithTransport(
          transport: context.transport,
          envelope: envelope,
          recipientCidNumber: recipientCidNumber,
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
      final cacheDirectory = await _attachmentDirectory();
      final cached = await ChatFlow.readCachedAttachment(
        conversationId: conversationId,
        attachmentId: attachmentId,
        fileName: fileName,
        contentType: contentType,
        clearByteSize: clearByteSize,
        cacheDirectory: cacheDirectory,
        attachmentKey: await _attachmentKey(),
        plainDirectory: await _plainDirectory(),
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
    final cacheDirectory = await _attachmentDirectory();
    final content = ChatPayloadCodec.decode(controlPlaintext);
    if (content.isRelayMedia) {
      return _downloadRelayAttachment(conversationId, content, cacheDirectory);
    }
    return ChatFlow.downloadAttachment(
      conversationId: conversationId,
      controlPlaintext: controlPlaintext,
      cacheDirectory: cacheDirectory,
      attachmentKey: await _attachmentKey(),
      plainDirectory: await _plainDirectory(),
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
      attachmentKey: await _attachmentKey(),
      plainDirectory: await _plainDirectory(),
    );
    if (cached != null) return cached;

    final context = await _readyContext(await _readAccount());
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
    final account = await _readAccount();
    await _store.deleteConversation(account.cidNumber, conversationId);
    final attachmentDir = Directory(
      '${(await _attachmentDirectory()).path}/${_safePath(conversationId)}',
    );
    if (await attachmentDir.exists()) {
      await attachmentDir.delete(recursive: true);
    }
    // purge 点之三:删会话同时清掉可能已解密出来的短命明文。
    await purgePlainAttachments();
  }

  /// 重试发送设备本机队列中的密文,并补发待设备投递的媒体字节。
  /// 密文成功转交在线接收设备后立即删队列项;媒体字节收到 WebRTC ack 后删待投递行。
  Future<int> retryOutgoing({String? recipientCidNumber}) async {
    final context = await _readyContext(await _readAccount());
    final queued = await _store.readQueuedEnvelopes(
      ownerCidNumber: context.account.cidNumber,
      recipientCidNumber: recipientCidNumber,
    );
    var sent = 0;
    for (final item in queued) {
      final result = await context.transport.sendEncryptedEnvelope(
        envelopeId: item.envelopeId,
        envelopeBytes: item.envelopeBytes,
        recipientCidNumber: item.recipientCidNumber,
      );
      await _store.markOutgoingDelivery(
        ownerCidNumber: context.account.cidNumber,
        envelopeId: item.envelopeId,
        state: result.state,
        errorMessage: result.errorMessage,
      );
      if (result.state == ChatMessageDeliveryState.sent) sent += 1;
    }
    // 媒体字节补发**只在明确对端时(peer_ready 确知在线)**触发,且**不阻塞**:
    // 绝不在无差别的轮询/实时启动(recipientCidNumber==null)路径对离线对端反复整块
    // 重连重发(每条阻塞 45 秒、无退避),那会拖垮轮询与后台唤醒窗口。
    if (recipientCidNumber != null) {
      unawaited(
          _resendPendingMedia(context, recipientCidNumber: recipientCidNumber));
    }
    return sent;
  }

  /// 上线补发:遍历待设备投递的媒体,从本机缓存副本重发 WebRTC 字节。核心去重/清孤
  /// 儿/删行逻辑在可测的 [MediaResend.run];缓存路径按**当前 Documents 目录重算**
  /// (不用持久化的绝对路径,避免容器 UUID 变更后误判丢失)。
  Future<void> _resendPendingMedia(
    _ChatAccountContext context, {
    required String recipientCidNumber,
  }) async {
    final pending = await _store.readPendingOutgoingMedia(
      ownerCidNumber: context.account.cidNumber,
      recipientCidNumber: recipientCidNumber,
    );
    if (pending.isEmpty) return;
    final cacheDir = await _attachmentDirectory();
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
        recipientCidNumber: media.recipientCidNumber,
        conversationId: media.conversationId,
        attachmentId: media.attachmentId,
        fileName: media.fileName,
        contentType: media.contentType,
        sourcePath: path,
        byteSize: media.byteSize,
      ),
      deletePending: (media) => _store.deleteOutgoingMedia(
        context.account.cidNumber,
        media.attachmentId,
        media.recipientCidNumber,
      ),
    );
  }

  /// 门③:接收端落盘二次门控。委托给可单测的 [ChatFlow.acceptReceivedMediaToCache]
  /// (超限删临时不入缓存;否则移入缓存)。
  Future<void> _saveReceivedAttachmentToCache({
    required String senderCidNumber,
    required String conversationId,
    required String attachmentId,
    required String fileName,
    required String contentType,
    required String filePath,
    required int byteSize,
  }) async {
    final cacheDirectory = await _attachmentDirectory();
    await ChatFlow.acceptReceivedMediaToCache(
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: contentType,
      tempFilePath: filePath,
      byteSize: byteSize,
      cacheDirectory: cacheDirectory,
      attachmentKey: await _attachmentKey(),
      plainDirectory: await _plainDirectory(),
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
    final cacheDirectory = await _attachmentDirectory();
    await ChatFlow.importAttachmentFileToCache(
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: contentType,
      sourcePath: sourcePath,
      byteSize: byteSize,
      moveSource: false,
      cacheDirectory: cacheDirectory,
      attachmentKey: await _attachmentKey(),
      plainDirectory: await _plainDirectory(),
    );
  }

  Future<Future<void> Function()?> startRealtimeSync({
    required Future<void> Function() onNotice,
    Future<void> Function()? onDisconnected,
  }) async {
    final context = await _readyContext(await _readAccount());
    final stopSocket = await context.transport.connectRealtime(
      onMessage: (message) async {
        final type = message['type'];
        if (type == 'citizen_chat_envelope') {
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
        if (type == 'citizen_chat_signal') {
          // Worker 按身份主键投递信令：发件人以 CID 号标识（与推送/路由同口径）。
          final senderCidNumber = message['sender_cid_number'];
          final signal = message['signal'];
          if (senderCidNumber is! String || signal is! Map<String, dynamic>) {
            return;
          }
          if (signal['kind'] == 'peer_ready') {
            await retryOutgoing(recipientCidNumber: senderCidNumber);
          } else {
            await context.webrtc.handleSignal(senderCidNumber, signal);
          }
        }
      },
      onDisconnected: onDisconnected,
    );
    if (stopSocket == null) return null;

    Future<void> notifySenderReady(String senderCidNumber) async {
      if (senderCidNumber.isEmpty) return;
      await context.transport.sendSignal(
        recipientCidNumber: senderCidNumber,
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

  Future<void> _refreshPushRegistration(_ChatAccountContext context) async {
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

  Future<_ChatAccountContext> _readyContext(_ChatAccount account) {
    final knownKey = _accountContextKeys[account.accountId];
    final cached = knownKey == null ? null : _readyContexts[knownKey];
    if (cached != null && cached.isUsable) {
      return Future.value(cached);
    }
    if (knownKey != null) {
      final context = _readyContexts.remove(knownKey);
      if (context != null) unawaited(context.dispose());
    }

    final flightKey =
        '${account.cidNumber}|${account.bindingRevision}|${account.accountId}';
    final existing = _readyFlights[flightKey];
    if (existing != null) {
      return existing;
    }

    final generation = _accountGenerations[account.accountId] ?? 0;
    late final Future<_ChatAccountContext> created;
    created = _buildAccountContext(account).then((context) async {
      if ((_accountGenerations[account.accountId] ?? 0) != generation) {
        await context.dispose();
        throw StateError('CID 当前绑定已切换，本次旧初始化结果已丢弃');
      }
      final contextKey = _contextKey(context.account, context.identity);
      final previousKey = _accountContextKeys[account.accountId];
      if (previousKey != null && previousKey != contextKey) {
        final previous = _readyContexts.remove(previousKey);
        if (previous != null) await previous.dispose();
      }
      _accountContextKeys[account.accountId] = contextKey;
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

  Future<_ChatAccountContext> _buildAccountContext(_ChatAccount account) async {
    final prefs = await _prefs;
    var deviceId = prefs.getString(_kDeviceId);
    if (deviceId == null || deviceId.isEmpty) {
      deviceId = 'chat-${_newNonce()}';
      await prefs.setString(_kDeviceId, deviceId);
    }

    var devicePublicKey = prefs.getString(_kDevicePublicKeyHex) ?? '';
    final stateStore = await _stateStore(
      account.cidNumber,
      account.bindingRevision,
      account.accountId,
      deviceId,
    );
    ChatCloudTransport? transport;
    var keepStateStore = false;
    try {
      var identity = ChatDevice(
        cidNumber: account.cidNumber,
        deviceId: deviceId,
        devicePublicKey: devicePublicKey.isEmpty ? '00' : devicePublicKey,
      );
      final crypto = _cryptoFactory?.call(identity, stateStore) ??
          NativeMlsCrypto(identity: identity, stateStore: stateStore);
      MlsKeyPackage? freshKeyPackage;
      if (devicePublicKey.isEmpty) {
        freshKeyPackage = await crypto.createKeyPackage(identity);
        final keyPackage = freshKeyPackage;
        if (keyPackage.devicePublicKey.isEmpty) {
          throw StateError('OpenMLS native 未返回 Chat 设备公钥，请先重编 native 库');
        }
        devicePublicKey = keyPackage.devicePublicKey;
        await prefs.setString(_kDevicePublicKeyHex, devicePublicKey);
        identity = ChatDevice(
          cidNumber: account.cidNumber,
          deviceId: deviceId,
          devicePublicKey: devicePublicKey,
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
      transport = service.transport;
      final tempDirectory = '${(await _attachmentDirectory()).path}/.tmp';
      // 回收被永久放弃的续传残档(对端删会话/待投递后不会再续写的 .part)。
      unawaited(ChatAttachmentReceiveBuffer.sweepStalePartials(tempDirectory));
      final webrtc = ChatWebrtcTransport(
        accountId: account.accountId,
        cloud: transport,
        tempDirectory: tempDirectory,
        onAttachment: _saveReceivedAttachmentToCache,
      );
      keepStateStore = true;
      return _ChatAccountContext(
        account: account,
        deviceId: deviceId,
        devicePublicKey: identity.devicePublicKey,
        crypto: finalCrypto,
        transport: transport,
        webrtc: webrtc,
        sessionExpiresAt: service.session.expiresAt,
      );
    } finally {
      if (!keepStateStore) {
        transport?.dispose();
        stateStore.dispose();
      }
    }
  }

  Future<_ChatServiceContext> _ensureServiceReady({
    required _ChatAccount account,
    required ChatDevice identity,
    required MlsCrypto crypto,
    required SharedPreferences prefs,
    MlsKeyPackage? initialKeyPackage,
  }) async {
    // 这是用户实际进入 Chat 后的会话需求：已有 P-256 子钥静默登录；Worker 明确报告
    // device_not_registered 时才鉴权一次生成并登记。后台推送预热仍不传该回调。
    var session = await _squareApiClient.ensureSession(
      accountId: account.accountId,
      signLoginPayload: (payload) => _signSquareLoginPayload(account, payload),
      onDeviceNotRegistered: () => _registerMissingDeviceSubkey(account),
    );
    if (session.cidNumber != account.cidNumber ||
        session.bindingRevision != account.bindingRevision ||
        session.accountId != account.accountId) {
      _squareApiClient.clearSession(account.accountId);
      session = await _squareApiClient.ensureSession(
        accountId: account.accountId,
        signLoginPayload: (payload) =>
            _signSquareLoginPayload(account, payload),
        onDeviceNotRegistered: () => _registerMissingDeviceSubkey(account),
      );
    }
    if (session.cidNumber != account.cidNumber ||
        session.bindingRevision != account.bindingRevision ||
        session.accountId != account.accountId) {
      throw StateError('聊天会话与 finalized 当前 CID 绑定不一致');
    }
    final transport = _cloudTransportFactory?.call(
          accountId: account.accountId,
          localDeviceId: identity.deviceId,
          serviceBaseUrl: _squareApiClient.baseUri,
          sessionToken: session.sessionToken,
        ) ??
        ChatCloudTransport(
          accountId: account.accountId,
          localDeviceId: identity.deviceId,
          serviceBaseUrl: _squareApiClient.baseUri,
          sessionToken: session.sessionToken,
          requestSigner: session.signRequest,
        );
    try {
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
        transport: transport,
      );
    } catch (_) {
      transport.dispose();
      rethrow;
    }
  }

  Future<void> _ensureDeviceRegistered({
    required _ChatAccount account,
    required ChatDevice identity,
    required SharedPreferences prefs,
    required ChatCloudTransport transport,
  }) async {
    final cacheKey = _deviceBindingCacheKey(account, identity);
    final cachedExpiresAt = prefs.getInt(cacheKey) ?? 0;
    final now = DateTime.now().millisecondsSinceEpoch;
    final pushToken = await _readPushToken();
    final pushCacheKey = _pushTokenCacheKey(account, identity);
    if (cachedExpiresAt - _keyPackageRefreshSkewMillis > now &&
        prefs.getString(pushCacheKey) == pushToken.token) {
      return;
    }

    final expiresAt = DateTime.now().toUtc().add(_deviceBindingTtl);
    final binding = ChatDeviceBinding(
      cidNumber: account.cidNumber,
      bindingRevision: account.bindingRevision,
      accountId: account.accountId,
      deviceId: identity.deviceId,
      devicePublicKey: identity.devicePublicKey,
      expiresAt: expiresAt,
      nonce: _newNonce(),
    );
    final signatureHex = await _signDeviceBinding(
      account: account,
      bindingMessage: binding.signingMessage(),
    );
    await transport.registerDevice(
      devicePublicKey: identity.devicePublicKey,
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
    final account = await _readAccount();
    if (identity.cidNumber != account.cidNumber) {
      throw StateError('Chat 设备 CID 与 finalized 当前身份不一致');
    }
    final cacheKey = _keyPackageCacheKey(account, identity);
    final cachedUntil = prefs.getInt(cacheKey) ?? 0;
    final now = DateTime.now().millisecondsSinceEpoch;
    if (cachedUntil - _keyPackageRefreshSkewMillis > now) {
      return;
    }

    final keyPackage =
        initialKeyPackage ?? await crypto.createKeyPackage(identity);
    if (keyPackage.devicePublicKey.isNotEmpty &&
        keyPackage.devicePublicKey.toLowerCase() !=
            identity.devicePublicKey.toLowerCase()) {
      throw StateError('OpenMLS native 返回的 Chat 设备公钥与本机身份不一致');
    }
    await transport.publishKeyPackage(
      keyPackage,
      cidNumber: await _readSelfCidNumber(),
    );
    await prefs.setInt(cacheKey, keyPackage.expiresAtMillis);
  }

  /// 本身份主键 CID 号(发布本机 KeyPackage 时按 CID 归档)。
  ///
  /// 取自身份账户单源 [IdentityAccountCache] 的链上闭环快照;未绑定 CID(纯访客/
  /// 未注册)则抛错——无 CID 不能参与聊天路由,fail-closed。
  Future<String> _readSelfCidNumber() async {
    final resolved = await _identityCache.resolve();
    final cidNumber = resolved?.snapshot?.cidNumber ?? '';
    if (cidNumber.isEmpty) {
      throw StateError('当前身份尚未绑定 CID，无法发布聊天密钥');
    }
    return cidNumber;
  }

  Future<String> _signSquareLoginPayload(
    _ChatAccount account,
    Uint8List loginMessage,
  ) async {
    final signer = _loginSigner;
    if (signer != null) {
      return signer(
        walletIndex: account.walletIndex,
        accountId: account.accountId,
        loginMessage: loginMessage,
      );
    }
    // 会话握手 = 非用户动权 → P-256 硬件子钥静默签名 signing_message 摘要（不读 seed、不弹生物识别）。
    final raw =
        await _deviceSubkey.signRawHex(account.walletIndex, loginMessage);
    return '0x$raw';
  }

  Future<String> _signDeviceBinding({
    required _ChatAccount account,
    required Uint8List bindingMessage,
  }) async {
    final signer = _deviceBindingSigner;
    if (signer != null) {
      return signer(
        walletIndex: account.walletIndex,
        accountId: account.accountId,
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

  Future<void> _registerMissingDeviceSubkey(_ChatAccount account) async {
    final binding =
        await _walletManager.accountDataBindingForAccountId(account.accountId);
    await _walletManager.registerDeviceSubkeyForBinding(binding);
  }

  Future<_ChatAccount> _readAccount({String? expectedAccountId}) async {
    // CID 是永久身份主键；当前绑定账户负责签名、鉴权与派生本版本私有数据密钥。
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) {
      throw StateError('请先在「我的 → 我的钱包」创建热钱包');
    }
    if (!wallet.isHotWallet) {
      throw StateError('身份账户必须是热钱包');
    }
    final binding = await _identityCache.binding();
    final cidNumber = binding?.cidNumber ?? '';
    final bindingRevision = binding?.bindingRevision ?? 0;
    final accountId = binding?.accountId ?? '';
    if (cidNumber.isEmpty || bindingRevision <= 0 || accountId.isEmpty) {
      throw StateError('当前钱包尚未注册 CID，无法使用聊天');
    }
    if (expectedAccountId != null && accountId != expectedAccountId) {
      throw StateError('身份账户已切换，请重新进入聊天');
    }
    await _walletManager.activateAccountDataBinding(
      genesisHash: binding!.genesisHash,
      cidNumber: cidNumber,
      bindingRevision: bindingRevision,
      accountId: accountId,
    );
    return _ChatAccount(
      walletIndex: wallet.walletIndex,
      cidNumber: cidNumber,
      bindingRevision: bindingRevision,
      accountId: accountId,
      walletName: wallet.walletName,
    );
  }

  ChatFlow _messageFlow(_ChatAccountContext context) {
    return ChatFlow(
      crypto: context.crypto,
      store: _store,
      ownerCidNumber: context.account.cidNumber,
      currentAccountId: context.account.accountId,
      deliverer: (envelope, _, recipientCidNumber) {
        return ChatFlow.deliverWithTransport(
          transport: context.transport,
          envelope: envelope,
          recipientCidNumber: recipientCidNumber,
        );
      },
    );
  }

  Future<MlsStateStore> _stateStore(
    String ownerCidNumber,
    int bindingRevision,
    String accountId,
    String deviceId,
  ) async {
    final factory = _stateStoreFactory;
    if (factory != null) {
      return factory(ownerCidNumber, deviceId);
    }
    final bindingDirectory = await _bindingDirectory(
      cidNumber: ownerCidNumber,
      bindingRevision: bindingRevision,
      accountId: accountId,
    );
    final safeDevice = _safePath(deviceId);
    // MLS 状态（设备签名私钥 + 群 ratchet 秘密）落盘必须加密：已有 mls 用途钥从
    // 设备数据钥金库静默解封，真实缺钥时才鉴权一次生成。
    return MlsStateStore(
      Directory('${bindingDirectory.path}/mls/$safeDevice'),
      ownerCidNumber: ownerCidNumber,
      stateKey: await _walletManager.readDataKeyForCurrentBinding(
        accountId,
        LocalKeyPurpose.mls,
      ),
    );
  }
}

class _ChatServiceContext {
  const _ChatServiceContext({
    required this.baseUri,
    required this.session,
    required this.transport,
  });

  final Uri baseUri;
  final SquareSession session;
  final ChatCloudTransport transport;
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

String _contextKey(_ChatAccount account, ChatDevice identity) {
  return '${account.cidNumber}|${account.bindingRevision}|${account.accountId}|'
      '${identity.deviceId}|'
      '${identity.devicePublicKey.toLowerCase()}';
}

String _deviceBindingCacheKey(_ChatAccount account, ChatDevice identity) {
  return '${ChatRuntime._kDeviceBindingPrefix}.'
      '${_safePath(account.cidNumber)}.${account.bindingRevision}.'
      '${_safePath(account.accountId)}.'
      '${_safePath(identity.deviceId)}.${identity.devicePublicKey}';
}

String _keyPackageCacheKey(_ChatAccount account, ChatDevice identity) {
  return '${ChatRuntime._kKeyPackagePublishedPrefix}.'
      '${_safePath(account.cidNumber)}.${account.bindingRevision}.'
      '${_safePath(account.accountId)}.'
      '${_safePath(identity.deviceId)}.${identity.devicePublicKey}';
}

String _pushTokenCacheKey(_ChatAccount account, ChatDevice identity) {
  return '${ChatRuntime._kPushTokenPrefix}.'
      '${_safePath(account.cidNumber)}.${account.bindingRevision}.'
      '${_safePath(account.accountId)}.${_safePath(identity.deviceId)}';
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
