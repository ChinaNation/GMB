import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:http/http.dart' as http;

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/chat/chat_media_limits.dart';
import 'package:citizenapp/signer/signing.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart' show hexToBytes;
import 'package:citizenapp/8964/services/square_request_signer.dart';

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
    this.signRequest,
  });

  final String sessionToken;
  final String ownerAccount;
  final int expiresAt;
  final SquareDeviceSigner? signRequest;

  bool get isUsable => expiresAt > DateTime.now().millisecondsSinceEpoch;
}

/// 会员订阅态（ADR-036：与身份彻底解耦）。只描述付费订阅本身，不含任何链上身份信息；
/// 身份展示由电子护照（myid）单独负责。
class SquareMembershipState {
  const SquareMembershipState({
    required this.active,
    required this.paidUntil,
    this.membershipLevel,
    this.subscriptionStatus,
    this.subscriptionActive = false,
    this.lastChargedAt = 0,
    this.plans = const <SquareMembershipPlan>[],
  });

  final bool active;
  final int paidUntil;
  final String? membershipLevel;

  /// 订阅生命周期态（链上单源镜像）：`active`=自动续费授权有效 /
  /// `terminated`=到期扣款失败并终止 / `cancelled`=用户已签名取消。
  /// 按钮双态与横幅文案据此判定。
  final String? subscriptionStatus;

  /// 订阅是否已支付且未过期（worker `subscription_active`）。解耦后权益态即订阅态，
  /// [active] 与本字段等值；按钮双态与徽章勾均据此判定。
  final bool subscriptionActive;

  /// 最近一次真实扣款时间（毫秒）；与 [paidUntil] 组成会员卡当前已付周期展示。
  final int lastChargedAt;

  final List<SquareMembershipPlan> plans;

  SquareMembershipPlan? planForLevel(String? level) {
    if (level == null) return null;
    for (final plan in plans) {
      if (plan.membershipLevel == level) return plan;
    }
    return null;
  }

  SquareMembershipPlan? get activePlan =>
      active ? planForLevel(membershipLevel) : null;

  /// 有可展示的订阅起止窗口（已支付且起止时间齐备）。
  bool get hasSubscriptionWindow =>
      subscriptionActive && lastChargedAt > 0 && paidUntil > 0;
}

class SquareMembershipPlan {
  const SquareMembershipPlan({
    required this.membershipLevel,
    required this.displayName,
    required this.chatFileMaxBytes,
    required this.dynamicTextMaxChars,
    required this.dynamicImageQuality,
    required this.dynamicMaxImages,
    required this.dynamicVideoQuality,
    required this.dynamicMaxVideos,
    required this.dynamicMaxVideoSeconds,
    required this.dynamicMaxVideoBytes,
    required this.articleTitleMinChars,
    required this.articleTitleMaxChars,
    required this.articleBodyMaxChars,
    required this.articleCoverQuality,
    required this.articleImageQuality,
    required this.articleMaxImages,
    required this.monthlyImages,
    required this.monthlyVideoSeconds,
    required this.activeUploads,
  });

  final String membershipLevel;
  final String displayName;

  /// 聊天文件大小上限（字节，会员权益之一，ADR-036）：自由 10MB / 民主 100MB / 薪火 5GB。
  final int chatFileMaxBytes;
  final int dynamicTextMaxChars;
  final String dynamicImageQuality;
  final int dynamicMaxImages;
  final String dynamicVideoQuality;
  final int dynamicMaxVideos;
  final int dynamicMaxVideoSeconds;

  /// 单个动态视频体积上限（字节）：自由 40MB / 民主 1536MB / 薪火 8GB。
  final int dynamicMaxVideoBytes;
  final int articleTitleMinChars;
  final int articleTitleMaxChars;
  final int articleBodyMaxChars;
  final String articleCoverQuality;
  final String articleImageQuality;
  final int articleMaxImages;

  /// 订阅周期累计用量额度（每月）：图片张数 / 视频总秒数 / 并发上传数。
  final int monthlyImages;
  final int monthlyVideoSeconds;
  final int activeUploads;

