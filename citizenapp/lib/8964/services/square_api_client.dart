import 'dart:convert';

import 'package:http/http.dart' as http;

import 'package:citizenapp/8964/models/square_models.dart';

class SquareApiException implements Exception {
  const SquareApiException(this.message, {this.statusCode, this.errorCode});

  final String message;
  final int? statusCode;
  final String? errorCode;

  @override
  String toString() => message;
}

class SquareSession {
  const SquareSession({
    required this.sessionToken,
    required this.ownerAccount,
    required this.expiresAt,
  });

  final String sessionToken;
  final String ownerAccount;
  final int expiresAt;

  bool get isUsable => expiresAt > DateTime.now().millisecondsSinceEpoch;
}

class SquareMembershipState {
  const SquareMembershipState({
    required this.active,
    required this.expiresAt,
    required this.storageQuotaBytes,
    required this.storageUsedBytes,
  });

  final bool active;
  final int expiresAt;
  final int storageQuotaBytes;
  final int storageUsedBytes;
}

class SquareUploadMediaRequest {
  const SquareUploadMediaRequest({
    required this.mediaKind,
    required this.contentType,
    required this.byteSize,
    required this.fileExt,
  });

  final SquareMediaKind mediaKind;
  final String contentType;
  final int byteSize;
  final String fileExt;

  Map<String, Object?> toJson() => {
        'media_kind': mediaKind.workerValue,
        'content_type': contentType,
        'byte_size': byteSize,
        if (fileExt.isNotEmpty) 'file_ext': fileExt,
      };
}

class SquarePreparedMediaUpload {
  const SquarePreparedMediaUpload({
    required this.mediaKind,
    required this.contentType,
    required this.byteSize,
    required this.objectKey,
    required this.uploadUrl,
  });

  final SquareMediaKind mediaKind;
  final String contentType;
  final int byteSize;
  final String objectKey;
  final String uploadUrl;
}

class SquarePreparedUpload {
  const SquarePreparedUpload({
    required this.uploadId,
    required this.postId,
    required this.storageReceiptId,
    required this.expiresAt,
    required this.estimatedBytes,
    required this.manifestObjectKey,
    required this.manifestUploadUrl,
    required this.mediaItems,
  });

  final String uploadId;
  final String postId;
  final String storageReceiptId;
  final int expiresAt;
  final int estimatedBytes;
  final String manifestObjectKey;
  final String manifestUploadUrl;
  final List<SquarePreparedMediaUpload> mediaItems;
}

class SquareCompletedUpload {
  const SquareCompletedUpload({
    required this.uploadId,
    required this.postId,
    required this.contentHash,
    required this.storageReceiptId,
  });

  final String uploadId;
  final String postId;
  final String contentHash;
  final String storageReceiptId;
}

abstract class SquareFeedSource {
  Future<List<SquarePost>> fetchFeed({
    required SquareFeedKind feedKind,
    int limit,
    SquareSession? session,
  });
}

abstract class SquarePublicationConfirmer {
  Future<SquarePost> confirmPublishedPost({
    required SquareSession session,
    required String postId,
    required String blockHashHex,
    required String txHash,
  });
}

typedef SquareLoginSigner = Future<String> Function(String signingPayload);

class SquareApiConfig {
  const SquareApiConfig._();

  static const baseUrlDefineName = 'CITIZENAPP_SQUARE_API_BASE_URL';
  static const localDevBaseUrl = 'http://127.0.0.1:8787';

  static const _configuredBaseUrl = String.fromEnvironment(baseUrlDefineName);
  static const _isProductBuild = bool.fromEnvironment('dart.vm.product');

  static String get defaultBaseUrl {
    if (_configuredBaseUrl.trim().isNotEmpty) {
      return normalizeBaseUrl(_configuredBaseUrl);
    }
    if (_isProductBuild) {
      throw UnsupportedError(
        '生产广场 API 地址必须通过 --dart-define=$baseUrlDefineName=https://... 显式提供',
      );
    }
    return localDevBaseUrl;
  }

  static String normalizeBaseUrl(String value) {
    final trimmed = value.trim().replaceFirst(RegExp(r'/+$'), '');
    final uri = Uri.tryParse(trimmed);
    if (trimmed.isEmpty || uri == null || !uri.hasScheme || uri.host.isEmpty) {
      throw UnsupportedError('$baseUrlDefineName 必须是完整的 Worker API URL');
    }
    final isLocalHttp = uri.scheme == 'http' &&
        (uri.host == '127.0.0.1' ||
            uri.host == 'localhost' ||
            uri.host == '::1');
    if (uri.scheme != 'https' && !isLocalHttp) {
      throw UnsupportedError(
          '$baseUrlDefineName 只允许 HTTPS，或本地调试 http://127.0.0.1');
    }
    return trimmed;
  }
}

class SquareApiClient implements SquareFeedSource, SquarePublicationConfirmer {
  SquareApiClient({
    String? baseUrl,
    http.Client? httpClient,
  })  : baseUrl = SquareApiConfig.normalizeBaseUrl(
          baseUrl ?? SquareApiConfig.defaultBaseUrl,
        ),
        _http = httpClient ?? http.Client();

