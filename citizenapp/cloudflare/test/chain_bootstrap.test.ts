import { describe, expect, it } from 'vitest';
import { buildChainBootstrapResponse } from '../src/chain/bootstrap';
import { routeRequest } from '../src/routes';
import type { Env } from '../src/types';

const bootnodeA =
  '/dns4/nrcgch.crcfrcn.com/tcp/30333/wss/p2p/12D3KooWHepcMGD3h9VC1XNWmrac3pXo63RimV5jhTU2nC2TLAyS';
const bootnodeB =
  '/dns4/prczss.crcfrcn.com/tcp/30333/wss/p2p/12D3KooWPjWNXvCzPv6PPuiGnF3J5uToW3ySfaB7rKkwUrN2CALv';

describe('chain bootstrap manifest', () => {
  it('returns a light-node bootstrap manifest without exposing RPC', () => {
    const response = buildChainBootstrapResponse(
      new Request('https://api.onchina.org/v1/chain/bootstrap'),
      env({
        CITIZEN_CHAIN_BOOTNODES: `${bootnodeA}\n${bootnodeB}\n${bootnodeA}`,
        CITIZEN_CHAIN_BOOTSTRAP_TTL_SECONDS: '120',
        CITIZEN_CHAIN_LIGHT_SYNC_STATE_URL:
          'https://api.onchina.org/v1/chain/light-sync-state.json',
        CITIZEN_CHAIN_RPC_URL: 'https://rpc.internal.example',
        CITIZEN_CHAIN_RPC_ACCESS_CLIENT_ID: 'worker-rpc.access',
        CITIZEN_CHAIN_RPC_ACCESS_CLIENT_SECRET: 'test-access-secret'
      })
    );

    expect(response.schema).toBe('citizenapp.chain.bootstrap.v1');
    expect(response.chain.ss58_format).toBe(2027);
    expect(response.light_client.mode).toBe('smoldot');
    expect(response.light_client.api_is_truth).toBe(false);
    expect(response.light_client.checkpoint.source).toBe('remote_url');
    expect(response.p2p.bootnodes).toEqual([bootnodeA, bootnodeB]);
    expect(response.services.square_base_url).toBe('https://api.onchina.org/v1/square');
    expect(response.services.chat_base_url).toBe('https://api.onchina.org/v1/chat');
    expect(response.services.signed_extrinsic_relay.enabled).toBe(false);
    expect(response.services.signed_extrinsic_relay.path).toBeNull();
    expect(response.security.rpc_proxy).toBe(false);
    expect(JSON.stringify(response)).not.toContain('rpc.internal.example');
  });

  it('exposes only the signed extrinsic relay path when the relay is enabled', () => {
    const response = buildChainBootstrapResponse(
      new Request('https://api.onchina.org/v1/chain/bootstrap'),
      env({
        CHAIN_EXTRINSIC_RELAY_ENABLED: '1',
        CITIZEN_CHAIN_RPC_URL: 'https://rpc.internal.example',
        CITIZEN_CHAIN_RPC_ACCESS_CLIENT_ID: 'worker-rpc.access',
        CITIZEN_CHAIN_RPC_ACCESS_CLIENT_SECRET: 'test-access-secret'
      })
    );

    expect(response.services.signed_extrinsic_relay).toEqual({
      enabled: true,
      path: '/v1/chain/extrinsics/relay'
    });
    expect(JSON.stringify(response)).not.toContain('rpc.internal.example');
  });

  it('keeps the relay disabled when the Access service token is incomplete', () => {
    const response = buildChainBootstrapResponse(
      new Request('https://api.onchina.org/v1/chain/bootstrap'),
      env({
        CHAIN_EXTRINSIC_RELAY_ENABLED: '1',
        CITIZEN_CHAIN_RPC_URL: 'https://rpc.internal.example',
        CITIZEN_CHAIN_RPC_ACCESS_CLIENT_ID: 'worker-rpc.access'
      })
    );

    expect(response.services.signed_extrinsic_relay).toEqual({
      enabled: false,
      path: null
    });
  });

  it('falls back to bundled chainspec bootNodes when Worker config is empty', () => {
    const response = buildChainBootstrapResponse(
      new Request('https://worker.test/v1/chain/bootstrap'),
      env()
    );

    expect(response.p2p.bootnodes).toEqual([]);
    expect(response.p2p.bootnodes_source).toBe('bundled_chainspec');
    expect(response.light_client.checkpoint.source).toBe('bundled_asset');
    expect(response.light_client.checkpoint.light_sync_state_url).toBeNull();
    expect(response.light_client.checkpoint.light_sync_state_sha256).toMatch(/^[0-9a-f]{64}$/);
  });

  it('routes GET /v1/chain/bootstrap with cache headers', async () => {
    const response = await routeRequest(
      new Request('https://api.onchina.org/v1/chain/bootstrap'),
      env({ CITIZEN_CHAIN_BOOTSTRAP_TTL_SECONDS: '90' })
    );

    expect(response.status).toBe(200);
    expect(response.headers.get('cache-control')).toBe('public, max-age=90');
    const body = (await response.json()) as { schema: string; ok: boolean };
    expect(body).toMatchObject({
      ok: true,
      schema: 'citizenapp.chain.bootstrap.v1'
    });
  });
});

function env(overrides: Partial<Env> = {}): Env {
  return {
    DB: {} as D1Database,
    SQUARE_MEDIA: {} as R2Bucket,
    FEED_CACHE: {} as KVNamespace,
    ...overrides
  };
}
