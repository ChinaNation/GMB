import { describe, expect, it } from 'vitest';
import {
  getUserFollowsRoute,
  getUserPostsRoute,
  getUserProfileRoute,
  putProfileRoute
} from '../src/profiles/service';
import { setFollowNotifyRoute } from '../src/feeds/follows';
import { readProfileDoc, writeProfileDoc } from '../src/profiles/repository';
import { profileObjectKey } from '../src/storage/r2_keys';
import type { CitizenProfileDoc, Env, SessionState } from '../src/types';

// 社交面统一按身份主键 cid_number 寻址(F1):路由第三参 = 目标 cid_number(不再是 account_id)。
const accountId = '0x1111111111111111111111111111111111111111111111111111111111111111';
const viewer = '0x2222222222222222222222222222222222222222222222222222222222222222';
// 账户↔CID 固定映射:身份主键 = cid_number(换绑不丢),归属数据全部按 cid 存取。
const targetCid = 'CN001-CTZN-000000001-2026'; // accountId 绑定的 CID
const viewerCid = 'CN220-CTZN2-198805200-2026'; // viewer 绑定的 CID(= 标准会话 CID)
const candidateCid = 'CN001-CTZN-000000009-2026'; // 独立于 targetCid 的另一目标身份主键

interface PostSeed {
  post_id: string;
  account_id: string;
  /// 身份主键:发布者 cid_number(帖子归属键)。
  cid_number: string;
  post_category: 'normal' | 'campaign';
  content_format: 'normal' | 'article';
  created_at: number;
  post_state: string;
}

interface FollowSeed {
  /// 关注关系双端均为身份主键 cid_number。
  follower_cid_number: string;
  followed_cid_number: string;
  created_at?: number;
  /// 关注即默认开通知（1）；0=对该关注静音。缺省视为 1。
  notify_enabled?: number;
}

describe('citizen profile repository', () => {
  it('round-trips a profile doc through R2', async () => {
    const env = fakeEnv();
    const doc: CitizenProfileDoc = {
      schema: 'citizenapp.square.profile.v1',
      cid_number: targetCid,
      display_name: '轻节点',
      bio: '链上公民',
      avatar_object_key: `profile/${targetCid}/avatar`,
      avatar_content_hash: '0xabc',
      banner_object_key: null,
      banner_content_hash: null,
      updated_at: 123
    };

    await writeProfileDoc(env, doc);
    const loaded = await readProfileDoc(env, targetCid);

    expect(loaded).toMatchObject({
      display_name: '轻节点',
      bio: '链上公民',
      avatar_object_key: `profile/${targetCid}/avatar`,
      updated_at: 123
    });
  });

  it('returns null for a missing or schema-invalid profile', async () => {
    const env = fakeEnv();
    expect(await readProfileDoc(env, targetCid)).toBeNull();

    await env.SQUARE_MEDIA.put(profileObjectKey(targetCid), JSON.stringify({ schema: 'wrong' }));
    expect(await readProfileDoc(env, targetCid)).toBeNull();
  });
});

