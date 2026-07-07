import { describe, expect, it } from 'vitest';
import { prepareProfileAsset } from '../src/profiles/assets';
import type { Env, SessionState } from '../src/types';

const owner = '5GrwvaEF5zXb26Fz9rcQpDWS7u4m6DXb6T6TQvF9j5uQ8g6U';
const sha = 'a'.repeat(64);

function fakeEnv(): Env {
  const session: SessionState = {
    owner_account: owner,
    created_at: 0,
    expires_at: Date.now() + 60_000
  };
  return {
    FEED_CACHE: {
      get: async (key: string) =>
        key === 'square_session:tok' ? session : null
    } as unknown as KVNamespace,
    SQUARE_DEV_UPLOAD_PROXY: '1'
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
  it('returns an object key under the owner profile prefix with sha in the name', async () => {
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

    expect(body.object_key).toBe(`profile/${owner}/avatar_${sha}.webp`);
    expect(body.content_hash).toBe(sha);
    expect(body.upload_url).toContain('/v1/square/profile/assets/dev-put');
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
    ).rejects.toMatchObject({ code: 'invalid_content_type' });
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
