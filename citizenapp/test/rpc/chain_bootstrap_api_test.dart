import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:citizenapp/rpc/chain_bootstrap_api.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';

const _bootnodeA =
    '/dns4/nrcgch.crcfrcn.com/tcp/30333/wss/p2p/12D3KooWHepcMGD3h9VC1XNWmrac3pXo63RimV5jhTU2nC2TLAyS';
const _bootnodeB =
    '/dns4/prczss.crcfrcn.com/tcp/30333/wss/p2p/12D3KooWPjWNXvCzPv6PPuiGnF3J5uToW3ySfaB7rKkwUrN2CALv';
const _bootnodeHbs =
    '/dns4/prchbs.crcfrcn.com/tcp/30333/wss/p2p/12D3KooWMXQoZ9F6nxMuoC2ZnzxEKAn4z2qPKAugP2CZFEcXDqkT';
const _bootnodeHes =
    '/dns4/prches.crcfrcn.com/tcp/30333/wss/p2p/12D3KooWSkKBEJ2KZXckFhzLvrqqbhpq4PVKeFuWsxdTF7hfzoGc';
const _bootnodeSds =
    '/dns4/prcsds.crcfrcn.com/tcp/30333/wss/p2p/12D3KooWFgD8cFDqherjpiuRkHwHfAcCwaqXcBjTS2G3LkwUBTsq';
const _bootnodeSxs =
    '/dns4/prcsxs.crcfrcn.com/tcp/30333/wss/p2p/12D3KooWQY3DEaJy9wEBE2bQ9gG1B8XByfVaz839jf1ov75kRmD9';
const _stateRoot =
    '0x6a380e96686b152d1eaff8aafc526c23da43058cac2b98be8e98ea1f9e5eff63';

void main() {
  test('安装包 chainspec 只登记当前六个已部署 bootnode', () async {
    final spec = jsonDecode(await File('assets/chainspec.json').readAsString())
        as Map<String, dynamic>;

    expect(spec['bootNodes'], [
      _bootnodeA,
      _bootnodeB,
      _bootnodeSds,
      _bootnodeSxs,
      _bootnodeHes,
      _bootnodeHbs,
    ]);
  });

  test('ChainBootstrapApi 拉取并解析安全启动清单', () async {
    final api = ChainBootstrapApi(
      baseUrl: 'http://127.0.0.1:8787',
      httpClient: MockClient((request) async {
        expect(request.url.path, '/v1/chain/bootstrap');
        return http.Response(
          jsonEncode(_manifest()),
          200,
          headers: {'content-type': 'application/json'},
        );
      }),
    );

    final manifest = await api.fetchManifest();

    expect(manifest.chain.chainId, 'citizenchain');
    expect(manifest.chain.ss58Format, 2027);
    expect(manifest.lightClient.apiIsTruth, isFalse);
    expect(manifest.lightClient.bundledAssetsRequired, [
      'assets/chainspec.json',
      'assets/light_sync_state.json',
    ]);
    expect(manifest.security.rpcProxy, isFalse);
    expect(manifest.services.signedExtrinsicRelayEnabled, isFalse);
    expect(manifest.p2p.bootnodes, [_bootnodeA, _bootnodeB]);
  });

  test('ChainBootstrapApi 拒绝 API-only 或 RPC proxy 清单', () {
    final apiTruth = _manifest();
    (apiTruth['light_client'] as Map<String, dynamic>)['api_is_truth'] = true;

    expect(
      () => ChainBootstrapManifest.fromJson(apiTruth),
      throwsA(isA<ChainBootstrapApiException>()),
    );

    final rpcProxy = _manifest();
    (rpcProxy['security'] as Map<String, dynamic>)['rpc_proxy'] = true;

    expect(
      () => ChainBootstrapManifest.fromJson(rpcProxy),
      throwsA(isA<ChainBootstrapApiException>()),
    );
  });

  test('ChainBootstrapApi 拒绝任何 RPC URL 字段', () {
    final data = _manifest()
      ..['archive_rpc_url'] = 'https://rpc.example.invalid';

    expect(
      () => ChainBootstrapManifest.fromJson(data),
      throwsA(isA<ChainBootstrapApiException>()),
    );
  });

  test('ChainBootstrapApi 拒绝 v1 和任何远端 checkpoint 字段', () {
    final v1 = _manifest()..['schema'] = 'citizenapp.chain.bootstrap.v1';
    expect(
      () => ChainBootstrapManifest.fromJson(v1),
      throwsA(isA<ChainBootstrapApiException>()),
    );

    final checkpoint = _manifest();
    (checkpoint['light_client'] as Map<String, dynamic>)['checkpoint'] = {
      'source': 'remote_url',
      'light_sync_state_url': 'https://api.example/light-sync-state.json',
      'light_sync_state_sha256': 'ab' * 32,
    };
    expect(
      () => ChainBootstrapManifest.fromJson(checkpoint),
      throwsA(isA<ChainBootstrapApiException>()),
    );
  });

  test('ChainBootstrapApi 只接受固定 signed extrinsic relay path', () {
    final enabled = _manifest();
    (enabled['services'] as Map<String, dynamic>)['signed_extrinsic_relay'] = {
      'enabled': true,
      'path': '/v1/chain/extrinsics/relay',
    };

    final parsed = ChainBootstrapManifest.fromJson(enabled);
    expect(parsed.services.signedExtrinsicRelayEnabled, isTrue);
    expect(
      parsed.services.signedExtrinsicRelayPath,
      '/v1/chain/extrinsics/relay',
    );

    final badPath = _manifest();
    (badPath['services'] as Map<String, dynamic>)['signed_extrinsic_relay'] = {
      'enabled': true,
      'path': '/v1/chain/rpc',
    };

    expect(
      () => ChainBootstrapManifest.fromJson(badPath),
      throwsA(isA<ChainBootstrapApiException>()),
    );
  });

  test('ChainBootstrapApiConfig 只允许 HTTPS 或本地 HTTP', () {
    expect(
      ChainBootstrapApiConfig.normalizeBaseUrl('https://api.onchina.org/'),
      'https://api.onchina.org',
    );
    expect(
      ChainBootstrapApiConfig.normalizeBaseUrl('http://127.0.0.1:8787/'),
      'http://127.0.0.1:8787',
    );
    expect(
      () => ChainBootstrapApiConfig.normalizeBaseUrl('http://api.onchina.org'),
      throwsUnsupportedError,
    );
  });

  test('SmoldotClientManager 只在链参数匹配时注入推荐 bootnodes', () {
    final manifest = ChainBootstrapManifest.fromJson(_manifest());
    final injected =
        SmoldotClientManager.instance.injectBootstrapBootnodesForTest(
      jsonEncode(_chainSpec()),
      manifest,
    );
    final spec = jsonDecode(injected) as Map<String, dynamic>;

    expect(spec['bootNodes'],
        [_bootnodeA, _bootnodeB, '/dns4/old.example/tcp/30333/wss/p2p/old']);

    final mismatch = _manifest();
    (mismatch['chain'] as Map<String, dynamic>)['state_root'] =
        '0x${'11' * 32}';
    final unchanged =
        SmoldotClientManager.instance.injectBootstrapBootnodesForTest(
      jsonEncode(_chainSpec()),
      ChainBootstrapManifest.fromJson(mismatch),
    );

    expect(jsonDecode(unchanged), _chainSpec());
  });
}

