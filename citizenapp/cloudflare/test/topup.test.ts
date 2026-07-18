import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Env } from '../src/types';
import { topupConfigRoute, topupStatusRoute, topupSubmitRoute } from '../src/topup/orders';
import {
  topupExceptionRoute,
  topupPendingRoute,
  topupSettledRoute,
} from '../src/topup/settlement';

const TRANSFER_TOPIC = '0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef';
// 测试网 USDC(Base Sepolia)默认合约,与 config.ts 内置一致。
const USDC_TESTNET = '0x036cbd53842c5426634e7929541ec2318f3dcf7e';
const RECV = `0x${'ab'.repeat(20)}`;
const PAYER = `0x${'cd'.repeat(20)}`;
const GMB_ADDRESS = 'gmbwalletaddressabcdefghijklmnop';
const TX_HASH = `0x${'11'.repeat(32)}`;
const GMB_TX_HASH = `0x${'22'.repeat(32)}`;

describe('topup 稳定币充值后端', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('config 返回已配置币轨与套餐', async () => {
    const env = makeEnv(new FakeDb());
    const response = await topupConfigRoute(new Request('https://x.test/v1/square/topup/config'), env);
    const body = await response.json<{ rails: { token: string; chain_id: number }[]; packages: unknown[]; recv_address: string }>();
    // USDC 与 USDT 两轨始终同时提供。
    expect(body.rails).toHaveLength(2);
    expect(body.rails.map((rail) => rail.token)).toEqual(['USDC', 'USDT']);
    // USDC/USDT 同走 Base（沙箱 = Base Sepolia 84532）。
    expect(body.rails[0]).toMatchObject({ token: 'USDC', chain_id: 84532 });
    expect(body.rails[1]).toMatchObject({ token: 'USDT', chain_id: 84532 });
    expect(body.packages).toHaveLength(2);
    expect(body.recv_address).toBe(RECV);
  });

  it('submit 足额到账且已确认 → 落待支付入队', async () => {
    const db = new FakeDb();
    const env = makeEnv(db);
    vi.stubGlobal('fetch', evmFetch({ receipt: confirmedReceipt(15000000n, '0x10'), finalized: '0x20' }));

    const response = await submit(env);
    const body = await response.json<{ status: string; order_id: string }>();
    expect(body.status).toBe('pending');
    expect(body.order_id).toMatch(/^top_/);
    expect(db.rows.size).toBe(1);
    expect([...db.rows.values()][0]).toMatchObject({ status: 'pending', coin_fen: '1000000' });
  });

  it('submit 交易尚未上链 → confirming 不落库', async () => {
    const db = new FakeDb();
    const env = makeEnv(db);
    vi.stubGlobal('fetch', evmFetch({ receipt: null }));

    const body = await (await submit(env)).json<{ status: string }>();
    expect(body.status).toBe('confirming');
    expect(db.rows.size).toBe(0);
  });

  it('submit 金额不足 → 拒绝且不落库', async () => {
    const db = new FakeDb();
    const env = makeEnv(db);
    vi.stubGlobal('fetch', evmFetch({ receipt: confirmedReceipt(1000000n, '0x10'), finalized: '0x20' }));

    await expect(submit(env)).rejects.toMatchObject({ code: 'topup_payment_invalid' });
    expect(db.rows.size).toBe(0);
  });

  it('submit 同一 txHash 幂等 → 不重复入账', async () => {
    const db = new FakeDb();
    const env = makeEnv(db);
    vi.stubGlobal('fetch', evmFetch({ receipt: confirmedReceipt(15000000n, '0x10'), finalized: '0x20' }));

    const first = await (await submit(env)).json<{ order_id: string }>();
    const second = await (await submit(env)).json<{ order_id: string; status: string }>();
    expect(second.order_id).toBe(first.order_id);
    expect(second.status).toBe('pending');
    expect(db.rows.size).toBe(1);
  });

  it('settlement/pending 无结算令牌 → 401', async () => {
    const env = makeEnv(new FakeDb());
    await expect(
      topupPendingRoute(new Request('https://x.test/v1/square/topup/settlement/pending'), env),
    ).rejects.toMatchObject({ code: 'topup_settle_unauthorized' });
  });

  it('settlement/pending 有令牌 → 返回待支付队列', async () => {
    const db = new FakeDb();
    const env = makeEnv(db);
    vi.stubGlobal('fetch', evmFetch({ receipt: confirmedReceipt(15000000n, '0x10'), finalized: '0x20' }));
    await submit(env);

    const response = await topupPendingRoute(authGet('https://x.test/v1/square/topup/settlement/pending'), env);
    const body = await response.json<{ orders: { evm_tx_hash: string; coin_fen: string }[] }>();
    expect(body.orders).toHaveLength(1);
    expect(body.orders[0]).toMatchObject({ evm_tx_hash: TX_HASH, coin_fen: '1000000' });
  });

  it('settled 复核通过 → 置已支付并记 gmb_tx_hash', async () => {
    const db = new FakeDb();
    const env = makeEnv(db);
    vi.stubGlobal('fetch', evmFetch({ receipt: confirmedReceipt(15000000n, '0x10'), finalized: '0x20' }));
    const orderId = (await (await submit(env)).json<{ order_id: string }>()).order_id;

    const response = await topupSettledRoute(authPost(`https://x.test/v1/square/topup/settlement/${orderId}/settled`, { gmb_tx_hash: GMB_TX_HASH }), env, orderId);
    const body = await response.json<{ status: string }>();
    expect(body.status).toBe('paid');
    expect(db.rows.get(orderId)).toMatchObject({ status: 'paid', gmb_tx_hash: GMB_TX_HASH });
  });

  it('exception → 置异常交人工', async () => {
    const db = new FakeDb();
    const env = makeEnv(db);
    vi.stubGlobal('fetch', evmFetch({ receipt: confirmedReceipt(15000000n, '0x10'), finalized: '0x20' }));
    const orderId = (await (await submit(env)).json<{ order_id: string }>()).order_id;

    const response = await topupExceptionRoute(authPost(`https://x.test/v1/square/topup/settlement/${orderId}/exception`, { reason: 'disburse_failed' }), env, orderId);
    const body = await response.json<{ status: string }>();
    expect(body.status).toBe('exception');
    expect(db.rows.get(orderId)).toMatchObject({ status: 'exception', exception_reason: 'disburse_failed' });
  });

  it('status 查已入账订单 → 返回台账态', async () => {
    const db = new FakeDb();
    const env = makeEnv(db);
    vi.stubGlobal('fetch', evmFetch({ receipt: confirmedReceipt(15000000n, '0x10'), finalized: '0x20' }));
    await submit(env);

    const response = await topupStatusRoute(
      new Request(`https://x.test/v1/square/topup/status?chain_id=84532&evm_tx_hash=${TX_HASH}`),
      env,
    );
    const body = await response.json<{ status: string }>();
    expect(body.status).toBe('pending');
  });
});

