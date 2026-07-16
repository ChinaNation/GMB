import type { Env } from '../types';
import { jsonResponse, parsePositiveInt } from '../shared/http';
import { CHAIN_EXTRINSIC_RELAY_PATH, isChainExtrinsicRelayEnabled } from './extrinsic_relay';

const DEFAULT_BOOTSTRAP_TTL_SECONDS = 300;

export interface ChainBootstrapResponse {
  ok: true;
  schema: 'citizenapp.chain.bootstrap.v2';
  generated_at: number;
  cache_ttl_seconds: number;
  chain: {
    chain_id: 'citizenchain';
    chain_name: 'CitizenChain';
    chain_type: 'Live';
    protocol_id: 'citizenchain';
    genesis_hash: string;
    state_root: string;
    ss58_format: 2027;
    token_symbol: 'GMB';
    token_decimals: 2;
  };
  light_client: {
    mode: 'smoldot';
    truth_source: 'p2p_finalized_storage';
    api_is_truth: false;
    bundled_assets_required: ['assets/chainspec.json', 'assets/light_sync_state.json'];
  };
  p2p: {
    bootnodes: string[];
    bootnodes_source: 'worker_config' | 'bundled_chainspec';
    min_peer_count_hint: 1;
  };
  services: {
    square_base_url: string;
    chat_base_url: string;
    media_base_url: string;
    signed_extrinsic_relay: {
      enabled: boolean;
      path: string | null;
    };
  };
  security: {
    exposes_rpc_url: false;
    rpc_proxy: false;
    exposes_private_key_material: false;
    validator_rpc_public: false;
  };
  degradation: {
    p2p_unavailable: 'chat_square_continue_chain_state_degraded';
    chain_success_source: 'finalized_runtime_storage_or_events';
  };
}

export function chainBootstrapRoute(request: Request, env: Env): Response {
  const response = buildChainBootstrapResponse(request, env);
  return jsonResponse(response, {
    headers: {
      'cache-control': `public, max-age=${response.cache_ttl_seconds}`
    }
  });
}

export function buildChainBootstrapResponse(
  request: Request,
  env: Env
): ChainBootstrapResponse {
  const origin = new URL(request.url).origin;
  const bootnodes = parseBootnodes(env.CHAIN_BOOTNODES);
  const cacheTtlSeconds = parsePositiveInt(
    env.BOOT_TTL_SECONDS,
    DEFAULT_BOOTSTRAP_TTL_SECONDS
  );
  const relayEnabled = isChainExtrinsicRelayEnabled(env);

  return {
    ok: true,
    schema: 'citizenapp.chain.bootstrap.v2',
    generated_at: Date.now(),
    cache_ttl_seconds: cacheTtlSeconds,
    chain: {
      chain_id: 'citizenchain',
      chain_name: 'CitizenChain',
      chain_type: 'Live',
      protocol_id: 'citizenchain',
      // 中文注释：链身份只能来自随冻结流程同步的环境配置；缺失或非法必须失败，
      // 禁止静默回落到某次历史创世锚点。
      genesis_hash: requireHex32(env.CHAIN_GENESIS_HASH, 'CHAIN_GENESIS_HASH'),
      state_root: requireHex32(env.CHAIN_STATE_ROOT, 'CHAIN_STATE_ROOT'),
      ss58_format: 2027,
      token_symbol: 'GMB',
      token_decimals: 2
    },
    light_client: {
      mode: 'smoldot',
      truth_source: 'p2p_finalized_storage',
      api_is_truth: false,
      // 中文注释：checkpoint 只来自签名安装包；Worker 只声明必需资产，
      // 不下发 URL、摘要或任何可切换轻节点信任锚的字段。
      bundled_assets_required: ['assets/chainspec.json', 'assets/light_sync_state.json']
    },
    p2p: {
      bootnodes,
      // Worker 没配置 bootnodes 时，App 继续使用本地 chainspec 内置 bootNodes。
      bootnodes_source: bootnodes.length > 0 ? 'worker_config' : 'bundled_chainspec',
      min_peer_count_hint: 1
    },
    services: {
      square_base_url: `${origin}/v1/square`,
      chat_base_url: `${origin}/v1/chat`,
      media_base_url: `${origin}/v1/square/media`,
      signed_extrinsic_relay: {
        enabled: relayEnabled,
        path: relayEnabled ? CHAIN_EXTRINSIC_RELAY_PATH : null
      }
    },
    security: {
      exposes_rpc_url: false,
      rpc_proxy: false,
      exposes_private_key_material: false,
      validator_rpc_public: false
    },
    degradation: {
      p2p_unavailable: 'chat_square_continue_chain_state_degraded',
      chain_success_source: 'finalized_runtime_storage_or_events'
    }
  };
}

function parseBootnodes(value: string | undefined): string[] {
  if (!value) {
    return [];
  }

  const seen = new Set<string>();
  const bootnodes: string[] = [];
  for (const raw of value.split(/[\n,;]/)) {
    const bootnode = raw.trim();
    if (!isBootnode(bootnode) || seen.has(bootnode)) {
      continue;
    }
    seen.add(bootnode);
    bootnodes.push(bootnode);
  }
  return bootnodes;
}

function isBootnode(value: string): boolean {
  return value.startsWith('/') && value.includes('/p2p/') && value.length <= 256;
}

function requireHex32(value: string | undefined, field: string): string {
  if (!value || !/^0x[0-9a-fA-F]{64}$/.test(value)) {
    throw new Error(`${field} 缺失或不是 32 字节十六进制链锚点`);
  }
  return value.toLowerCase();
}
