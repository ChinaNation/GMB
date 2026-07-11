import { afterEach, describe, expect, it, vi } from 'vitest';
import { createProviderUpload, streamDetailsToAssetUpdate } from '../src/media/cloudflare_assets';
import type { Env } from '../src/types';

describe('Cloudflare media assets', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('creates a local Images direct upload plan in dev mode', async () => {
    const plan = await createProviderUpload(devEnv(), {
      ownerAccount: 'owner',
      uploadId: 'squ_dev',
      postId: 'sqp_dev',
      mediaIndex: 0,
      mediaKind: 'image',
      contentType: 'image/webp',
      byteSize: 1024,
      maxDurationSeconds: 60,
      requestOrigin: 'http://127.0.0.1:8787'
    });

    expect(plan).toMatchObject({
      provider: 'cloudflare_images',
      provider_asset_id: 'img_squ_dev_0',
      upload_method: 'direct_form',
      asset_state: 'prepared'
    });
    expect(plan.upload_url).toContain('/v1/square/uploads/dev-media');
  });

  it('uses Stream tus direct upload for videos over 200MB', async () => {
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
      byteSize: 201 * 1024 * 1024,
      maxDurationSeconds: 10_800,
      requestOrigin: 'https://worker.test'
    });

    expect(plan).toMatchObject({
      provider: 'cloudflare_stream',
      provider_asset_id: 'stream_uid',
      upload_method: 'tus',
      upload_url: 'https://upload.videodelivery.net/tus/stream_uid',
      playback_hls_url: 'https://customer.example/stream_uid/manifest/video.m3u8'
    });
    expect(fetch).toHaveBeenCalledWith(
      'https://api.cloudflare.com/client/v4/accounts/acct_123/stream?direct_user=true',
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({
          'tus-resumable': '1.0.0',
          'upload-length': String(201 * 1024 * 1024)
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
      playback_hls_url: 'https://play.test/video.m3u8',
      thumbnail_url: 'https://thumb.test/1.jpg',
      duration_seconds: 5.5,
      width: 560,
      height: 320
    });
  });
});

function devEnv(): Env {
  return {
    DEV_UPLOAD_PROXY: '1'
  } as unknown as Env;
}

function prodEnv(): Env {
  return {
    CF_ACCOUNT_ID: 'acct_123',
    CF_API_TOKEN: 'token',
    IMAGES_URL: 'https://imagedelivery.net/account',
    STREAM_URL: 'https://customer.example'
  } as unknown as Env;
}
