import { describe, it, expect } from 'vitest';
import type { Env } from '../src/types';
import { initChatRelay, ackChatRelay } from '../src/chat/relay';

const MiB = 1024 * 1024;

class FakeKv {
  store = new Map<string, unknown>();
  async get(key: string, type?: string): Promise<unknown> {
    const value = this.store.get(key);
    if (value === undefined) return null;
    return type === 'json' ? value : String(value);
  }
  async put(key: string, value: string): Promise<void> {
    this.store.set(key, value);
  }
  async delete(key: string): Promise<void> {
    this.store.delete(key);
  }
}

class FakeR2 {
  store = new Map<string, unknown>();
  async put(key: string, body: unknown): Promise<void> {
    this.store.set(key, body);
  }
  async get(key: string): Promise<unknown> {
    return this.store.has(key) ? { body: this.store.get(key) } : null;
  }
  async delete(key: string): Promise<void> {
    this.store.delete(key);
  }
}

function buildEnv(level: string | null = 'spark') {
  const kv = new FakeKv();
  const r2 = new FakeR2();
  kv.store.set('square_session:tok', {
    account_id: '0x9999999999999999999999999999999999999999999999999999999999999999',
    expires_at: Date.now() + 60_000,
  });
  const membership = level === null
    ? null
    : {
        account_id: '0x9999999999999999999999999999999999999999999999999999999999999999',
        membership_level: level,
        subscription_status: 'active',
        paid_until: Date.now() + 60_000,
        chain_timestamp: Date.now(),
        chain_observed_at: Date.now(),
      };
  const db = {
    prepare() {
      return {
        bind() {
          return this;
        },
        async first() {
          return membership;
        },
      };
    },
  };
  const env = { DB: db, SQUARE_CACHE: kv, CHAT_RELAY: r2 } as unknown as Env;
  return { env, kv, r2 };
}

function req(method: string, body?: unknown): Request {
  // init 走 readJson,按 pathname 查路由限额,故用真实 init 路径;ack 不读 body,路径无关。
  return new Request('https://x/v1/chat/relay/init', {
    method,
    headers: { authorization: 'Bearer tok', 'content-type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
}

describe('chat relay', () => {
  it('init 建 recipient_count;ack 达数后删 R2 + KV', async () => {
    const { env, kv, r2 } = buildEnv();
    const res = await initChatRelay(
      req('POST', { byte_size: 200 * MiB, recipient_count: 3 }),
      env,
    );
    const { object_key: key } = (await res.json()) as { object_key: string };
    expect(kv.store.get(`relay:count:${key}`)).toBe('3');
    await r2.put(key, 'ciphertext'); // 模拟已上传密文

    await ackChatRelay(req('POST'), env, key);
    await ackChatRelay(req('POST'), env, key);
    expect(r2.store.has(key)).toBe(true); // 未达 3 次,保留
    await ackChatRelay(req('POST'), env, key);
    expect(r2.store.has(key)).toBe(false); // 第 3 次归零 → 删
    expect(kv.store.has(`relay:count:${key}`)).toBe(false);
  });

  it('非薪火 init 被拒', async () => {
    const { env } = buildEnv('democracy');
    await expect(
      initChatRelay(req('POST', { byte_size: 200 * MiB, recipient_count: 1 }), env),
    ).rejects.toThrow();
  });

  it('≤100MB init 被拒(仅 >100MB 走中转)', async () => {
    const { env } = buildEnv();
    await expect(
      initChatRelay(req('POST', { byte_size: 50 * MiB, recipient_count: 1 }), env),
    ).rejects.toThrow();
  });
});
