import { describe, expect, it } from 'vitest';
import {
  getUserFollowsRoute,
  getUserPostsRoute,
  getUserProfileRoute,
  putProfileRoute
} from '../src/profiles/service';
import { readProfileDoc, writeProfileDoc } from '../src/profiles/repository';
import { profileObjectKey } from '../src/storage/r2_keys';
import type { CitizenProfileDoc, Env, SessionState } from '../src/types';

const owner = '5GrwvaEF5zXb26Fz9rcQpDWS7u4m6DXb6T6TQvF9j5uQ8g6U';
const viewer = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';

interface PostSeed {
  post_id: string;
  owner_account: string;
  cid_number: string | null;
  post_category: 'normal' | 'campaign';
  content_format: 'normal' | 'article';
  created_at: number;
  post_state: string;
}

interface FollowSeed {
  owner_account: string;
  followed_account: string;
  created_at?: number;
}

describe('citizen profile repository', () => {
  it('round-trips a profile doc through R2', async () => {
    const env = fakeEnv();
    const doc: CitizenProfileDoc = {
      schema: 'citizenapp.square.profile.v1',
      owner_account: owner,
      display_name: '轻节点',
      bio: '链上公民',
      avatar_object_key: `profile/${owner}/avatar`,
      avatar_content_hash: '0xabc',
      banner_object_key: null,
      banner_content_hash: null,
      updated_at: 123
    };

    await writeProfileDoc(env, doc);
    const loaded = await readProfileDoc(env, owner);

    expect(loaded).toMatchObject({
      display_name: '轻节点',
      bio: '链上公民',
      avatar_object_key: `profile/${owner}/avatar`,
      updated_at: 123
    });
  });

  it('returns null for a missing or schema-invalid profile', async () => {
    const env = fakeEnv();
    expect(await readProfileDoc(env, owner)).toBeNull();

    await env.SQUARE_MEDIA.put(profileObjectKey(owner), JSON.stringify({ schema: 'wrong' }));
    expect(await readProfileDoc(env, owner)).toBeNull();
  });
});

