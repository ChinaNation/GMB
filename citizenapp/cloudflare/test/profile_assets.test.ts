import { describe, expect, it } from 'vitest';
import { prepareProfileAsset } from '../src/profiles/assets';
import type { Env, SessionState } from '../src/types';

const accountId = '0x1111111111111111111111111111111111111111111111111111111111111111';
const sha = 'a'.repeat(64);

function fakeEnv(): Env {
  const session: SessionState = {
    account_id: accountId,
    device_key_hash: 'a'.repeat(64),
    created_at: 0,
    expires_at: Date.now() + 60_000
  };
  return {
    SQUARE_CACHE: {
      get: async (key: string) =>
        key === 'square_session:tok' ? session : null
    } as unknown as KVNamespace,
  } as unknown as Env;
}

function prepareRequest(body: unknown): Request {
  return new Request('https://worker/v1/square/profile/assets/prepare', {
    method: 'POST',
    headers: {
      authorization: 'Bearer tok',
      'content-type': 'application/json'
    },
    body: JSON.stringify(body)
  });
}

describe('profile asset upload prepare', () => {
  it('returns the fixed per-accountId avatar key and hash-bound upload URL', async () => {
    const response = await prepareProfileAsset(
      prepareRequest({
        kind: 'avatar',
        content_type: 'image/webp',
        byte_size: 1024,
        sha256: sha
      }),
      fakeEnv()
    );
    const body = (await response.json()) as {
      object_key: string;
      content_hash: string;
      upload_url: string;
    };

    expect(body.object_key).toBe(`profile/${accountId.slice(2)}/avatar`);
    expect(body.content_hash).toBe(sha);
    expect(body.upload_url).toContain('/v1/square/profile/assets?');
    expect(new URL(body.upload_url).searchParams.get('sha256')).toBe(sha);
  });

  it('rejects an unsupported content type', async () => {
    await expect(
      prepareProfileAsset(
        prepareRequest({
          kind: 'avatar',
          content_type: 'image/gif',
          byte_size: 10,
          sha256: sha
        }),
        fakeEnv()
      )
    ).rejects.toMatchObject({ code: 'resource_content_type_invalid' });
  });

  it('rejects an invalid kind', async () => {
    await expect(
      prepareProfileAsset(
        prepareRequest({
          kind: 'other',
          content_type: 'image/png',
          byte_size: 10,
          sha256: sha
        }),
        fakeEnv()
      )
    ).rejects.toMatchObject({ code: 'invalid_asset_kind' });
  });
});