describe('GET /v1/square/users/:account', () => {
  it('reports counts, certification and follow state for the viewer', async () => {
    const env = fakeEnv({
      // 目标身份的两条已发布帖(归属键 cid_number = targetCid)。
      posts: [
        published({ post_id: 'p1', created_at: 200 }),
        published({ post_id: 'p2', created_at: 100 })
      ],
      // 认证真源=链上身份：投票公民携带 cid，主页据此判认证。
      identity: { identity_level: 'voting', cid_number: targetCid },
      // 购买了民主会员且有效 → 徽章带勾（会员与身份解耦，勾只看会员是否有效）。
      membership: { membership_level: 'democracy' },
      // 关注关系全部按 cid:目标关注两人,viewer 关注目标。
      follows: [
        { follower_cid_number: targetCid, followed_cid_number: 'CN001-CTZN-000000004-2026' },
        { follower_cid_number: targetCid, followed_cid_number: 'CN001-CTZN-000000005-2026' },
        { follower_cid_number: viewerCid, followed_cid_number: targetCid }
      ],
      session: { token: 'tok', account_id: viewer }
    });

    const response = await getUserProfileRoute(
      request(`https://w/v1/square/users/${targetCid}`, { authToken: 'tok' }),
      env,
      targetCid
    );
    const body = (await response.json()) as { profile: Record<string, unknown> };

    expect(body.profile).toMatchObject({
      account_id: accountId,
      is_certified: true,
      identity_level: 'voting',
      membership_level: 'democracy',
      membership_active: true,
      cid_number: targetCid,
      is_following: true
    });
    expect(body.profile.counts).toEqual({ following: 2, followers: 1, posts: 2 });
  });

  it('reports identity and membership independently (decoupled)', async () => {
    // 会员与身份解耦（ADR-036）：竞选身份可只买自由会员，两轴各自上报、互不影响。
    const env = fakeEnv({
      identity: { identity_level: 'candidate', cid_number: candidateCid },
      membership: { membership_level: 'freedom' }
    });
    const response = await getUserProfileRoute(
      request(`https://w/v1/square/users/${candidateCid}`, { authToken: 'tok' }),
      env,
      candidateCid
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
      identity: { identity_level: 'voting', cid_number: targetCid },
      membership: { membership_level: 'democracy', subscription_status: 'cancelled' }
    });
    const response = await getUserProfileRoute(
      request(`https://w/v1/square/users/${targetCid}`, { authToken: 'tok' }),
      env,
      targetCid
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
      identity: { identity_level: 'candidate', cid_number: candidateCid }
    });
    const response = await getUserProfileRoute(
      request(`https://w/v1/square/users/${candidateCid}`, { authToken: 'tok' }),
      env,
      candidateCid
    );
    const body = (await response.json()) as { profile: Record<string, unknown> };

    expect(body.profile).toMatchObject({
      is_certified: true,
      identity_level: 'candidate',
      cid_number: candidateCid
    });
  });

  it('is wallet-readable and reports an unverified visitor when no chain identity', async () => {
    // 目标 cid 无 active 身份桩 + 未配 RPC → 软降级为访客（未认证，account_id 空、cid_number=目标 cid），
    // 不因链上不可用而报错。
    const env = fakeEnv({ posts: [], follows: [] });
    const response = await getUserProfileRoute(
      request(`https://w/v1/square/users/${targetCid}`, { authToken: 'tok' }),
      env,
      targetCid
    );
    const body = (await response.json()) as { profile: Record<string, unknown> };

    expect(body.profile).toMatchObject({
      account_id: '',
      is_certified: false,
      identity_level: 'visitor',
      cid_number: targetCid,
      is_following: false,
      display_name: ''
    });
  });
});