function submit(env: Env): Promise<Response> {
  return topupSubmitRoute(
    new Request('https://x.test/v1/square/topup/submit', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        token: 'USDC',
        package_id: 'pkg_15',
        gmb_address: GMB_ADDRESS,
        evm_tx_hash: TX_HASH,
        payer_address: PAYER,
      }),
    }),
    env,
  );
}

function authGet(url: string): Request {
  return new Request(url, { headers: { authorization: 'Bearer settle-secret' } });
}

function authPost(url: string, body: unknown): Request {
  return new Request(url, {
    method: 'POST',
    headers: { authorization: 'Bearer settle-secret', 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
}

function confirmedReceipt(value: bigint, block: string): unknown {
  return {
    status: '0x1',
    blockNumber: block,
    logs: [
      {
        address: USDC_TESTNET,
        topics: [TRANSFER_TOPIC, addrTopic(PAYER), addrTopic(RECV)],
        data: `0x${value.toString(16).padStart(64, '0')}`,
      },
    ],
  };
}

function addrTopic(address: string): string {
  return `0x${'0'.repeat(24)}${address.replace(/^0x/, '')}`;
}

function evmFetch(handlers: { receipt?: unknown; finalized?: string; latest?: string }) {
  return vi.fn(async (_url: string, init: RequestInit) => {
    const body = JSON.parse(init.body as string) as { method: string };
    if (body.method === 'eth_getTransactionReceipt') {
      return Response.json({ jsonrpc: '2.0', id: 1, result: handlers.receipt ?? null });
    }
    if (body.method === 'eth_getBlockByNumber') {
      return Response.json({ jsonrpc: '2.0', id: 1, result: handlers.finalized ? { number: handlers.finalized } : null });
    }
    if (body.method === 'eth_blockNumber') {
      return Response.json({ jsonrpc: '2.0', id: 1, result: handlers.latest ?? '0x0' });
    }
    return Response.json({ jsonrpc: '2.0', id: 1, result: null });
  });
}

function makeEnv(db: FakeDb): Env {
  return {
    DB: db,
    TOPUP_NETWORK: 'testnet',
    TOPUP_RECV_ADDRESS: RECV,
    TOPUP_BASE_RPC_URL: 'https://base-sepolia.test',
    TOPUP_ARBITRUM_RPC_URL: 'https://arb-sepolia.test',
    TOPUP_SETTLE_TOKEN: 'settle-secret',
  } as unknown as Env;
}

interface Row {
  order_id: string;
  chain_id: number;
  token: string;
  token_contract: string;
  evm_tx_hash: string;
  payer_address: string | null;
  recv_address: string;
  pay_amount: string;
  gmb_address: string;
  coin_fen: string;
  package_id: string;
  status: string;
  gmb_tx_hash: string | null;
  exception_reason: string | null;
  confirmed_at: number;
  settled_at: number | null;
}

/// 面向 topup SQL 的最小内存 D1 假库:支持插入(幂等)、按 txHash/order_id 查、待队列、状态更新。
class FakeDb {
  rows = new Map<string, Row>();

  prepare(sql: string) {
    return new FakeStmt(this, sql);
  }
}

class FakeStmt {
  private args: unknown[] = [];

  constructor(
    private readonly db: FakeDb,
    private readonly sql: string,
  ) {}

  bind(...args: unknown[]) {
    this.args = args;
    return this;
  }

  async first<T>(): Promise<T | null> {
    if (this.sql.includes('WHERE chain_id = ? AND evm_tx_hash = ?')) {
      const [chainId, txHash] = this.args as [number, string];
      for (const row of this.db.rows.values()) {
        if (row.chain_id === chainId && row.evm_tx_hash === txHash) return row as unknown as T;
      }
      return null;
    }
    if (this.sql.includes('WHERE order_id = ?')) {
      return (this.db.rows.get(this.args[0] as string) ?? null) as T | null;
    }
    return null;
  }

  async run(): Promise<{ meta: { changes: number } }> {
    if (this.sql.includes('INSERT OR IGNORE INTO topup_orders')) {
      const [orderId, chainId, token, tokenContract, txHash, payer, recv, payAmount, gmbAddress, coinFen, packageId, confirmedAt] =
        this.args as [string, number, string, string, string, string | null, string, string, string, string, string, number];
      // 幂等唯一键 (chain_id, evm_tx_hash)。
      for (const row of this.db.rows.values()) {
        if (row.chain_id === chainId && row.evm_tx_hash === txHash) return { meta: { changes: 0 } };
      }
      this.db.rows.set(orderId, {
        order_id: orderId,
        chain_id: chainId,
        token,
        token_contract: tokenContract,
        evm_tx_hash: txHash,
        payer_address: payer,
        recv_address: recv,
        pay_amount: payAmount,
        gmb_address: gmbAddress,
        coin_fen: coinFen,
        package_id: packageId,
        status: 'pending',
        gmb_tx_hash: null,
        exception_reason: null,
        confirmed_at: confirmedAt,
        settled_at: null,
      });
      return { meta: { changes: 1 } };
    }
    if (this.sql.includes("SET status = 'paid'")) {
      const [gmbTxHash, settledAt, orderId] = this.args as [string, number, string];
      const row = this.db.rows.get(orderId);
      if (row && row.status === 'pending') {
        row.status = 'paid';
        row.gmb_tx_hash = gmbTxHash;
        row.settled_at = settledAt;
        return { meta: { changes: 1 } };
      }
      return { meta: { changes: 0 } };
    }
    if (this.sql.includes("SET status = 'exception'")) {
      const [reason, settledAt, orderId] = this.args as [string, number, string];
      const row = this.db.rows.get(orderId);
      if (row && row.status === 'pending') {
        row.status = 'exception';
        row.exception_reason = reason;
        row.settled_at = settledAt;
        return { meta: { changes: 1 } };
      }
      return { meta: { changes: 0 } };
    }
    return { meta: { changes: 0 } };
  }

  async all<T>(): Promise<{ results: T[] }> {
    if (this.sql.includes("WHERE status = 'pending'")) {
      const results = [...this.db.rows.values()]
        .filter((row) => row.status === 'pending')
        .sort((a, b) => a.confirmed_at - b.confirmed_at);
      return { results: results as unknown as T[] };
    }
    return { results: [] };
  }
}
