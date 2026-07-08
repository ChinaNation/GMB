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
      httpMetadata: { contentType: object.contentType },
      httpEtag: '"etag"'
    };
  }
}

function fakeEnv(
  objects: Record<string, { body: string; contentType: string }>
): Env {
  return {
    SQUARE_MEDIA: new FakeR2(objects) as unknown as R2Bucket
  } as unknown as Env;
}

function call(env: Env, path: string) {
  return mediaRoute(new Request(`https://worker${path}`), env, path);
}

describe('media read channel', () => {
  it('streams a stored object with its content type', async () => {
    const env = fakeEnv({
      'profile/acct/avatar.webp': { body: 'IMG', contentType: 'image/webp' }
    });
    const response = await call(env, '/v1/square/media/profile/acct/avatar.webp');

    expect(response.status).toBe(200);
    expect(response.headers.get('content-type')).toBe('image/webp');
    expect(response.headers.get('cache-control')).toContain('public');
    expect(await response.text()).toBe('IMG');
  });

  it('404s a missing object', async () => {
    await expect(
      call(fakeEnv({}), '/v1/square/media/profile/a/avatar.webp')
    ).rejects.toMatchObject({ status: 404 });
  });

  it('rejects keys outside the profile prefix', async () => {
    await expect(
      call(fakeEnv({}), '/v1/square/media/secret/keys.txt')
    ).rejects.toMatchObject({ code: 'invalid_media_key' });
  });
});
