import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:path/path.dart' as path;

import 'im_mls_boundary.dart';
import 'im_mls_state_store.dart';

/// OpenMLS native smoke 结果。
class ImMlsNativeSmokeResult {
  const ImMlsNativeSmokeResult({
    required this.plaintext,
    required this.decryptedPlaintext,
    required this.cipherSuite,
    required this.aliceWireMessageHex,
    required this.bobKeyPackageHex,
    required this.welcomeHex,
  });

  final String plaintext;
  final String decryptedPlaintext;
  final String cipherSuite;
  final String aliceWireMessageHex;
  final String bobKeyPackageHex;
  final String welcomeHex;

  bool get roundTripOk => plaintext == decryptedPlaintext;

  factory ImMlsNativeSmokeResult.fromJson(Map<String, dynamic> json) {
    return ImMlsNativeSmokeResult(
      plaintext: (json['plaintext'] ?? '').toString(),
      decryptedPlaintext: (json['decrypted_plaintext'] ?? '').toString(),
      cipherSuite: (json['cipher_suite'] ?? '').toString(),
      aliceWireMessageHex: (json['alice_wire_message_hex'] ?? '').toString(),
      bobKeyPackageHex: (json['bob_key_package_hex'] ?? '').toString(),
      welcomeHex: (json['welcome_hex'] ?? '').toString(),
    );
  }
}

/// 通过现有 native 库调用 Rust OpenMLS。
///
/// 该类只负责跨 FFI 边界，密码学实现全部在 Rust OpenMLS 中完成。
class NativeImMlsCrypto implements ImMlsCryptoBoundary {
  NativeImMlsCrypto({
    ImMlsNativeBindings? bindings,
    ImMlsDeviceIdentity? identity,
    ImMlsStateStore? stateStore,
  })  : _bindings = bindings ?? ImMlsNativeBindings.load(),
        _identity = identity,
        _stateStore = stateStore;

  final ImMlsNativeBindings _bindings;
  final ImMlsDeviceIdentity? _identity;
  final ImMlsStateStore? _stateStore;

  @override
  Future<ImMlsKeyPackage> createKeyPackage(
    ImMlsDeviceIdentity identity,
  ) async {
    final error = identity.validate();
    if (error != null) {
      throw ArgumentError(error);
    }

    final response = _bindings.callJson(
      _bindings.createKeyPackage,
      {
        'wallet_chat_account': identity.walletChatAccount,
        'device_id': identity.deviceId,
        if (_stateStore != null) 'state_store_dir': _stateStore.path,
      },
    );
    return ImMlsKeyPackage(
      ownerChatAccount: identity.walletChatAccount,
      deviceId: identity.deviceId,
      devicePublicKeyHex: (response['device_public_key_hex'] ?? '').toString(),
      keyPackageId: (response['key_package_id'] ?? '').toString(),
      keyPackageBytes:
          _hexToBytes((response['key_package_hex'] ?? '').toString()),
      cipherSuite: (response['cipher_suite'] ?? '').toString(),
      createdAtMillis: (response['created_at_millis'] as num?)?.toInt() ?? 0,
      expiresAtMillis: (response['expires_at_millis'] as num?)?.toInt() ?? 0,
    );
  }

  /// 运行 Rust OpenMLS 两方 round-trip smoke。
  Future<ImMlsNativeSmokeResult> runTwoPartySmoke({
    required String plaintext,
  }) async {
    final response = _bindings.callJson(
      _bindings.twoPartySmoke,
      {'plaintext': plaintext},
    );
    return ImMlsNativeSmokeResult.fromJson(response);
  }

