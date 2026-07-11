import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:path/path.dart' as path;

import 'mls_boundary.dart';
import 'mls_state_store.dart';

/// OpenMLS native smoke 结果。
class MlsNativeSmokeResult {
  const MlsNativeSmokeResult({
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

  factory MlsNativeSmokeResult.fromJson(Map<String, dynamic> json) {
    return MlsNativeSmokeResult(
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
class NativeMlsCrypto implements MlsCrypto {
  NativeMlsCrypto({
    MlsNativeBindings? bindings,
    ChatDevice? identity,
    MlsStateStore? stateStore,
  })  : _bindings = bindings ?? MlsNativeBindings.load(),
        _identity = identity,
        _stateStore = stateStore;

  final MlsNativeBindings _bindings;
  final ChatDevice? _identity;
  final MlsStateStore? _stateStore;

  @override
  Future<MlsKeyPackage> createKeyPackage(
    ChatDevice identity,
  ) async {
    final error = identity.validate();
    if (error != null) {
      throw ArgumentError(error);
    }

    final response = _bindings.callJson(
      _bindings.createKeyPackage,
      {
        'owner_account': identity.ownerAccount,
        'device_id': identity.deviceId,
        if (_stateStore != null) 'state_store_dir': _stateStore.path,
      },
    );
    return MlsKeyPackage(
      ownerAccount: identity.ownerAccount,
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
  Future<MlsNativeSmokeResult> runTwoPartySmoke({
    required String plaintext,
  }) async {
    final response = _bindings.callJson(
      _bindings.twoPartySmoke,
      {'plaintext': plaintext},
    );
    return MlsNativeSmokeResult.fromJson(response);
  }

  @override
  Future<MlsOutboundMessage> encrypt({
    required String conversationId,
    required String recipientAccount,
    MlsKeyPackage? recipientKeyPackage,
    required List<int> plaintext,
  }) async {
    final identity = _requireIdentity();
    final stateStore = _requireStateStore();
    await stateStore.ensureReady();
    final response = _bindings.callJson(
      _bindings.encrypt,
      {
        'state_store_dir': stateStore.path,
        'owner_account': identity.ownerAccount,
        'device_id': identity.deviceId,
        'conversation_id': conversationId,
        'recipient_account': recipientAccount,
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
        : MlsWireMessage(
            wireBytes: _hexToBytes(welcomeHex),
            cipherSuite: cipherSuite,
            conversationId: conversationId,
            messageKind: MlsMessageKind.welcome,
            ratchetTreeBytes: ratchetTreeHex == null || ratchetTreeHex.isEmpty
                ? null
                : _hexToBytes(ratchetTreeHex),
          );
    return MlsOutboundMessage(
      conversationId: conversationId,
      welcomeMessage: welcomeMessage,
      applicationMessage: MlsWireMessage(
        wireBytes: _hexToBytes(
          (response['application_wire_message_hex'] ?? '').toString(),
        ),
        cipherSuite: cipherSuite,
        conversationId: conversationId,
        messageKind: MlsMessageKind.application,
      ),
    );
  }

  @override
  Future<List<int>> decrypt(MlsWireMessage message) async {
    final inbound = await processIncoming(message);
    return inbound.plaintext ?? const [];
  }

  /// 处理 Welcome 或 application wire message。
  @override
  Future<MlsInboundMessage> processIncoming(MlsWireMessage message) async {
    final identity = _requireIdentity();
    final stateStore = _requireStateStore();
    await stateStore.ensureReady();
    final response = _bindings.callJson(
      _bindings.decrypt,
      {
        'state_store_dir': stateStore.path,
        'owner_account': identity.ownerAccount,
        'device_id': identity.deviceId,
        'conversation_id': message.conversationId,
        'wire_message_hex': message.wireHex,
        if (message.ratchetTreeHex != null)
          'ratchet_tree_hex': message.ratchetTreeHex,
      },
    );
    final plaintextHex = response['plaintext_hex']?.toString();
    return MlsInboundMessage(
      conversationId:
          (response['conversation_id'] ?? message.conversationId).toString(),
      messageKind: MlsMessageKind.fromWireName(
        (response['message_kind'] ?? '').toString(),
      ),
      plaintext: plaintextHex == null || plaintextHex.isEmpty
          ? null
          : _hexToBytes(plaintextHex),
    );
  }

  ChatDevice _requireIdentity() {
    final identity = _identity;
    if (identity == null) {
      throw StateError('NativeMlsCrypto 需要 Chat 设备身份才能执行会话加解密');
    }
    final error = identity.validate();
    if (error != null) {
      throw ArgumentError(error);
    }
    return identity;
  }

  MlsStateStore _requireStateStore() {
    final store = _stateStore;
    if (store == null) {
      throw StateError('NativeMlsCrypto 需要 MLS stateStore 才能持久化会话');
    }
    return store;
  }
}

typedef MlsJsonNative = Pointer<Utf8> Function(
  Pointer<Utf8> requestJson,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef MlsJsonDart = Pointer<Utf8> Function(
  Pointer<Utf8> requestJson,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef MlsFreeStringNative = Void Function(Pointer<Utf8> ptr);
typedef MlsFreeStringDart = void Function(Pointer<Utf8> ptr);

/// Chat MLS native bindings。
class MlsNativeBindings {
  MlsNativeBindings._({
    required this.createKeyPackage,
    required this.twoPartySmoke,
    required this.encrypt,
    required this.decrypt,
    required MlsFreeStringDart freeString,
  }) : _freeString = freeString;

  final MlsJsonDart createKeyPackage;
  final MlsJsonDart twoPartySmoke;
  final MlsJsonDart encrypt;
  final MlsJsonDart decrypt;
  final MlsFreeStringDart _freeString;

  static MlsNativeBindings load() {
    final library = _loadSmoldotLibrary();
    return MlsNativeBindings._(
      createKeyPackage: library.lookupFunction<MlsJsonNative, MlsJsonDart>(
        'gmb_chat_mls_create_key_package_json',
      ),
      twoPartySmoke: library.lookupFunction<MlsJsonNative, MlsJsonDart>(
        'gmb_chat_mls_two_party_smoke_json',
      ),
      encrypt: library.lookupFunction<MlsJsonNative, MlsJsonDart>(
        'gmb_chat_mls_encrypt_json',
      ),
      decrypt: library.lookupFunction<MlsJsonNative, MlsJsonDart>(
        'gmb_chat_mls_decrypt_json',
      ),
      freeString:
          library.lookupFunction<MlsFreeStringNative, MlsFreeStringDart>(
        'smoldot_free_string',
      ),
    );
  }

  Map<String, dynamic> callJson(
    MlsJsonDart function,
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