describe('PUT /v1/square/profile', () => {
  it('persists display_name and bio for the session cid', async () => {
    const env = fakeEnv({
      session: { token: 'tok', account_id: accountId },
      identity: { identity_level: 'voting', cid_number: targetCid }
    });
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
    expect(await readProfileDoc(env, targetCid)).toMatchObject({ display_name: '轻节点' });
  });

  it('rejects an avatar key outside the cid profile directory', async () => {
    const env = fakeEnv({ session: { token: 'tok', account_id: accountId } });
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

  it('rejects a non-fixed avatar key inside the cid profile directory', async () => {
    const env = fakeEnv({ session: { token: 'tok', account_id: accountId } });
    await expect(
      putProfileRoute(
        request('https://w/v1/square/profile', {
          method: 'PUT',
          authToken: 'tok',
          body: { avatar_object_key: `profile/${targetCid}/avatar_extra` }
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'invalid_asset_key' });
  });

  it('rejects a display_name over the length limit', async () => {
    const env = fakeEnv({ session: { token: 'tok', account_id: accountId } });
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
      identity: { identity_level: 'voting', cid_number: targetCid },
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
      identity: { identity_level: 'voting', cid_number: targetCid },
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
      request(`https://w/v1/square/users/${targetCid}/posts?${query}`, { authToken: 'tok' }),
      env,
      targetCid
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
      identity: { identity_level: 'voting', cid_number: targetCid },
      follows: [
        { follower_cid_number: targetCid, followed_cid_number: 'CN001-CTZN-000000004-2026', created_at: 100 },
        { follower_cid_number: targetCid, followed_cid_number: 'CN001-CTZN-000000005-2026', created_at: 200 },
        { follower_cid_number: 'CN001-CTZN-000000006-2026', followed_cid_number: targetCid, created_at: 300 }
      ]
    });

    // 列表项为身份主键 cid_number(响应字段 entries)。
    const following = await readFollows(env, 'type=following');
    expect(following.entries.map((e) => e.cid_number)).toEqual([
      'CN001-CTZN-000000005-2026',
      'CN001-CTZN-000000004-2026'
    ]);

    const followers = await readFollows(env, 'type=followers');
    expect(followers.entries.map((e) => e.cid_number)).toEqual(['CN001-CTZN-000000006-2026']);
  });

  async function readFollows(
    env: Env,
    query: string
  ): Promise<{
    entries: Array<{ cid_number: string; created_at: number }>;
    next_cursor: number | null;
  }> {
    const response = await getUserFollowsRoute(
      request(`https://w/v1/square/users/${targetCid}/follows?${query}`, { authToken: 'tok' }),
      env,
      targetCid
    );
    return (await response.json()) as {
      entries: Array<{ cid_number: string; created_at: number }>;
      next_cursor: number | null;
    };
  }
});

describe('post notify (is_notifying + PUT .../notify)', () => {
  it('reports is_notifying true when following with notify enabled (default)', async () => {
    const env = fakeEnv({
      identity: { identity_level: 'voting', cid_number: targetCid },
      follows: [{ follower_cid_number: viewerCid, followed_cid_number: targetCid }],
      session: { token: 'tok', account_id: viewer }
    });
    const body = await readProfile(env);
    expect(body.profile).toMatchObject({ is_following: true, is_notifying: true });
  });

  it('reports is_notifying false when following but muted', async () => {
    const env = fakeEnv({
      identity: { identity_level: 'voting', cid_number: targetCid },
      follows: [
        { follower_cid_number: viewerCid, followed_cid_number: targetCid, notify_enabled: 0 }
      ],
      session: { token: 'tok', account_id: viewer }
    });
    const body = await readProfile(env);
    expect(body.profile).toMatchObject({ is_following: true, is_notifying: false });
  });

  it('reports is_notifying false when not following', async () => {
    const env = fakeEnv({
      identity: { identity_level: 'voting', cid_number: targetCid },
      follows: [],
      session: { token: 'tok', account_id: viewer }
    });
    const body = await readProfile(env);
    expect(body.profile).toMatchObject({ is_following: false, is_notifying: false });
  });

  it('PUT .../notify accepts a boolean and echoes the new state', async () => {
    const env = fakeEnv({
      identity: { identity_level: 'voting', cid_number: targetCid },
      follows: [{ follower_cid_number: viewerCid, followed_cid_number: targetCid }],
      session: { token: 'tok', account_id: viewer }
    });
    const response = await setFollowNotifyRoute(
      request(`https://w/v1/square/follows/${targetCid}/notify`, {
        method: 'PUT',
        authToken: 'tok',
        body: { enabled: false }
      }),
      env
    );
    const body = (await response.json()) as { ok: boolean; notify_enabled: boolean };
    expect(body).toMatchObject({ ok: true, notify_enabled: false });
  });

  it('PUT .../notify rejects a non-boolean enabled', async () => {
    const env = fakeEnv({ session: { token: 'tok', account_id: viewer } });
    await expect(
      setFollowNotifyRoute(
        request(`https://w/v1/square/follows/${targetCid}/notify`, {
          method: 'PUT',
          authToken: 'tok',
          body: { enabled: 'yes' }
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'invalid_enabled' });
  });

  async function readProfile(env: Env): Promise<{ profile: Record<string, unknown> }> {
    const response = await getUserProfileRoute(
      request(`https://w/v1/square/users/${targetCid}`, { authToken: 'tok' }),
      env,
      targetCid
    );
    return (await response.json()) as { profile: Record<string, unknown> };
  }
});

function published(overrides: Partial<PostSeed> & Pick<PostSeed, 'post_id'>): PostSeed {
  return {
    account_id: accountId,
    cid_number: targetCid,
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
  session?: { token: string; account_id: string };
  /// 预置目标 cid 的链上身份（写进 SQUARE_CACHE 命中 fetchChainIdentityStateByCidCached）；缺省=未配置→软降级为访客（account_id 空）。
  identity?: { identity_level: 'visitor' | 'voting' | 'candidate'; cid_number?: string | null };
  /// 预置 accountId 的会员购买（对应 D1 square_memberships 一行）；缺省=未购买（无行）。
  membership?: {
    membership_level: 'freedom' | 'democracy' | 'spark';
    subscription_status?: string;
    paid_until?: number;
  };
}

/// 会话身份主键 = 账户绑定的 cid_number(与路由内链上 resolve 同一映射)。
function cidForAccount(account: string): string {
  return account === accountId ? targetCid : viewerCid;
}

function fakeEnv(options: FakeEnvOptions = {}): Env {
  const posts = options.posts ?? [];
  const follows = options.follows ?? [];
  const kv = new Map<string, unknown>();
  const sessionToken = options.session?.token ?? 'tok';
  if (sessionToken !== 'tok') {
    throw new Error('profiles fixture only supports token=tok');
  }
  const sessionAccount = options.session?.account_id ?? viewer;
  const session: SessionState = {
    cid_number: cidForAccount(sessionAccount),
    account_id: sessionAccount,
    device_key_hash: 'a'.repeat(64),
    created_at: 0,
    expires_at: Date.now() + 60_000
  };
  // 本文件所有鉴权请求固定使用 token=tok；KV 键只保存 token 的 SHA-256。
  kv.set(
    'square_session:1a7674eb4ee78df7e1ac439a93c3fa8e3c945784d4dec9fd8e3011738b2f1d62',
    session
  );
  if (options.identity) {
    const level = options.identity.identity_level;
    // 社交面按身份主键 cid_number 寻址:身份缓存键 = square_identity_cid:<cid>
    // (命中 fetchChainIdentityStateByCidCached);account_id = 该 cid 当前绑定钱包账户。
    const identityCid = options.identity.cid_number ?? null;
    kv.set(
      `square_identity_cid:${identityCid}`,
      JSON.stringify({
        account_id: accountId,
        identity_level: level,
        has_voting_identity: level !== 'visitor',
        has_candidate_identity: level === 'candidate',
        cid_number: identityCid,
        checked_at: 0
      })
    );
  }

  const membershipRow = options.membership
    ? {
        // 身份主键 cid_number = 目标账户链上 resolve 出的 cid（buildProfileResponse 按此查会员）。
        cid_number: options.identity?.cid_number ?? null,
        account_id: accountId,
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

    // 会员按身份主键 cid_number 命中（getMembership 绑定目标 cid）。
    if (sql.includes('square_memberships')) {
      const m = this.membership;
      return m && m.cid_number === b0 ? (m as T) : null;
    }

    // isFollowing / isNotifying：关注关系双端 cid（follower + followed），非计数。
    if (
      sql.includes('square_follows') &&
      sql.includes('follower_cid_number = ?') &&
      sql.includes('followed_cid_number = ?')
    ) {
      const b1 = this.binds[1] as string;
      const follow = this.follows.find(
        (f) => f.follower_cid_number === b0 && f.followed_cid_number === b1
      );
      // isNotifying 读 notify_enabled；isFollowing 读 1 AS n。
      if (sql.includes('notify_enabled')) {
        return follow ? ({ notify_enabled: follow.notify_enabled ?? 1 } as T) : null;
      }
      return follow ? ({ n: 1 } as T) : null;
    }
    // 粉丝数：followed_cid_number = ? 命中该身份被关注的条数。
    if (sql.includes('COUNT(*)') && sql.includes('square_follows') &&
      sql.includes('followed_cid_number = ?')) {
      return { n: this.follows.filter((f) => f.followed_cid_number === b0).length } as T;
    }
    // 关注数：follower_cid_number = ? 命中该身份主动关注的条数。
    if (sql.includes('COUNT(*)') && sql.includes('square_follows') &&
      sql.includes('follower_cid_number = ?')) {
      return { n: this.follows.filter((f) => f.follower_cid_number === b0).length } as T;
    }
    // 帖子数：按身份主键 cid_number 聚合已发布帖。
    if (sql.includes('COUNT(*)') && sql.includes('square_posts')) {
      return {
        n: this.posts.filter(
          (p) => p.cid_number === b0 && p.post_state === 'published'
        ).length
      } as T;
    }
    if (sql.includes('FROM square_uploads')) {
      return null;
    }
    return null;
  }

  async all<T>(): Promise<{ results: T[] }> {
    // 批量会员：按身份主键 cid_number IN(...) 命中；测试仅置一行。
    if (this.sql.includes('square_memberships')) {
      const cidNumbers = this.binds as string[];
      const rows = this.membership && cidNumbers.includes(this.membership.cid_number as string)
        ? [this.membership]
        : [];
      return { results: rows as unknown as T[] };
    }

    if (this.sql.includes('FROM square_follows')) {
      // following: WHERE follower_cid_number = ?（选 followed 列）；followers 反之。
      const isFollowing = this.sql.includes('follower_cid_number = ?');
      let fi = 0;
      const key = this.binds[fi++] as string;
      const cursor = this.sql.includes('created_at < ?')
        ? (this.binds[fi++] as number)
        : null;
      const limit = this.binds[fi++] as number;
      const rows = this.follows
        .filter((f) =>
          isFollowing ? f.follower_cid_number === key : f.followed_cid_number === key
        )
        .map((f) => ({
          cid_number: isFollowing ? f.followed_cid_number : f.follower_cid_number,
          created_at: f.created_at ?? 0
        }))
        .filter((r) => (cursor !== null ? r.created_at < cursor : true))
        .sort((a, b) => b.created_at - a.created_at)
        .slice(0, limit);
      return { results: rows as unknown as T[] };
    }

    // listAuthorPosts：按身份主键 cid_number 过滤已发布帖。
    let i = 0;
    const cidNumber = this.binds[i++] as string;
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
      .filter((p) => p.cid_number === cidNumber && p.post_state === 'published')
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
