import 'dart:convert';

import 'package:http/http.dart' as http;

import 'package:citizenapp/8964/services/square_api_client.dart'
    show SquareApiConfig, SquareSession;
import 'package:citizenapp/8964/services/square_request_signer.dart';
import 'package:citizenapp/my/creator/models/creator_overview.dart';
import 'package:citizenapp/my/creator/models/creator_plan.dart';

/// 创作者档位/概览的边缘（Cloudflare）数据源。
///
/// tier_id/周期/价格以 finalized 链状态为真源；Cloudflare 只保存名称等展示数据。
abstract interface class CreatorApi {
  /// 读我的档位；无档位返回 null。
  Future<CreatorPlan?> fetchMyPlan(SquareSession session);

  /// 读概览（订阅人数 / 预计月收入 / 档位数，均为预计值）。
  Future<CreatorOverview> fetchOverview(SquareSession session);

  /// 链上一次签名交易 finalized 后保存展示数据；本请求不得再触发账户业务签名。
  Future<CreatorPlan> saveMyPlan({
    required SquareSession session,
    required String txHash,
    required List<CreatorTier> tiers,
  });

  /// 读某创作者的档位（订阅者在他人主页选档用）。无档返回 null。
  Future<CreatorPlan?> fetchPlanOf(
      SquareSession session, String creatorAccount);

  /// 订阅/取消创作者会员上链后回执镜像（best-effort，链上已是真源）。
  Future<void> confirmCreatorSubscription({
    required SquareSession session,
    required String txHash,
    required String creatorAccount,
    String? tierId,
    String? period,
  });
}

class CreatorApiException implements Exception {
  const CreatorApiException(this.message);
  final String message;
  @override
  String toString() => 'CreatorApiException: $message';
}

/// 生产实现：直连 Cloudflare Worker，复用会话与设备级请求认证。
///
/// 依赖 BFF 端点（另立 Cloudflare 卡实现）：
///   GET  /v1/square/creator/plan
///   GET  /v1/square/creator/overview
///   POST /v1/square/creator/plan             （校验 finalized 链状态后覆盖展示镜像）
class CreatorApiHttp implements CreatorApi {
  CreatorApiHttp({String? baseUrl, http.Client? httpClient})
      : baseUrl = SquareApiConfig.normalizeBaseUrl(
          baseUrl ?? SquareApiConfig.defaultBaseUrl,
        ),
        _http = httpClient ?? http.Client();

  final String baseUrl;
  final http.Client _http;

  @override
  Future<CreatorPlan?> fetchMyPlan(SquareSession session) async {
    final data = await _getJson('/v1/square/creator/plan', session);
    final plan = data['plan'];
    if (plan is! Map<String, dynamic>) return null;
    return CreatorPlan.fromJson(plan);
  }

  @override
  Future<CreatorOverview> fetchOverview(SquareSession session) async {
    final data = await _getJson('/v1/square/creator/overview', session);
    final overview = data['overview'];
    if (overview is! Map<String, dynamic>) return CreatorOverview.zero;
    return CreatorOverview.fromJson(overview);
  }

  @override
  Future<CreatorPlan> saveMyPlan({
    required SquareSession session,
    required String txHash,
    required List<CreatorTier> tiers,
  }) async {
    final tiersJson = tiers.map((tier) => tier.toJson()).toList();
    final saved = await _postFinalizedMirrorJson(
      '/v1/square/creator/plan',
      {
        'tx_hash': txHash,
        'tiers': tiersJson,
      },
      session,
    );
    final plan = saved['plan'];
    if (plan is! Map<String, dynamic>) {
      throw const CreatorApiException('创作者档位保存响应不完整');
    }
    return CreatorPlan.fromJson(plan);
  }

  @override
  Future<CreatorPlan?> fetchPlanOf(
      SquareSession session, String creatorAccount) async {
    final data = await _getJson(
      '/v1/square/creator/plan/${Uri.encodeComponent(creatorAccount)}',
      session,
    );
    final plan = data['plan'];
    if (plan is! Map<String, dynamic>) return null;
    return CreatorPlan.fromJson(plan);
  }

