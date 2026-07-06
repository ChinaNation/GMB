import 'dart:convert';
import 'dart:io';
import 'dart:math';
import 'dart:typed_data';

import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../8964/services/square_api_client.dart';
import '../signer/signing.dart';
import '../wallet/core/wallet_manager.dart';
import 'crypto/im_identity_binding.dart';
import 'crypto/im_mls_boundary.dart';
import 'crypto/im_mls_native.dart';
import 'crypto/im_mls_state_store.dart';
import 'im_message_flow.dart';
import 'im_session_models.dart';
import 'storage/im_isar_store.dart';
import 'transport/im_cloudflare_transport.dart';
import 'transport/im_transport.dart';

typedef ImSquareLoginPayloadSigner = Future<String> Function({
  required int walletIndex,
  required String ownerAccount,
  required String signingPayload,
});

typedef ImWalletPayloadSigner = Future<String> Function({
  required int walletIndex,
  required String ownerAccount,
  required Uint8List payload,
});

typedef ImCloudflareTransportFactory = ImCloudflareTransport Function({
  required String ownerChatAccount,
  required String ownerDeviceId,
  Uri? mailboxBaseUrl,
  String? sessionToken,
});

typedef ImMlsStateStoreFactory = Future<ImMlsStateStore> Function(
  String walletAccount,
  String deviceId,
);

class _ImOwnerContext {
  const _ImOwnerContext({
    required this.account,
    required this.deviceId,
    required this.devicePublicKeyHex,
    required this.crypto,
    required this.transport,
  });

  final _ImCommunicationAccount account;
  final String deviceId;
  final String devicePublicKeyHex;
  final ImMlsCryptoBoundary crypto;
  final ImCloudflareTransport transport;

  ImMlsDeviceIdentity get identity => ImMlsDeviceIdentity(
        walletChatAccount: account.address,
        deviceId: deviceId,
        devicePublicKeyHex: devicePublicKeyHex,
      );
}

class _ImCommunicationAccount {
  const _ImCommunicationAccount({
    required this.walletIndex,
    required this.address,
    required this.walletName,
  });

  final int walletIndex;
  final String address;
  final String walletName;
}

/// 公民 IM 运行态编排服务。
///
/// 页面层不直接操作 OpenMLS、Cloudflare mailbox、近场通道和 Isar。
/// 这个服务负责读取用户资料中的通信账户、建立设备身份，并把聊天发送
/// /同步接到正式 transport。钱包私钥只用于设备绑定证明，不参与消息加密。
class ImRuntime {
  ImRuntime({
    ImIsarStore? store,
    WalletManager? walletManager,
    SharedPreferences? preferences,
    SquareApiClient? squareApiClient,
    ImSquareLoginPayloadSigner? squareLoginPayloadSigner,
    ImWalletPayloadSigner? walletPayloadSigner,
    ImMlsStateStoreFactory? stateStoreFactory,
    ImMlsCryptoBoundary Function(
      ImMlsDeviceIdentity identity,
      ImMlsStateStore stateStore,
    )? cryptoFactory,
    ImCloudflareTransportFactory? cloudflareTransportFactory,
  })  : _store = store ?? ImIsarStore(),
        _walletManager = walletManager ?? WalletManager(),
        _preferences = preferences,
        _squareApiClient = squareApiClient ?? SquareApiClient(),
        _squareLoginPayloadSigner = squareLoginPayloadSigner,
        _walletPayloadSigner = walletPayloadSigner,
        _stateStoreFactory = stateStoreFactory,
        _cryptoFactory = cryptoFactory,
        _cloudflareTransportFactory = cloudflareTransportFactory;

  static const _kDeviceId = 'im.device.id';
  static const _kDevicePublicKeyHex = 'im.device.public_key_hex';
  static const _kDeviceBindingPrefix = 'im.cloudflare.device_binding';
  static const _kKeyPackagePublishedPrefix = 'im.cloudflare.key_package_until';
  static const _mailboxBindingTtl = Duration(days: 90);
  static const _keyPackageRefreshSkewMillis = 24 * 60 * 60 * 1000;