  @override
  Future<ImMlsOutboundMessage> encrypt({
    required String conversationId,
    required String recipientChatAccount,
    ImMlsKeyPackage? recipientKeyPackage,
    required List<int> plaintext,
  }) async {
    final identity = _requireIdentity();
    final stateStore = _requireStateStore();
    await stateStore.ensureReady();
    final response = _bindings.callJson(
      _bindings.encrypt,
      {
        'state_store_dir': stateStore.path,
        'wallet_chat_account': identity.walletChatAccount,
        'device_id': identity.deviceId,
        'conversation_id': conversationId,
        'recipient_chat_account': recipientChatAccount,
        'plaintext_hex': _bytesToHex(plaintext),
        if (recipientKeyPackage != null)
          'recipient_key_package_hex': recipientKeyPackage.keyPackageHex,
      },
    );
    final cipherSuite = (response['cipher_suite'] ?? '').toString();
    final welcomeHex = response['welcome_wire_message_hex']?.toString();
    final ratchetTreeHex = response['ratchet_tree_hex']?.toString();
    final welcomeMessage = welcomeHex == null || welcomeHex.isEmpty
        ? null
        : ImMlsWireMessage(
            wireBytes: _hexToBytes(welcomeHex),
            cipherSuite: cipherSuite,
            conversationId: conversationId,
            messageKind: ImMlsMessageKind.welcome,
            ratchetTreeBytes: ratchetTreeHex == null || ratchetTreeHex.isEmpty
                ? null
                : _hexToBytes(ratchetTreeHex),
          );
    return ImMlsOutboundMessage(
      conversationId: conversationId,
      welcomeMessage: welcomeMessage,
      applicationMessage: ImMlsWireMessage(
        wireBytes: _hexToBytes(
          (response['application_wire_message_hex'] ?? '').toString(),
        ),
        cipherSuite: cipherSuite,
        conversationId: conversationId,
        messageKind: ImMlsMessageKind.application,
      ),
    );
  }

  @override
  Future<List<int>> decrypt(ImMlsWireMessage message) async {
    final inbound = await processIncoming(message);
    return inbound.plaintext ?? const [];
  }

  /// 处理 Welcome 或 application wire message。
  @override
  Future<ImMlsInboundMessage> processIncoming(ImMlsWireMessage message) async {
    final identity = _requireIdentity();
    final stateStore = _requireStateStore();
    await stateStore.ensureReady();
    final response = _bindings.callJson(
      _bindings.decrypt,
      {
        'state_store_dir': stateStore.path,
        'wallet_chat_account': identity.walletChatAccount,
        'device_id': identity.deviceId,
        'conversation_id': message.conversationId,
        'wire_message_hex': message.wireHex,
        if (message.ratchetTreeHex != null)
          'ratchet_tree_hex': message.ratchetTreeHex,
      },
    );
    final plaintextHex = response['plaintext_hex']?.toString();
    return ImMlsInboundMessage(
      conversationId:
          (response['conversation_id'] ?? message.conversationId).toString(),
      messageKind: ImMlsMessageKind.fromWireName(
        (response['message_kind'] ?? '').toString(),
      ),
      plaintext: plaintextHex == null || plaintextHex.isEmpty
          ? null
          : _hexToBytes(plaintextHex),
    );
  }

  ImMlsDeviceIdentity _requireIdentity() {
    final identity = _identity;
    if (identity == null) {
      throw StateError('NativeImMlsCrypto 需要 IM 设备身份才能执行会话加解密');
    }
    final error = identity.validate();
    if (error != null) {
      throw ArgumentError(error);
    }
    return identity;
  }

  ImMlsStateStore _requireStateStore() {
    final store = _stateStore;
    if (store == null) {
      throw StateError('NativeImMlsCrypto 需要 MLS stateStore 才能持久化会话');
    }
    return store;
  }
}

