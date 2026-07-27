import { blake2AsU8a } from '@polkadot/util-crypto';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Env, SessionState } from '../src/types';
import {
  topupConfigRoute,
  topupConfirmRoute,
  topupIntentRoute,
  topupStatusRoute,
} from '../src/topup/orders';
import {
  topupClaimRoute,
  topupExceptionRoute,
  topupPendingRoute,
  topupSettledRoute,
} from '../src/topup/settlement';

const TRANSFER_TOPIC = '0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef';
const USDC_FIXTURE = '0x833589fcd6edb6e08f4c7c32d4f71b54bda02913';
const RECV = `0x${'ab'.repeat(20)}`;
const PAYER = `0x${'cd'.repeat(20)}`;
const ACCOUNT_ID = `0x${'77'.repeat(32)}`;
const OTHER_ACCOUNT_ID = `0x${'66'.repeat(32)}`;
const DISBURSE_ACCOUNT_ID = `0x${'55'.repeat(32)}`;
const TX_HASH = `0x${'11'.repeat(32)}`;
const BLOCK_HASH = `0x${'33'.repeat(32)}`;
const GENESIS_HASH = `0x${'44'.repeat(32)}`;

describe('topup 稳定币充值后端', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('config 仅返回已配置公开报价', async () => {
    const response = await topupConfigRoute(
      new Request('https://x.test/v1/square/topup/config'),
      makeEnv(new FakeDb()),
    );
    const body = await response.json<{ rails: { token: string; chain_id: number }[]; packages: unknown[] }>();
    expect(body.rails.map((rail) => rail.token)).toEqual(['USDC', 'USDT']);
    expect(body.rails[0].chain_id).toBe(8453);
    expect(body.packages).toHaveLength(2);
  });

  it('intent 把会话 account_id 与 payer、报价绑定，客户端不能指定目标账户', async () => {
    const env = makeEnv(new FakeDb());
    const intent = await createIntent(env, ACCOUNT_ID);
    const payload = JSON.parse(Buffer.from(intent.split('.')[0], 'base64url').toString()) as {
      account_id: string;
      payer_address: string;
      pay_amount: string;
    };
    expect(payload).toMatchObject({
      account_id: ACCOUNT_ID,
      payer_address: PAYER,
      pay_amount: '15000000',
    });
  });

  it('confirm 篡改 intent → 拒绝且不访问 EVM', async () => {
    const env = makeEnv(new FakeDb());
    const intent = await createIntent(env, ACCOUNT_ID);
    const fetch = vi.fn();
    vi.stubGlobal('fetch', fetch);
    await expect(confirm(env, `${intent.slice(0, -1)}x`, ACCOUNT_ID)).rejects.toMatchObject({
      code: 'topup_intent_invalid',
    });
    expect(fetch).not.toHaveBeenCalled();
  });

  it('confirm 足额 finalized 到账 → 创建 pending 订单并保持幂等', async () => {
    const db = new FakeDb();
    const env = makeEnv(db);
    const intent = await createIntent(env, ACCOUNT_ID);
    vi.stubGlobal('fetch', rpcFetch({ signedExtrinsicHex: '' }));

    const first = await (await confirm(env, intent, ACCOUNT_ID)).json<{ status: string; order_id: string }>();
    const second = await (await confirm(env, intent, ACCOUNT_ID)).json<{ status: string; order_id: string }>();
    expect(first.status).toBe('pending');
    expect(second.order_id).toBe(first.order_id);
    expect(db.rows.size).toBe(1);
    expect(db.rows.get(first.order_id)).toMatchObject({
      account_id: ACCOUNT_ID,
      payer_address: PAYER,
      coin_fen: '1000000',
    });
  });

  it('status 只允许订单所属会话账户读取', async () => {
    const db = new FakeDb();
    const env = makeEnv(db);
    const intent = await createIntent(env, ACCOUNT_ID);
    vi.stubGlobal('fetch', rpcFetch({ signedExtrinsicHex: '' }));
    const orderId = (await (await confirm(env, intent, ACCOUNT_ID)).json<{ order_id: string }>()).order_id;
    await expect(
      topupStatusRoute(sessionGet(`https://x.test/v1/square/topup/status/${orderId}`, OTHER_ACCOUNT_ID), env, orderId),
    ).rejects.toMatchObject({ code: 'topup_order_not_found' });
  });

  it('claim 原子抢占且不自动过期，第二个结算流程不能重复抢占', async () => {
    const { env, orderId } = await preparedOrder();
    const response = await topupClaimRoute(settlePost(`https://x.test/v1/square/topup/settlement/${orderId}/claim`, {}), env, orderId);
    const claim = await response.json<{ claim_id: string }>();
    expect(claim.claim_id).toMatch(/^tpc_[0-9a-f]{32}$/);
    await expect(
      topupClaimRoute(settlePost(`https://x.test/v1/square/topup/settlement/${orderId}/claim`, {}), env, orderId),
    ).rejects.toMatchObject({ code: 'topup_order_already_claimed' });
  });

  it('settled 必须同时通过 EVM 与 finalized CitizenChain 完整交易证明', async () => {
    const { env, db, orderId } = await preparedOrder();
    const claim = await (
      await topupClaimRoute(settlePost(`https://x.test/v1/square/topup/settlement/${orderId}/claim`, {}), env, orderId)
    ).json<{ claim_id: string }>();
    const signedExtrinsicHex = makeDisbursementExtrinsic(orderId);
    const gmbTxHash = `0x${Buffer.from(blake2AsU8a(hexBytes(signedExtrinsicHex), 256)).toString('hex')}`;
    vi.stubGlobal('fetch', rpcFetch({ signedExtrinsicHex }));

    const response = await topupSettledRoute(
      settlePost(`https://x.test/v1/square/topup/settlement/${orderId}/settled`, {
        claim_id: claim.claim_id,
        gmb_tx_hash: gmbTxHash,
        gmb_block_hash: BLOCK_HASH,
        gmb_extrinsic_index: 0,
        signed_extrinsic_hex: signedExtrinsicHex,
      }),
      env,
      orderId,
    );
    expect((await response.json<{ status: string }>()).status).toBe('paid');
    expect(db.rows.get(orderId)).toMatchObject({
      status: 'paid',
      gmb_tx_hash: gmbTxHash,
      gmb_block_hash: BLOCK_HASH,
      gmb_extrinsic_index: 0,
    });
  });

  it('exception 没有匹配 claim 时 fail-closed', async () => {
    const { env, orderId } = await preparedOrder();
    await expect(
      topupExceptionRoute(
        settlePost(`https://x.test/v1/square/topup/settlement/${orderId}/exception`, { reason: 'bad' }),
        env,
        orderId,
      ),
    ).rejects.toMatchObject({ code: 'topup_claim_mismatch' });
  });

  it('pending 队列只暴露是否已 claim，不泄露 claim_id', async () => {
    const { env, orderId } = await preparedOrder();
    await topupClaimRoute(settlePost(`https://x.test/v1/square/topup/settlement/${orderId}/claim`, {}), env, orderId);
    const response = await topupPendingRoute(settleGet('https://x.test/v1/square/topup/settlement/pending'), env);
    const body = await response.json<{ orders: Record<string, unknown>[] }>();
    expect(body.orders[0].settlement_claimed).toBe(true);
    expect(body.orders[0]).not.toHaveProperty('settlement_claim_id');
  });
});

