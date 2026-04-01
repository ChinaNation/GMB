import 'package:meta/meta.dart';

/// Configuration options for initializing the smoldot client
@immutable
class SmoldotConfig {
  /// Maximum log level to output (0=off, 1=error, 2=warn, 3=info, 4=debug, 5=trace)
  final int maxLogLevel;

  /// Maximum number of chains that can be simultaneously added
  final int maxChains;

  /// CPU rate limit (0.0 to 1.0, where 1.0 means no limit)
  final double cpuRateLimit;

  /// Enable WebAssembly CPU metering
  final bool wasmCpuMetering;

  const SmoldotConfig({
    this.maxLogLevel = 3,
    this.maxChains = 8,
    this.cpuRateLimit = 1.0,
    this.wasmCpuMetering = false,
  });

  Map<String, dynamic> toJson() => {
        'maxLogLevel': maxLogLevel,
        'maxChains': maxChains,
        'cpuRateLimit': cpuRateLimit,
        'wasmCpuMetering': wasmCpuMetering,
      };
}

/// Configuration for adding a chain to the smoldot client
@immutable
class AddChainConfig {
  /// Chain specification in JSON format
  final String chainSpec;

  /// Optional database content to restore chain state
  final String? databaseContent;

  /// Potential relay chains this chain can connect to (chain handles)
  final List<int>? potentialRelayChains;

  /// Disable JSON-RPC for this chain
  final bool disableJsonRpc;

  const AddChainConfig({
    required this.chainSpec,
    this.databaseContent,
    this.potentialRelayChains,
    this.disableJsonRpc = false,
  });

  Map<String, dynamic> toJson() => {
        'chainSpec': chainSpec,
        if (databaseContent != null) 'databaseContent': databaseContent,
        if (potentialRelayChains != null && potentialRelayChains!.isNotEmpty)
          'potentialRelayChains': potentialRelayChains,
        'disableJsonRpc': disableJsonRpc,
      };
}

/// Result of a JSON-RPC request
@immutable
class JsonRpcResponse {
  /// Request ID
  final String id;

  /// Result value if successful
  final dynamic result;

  /// Error information if failed
  final JsonRpcError? error;

  const JsonRpcResponse({
    required this.id,
    this.result,
    this.error,
  });

  bool get isError => error != null;
  bool get isSuccess => error == null;

  factory JsonRpcResponse.fromJson(Map<String, dynamic> json) {
    return JsonRpcResponse(
      id: json['id']?.toString() ?? '',
      result: json['result'],
      error: json['error'] != null
          ? JsonRpcError.fromJson(json['error'] as Map<String, dynamic>)
          : null,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        if (result != null) 'result': result,
        if (error != null) 'error': error!.toJson(),
      };
}

/// JSON-RPC error information
@immutable
class JsonRpcError {
  /// Error code
  final int code;

  /// Error message
  final String message;

  /// Additional error data
  final dynamic data;

  const JsonRpcError({
    required this.code,
    required this.message,
    this.data,
  });

  factory JsonRpcError.fromJson(Map<String, dynamic> json) {
    return JsonRpcError(
      code: json['code'] as int? ?? 0,
      message: json['message'] as String? ?? '',
      data: json['data'],
    );
  }

  Map<String, dynamic> toJson() => {
        'code': code,
        'message': message,
        if (data != null) 'data': data,
      };

  @override
  String toString() => 'JsonRpcError(code: $code, message: $message)';
}

/// Chain status information
enum ChainStatus {
  /// Chain is syncing
  syncing,

  /// Chain is synced
  synced,

  /// Chain has encountered an error
  error,
}

/// Chain information
@immutable
class ChainInfo {
  /// Chain ID (handle from Rust)
  final int chainId;

  /// Chain name
  final String name;

  /// Chain status
  final ChainStatus status;

  /// Number of peers connected
  final int peerCount;

  /// Current best block number
  final int? bestBlockNumber;

  /// Current best block hash
  final String? bestBlockHash;

  const ChainInfo({
    required this.chainId,
    required this.name,
    required this.status,
    this.peerCount = 0,
    this.bestBlockNumber,
    this.bestBlockHash,
  });

  Map<String, dynamic> toJson() => {
        'chainId': chainId,
        'name': name,
        'status': status.name,
        'peerCount': peerCount,
        if (bestBlockNumber != null) 'bestBlockNumber': bestBlockNumber,
        if (bestBlockHash != null) 'bestBlockHash': bestBlockHash,
      };
}

/// 中文注释：轻节点状态快照，作为 typed capability 的第一批结构化输出。
@immutable
class LightClientStatusSnapshot {
  final int peerCount;
  final bool isSyncing;
  final int? bestBlockNumber;
  final String? bestBlockHash;
  final int? finalizedBlockNumber;
  final String? finalizedBlockHash;