  @override
  Future<void> confirmCreatorSubscription({
    required SquareSession session,
    required String txHash,
    required String creatorAccount,
    String? tierId,
    String? period,
  }) async {
    await _postFinalizedMirrorJson(
      '/v1/square/creator/subscription/confirm',
      {
        'tx_hash': txHash,
        'creator_account': creatorAccount,
        if (tierId != null) 'tier_id': tierId,
        if (period != null) 'period': period,
      },
      session,
    );
  }

  Future<Map<String, dynamic>> _getJson(
      String path, SquareSession session) async {
    final uri = Uri.parse('$baseUrl$path');
    final response = await _http
        .get(uri, headers: await _headers('GET', uri, '', session))
        .timeout(const Duration(seconds: 20));
    return _decode(response);
  }

  /// 链上业务已经由账户签名并 finalized；镜像请求只携带会话，不再生成设备签名。
  Future<Map<String, dynamic>> _postFinalizedMirrorJson(
    String path,
    Map<String, Object?> body,
    SquareSession session,
  ) async {
    final encoded = jsonEncode(body);
    final uri = Uri.parse('$baseUrl$path');
    final response = await _http
        .post(
          uri,
          headers: {
            'content-type': 'application/json; charset=utf-8',
            'authorization': 'Bearer ${session.sessionToken}',
          },
          body: encoded,
        )
        .timeout(const Duration(seconds: 20));
    return _decode(response);
  }

  Future<Map<String, String>> _headers(
    String method,
    Uri uri,
    String body,
    SquareSession session,
  ) async {
    final signer = session.signRequest;
    if (signer == null) {
      throw const CreatorApiException('设备请求签名器缺失，请重新登录');
    }
    return {
      'content-type': 'application/json; charset=utf-8',
      'authorization': 'Bearer ${session.sessionToken}',
      ...await squareRequestHeaders(
        method: method,
        uri: uri,
        body: body,
        sessionToken: session.sessionToken,
        sign: signer,
      ),
    };
  }

  Map<String, dynamic> _decode(http.Response response) {
    final Object? decoded;
    try {
      decoded = jsonDecode(response.body);
    } on FormatException {
      throw CreatorApiException('创作者服务响应不是 JSON：${response.statusCode}');
    }
    if (decoded is! Map<String, dynamic>) {
      throw CreatorApiException('创作者服务响应结构不合法：${response.statusCode}');
    }
    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw CreatorApiException(
        decoded['message']?.toString() ?? '创作者服务请求失败（${response.statusCode}）',
      );
    }
    return decoded;
  }
}

/// 离线内存实现：本地开发 / 测试用（不依赖真 Cloudflare）。
class FakeCreatorApi implements CreatorApi {
  FakeCreatorApi({CreatorPlan? initialPlan, CreatorOverview? overview})
      : _plan = initialPlan,
        _overview = overview;

  CreatorPlan? _plan;
  final CreatorOverview? _overview;

  String? lastSaveTxHash;

  @override
  Future<CreatorPlan?> fetchMyPlan(SquareSession session) async => _plan;

  @override
  Future<CreatorOverview> fetchOverview(SquareSession session) async =>
      _overview ??
      CreatorOverview(
        subscriberCount: 0,
        monthIncomeFen: 0,
        tierCount: _plan?.tiers.length ?? 0,
      );

  @override
  Future<CreatorPlan> saveMyPlan({
    required SquareSession session,
    required String txHash,
    required List<CreatorTier> tiers,
  }) async {
    lastSaveTxHash = txHash;
    _plan = CreatorPlan(
      creatorAccount: session.ownerAccount,
      tiers: tiers,
      updatedAt: 0,
    );
    return _plan!;
  }

  @override
  Future<CreatorPlan?> fetchPlanOf(
          SquareSession session, String creatorAccount) async =>
      _plan;

  @override
  Future<void> confirmCreatorSubscription({
    required SquareSession session,
    required String txHash,
    required String creatorAccount,
    String? tierId,
    String? period,
  }) async {}
}