async function preparedOrder(): Promise<{ env: Env; db: FakeDb; orderId: string }> {
  const db = new FakeDb();
  const env = makeEnv(db);
  const intent = await createIntent(env, ACCOUNT_ID);
  vi.stubGlobal('fetch', rpcFetch({ signedExtrinsicHex: '' }));
  const orderId = (await (await confirm(env, intent, ACCOUNT_ID)).json<{ order_id: string }>()).order_id;
  return { env, db, orderId };
}

async function createIntent(env: Env, accountId: string): Promise<string> {
  const response = await topupIntentRoute(
    sessionPost('https://x.test/v1/square/topup/intent', {
      token: 'USDC',
      package_id: 'pkg_15',
      payer_address: PAYER,
      account_id: OTHER_ACCOUNT_ID,
    }, accountId),
    env,
  );
  return (await response.json<{ payment_intent: string }>()).payment_intent;
}

function confirm(env: Env, paymentIntent: string, accountId: string): Promise<Response> {
  return topupConfirmRoute(
    sessionPost('https://x.test/v1/square/topup/confirm', {
      payment_intent: paymentIntent,
      evm_tx_hash: TX_HASH,
    }, accountId),
    env,
  );
}

function sessionPost(url: string, body: unknown, accountId: string): Request {
  return new Request(url, {
    method: 'POST',
    headers: { authorization: `Bearer session-${accountId}`, 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
}

function sessionGet(url: string, accountId: string): Request {
  return new Request(url, { headers: { authorization: `Bearer session-${accountId}` } });
}

function settleGet(url: string): Request {
  return new Request(url, { headers: { authorization: 'Bearer settle-secret' } });
}

function settlePost(url: string, body: unknown): Request {
  return new Request(url, {
    method: 'POST',
    headers: { authorization: 'Bearer settle-secret', 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
}

function rpcFetch({ signedExtrinsicHex }: { signedExtrinsicHex: string }) {
  return vi.fn(async (_url: string, init: RequestInit) => {
    const body = JSON.parse(init.body as string) as { method: string; params: unknown[]; id: number };
    if (body.method === 'eth_getTransactionReceipt') {
      return Response.json({ jsonrpc: '2.0', id: 1, result: confirmedReceipt() });
    }
    if (body.method === 'eth_getBlockByNumber') {
      return Response.json({ jsonrpc: '2.0', id: 1, result: { number: '0x20' } });
    }
    if (body.method === 'chain_getFinalizedHead') {
      return Response.json({ jsonrpc: '2.0', id: body.id, result: BLOCK_HASH });
    }
    if (body.method === 'chain_getHeader') {
      return Response.json({ jsonrpc: '2.0', id: body.id, result: { number: '0x10' } });
    }
    if (body.method === 'chain_getBlock') {
      return Response.json({
        jsonrpc: '2.0',
        id: body.id,
        result: { block: { header: { number: '0x10' }, extrinsics: [signedExtrinsicHex] } },
      });
    }
    if (body.method === 'chain_getBlockHash') {
      const result = body.params[0] === 0 ? GENESIS_HASH : BLOCK_HASH;
      return Response.json({ jsonrpc: '2.0', id: body.id, result });
    }
    return Response.json({ jsonrpc: '2.0', id: body.id, result: null });
  });
}

function confirmedReceipt(): unknown {
  return {
    status: '0x1',
    blockNumber: '0x10',
    logs: [{
      address: USDC_FIXTURE,
      topics: [TRANSFER_TOPIC, addrTopic(PAYER), addrTopic(RECV)],
      data: `0x${15000000n.toString(16).padStart(64, '0')}`,
    }],
  };
}

function addrTopic(address: string): string {
  return `0x${'0'.repeat(24)}${address.slice(2)}`;
}

function makeDisbursementExtrinsic(orderId: string): string {
  const body = [
    0x84,
    0x00,
    ...hexBytes(DISBURSE_ACCOUNT_ID),
    0x01,
    ...new Uint8Array(64),
    0x00,
    0x00,
    0x00,
    4,
    0,
    ...hexBytes(ACCOUNT_ID),
    ...u128Le(1000000n),
    ...scaleBytes(new TextEncoder().encode(`topup:${orderId}`)),
  ];
  const encoded = Uint8Array.from([...compact(BigInt(body.length)), ...body]);
  return `0x${Buffer.from(encoded).toString('hex')}`;
}

function compact(value: bigint): number[] {
  if (value < 64n) return [Number(value << 2n)];
  if (value < 16384n) {
    const encoded = Number((value << 2n) | 1n);
    return [encoded & 0xff, encoded >> 8];
  }
  throw new Error('test compact too large');
}

function scaleBytes(bytes: Uint8Array): number[] {
  return [...compact(BigInt(bytes.length)), ...bytes];
}

function u128Le(value: bigint): number[] {
  const bytes: number[] = [];
  for (let index = 0; index < 16; index += 1) {
    bytes.push(Number(value & 0xffn));
    value >>= 8n;
  }
  return bytes;
}

function hexBytes(value: string): Uint8Array {
  return Uint8Array.from(Buffer.from(value.slice(2), 'hex'));
}

function makeEnv(db: FakeDb): Env {
  return {
    DB: db,
    SQUARE_CACHE: {
      get: async (key: string) => {
        const accountId = key.slice('square_session:session-'.length);
        return {
          account_id: accountId,
          device_key_hash: 'device',
          created_at: Date.now(),
          expires_at: Date.now() + 60000,
        } satisfies SessionState;
      },
    },
    TOPUP_NETWORK: 'mainnet',
    TOPUP_RECV_ADDRESS: RECV,
    TOPUP_BASE_RPC_URL: 'https://base-mainnet.example',
    TOPUP_SETTLE_TOKEN: 'settle-secret',
    TOPUP_INTENT_SECRET: 'intent-secret-that-is-longer-than-thirty-two-bytes',
    TOPUP_DISBURSE_ACCOUNT_ID: DISBURSE_ACCOUNT_ID,
    CHAIN_URL: 'https://chain.test',
    CHAIN_ID: 'access-id',
    CHAIN_SECRET: 'access-secret',
    CHAIN_GENESIS_HASH: GENESIS_HASH,
  } as unknown as Env;
}

interface Row {
  order_id: string;
  intent_id: string;
  chain_id: number;
  token: string;
  token_contract: string;
  evm_tx_hash: string;
  payer_address: string;
  recv_address: string;
  pay_amount: string;
  account_id: string;
  coin_fen: string;
  package_id: string;
  status: 'pending' | 'paid' | 'exception';
  settlement_claim_id: string | null;
  settlement_claimed_at: number | null;
  gmb_tx_hash: string | null;
  gmb_block_hash: string | null;
  gmb_extrinsic_index: number | null;
  exception_reason: string | null;
  confirmed_at: number;
  settled_at: number | null;
}

class FakeDb {
  rows = new Map<string, Row>();
  prepare(sql: string) { return new FakeStmt(this, sql); }
}

class FakeStmt {
  private args: unknown[] = [];
  constructor(private readonly db: FakeDb, private readonly sql: string) {}
  bind(...args: unknown[]) { this.args = args; return this; }

  async first<T>(): Promise<T | null> {
    if (this.sql.includes('WHERE chain_id = ? AND evm_tx_hash = ?')) {
      const [chainId, txHash] = this.args as [number, string];
      return ([...this.db.rows.values()].find((row) => row.chain_id === chainId && row.evm_tx_hash === txHash) ?? null) as T | null;
    }
    if (this.sql.includes('WHERE intent_id = ?')) {
      return ([...this.db.rows.values()].find((row) => row.intent_id === this.args[0]) ?? null) as T | null;
    }
    if (this.sql.includes('WHERE order_id = ?')) {
      return (this.db.rows.get(this.args[0] as string) ?? null) as T | null;
    }
    return null;
  }

  async run(): Promise<{ meta: { changes: number } }> {
    if (this.sql.includes('INSERT OR IGNORE INTO topup_orders')) {
      const [orderId, intentId, chainId, token, tokenContract, txHash, payer, recv, payAmount, accountId, coinFen, packageId, confirmedAt] = this.args;
      if ([...this.db.rows.values()].some((row) => row.intent_id === intentId || (row.chain_id === chainId && row.evm_tx_hash === txHash))) {
        return { meta: { changes: 0 } };
      }
      this.db.rows.set(orderId as string, {
        order_id: orderId as string,
        intent_id: intentId as string,
        chain_id: chainId as number,
        token: token as string,
        token_contract: tokenContract as string,
        evm_tx_hash: txHash as string,
        payer_address: payer as string,
        recv_address: recv as string,
        pay_amount: payAmount as string,
        account_id: accountId as string,
        coin_fen: coinFen as string,
        package_id: packageId as string,
        status: 'pending',
        settlement_claim_id: null,
        settlement_claimed_at: null,
        gmb_tx_hash: null,
        gmb_block_hash: null,
        gmb_extrinsic_index: null,
        exception_reason: null,
        confirmed_at: confirmedAt as number,
        settled_at: null,
      });
      return { meta: { changes: 1 } };
    }
    if (this.sql.includes('SET settlement_claim_id = ?')) {
      const [claimId, claimedAt, orderId] = this.args as [string, number, string];
      const row = this.db.rows.get(orderId);
      if (!row || row.status !== 'pending' || row.settlement_claim_id) return { meta: { changes: 0 } };
      row.settlement_claim_id = claimId;
      row.settlement_claimed_at = claimedAt;
      return { meta: { changes: 1 } };
    }
    if (this.sql.includes("SET status = 'paid'")) {
      const [txHash, blockHash, index, settledAt, orderId, claimId] = this.args as [string, string, number, number, string, string];
      const row = this.db.rows.get(orderId);
      if (!row || row.status !== 'pending' || row.settlement_claim_id !== claimId) return { meta: { changes: 0 } };
      Object.assign(row, { status: 'paid', gmb_tx_hash: txHash, gmb_block_hash: blockHash, gmb_extrinsic_index: index, settled_at: settledAt });
      return { meta: { changes: 1 } };
    }
    if (this.sql.includes("SET status = 'exception'")) {
      const [reason, settledAt, orderId, claimId] = this.args as [string, number, string, string];
      const row = this.db.rows.get(orderId);
      if (!row || row.status !== 'pending' || row.settlement_claim_id !== claimId) return { meta: { changes: 0 } };
      Object.assign(row, { status: 'exception', exception_reason: reason, settled_at: settledAt });
      return { meta: { changes: 1 } };
    }
    return { meta: { changes: 0 } };
  }

  async all<T>(): Promise<{ results: T[] }> {
    const results = [...this.db.rows.values()]
      .filter((row) => row.status === 'pending')
      .sort((a, b) => a.confirmed_at - b.confirmed_at);
    return { results: results as T[] };
  }
}
