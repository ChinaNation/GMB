import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../src/media/cloudflare_assets', () => ({
  createStreamDownloadUrl: vi.fn(async () => 'https://download.test/video.mp4'),
  deleteProviderAsset: vi.fn(async () => undefined),
  copyStreamFromUrl: vi.fn(async () => 'str_uid_restored'),
}));
vi.mock('../src/storage/presigned', () => ({
  signR2GetUrl: vi.fn(async () => 'https://r2.test/archive.mp4'),
}));
import type { Env } from '../src/types';
import { restoreOwnerVideos, runVideoArchiveSweep } from '../src/membership/archive';

const DAY_MS = 24 * 60 * 60 * 1000;

interface FakeMembership {
  owner_account: string;
  subscription_status: string;
  entitlement_lapsed_at: number | null;
}

interface FakeVideo {
  upload_id: string;
  media_index: number;
  owner_account: string;
  media_kind: 'video' | 'image';
  provider: string;
  provider_asset_id: string;
  archive_state: string;
  archived_at: number | null;
  r2_archive_key: string | null;
  asset_state: string;
}

class FakeStmt {
  private args: unknown[] = [];
  constructor(private db: FakeDb, private sql: string) {}

  bind(...args: unknown[]): FakeStmt {
    this.args = args;
    return this;
  }

  async all<T>(): Promise<{ results: T[] }> {
    if (this.sql.includes('SELECT DISTINCT m.owner_account')) {
      const cutoff = this.args[0] as number;
      const limit = this.args[1] as number;
      const owners = [
        ...new Set(
          this.db.memberships
            .filter(
              (m) =>
                m.entitlement_lapsed_at !== null &&
                m.entitlement_lapsed_at <= cutoff &&
                (m.subscription_status === 'cancelled' ||
                  m.subscription_status === 'terminated') &&
                this.db.videos.some(
                  (v) =>
                    v.owner_account === m.owner_account &&
                    v.media_kind === 'video' &&
                    v.archive_state === 'live'
                )
            )
            .map((m) => m.owner_account)
        )
      ].slice(0, limit);
      return { results: owners.map((owner_account) => ({ owner_account })) as T[] };
    }
    if (this.sql.includes('FROM square_media_assets') && this.sql.includes('archive_state = ?')) {
      const owner = this.args[0] as string;
      const state = this.args[1] as string;
      const rows = this.db.videos.filter(
        (v) => v.owner_account === owner && v.media_kind === 'video' && v.archive_state === state
      );
      return { results: rows as unknown as T[] };
    }
    return { results: [] };
  }

  async run(): Promise<{ success: boolean }> {
    if (this.sql.includes("SET archive_state = 'archived'")) {
      // markArchived: bind(now, r2Key, now, upload_id, media_index)
      const r2Key = this.args[1] as string;
      const uploadId = this.args[3] as string;
      const mediaIndex = this.args[4] as number;
      const video = this.find(uploadId, mediaIndex);
      if (video) {
        video.archive_state = 'archived';
        video.archived_at = this.args[0] as number;
        video.r2_archive_key = r2Key;
      }
    } else if (
      this.sql.includes("archive_state = 'restoring'") &&
      this.sql.includes('provider_asset_id')
    ) {
      const video = this.find(this.args[2] as string, this.args[3] as number);
      if (video) {
        video.provider_asset_id = this.args[0] as string;
        video.archive_state = 'restoring';
        video.asset_state = 'processing';
      }
    } else if (
      this.sql.includes("archive_state = 'live'") &&
      this.sql.includes('asset_state')
    ) {
      // markRestoredLive: bind(uid, hls, dash, thumb, now, now, upload_id, media_index)
      const uid = this.args[0] as string;
      const uploadId = this.args[this.args.length - 2] as string;
      const mediaIndex = this.args[this.args.length - 1] as number;
      const video = this.find(uploadId, mediaIndex);
      if (video) {
        video.archive_state = 'live';
        video.provider_asset_id = uid;
        video.asset_state = 'ready';
        video.r2_archive_key = null;
      }
    } else if (this.sql.includes('SET archive_state = ?')) {
      // setArchiveState: bind(state, now, upload_id, media_index)
      const state = this.args[0] as string;
      const uploadId = this.args[2] as string;
      const mediaIndex = this.args[3] as number;
      const video = this.find(uploadId, mediaIndex);
      if (video) video.archive_state = state;
    }
    return { success: true };
  }