  const LightClientStatusSnapshot({
    required this.peerCount,
    required this.isSyncing,
    this.bestBlockNumber,
    this.bestBlockHash,
    this.finalizedBlockNumber,
    this.finalizedBlockHash,
  });

  bool get hasPeers => peerCount > 0;

  bool get isUsable =>
      hasPeers && !isSyncing && finalizedBlockHash != null && finalizedBlockHash!.isNotEmpty;

  Map<String, dynamic> toJson() => {
        'peerCount': peerCount,
        'isSyncing': isSyncing,
        if (bestBlockNumber != null) 'bestBlockNumber': bestBlockNumber,
        if (bestBlockHash != null) 'bestBlockHash': bestBlockHash,
        if (finalizedBlockNumber != null)
          'finalizedBlockNumber': finalizedBlockNumber,
        if (finalizedBlockHash != null) 'finalizedBlockHash': finalizedBlockHash,
      };

  factory LightClientStatusSnapshot.fromJson(Map<String, dynamic> json) {
    return LightClientStatusSnapshot(
      peerCount: json['peerCount'] as int? ?? 0,
      isSyncing: json['isSyncing'] as bool? ?? false,
      bestBlockNumber: json['bestBlockNumber'] as int?,
      bestBlockHash: json['bestBlockHash'] as String?,
      finalizedBlockNumber: json['finalizedBlockNumber'] as int?,
      finalizedBlockHash: json['finalizedBlockHash'] as String?,
    );
  }
}

/// 中文注释：`System.Account` 的原生读取结果，后续钱包余额迁移直接基于该结构。
@immutable
class SystemAccountSnapshot {
  final String storageKey;
  final bool exists;
  final String? valueHex;
  final int? nonce;
  final BigInt? freeFen;

  const SystemAccountSnapshot({
    required this.storageKey,
    required this.exists,
    this.valueHex,
    this.nonce,
    this.freeFen,
  });

  double? get freeYuan =>
      freeFen == null ? null : freeFen!.toDouble() / 100.0;

  Map<String, dynamic> toJson() => {
        'storageKey': storageKey,
        'exists': exists,
        if (valueHex != null) 'valueHex': valueHex,
        if (nonce != null) 'nonce': nonce,
        if (freeFen != null) 'freeFen': freeFen.toString(),
      };

  factory SystemAccountSnapshot.fromJson(Map<String, dynamic> json) {
    return SystemAccountSnapshot(
      storageKey: json['storageKey'] as String? ?? '',
      exists: json['exists'] as bool? ?? false,
      valueHex: json['valueHex'] as String?,
      nonce: json['nonce'] as int?,
      freeFen: json['freeFen'] == null
          ? null
          : BigInt.parse(json['freeFen'].toString()),
    );
  }
}

/// Log level enumeration
enum LogLevel {
  /// No logs
  off(0),

  /// Error logs only
  error(1),

  /// Warning and error logs
  warn(2),

  /// Info, warning and error logs
  info(3),

  /// Debug and all lower level logs
  debug(4),

  /// All logs including trace
  trace(5);

  const LogLevel(this.value);
  final int value;
}

/// Log message from smoldot
@immutable
class LogMessage {
  /// Log level
  final LogLevel level;

  /// Log message
  final String message;

  /// Target component
  final String target;

  /// Timestamp
  final DateTime timestamp;

  const LogMessage({
    required this.level,
    required this.message,
    required this.target,
    required this.timestamp,
  });

  factory LogMessage.fromJson(Map<String, dynamic> json) {
    return LogMessage(
      level: LogLevel.values[json['level'] as int],
      message: json['message'] as String,
      target: json['target'] as String,
      timestamp: DateTime.parse(json['timestamp'] as String),
    );
  }

  @override
  String toString() => '[$level] $target: $message';
}

/// Exception thrown when smoldot operations fail
class SmoldotException implements Exception {
  final String message;
  final String? details;
  final StackTrace? stackTrace;

  SmoldotException(this.message, {this.details, this.stackTrace});

  @override
  String toString() {
    final buffer = StringBuffer('SmoldotException: $message');
    if (details != null) {
      buffer.write('\nDetails: $details');
    }
    return buffer.toString();
  }
}

/// Exception thrown when FFI operations fail
class SmoldotFfiException extends SmoldotException {
  SmoldotFfiException(super.message, {super.details, super.stackTrace});
}

/// Exception thrown when chain operations fail
class ChainException extends SmoldotException {
  final int chainId;

  ChainException(this.chainId, super.message,
      {super.details, super.stackTrace});

  @override
  String toString() => 'ChainException[$chainId]: $message';
}

/// Exception thrown when JSON-RPC operations fail
class JsonRpcException extends SmoldotException {
  final JsonRpcError? error;

  JsonRpcException(super.message,
      {this.error, super.details, super.stackTrace});

  @override
  String toString() {
    if (error != null) {
      return 'JsonRpcException: ${error.toString()}';
    }
    return 'JsonRpcException: $message';
  }
}
