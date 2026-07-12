import { describe, expect, it } from 'vitest';
import { mediaRoute } from '../src/media/service';
import type { Env } from '../src/types';

class FakeR2 {
  constructor(
    private readonly objects: Record<string, { body: string; contentType: string }>
  ) {}

  async get(key: string) {
    const object = this.objects[key];
    if (!object) return null;
    return {
      body: object.body,
      size: new TextEncoder().encode(object.body).byteLength,
      httpMetadata: { contentType: object.contentType },
      httpEtag: '"etag"'
    };
  }
}

function fakeEnv(
  objects: Record<string, { body: string; contentType: string }>
): Env {
  return {
    SQUARE_MEDIA: new FakeR2(objects) as unknown as R2Bucket,
    SQUARE_CACHE: {
      get: async (key: string) => key === 'square_session:test' ? {
        owner_account: '5GrwvaEF5zXb26Fz9rcQpDWS7u4m6DXb6T6TQvF9j5uQ8g6U',
        created_at: 0,
        expires_at: Date.now() + 60_000
      } : null
    } as unknown as KVNamespace
  } as unknown as Env;
}

function call(env: Env, path: string) {
  return mediaRoute(new Request(`https://worker${path}`, {
    headers: { authorization: 'Bearer test' }
  }), env, path);
}

describe('media read channel', () => {
  it('streams a stored object with its content type', async () => {
    const key = 'profile/acct/avatar';
    const env = fakeEnv({
      [key]: { body: 'IMG', contentType: 'image/webp' }
    });
    const response = await call(env, `/v1/square/media/${key}`);

    expect(response.status).toBe(200);
    expect(response.headers.get('content-type')).toBe('image/webp');
    expect(response.headers.get('cache-control')).toBe('private, no-store');
    expect(await response.text()).toBe('IMG');
  });

  it('404s a missing object', async () => {
    const key = 'profile/a/avatar';
    await expect(
      call(fakeEnv({}), `/v1/square/media/${key}`)
    ).rejects.toMatchObject({ status: 404 });
  });

  it('rejects keys outside the profile prefix', async () => {
    await expect(
      call(fakeEnv({}), '/v1/square/media/secret/keys.txt')
    ).rejects.toMatchObject({ code: 'invalid_media_key' });
  });
});