  /// 大文件（>100MB）中转仅薪火可用（后端 relay.ts 强制）。
  bool get supportsLargeFileRelay => membershipLevel == 'spark';

  /// 提炼展示用短串（卡片与详情页共用，杜绝口径漂移）。
  String get chatFileSizeLabel => _fileSize(chatFileMaxBytes);
  String get dynamicImageQualityLabel => _quality(dynamicImageQuality);
  String get dynamicVideoQualityLabel => _quality(dynamicVideoQuality);
  String get dynamicVideoDurationLabel => _duration(dynamicMaxVideoSeconds);
  String get dynamicVideoBytesLabel => _fileSize(dynamicMaxVideoBytes);
  String get articleImageQualityLabel => _quality(articleImageQuality);
  String get articleCoverQualityLabel => _quality(articleCoverQuality);
  String get monthlyVideoDurationLabel => _duration(monthlyVideoSeconds);

  String get chatFileLabel => '聊天文件：单个 ≤ ${_fileSize(chatFileMaxBytes)}';

  String get dynamicLabel =>
      '动态：$dynamicTextMaxChars 字、$dynamicMaxImages 张${_quality(dynamicImageQuality)}图片、$dynamicMaxVideos 个${_duration(dynamicMaxVideoSeconds)}${_quality(dynamicVideoQuality)}视频';

  String get articleLabel =>
      '文章：$articleBodyMaxChars 字、$articleMaxImages 张${_quality(articleImageQuality)}图片、1 张${_quality(articleCoverQuality)}首图、标题 $articleTitleMinChars-$articleTitleMaxChars 字';

  static String _quality(String value) => value == 'hd' ? '高清' : '标清';

  static String _duration(int seconds) {
    if (seconds >= 3600) return '${seconds ~/ 3600} 小时';
    if (seconds >= 60) return '${seconds ~/ 60} 分钟';
    return '$seconds 秒';
  }

  static String _fileSize(int bytes) {
    const mib = 1024 * 1024;
    if (bytes >= 1024 * mib) {
      // 非整 GB 保留一位小数，避免 1536MB 被四舍五入成误导性的「2GB」。
      final gb = bytes / (1024 * mib);
      return gb == gb.roundToDouble()
          ? '${gb.round()}GB'
          : '${gb.toStringAsFixed(1)}GB';
    }
    return '${(bytes / mib).round()}MB';
  }
}

class SquareUploadMediaRequest {
  const SquareUploadMediaRequest({
    required this.mediaKind,
    required this.contentType,
    required this.byteSize,
    required this.fileExt,
    this.durationSeconds,
  });

  final SquareMediaKind mediaKind;
  final String contentType;
  final int byteSize;
  final String fileExt;
  final int? durationSeconds;

  Map<String, Object?> toJson() => {
        'media_kind': mediaKind.workerValue,
        'content_type': contentType,
        'byte_size': byteSize,
        if (durationSeconds != null) 'duration_seconds': durationSeconds,
        if (fileExt.isNotEmpty) 'file_ext': fileExt,
      };
}

class SquarePreparedMediaUpload {
  const SquarePreparedMediaUpload({
    required this.mediaKind,
    required this.contentType,
    required this.byteSize,
    required this.provider,
    required this.providerAssetId,
    required this.uploadMethod,
    required this.uploadUrl,
  });

  final SquareMediaKind mediaKind;
  final String contentType;
  final int byteSize;
  final String provider;
  final String providerAssetId;
  final String uploadMethod;
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
    required this.storageState,
  });

  final String uploadId;
  final String postId;
  final String contentHash;
  final String storageReceiptId;
  final String storageState;
}

class SquareBrowseState {
  const SquareBrowseState({
    required this.browseDay,
    required this.browseCount,
    required this.browseLimit,
    required this.browseLeft,
  });

  final String browseDay;
  final int browseCount;
  final int? browseLimit;
  final int? browseLeft;
}

/// Cloudflare 只可见的单条通讯录密文信封。联系人账户和私人名称只存在于
/// [ciphertext] 内，Worker 不参与解密。
class SquareEncryptedContact {
  const SquareEncryptedContact({
    required this.contactId,
    required this.ciphertext,
    required this.nonce,
    required this.mac,
    required this.updatedAt,
  });

