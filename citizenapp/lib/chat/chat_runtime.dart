import 'dart:io';
import 'dart:math';
import 'dart:typed_data';

import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../8964/services/square_api_client.dart';
import '../wallet/core/device_subkey.dart';
import '../wallet/core/wallet_manager.dart';
import 'crypto/chat_device_binding.dart';
import 'crypto/mls_boundary.dart';
import 'crypto/mls_native.dart';
import 'crypto/mls_state_store.dart';
import 'chat_flow.dart';
import 'chat_models.dart';
import 'storage/chat_store.dart';
import 'transport/chat_cloud_transport.dart';
import 'transport/chat_transport.dart';

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
  Uri? mailboxBaseUrl,
  String? sessionToken,
});

typedef MlsStateStoreFactory = Future<MlsStateStore> Function(
  String ownerAccount,
  String deviceId,
);

class _ChatOwnerContext {
  const _ChatOwnerContext({
    required this.account,
    required this.deviceId,
    required this.devicePublicKeyHex,
    required this.crypto,
    required this.transport,
    required this.sessionExpiresAt,
  });

  final _ChatOwner account;
  final String deviceId;
  final String devicePublicKeyHex;
  final MlsCrypto crypto;
  final ChatCloudTransport transport;
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
/// 页面层不直接操作 OpenMLS、Cloudflare mailbox、近场通道和 Isar。
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
  })  : _store = store ?? ChatStore(),
        _walletManager = walletManager ?? WalletManager(),
        _preferences = preferences,
        _squareApiClient = squareApiClient ?? SquareApiClient(),
        _loginSigner = loginSigner,
        _deviceBindingSigner = deviceBindingSigner,
        _deviceSubkey = deviceSubkey ?? DeviceSubkey(),
        _stateStoreFactory = stateStoreFactory,
        _cryptoFactory = cryptoFactory,
        _cloudTransportFactory = cloudTransportFactory;

  static const _kDeviceId = 'chat.device.id';
  static const _kDevicePublicKeyHex = 'chat.device.public_key_hex';
  static const _kDeviceBindingPrefix = 'chat.cloudflare.device_binding';
  static const _kKeyPackagePublishedPrefix =
      'chat.cloudflare.key_package_until';
  static const _mailboxBindingTtl = Duration(days: 90);
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
      mailboxStatus: ChatMailboxStatus.unavailable,
      ownerAccount: account,
      mailboxEndpoint: null,
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

  Future<List<ChatDeliveryResult>> sendAttachment({
    required String peerAccount,
    required String conversationId,
    required ChatAttachmentDraft attachment,
  }) async {
    final context = await _readyContext(await _readOwner());
    final flow = _messageFlow(context);
    try {
      return await flow.sendAttachment(
        conversationId: conversationId,
        senderAccount: context.account.address,
        recipientAccount: peerAccount,
        senderDeviceId: context.deviceId,
        attachment: attachment,
        prepareAttachmentUpload: context.transport.prepareAttachmentUpload,
        uploadAttachmentObject: context.transport.uploadAttachmentObject,
        completeAttachmentUpload: context.transport.completeAttachmentUpload,
        saveLocalAttachment: _saveAttachmentBytesToCache,
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
      return flow.sendAttachment(
        conversationId: conversationId,
        senderAccount: context.account.address,
        recipientAccount: peerAccount,
        senderDeviceId: context.deviceId,
        recipientKeyPackage: consumed,
        attachment: attachment,
        prepareAttachmentUpload: context.transport.prepareAttachmentUpload,
        uploadAttachmentObject: context.transport.uploadAttachmentObject,
        completeAttachmentUpload: context.transport.completeAttachmentUpload,
        saveLocalAttachment: _saveAttachmentBytesToCache,
      );
    }
  }

  Future<ChatDownloadedAttachment> downloadAttachment({
    required String conversationId,
    required String controlPlaintext,
  }) async {
    final context = await _readyContext(await _readOwner());
    final dir = await getApplicationDocumentsDirectory();
    return ChatFlow.downloadAttachment(
      conversationId: conversationId,
      controlPlaintext: controlPlaintext,
      cacheDirectory: Directory('${dir.path}/chat/attachments'),
      prepareAttachmentDownload: context.transport.prepareAttachmentDownload,
      downloadAttachmentObject: context.transport.downloadAttachmentObject,
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

  Future<int> syncPending() async {
    final context = await _readyContext(await _readOwner());
    final flow = ChatFlow(
      crypto: context.crypto,
      store: _store,
      deliverer: (envelope, _) async => ChatDeliveryResult(
        envelopeId: envelope.envelopeId,
        transportType: ChatTransportType.cloudflare,
        state: ChatMessageDeliveryState.failed,
        errorMessage: '入站同步不执行投递',
      ),
    );
    return flow.fetchAndProcessPending(
      fetchPending: context.transport.fetchPending,
      ackEnvelope: context.transport.ackEnvelope,
      cacheIncomingAttachment: (conversationId, controlPlaintext) =>
          downloadAttachment(
        conversationId: conversationId,
        controlPlaintext: controlPlaintext,
      ),
    );
  }

  Future<void> _saveAttachmentBytesToCache({
    required String conversationId,
    required String attachmentId,
    required String fileName,
    required String contentType,
    required List<int> bytes,
  }) async {
    final dir = await getApplicationDocumentsDirectory();
    await ChatFlow.saveAttachmentBytesToCache(
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: contentType,
      bytes: bytes,
      cacheDirectory: Directory('${dir.path}/chat/attachments'),
    );
  }

  Future<Future<void> Function()?> startRealtimeSync({
    required Future<void> Function() onNotice,
    Future<void> Function()? onDisconnected,
  }) async {
    final context = await _readyContext(await _readOwner());
    return context.transport.connectRealtime(
      onNotification: (message) async {
        // WebSocket 只作为新密文提醒；正式拉取、解密、ack 仍走 syncPending。
        if (message['type'] == 'gmb_chat_new_envelope_v1') {
          await onNotice();
        }
      },
      onDisconnected: onDisconnected,
    );
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
    final mailbox = await _ensureMailboxReady(
      account: account,
      identity: identity,
      crypto: finalCrypto,
      prefs: prefs,
      initialKeyPackage: freshKeyPackage,
    );
    final transport = _cloudTransportFactory?.call(
          ownerAccount: account.address,
          ownerDeviceId: deviceId,
          mailboxBaseUrl: mailbox.baseUri,
          sessionToken: mailbox.session.sessionToken,
        ) ??
        ChatCloudTransport(
          ownerAccount: account.address,
          ownerDeviceId: deviceId,
          mailboxBaseUrl: mailbox.baseUri,
          sessionToken: mailbox.session.sessionToken,
        );
    return _ChatOwnerContext(
      account: account,
      deviceId: deviceId,
      devicePublicKeyHex: identity.devicePublicKeyHex,
      crypto: finalCrypto,
      transport: transport,
      sessionExpiresAt: mailbox.session.expiresAt,
    );
  }

  Future<_ChatMailboxContext> _ensureMailboxReady({
    required _ChatOwner account,
    required ChatDevice identity,
    required MlsCrypto crypto,
    required SharedPreferences prefs,
    MlsKeyPackage? initialKeyPackage,
  }) async {
    // 后台会话握手绝不读 seed / 不弹窗 / 不懒注册：子钥只在钱包创建时静默注册。
    final session = await _squareApiClient.ensureSession(
      ownerAccount: account.address,
      signLoginPayload: (payload) => _signSquareLoginPayload(account, payload),
    );
    final transport = _cloudTransportFactory?.call(
          ownerAccount: account.address,
          ownerDeviceId: identity.deviceId,
          mailboxBaseUrl: _squareApiClient.baseUri,
          sessionToken: session.sessionToken,
        ) ??
        ChatCloudTransport(
          ownerAccount: account.address,
          ownerDeviceId: identity.deviceId,
          mailboxBaseUrl: _squareApiClient.baseUri,
          sessionToken: session.sessionToken,
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
    return _ChatMailboxContext(
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
    if (cachedExpiresAt - _keyPackageRefreshSkewMillis > now) {
      return;
    }

    final expiresAt = DateTime.now().toUtc().add(_mailboxBindingTtl);
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
      bindingSignature: signatureHex,
      expiresAtMillis: expiresAt.millisecondsSinceEpoch,
      nonce: binding.nonce,
    );
    await prefs.setInt(cacheKey, expiresAt.millisecondsSinceEpoch);
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

class _ChatMailboxContext {
  const _ChatMailboxContext({
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
