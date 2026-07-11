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
        CHAIN_URL: 'https://rpc.internal.example',
        CHAIN_ID: 'worker-rpc.access',
        CHAIN_SECRET: 'test-access-secret'
      })
    );

    expect(response.schema).toBe('citizenapp.chain.bootstrap.v2');
    expect(response.chain.ss58_format).toBe(2027);
    expect(response.light_client.mode).toBe('smoldot');
    expect(response.light_client.api_is_truth).toBe(false);
    expect(response.light_client).toEqual({
      mode: 'smoldot',
      truth_source: 'p2p_finalized_storage',
      api_is_truth: false,
      bundled_assets_required: ['assets/chainspec.json', 'assets/light_sync_state.json']
    });
    expect(response.p2p.bootnodes).toEqual([bootnodeA, bootnodeB]);
    expect(response.services.square_base_url).toBe('https://api.onchina.org/v1/square');
    expect(response.services.chat_base_url).toBe('https://api.onchina.org/v1/chat');
    expect(response.services.signed_extrinsic_relay.enabled).toBe(false);
    expect(response.services.signed_extrinsic_relay.path).toBeNull();
    expect(response.security.rpc_proxy).toBe(false);
    const serialized = JSON.stringify(response);
    expect(serialized).not.toContain('rpc.internal.example');
    expect(serialized).not.toContain('checkpoint');
    expect(serialized).not.toContain('light_sync_state_url');
    expect(serialized).not.toContain('light_sync_state_sha256');
  });

  it('exposes only the signed extrinsic relay path when the relay is enabled', () => {
    const response = buildChainBootstrapResponse(
      new Request('https://api.onchina.org/v1/chain/bootstrap'),
      env({
        CHAIN_EXTRINSIC_RELAY_ENABLED: '1',
        CHAIN_URL: 'https://rpc.internal.example',
        CHAIN_ID: 'worker-rpc.access',
        CHAIN_SECRET: 'test-access-secret'
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
        CHAIN_URL: 'https://rpc.internal.example',
        CHAIN_ID: 'worker-rpc.access'
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
    expect(Object.keys(response.light_client).sort()).toEqual(
      ['api_is_truth', 'bundled_assets_required', 'mode', 'truth_source'].sort()
    );
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
      schema: 'citizenapp.chain.bootstrap.v2'
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