  final String contactId;
  final String ciphertext;
  final String nonce;
  final String mac;
  final int updatedAt;

  factory SquareEncryptedContact.fromJson(Map<String, dynamic> json) {
    return SquareEncryptedContact(
      contactId: json['contact_id']?.toString() ?? '',
      ciphertext: json['ciphertext']?.toString() ?? '',
      nonce: json['nonce']?.toString() ?? '',
      mac: json['mac']?.toString() ?? '',
      updatedAt: SquareApiClient._asInt(json['updated_at']),
    );
  }
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

abstract class SquarePostDeletionService {
  Future<void> deletePost({
    required SquareSession session,
    required String postId,
  });
}

/// 广场登录签名器：对 `signing_message(OP_SIGN_SQUARE_LOGIN)` 的 32 字节摘要
/// 做签名，返回 `0x` hex 签名。摘要由 [_establishSession] 统一构造（客户端钉死
/// op_tag，绝不采信服务端下发的 op_tag）。
typedef SquareLoginSigner = Future<String> Function(Uint8List loginMessage);

/// 账户敏感动作（注销/退订）签名器：对 `signing_message(OP_SIGN_SQUARE_ACTION)`
/// 的 32 字节摘要用 sr25519 **主钥**签名，返回 `0x` hex 签名（动钱动权，弹生物识别）。
typedef SquareActionSigner = Future<String> Function(Uint8List actionMessage);

class SquareApiConfig {
  const SquareApiConfig._();

  static const baseUrlDefineName = 'SQUARE_API_URL';

  /// 线上 Worker 唯一默认地址：聊天瞬时转发与广场共用同一个 Cloudflare Worker。
  /// 默认即连生产 Cloudflare，绝不回落本机；开发者要连本机 wrangler dev 时，
  /// 显式传 --dart-define=SQUARE_API_URL=http://127.0.0.1:8787。
  static const prodBaseUrl = 'https://www.crcfrcn.com/api';

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

class SquareApiClient
    implements
        SquareFeedSource,
        SquarePublicationConfirmer,
        SquarePostDeletionService {
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
  SquareBrowseState? lastBrowseState;
  final Map<String, SquareSession> _sessions = {};

  /// Worker API 根地址。Chat 瞬时转发复用同一个 Worker 登录态。
  Uri get baseUri => Uri.parse(baseUrl);

  Future<SquareSession> ensureSession({
    required String ownerAccount,
    required SquareLoginSigner signLoginPayload,
    Future<void> Function()? onDeviceNotRegistered,
  }) async {
    final cached = _sessions[ownerAccount];
    if (cached != null && cached.isUsable) return cached;

    try {
      return await _establishSession(ownerAccount, signLoginPayload);
    } on SquareApiException catch (e) {
      // 设备子钥未注册（首次 / 换机 / 重装）→ 懒注册后重试一次。
      if (e.errorCode != 'device_not_registered' ||
          onDeviceNotRegistered == null) {
        rethrow;
      }
      await onDeviceNotRegistered();
      return _establishSession(ownerAccount, signLoginPayload);
    }
  }

  Future<SquareSession> _establishSession(
    String ownerAccount,
    SquareLoginSigner signLoginPayload,
  ) async {
    final challenge = await _postJson('/v1/square/auth/challenge', {
      'owner_account': ownerAccount,
    });
    final signingPayloadHex = challenge['signing_payload_hex'];
    final challengeId = challenge['challenge_id'];
    if (signingPayloadHex is! String || challengeId is! String) {
      throw const SquareApiException('广场登录挑战响应不完整');
    }

    // 客户端钉死 op_tag（登录 = OP_SIGN_SQUARE_LOGIN），只对 worker 下发的 SCALE
    // payload 重算 signing_message 摘要后签名，杜绝服务端诱导跨域签名。
    final loginMessage = signingMessage(
      opTag: kOpSignSquareLogin,
      scalePayload: hexToBytes(signingPayloadHex),
    );
    final signature = await signLoginPayload(loginMessage);
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
      signRequest: signLoginPayload,
    );
    _sessions[ownerAccount] = next;
    return next;
  }