describe('GET /v1/square/users/:account', () => {
  it('reports counts, certification and follow state for the viewer', async () => {
    const env = fakeEnv({
      posts: [
        published({ post_id: 'p1', cid_number: 'CN001-CTZN-000000001-2026', created_at: 200 }),
        published({ post_id: 'p2', cid_number: null, created_at: 100 })
      ],
      // 认证真源=链上身份：投票公民携带 cid，主页据此判认证。
      identity: { identity_level: 'voting', cid_number: 'CN001-CTZN-000000001-2026' },
      // 购买了民主会员且有效 → 徽章带勾（会员与身份解耦，勾只看会员是否有效）。
      membership: { membership_level: 'democracy' },
      follows: [
        { owner_account: owner, followed_account: 'a______________1' },
        { owner_account: owner, followed_account: 'a______________2' },
        { owner_account: viewer, followed_account: owner }
      ],
      session: { token: 'tok', owner_account: viewer }
    });

    const response = await getUserProfileRoute(
      request(`https://w/v1/square/users/${owner}`, { authToken: 'tok' }),
      env,
      owner
    );
    const body = (await response.json()) as { profile: Record<string, unknown> };

    expect(body.profile).toMatchObject({
      owner_account: owner,
      is_certified: true,
      identity_level: 'voting',
      membership_level: 'democracy',
      membership_active: true,
      cid_number: 'CN001-CTZN-000000001-2026',
      is_following: true
    });
    expect(body.profile.counts).toEqual({ following: 2, followers: 1, posts: 2 });
  });

  it('reports identity and membership independently (decoupled)', async () => {
    // 会员与身份解耦（ADR-036）：竞选身份可只买自由会员，两轴各自上报、互不影响。
    const env = fakeEnv({
      identity: { identity_level: 'candidate', cid_number: 'CN001-CTZN-000000009-2026' },
      membership: { membership_level: 'freedom' }
    });
    const response = await getUserProfileRoute(
      request(`https://w/v1/square/users/${owner}`, { authToken: 'tok' }),
      env,
      owner
    );
    const body = (await response.json()) as { profile: Record<string, unknown> };
    expect(body.profile).toMatchObject({
      identity_level: 'candidate',
      membership_level: 'freedom',
      membership_active: true
    });
  });

  it('reports a cancelled membership as active until paid_until', async () => {
    const env = fakeEnv({
      identity: { identity_level: 'voting', cid_number: 'CN001-CTZN-000000001-2026' },
      membership: { membership_level: 'democracy', subscription_status: 'cancelled' }
    });
    const response = await getUserProfileRoute(
      request(`https://w/v1/square/users/${owner}`, { authToken: 'tok' }),
      env,
      owner
    );
    const body = (await response.json()) as { profile: Record<string, unknown> };
    expect(body.profile).toMatchObject({
      identity_level: 'voting',
      membership_level: 'democracy',
      membership_active: true
    });
  });

  it('marks a candidate identity account as certified candidate', async () => {
    const env = fakeEnv({
      identity: { identity_level: 'candidate', cid_number: 'CN001-CTZN-000000009-2026' }
    });
    const response = await getUserProfileRoute(
      request(`https://w/v1/square/users/${owner}`, { authToken: 'tok' }),
      env,
      owner
    );
    const body = (await response.json()) as { profile: Record<string, unknown> };

    expect(body.profile).toMatchObject({
      is_certified: true,
      identity_level: 'candidate',
      cid_number: 'CN001-CTZN-000000009-2026'
    });
  });

  it('is wallet-readable and reports an unverified visitor when no chain identity', async () => {
    // 无身份桩 + 未配 RPC → 软降级为访客（未认证），不因链上不可用而报错。
    const env = fakeEnv({ posts: [], follows: [] });
    const response = await getUserProfileRoute(
      request(`https://w/v1/square/users/${owner}`, { authToken: 'tok' }),
      env,
      owner
    );
    const body = (await response.json()) as { profile: Record<string, unknown> };

    expect(body.profile).toMatchObject({
      is_certified: false,
      identity_level: 'visitor',
      cid_number: null,
      is_following: false,
      display_name: ''
    });
  });
});