Map<String, dynamic> _manifest() => {
      'ok': true,
      'schema': 'citizenapp.chain.bootstrap.v2',
      'generated_at': 1800000000000,
      'cache_ttl_seconds': 300,
      'chain': {
        'chain_id': 'citizenchain',
        'chain_name': 'CitizenChain',
        'chain_type': 'Live',
        'protocol_id': 'citizenchain',
        'genesis_hash':
            '0xb57c61a97f2b1fd7fa78756060a0c3e9a0ed6b1048bb8424b034a8f5f99a9971',
        'state_root': _stateRoot,
        'ss58_format': 2027,
        'token_symbol': 'GMB',
        'token_decimals': 2,
      },
      'light_client': {
        'mode': 'smoldot',
        'truth_source': 'p2p_finalized_storage',
        'api_is_truth': false,
        'bundled_assets_required': [
          'assets/chainspec.json',
          'assets/light_sync_state.json',
        ],
      },
      'p2p': {
        'bootnodes': [_bootnodeA, _bootnodeB],
        'bootnodes_source': 'worker_config',
        'min_peer_count_hint': 1,
      },
      'services': {
        'square_base_url': 'https://api.onchina.org/v1/square',
        'chat_base_url': 'https://api.onchina.org/v1/chat',
        'media_base_url': 'https://api.onchina.org/v1/square/media',
        'signed_extrinsic_relay': {
          'enabled': false,
          'path': null,
        },
      },
      'security': {
        'exposes_rpc_url': false,
        'rpc_proxy': false,
        'exposes_private_key_material': false,
        'validator_rpc_public': false,
      },
      'degradation': {
        'p2p_unavailable': 'chat_square_continue_chain_state_degraded',
        'chain_success_source': 'finalized_runtime_storage_or_events',
      },
    };

Map<String, dynamic> _chainSpec() => {
      'name': 'CitizenChain',
      'id': 'citizenchain',
      'chainType': 'Live',
      'protocolId': 'citizenchain',
      'properties': {
        'ss58Format': 2027,
        'tokenDecimals': 2,
        'tokenSymbol': 'GMB',
      },
      'genesis': {
        'stateRootHash': _stateRoot,
      },
      'bootNodes': ['/dns4/old.example/tcp/30333/wss/p2p/old'],
    };
