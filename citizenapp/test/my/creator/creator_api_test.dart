import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/8964/services/square_api_client.dart'
    show SquareSession;
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/my/creator/creator_api.dart';
import 'package:citizenapp/my/creator/creator_service.dart';
import 'package:citizenapp/my/creator/models/creator_plan.dart';
import 'package:citizenapp/rpc/chain_rpc.dart' show TxPoolWatchCallback;
import 'package:citizenapp/rpc/subscription_rpc.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  const session = SquareSession(
    sessionToken: 't',
    accountId:
        '0x7777777777777777777777777777777777777777777777777777777777777777',
    expiresAt: 9999999999999,
  );

  const tier = CreatorTier(
    tierId: 't1',
    name: '铁杆粉丝',
    pricesFen: {BillingPeriod.monthly: 990},
  );

  test('FakeCreatorApi 只接收已 finalized 交易哈希，不再触发第二次业务签名', () async {
    final api = FakeCreatorApi();

    final plan = await api.saveMyPlan(
      session: session,
      txHash: '0x${List.filled(64, 'a').join()}',
      blockHashHex: '0x${List.filled(64, 'b').join()}',
      signedExtrinsicHex: '0x0102',
      tiers: const [tier],
    );

    expect(api.lastSaveTxHash, '0x${List.filled(64, 'a').join()}');
    expect(plan.creatorAccountId,
        '0x7777777777777777777777777777777777777777777777777777777777777777');
    expect(plan.tiers, hasLength(1));
    expect(await api.fetchMyPlan(session), isNotNull);
  });

  test('CreatorApiHttp 保存只调用一次 plan 接口且携带链上交易哈希', () async {
    final paths = <String>[];
    var deviceSignCount = 0;
    final txHash = '0x${List.filled(64, 'b').join()}';
    final blockHash = '0x${List.filled(64, 'c').join()}';
    final api = CreatorApiHttp(
      baseUrl: 'https://creator.test',
      httpClient: MockClient((request) async {
        paths.add(request.url.path);
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        expect(body['tx_hash'], txHash);
        expect(body['block_hash'], blockHash);
        expect(body['signed_extrinsic_hex'], '0x0102');
        expect(body, isNot(contains('challenge_id')));
        expect(body, isNot(contains('signature')));
        expect(request.headers['authorization'], 'Bearer t');
        expect(request.headers, isNot(contains('x-device-signature')));
        return http.Response.bytes(
          utf8.encode(jsonEncode({
            'plan': {
              'creator_account_id':
                  '0x7777777777777777777777777777777777777777777777777777777777777777',
              'tiers': [tier.toJson()],
              'updated_at': 1,
            },
          })),
          200,
          headers: {'content-type': 'application/json; charset=utf-8'},
        );
      }),
    );
    final signedSession = SquareSession(
      sessionToken: 't',
      accountId:
          '0x7777777777777777777777777777777777777777777777777777777777777777',
      expiresAt: 9999999999999,
      signRequest: (_) async {
        deviceSignCount++;
        return 'device-signature';
      },
    );

    await api.saveMyPlan(
      session: signedSession,
      txHash: txHash,
      blockHashHex: blockHash,
      signedExtrinsicHex: '0x0102',
      tiers: const [tier],
    );

    expect(paths, ['/v1/square/creator/plan']);
    expect(deviceSignCount, 0, reason: 'finalized 后的 Cloudflare 镜像不得产生第二次签名');
  });

  test('FakeCreatorApi 概览默认按档位数', () async {
    final api = FakeCreatorApi(
      initialPlan: const CreatorPlan(
        creatorAccountId:
            '0x7777777777777777777777777777777777777777777777777777777777777777',
        tiers: [tier],
        updatedAt: 0,
      ),
    );
    final overview = await api.fetchOverview(session);
    expect(overview.tierCount, 1);
    expect(overview.subscriberCount, 0);
  });

  test('链上 finalized 后 Cloudflare 失败只重试镜像，不产生第二次链上签名', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final api = _FlakyCreatorApi()..failSave = true;
    final rpc = _FakeSubscriptionRpc();
    final service = CreatorService(
      api: api,
      subscriptionRpc: rpc,
      walletManager: _FakeWalletManager(),
      sessionProvider: _FakeSessionProvider(),
      preferences: prefs,
    );

    final saved = await service.saveTiers(const [tier]);
    expect(saved.tiers.single.name, '铁杆粉丝');
    expect(rpc.setPlansCount, 1);
    expect(rpc.signCount, 1);
    expect(api.saveCount, 1);
    expect(
      prefs.getString('creator_plan_mirror_pending:$_accountId'),
      isNotNull,
    );

    api.failSave = false;
    await service.load();
    expect(api.saveCount, 2, reason: '再次进入页面只重试 Cloudflare 展示镜像');
    expect(rpc.setPlansCount, 1, reason: '同一业务不得再次提交链上交易');
    expect(rpc.signCount, 1, reason: '同一业务不得再次账户签名');
    expect(
      prefs.getString('creator_plan_mirror_pending:$_accountId'),
      isNull,
    );
  });
}

