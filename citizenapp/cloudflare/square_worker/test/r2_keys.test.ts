import { describe, expect, it } from 'vitest';
import { buildObjectKeyPlan, sanitizeOwnerAccount } from '../src/storage/r2_keys';

describe('R2 object key plan', () => {
  it('keeps every square object under the wallet-owned post directory', () => {
    const plan = buildObjectKeyPlan('wallet/account:001', 'sqp_abc', [
      { media_kind: 'image', content_type: 'image/webp', byte_size: 100 },
      { media_kind: 'video', content_type: 'video/mp4', byte_size: 200 },
      { media_kind: 'cover', content_type: 'image/jpeg', byte_size: 50 }
    ]);

    expect(sanitizeOwnerAccount('wallet/account:001')).toBe('wallet_account_001');
    expect(plan.manifest_object_key).toBe('square/wallet_account_001/posts/sqp_abc/manifest.json');
    expect(plan.object_keys).toEqual([
      'square/wallet_account_001/posts/sqp_abc/manifest.json',
      'square/wallet_account_001/posts/sqp_abc/media_001.webp',
      'square/wallet_account_001/posts/sqp_abc/video_001.mp4',
      'square/wallet_account_001/posts/sqp_abc/cover.jpg'
    ]);
  });
});
