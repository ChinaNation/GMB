import { describe, expect, it } from 'vitest';
import { fanOutPage } from '../src/feeds/notify_fanout';
import type { Env, SquareNotifyJob } from '../src/types';

const author = '0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd';
const FAR_FUTURE = 9999999999999;

interface FollowRow {
  account_id: string;
  followed_account_id: string;
  notify_enabled: number;
  created_at: number;
}
interface DeviceRow {
  account_id: string;
  push_provider: 'apns' | 'fcm';
  push_token: string;
  expires_at: number;
}

describe('fanOutPage', () => {
  it('pushes only to notify-enabled followers with non-expired devices', async () => {
    const { env, db } = fakeEnv({
      follows: [follow('0x0101010101010101010101010101010101010101010101010101010101010101', 1, 100), follow('0x0202020202020202020202020202020202020202020202020202020202020202', 0, 110), follow('0x0303030303030303030303030303030303030303030303030303030303030303', 1, 120)],
      devices: [device('0x0101010101010101010101010101010101010101010101010101010101010101', FAR_FUTURE), device('0x0202020202020202020202020202020202020202020202020202020202020202', FAR_FUTURE), device('0x0303030303030303030303030303030303030303030303030303030303030303', 1)],
    });
    await fanOutPage(env, job(), 100);
    // 静音（notify_enabled=0）与过期 token 都排除。
    expect(db.pushedAccounts).toEqual(['0x0101010101010101010101010101010101010101010101010101010101010101']);
  });

  it('does not re-enqueue when the page is not full', async () => {
    const { env, queue } = fakeEnv({
      follows: [follow('0x1111111111111111111111111111111111111111111111111111111111111111', 1, 100), follow('0x2222222222222222222222222222222222222222222222222222222222222222', 1, 110)],
      devices: [device('0x1111111111111111111111111111111111111111111111111111111111111111', FAR_FUTURE), device('0x2222222222222222222222222222222222222222222222222222222222222222', FAR_FUTURE)],
    });
    await fanOutPage(env, job(), 100);
    expect(queue.sent).toHaveLength(0);
  });

  it('re-enqueues a continuation cursor when the page is full', async () => {
    const { env, queue } = fakeEnv({
      follows: [follow('0x1111111111111111111111111111111111111111111111111111111111111111', 1, 100), follow('0x2222222222222222222222222222222222222222222222222222222222222222', 1, 110), follow('0x3333333333333333333333333333333333333333333333333333333333333333', 1, 120)],
      devices: [device('0x1111111111111111111111111111111111111111111111111111111111111111', FAR_FUTURE), device('0x2222222222222222222222222222222222222222222222222222222222222222', FAR_FUTURE), device('0x3333333333333333333333333333333333333333333333333333333333333333', FAR_FUTURE)],
    });
    await fanOutPage(env, job(), 2); // 页大小 2、3 个合格粉丝 → 满页续跑

    expect(queue.sent).toHaveLength(1);
    expect(queue.sent[0]).toMatchObject({
      author_account_id: author,
      post_id: 'p1',
      cursor: { created_at: 110, account_id: '0x2222222222222222222222222222222222222222222222222222222222222222' }, // 本页末个粉丝
    });
  });
});

function job(): SquareNotifyJob {
  return {
    author_account_id: author,
    author_name: '林正华',
    content_format: 'normal',
    post_id: 'p1',
  };
}

function follow(accountId: string, notify: number, createdAt: number): FollowRow {
  return {
    account_id: accountId,
    followed_account_id: author,
    notify_enabled: notify,
    created_at: createdAt,
  };
}

function device(accountId: string, expiresAt: number): DeviceRow {
  return {
    account_id: accountId,
    push_provider: 'fcm',
    push_token: `tok_${accountId}`,
    expires_at: expiresAt,
  };
}

function fakeEnv(options: { follows: FollowRow[]; devices: DeviceRow[] }): {
  env: Env;
  queue: FakeQueue;
  db: FakeDb;
} {
  const queue = new FakeQueue();
  const db = new FakeDb(options.follows, options.devices);
  const env = {
    DB: db as unknown as D1Database,
    SQUARE_NOTIFY_QUEUE: queue as unknown as Queue<SquareNotifyJob>,
    // 无 APNS/FCM 密钥 → sendSquarePostAlert 早退 false，不触真推送。
  } as unknown as Env;
  return { env, queue, db };
}

class FakeQueue {
  sent: SquareNotifyJob[] = [];
  async send(message: SquareNotifyJob): Promise<void> {
    this.sent.push(message);
  }
}

class FakeDb {
  pushedAccounts: string[] = [];
  constructor(
    readonly follows: FollowRow[],
    readonly devices: DeviceRow[],
  ) {}
  prepare(sql: string): FakeStmt {
    return new FakeStmt(sql, this);
  }
}

class FakeStmt {
  private binds: unknown[] = [];
  constructor(
    private readonly sql: string,
    private readonly db: FakeDb,
  ) {}

  bind(...args: unknown[]): FakeStmt {
    this.binds = args;
    return this;
  }

  async all<T>(): Promise<{ results: T[] }> {
    if (this.sql.includes('FROM square_follows')) {
      const [followed, cursorAt, cursorAccountId, limit] = this.binds as [
        string,
        number,
        string,
        number,
      ];
      const rows = this.db.follows
        .filter(
          (f) =>
            f.followed_account_id === followed &&
            f.notify_enabled === 1 &&
            (f.created_at > cursorAt ||
              (f.created_at === cursorAt && f.account_id > cursorAccountId)),
        )
        .sort(
          (a, b) =>
            a.created_at - b.created_at || a.account_id.localeCompare(b.account_id),
        )
        .slice(0, limit)
        .map((f) => ({ account_id: f.account_id, created_at: f.created_at }));
      return { results: rows as unknown as T[] };
    }

    if (this.sql.includes('FROM chat_devices')) {
      const accounts = this.binds.slice(0, -1) as string[];
      const now = this.binds[this.binds.length - 1] as number;
      const hit = this.db.devices.filter(
        (d) => accounts.includes(d.account_id) && d.expires_at > now,
      );
      this.db.pushedAccounts.push(...hit.map((d) => d.account_id));
      return {
        results: hit.map((d) => ({
          push_provider: d.push_provider,
          push_token: d.push_token,
        })) as unknown as T[],
      };
    }

    return { results: [] };
  }
}