  final ImIsarStore _store;
  final WalletManager _walletManager;
  final SharedPreferences? _preferences;
  final SquareApiClient _squareApiClient;
  final ImSquareLoginPayloadSigner? _squareLoginPayloadSigner;
  final ImWalletPayloadSigner? _walletPayloadSigner;
  final ImMlsStateStoreFactory? _stateStoreFactory;
  final ImMlsCryptoBoundary Function(
    ImMlsDeviceIdentity identity,
    ImMlsStateStore stateStore,
  )? _cryptoFactory;
  final ImCloudflareTransportFactory? _cloudflareTransportFactory;
  final Set<int> _authenticatedWalletIndexes = <int>{};

  Future<SharedPreferences> get _prefs async {
    final provided = _preferences;
    if (provided != null) {
      return provided;
    }
    return SharedPreferences.getInstance();
  }

  Future<ImInboxOverview> readOverview({
    String? boundWalletAddress,
    required int pendingOutgoing,
    required int unreadCount,
  }) async {
    final account = boundWalletAddress ?? await readCommunicationAddress();
    return ImInboxOverview(
      mailboxStatus: ImMailboxStatus.unavailable,
      boundWalletAddress: account,
      mailboxEndpoint: null,
      pendingOutgoing: pendingOutgoing,
      unreadCount: unreadCount,
    );
  }

  Future<String?> readCommunicationAddress() async {
    final wallet = await _walletManager.getDefaultWallet();
    return wallet?.address;
  }

  static String directConversationId(
    String senderWalletAddress,
    String peerWalletAddress,
  ) {
    return 'dm:$senderWalletAddress:$peerWalletAddress';
  }

  Future<List<ImDeliveryResult>> sendText({
    required String peerWalletAddress,
    required String conversationId,
    required String text,
  }) async {
    await _readCommunicationAccount();
    final context = await _ensureOwnerContext(prepareMailbox: true);
    final flow = _messageFlow(context);
    try {
      return await flow.sendText(
        conversationId: conversationId,
        senderChatAccount: context.account.address,
        recipientChatAccount: peerWalletAddress,
        senderDeviceId: context.deviceId,
        text: text,
      );
    } catch (error) {
      if (!_needsFirstKeyPackage(error)) {
        rethrow;
      }
      final packages = await context.transport.fetchKeyPackages(
        ownerChatAccount: peerWalletAddress,
        requesterChatAccount: context.account.address,
      );
      if (packages.isEmpty) {
        throw StateError('对方没有可用 IM KeyPackage');
      }
      final consumed = await context.transport.consumeKeyPackage(
        ownerChatAccount: peerWalletAddress,
        keyPackageId: packages.first.keyPackageId,
        requesterChatAccount: context.account.address,
      );
      return flow.sendText(
        conversationId: conversationId,
        senderChatAccount: context.account.address,
        recipientChatAccount: peerWalletAddress,
        senderDeviceId: context.deviceId,
        recipientKeyPackage: consumed,
        text: text,
      );
    }
  }