const _signerSs58Address = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const _accountId =
    '0x0000000000000000000000000000000000000000000000000000000000000000';

class _FakeSessionProvider extends SquareSessionProvider {
  @override
  Future<SquareSession?> ensureSession() async => const SquareSession(
        sessionToken: 'creator-session',
        accountId: _accountId,
        expiresAt: 9999999999999,
      );
}

class _FakeWalletManager extends WalletManager {
  @override
  Future<WalletProfile?> getDefaultWallet() async => const WalletProfile(
        walletIndex: 1,
        walletName: 'creator',
        walletIcon: '',
        balance: 0,
        ss58Address: _signerSs58Address,
        accountId: _accountId,
        alg: 'sr25519',
        ss58: 2027,
        createdAtMillis: 0,
        source: 'test',
        signMode: 'local',
      );

  @override
  Future<Uint8List> signWithWallet(int walletIndex, Uint8List payload) async =>
      Uint8List(64);
}

class _FakeSubscriptionRpc extends SubscriptionRpc {
  int setPlansCount = 0;
  int signCount = 0;

  @override
  Future<FinalizedSubscriptionSnapshot> fetchSubscriptionSnapshot({
    required String subscriberAccountId,
    String? creatorAccountId,
  }) async =>
      FinalizedSubscriptionSnapshot(
        state: ChainSubscriptionState(
          plan: const ChainSubscriptionPlan.platform('freedom'),
          startedAt: 1000,
          lastChargedAt: 1000,
          lastChargedPriceFen: BigInt.one,
          paidUntil: 3000,
          status: 'active',
          authorizedPriceFen: BigInt.one,
          suspendReason: null,
        ),
        chainNowMs: 2000,
        blockHashHex: '0x${List.filled(64, '0').join()}',
      );

  @override
  Future<FinalizedSubscriptionTransaction> setCreatorPlans({
    required String fromSs58Address,
    required Uint8List signerPublicKey,
    required List<CreatorTierInput> tiers,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    setPlansCount++;
    await sign(Uint8List.fromList([1]));
    signCount++;
    return (
      txHash: '0x${List.filled(64, 'c').join()}',
      usedNonce: 1,
      blockHashHex: '0x${List.filled(64, 'd').join()}',
      signedExtrinsicHex: '0x0102',
    );
  }

  @override
  Future<List<ChainCreatorTier>> fetchCreatorPlans(
          String creatorAccountId) async =>
      [
        ChainCreatorTier(
          tierId: 't1',
          pricesFen: {'monthly': BigInt.from(990)},
        ),
      ];
}

class _FlakyCreatorApi extends FakeCreatorApi {
  bool failSave = false;
  int saveCount = 0;

  @override
  Future<CreatorPlan> saveMyPlan({
    required SquareSession session,
    required String txHash,
    required String blockHashHex,
    required String signedExtrinsicHex,
    required List<CreatorTier> tiers,
  }) async {
    saveCount++;
    if (failSave) throw const CreatorApiException('temporary');
    return super.saveMyPlan(
      session: session,
      txHash: txHash,
      blockHashHex: blockHashHex,
      signedExtrinsicHex: signedExtrinsicHex,
      tiers: tiers,
    );
  }
}
