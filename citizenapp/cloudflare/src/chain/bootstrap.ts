import type { Env } from '../types';
import { jsonResponse, parsePositiveInt } from '../shared/http';
import { CHAIN_EXTRINSIC_RELAY_PATH, isChainExtrinsicRelayEnabled } from './extrinsic_relay';

const DEFAULT_GENESIS_HASH =
  '0xb57c61a97f2b1fd7fa78756060a0c3e9a0ed6b1048bb8424b034a8f5f99a9971';
const DEFAULT_STATE_ROOT =
  '0x6a380e96686b152d1eaff8aafc526c23da43058cac2b98be8e98ea1f9e5eff63';
const DEFAULT_LIGHT_SYNC_STATE_SHA256 =
  'c5005187368b7ffbb0a95f67cf9f6f3d0dbfbe1ae91d456269198a2a311710b8';
const DEFAULT_BOOTSTRAP_TTL_SECONDS = 300;

export interface ChainBootstrapResponse {
  ok: true;
  schema: 'citizenapp.chain.bootstrap.v1';
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
    checkpoint: {
      source: 'bundled_asset' | 'remote_url';
      light_sync_state_url: string | null;
      light_sync_state_sha256: string;
    };
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
  const bootnodes = parseBootnodes(env.CITIZEN_CHAIN_BOOTNODES);
  const cacheTtlSeconds = parsePositiveInt(
    env.CITIZEN_CHAIN_BOOTSTRAP_TTL_SECONDS,
    DEFAULT_BOOTSTRAP_TTL_SECONDS
  );
  const lightSyncStateUrl = normalizePublicUrl(env.CITIZEN_CHAIN_LIGHT_SYNC_STATE_URL);
  const relayEnabled = isChainExtrinsicRelayEnabled(env);

  return {
    ok: true,
    schema: 'citizenapp.chain.bootstrap.v1',
    generated_at: Date.now(),
    cache_ttl_seconds: cacheTtlSeconds,
    chain: {
      chain_id: 'citizenchain',
      chain_name: 'CitizenChain',
      chain_type: 'Live',
      protocol_id: 'citizenchain',
      genesis_hash: normalizeHex32(env.CITIZEN_CHAIN_GENESIS_HASH, DEFAULT_GENESIS_HASH),
      state_root: normalizeHex32(env.CITIZEN_CHAIN_STATE_ROOT, DEFAULT_STATE_ROOT),
      ss58_format: 2027,
      token_symbol: 'GMB',
      token_decimals: 2
    },
    light_client: {
      mode: 'smoldot',
      truth_source: 'p2p_finalized_storage',
      api_is_truth: false,
      bundled_assets_required: ['assets/chainspec.json', 'assets/light_sync_state.json'],
      checkpoint: {
        source: lightSyncStateUrl ? 'remote_url' : 'bundled_asset',
        light_sync_state_url: lightSyncStateUrl,
        light_sync_state_sha256:
          normalizeSha256(env.CITIZEN_CHAIN_LIGHT_SYNC_STATE_SHA256) ??
          DEFAULT_LIGHT_SYNC_STATE_SHA256
      }
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

function normalizePublicUrl(value: string | undefined): string | null {
  if (!value) {
    return null;
  }
  try {
    const url = new URL(value);
    if (url.protocol !== 'https:') {
      return null;
    }
    return url.toString();
  } catch {
    return null;
  }
}

function normalizeHex32(value: string | undefined, fallback: string): string {
  if (!value) {
    return fallback;
  }
  return /^0x[0-9a-fA-F]{64}$/.test(value) ? value.toLowerCase() : fallback;
}

function normalizeSha256(value: string | undefined): string | null {
  if (!value) {
    return null;
  }
  return /^[0-9a-fA-F]{64}$/.test(value) ? value.toLowerCase() : null;
}
