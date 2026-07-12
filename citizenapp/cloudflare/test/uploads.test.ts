import { describe, expect, it } from 'vitest';
import { assertAllowedOrigin, assertRequestSize, normalizeApiPath } from '../src/security/request_guard';
import { HttpError } from '../src/shared/http';
import type { Env } from '../src/types';
import { createStorageReceiptId, estimateUploadBytes, validateUploadItems } from '../src/uploads/service';

describe('upload validation', () => {
  it('accepts supported image and video content types', () => {
    const items = validateUploadItems([
      { media_kind: 'image', content_type: 'image/png', byte_size: 1024 },
      { media_kind: 'video', content_type: 'video/mp4', byte_size: 2048, duration_seconds: 30 }
    ]);

    expect(items).toHaveLength(2);
    expect(estimateUploadBytes(items)).toBe(512 * 1024 + 3072);
  });

  it('rejects unsupported media content types', () => {
    expect(() =>
      validateUploadItems([{ media_kind: 'video', content_type: 'application/octet-stream', byte_size: 1 }])
    ).toThrow(HttpError);
  });

  it('pre-generates stable storage receipt before external media upload', async () => {
    const input = {
      uploadId: 'squ_test',
      postId: 'sqp_test',
      ownerAccount: 'gmb_owner',
      manifestHash: '11'.repeat(32)
    };

    const first = await createStorageReceiptId(input);
    const second = await createStorageReceiptId(input);

    expect(first).toBe(second);
    expect(first).toMatch(/^sqr_[a-f0-9]{64}$/);
  });

  it('normalizes only the two same-domain API prefixes', () => {
    expect(normalizeApiPath('/api/v1/square/feed')).toBe('/v1/square/feed');
    expect(normalizeApiPath('/api-staging/v1/square/feed')).toBe('/v1/square/feed');
    expect(normalizeApiPath('/v1/square/feed')).toBe('/v1/square/feed');
  });

  it('accepts the exact website origin and rejects lookalike origins', () => {
    const env = { WEB_ORIGIN: 'https://www.crcfrcn.com' } as Env;
    expect(() => assertAllowedOrigin(new Request('https://worker.test', {
      headers: { origin: 'https://www.crcfrcn.com' }
    }), env)).not.toThrow();
    expect(() => assertAllowedOrigin(new Request('https://worker.test', {
      headers: { origin: 'https://www.crcfrcn.com.evil.example' }
    }), env)).toThrowError(HttpError);
  });

  it('rejects oversized API JSON before parsing', () => {
    const request = new Request('https://worker.test/api/v1/square/uploads/prepare', {
      method: 'POST',
      headers: { 'content-length': String(256 * 1024 + 1) }
    });
    expect(() => assertRequestSize(request, '/v1/square/uploads/prepare')).toThrowError(HttpError);
  });
});