  static String get defaultBaseUrl => SquareApiConfig.defaultBaseUrl;

  final String baseUrl;
  final http.Client _http;
  final Map<String, SquareSession> _sessions = {};

  /// Worker API 根地址。IM Cloudflare mailbox 复用同一个 Worker 登录态。
  Uri get baseUri => Uri.parse(baseUrl);

  Future<SquareSession> ensureSession({
    required String ownerAccount,
    required SquareLoginSigner signLoginPayload,
  }) async {
    final cached = _sessions[ownerAccount];
    if (cached != null && cached.isUsable) return cached;

    final challenge = await _postJson('/v1/square/auth/challenge', {
      'owner_account': ownerAccount,
    });
    final signingPayload = challenge['signing_payload'];
    final challengeId = challenge['challenge_id'];
    if (signingPayload is! String || challengeId is! String) {
      throw const SquareApiException('广场登录挑战响应不完整');
    }

    final signature = await signLoginPayload(signingPayload);
    final session = await _postJson('/v1/square/auth/session', {
      'challenge_id': challengeId,
      'owner_account': ownerAccount,
      'signature': signature,
    });
    final token = session['session_token'];
    final expiresAt = session['expires_at'];
    if (token is! String || expiresAt is! int) {
      throw const SquareApiException('广场登录态响应不完整');
    }

    final next = SquareSession(
      sessionToken: token,
      ownerAccount: ownerAccount,
      expiresAt: expiresAt,
    );
    _sessions[ownerAccount] = next;
    return next;
  }

  Future<SquareMembershipState> fetchMembership(SquareSession session) async {
    final data = await _getJson(
      '/v1/square/membership',
      session: session,
    );
    final membership = data['membership'];
    final active = data['active'] == true;
    if (membership is! Map<String, dynamic>) {
      return const SquareMembershipState(
        active: false,
        expiresAt: 0,
        storageQuotaBytes: 0,
        storageUsedBytes: 0,
      );
    }
    return SquareMembershipState(
      active: active,
      expiresAt: _asInt(membership['expires_at']),
      storageQuotaBytes: _asInt(membership['storage_quota_bytes']),
      storageUsedBytes: _asInt(membership['storage_used_bytes']),
    );
  }

  Future<SquarePreparedUpload> prepareUpload({
    required SquareSession session,
    required SquarePostCategory postCategory,
    required String manifestHash,
    required List<SquareUploadMediaRequest> mediaItems,
  }) async {
    final data = await _postJson(
      '/v1/square/uploads/prepare',
      {
        'post_category': postCategory.workerValue,
        'manifest_hash': manifestHash,
        'media_items': mediaItems.map((item) => item.toJson()).toList(),
      },
      session: session,
    );
    final rawMediaItems = data['media_items'];
    if (rawMediaItems is! List) {
      throw const SquareApiException('上传准备响应缺少媒体对象列表');
    }
    return SquarePreparedUpload(
      uploadId: _requireString(data, 'upload_id'),
      postId: _requireString(data, 'post_id'),
      storageReceiptId: _requireString(data, 'storage_receipt_id'),
      expiresAt: _asInt(data['expires_at']),
      estimatedBytes: _asInt(data['estimated_bytes']),
      manifestObjectKey: _requireString(data, 'manifest_object_key'),
      manifestUploadUrl: _requireString(data, 'manifest_upload_url'),
      mediaItems: rawMediaItems
          .map((item) => _parsePreparedMedia(item as Map<String, dynamic>))
          .toList(growable: false),
    );
  }

  Future<void> uploadObject({
    required String uploadUrl,
    required String contentType,
    required int contentLength,
    required Stream<List<int>> body,
  }) async {
    final request = http.StreamedRequest('PUT', Uri.parse(uploadUrl))
      ..headers['content-type'] = contentType
      ..contentLength = contentLength;
    await request.sink.addStream(body);
    await request.sink.close();
    final response =
        await _http.send(request).timeout(const Duration(minutes: 10));
    if (response.statusCode < 200 || response.statusCode >= 300) {
      final text = await response.stream.bytesToString();
      throw SquareApiException(
        'R2 对象上传失败：${response.statusCode} $text',
        statusCode: response.statusCode,
      );
    }
  }

  Future<SquareCompletedUpload> completeUpload({
    required SquareSession session,
    required String uploadId,
    required String manifestHash,
    required String contentHash,
  }) async {
    final data = await _postJson(
      '/v1/square/uploads/complete',
      {
        'upload_id': uploadId,
        'manifest_hash': manifestHash,
        'content_hash': contentHash,
      },
      session: session,
    );
    return SquareCompletedUpload(
      uploadId: _requireString(data, 'upload_id'),
      postId: _requireString(data, 'post_id'),
      contentHash: _requireString(data, 'content_hash'),
      storageReceiptId: _requireString(data, 'storage_receipt_id'),
    );
  }

