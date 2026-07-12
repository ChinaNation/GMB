import { afterEach, describe, expect, it, vi } from 'vitest';
import { createProviderUpload, streamDetailsToAssetUpdate } from '../src/media/cloudflare_assets';
import { signedImageUrl, signedMediaUrls } from '../src/media/signed_urls';
import type { Env } from '../src/types';

describe('Cloudflare media assets', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('routes image bytes through the bounded Worker upload endpoint', async () => {
    const plan = await createProviderUpload({} as Env, {
      ownerAccount: 'owner',
      uploadId: 'squ_dev',
      postId: 'sqp_dev',
      mediaIndex: 0,
      mediaKind: 'image',
      contentType: 'image/webp',
      byteSize: 1024,
      maxDurationSeconds: 60,
      workerUploadUrl: 'https://worker.test/v1/square/uploads/media?upload_id=squ_dev&media_index=0'
    });

    expect(plan).toMatchObject({
      provider: 'cloudflare_images',
      provider_asset_id: 'pending:squ_dev:0',
      upload_method: 'worker',
      asset_state: 'prepared'
    });
    expect(plan.upload_url).toContain('/v1/square/uploads/media');
  });

  it('uses Stream TUS for every video size', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        new Response(null, {
          status: 201,
          headers: {
            location: 'https://upload.videodelivery.net/tus/stream_uid',
            'stream-media-id': 'stream_uid'
          }
        })
      )
    );

    const plan = await createProviderUpload(prodEnv(), {
      ownerAccount: 'owner',
      uploadId: 'squ_stream',
      postId: 'sqp_stream',
      mediaIndex: 0,
      mediaKind: 'video',
      contentType: 'video/mp4',
      byteSize: 40 * 1024 * 1024,
      maxDurationSeconds: 10_800,
      workerUploadUrl: 'https://worker.test/v1/square/uploads/media'
    });

    expect(plan).toMatchObject({
      provider: 'cloudflare_stream',
      provider_asset_id: 'stream_uid',
      upload_method: 'tus',
      upload_url: 'https://upload.videodelivery.net/tus/stream_uid'
    });
    expect(fetch).toHaveBeenCalledWith(
      'https://api.cloudflare.com/client/v4/accounts/acct_123/stream?direct_user=true',
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({
          'tus-resumable': '1.0.0',
          'upload-length': String(40 * 1024 * 1024)
        })
      })
    );
  });

  it('maps Stream ready webhook details into media asset fields', () => {
    const update = streamDetailsToAssetUpdate(
      prodEnv(),
      {
        uid: 'stream_uid',
        readyToStream: true,
        thumbnail: 'https://thumb.test/1.jpg',
        duration: 5.5,
        input: { width: 560, height: 320 },
        status: { state: 'ready' },
        playback: {
          hls: 'https://play.test/video.m3u8',
          dash: 'https://play.test/video.mpd'
        }
      },
      'stream_uid'
    );

    expect(update).toMatchObject({
      asset_state: 'ready',
      duration_seconds: 5.5,
      width: 560,
      height: 320
    });
  });

  it('signs Images delivery URLs with a short expiry', async () => {
    const nowSeconds = Math.floor(Date.now() / 1000);
    const url = new URL(await signedImageUrl({
      IMAGES_URL: 'https://imagedelivery.net/account',
      IMAGES_SIGNING_KEY: 'test-signing-key',
      MEDIA_TTL_SECONDS: '300'
    } as Env, 'image id'));

    expect(url.pathname).toBe('/account/image%20id/public');
    expect(Number(url.searchParams.get('exp'))).toBeGreaterThanOrEqual(nowSeconds + 299);
    expect(url.searchParams.get('sig')).toMatch(/^[a-f0-9]{64}$/);
  });

  it('uses the Stream binding token instead of a public video id', async () => {
    const urls = await signedMediaUrls({
      STREAM_URL: 'https://customer.example',
      STREAM: {
        video: (uid: string) => ({
          generateToken: async () => `signed-${uid}`
        })
      }
    } as unknown as Env, {
      provider: 'cloudflare_stream',
      provider_asset_id: 'stream_uid'
    });

    expect(urls.url).toBe('https://customer.example/signed-stream_uid/manifest/video.m3u8');
    expect(urls.thumbnail_url).toBe(
      'https://customer.example/signed-stream_uid/thumbnails/thumbnail.jpg'
    );
    expect(urls.url).not.toContain('/stream_uid/');
  });
});

function prodEnv(): Env {
  return {
    CF_ACCOUNT_ID: 'acct_123',
    CF_API_TOKEN: 'token',
    IMAGES_URL: 'https://imagedelivery.net/account',
    STREAM_URL: 'https://customer.example'
  } as unknown as Env;
}