  /// 清除某账户的本地会话缓存（注销后调用，配合 Worker 端会话失效实现零残留）。
  void clearSession(String ownerAccount) {
    _sessions.remove(ownerAccount);
  }

  /// 注销账户：硬删除该用户在 Cloudflare 的全部数据（链上数据不受影响）。
  Future<void> deleteAccount({
    required String ownerAccount,
    required SquareActionSigner signAction,
  }) {
    return _consumeAccountAction(
      ownerAccount: ownerAccount,
      challengePath: '/v1/square/account/delete/challenge',
      confirmPath: '/v1/square/account/delete',
      signAction: signAction,
    );
  }

  /// 账户敏感动作签名往返：取挑战 → 客户端**钉死** op_tag 重算摘要并签 → 提交确认。
  /// 绝不采信服务端下发的 op_tag（固定 [kOpSignSquareAction]），防被诱导跨域签名。
  Future<void> _consumeAccountAction({
    required String ownerAccount,
    required String challengePath,
    required String confirmPath,
    required SquareActionSigner signAction,
  }) async {
    final challenge = await _postJson(challengePath, {
      'owner_account': ownerAccount,
    });
    final signingPayloadHex = challenge['signing_payload_hex'];
    final challengeId = challenge['challenge_id'];
    if (signingPayloadHex is! String || challengeId is! String) {
      throw const SquareApiException('动作挑战响应不完整');
    }
    final message = signingMessage(
      opTag: kOpSignSquareAction,
      scalePayload: hexToBytes(signingPayloadHex),
    );
    final signature = await signAction(message);
    await _postJson(confirmPath, {
      'owner_account': ownerAccount,
      'challenge_id': challengeId,
      'signature': signature,
    });
  }

  /// 注册 P-256 设备子钥：绑定证明由 sr25519 主钥对
  /// [buildDeviceBindingSigningMessage]（op_tag 摘要）签名，后端验签后落库。
  /// 此后登录挑战改由子钥静默签名。
  Future<void> registerDeviceSubkey({
    required String ownerAccount,
    required String p256PubkeyHex,
    required int issuedAt,
    required String bindingSignatureHex,
    String? turnstileToken,
  }) async {
    await _postJson('/v1/square/auth/device/register', {
      'owner_account': ownerAccount,
      'p256_pubkey': p256PubkeyHex,
      'issued_at': issuedAt,
      'binding_signature': bindingSignatureHex,
      if (turnstileToken != null) 'turnstile_token': turnstileToken,
    });
  }

  Future<SquareMembershipState> fetchMembership(SquareSession session) async {
    final data = await _getJson(
      '/v1/square/membership',
      session: session,
    );
    final membership = data['membership'];
    final active = data['active'] == true;
    final subscriptionActive = data['subscription_active'] == true;
    final plans = _parseMembershipPlans(data['plans']);
    // 会员与身份解耦（ADR-036）：响应只含订阅与套餐，无身份/冻结字段。
    if (membership is! Map<String, dynamic>) {
      // 无订阅 → 聊天文件上限 fail-closed 到自由档（ADR-036 会员权益）。
      ChatMediaLimits.applyMembershipLevel(null);
      return SquareMembershipState(
        active: false,
        paidUntil: 0,
        plans: plans,
      );
    }
    final membershipLevel = membership['membership_level']?.toString();
    // 会员权益之一 = 聊天文件上限按档；订阅有效才享该档，失效回落自由档。
    ChatMediaLimits.applyMembershipLevel(active ? membershipLevel : null);
    return SquareMembershipState(
      active: active,
      paidUntil: _asInt(membership['paid_until']),
      membershipLevel: membershipLevel,
      subscriptionStatus: membership['subscription_status']?.toString(),
      subscriptionActive: subscriptionActive,
      lastChargedAt: _asInt(membership['last_charged_at']),
      plans: plans,
    );
  }