  Future<List<ImDeliveryResult>> sendAttachment({
    required String peerWalletAddress,
    required String conversationId,
    required ImAttachmentDraft attachment,
  }) async {
    await _readCommunicationAccount();
    final context = await _ensureOwnerContext(prepareMailbox: true);
    final flow = _messageFlow(context);
    try {
      return await flow.sendAttachment(
        conversationId: conversationId,
        senderChatAccount: context.account.address,
        recipientChatAccount: peerWalletAddress,
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
        ownerChatAccount: peerWalletAddress,
        requesterChatAccount: context.account.address,
      );
      if (packages.isEmpty) {
        throw StateError('对方没有可用 IM KeyPackage');
      }
      final consumed = await context.transport.consumeKeyPackage(
        ownerChatAccount: peerWalletAddress,
        keyPackageId: packages.first.keyPackageId,
        requesterChatAccount: context.account.address,
      );
      return flow.sendAttachment(
        conversationId: conversationId,
        senderChatAccount: context.account.address,
        recipientChatAccount: peerWalletAddress,
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

  Future<ImDownloadedAttachment> downloadAttachment({
    required String conversationId,
    required String controlPlaintext,
  }) async {
    final context = await _ensureOwnerContext(prepareMailbox: true);
    final dir = await getApplicationDocumentsDirectory();
    return ImMessageFlow.downloadAttachment(
      conversationId: conversationId,
      controlPlaintext: controlPlaintext,
      cacheDirectory: Directory('${dir.path}/im/attachments'),
      prepareAttachmentDownload: context.transport.prepareAttachmentDownload,
      downloadAttachmentObject: context.transport.downloadAttachmentObject,
    );
  }

  Future<void> deleteLocalConversation(String conversationId) async {
    await _store.deleteConversation(conversationId);
    final dir = await getApplicationDocumentsDirectory();
    final attachmentDir = Directory(
      '${dir.path}/im/attachments/${_safePath(conversationId)}',
    );
    if (await attachmentDir.exists()) {
      await attachmentDir.delete(recursive: true);
    }
  }

  Future<int> syncPending() async {
    final context = await _ensureOwnerContext(prepareMailbox: true);
    final flow = ImMessageFlow(
      crypto: context.crypto,
      store: _store,
      deliverer: (envelope, _) async => ImDeliveryResult(
        envelopeId: envelope.envelopeId,
        transportType: ImTransportType.cloudflare,
        state: ImMessageDeliveryState.failed,
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
    await ImMessageFlow.saveAttachmentBytesToCache(
      conversationId: conversationId,
      attachmentId: attachmentId,
      fileName: fileName,
      contentType: contentType,
      bytes: bytes,
      cacheDirectory: Directory('${dir.path}/im/attachments'),
    );
  }

  Future<Future<void> Function()?> startRealtimeSync({
    required Future<void> Function() onNotice,
    Future<void> Function()? onDisconnected,
  }) async {
    final context = await _ensureOwnerContext(prepareMailbox: true);
    return context.transport.connectRealtime(
      onNotification: (message) async {
        // WebSocket 只作为新密文提醒；正式拉取、解密、ack 仍走 syncPending。
        if (message['type'] == 'gmb_im_new_envelope_v1') {
          await onNotice();
        }
      },
      onDisconnected: onDisconnected,
    );
  }

  Future<_ImOwnerContext> _ensureOwnerContext({
    required bool prepareMailbox,
  }) async {
    final account = await _readCommunicationAccount();
    final prefs = await _prefs;
    var deviceId = prefs.getString(_kDeviceId);
    if (deviceId == null || deviceId.isEmpty) {
      deviceId = 'im-${_newNonce()}';
      await prefs.setString(_kDeviceId, deviceId);
    }

    var devicePublicKeyHex = prefs.getString(_kDevicePublicKeyHex) ?? '';
    final stateStore = await _stateStore(account.address, deviceId);
    var identity = ImMlsDeviceIdentity(
      walletChatAccount: account.address,
      deviceId: deviceId,
      devicePublicKeyHex:
          devicePublicKeyHex.isEmpty ? '00' : devicePublicKeyHex,
    );
    final crypto = _cryptoFactory?.call(identity, stateStore) ??
        NativeImMlsCrypto(identity: identity, stateStore: stateStore);
    ImMlsKeyPackage? freshKeyPackage;
    if (devicePublicKeyHex.isEmpty) {
      freshKeyPackage = await crypto.createKeyPackage(identity);
      final keyPackage = freshKeyPackage;
      if (keyPackage.devicePublicKeyHex.isEmpty) {
        throw StateError('OpenMLS native 未返回 IM 设备公钥，请先重编 native 库');
      }
      devicePublicKeyHex = keyPackage.devicePublicKeyHex;
      await prefs.setString(_kDevicePublicKeyHex, devicePublicKeyHex);
      identity = ImMlsDeviceIdentity(
        walletChatAccount: account.address,
        deviceId: deviceId,
        devicePublicKeyHex: devicePublicKeyHex,
      );
    }
    final finalCrypto = _cryptoFactory?.call(identity, stateStore) ??
        NativeImMlsCrypto(identity: identity, stateStore: stateStore);
    final mailbox = prepareMailbox
        ? await _ensureMailboxReady(
            account: account,
            identity: identity,
            crypto: finalCrypto,
            prefs: prefs,
            initialKeyPackage: freshKeyPackage,
          )
        : null;
    final transport = _cloudflareTransportFactory?.call(
          ownerChatAccount: account.address,
          ownerDeviceId: deviceId,
          mailboxBaseUrl: mailbox?.baseUri,
          sessionToken: mailbox?.session.sessionToken,
        ) ??
        ImCloudflareTransport(
          ownerChatAccount: account.address,
          ownerDeviceId: deviceId,
          mailboxBaseUrl: mailbox?.baseUri,
          sessionToken: mailbox?.session.sessionToken,
        );
    return _ImOwnerContext(
      account: account,
      deviceId: deviceId,
      devicePublicKeyHex: identity.devicePublicKeyHex,
      crypto: finalCrypto,
      transport: transport,
    );
  }

  Future<_ImMailboxContext> _ensureMailboxReady({
    required _ImCommunicationAccount account,
    required ImMlsDeviceIdentity identity,
    required ImMlsCryptoBoundary crypto,
    required SharedPreferences prefs,
    ImMlsKeyPackage? initialKeyPackage,
  }) async {
    final session = await _squareApiClient.ensureSession(
      ownerAccount: account.address,
      signLoginPayload: (payload) => _signSquareLoginPayload(account, payload),
    );
    final transport = _cloudflareTransportFactory?.call(
          ownerChatAccount: account.address,
          ownerDeviceId: identity.deviceId,
          mailboxBaseUrl: _squareApiClient.baseUri,
          sessionToken: session.sessionToken,
        ) ??
        ImCloudflareTransport(
          ownerChatAccount: account.address,
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
    return _ImMailboxContext(
      baseUri: _squareApiClient.baseUri,
      session: session,
    );
  }

  Future<void> _ensureDeviceRegistered({
    required _ImCommunicationAccount account,
    required ImMlsDeviceIdentity identity,
    required SharedPreferences prefs,
    required ImCloudflareTransport transport,
  }) async {
    final cacheKey = _deviceBindingCacheKey(identity);
    final cachedExpiresAt = prefs.getInt(cacheKey) ?? 0;
    final now = DateTime.now().millisecondsSinceEpoch;
    if (cachedExpiresAt - _keyPackageRefreshSkewMillis > now) {
      return;
    }

    final expiresAt = DateTime.now().toUtc().add(_mailboxBindingTtl);
    final draft = ImWalletBindingDraft(
      walletAccount: account.address,
      imDeviceId: identity.deviceId,
      imDevicePubkey: identity.devicePublicKeyHex,
      expiresAt: expiresAt,
      nonce: _newNonce(),
    );
    final signingMessage = signingBytesForImWalletBinding(
      draft.signingPayloadBytes(),
    );
    final signatureHex = await _signWalletPayload(
      account: account,
      payload: signingMessage,
    );
    await transport.registerDevice(
      devicePublicKeyHex: identity.devicePublicKeyHex,
      bindingSignature: signatureHex,
      expiresAtMillis: expiresAt.millisecondsSinceEpoch,
      nonce: draft.nonce,
    );
    await prefs.setInt(cacheKey, expiresAt.millisecondsSinceEpoch);
  }

  Future<void> _ensureOwnKeyPackagePublished({
    required ImMlsDeviceIdentity identity,
    required ImMlsCryptoBoundary crypto,
    required SharedPreferences prefs,
    required ImCloudflareTransport transport,
    ImMlsKeyPackage? initialKeyPackage,
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
      throw StateError('OpenMLS native 返回的 IM 设备公钥与本机身份不一致');
    }
    await transport.publishKeyPackage(keyPackage);
    await prefs.setInt(cacheKey, keyPackage.expiresAtMillis);
  }

  Future<String> _signSquareLoginPayload(
    _ImCommunicationAccount account,
    String signingPayload,
  ) async {
    final signer = _squareLoginPayloadSigner;
    if (signer != null) {
      return signer(
        walletIndex: account.walletIndex,
        ownerAccount: account.address,
        signingPayload: signingPayload,
      );
    }
    final payload = Uint8List.fromList(utf8.encode(signingPayload));
    return _signWalletPayload(account: account, payload: payload);
  }

  Future<String> _signWalletPayload({
    required _ImCommunicationAccount account,
    required Uint8List payload,
  }) async {
    final signer = _walletPayloadSigner;
    if (signer != null) {
      return signer(
        walletIndex: account.walletIndex,
        ownerAccount: account.address,
        payload: payload,
      );
    }
    final wallet = await _walletManager.getWalletByIndex(account.walletIndex);
    if (wallet == null || wallet.address != account.address) {
      throw StateError('通信账户钱包不存在，请重新设置通信账户');
    }
    if (!wallet.isHotWallet) {
      throw StateError('冷钱包通信账户需要在聊天页接入扫码签名后才能自动登记 IM 设备');
    }
    if (!_authenticatedWalletIndexes.contains(account.walletIndex)) {
      // 首次自动登记或登录 mailbox 时验证一次，之后只签 Worker 登录和设备绑定。
      await _walletManager.authenticateForSigning();
      _authenticatedWalletIndexes.add(account.walletIndex);
    }
    final signature =
        await _walletManager.signWithWalletNoAuth(account.walletIndex, payload);
    return '0x${_hexEncode(signature)}';
  }

  Future<_ImCommunicationAccount> _readCommunicationAccount() async {
    // 身份统一取默认用户钱包（钱包列表中最靠前的热钱包）。
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) {
      throw StateError('请先在「我的 → 我的钱包」创建热钱包，第一个热钱包即默认用户');
    }
    return _ImCommunicationAccount(
      walletIndex: wallet.walletIndex,
      address: wallet.address,
      walletName: wallet.walletName,
    );
  }

  ImMessageFlow _messageFlow(_ImOwnerContext context) {
    return ImMessageFlow(
      crypto: context.crypto,
      store: _store,
      deliverer: (envelope, _) {
        return ImMessageFlow.deliverWithTransport(
          transport: context.transport,
          envelope: envelope,
        );
      },
    );
  }

  Future<ImMlsStateStore> _stateStore(
    String walletAccount,
    String deviceId,
  ) async {
    final factory = _stateStoreFactory;
    if (factory != null) {
      return factory(walletAccount, deviceId);
    }
    final dir = await getApplicationDocumentsDirectory();
    final safeWallet = _safePath(walletAccount);
    final safeDevice = _safePath(deviceId);
    return ImMlsStateStore(
      Directory('${dir.path}/im/mls/$safeWallet/$safeDevice'),
    );
  }
}

class _ImMailboxContext {
  const _ImMailboxContext({
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

Uint8List signingBytesForImWalletBinding(Uint8List scalePayload) {
  return signingMessage(
    opTag: kOpSignImWalletBinding,
    scalePayload: scalePayload,
  );
}

String _safePath(String value) {
  return value.replaceAll(RegExp(r'[^a-zA-Z0-9_.-]'), '_');
}

String _deviceBindingCacheKey(ImMlsDeviceIdentity identity) {
  return '${ImRuntime._kDeviceBindingPrefix}.'
      '${_safePath(identity.walletChatAccount)}.'
      '${_safePath(identity.deviceId)}.${identity.devicePublicKeyHex}';
}

String _keyPackageCacheKey(ImMlsDeviceIdentity identity) {
  return '${ImRuntime._kKeyPackagePublishedPrefix}.'
      '${_safePath(identity.walletChatAccount)}.'
      '${_safePath(identity.deviceId)}.${identity.devicePublicKeyHex}';
}

String _hexEncode(List<int> bytes) {
  const chars = '0123456789abcdef';
  final buffer = StringBuffer();
  for (final byte in bytes) {
    buffer
      ..write(chars[(byte >> 4) & 0x0f])
      ..write(chars[byte & 0x0f]);
  }
  return buffer.toString();
}
