import { describe, expect, it } from 'vitest';
import { listContactsRoute, putContactRoute } from '../src/contacts/service';
import { getMembership } from '../src/membership/service';
import type { Env, MembershipRow, SessionState } from '../src/types';

// R6 门禁核心:换绑前后同一身份主键 cid_number 的社交数据不丢。
// 模型:cid = 稳定身份主键(用户不可改);account_id = 控制该身份的钱包账户(可换绑)。
// worker 所有用户数据按 cid 归属,故账户 A 写入的数据,换绑到账户 B(同一 cid)后仍可取回。
const CID_X = 'CN220-CTZN2-198805200-2026';
const ACCOUNT_A = '0x1111111111111111111111111111111111111111111111111111111111111111';
const ACCOUNT_B = '0x2222222222222222222222222222222222222222222222222222222222222222';
const CONTACT_ID = 'ab'.repeat(32); // 64 位小写 hex
// 合法端到端密文字段:nonce 恰 12 字节、mac 恰 16 字节(base64url,'A'=0x00)。
const CIPHERTEXT = 'AAAA'; // 3 字节
const NONCE = 'A'.repeat(16); // 12 字节
const MAC = 'A'.repeat(22); // 16 字节

/// 会话缓存:token → 会话身份态。两会话身份主键同为 CID_X,仅当前绑定账户不同(模拟换绑)。
function sessionKv(): KVNamespace {
  const sessionA: SessionState = {
    cid_number: CID_X,
    binding_revision: 1,
    account_id: ACCOUNT_A,
    device_key_hash: 'a'.repeat(64),
    created_at: 0,
    expires_at: Date.now() + 60_000
  };
  const sessionB: SessionState = {
    cid_number: CID_X,
    binding_revision: 2,
    account_id: ACCOUNT_B,
    device_key_hash: 'b'.repeat(64),
    created_at: 0,
    expires_at: Date.now() + 60_000
  };
  const store = new Map<string, unknown>([
    [
      'square_session:4f66a4283f8bc9768c3cb97fd06d267b79315aee941c9c1727b9354509242ffe',
      sessionA
    ],
    [
      'square_session:efa1cd32d437a4dd30463a379503cadfb2b13481660f6345110f3bde01f2e773',
      sessionB
    ]
  ]);
  return {
    get: async (key: string) => (store.get(key) as unknown) ?? null
  } as unknown as KVNamespace;
}

interface ContactRow {
  cid_number: string;
  contact_id: string;
  ciphertext: string;
  nonce: string;
  mac: string;
  updated_at: number;
}

/// 通讯录密文表 mock,按身份主键 cid_number 归属(对齐 R4 真实 schema)。
class ContactsDb {
  readonly rows = new Map<string, ContactRow>();
  prepare(sql: string): ContactsStmt {
    return new ContactsStmt(this, sql);
  }
}

class ContactsStmt {
  private binds: unknown[] = [];
  constructor(private readonly db: ContactsDb, private readonly sql: string) {}
  bind(...args: unknown[]): ContactsStmt {
    this.binds = args;
    return this;
  }
  async run(): Promise<{ meta: { changes: number } }> {
    if (this.sql.includes('INSERT INTO square_contacts')) {
      const row: ContactRow = {
        cid_number: this.binds[0] as string,
        contact_id: this.binds[1] as string,
        ciphertext: this.binds[2] as string,
        nonce: this.binds[3] as string,
        mac: this.binds[4] as string,
        updated_at: this.binds[5] as number
      };
      this.db.rows.set(`${row.cid_number}:${row.contact_id}`, row);
      return { meta: { changes: 1 } };
    }
    return { meta: { changes: 0 } };
  }
  async all<T>(): Promise<{ results: T[] }> {
    if (this.sql.includes('FROM square_contacts')) {
      const cidNumber = this.binds[0] as string;
      const limit = this.binds[this.binds.length - 1] as number;
      const rows = [...this.db.rows.values()]
        .filter((row) => row.cid_number === cidNumber)
        .slice(0, limit);
      return { results: rows as T[] };
    }
    return { results: [] };
  }
}

function env(db: ContactsDb): Env {
  return { DB: db as unknown as D1Database, SQUARE_CACHE: sessionKv() } as unknown as Env;
}

function putRequest(token: string): Request {
  return new Request(`https://worker.test/v1/square/contacts/${CONTACT_ID}`, {
    method: 'PUT',
    headers: { authorization: `Bearer ${token}`, 'content-type': 'application/json' },
    body: JSON.stringify({ ciphertext: CIPHERTEXT, nonce: NONCE, mac: MAC, updated_at: 1_000 })
  });
}

function listRequest(token: string): Request {
  return new Request('https://worker.test/v1/square/contacts', {
    headers: { authorization: `Bearer ${token}` }
  });
}

describe('换绑不丢:同一 cid_number 的社交数据跨账户存续', () => {
  it('账户 A 写入的通讯录密文,换绑到账户 B(同一 cid)后仍可取回', async () => {
    const db = new ContactsDb();

    // 账户 A(cid=CID_X)写入一条端到端密文通讯录。
    const putResponse = await putContactRoute(putRequest('tok-a'), env(db), CONTACT_ID);
    expect(((await putResponse.json()) as { applied: boolean }).applied).toBe(true);
    // 数据按身份主键 cid_number 归属(非账户 A)。
    expect(db.rows.has(`${CID_X}:${CONTACT_ID}`)).toBe(true);
    expect(db.rows.get(`${CID_X}:${CONTACT_ID}`)?.cid_number).toBe(CID_X);

    // 换绑:同一 cid 现绑定账户 B。账户 B 的会话拉取通讯录仍按 cid_number 命中同一密文。
    const listResponse = await listContactsRoute(listRequest('tok-b'), env(db));
    const body = (await listResponse.json()) as {
      items: Array<{ contact_id: string; ciphertext: string }>;
    };
    expect(body.items).toHaveLength(1);
    expect(body.items[0].contact_id).toBe(CONTACT_ID);
    expect(body.items[0].ciphertext).toBe(CIPHERTEXT);
    // 响应不下发属主键(cid_number/account_id 均不出现在联系人项)。
    expect(Object.keys(body.items[0]).sort()).toEqual([
      'ciphertext', 'contact_id', 'mac', 'nonce', 'updated_at'
    ]);
  });

  it('会员镜像按 cid_number 读取:换绑账户后同一 cid 的会员权益不丢', async () => {
    const membershipRow: MembershipRow = {
      cid_number: CID_X,
      account_id: ACCOUNT_A, // 发放时的付款账户(换绑后仍是历史事实)
      membership_level: 'democracy',
      started_at: 1,
      last_charged_at: 1,
      last_charged_price_fen: 999,
      paid_until: 9_999_999_999_999,
      subscription_status: 'active',
      finalized_block_number: 1,
      finalized_block_hash: '0x',
      verified_at: 1,
      entitlement_lapsed_at: null,
      last_tx_hash: null,
      chain_timestamp: 2,
      chain_observed_at: 2
    };
    // getMembership 现按 cid_number 查(R3);换绑后新账户会话仍持有同一 cid,取回同一会员镜像。
    const membershipDb = {
      prepare: (sql: string) => ({
        bind: (...binds: unknown[]) => ({
          first: async () =>
            sql.includes('FROM square_memberships') && binds[0] === CID_X ? membershipRow : null
        })
      })
    } as unknown as D1Database;
    const membership = await getMembership({ DB: membershipDb } as unknown as Env, CID_X);
    expect(membership?.cid_number).toBe(CID_X);
    expect(membership?.membership_level).toBe('democracy');
  });
});