  /// 平台会员订阅/取消上链后回执镜像（best-effort，链上已是真源，失败不阻塞）。
  /// 带 [level]=订阅确认（镜像 active）；缺 [level]=取消确认（镜像 cancelled）。
  /// owner 由 Worker 从 session 派生，客户端不上传。
  Future<void> confirmPlatformSubscription({
    required SquareSession session,
    required String txHash,
    required String blockHashHex,
    required String signedExtrinsicHex,
    required String action,
    String? membershipLevel,
  }) async {
    await _postJson(
      '/v1/square/membership/confirm',
      {
        'tx_hash': txHash,
        'block_hash': blockHashHex,
        'signed_extrinsic_hex': signedExtrinsicHex,
        'action': action,
        if (membershipLevel != null) 'membership_level': membershipLevel,
      },
      session: session,
      finalizedMirror: true,
    );
  }

  /// 分页拉取当前 session 所属账户的通讯录密文。
  Future<({List<SquareEncryptedContact> items, String? nextCursor})>
      fetchEncryptedContacts({
    required SquareSession session,
    String? cursor,
    int limit = 100,
  }) async {
    final query = <String>['limit=$limit'];
    if (cursor != null && cursor.isNotEmpty) {
      query.add('cursor=${Uri.encodeQueryComponent(cursor)}');
    }
    final data = await _getJson(
      '/v1/square/contacts?${query.join('&')}',
      session: session,
    );
    final rawItems = data['items'];
    if (rawItems is! List) {
      throw const SquareApiException('通讯录响应缺少密文列表');
    }
    final next = data['next_cursor']?.toString().trim();
    return (
      items: rawItems
          .whereType<Map<String, dynamic>>()
          .map(SquareEncryptedContact.fromJson)
          .toList(growable: false),
      nextCursor: next == null || next.isEmpty ? null : next,
    );
  }

  /// 幂等写入一条通讯录密文；owner 只能由 Worker 从 session 派生。
  Future<void> putEncryptedContact({
    required SquareSession session,
    required SquareEncryptedContact contact,
  }) async {
    await _putJson(
      '/v1/square/contacts/${Uri.encodeComponent(contact.contactId)}',
      <String, Object?>{
        'ciphertext': contact.ciphertext,
        'nonce': contact.nonce,
        'mac': contact.mac,
        'updated_at': contact.updatedAt,
      },
      session: session,
    );
  }

  /// 删除当前 session 所属账户的一条通讯录密文。
  Future<void> deleteEncryptedContact({
    required SquareSession session,
    required String contactId,
  }) async {
    await _deleteJson(
      '/v1/square/contacts/${Uri.encodeComponent(contactId)}',
      session: session,
    );
  }

