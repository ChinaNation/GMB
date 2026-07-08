import { describe, expect, it } from 'vitest';
import { HttpError } from '../src/shared/http';
import { createStorageReceiptId, estimateUploadBytes, validateUploadItems } from '../src/uploads/service';

describe('upload validation', () => {
  it('accepts supported image and video content types', () => {
    const items = validateUploadItems([
      { media_kind: 'image', content_type: 'image/png', byte_size: 1024 },
      { media_kind: 'video', content_type: 'video/mp4', byte_size: 2048 }
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
});
