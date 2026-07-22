import { describe, expect, it } from 'vitest';
import { getNotifyUnreadRoute, markNotifyReadRoute } from '../src/feeds/notify';
import type { Env, SessionState } from '../src/types';

const viewer = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';

interface PostRow {
  owner_account: string;
  created_at: number;
  post_state: string;
}
interface FollowRow {
  owner_account: string;
  followed_account: string;
  notify_enabled: number;
}

describe('GET /v1/square/notify/unread', () => {
  it('counts new posts from notify-enabled follows since each cursor (no reads row = 0)', async () => {
    const env = fakeEnv({
      follows: [
        { owner_account: viewer, followed_account: 'author_a', notify_enabled: 1 },
        { owner_account: viewer, followed_account: 'author_b', notify_enabled: 1 }
      ],
      posts: [
        { owner_account: 'author_a', created_at: 100, post_state: 'published' },
        { owner_account: 'author_b', created_at: 200, post_state: 'published' }
      ]
    });
    const body = await readUnread(env);
    expect(body).toMatchObject({ square_unread: 2, following_unread: 2 });
  });

  it('excludes muted follows and unpublished posts', async () => {
    const env = fakeEnv({
      follows: [
        { owner_account: viewer, followed_account: 'author_a', notify_enabled: 1 },
        { owner_account: viewer, followed_account: 'author_muted', notify_enabled: 0 }
      ],
      posts: [
        { owner_account: 'author_a', created_at: 100, post_state: 'published' },
        { owner_account: 'author_a', created_at: 150, post_state: 'draft' },
        { owner_account: 'author_muted', created_at: 200, post_state: 'published' }
      ]
    });
    const body = await readUnread(env);
    expect(body).toMatchObject({ square_unread: 1, following_unread: 1 });
  });

  it('respects an existing cursor', async () => {
    const env = fakeEnv({
      follows: [
        { owner_account: viewer, followed_account: 'author_a', notify_enabled: 1 }
      ],
      posts: [
        { owner_account: 'author_a', created_at: 100, post_state: 'published' },
        { owner_account: 'author_a', created_at: 300, post_state: 'published' }
      ],
      reads: { last_seen_square_at: 200, last_seen_following_at: 0 }
    });
    const body = await readUnread(env);
    // square 游标=200 → 只 300 未读；following 游标=0 → 两条都未读。
    expect(body).toMatchObject({ square_unread: 1, following_unread: 2 });
  });

  it('marking square read clears only the square badge, following stays', async () => {
    const env = fakeEnv({
      follows: [
        { owner_account: viewer, followed_account: 'author_a', notify_enabled: 1 }
      ],
      posts: [
        { owner_account: 'author_a', created_at: 100, post_state: 'published' },
        { owner_account: 'author_a', created_at: 200, post_state: 'published' }
      ]
    });
    expect(await readUnread(env)).toMatchObject({ square_unread: 2, following_unread: 2 });

    await markNotifyReadRoute(
      request('https://w/v1/square/notify/read', {
        method: 'POST',
        authToken: 'tok',
        body: { scope: 'square' }
      }),
      env
    );

    // 帖子 created_at(≤200) 远小于 now → 广场游标推进后广场清零；关注游标未动仍为 2。
    expect(await readUnread(env)).toMatchObject({ square_unread: 0, following_unread: 2 });
  });

  it('rejects an invalid scope', async () => {
    const env = fakeEnv({});
    await expect(
      markNotifyReadRoute(
        request('https://w/v1/square/notify/read', {
          method: 'POST',
          authToken: 'tok',
          body: { scope: 'all' }
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'invalid_scope' });
  });

  async function readUnread(
    env: Env
  ): Promise<{ square_unread: number; following_unread: number }> {
    const response = await getNotifyUnreadRoute(
      request('https://w/v1/square/notify/unread', { authToken: 'tok' }),
      env
    );
    return (await response.json()) as {
      square_unread: number;
      following_unread: number;
    };
  }
});

interface FakeEnvOptions {
  posts?: PostRow[];
  follows?: FollowRow[];
  reads?: { last_seen_square_at: number; last_seen_following_at: number };
}

function fakeEnv(options: FakeEnvOptions): Env {
  const kv = new Map<string, unknown>();
  const session: SessionState = {
    owner_account: viewer,
    device_key_hash: 'a'.repeat(64),
    created_at: 0,
    expires_at: Date.now() + 60_000
  };
  kv.set('square_session:tok', session);

  const reads = new Map<string, { square: number; following: number }>();
  if (options.reads) {
    reads.set(viewer, {
      square: options.reads.last_seen_square_at,
      following: options.reads.last_seen_following_at
    });
  }

  return {
    DB: new FakeDb(options.posts ?? [], options.follows ?? [], reads) as unknown as D1Database,
    SQUARE_CACHE: new FakeKv(kv) as unknown as KVNamespace
  } as unknown as Env;
}

function request(
  url: string,
  init: { method?: string; authToken?: string; body?: unknown } = {}
): Request {
  const headers = new Headers();
  if (init.authToken) headers.set('authorization', `Bearer ${init.authToken}`);
  if (init.body !== undefined) headers.set('content-type', 'application/json');
  return new Request(url, {
    method: init.method ?? 'GET',
    headers,
    body: init.body !== undefined ? JSON.stringify(init.body) : undefined
  });
}

class FakeKv {
  constructor(private readonly store: Map<string, unknown>) {}
  async get<T>(key: string): Promise<T | null> {
    return (this.store.get(key) as T) ?? null;
  }
}

class FakeDb {
  constructor(
    private readonly posts: PostRow[],
    private readonly follows: FollowRow[],
    private readonly reads: Map<string, { square: number; following: number }>
  ) {}
  prepare(sql: string): FakeStmt {
    return new FakeStmt(this.posts, this.follows, this.reads, sql);
  }
}

class FakeStmt {
  private binds: unknown[] = [];
  constructor(
    private readonly posts: PostRow[],
    private readonly follows: FollowRow[],
    private readonly reads: Map<string, { square: number; following: number }>,
    private readonly sql: string
  ) {}

  bind(...args: unknown[]): FakeStmt {
    this.binds = args;
    return this;
  }

  async first<T>(): Promise<T | null> {
    if (this.sql.includes('COUNT(*)') && this.sql.includes('square_posts')) {
      const viewerAccount = this.binds[0] as string;
      const since = this.binds[1] as number;
      const n = this.posts.filter(
        (p) =>
          p.post_state === 'published' &&
          p.created_at > since &&
          this.follows.some(
            (f) =>
              f.owner_account === viewerAccount &&
              f.followed_account === p.owner_account &&
              f.notify_enabled === 1
          )
      ).length;
      return { n } as T;
    }
    if (this.sql.includes('FROM square_notify_reads')) {
      const owner = this.binds[0] as string;
      const row = this.reads.get(owner);
      return row
        ? ({ last_seen_square_at: row.square, last_seen_following_at: row.following } as T)
        : null;
    }
    return null;
  }

  async run(): Promise<{ meta: { changes: number } }> {
    if (this.sql.includes('INSERT INTO square_notify_reads')) {
      const owner = this.binds[0] as string;
      const value = this.binds[1] as number;
      const existing = this.reads.get(owner) ?? { square: 0, following: 0 };
      if (this.sql.includes('last_seen_square_at')) {
        existing.square = value;
      } else {
        existing.following = value;
      }
      this.reads.set(owner, existing);
    }
    return { meta: { changes: 1 } };
  }
}