typedef ImMlsJsonNative = Pointer<Utf8> Function(
  Pointer<Utf8> requestJson,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef ImMlsJsonDart = Pointer<Utf8> Function(
  Pointer<Utf8> requestJson,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef ImMlsFreeStringNative = Void Function(Pointer<Utf8> ptr);
typedef ImMlsFreeStringDart = void Function(Pointer<Utf8> ptr);

/// IM MLS native bindings。
class ImMlsNativeBindings {
  ImMlsNativeBindings._({
    required this.createKeyPackage,
    required this.twoPartySmoke,
    required this.encrypt,
    required this.decrypt,
    required ImMlsFreeStringDart freeString,
  }) : _freeString = freeString;

  final ImMlsJsonDart createKeyPackage;
  final ImMlsJsonDart twoPartySmoke;
  final ImMlsJsonDart encrypt;
  final ImMlsJsonDart decrypt;
  final ImMlsFreeStringDart _freeString;

  static ImMlsNativeBindings load() {
    final library = _loadSmoldotLibrary();
    return ImMlsNativeBindings._(
      createKeyPackage: library.lookupFunction<ImMlsJsonNative, ImMlsJsonDart>(
        'gmb_im_mls_create_key_package_json',
      ),
      twoPartySmoke: library.lookupFunction<ImMlsJsonNative, ImMlsJsonDart>(
        'gmb_im_mls_two_party_smoke_json',
      ),
      encrypt: library.lookupFunction<ImMlsJsonNative, ImMlsJsonDart>(
        'gmb_im_mls_encrypt_json',
      ),
      decrypt: library.lookupFunction<ImMlsJsonNative, ImMlsJsonDart>(
        'gmb_im_mls_decrypt_json',
      ),
      freeString:
          library.lookupFunction<ImMlsFreeStringNative, ImMlsFreeStringDart>(
        'smoldot_free_string',
      ),
    );
  }

  Map<String, dynamic> callJson(
    ImMlsJsonDart function,
    Map<String, Object?> request,
  ) {
    final requestPtr = jsonEncode(request).toNativeUtf8();
    final errorOut = calloc<Pointer<Utf8>>();
    Pointer<Utf8> resultPtr = nullptr;
    try {
      resultPtr = function(requestPtr, errorOut);
      if (resultPtr == nullptr) {
        final errorPtr = errorOut.value;
        final message = errorPtr == nullptr
            ? 'OpenMLS native 调用失败'
            : errorPtr.toDartString();
        if (errorPtr != nullptr) {
          _freeString(errorPtr);
        }
        throw StateError(message);
      }
      final json = jsonDecode(resultPtr.toDartString());
      return (json as Map).cast<String, dynamic>();
    } finally {
      calloc.free(requestPtr);
      calloc.free(errorOut);
      if (resultPtr != nullptr) {
        _freeString(resultPtr);
      }
    }
  }
}

DynamicLibrary _loadSmoldotLibrary() {
  final candidates = <String>[];
  final cwd = Directory.current.path;
  if (Platform.isMacOS || Platform.isIOS) {
    candidates.addAll([
      path.join(cwd, 'native', 'libsmoldot.dylib'),
      path.join(cwd, 'rust', 'target', 'release', 'libsmoldot.dylib'),
      'libsmoldot.dylib',
    ]);
  } else if (Platform.isWindows) {
    candidates.addAll([
      path.join(cwd, 'native', 'smoldot.dll'),
      path.join(cwd, 'rust', 'target', 'release', 'smoldot.dll'),
      'smoldot.dll',
    ]);
  } else {
    candidates.addAll([
      path.join(cwd, 'native', 'libsmoldot.so'),
      path.join(cwd, 'rust', 'target', 'release', 'libsmoldot.so'),
      'libsmoldot.so',
    ]);
  }

  Object? lastError;
  for (final candidate in candidates) {
    if (candidate.startsWith('lib') || File(candidate).existsSync()) {
      try {
        return DynamicLibrary.open(candidate);
      } catch (e) {
        lastError = e;
      }
    }
  }
  throw StateError('无法加载 libsmoldot native 库: $lastError');
}

List<int> _hexToBytes(String value) {
  final normalized = value.startsWith('0x') ? value.substring(2) : value;
  if (normalized.length.isOdd) {
    throw const FormatException('OpenMLS hex 长度必须为偶数');
  }
  if (normalized.isNotEmpty &&
      !RegExp(r'^[0-9a-fA-F]+$').hasMatch(normalized)) {
    throw const FormatException('OpenMLS hex 必须合法');
  }
  final bytes = <int>[];
  for (var i = 0; i < normalized.length; i += 2) {
    bytes.add(int.parse(normalized.substring(i, i + 2), radix: 16));
  }
  return bytes;
}

String _bytesToHex(List<int> bytes) {
  return bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}
