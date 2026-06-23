import 'dart:math';
import 'dart:io';

import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../my/user/user_service.dart';
import '../qr/bodies/im_node_pairing_body.dart';
import '../wallet/core/wallet_manager.dart';
import 'crypto/im_mls_boundary.dart';
import 'crypto/im_mls_native.dart';
import 'crypto/im_mls_state_store.dart';
import 'im_message_flow.dart';
import 'im_session_models.dart';
import 'storage/im_isar_store.dart';
import 'transport/im_private_node_transport.dart';
import 'transport/im_transport.dart';

/// CitizenApp 与用户电脑通信节点完成配对后的本机配置。
class ImPairedNodeConfig {
  const ImPairedNodeConfig({
    required this.peerId,
    required this.multiaddr,
    this.pairedAtMillis,
  });

  static const empty = ImPairedNodeConfig(
    peerId: '',
    multiaddr: '',
  );

  final String peerId;
  final String multiaddr;
  final int? pairedAtMillis;

  bool get isComplete =>
      peerId.trim().isNotEmpty && multiaddr.trim().isNotEmpty;

  ImPrivateNodeEndpoint toEndpoint() {
    return ImPrivateNodeEndpoint(peerId: peerId, multiaddr: multiaddr);
  }

  String? validate() {
    final endpointError = toEndpoint().validate();
    if (endpointError != null) {
      return endpointError;
    }
    return null;
  }

  String get shortPeerId {
    final value = peerId.trim();
    if (value.length <= 16) return value;
    return '${value.substring(0, 8)}...${value.substring(value.length - 6)}';
  }
}

class _ImOwnerContext {
  const _ImOwnerContext({
    required this.account,
    required this.config,
    required this.deviceId,
    required this.devicePublicKeyHex,
    required this.crypto,
    required this.transport,
  });

  final _ImCommunicationAccount account;
  final ImPairedNodeConfig config;
  final String deviceId;
  final String devicePublicKeyHex;
  final ImMlsCryptoBoundary crypto;
  final ImPrivateNodeTransport transport;

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
/// 中文注释：页面层不直接操作 OpenMLS、P2P 通道和 Isar。这个服务负责读取
/// 本机通信节点配置、读取用户资料中的通信账户、建立设备身份，并把聊天发送
/// /同步接到后续专用 IM P2P 通道。
class ImRuntime {
  ImRuntime({
    ImIsarStore? store,
    WalletManager? walletManager,
    UserProfileService? profileService,
    SharedPreferences? preferences,
    ImMlsCryptoBoundary Function(
      ImMlsDeviceIdentity identity,
      ImMlsStateStore stateStore,
    )? cryptoFactory,
  })  : _store = store ?? ImIsarStore(),
        _walletManager = walletManager ?? WalletManager(),
        _profileService = profileService ?? UserProfileService(),
        _preferences = preferences,
        _cryptoFactory = cryptoFactory;

  static const _kPeerId = 'im.paired_node.peer_id';
  static const _kMultiaddr = 'im.paired_node.multiaddr';
  static const _kPairedAtMillis = 'im.paired_node.paired_at_millis';
  static const _kDeviceId = 'im.device.id';
  static const _kDevicePublicKeyHex = 'im.device.public_key_hex';

  final ImIsarStore _store;
  final WalletManager _walletManager;
  final UserProfileService _profileService;
  final SharedPreferences? _preferences;
  final ImMlsCryptoBoundary Function(
    ImMlsDeviceIdentity identity,
    ImMlsStateStore stateStore,
  )? _cryptoFactory;

  Future<SharedPreferences> get _prefs async {
    final provided = _preferences;
    if (provided != null) {
      return provided;
    }
    return SharedPreferences.getInstance();
  }

  Future<ImPairedNodeConfig> readPairedNodeConfig() async {
    final prefs = await _prefs;
    return ImPairedNodeConfig(
      peerId: prefs.getString(_kPeerId) ?? '',
      multiaddr: prefs.getString(_kMultiaddr) ?? '',
      pairedAtMillis: prefs.getInt(_kPairedAtMillis),
    );
  }

  Future<void> savePairedNodeConfig(ImPairedNodeConfig config) async {
    final error = config.validate();
    if (error != null) {
      throw ArgumentError(error);
    }
    final prefs = await _prefs;
    await prefs.setString(_kPeerId, config.peerId.trim());
    await prefs.setString(_kMultiaddr, config.multiaddr.trim());
    await prefs.setInt(
      _kPairedAtMillis,
      config.pairedAtMillis ?? DateTime.now().millisecondsSinceEpoch,
    );
  }

  /// 扫描区块链软件通信节点二维码后只保存通信节点信息。
  ///
  /// 中文注释：配对二维码只绑定用户自己的电脑通信节点；联系人入口仍在
  /// “我的通讯录”，信息 Tab 不承担节点设置。扫码阶段不得连接节点 RPC，
  /// 手机后续通过专用 IM P2P 通道连接自己的通信节点。
  Future<ImPairedNodeConfig> pairCommunicationNode(
    ImNodePairingBody body,
  ) async {
    final config = ImPairedNodeConfig(
      peerId: body.nodePeerId,
      multiaddr: body.nodeMultiaddr,
      pairedAtMillis: DateTime.now().millisecondsSinceEpoch,
    );
    await savePairedNodeConfig(config);
    return config;
  }

