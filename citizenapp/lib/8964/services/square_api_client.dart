import 'dart:convert';

import 'package:http/http.dart' as http;

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/profile/models/citizen_profile.dart';

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

  /// 线上 Worker 唯一默认地址：聊天 mailbox 与广场共用同一个 Cloudflare Worker。
  /// 默认即连生产 Cloudflare，绝不回落本机；开发者要连本机 wrangler dev 时，
  /// 显式传 --dart-define=CITIZENAPP_SQUARE_API_BASE_URL=http://127.0.0.1:8787。
  static const prodBaseUrl =
      'https://citizenapp-square-api.stews87-fawn.workers.dev';

  static const _configuredBaseUrl = String.fromEnvironment(baseUrlDefineName);

  static String get defaultBaseUrl {
    if (_configuredBaseUrl.trim().isNotEmpty) {
      return normalizeBaseUrl(_configuredBaseUrl);
    }
    return prodBaseUrl;
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

  /// 把 R2 object_key 拼成公开媒体读取 URL，供 `Image.network` 与 CDN 使用。
  String mediaUrl(String objectKey) {
    final encoded = objectKey.split('/').map(Uri.encodeComponent).join('/');
    return '$baseUrl/v1/square/media/$encoded';
  }

  /// 拉取某账户的用户主页资料（公开可读；带 session 时附带 is_following）。
  Future<CitizenProfile> fetchUserProfile({
    required String ownerAccount,
    SquareSession? session,
  }) async {
    final data = await _getJson(
      '/v1/square/users/${Uri.encodeComponent(ownerAccount)}',
      session: session,
    );
    final profile = data['profile'];
    if (profile is! Map<String, dynamic>) {
      throw const SquareApiException('用户主页响应缺少资料数据');
    }
    return CitizenProfile.fromJson(profile);
  }

  /// 按作者分页拉帖。[category]/[contentFormat] 为空表示不过滤；[cursor] 为上一页 nextCursor。
  Future<({List<SquarePost> posts, int? nextCursor})> fetchAuthorPosts({
    required String ownerAccount,
    SquarePostCategory? category,
    SquarePostContentFormat? contentFormat,
    int limit = 20,
    int? cursor,
    SquareSession? session,
  }) async {
    final params = <String, String>{'limit': '$limit'};
    if (category != null) {
      params['category'] = category.workerValue;
    }
    if (contentFormat != null) {
      params['content_format'] = contentFormat.workerValue;
    }
    if (cursor != null) {
      params['cursor'] = '$cursor';
    }
    final query = params.entries
        .map((entry) => '${entry.key}=${Uri.encodeQueryComponent(entry.value)}')
        .join('&');
    final data = await _getJson(
      '/v1/square/users/${Uri.encodeComponent(ownerAccount)}/posts?$query',
      session: session,
    );
    final posts = data['posts'];
    if (posts is! List) {
      throw const SquareApiException('用户主页响应缺少动态列表');
    }
    return (
      posts: posts.whereType<Map<String, dynamic>>().map(_parsePost).toList(
            growable: false,
          ),
      nextCursor: _nullableInt(data['next_cursor']),
    );
  }

  /// 申请头像/背景上传授权：返回 object_key、内容哈希与短期上传 URL。
  Future<({String objectKey, String contentHash, String uploadUrl})>
      prepareProfileAsset({
    required SquareSession session,
    required String kind,
    required String contentType,
    required int byteSize,
    required String sha256Hex,
  }) async {
    final data = await _postJson(
      '/v1/square/profile/assets/prepare',
      {
        'kind': kind,
        'content_type': contentType,
        'byte_size': byteSize,
        'sha256': sha256Hex,
      },
      session: session,
    );
    return (
      objectKey: _requireString(data, 'object_key'),
      contentHash: _requireString(data, 'content_hash'),
      uploadUrl: _requireString(data, 'upload_url'),
    );
  }

  /// 把字节 PUT 到上传 URL。dev-put 同源需 Bearer；生产预签名 URL 绝不能带 Authorization。
  Future<void> uploadBytesTo(
    String uploadUrl,
    List<int> bytes,
    String contentType, {
    SquareSession? session,
  }) async {
    final uri = Uri.parse(uploadUrl);
    final headers = <String, String>{'content-type': contentType};
    if (session != null && uri.origin == baseUri.origin) {
      headers['authorization'] = 'Bearer ${session.sessionToken}';
    }
    final response = await _http
        .put(uri, headers: headers, body: bytes)
        .timeout(const Duration(seconds: 60));
    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw SquareApiException(
        '资源上传失败：${response.statusCode}',
        statusCode: response.statusCode,
      );
    }
  }

  /// 更新本人公开资料（仅传要改的字段；owner 由 Worker 从 session 派生）。
  Future<CitizenProfile> updateProfile({
    required SquareSession session,
    String? displayName,
    String? bio,
    String? avatarObjectKey,
    String? avatarContentHash,
    String? bannerObjectKey,
    String? bannerContentHash,
  }) async {
    final body = <String, Object?>{
      if (displayName != null) 'display_name': displayName,
      if (bio != null) 'bio': bio,
      if (avatarObjectKey != null) 'avatar_object_key': avatarObjectKey,
      if (avatarContentHash != null) 'avatar_content_hash': avatarContentHash,
      if (bannerObjectKey != null) 'banner_object_key': bannerObjectKey,
      if (bannerContentHash != null) 'banner_content_hash': bannerContentHash,
    };
    final data = await _putJson('/v1/square/profile', body, session: session);
    final profile = data['profile'];
    if (profile is! Map<String, dynamic>) {
      throw const SquareApiException('更新资料响应缺少资料数据');
    }
    return CitizenProfile.fromJson(profile);
  }

  /// 关注一个账户（写接口带 session；owner 由 Worker 从 session 派生）。
  Future<void> followUser({
    required SquareSession session,
    required String followedAccount,
  }) async {
    await _postJson(
      '/v1/square/follows',
      {'followed_account': followedAccount},
      session: session,
    );
  }

  /// 取消关注一个账户。
  Future<void> unfollowUser({
    required SquareSession session,
    required String followedAccount,
  }) async {
    await _deleteJson(
      '/v1/square/follows/${Uri.encodeComponent(followedAccount)}',
      session: session,
    );
  }

  /// 拉取关注/粉丝列表。
  Future<({List<SquareFollowEntry> accounts, int? nextCursor})> fetchFollows({
    required String ownerAccount,
    required String type,
    int limit = 20,
    int? cursor,
    SquareSession? session,
  }) async {
    final params = <String, String>{'type': type, 'limit': '$limit'};
    if (cursor != null) {
      params['cursor'] = '$cursor';
    }
    final query = params.entries
        .map((entry) => '${entry.key}=${Uri.encodeQueryComponent(entry.value)}')
        .join('&');
    final data = await _getJson(
      '/v1/square/users/${Uri.encodeComponent(ownerAccount)}/follows?$query',
      session: session,
    );
    final accounts = data['accounts'];
    if (accounts is! List) {
      throw const SquareApiException('关注列表响应缺少账户列表');
    }
    return (
      accounts: accounts
          .whereType<Map<String, dynamic>>()
          .map(SquareFollowEntry.fromJson)
          .toList(growable: false),
      nextCursor: _nullableInt(data['next_cursor']),
    );
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

  Future<Map<String, dynamic>> _putJson(
    String path,
    Map<String, Object?> body, {
    SquareSession? session,
  }) async {
    final response = await _http
        .put(
          _uri(path),
          headers: _headers(session),
          body: jsonEncode(body),
        )
        .timeout(const Duration(seconds: 20));
    return _decodeResponse(response);
  }

  Future<Map<String, dynamic>> _deleteJson(
    String path, {
    SquareSession? session,
  }) async {
    final response = await _http
        .delete(_uri(path), headers: _headers(session))
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
      contentFormat: data['content_format'] == 'article'
          ? SquarePostContentFormat.article
          : SquarePostContentFormat.normal,
      title: (data['title']?.toString().trim().isNotEmpty ?? false)
          ? data['title'].toString().trim()
          : null,
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
      // 竞选目标（预留）：Worker 暂未返回，待公民身份上链落地后填充。
      campaignInstitutionCid: data['campaign_institution_cid']?.toString(),
      campaignPosition: data['campaign_position']?.toString(),
    );
  }

  SquareMediaItem _parseMediaItem(Map<String, dynamic> data) {
    final objectKey =
        data['object_key']?.toString() ?? data['url']?.toString() ?? '';
    final coverKey = data['cover_object_key']?.toString() ??
        data['cover_url']?.toString() ??
        '';
    return SquareMediaItem(
      mediaKind: data['media_kind'] == 'video'
          ? SquareMediaKind.video
          : SquareMediaKind.image,
      url: objectKey.isEmpty ? '' : mediaUrl(objectKey),
      coverUrl: coverKey.isEmpty ? null : mediaUrl(coverKey),
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