  @override
  Future<SquarePost> confirmPublishedPost({
    required SquareSession session,
    required String postId,
    required String blockHashHex,
    required String txHash,
  }) async {
    final data = await _postJson(
      '/v1/square/posts/confirm',
      {
        'post_id': postId,
        'block_hash': blockHashHex,
        'tx_hash': txHash,
      },
      session: session,
    );
    final post = data['post'];
    if (post is! Map<String, dynamic>) {
      throw const SquareApiException('广场确认发布响应缺少动态数据');
    }
    return _parsePost(post);
  }

  @override
  Future<List<SquarePost>> fetchFeed({
    required SquareFeedKind feedKind,
    int limit = 20,
    SquareSession? session,
  }) async {
    final data = await _getJson(
      '/v1/square/feed/${feedKind.workerValue}?limit=$limit',
      session: session,
    );
    final posts = data['posts'];
    if (posts is! List) {
      throw const SquareApiException('广场 feed 响应缺少动态列表');
    }
    return posts
        .whereType<Map<String, dynamic>>()
        .map(_parsePost)
        .toList(growable: false);
  }

  Future<Map<String, dynamic>> _getJson(
    String path, {
    SquareSession? session,
  }) async {
    final response = await _http
        .get(_uri(path), headers: _headers(session))
        .timeout(const Duration(seconds: 20));
    return _decodeResponse(response);
  }

  Future<Map<String, dynamic>> _postJson(
    String path,
    Map<String, Object?> body, {
    SquareSession? session,
  }) async {
    final response = await _http
        .post(
          _uri(path),
          headers: _headers(session),
          body: jsonEncode(body),
        )
        .timeout(const Duration(seconds: 20));
    return _decodeResponse(response);
  }

  Uri _uri(String path) => Uri.parse('$baseUrl$path');

  Map<String, String> _headers(SquareSession? session) => {
        'content-type': 'application/json; charset=utf-8',
        if (session != null) 'authorization': 'Bearer ${session.sessionToken}',
      };

  Map<String, dynamic> _decodeResponse(http.Response response) {
    final dynamic decoded;
    try {
      decoded = jsonDecode(response.body);
    } catch (_) {
      throw SquareApiException(
        '广场服务响应不是 JSON：${response.statusCode}',
        statusCode: response.statusCode,
      );
    }
    if (decoded is! Map<String, dynamic>) {
      throw SquareApiException(
        '广场服务响应结构不合法：${response.statusCode}',
        statusCode: response.statusCode,
      );
    }
    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw SquareApiException(
        decoded['message']?.toString() ?? '广场服务请求失败',
        statusCode: response.statusCode,
        errorCode: decoded['error_code']?.toString(),
      );
    }
    return decoded;
  }

  SquarePreparedMediaUpload _parsePreparedMedia(Map<String, dynamic> item) {
    final mediaKind = switch (_requireString(item, 'media_kind')) {
      'video' => SquareMediaKind.video,
      _ => SquareMediaKind.image,
    };
    return SquarePreparedMediaUpload(
      mediaKind: mediaKind,
      contentType: _requireString(item, 'content_type'),
      byteSize: _asInt(item['byte_size']),
      objectKey: _requireString(item, 'object_key'),
      uploadUrl: _requireString(item, 'upload_url'),
    );
  }

  SquarePost _parsePost(Map<String, dynamic> data) {
    final mediaItems = data['media_items'];
    return SquarePost(
      postId: _requireString(data, 'post_id'),
      author: SquareAuthor(
        ownerAccount: _requireString(data, 'owner_account'),
        cidNumber: data['cid_number']?.toString(),
      ),
      postCategory: _parseCategory(data['post_category']),
      text: data['text']?.toString() ?? '',
      createdAt:
          DateTime.fromMillisecondsSinceEpoch(_asInt(data['created_at'])),
      mediaItems: mediaItems is List
          ? mediaItems
              .whereType<Map<String, dynamic>>()
              .map(_parseMediaItem)
              .toList(growable: false)
          : const <SquareMediaItem>[],
      contentHash: data['content_hash']?.toString(),
      storageReceiptId: data['storage_receipt_id']?.toString(),
      chainBlock: _nullableInt(data['chain_block']),
    );
  }

  SquareMediaItem _parseMediaItem(Map<String, dynamic> data) {
    return SquareMediaItem(
      mediaKind: data['media_kind'] == 'video'
          ? SquareMediaKind.video
          : SquareMediaKind.image,
      url: data['url']?.toString() ?? data['object_key']?.toString() ?? '',
      byteSize: _nullableInt(data['byte_size']),
    );
  }

  SquarePostCategory _parseCategory(Object? value) {
    return value == 'campaign'
        ? SquarePostCategory.campaign
        : SquarePostCategory.normal;
  }

  static String _requireString(Map<String, dynamic> data, String key) {
    final value = data[key];
    if (value is String && value.isNotEmpty) return value;
    throw SquareApiException('广场服务响应缺少 $key');
  }

  static int _asInt(Object? value) {
    if (value is int) return value;
    if (value is num) return value.toInt();
    return int.tryParse(value?.toString() ?? '') ?? 0;
  }

  static int? _nullableInt(Object? value) {
    if (value == null) return null;
    return _asInt(value);
  }

  void close() => _http.close();
}
