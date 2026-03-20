import 'dart:convert';
import 'dart:io';

import 'package:http/http.dart' as http;

class HealthStatus {
  const HealthStatus({
    required this.service,
    required this.version,
    required this.status,
  });

  final String service;
  final String version;
  final String status;
}

/// SFID 人口快照响应。
class PopulationSnapshotResponse {
  const PopulationSnapshotResponse({
    required this.eligibleTotal,
    required this.snapshotNonce,
    required this.snapshotSignature,
    required this.who,
    required this.asOf,
  });

  final int eligibleTotal;

  /// 快照 nonce（UTF-8 字符串，直接作为 BoundedVec<u8> 提交）。
  final String snapshotNonce;

  /// sr25519 签名（hex 编码，提交时需解码为原始字节）。
  final String snapshotSignature;

  /// 归一化后的账户公钥 hex。
  final String who;

  /// 快照生成时间（Unix 秒）。
  final int asOf;
}

class ApiClient {
  ApiClient({String? baseUrl})
      : _baseUrl = baseUrl ?? _defaultBaseUrl;

  final String _baseUrl;

  static String get _defaultBaseUrl {
    const fromDefine = String.fromEnvironment('WUMINAPP_API_BASE_URL');
    if (fromDefine.isNotEmpty) {
      return fromDefine;
    }
    return 'https://sfid.wuminapp.com';
  }

  Map<String, String> _headers({
    bool includeContentType = false,
  }) {
    final out = <String, String>{};
    if (includeContentType) {
      out['Content-Type'] = 'application/json';
    }
    return out;
  }

  Future<HealthStatus> fetchHealth() async {
    final uri = Uri.parse('$_baseUrl/api/v1/health');
    final response = await http.get(uri, headers: _headers());
    if (response.statusCode != 200) {
      throw Exception('health check failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final data = payload['data'] as Map<String, dynamic>;
    return HealthStatus(
      service: data['service'] as String? ?? '-',
      version: data['version'] as String? ?? '-',
      status: data['status'] as String? ?? '-',
    );
  }

  Future<void> requestChainBindByPubkey(String pubkeyHex) async {
    final normalized = _normalizePubkeyHex(pubkeyHex);
    final uri = Uri.parse('$_baseUrl/api/v1/app/bind/request');
    http.Response response;
    try {
      response = await http.post(
        uri,
        headers: _headers(includeContentType: true),
        body: jsonEncode({'account_pubkey': normalized}),
      );
    } on SocketException catch (_) {
      if ((Platform.isAndroid || Platform.isIOS) &&
          _baseUrl.contains('127.0.0.1')) {
        throw Exception(
          '当前使用$_baseUrl，手机真机无法访问本机回环地址。请用 --dart-define=WUMINAPP_API_BASE_URL=http://<电脑局域网IP>:8787',
        );
      }
      rethrow;
    }
    if (response.statusCode != 200) {
      throw Exception('chain bind request failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception(
        'chain bind request rejected: code=$code message=$message',
      );
    }
  }

  String _normalizePubkeyHex(String value) {
    final trimmed = value.trim();
    if (trimmed.isEmpty) {
      throw Exception('pubkey is empty');
    }
    return trimmed.startsWith('0x') ? trimmed : '0x$trimmed';
  }

  /// 获取公民人口快照（eligible_total + nonce + signature）。
  ///
  /// 用于联合投票提案创建时附带人口证明。
  Future<PopulationSnapshotResponse> fetchPopulationSnapshot(
      String accountPubkeyHex) async {
    final normalized = _normalizePubkeyHex(accountPubkeyHex);
    final uri = Uri.parse(
        '$_baseUrl/api/v1/app/voters/count?account_pubkey=$normalized');
    http.Response response;
    try {
      response = await http.get(
        uri,
        headers: _headers(),
      );
    } on SocketException catch (_) {
      if ((Platform.isAndroid || Platform.isIOS) &&
          _baseUrl.contains('127.0.0.1')) {
        throw Exception(
          '当前使用$_baseUrl，手机真机无法访问本机回环地址。请用 --dart-define=WUMINAPP_API_BASE_URL=http://<电脑局域网IP>:8787',
        );
      }
      rethrow;
    }
    if (response.statusCode != 200) {
      throw Exception('population snapshot failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception(
        'population snapshot rejected: code=$code message=$message',
      );
    }

    final data = payload['data'];
    if (data is! Map<String, dynamic>) {
      throw Exception('population snapshot invalid response: missing data');
    }

    return PopulationSnapshotResponse(
      eligibleTotal: (data['eligible_total'] as num?)?.toInt() ?? 0,
      snapshotNonce: (data['snapshot_nonce']?.toString() ?? '').trim(),
      snapshotSignature:
          (data['snapshot_signature']?.toString() ?? '').trim(),
      who: (data['who']?.toString() ?? '').trim(),
      asOf: (data['as_of'] as num?)?.toInt() ?? 0,
    );
  }

  /// 从 SFID 获取公民投票凭证。
  ///
  /// 公民投票时，App 先从 SFID 获取投票资格证明（sfid_hash + vote_nonce + signature），
  /// 再将凭证提交到链上。
  Future<VoteCredentialResponse> fetchVoteCredential(
      String accountPubkeyHex, int proposalId) async {
    final normalized = _normalizePubkeyHex(accountPubkeyHex);
    final uri = Uri.parse('$_baseUrl/api/v1/app/vote/credential');
    final body = jsonEncode({
      'account_pubkey': normalized,
      'proposal_id': proposalId,
    });
    http.Response response;
    try {
      response = await http.post(
        uri,
        headers: _headers(includeContentType: true),
        body: body,
      );
    } on SocketException catch (_) {
      if ((Platform.isAndroid || Platform.isIOS) &&
          _baseUrl.contains('127.0.0.1')) {
        throw Exception(
          '当前使用$_baseUrl，手机真机无法访问本机回环地址。请用 --dart-define=WUMINAPP_API_BASE_URL=http://<电脑局域网IP>:8787',
        );
      }
      rethrow;
    }
    if (response.statusCode != 200) {
      throw Exception('vote credential failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception(
        'vote credential rejected: code=$code message=$message',
      );
    }

    final data = payload['data'];
    if (data is! Map<String, dynamic>) {
      throw Exception('vote credential invalid response: missing data');
    }

    return VoteCredentialResponse(
      accountPubkey: (data['account_pubkey']?.toString() ?? '').trim(),
      isBound: data['is_bound'] as bool? ?? false,
      hasVoteEligibility: data['has_vote_eligibility'] as bool? ?? false,
      sfidHash: data['sfid_hash']?.toString(),
      proposalId: (data['proposal_id'] as num?)?.toInt() ?? proposalId,
      voteNonce: data['vote_nonce']?.toString(),
      voteSignature: data['vote_signature']?.toString(),
      message: data['message']?.toString() ?? '',
    );
  }
}

class VoteCredentialResponse {
  final String accountPubkey;
  final bool isBound;
  final bool hasVoteEligibility;
  final String? sfidHash;
  final int proposalId;
  final String? voteNonce;
  final String? voteSignature;
  final String message;

  VoteCredentialResponse({
    required this.accountPubkey,
    required this.isBound,
    required this.hasVoteEligibility,
    this.sfidHash,
    required this.proposalId,
    this.voteNonce,
    this.voteSignature,
    required this.message,
  });
}