  private find(uploadId: string, mediaIndex: number): FakeVideo | undefined {
    return this.db.videos.find((v) => v.upload_id === uploadId && v.media_index === mediaIndex);
  }
}

class FakeDb {
  memberships: FakeMembership[] = [];
  videos: FakeVideo[] = [];
  prepare(sql: string): FakeStmt {
    return new FakeStmt(this, sql);
  }
}

class FakeR2 {
  private readonly sizes = new Map<string, number>();

  async put(key: string, value: ReadableStream<Uint8Array>): Promise<void> {
    const bytes = await new Response(value).arrayBuffer();
    this.sizes.set(key, bytes.byteLength);
  }

  async head(key: string): Promise<{ size: number } | null> {
    const size = this.sizes.get(key);
    return size === undefined ? null : { size };
  }
}

function video(overrides: Partial<FakeVideo> = {}): FakeVideo {
  return {
    upload_id: 'squ_1',
    media_index: 0,
    owner_account: 'owner_1',
    media_kind: 'video',
    provider: 'cloudflare_stream',
    provider_asset_id: 'str_uid_1',
    archive_state: 'live',
    archived_at: null,
    r2_archive_key: null,
    asset_state: 'ready',
    ...overrides
  };
}

function env(db: FakeDb, overrides: Partial<Env> = {}): Env {
  return {
    DB: db,
    SQUARE_MEDIA: new FakeR2(),
    ARCHIVE_ENABLED: '1',
    ARCHIVE_LAPSE_DAYS: '90',
    ...overrides
  } as unknown as Env;
}

describe('video cold archive', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response(new Uint8Array([1, 2, 3]), {
      headers: { 'content-length': '3', 'content-type': 'video/mp4' },
    })));
  });
  afterEach(() => vi.unstubAllGlobals());
  it('archives live video of an account lapsed past the threshold', async () => {
    const db = new FakeDb();
    db.memberships.push({
      owner_account: 'owner_1',
      subscription_status: 'cancelled',
      entitlement_lapsed_at: Date.now() - 100 * DAY_MS
    });
    db.videos.push(video());

    const result = await runVideoArchiveSweep(env(db));

    expect(result).toEqual({ owners: 1, archived: 1 });
    expect(db.videos[0].archive_state).toBe('archived');
    expect(db.videos[0].r2_archive_key).toBe('archive/owner_1/str_uid_1.mp4');
  });

  it('skips accounts that have not lapsed 90 days yet', async () => {
    const db = new FakeDb();
    db.memberships.push({
      owner_account: 'owner_1',
      subscription_status: 'cancelled',
      entitlement_lapsed_at: Date.now() - 10 * DAY_MS
    });
    db.videos.push(video());

    const result = await runVideoArchiveSweep(env(db));

    expect(result).toEqual({ owners: 0, archived: 0 });
    expect(db.videos[0].archive_state).toBe('live');
  });

  it('does nothing when the feature flag is off', async () => {
    const db = new FakeDb();
    db.memberships.push({
      owner_account: 'owner_1',
      subscription_status: 'cancelled',
      entitlement_lapsed_at: Date.now() - 100 * DAY_MS
    });
    db.videos.push(video());

    const result = await runVideoArchiveSweep(env(db, { ARCHIVE_ENABLED: '0' }));

    expect(result).toEqual({ owners: 0, archived: 0 });
    expect(db.videos[0].archive_state).toBe('live');
  });

  it('restores an archived video into processing until Stream webhook confirms ready', async () => {
    const db = new FakeDb();
    db.videos.push(
      video({ archive_state: 'archived', r2_archive_key: 'archive/owner_1/str_uid_1.mp4' })
    );

    const result = await restoreOwnerVideos(env(db), 'owner_1');

    expect(result).toEqual({ restored: 1 });
    expect(db.videos[0].archive_state).toBe('restoring');
    expect(db.videos[0].provider_asset_id).toBe('str_uid_restored');
    expect(db.videos[0].r2_archive_key).toBe('archive/owner_1/str_uid_1.mp4');
  });
});