describe('PUT /v1/square/profile', () => {
  it('persists display_name and bio for the session owner', async () => {
    const env = fakeEnv({ session: { token: 'tok', owner_account: owner } });
    const response = await putProfileRoute(
      request('https://w/v1/square/profile', {
        method: 'PUT',
        authToken: 'tok',
        body: { display_name: '  轻节点  ', bio: '个性签名' }
      }),
      env
    );
    const body = (await response.json()) as { profile: CitizenProfileDoc };

    expect(body.profile.display_name).toBe('轻节点');
    expect(body.profile.bio).toBe('个性签名');
    expect(await readProfileDoc(env, owner)).toMatchObject({ display_name: '轻节点' });
  });

  it('rejects an avatar key outside the owner profile directory', async () => {
    const env = fakeEnv({ session: { token: 'tok', owner_account: owner } });
    await expect(
      putProfileRoute(
        request('https://w/v1/square/profile', {
          method: 'PUT',
          authToken: 'tok',
          body: { avatar_object_key: `profile/${viewer}/avatar` }
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'invalid_asset_key' });
  });

  it('rejects a non-fixed avatar key inside the owner profile directory', async () => {
    const env = fakeEnv({ session: { token: 'tok', owner_account: owner } });
    await expect(
      putProfileRoute(
        request('https://w/v1/square/profile', {
          method: 'PUT',
          authToken: 'tok',
          body: { avatar_object_key: `profile/${owner}/avatar_extra` }
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'invalid_asset_key' });
  });

  it('rejects a display_name over the length limit', async () => {
    const env = fakeEnv({ session: { token: 'tok', owner_account: owner } });
    await expect(
      putProfileRoute(
        request('https://w/v1/square/profile', {
          method: 'PUT',
          authToken: 'tok',
          body: { display_name: 'x'.repeat(41) }
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'profile_field_too_long' });
  });
});

describe('GET /v1/square/users/:account/posts', () => {
  it('filters by category and paginates by cursor', async () => {
    const env = fakeEnv({
      posts: [
        published({ post_id: 'c1', post_category: 'campaign', created_at: 300 }),
        published({ post_id: 'n1', post_category: 'normal', created_at: 200 }),
        published({ post_id: 'n2', post_category: 'normal', created_at: 100 })
      ]
    });

    const campaign = await readPosts(env, `category=campaign`);
    expect(campaign.posts.map((p) => p.post_id)).toEqual(['c1']);

    const page1 = await readPosts(env, `limit=2`);
    expect(page1.posts.map((p) => p.post_id)).toEqual(['c1', 'n1']);
    expect(page1.next_cursor).toBe(200);

    const page2 = await readPosts(env, `limit=2&cursor=200`);
    expect(page2.posts.map((p) => p.post_id)).toEqual(['n2']);
    expect(page2.next_cursor).toBeNull();
  });

  it('filters by content_format so articles and short posts separate', async () => {
    const env = fakeEnv({
      posts: [
        published({ post_id: 'a1', content_format: 'article', created_at: 300 }),
        published({ post_id: 'p1', content_format: 'normal', created_at: 200 })
      ]
    });

    const articles = await readPosts(env, 'content_format=article');
    expect(articles.posts.map((p) => p.post_id)).toEqual(['a1']);

    const shorts = await readPosts(env, 'category=normal&content_format=normal');
    expect(shorts.posts.map((p) => p.post_id)).toEqual(['p1']);
  });

  async function readPosts(
    env: Env,
    query: string
  ): Promise<{ posts: Array<{ post_id: string }>; next_cursor: number | null }> {
    const response = await getUserPostsRoute(
      request(`https://w/v1/square/users/${owner}/posts?${query}`, { authToken: 'tok' }),
      env,
      owner
    );
    return (await response.json()) as {
      posts: Array<{ post_id: string }>;
      next_cursor: number | null;
    };
  }
});

describe('GET /v1/square/users/:account/follows', () => {
  it('lists following and followers ordered by recency', async () => {
    const env = fakeEnv({
      follows: [
        { owner_account: owner, followed_account: 'a______________1', created_at: 100 },
        { owner_account: owner, followed_account: 'a______________2', created_at: 200 },
        { owner_account: 'a______________3', followed_account: owner, created_at: 300 }
      ]
    });

    const following = await readFollows(env, 'type=following');
    expect(following.accounts.map((a) => a.owner_account)).toEqual([
      'a______________2',
      'a______________1'
    ]);

    const followers = await readFollows(env, 'type=followers');
    expect(followers.accounts.map((a) => a.owner_account)).toEqual(['a______________3']);
  });

  async function readFollows(
    env: Env,
    query: string
  ): Promise<{
    accounts: Array<{ owner_account: string; created_at: number }>;
    next_cursor: number | null;
  }> {
    const response = await getUserFollowsRoute(
      request(`https://w/v1/square/users/${owner}/follows?${query}`, { authToken: 'tok' }),
      env,
      owner
    );
    return (await response.json()) as {
      accounts: Array<{ owner_account: string; created_at: number }>;
      next_cursor: number | null;
    };
  }
});

function published(overrides: Partial<PostSeed> & Pick<PostSeed, 'post_id'>): PostSeed {
  return {
    owner_account: owner,
    cid_number: null,
    post_category: 'normal',
    content_format: 'normal',
    created_at: 0,
    post_state: 'published',
    ...overrides
  };
}

interface FakeEnvOptions {
  posts?: PostSeed[];
  follows?: FollowSeed[];
  session?: { token: string; owner_account: string };
  /// 预置 owner 的链上身份（写进 SQUARE_CACHE 命中缓存版身份读取）；缺省=未配置→软降级为访客。
  identity?: { identity_level: 'visitor' | 'voting' | 'candidate'; cid_number?: string | null };
  /// 预置 owner 的会员购买（对应 D1 square_memberships 一行）；缺省=未购买（无行）。
  membership?: {
    membership_level: 'freedom' | 'democracy' | 'spark';
    subscription_status?: string;
    paid_until?: number;
  };
}

function fakeEnv(options: FakeEnvOptions = {}): Env {
  const posts = options.posts ?? [];
  const follows = options.follows ?? [];
  const kv = new Map<string, unknown>();
  if (!options.session) {
    const defaultSession: SessionState = {
      owner_account: viewer,
      device_key_hash: 'a'.repeat(64),
      created_at: 0,
      expires_at: Date.now() + 60_000
    };
    kv.set('square_session:tok', defaultSession);
  }
  if (options.session) {
    const session: SessionState = {
      owner_account: options.session.owner_account,
      device_key_hash: 'a'.repeat(64),
      created_at: 0,
      expires_at: Date.now() + 60_000
    };
    kv.set(`square_session:${options.session.token}`, session);
  }
  if (options.identity) {
    const level = options.identity.identity_level;
    kv.set(
      `square_identity:${owner}`,
      JSON.stringify({
        owner_account: owner,
        identity_level: level,
        has_voting_identity: level !== 'visitor',
        has_candidate_identity: level === 'candidate',
        cid_number: options.identity.cid_number ?? null,
        checked_at: 0
      })
    );
  }

  const membershipRow = options.membership
    ? {
        owner_account: owner,
        membership_level: options.membership.membership_level,
        subscription_status: options.membership.subscription_status ?? 'active',
        paid_until: options.membership.paid_until ?? Date.now() + 60_000,
        chain_timestamp: Date.now(),
        chain_observed_at: Date.now()
      }
    : null;

  return {
    DB: new FakeDb(posts, follows, membershipRow) as unknown as D1Database,
    SQUARE_MEDIA: new FakeR2() as unknown as R2Bucket,
    SQUARE_CACHE: new FakeKv(kv) as unknown as KVNamespace
  } as unknown as Env;
}

function request(
  url: string,
  init: { method?: string; authToken?: string; body?: unknown } = {}
): Request {
  const headers = new Headers();
  if (init.authToken) {
    headers.set('authorization', `Bearer ${init.authToken}`);
  }
  if (init.body !== undefined) {
    headers.set('content-type', 'application/json');
  }
  return new Request(url, {
    method: init.method ?? 'GET',
    headers,
    body: init.body !== undefined ? JSON.stringify(init.body) : undefined
  });
}

class FakeR2 {
  private readonly store = new Map<string, string>();

  async get(key: string): Promise<{ text: () => Promise<string> } | null> {
    const value = this.store.get(key);
    return value === undefined ? null : { text: async () => value };
  }

  async put(key: string, value: string | ArrayBuffer | ArrayBufferView): Promise<void> {
    if (typeof value === 'string') {
      this.store.set(key, value);
      return;
    }
    const bytes = value instanceof ArrayBuffer
      ? new Uint8Array(value)
      : new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
    this.store.set(key, new TextDecoder().decode(bytes));
  }
}

class FakeKv {
  constructor(private readonly store: Map<string, unknown>) {}

  async get<T>(key: string): Promise<T | null> {
    return (this.store.get(key) as T) ?? null;
  }
}

class FakeDb {
  constructor(
    private readonly posts: PostSeed[],
    private readonly follows: FollowSeed[],
    private readonly membership: Record<string, unknown> | null = null
  ) {}

  prepare(sql: string): FakeStmt {
    return new FakeStmt(this.posts, this.follows, this.membership, sql);
  }
}

class FakeStmt {
  private binds: unknown[] = [];

  constructor(
    private readonly posts: PostSeed[],
    private readonly follows: FollowSeed[],
    private readonly membership: Record<string, unknown> | null,
    private readonly sql: string
  ) {}

  bind(...args: unknown[]): FakeStmt {
    this.binds = args;
    return this;
  }

  async first<T>(): Promise<T | null> {
    const sql = this.sql;
    const b0 = this.binds[0] as string;

    if (sql.includes('square_memberships')) {
      const m = this.membership;
      return m && m.owner_account === b0 ? (m as T) : null;
    }

    if (sql.includes('square_follows') && sql.includes('followed_account = ?') &&
      sql.includes('owner_account = ?')) {
      const b1 = this.binds[1] as string;
      const hit = this.follows.some(
        (f) => f.owner_account === b0 && f.followed_account === b1
      );
      return (hit ? ({ n: 1 } as T) : null);
    }
    if (sql.includes('COUNT(*)') && sql.includes('square_follows') &&
      sql.includes('followed_account = ?')) {
      return { n: this.follows.filter((f) => f.followed_account === b0).length } as T;
    }
    if (sql.includes('COUNT(*)') && sql.includes('square_follows')) {
      return { n: this.follows.filter((f) => f.owner_account === b0).length } as T;
    }
    if (sql.includes('COUNT(*)') && sql.includes('square_posts')) {
      return {
        n: this.posts.filter(
          (p) => p.owner_account === b0 && p.post_state === 'published'
        ).length
      } as T;
    }
    if (sql.includes('cid_number FROM square_posts')) {
      const row = this.posts
        .filter(
          (p) =>
            p.owner_account === b0 &&
            p.post_state === 'published' &&
            p.cid_number !== null
        )
        .sort((a, b) => b.created_at - a.created_at)[0];
      return row ? ({ cid_number: row.cid_number } as T) : null;
    }
    if (sql.includes('FROM square_uploads')) {
      return null;
    }
    return null;
  }

  async all<T>(): Promise<{ results: T[] }> {
    if (this.sql.includes('FROM square_follows')) {
      const isFollowing = this.sql.includes('followed_account AS owner_account');
      let fi = 0;
      const account = this.binds[fi++] as string;
      const cursor = this.sql.includes('created_at < ?')
        ? (this.binds[fi++] as number)
        : null;
      const limit = this.binds[fi++] as number;
      const rows = this.follows
        .filter((f) =>
          isFollowing ? f.owner_account === account : f.followed_account === account
        )
        .map((f) => ({
          owner_account: isFollowing ? f.followed_account : f.owner_account,
          created_at: f.created_at ?? 0
        }))
        .filter((r) => (cursor !== null ? r.created_at < cursor : true))
        .sort((a, b) => b.created_at - a.created_at)
        .slice(0, limit);
      return { results: rows as unknown as T[] };
    }

    let i = 0;
    const ownerAccount = this.binds[i++] as string;
    const category = this.sql.includes('post_category = ?')
      ? (this.binds[i++] as string)
      : null;
    const contentFormat = this.sql.includes('content_format = ?')
      ? (this.binds[i++] as string)
      : null;
    const cursor = this.sql.includes('created_at < ?')
      ? (this.binds[i++] as number)
      : null;
    const limit = this.binds[i++] as number;

    const results = this.posts
      .filter((p) => p.owner_account === ownerAccount && p.post_state === 'published')
      .filter((p) => (category ? p.post_category === category : true))
      .filter((p) => (contentFormat ? p.content_format === contentFormat : true))
      .filter((p) => (cursor !== null ? p.created_at < cursor : true))
      .sort((a, b) => b.created_at - a.created_at)
      .slice(0, limit);

    return { results: results as unknown as T[] };
  }

  async run(): Promise<{ meta: { changes: number } }> {
    return { meta: { changes: 1 } };
  }
}
