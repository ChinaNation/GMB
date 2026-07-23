import { describe, expect, it } from 'vitest';
import { accountIdPathSegment, buildObjectKeyPlan } from '../src/storage/r2_keys';

describe('R2 object key plan', () => {
  it('keeps the square manifest under the wallet-owned post directory', () => {
    const accountId = `0x${'ab'.repeat(32)}`;
    const plan = buildObjectKeyPlan(accountId, 'sqp_abc');

    expect(accountIdPathSegment(accountId)).toBe('ab'.repeat(32));
    expect(plan.manifest_object_key).toBe(`square/${'ab'.repeat(32)}/posts/sqp_abc/manifest.json`);
    expect(plan.object_keys).toEqual([`square/${'ab'.repeat(32)}/posts/sqp_abc/manifest.json`]);
  });

  it('rejects non-canonical account IDs instead of sanitizing them', () => {
    expect(() => accountIdPathSegment('wallet/account:001')).toThrow();
    expect(() => accountIdPathSegment(`0X${'ab'.repeat(32)}`)).toThrow();
    expect(() => accountIdPathSegment(`0x${'AB'.repeat(32)}`)).toThrow();
  });
});
