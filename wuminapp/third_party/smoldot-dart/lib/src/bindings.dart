import 'dart:ffi';
import 'package:ffi/ffi.dart';
import 'platform.dart';

// Native type definitions matching C header
typedef ClientHandle = Uint64;
typedef ChainHandle = Uint64;

// Dart callback type
typedef DartCallbackNative = Void Function(
    Int64 callbackId, Int64 result, Pointer<Utf8> error);
typedef DartCallbackDart = void Function(
    int callbackId, int result, Pointer<Utf8> error);

// Native function signatures
typedef SmoldotClientInitNative = ClientHandle Function(
    Pointer<Utf8> configJson, Pointer<Pointer<Utf8>> errorOut);
typedef SmoldotClientInitDart = int Function(
    Pointer<Utf8> configJson, Pointer<Pointer<Utf8>> errorOut);

typedef SmoldotAddChainNative = Int32 Function(
  ClientHandle clientHandle,
  Pointer<Utf8> chainSpecJson,
  Pointer<ChainHandle> potentialRelayChains,
  Int32 relayCount,
  Pointer<Utf8> databaseContent,
  Int64 callbackId,
  Pointer<NativeFunction<DartCallbackNative>> callback,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef SmoldotAddChainDart = int Function(
  int clientHandle,
  Pointer<Utf8> chainSpecJson,
  Pointer<Uint64> potentialRelayChains,
  int relayCount,
  Pointer<Utf8> databaseContent,
  int callbackId,
  Pointer<NativeFunction<DartCallbackNative>> callback,
  Pointer<Pointer<Utf8>> errorOut,
);

typedef SmoldotSendJsonRpcNative = Int32 Function(
  ChainHandle chainHandle,
  Pointer<Utf8> requestJson,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef SmoldotSendJsonRpcDart = int Function(
  int chainHandle,
  Pointer<Utf8> requestJson,
  Pointer<Pointer<Utf8>> errorOut,
);

typedef SmoldotNextJsonRpcResponseNative = Int32 Function(
  ChainHandle chainHandle,
  Int64 callbackId,
  Pointer<NativeFunction<DartCallbackNative>> callback,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef SmoldotNextJsonRpcResponseDart = int Function(
  int chainHandle,
  int callbackId,
  Pointer<NativeFunction<DartCallbackNative>> callback,
  Pointer<Pointer<Utf8>> errorOut,
);

typedef SmoldotRemoveChainNative = Int32 Function(
    ChainHandle chainHandle, Pointer<Pointer<Utf8>> errorOut);
typedef SmoldotRemoveChainDart = int Function(
    int chainHandle, Pointer<Pointer<Utf8>> errorOut);

typedef SmoldotClientDestroyNative = Int32 Function(
    ClientHandle clientHandle, Pointer<Pointer<Utf8>> errorOut);
typedef SmoldotClientDestroyDart = int Function(
    int clientHandle, Pointer<Pointer<Utf8>> errorOut);

typedef SmoldotFreeStringNative = Void Function(Pointer<Utf8> ptr);
typedef SmoldotFreeStringDart = void Function(Pointer<Utf8> ptr);

typedef SmoldotVersionNative = Pointer<Utf8> Function();
typedef SmoldotVersionDart = Pointer<Utf8> Function();

typedef SmoldotGetStatusSnapshotNative = Pointer<Utf8> Function(
    ChainHandle chainHandle, Pointer<Pointer<Utf8>> errorOut);
typedef SmoldotGetStatusSnapshotDart = Pointer<Utf8> Function(
    int chainHandle, Pointer<Pointer<Utf8>> errorOut);

typedef SmoldotGetRuntimeVersionNative = Pointer<Utf8> Function(
    ChainHandle chainHandle, Pointer<Pointer<Utf8>> errorOut);
typedef SmoldotGetRuntimeVersionDart = Pointer<Utf8> Function(
    int chainHandle, Pointer<Pointer<Utf8>> errorOut);

typedef SmoldotGetMetadataNative = Pointer<Utf8> Function(
    ChainHandle chainHandle, Pointer<Pointer<Utf8>> errorOut);
typedef SmoldotGetMetadataDart = Pointer<Utf8> Function(
    int chainHandle, Pointer<Pointer<Utf8>> errorOut);

typedef SmoldotGetAccountNextIndexNative = Pointer<Utf8> Function(
  ChainHandle chainHandle,
  Pointer<Utf8> ss58Address,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef SmoldotGetAccountNextIndexDart = Pointer<Utf8> Function(
  int chainHandle,
  Pointer<Utf8> ss58Address,
  Pointer<Pointer<Utf8>> errorOut,
);

typedef SmoldotGetBlockHashNative = Pointer<Utf8> Function(
  ChainHandle chainHandle,
  Pointer<Utf8> blockNumber,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef SmoldotGetBlockHashDart = Pointer<Utf8> Function(
  int chainHandle,
  Pointer<Utf8> blockNumber,
  Pointer<Pointer<Utf8>> errorOut,
);

typedef SmoldotGetBlockExtrinsicsNative = Pointer<Utf8> Function(
  ChainHandle chainHandle,
  Pointer<Utf8> blockHashHex,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef SmoldotGetBlockExtrinsicsDart = Pointer<Utf8> Function(
  int chainHandle,
  Pointer<Utf8> blockHashHex,
  Pointer<Pointer<Utf8>> errorOut,
);

typedef SmoldotSubmitExtrinsicNative = Pointer<Utf8> Function(
  ChainHandle chainHandle,
  Pointer<Utf8> extrinsicHex,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef SmoldotSubmitExtrinsicDart = Pointer<Utf8> Function(
  int chainHandle,
  Pointer<Utf8> extrinsicHex,
  Pointer<Pointer<Utf8>> errorOut,
);

typedef SmoldotGetSystemAccountNative = Pointer<Utf8> Function(
  ChainHandle chainHandle,
  Pointer<Utf8> accountIdHex,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef SmoldotGetSystemAccountDart = Pointer<Utf8> Function(
  int chainHandle,
  Pointer<Utf8> accountIdHex,
  Pointer<Pointer<Utf8>> errorOut,
);

typedef SmoldotGetStorageValueNative = Pointer<Utf8> Function(
  ChainHandle chainHandle,
  Pointer<Utf8> storageKeyHex,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef SmoldotGetStorageValueDart = Pointer<Utf8> Function(
  int chainHandle,
  Pointer<Utf8> storageKeyHex,
  Pointer<Pointer<Utf8>> errorOut,
);

typedef SmoldotGetStorageValuesNative = Pointer<Utf8> Function(
  ChainHandle chainHandle,
  Pointer<Utf8> storageKeysJson,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef SmoldotGetStorageValuesDart = Pointer<Utf8> Function(
  int chainHandle,
  Pointer<Utf8> storageKeysJson,
  Pointer<Pointer<Utf8>> errorOut,
);

// ──── 异步 FFI 类型声明（不阻塞 Dart 主线程） ────

typedef SmoldotAsyncNoArgNative = Int32 Function(
  ChainHandle chainHandle,
  Int64 callbackId,
  Pointer<NativeFunction<DartCallbackNative>> callback,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef SmoldotAsyncNoArgDart = int Function(
  int chainHandle,
  int callbackId,
  Pointer<NativeFunction<DartCallbackNative>> callback,
  Pointer<Pointer<Utf8>> errorOut,
);

typedef SmoldotAsyncOneArgNative = Int32 Function(
  ChainHandle chainHandle,
  Pointer<Utf8> arg1,
  Int64 callbackId,
  Pointer<NativeFunction<DartCallbackNative>> callback,
  Pointer<Pointer<Utf8>> errorOut,
);
typedef SmoldotAsyncOneArgDart = int Function(
  int chainHandle,
  Pointer<Utf8> arg1,
  int callbackId,
  Pointer<NativeFunction<DartCallbackNative>> callback,
  Pointer<Pointer<Utf8>> errorOut,
);

/// FFI bindings for smoldot-light native library
class SmoldotBindings {
  late final DynamicLibrary _library;
  late final Allocator _allocator;

  // Function pointers
  late final SmoldotClientInitDart _clientInit;
  late final SmoldotAddChainDart _addChain;
  late final SmoldotSendJsonRpcDart _sendJsonRpc;
  late final SmoldotNextJsonRpcResponseDart _nextJsonRpcResponse;
  late final SmoldotRemoveChainDart _removeChain;
  late final SmoldotClientDestroyDart _clientDestroy;
  late final SmoldotFreeStringDart _freeString;
  late final SmoldotVersionDart _version;
  late final SmoldotGetStatusSnapshotDart _getStatusSnapshot;
  late final SmoldotGetRuntimeVersionDart _getRuntimeVersion;
  late final SmoldotGetMetadataDart _getMetadata;
  late final SmoldotGetAccountNextIndexDart _getAccountNextIndex;
  late final SmoldotGetBlockHashDart _getBlockHash;
  late final SmoldotGetBlockExtrinsicsDart _getBlockExtrinsics;
  late final SmoldotSubmitExtrinsicDart _submitExtrinsic;
  late final SmoldotGetSystemAccountDart _getSystemAccount;
  late final SmoldotGetStorageValueDart _getStorageValue;
  late final SmoldotGetStorageValuesDart _getStorageValues;

  // 异步版本
  late final SmoldotAsyncNoArgDart _getStatusSnapshotAsync;
  late final SmoldotAsyncNoArgDart _getRuntimeVersionAsync;
  late final SmoldotAsyncNoArgDart _getMetadataAsync;
  late final SmoldotAsyncOneArgDart _getAccountNextIndexAsync;
  late final SmoldotAsyncOneArgDart _getBlockHashAsync;
  late final SmoldotAsyncOneArgDart _getBlockExtrinsicsAsync;
  late final SmoldotAsyncOneArgDart _submitExtrinsicAsync;
  late final SmoldotAsyncOneArgDart _getSystemAccountAsync;
  late final SmoldotAsyncOneArgDart _getStorageValueAsync;
  late final SmoldotAsyncOneArgDart _getStorageValuesAsync;

  /// Initialize the bindings by loading the native library
  SmoldotBindings() {
    _library = SmoldotPlatform.loadLibrary();
    _allocator = malloc;
    _initializeBindings();
  }

  /// Initialize function pointers from the library
  void _initializeBindings() {
    _clientInit =
        _library.lookupFunction<SmoldotClientInitNative, SmoldotClientInitDart>(
            'smoldot_client_init');
    _addChain =
        _library.lookupFunction<SmoldotAddChainNative, SmoldotAddChainDart>(
            'smoldot_add_chain');
    _sendJsonRpc = _library.lookupFunction<SmoldotSendJsonRpcNative,
        SmoldotSendJsonRpcDart>('smoldot_send_json_rpc');
    _nextJsonRpcResponse = _library.lookupFunction<
        SmoldotNextJsonRpcResponseNative,
        SmoldotNextJsonRpcResponseDart>('smoldot_next_json_rpc_response');
    _removeChain = _library.lookupFunction<SmoldotRemoveChainNative,
        SmoldotRemoveChainDart>('smoldot_remove_chain');
    _clientDestroy = _library.lookupFunction<SmoldotClientDestroyNative,
        SmoldotClientDestroyDart>('smoldot_client_destroy');
    _freeString =
        _library.lookupFunction<SmoldotFreeStringNative, SmoldotFreeStringDart>(
            'smoldot_free_string');
    _version =
        _library.lookupFunction<SmoldotVersionNative, SmoldotVersionDart>(
            'smoldot_version');
    _getStatusSnapshot = _library.lookupFunction<
        SmoldotGetStatusSnapshotNative,
        SmoldotGetStatusSnapshotDart>('smoldot_get_status_snapshot');
    _getRuntimeVersion = _library.lookupFunction<
        SmoldotGetRuntimeVersionNative,
        SmoldotGetRuntimeVersionDart>('smoldot_get_runtime_version');
    _getMetadata = _library.lookupFunction<SmoldotGetMetadataNative,
        SmoldotGetMetadataDart>('smoldot_get_metadata');
    _getAccountNextIndex = _library.lookupFunction<
        SmoldotGetAccountNextIndexNative,
        SmoldotGetAccountNextIndexDart>('smoldot_get_account_next_index');
    _getBlockHash = _library.lookupFunction<SmoldotGetBlockHashNative,
        SmoldotGetBlockHashDart>('smoldot_get_block_hash');
    _getBlockExtrinsics = _library.lookupFunction<
        SmoldotGetBlockExtrinsicsNative,
        SmoldotGetBlockExtrinsicsDart>('smoldot_get_block_extrinsics');
    _submitExtrinsic = _library.lookupFunction<SmoldotSubmitExtrinsicNative,
        SmoldotSubmitExtrinsicDart>('smoldot_submit_extrinsic');
    _getSystemAccount = _library.lookupFunction<SmoldotGetSystemAccountNative,
        SmoldotGetSystemAccountDart>('smoldot_get_system_account');
    _getStorageValue = _library.lookupFunction<SmoldotGetStorageValueNative,
        SmoldotGetStorageValueDart>('smoldot_get_storage_value');
    _getStorageValues = _library.lookupFunction<SmoldotGetStorageValuesNative,
        SmoldotGetStorageValuesDart>('smoldot_get_storage_values');

    // 异步版本
    _getStatusSnapshotAsync = _library.lookupFunction<SmoldotAsyncNoArgNative,
        SmoldotAsyncNoArgDart>('smoldot_get_status_snapshot_async');
    _getRuntimeVersionAsync = _library.lookupFunction<SmoldotAsyncNoArgNative,
        SmoldotAsyncNoArgDart>('smoldot_get_runtime_version_async');
    _getMetadataAsync = _library.lookupFunction<SmoldotAsyncNoArgNative,
        SmoldotAsyncNoArgDart>('smoldot_get_metadata_async');
    _getAccountNextIndexAsync = _library.lookupFunction<
        SmoldotAsyncOneArgNative,
        SmoldotAsyncOneArgDart>('smoldot_get_account_next_index_async');
    _getBlockHashAsync = _library.lookupFunction<SmoldotAsyncOneArgNative,
        SmoldotAsyncOneArgDart>('smoldot_get_block_hash_async');
    _getBlockExtrinsicsAsync = _library.lookupFunction<
        SmoldotAsyncOneArgNative,
        SmoldotAsyncOneArgDart>('smoldot_get_block_extrinsics_async');
    _submitExtrinsicAsync = _library.lookupFunction<SmoldotAsyncOneArgNative,
        SmoldotAsyncOneArgDart>('smoldot_submit_extrinsic_async');
    _getSystemAccountAsync = _library.lookupFunction<SmoldotAsyncOneArgNative,
        SmoldotAsyncOneArgDart>('smoldot_get_system_account_async');
    _getStorageValueAsync = _library.lookupFunction<SmoldotAsyncOneArgNative,
        SmoldotAsyncOneArgDart>('smoldot_get_storage_value_async');
    _getStorageValuesAsync = _library.lookupFunction<SmoldotAsyncOneArgNative,
        SmoldotAsyncOneArgDart>('smoldot_get_storage_values_async');
  }

  // ===== Core Client Functions =====

  /// Initialize the smoldot client
  ///
  /// Takes a JSON configuration string and returns a client handle.
  /// Returns 0 if initialization fails.
  int initClient(String configJson) {
    final configPtr = configJson.toNativeUtf8(allocator: _allocator);
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;

    try {
      final handle = _clientInit(configPtr, errorOutPtr);

      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to initialize client: $error');
      }

      if (handle == 0) {
        throw Exception('Failed to initialize client: returned null handle');
      }

      return handle;
    } finally {
      _allocator.free(configPtr);
      _allocator.free(errorOutPtr);
    }
  }

  /// Destroy the smoldot client and free resources
  void destroyClient(int clientHandle) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;

    try {
      final result = _clientDestroy(clientHandle, errorOutPtr);

      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to destroy client: $error');
      }

      if (result != 0) {
        throw Exception('Failed to destroy client: error code $result');
      }
    } finally {
      _allocator.free(errorOutPtr);
    }
  }

  /// Add a chain to the client (async operation via callback)
  ///
  /// Returns immediately, actual result comes via callback.
  void addChain({
    required int clientHandle,
    required String chainSpecJson,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
    List<int>? potentialRelayChains,
    String? databaseContent,
  }) {
    final chainSpecPtr = chainSpecJson.toNativeUtf8(allocator: _allocator);
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;

    Pointer<Uint64>? relayChainPtr;
    Pointer<Utf8>? dbContentPtr;

    try {
      // Handle relay chains
      if (potentialRelayChains != null && potentialRelayChains.isNotEmpty) {
        relayChainPtr = _allocator<Uint64>(potentialRelayChains.length);
        for (var i = 0; i < potentialRelayChains.length; i++) {
          relayChainPtr[i] = potentialRelayChains[i];
        }
      }

      // Handle database content
      if (databaseContent != null) {
        dbContentPtr = databaseContent.toNativeUtf8(allocator: _allocator);
      }

      final result = _addChain(
        clientHandle,
        chainSpecPtr,
        relayChainPtr ?? nullptr,
        potentialRelayChains?.length ?? 0,
        dbContentPtr ?? nullptr,
        callbackId,
        callback,
        errorOutPtr,
      );

      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to add chain: $error');
      }

      if (result != 0) {
        throw Exception('Failed to add chain: error code $result');
      }
    } finally {
      _allocator.free(chainSpecPtr);
      _allocator.free(errorOutPtr);
      if (relayChainPtr != null) _allocator.free(relayChainPtr);
      if (dbContentPtr != null) _allocator.free(dbContentPtr);
    }
  }

  /// Remove a chain from the client
  void removeChain(int chainHandle) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;

    try {
      final result = _removeChain(chainHandle, errorOutPtr);

      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to remove chain: $error');
      }

      if (result != 0) {
        throw Exception('Failed to remove chain: error code $result');
      }
    } finally {
      _allocator.free(errorOutPtr);
    }
  }

  // ===== JSON-RPC Functions =====

  /// Send a JSON-RPC request to a chain
  void sendJsonRpcRequest(int chainHandle, String requestJson) {
    final requestPtr = requestJson.toNativeUtf8(allocator: _allocator);
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;

    try {
      final result = _sendJsonRpc(chainHandle, requestPtr, errorOutPtr);

      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to send JSON-RPC request: $error');
      }

      if (result != 0) {
        throw Exception('Failed to send JSON-RPC request: error code $result');
      }
    } finally {
      _allocator.free(requestPtr);
      _allocator.free(errorOutPtr);
    }
  }

  /// Get the next JSON-RPC response (async operation via callback)
  void nextJsonRpcResponse({
    required int chainHandle,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
  }) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;

    try {
      final result = _nextJsonRpcResponse(
        chainHandle,
        callbackId,
        callback,
        errorOutPtr,
      );

      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to get next JSON-RPC response: $error');
      }

      if (result != 0) {
        throw Exception(
            'Failed to get next JSON-RPC response: error code $result');
      }
    } finally {
      _allocator.free(errorOutPtr);
    }
  }

  /// Free a string allocated by Rust
  void freeString(Pointer<Utf8> ptr) {
    if (ptr != nullptr) {
      _freeString(ptr);
    }
  }

  /// Get the version of the smoldot FFI library
  String getVersion() {
    final versionPtr = _version();
    try {
      return versionPtr.toDartString();
    } finally {
      _freeString(versionPtr);
    }
  }

  // ──── 以下同步方法已废弃（阻塞 Dart 主线程），请使用对应的 *Async 版本 ────

  @Deprecated('Use getStatusSnapshotAsync instead')
  String getStatusSnapshotJson(int chainHandle) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;

    try {
      final resultPtr = _getStatusSnapshot(chainHandle, errorOutPtr);
      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to get status snapshot: $error');
      }
      if (resultPtr == nullptr) {
        throw Exception('Failed to get status snapshot: null result');
      }
      try {
        return resultPtr.toDartString();
      } finally {
        _freeString(resultPtr);
      }
    } finally {
      _allocator.free(errorOutPtr);
    }
  }

  @Deprecated('Use getRuntimeVersionAsync instead')
  String getRuntimeVersionJson(int chainHandle) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;

    try {
      final resultPtr = _getRuntimeVersion(chainHandle, errorOutPtr);
      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to get runtime version: $error');
      }
      if (resultPtr == nullptr) {
        throw Exception('Failed to get runtime version: null result');
      }
      try {
        return resultPtr.toDartString();
      } finally {
        _freeString(resultPtr);
      }
    } finally {
      _allocator.free(errorOutPtr);
    }
  }

  @Deprecated('Use getMetadataAsync instead')
  String getMetadataHex(int chainHandle) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;

    try {
      final resultPtr = _getMetadata(chainHandle, errorOutPtr);
      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to get metadata: $error');
      }
      if (resultPtr == nullptr) {
        throw Exception('Failed to get metadata: null result');
      }
      try {
        return resultPtr.toDartString();
      } finally {
        _freeString(resultPtr);
      }
    } finally {
      _allocator.free(errorOutPtr);
    }
  }

  @Deprecated('Use getAccountNextIndexAsync instead')
  String getAccountNextIndex(int chainHandle, String accountIdHex) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;
    final accountIdPtr = accountIdHex.toNativeUtf8(allocator: _allocator);

    try {
      final resultPtr =
          _getAccountNextIndex(chainHandle, accountIdPtr, errorOutPtr);
      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to get account next index: $error');
      }
      if (resultPtr == nullptr) {
        throw Exception('Failed to get account next index: null result');
      }
      try {
        return resultPtr.toDartString();
      } finally {
        _freeString(resultPtr);
      }
    } finally {
      _allocator.free(accountIdPtr);
      _allocator.free(errorOutPtr);
    }
  }

  @Deprecated('Use getBlockHashAsync instead')
  String getBlockHash(int chainHandle, int blockNumber) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;
    final blockNumberPtr =
        blockNumber.toString().toNativeUtf8(allocator: _allocator);

    try {
      final resultPtr = _getBlockHash(chainHandle, blockNumberPtr, errorOutPtr);
      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to get block hash: $error');
      }
      if (resultPtr == nullptr) {
        throw Exception('Failed to get block hash: null result');
      }
      try {
        return resultPtr.toDartString();
      } finally {
        _freeString(resultPtr);
      }
    } finally {
      _allocator.free(blockNumberPtr);
      _allocator.free(errorOutPtr);
    }
  }

  @Deprecated('Use getBlockExtrinsicsAsync instead')
  String getBlockExtrinsicsJson(int chainHandle, String blockHashHex) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;
    final blockHashPtr = blockHashHex.toNativeUtf8(allocator: _allocator);

    try {
      final resultPtr =
          _getBlockExtrinsics(chainHandle, blockHashPtr, errorOutPtr);
      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to get block extrinsics: $error');
      }
      if (resultPtr == nullptr) {
        throw Exception('Failed to get block extrinsics: null result');
      }
      try {
        return resultPtr.toDartString();
      } finally {
        _freeString(resultPtr);
      }
    } finally {
      _allocator.free(blockHashPtr);
      _allocator.free(errorOutPtr);
    }
  }

  @Deprecated('Use submitExtrinsicAsync instead')
  String submitExtrinsicHex(int chainHandle, String extrinsicHex) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;
    final extrinsicPtr = extrinsicHex.toNativeUtf8(allocator: _allocator);

    try {
      final resultPtr =
          _submitExtrinsic(chainHandle, extrinsicPtr, errorOutPtr);
      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to submit extrinsic: $error');
      }
      if (resultPtr == nullptr) {
        throw Exception('Failed to submit extrinsic: null result');
      }
      try {
        return resultPtr.toDartString();
      } finally {
        _freeString(resultPtr);
      }
    } finally {
      _allocator.free(extrinsicPtr);
      _allocator.free(errorOutPtr);
    }
  }

  @Deprecated('Use getSystemAccountAsync instead')
  String getSystemAccountJson(int chainHandle, String accountIdHex) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;
    final accountIdPtr = accountIdHex.toNativeUtf8(allocator: _allocator);

    try {
      final resultPtr = _getSystemAccount(chainHandle, accountIdPtr, errorOutPtr);
      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to get system account: $error');
      }
      if (resultPtr == nullptr) {
        throw Exception('Failed to get system account: null result');
      }
      try {
        return resultPtr.toDartString();
      } finally {
        _freeString(resultPtr);
      }
    } finally {
      _allocator.free(accountIdPtr);
      _allocator.free(errorOutPtr);
    }
  }

  @Deprecated('Use getStorageValueAsync instead')
  String getStorageValueJson(int chainHandle, String storageKeyHex) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;
    final storageKeyPtr = storageKeyHex.toNativeUtf8(allocator: _allocator);

    try {
      final resultPtr = _getStorageValue(chainHandle, storageKeyPtr, errorOutPtr);
      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to get storage value: $error');
      }
      if (resultPtr == nullptr) {
        throw Exception('Failed to get storage value: null result');
      }
      try {
        return resultPtr.toDartString();
      } finally {
        _freeString(resultPtr);
      }
    } finally {
      _allocator.free(storageKeyPtr);
      _allocator.free(errorOutPtr);
    }
  }

  @Deprecated('Use getStorageValuesAsync instead')
  String getStorageValuesJson(int chainHandle, String storageKeysJson) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;
    final storageKeysPtr = storageKeysJson.toNativeUtf8(allocator: _allocator);

    try {
      final resultPtr =
          _getStorageValues(chainHandle, storageKeysPtr, errorOutPtr);
      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('Failed to get storage values: $error');
      }
      if (resultPtr == nullptr) {
        throw Exception('Failed to get storage values: null result');
      }
      try {
        return resultPtr.toDartString();
      } finally {
        _freeString(resultPtr);
      }
    } finally {
      _allocator.free(storageKeysPtr);
      _allocator.free(errorOutPtr);
    }
  }

  // ──── 异步版本（不阻塞 Dart 主线程） ────

  void _invokeAsyncNoArg(
    SmoldotAsyncNoArgDart fn, {
    required int chainHandle,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
    required String debugName,
  }) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;
    try {
      final result = fn(chainHandle, callbackId, callback, errorOutPtr);
      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('$debugName failed: $error');
      }
      if (result != 0) {
        throw Exception('$debugName failed: error code $result');
      }
    } finally {
      _allocator.free(errorOutPtr);
    }
  }

  void _invokeAsyncOneArg(
    SmoldotAsyncOneArgDart fn, {
    required int chainHandle,
    required String arg,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
    required String debugName,
  }) {
    final errorOutPtr = _allocator<Pointer<Utf8>>();
    errorOutPtr.value = nullptr;
    final argPtr = arg.toNativeUtf8(allocator: _allocator);
    try {
      final result = fn(chainHandle, argPtr, callbackId, callback, errorOutPtr);
      if (errorOutPtr.value != nullptr) {
        final error = errorOutPtr.value.toDartString();
        _freeString(errorOutPtr.value);
        throw Exception('$debugName failed: $error');
      }
      if (result != 0) {
        throw Exception('$debugName failed: error code $result');
      }
    } finally {
      _allocator.free(argPtr);
      _allocator.free(errorOutPtr);
    }
  }

  void getStatusSnapshotAsync({
    required int chainHandle,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
  }) => _invokeAsyncNoArg(_getStatusSnapshotAsync,
      chainHandle: chainHandle, callbackId: callbackId,
      callback: callback, debugName: 'getStatusSnapshotAsync');

  void getRuntimeVersionAsync({
    required int chainHandle,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
  }) => _invokeAsyncNoArg(_getRuntimeVersionAsync,
      chainHandle: chainHandle, callbackId: callbackId,
      callback: callback, debugName: 'getRuntimeVersionAsync');

  void getMetadataAsync({
    required int chainHandle,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
  }) => _invokeAsyncNoArg(_getMetadataAsync,
      chainHandle: chainHandle, callbackId: callbackId,
      callback: callback, debugName: 'getMetadataAsync');

  void getAccountNextIndexAsync({
    required int chainHandle,
    required String accountIdHex,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
  }) => _invokeAsyncOneArg(_getAccountNextIndexAsync,
      chainHandle: chainHandle, arg: accountIdHex, callbackId: callbackId,
      callback: callback, debugName: 'getAccountNextIndexAsync');

  void getBlockHashAsync({
    required int chainHandle,
    required String blockNumber,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
  }) => _invokeAsyncOneArg(_getBlockHashAsync,
      chainHandle: chainHandle, arg: blockNumber, callbackId: callbackId,
      callback: callback, debugName: 'getBlockHashAsync');

  void getBlockExtrinsicsAsync({
    required int chainHandle,
    required String blockHashHex,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
  }) => _invokeAsyncOneArg(_getBlockExtrinsicsAsync,
      chainHandle: chainHandle, arg: blockHashHex, callbackId: callbackId,
      callback: callback, debugName: 'getBlockExtrinsicsAsync');

  void submitExtrinsicAsync({
    required int chainHandle,
    required String extrinsicHex,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
  }) => _invokeAsyncOneArg(_submitExtrinsicAsync,
      chainHandle: chainHandle, arg: extrinsicHex, callbackId: callbackId,
      callback: callback, debugName: 'submitExtrinsicAsync');

  void getSystemAccountAsync({
    required int chainHandle,
    required String accountIdHex,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
  }) => _invokeAsyncOneArg(_getSystemAccountAsync,
      chainHandle: chainHandle, arg: accountIdHex, callbackId: callbackId,
      callback: callback, debugName: 'getSystemAccountAsync');

  void getStorageValueAsync({
    required int chainHandle,
    required String storageKeyHex,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
  }) => _invokeAsyncOneArg(_getStorageValueAsync,
      chainHandle: chainHandle, arg: storageKeyHex, callbackId: callbackId,
      callback: callback, debugName: 'getStorageValueAsync');

  void getStorageValuesAsync({
    required int chainHandle,
    required String storageKeysJson,
    required int callbackId,
    required Pointer<NativeFunction<DartCallbackNative>> callback,
  }) => _invokeAsyncOneArg(_getStorageValuesAsync,
      chainHandle: chainHandle, arg: storageKeysJson, callbackId: callbackId,
      callback: callback, debugName: 'getStorageValuesAsync');
}
