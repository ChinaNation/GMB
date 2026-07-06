import { describe, expect, it } from 'vitest';
import { buildLoginPayload } from '../src/auth/wallet_signature';

describe('wallet login payload', () => {
  it('uses a stable domain-separated message', () => {
    const payload = buildLoginPayload({
      owner_account: '5GrwvaEF5zXb26Fz9rcQpDWS7u4m6DXb6T6TQvF9j5uQ8g6U',
      challenge_id: 'sqc_001',
      expires_at: 1_800_000
    });

    expect(payload).toBe(
      [
        'GMB_SQUARE_LOGIN_V1',
        'owner_account:5GrwvaEF5zXb26Fz9rcQpDWS7u4m6DXb6T6TQvF9j5uQ8g6U',
        'challenge_id:sqc_001',
        'expires_at:1800000'
      ].join('\n')
    );
  });
});