  Future<SquarePreparedUpload> prepareUpload({
    required SquareSession session,
    required SquarePostCategory postCategory,
    required SquarePostContentFormat contentFormat,
    required int titleLength,
    required int textLength,
    required String manifestHash,
    required List<SquareUploadMediaRequest> mediaItems,
  }) async {
    final data = await _postJson(
      '/v1/square/uploads/prepare',
      {
        'post_category': postCategory.workerValue,
        'content_format': contentFormat.workerValue,
        'title_length': titleLength,
        'text_length': textLength,
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
    required Uint8List body,
    required SquareSession session,
  }) async {
    await uploadBytesTo(uploadUrl, body, contentType, session: session);
  }

  Future<void> uploadMediaAsset({
    required SquarePreparedMediaUpload upload,
    required String filePath,
    required SquareSession session,
  }) async {
    if (upload.uploadMethod == 'tus') {
      await _uploadTusMedia(
        uploadUrl: upload.uploadUrl,
        filePath: filePath,
        contentLength: upload.byteSize,
      );
      return;
    }
    if (upload.uploadMethod != 'worker') {
      throw const SquareApiException('媒体上传方式不受支持');
    }
    final bytes = await File(filePath).readAsBytes();
    if (bytes.length != upload.byteSize) {
      throw const SquareApiException('媒体实际大小与上传授权不一致');
    }
    await uploadBytesTo(upload.uploadUrl, bytes, upload.contentType,
        session: session);
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
      storageState: _requireString(data, 'storage_state'),
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
  Future<void> deletePost({
    required SquareSession session,
    required String postId,
  }) async {
    await _deleteJson(
      '/v1/square/posts/${Uri.encodeComponent(postId)}',
      session: session,
    );
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
    lastBrowseState = _parseBrowseState(data);
    return posts
        .whereType<Map<String, dynamic>>()
        .map(_parsePost)
        .toList(growable: false);
  }

  /// 把头像/背景等 R2 object_key 拼成钱包 session 保护的读取 URL；调用方必须
  /// 在 Image.network headers 中携带 Bearer。广场主媒体直接使用 Images / Stream URL。
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
    lastBrowseState = _parseBrowseState(data);
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

  /// 用户小文件只允许 PUT 到同域 Worker，并对原始字节生成设备请求签名。
  Future<void> uploadBytesTo(
    String uploadUrl,
    List<int> bytes,
    String contentType, {
    SquareSession? session,
  }) async {
    final uri = Uri.parse(uploadUrl);
    if (session == null || uri.origin != baseUri.origin) {
      throw const SquareApiException('资源上传地址必须是当前 Worker 且携带钱包会话');
    }
    final signer = session.signRequest;
    if (signer == null) {
      throw const SquareApiException('设备请求签名器缺失，请重新登录');
    }
    final body = Uint8List.fromList(bytes);
    final headers = <String, String>{
      'content-type': contentType,
      'authorization': 'Bearer ${session.sessionToken}',
      ...await squareRequestHeadersForBytes(
        method: 'PUT',
        uri: uri,
        body: body,
        sessionToken: session.sessionToken,
        sign: signer,
      ),
    };
    final response = await _http
        .put(uri, headers: headers, body: body)
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

  /// 开/关对某关注的发帖通知（通知归属挂在关注关系上；须已关注，未关注 Worker 回 409）。
  Future<void> setNotify({
    required SquareSession session,
    required String followedAccount,
    required bool enabled,
  }) async {
    await _putJson(
      '/v1/square/follows/${Uri.encodeComponent(followedAccount)}/notify',
      {'enabled': enabled},
      session: session,
    );
  }

  /// 拉取发帖通知双游标红点计数：广场底部 tab 与关注子 tab 各一。
  Future<({int squareUnread, int followingUnread})> fetchNotifyUnread({
    required SquareSession session,
  }) async {
    final data = await _getJson('/v1/square/notify/unread', session: session);
    return (
      squareUnread: (data['square_unread'] as num?)?.toInt() ?? 0,
      followingUnread: (data['following_unread'] as num?)?.toInt() ?? 0,
    );
  }

  /// 推进某作用域的已读游标（`square` 进广场清、`following` 进关注子 tab 清），红点归零。
  Future<void> markNotifyRead({
    required SquareSession session,
    required String scope,
  }) async {
    await _postJson(
      '/v1/square/notify/read',
      {'scope': scope},
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
    final uri = _uri(path);
    final response = await _http
        .get(uri, headers: await _headers('GET', uri, '', session))
        .timeout(const Duration(seconds: 20));
    return _decodeResponse(response);
  }

  Future<Map<String, dynamic>> _postJson(
    String path,
    Map<String, Object?> body, {
    SquareSession? session,
    bool finalizedMirror = false,
  }) async {
    final encoded = jsonEncode(body);
    final uri = _uri(path);
    final response = await _http
        .post(
          uri,
          headers: finalizedMirror
              ? _finalizedMirrorHeaders(session)
              : await _headers('POST', uri, encoded, session),
          body: encoded,
        )
        .timeout(const Duration(seconds: 20));
    return _decodeResponse(response);
  }

  /// 业务交易已经账户签名并 finalized；回执只用会话鉴权，不能再生成设备签名。
  Map<String, String> _finalizedMirrorHeaders(SquareSession? session) {
    if (session == null) {
      throw const SquareApiException('会员镜像回执缺少登录态');
    }
    return {
      'content-type': 'application/json; charset=utf-8',
      'authorization': 'Bearer ${session.sessionToken}',
    };
  }

  Future<Map<String, dynamic>> _putJson(
    String path,
    Map<String, Object?> body, {
    SquareSession? session,
  }) async {
    final encoded = jsonEncode(body);
    final uri = _uri(path);
    final response = await _http
        .put(
          uri,
          headers: await _headers('PUT', uri, encoded, session),
          body: encoded,
        )
        .timeout(const Duration(seconds: 20));
    return _decodeResponse(response);
  }

  Future<Map<String, dynamic>> _deleteJson(
    String path, {
    SquareSession? session,
  }) async {
    final uri = _uri(path);
    final response = await _http
        .delete(uri, headers: await _headers('DELETE', uri, '', session))
        .timeout(const Duration(seconds: 20));
    return _decodeResponse(response);
  }

  Uri _uri(String path) => Uri.parse('$baseUrl$path');

  Future<Map<String, String>> _headers(
    String method,
    Uri uri,
    String body,
    SquareSession? session,
  ) async {
    final headers = <String, String>{
      'content-type': 'application/json; charset=utf-8',
    };
    if (session == null) return headers;
    headers['authorization'] = 'Bearer ${session.sessionToken}';
    final signer = session.signRequest;
    if (signer == null) {
      throw const SquareApiException('设备请求签名器缺失，请重新登录');
    }
    headers.addAll(await squareRequestHeaders(
      method: method,
      uri: uri,
      body: body,
      sessionToken: session.sessionToken,
      sign: signer,
    ));
    return headers;
  }

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
      provider: _requireString(item, 'provider'),
      providerAssetId: _requireString(item, 'provider_asset_id'),
      uploadMethod: _requireString(item, 'upload_method'),
      uploadUrl: _requireString(item, 'upload_url'),
    );
  }

  List<SquareMembershipPlan> _parseMembershipPlans(Object? value) {
    if (value is! List) {
      return const <SquareMembershipPlan>[];
    }
    return value
        .whereType<Map<String, dynamic>>()
        .map(_parseMembershipPlan)
        .toList(growable: false);
  }

  SquareMembershipPlan _parseMembershipPlan(Map<String, dynamic> data) {
    final dynamicQuota = data['dynamic'] is Map<String, dynamic>
        ? data['dynamic'] as Map<String, dynamic>
        : const <String, dynamic>{};
    final articleQuota = data['article'] is Map<String, dynamic>
        ? data['article'] as Map<String, dynamic>
        : const <String, dynamic>{};
    final usageQuota = data['usage'] is Map<String, dynamic>
        ? data['usage'] as Map<String, dynamic>
        : const <String, dynamic>{};
    return SquareMembershipPlan(
      membershipLevel: _requireString(data, 'membership_level'),
      displayName: _requireString(data, 'display_name'),
      chatFileMaxBytes: _asInt(data['chat_file_max_bytes']),
      dynamicTextMaxChars: _asInt(dynamicQuota['text_max_chars']),
      dynamicImageQuality: dynamicQuota['image_quality']?.toString() ?? 'sd',
      dynamicMaxImages: _asInt(dynamicQuota['max_images']),
      dynamicVideoQuality: dynamicQuota['video_quality']?.toString() ?? 'sd',
      dynamicMaxVideos: _asInt(dynamicQuota['max_videos']),
      dynamicMaxVideoSeconds: _asInt(dynamicQuota['max_video_seconds']),
      dynamicMaxVideoBytes: _asInt(dynamicQuota['max_video_bytes']),
      articleTitleMinChars: _asInt(articleQuota['title_min_chars']),
      articleTitleMaxChars: _asInt(articleQuota['title_max_chars']),
      articleBodyMaxChars: _asInt(articleQuota['body_max_chars']),
      articleCoverQuality: articleQuota['cover_quality']?.toString() ?? 'hd',
      articleImageQuality: articleQuota['image_quality']?.toString() ?? 'sd',
      articleMaxImages: _asInt(articleQuota['max_images']),
      monthlyImages: _asInt(usageQuota['monthly_images']),
      monthlyVideoSeconds: _asInt(usageQuota['monthly_video_seconds']),
      activeUploads: _asInt(usageQuota['active_uploads']),
    );
  }

  SquarePost _parsePost(Map<String, dynamic> data) {
    final mediaItems = data['media_items'];
    return SquarePost(
      postId: _requireString(data, 'post_id'),
      author: SquareAuthor(
        ownerAccount: _requireString(data, 'owner_account'),
        cidNumber: data['cid_number']?.toString(),
        // 昵称与头像来自作者 profile.json（Worker feed 按去重作者回填）；缺失时
        // Flutter 按作者账户稳定选择本地默认昵称和照片，绝不把账户当昵称。
        displayName:
            (data['display_name']?.toString().trim().isNotEmpty ?? false)
                ? data['display_name'].toString().trim()
                : null,
        avatarObjectKey:
            (data['avatar_object_key']?.toString().isNotEmpty ?? false)
                ? data['avatar_object_key'].toString()
                : null,
        // 作者徽章信号（Worker feed 已按去重作者读链身份+会员回填）。
        identityLevel: data['identity_level']?.toString(),
        membershipLevel: data['membership_level']?.toString(),
        membershipActive: data['membership_active'] == true,
      ),
      postCategory: _parseCategory(data['post_category']),
      contentFormat: data['content_format'] == 'article'
          ? SquarePostContentFormat.article
          : SquarePostContentFormat.normal,
      title: (data['title']?.toString().trim().isNotEmpty ?? false)
          ? data['title'].toString().trim()
          : null,
      contentBlocks: parseArticleContentBlocks(data['content_blocks']),
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
    final url = data['url']?.toString() ?? data['object_key']?.toString() ?? '';
    final coverUrl = data['thumbnail_url']?.toString() ??
        data['cover_url']?.toString() ??
        data['cover_object_key']?.toString() ??
        '';
    return SquareMediaItem(
      mediaKind: data['media_kind'] == 'video'
          ? SquareMediaKind.video
          : SquareMediaKind.image,
      url: _resolveMediaUrl(url),
      coverUrl: coverUrl.isEmpty ? null : _resolveMediaUrl(coverUrl),
      byteSize: _nullableInt(data['byte_size']),
      assetState: data['asset_state']?.toString(),
      archiveState: data['archive_state']?.toString(),
      // 横竖屏判定所需原始尺寸；Worker feed 已随 media_items 回传。
      width: _nullableInt(data['width']),
      height: _nullableInt(data['height']),
    );
  }

  Future<void> _uploadTusMedia({
    required String uploadUrl,
    required String filePath,
    required int contentLength,
  }) async {
    final uri = Uri.parse(uploadUrl);
    final request = http.StreamedRequest('PATCH', uri)
      ..headers['tus-resumable'] = '1.0.0'
      ..headers['upload-offset'] = '0'
      ..headers['content-type'] = 'application/offset+octet-stream'
      ..contentLength = contentLength;
    await request.sink.addStream(File(filePath).openRead());
    await request.sink.close();
    final response =
        await _http.send(request).timeout(const Duration(hours: 4));
    if (response.statusCode < 200 || response.statusCode >= 300) {
      final text = await response.stream.bytesToString();
      throw SquareApiException(
        '视频 tus 上传失败：${response.statusCode} $text',
        statusCode: response.statusCode,
      );
    }
  }

  String _resolveMediaUrl(String value) {
    if (value.isEmpty) return '';
    final uri = Uri.tryParse(value);
    if (uri != null && uri.hasScheme && uri.host.isNotEmpty) {
      return value;
    }
    return mediaUrl(value);
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

  static SquareBrowseState? _parseBrowseState(Map<String, dynamic> data) {
    final day = data['browse_day'];
    if (day is! String || day.isEmpty) return null;
    return SquareBrowseState(
      browseDay: day,
      browseCount: _asInt(data['browse_count']),
      browseLimit: _nullableInt(data['browse_limit']),
      browseLeft: _nullableInt(data['browse_left']),
    );
  }

  void close() => _http.close();
}
