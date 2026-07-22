import { describe, expect, it } from 'vitest';
import { fanOutPage } from '../src/feeds/notify_fanout';
import type { Env, SquareNotifyJob } from '../src/types';

const author = 'author_account';
const FAR_FUTURE = 9999999999999;

interface FollowRow {
  owner_account: string;
  followed_account: string;
  notify_enabled: number;
  created_at: number;
}
interface DeviceRow {
  owner_account: string;
  push_provider: 'apns' | 'fcm';
  push_token: string;
  expires_at: number;
}

describe('fanOutPage', () => {
  it('pushes only to notify-enabled followers with non-expired devices', async () => {
    const { env, db } = fakeEnv({
      follows: [follow('f_on', 1, 100), follow('f_muted', 0, 110), follow('f_expired', 1, 120)],
      devices: [device('f_on', FAR_FUTURE), device('f_muted', FAR_FUTURE), device('f_expired', 1)],
    });
    await fanOutPage(env, job(), 100);
    // 静音（notify_enabled=0）与过期 token 都排除。
    expect(db.pushedAccounts).toEqual(['f_on']);
  });

  it('does not re-enqueue when the page is not full', async () => {
    const { env, queue } = fakeEnv({
      follows: [follow('f1', 1, 100), follow('f2', 1, 110)],
      devices: [device('f1', FAR_FUTURE), device('f2', FAR_FUTURE)],
    });
    await fanOutPage(env, job(), 100);
    expect(queue.sent).toHaveLength(0);
  });

  it('re-enqueues a continuation cursor when the page is full', async () => {
    const { env, queue } = fakeEnv({
      follows: [follow('f1', 1, 100), follow('f2', 1, 110), follow('f3', 1, 120)],
      devices: [device('f1', FAR_FUTURE), device('f2', FAR_FUTURE), device('f3', FAR_FUTURE)],
    });
    await fanOutPage(env, job(), 2); // 页大小 2、3 个合格粉丝 → 满页续跑

    expect(queue.sent).toHaveLength(1);
    expect(queue.sent[0]).toMatchObject({
      author_account: author,
      post_id: 'p1',
      cursor: { created_at: 110, owner_account: 'f2' }, // 本页末个粉丝
    });
  });
});

function job(): SquareNotifyJob {
  return {
    author_account: author,
    author_name: '林正华',
    content_format: 'normal',
    post_id: 'p1',
  };
}

function follow(owner: string, notify: number, createdAt: number): FollowRow {
  return {
    owner_account: owner,
    followed_account: author,
    notify_enabled: notify,
    created_at: createdAt,
  };
}

function device(owner: string, expiresAt: number): DeviceRow {
  return {
    owner_account: owner,
    push_provider: 'fcm',
    push_token: `tok_${owner}`,
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
      const [followed, cursorAt, cursorAccount, limit] = this.binds as [
        string,
        number,
        string,
        number,
      ];
      const rows = this.db.follows
        .filter(
          (f) =>
            f.followed_account === followed &&
            f.notify_enabled === 1 &&
            (f.created_at > cursorAt ||
              (f.created_at === cursorAt && f.owner_account > cursorAccount)),
        )
        .sort(
          (a, b) =>
            a.created_at - b.created_at || a.owner_account.localeCompare(b.owner_account),
        )
        .slice(0, limit)
        .map((f) => ({ owner_account: f.owner_account, created_at: f.created_at }));
      return { results: rows as unknown as T[] };
    }

    if (this.sql.includes('FROM chat_devices')) {
      const accounts = this.binds.slice(0, -1) as string[];
      const now = this.binds[this.binds.length - 1] as number;
      const hit = this.db.devices.filter(
        (d) => accounts.includes(d.owner_account) && d.expires_at > now,
      );
      this.db.pushedAccounts.push(...hit.map((d) => d.owner_account));
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
