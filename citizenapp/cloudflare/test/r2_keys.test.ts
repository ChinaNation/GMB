import { describe, expect, it } from 'vitest';
import { buildObjectKeyPlan, sanitizeOwnerAccount } from '../src/storage/r2_keys';

describe('R2 object key plan', () => {
  it('keeps the square manifest under the wallet-owned post directory', () => {
    const plan = buildObjectKeyPlan('wallet/account:001', 'sqp_abc');

    expect(sanitizeOwnerAccount('wallet/account:001')).toBe('wallet_account_001');
    expect(plan.manifest_object_key).toBe('square/wallet_account_001/posts/sqp_abc/manifest.json');
    expect(plan.object_keys).toEqual(['square/wallet_account_001/posts/sqp_abc/manifest.json']);
  });
});
