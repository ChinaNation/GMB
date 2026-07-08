import { afterEach, describe, expect, it, vi } from 'vitest';
import { encodeAddress } from '@polkadot/util-crypto';
import { stripeCheckoutRoute } from '../src/membership/checkout';
import type { Env, SessionState } from '../src/types';

const ownerBytes = Uint8Array.from(Array.from({ length: 32 }, (_, index) => index + 41));
const otherBytes = Uint8Array.from(Array.from({ length: 32 }, (_, index) => index + 73));
const owner = encodeAddress(ownerBytes, 2027);
const otherOwner = encodeAddress(otherBytes, 2027);
const sessionToken = 'session_checkout';

describe('stripe checkout route', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('creates visitor checkout without reading chain identity', async () => {
    let capturedBody = '';
    const env = fakeEnv({
      storageResponses: [],
      stripeResponse: { id: 'cs_visitor', url: 'https://checkout.stripe.com/c/pay/cs_visitor' },
      onStripeBody: (body) => {
        capturedBody = body;
      }
    });

    const response = await stripeCheckoutRoute(checkoutRequest('visitor'), env);
    const json = (await response.json()) as {
      checkout_session_id: string;
      checkout_url: string;
      membership_level: string;
    };
    const params = new URLSearchParams(capturedBody);

    expect(json).toMatchObject({
      checkout_session_id: 'cs_visitor',
      checkout_url: 'https://checkout.stripe.com/c/pay/cs_visitor',
      membership_level: 'visitor'
    });
    expect(params.get('mode')).toBe('subscription');
    expect(params.get('line_items[0][price]')).toBe('price_visitor');
    expect(params.get('subscription_data[metadata][owner_account]')).toBe(owner);
    expect(params.get('subscription_data[metadata][membership_level]')).toBe('visitor');
  });

  it('creates candidate checkout only when chain identity is candidate', async () => {
    const env = fakeEnv({
      storageResponses: [votingIdentityHex(), '0x01'],
      stripeResponse: { id: 'cs_candidate', url: 'https://checkout.stripe.com/c/pay/cs_candidate' }
    });

    const response = await stripeCheckoutRoute(checkoutRequest('candidate'), env);
    const json = (await response.json()) as { membership_level: string };

    expect(json.membership_level).toBe('candidate');
  });

  it('rejects candidate checkout when owner only has voting identity', async () => {
    const env = fakeEnv({
      storageResponses: [votingIdentityHex(), null],
      stripeResponse: { id: 'cs_unused', url: 'https://checkout.stripe.com/c/pay/cs_unused' }
    });

    await expect(stripeCheckoutRoute(checkoutRequest('candidate'), env)).rejects.toMatchObject({
      code: 'membership_identity_required'
    });
  });

  it('rejects checkout when bearer session owner differs from request owner', async () => {
    const env = fakeEnv({
      storageResponses: [],
      stripeResponse: { id: 'cs_unused', url: 'https://checkout.stripe.com/c/pay/cs_unused' }
    });

    await expect(
      stripeCheckoutRoute(
        checkoutRequest('visitor', {
          ownerAccount: otherOwner,
          session: true
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'owner_account_mismatch' });
  });

  it('surfaces Stripe checkout errors without writing membership state', async () => {
    const env = fakeEnv({
      storageResponses: [],
      stripeStatus: 400,
      stripeResponse: { error: { message: 'No such price' } }
    });

    await expect(stripeCheckoutRoute(checkoutRequest('visitor'), env)).rejects.toMatchObject({
      code: 'stripe_checkout_failed',
      message: 'No such price'
    });
  });
});

function fakeEnv(input: {
  storageResponses: Array<string | null>;
  stripeStatus?: number;
  stripeResponse: unknown;
  onStripeBody?: (body: string) => void;
}): Env {
  const session: SessionState = {
    owner_account: owner,
    created_at: 0,
    expires_at: Date.now() + 60_000
  };
  const kv = new FakeKv(new Map([[`square_session:${sessionToken}`, session]]));
  const responses = [...input.storageResponses];
  vi.stubGlobal(
    'fetch',
    vi.fn(async (request: RequestInfo | URL, init?: RequestInit) => {
      const url = request.toString();
      if (url.startsWith('https://api.stripe.com/')) {
        input.onStripeBody?.(init?.body?.toString() ?? '');
        return Response.json(input.stripeResponse, { status: input.stripeStatus ?? 200 });
      }
      return Response.json({
        jsonrpc: '2.0',
        id: 1,
        result: responses.shift() ?? null
      });
    })
  );

  return {
    DB: {} as D1Database,
    SQUARE_MEDIA: {} as R2Bucket,
    FEED_CACHE: kv as unknown as KVNamespace,
    SQUARE_CHAIN_RPC_URL: 'http://chain.test',
    STRIPE_SECRET_KEY: 'sk_test_secret',
    STRIPE_PRICE_VISITOR: 'price_visitor',
    STRIPE_PRICE_VOTING: 'price_voting',
    STRIPE_PRICE_CANDIDATE: 'price_candidate',
    CITIZENAPP_MEMBERSHIP_SUCCESS_URL: 'https://example.com/membership?checkout=success',
    CITIZENAPP_MEMBERSHIP_CANCEL_URL: 'https://example.com/membership?checkout=cancel'
  } as unknown as Env;
}

function checkoutRequest(
  membershipLevel: string,
  input: { ownerAccount?: string; session?: boolean } = {}
): Request {
  return new Request('https://w/v1/square/membership/stripe/checkout', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...(input.session ? { authorization: `Bearer ${sessionToken}` } : {})
    },
    body: JSON.stringify({
      owner_account: input.ownerAccount ?? owner,
      membership_level: membershipLevel
    })
  });
}

function votingIdentityHex(): string {
  const cid = new TextEncoder().encode('CN001-CTZN-000000001-2026');
  const body = concat([
    compactU32(cid.length),
    cid,
    u32Le(20200101),
    u32Le(20991231),
    Uint8Array.of(0)
  ]);
  return `0x${hex(body)}`;
}

function compactU32(value: number): Uint8Array {
  if (value < 1 << 6) return Uint8Array.of(value << 2);
  if (value < 1 << 14) {
    const encoded = (value << 2) | 0x01;
    return Uint8Array.of(encoded & 0xff, (encoded >> 8) & 0xff);
  }
  throw new Error('test compact only supports small values');
}

function u32Le(value: number): Uint8Array {
  const out = new Uint8Array(4);
  new DataView(out.buffer).setUint32(0, value, true);
  return out;
}

function concat(chunks: Uint8Array[]): Uint8Array {
  const length = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const out = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.length;
  }
  return out;
}

function hex(bytes: Uint8Array): string {
  return [...bytes].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}

class FakeKv {
  constructor(private readonly store: Map<string, unknown>) {}

  async get<T>(key: string): Promise<T | null> {
    return (this.store.get(key) as T) ?? null;
  }
}