  Future<ImInboxOverview> readOverview({
    String? boundWalletAddress,
    required int pendingOutgoing,
    required int unreadCount,
  }) async {
    final config = await readPairedNodeConfig();
    final account = boundWalletAddress ?? await readCommunicationAddress();
    return ImInboxOverview(
      nodeStatus: config.isComplete
          ? ImNodeBindingStatus.online
          : ImNodeBindingStatus.unbound,
      boundWalletAddress: account,
      nodeEndpoint: config.isComplete ? config.multiaddr : null,
      pendingOutgoing: pendingOutgoing,
      unreadCount: unreadCount,
    );
  }

  Future<String?> readCommunicationAddress() async {
    final profile = await _profileService.getState();
    final address = profile.communicationAddress?.trim() ?? '';
    return address.isEmpty ? null : address;
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
    final route = await _store.getRouteRecord(peerWalletAddress);
    if (route == null) {
      throw StateError('联系人暂未提供通信路由，暂不能发送消息');
    }
    final context = await _ensureOwnerContext(createDeviceKeyPackage: false);
    final flow = _messageFlow(context, route);
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
      final remoteEndpoint = _remoteEndpoint(route);
      final packages = await context.transport.fetchDirectKeyPackages(
        remoteEndpoint: remoteEndpoint,
        ownerChatAccount: route.walletChatAccount,
        requesterChatAccount: context.account.address,
      );
      if (packages.isEmpty) {
        throw StateError('对方私人通信全节点没有可用 IM KeyPackage');
      }
      final consumed = await context.transport.consumeDirectKeyPackage(
        remoteEndpoint: remoteEndpoint,
        ownerChatAccount: route.walletChatAccount,
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

  Future<int> syncPending() async {
    final context = await _ensureOwnerContext(createDeviceKeyPackage: false);
    final flow = ImMessageFlow(
      crypto: context.crypto,
      store: _store,
      deliverer: (envelope, _) async => ImDeliveryResult(
        envelopeId: envelope.envelopeId,
        transportType: ImTransportType.privateNode,
        state: ImMessageDeliveryState.failed,
        errorMessage: '入站同步不执行投递',
      ),
    );
    return flow.fetchAndProcessPending(
      fetchPending: context.transport.fetchPending,
      ackEnvelope: context.transport.ackEnvelope,
    );
  }

  Future<_ImOwnerContext> _ensureOwnerContext({
    required bool createDeviceKeyPackage,
  }) async {
    final account = await _readCommunicationAccount();
    final config = await readPairedNodeConfig();
    final configError = config.validate();
    if (configError != null) {
      throw StateError(configError);
    }
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
    if (devicePublicKeyHex.isEmpty || createDeviceKeyPackage) {
      final keyPackage = await crypto.createKeyPackage(identity);
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
    final context = _ImOwnerContext(
      account: account,
      config: config,
      deviceId: deviceId,
      devicePublicKeyHex: identity.devicePublicKeyHex,
      crypto: finalCrypto,
      transport: ImPrivateNodeTransport(
        ownerChatAccount: account.address,
        ownerDeviceId: deviceId,
        ownerNodeEndpoint: config.toEndpoint(),
      ),
    );
    return context;
  }

  Future<_ImCommunicationAccount> _readCommunicationAccount() async {
    final profile = await _profileService.getState();
    final walletIndex = profile.communicationWalletIndex;
    final address = profile.communicationAddress?.trim() ?? '';
    if (walletIndex == null || address.isEmpty) {
      throw StateError('请先在用户资料中设置通信账户');
    }
    final wallet = await _walletManager.getWalletByIndex(walletIndex);
    if (wallet == null) {
      throw StateError('通信账户钱包不存在，请重新设置通信账户');
    }
    if (wallet.address != address) {
      throw StateError('通信账户地址与本地钱包不一致，请重新设置通信账户');
    }
    return _ImCommunicationAccount(
      walletIndex: wallet.walletIndex,
      address: wallet.address,
      walletName: wallet.walletName,
    );
  }

  ImMessageFlow _messageFlow(
    _ImOwnerContext context,
    ImRouteRecord route,
  ) {
    final remoteEndpoint = _remoteEndpoint(route);
    return ImMessageFlow(
      crypto: context.crypto,
      store: _store,
      deliverer: (envelope, _) {
        return ImMessageFlow.deliverWithPrivateNode(
          transport: context.transport,
          remoteEndpoint: remoteEndpoint,
          envelope: envelope,
        );
      },
    );
  }

  ImPrivateNodeEndpoint _remoteEndpoint(ImRouteRecord route) {
    final endpoint = ImPrivateNodeEndpoint(
      peerId: route.nodePeerId,
      multiaddr: route.nodeMultiaddr,
    );
    final error = endpoint.validate();
    if (error != null) {
      throw StateError(error);
    }
    return endpoint;
  }

  Future<ImMlsStateStore> _stateStore(
    String walletAccount,
    String deviceId,
  ) async {
    final dir = await getApplicationDocumentsDirectory();
    final safeWallet = _safePath(walletAccount);
    final safeDevice = _safePath(deviceId);
    return ImMlsStateStore(
      Directory('${dir.path}/im/mls/$safeWallet/$safeDevice'),
    );
  }
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
