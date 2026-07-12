import { describe, expect, it } from 'vitest';
import { addBrowseCount, assertBrowseAvailable, getBrowseState } from '../src/feeds/browse';
import type { Env } from '../src/types';

const OWNER = '5GrwvaEF5zXb26Fz9rcQpDWS7u4m6DXb6T6TQvF9j5uQ8g6U';

class BrowseDb {
  count = 0;
  prepare(sql: string): BrowseStmt {
    return new BrowseStmt(this, sql);
  }
}

class BrowseStmt {
  private values: unknown[] = [];
  constructor(private readonly db: BrowseDb, private readonly sql: string) {}
  bind(...values: unknown[]): BrowseStmt {
    this.values = values;
    return this;
  }
  async first<T>(): Promise<T | null> {
    if (this.sql.includes('FROM square_memberships')) return null;
    if (this.sql.includes('FROM square_browse_days')) {
      return (this.db.count > 0 ? { browse_count: this.db.count } : null) as T | null;
    }
    return null;
  }
  async run(): Promise<{ meta: { changes: number } }> {
    if (this.sql.includes('INSERT INTO square_browse_days')) {
      const next = this.db.count + Number(this.values[2]);
      if (next > 100) return { meta: { changes: 0 } };
      this.db.count = next;
    }
    return { meta: { changes: 1 } };
  }
}

describe('wallet browse allowance', () => {
  it('starts unsubscribed wallets at 100 returned items per UTC day', async () => {
    const db = new BrowseDb();
    const env = { DB: db } as unknown as Env;
    const state = await getBrowseState(env, OWNER);
    expect(state).toMatchObject({ browse_count: 0, browse_limit: 100, browse_left: 100 });
  });

  it('counts only server-returned items and blocks after the allowance is exhausted', async () => {
    const db = new BrowseDb();
    const env = { DB: db } as unknown as Env;
    let state = await getBrowseState(env, OWNER);
    state = await addBrowseCount(env, OWNER, state, 40);
    expect(state.browse_left).toBe(60);
    state = await addBrowseCount(env, OWNER, state, 60);
    expect(state.browse_left).toBe(0);
    expect(() => assertBrowseAvailable(state)).toThrow(
      expect.objectContaining({ code: 'browse_limit_reached', status: 429 }),
    );
  });

  it('rejects a stale concurrent deduction instead of returning over-limit content', async () => {
    const db = new BrowseDb();
    db.count = 90;
    const env = { DB: db } as unknown as Env;
    const stale = await getBrowseState(env, OWNER);
    await addBrowseCount(env, OWNER, stale, 10);
    await expect(addBrowseCount(env, OWNER, stale, 10)).rejects.toMatchObject({
      code: 'browse_limit_reached',
      status: 429,
    });
    expect(db.count).toBe(100);
  });
});
