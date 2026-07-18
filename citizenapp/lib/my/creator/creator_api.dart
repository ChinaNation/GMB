import 'dart:convert';
import 'dart:typed_data';

import 'package:http/http.dart' as http;

import 'package:citizenapp/8964/services/square_api_client.dart'
    show SquareApiConfig, SquareSession;
import 'package:citizenapp/8964/services/square_request_signer.dart';
import 'package:citizenapp/my/creator/models/creator_overview.dart';
import 'package:citizenapp/my/creator/models/creator_plan.dart';
import 'package:citizenapp/signer/signing.dart';

/// 广场账户动作主钥签名器（0x1D 摘要 → sr25519 主钥签，读硬件金库弹一次生物识别）。
/// 与 `SquareApiClient.SquareActionSigner` 同形；由 [CreatorService] 用 WalletManager 提供。
typedef CreatorActionSigner = Future<String> Function(Uint8List actionMessage);

/// 创作者档位/概览的边缘（Cloudflare）数据源。
///
/// 档位定义（名称/周期/价格）全链下：读写都在 Cloudflare；写入经**现有广场账户动作
/// 统一签名**（`OP_SIGN_SQUARE_ACTION` 0x1D，challenge→主钥签→confirm），不新增签名协议。
abstract interface class CreatorApi {
  /// 读我的档位；无档位返回 null。
  Future<CreatorPlan?> fetchMyPlan(SquareSession session);

  /// 读概览（订阅人数 / 预计月收入 / 档位数，均为预计值）。
  Future<CreatorOverview> fetchOverview(SquareSession session);

  /// 覆盖式保存我的档位。走广场账户动作签名往返：challenge → [signAction]（生物识别）→ confirm。
  Future<CreatorPlan> saveMyPlan({
    required SquareSession session,
    required String ownerAccount,
    required List<CreatorTier> tiers,
    required CreatorActionSigner signAction,
  });

  /// 读某创作者的档位（订阅者在他人主页选档用）。无档返回 null。
  Future<CreatorPlan?> fetchPlanOf(
      SquareSession session, String creatorAccount);

  /// 我对某创作者的订阅态（按钮双态）：`active`/`past_due`/`cancelled`；未订阅返回 null。
  Future<String?> fetchMySubscriptionTo(
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

/// 生产实现：直连 Cloudflare Worker，复用现有会话鉴权与统一签名。
///
/// 依赖 BFF 端点（另立 Cloudflare 卡实现）：
///   GET  /v1/square/creator/plan
///   GET  /v1/square/creator/overview
///   POST /v1/square/creator/plan/challenge   （发起动作挑战，绑 tiers 哈希）
///   POST /v1/square/creator/plan             （验签后覆盖写 D1）
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
    required String ownerAccount,
    required List<CreatorTier> tiers,
    required CreatorActionSigner signAction,
  }) async {
    final tiersJson = tiers.map((tier) => tier.toJson()).toList();
    // 1) 发起动作挑战（Worker 绑 action='set_creator_plan' + owner + tiers 哈希 + 过期）。
    final challenge = await _postJson(
      '/v1/square/creator/plan/challenge',
      {'owner_account': ownerAccount, 'tiers': tiersJson},
      session,
    );
    final payloadHex = challenge['signing_payload_hex'];
    final challengeId = challenge['challenge_id'];
    if (payloadHex is! String || challengeId is! String) {
      throw const CreatorApiException('创作者档位挑战响应不完整');
    }
    // 2) 客户端钉死 op_tag（0x1D 广场账户动作），主钥签名（生物识别）。
    final message = signingMessage(
      opTag: kOpSignSquareAction,
      scalePayload: _hexToBytes(payloadHex),
    );
    final signature = await signAction(message);
    // 3) 提交确认：Worker 验签 + tiers 哈希一致 → 覆盖写 D1，回传最新计划。
    final saved = await _postJson(
      '/v1/square/creator/plan',
      {
        'owner_account': ownerAccount,
        'challenge_id': challengeId,
        'signature': signature,
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
  Future<String?> fetchMySubscriptionTo(
      SquareSession session, String creatorAccount) async {
    final data = await _getJson(
      '/v1/square/creator/subscription/${Uri.encodeComponent(creatorAccount)}',
      session,
    );
    final status = data['status'];
    return status is String ? status : null;
  }

  @override
  Future<void> confirmCreatorSubscription({
    required SquareSession session,
    required String txHash,
    required String creatorAccount,
    String? tierId,
    String? period,
  }) async {
    await _postJson(
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

  Future<Map<String, dynamic>> _postJson(
    String path,
    Map<String, Object?> body,
    SquareSession session,
  ) async {
    final encoded = jsonEncode(body);
    final uri = Uri.parse('$baseUrl$path');
    final response = await _http
        .post(uri,
            headers: await _headers('POST', uri, encoded, session),
            body: encoded)
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

  static Uint8List _hexToBytes(String input) {
    final text = input.startsWith('0x') || input.startsWith('0X')
        ? input.substring(2)
        : input;
    if (text.length.isOdd) {
      throw const CreatorApiException('hex 长度必须为偶数');
    }
    final out = Uint8List(text.length ~/ 2);
    for (var i = 0; i < out.length; i++) {
      out[i] = int.parse(text.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return out;
  }
}

/// 离线内存实现：本地开发 / 测试用（不依赖真 Cloudflare），仍触发真实的主钥签名（生物识别）。
class FakeCreatorApi implements CreatorApi {
  FakeCreatorApi({CreatorPlan? initialPlan, CreatorOverview? overview})
      : _plan = initialPlan,
        _overview = overview;

  CreatorPlan? _plan;
  final CreatorOverview? _overview;

  /// 记录最近一次保存是否真的调用了签名器（供测试断言"编辑必过生物识别"）。
  bool lastSaveSigned = false;

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
    required String ownerAccount,
    required List<CreatorTier> tiers,
    required CreatorActionSigner signAction,
  }) async {
    // 即便离线也走一次主钥签名，保证"编辑会员档=核心操作必过生物识别"的行为一致。
    await signAction(Uint8List.fromList(utf8.encode('set_creator_plan')));
    lastSaveSigned = true;
    _plan =
        CreatorPlan(creatorAccount: ownerAccount, tiers: tiers, updatedAt: 0);
    return _plan!;
  }

  @override
  Future<CreatorPlan?> fetchPlanOf(
          SquareSession session, String creatorAccount) async =>
      _plan;

  @override
  Future<String?> fetchMySubscriptionTo(
          SquareSession session, String creatorAccount) async =>
      null;

  @override
  Future<void> confirmCreatorSubscription({
    required SquareSession session,
    required String txHash,
    required String creatorAccount,
    String? tierId,
    String? period,
  }) async {}
}
