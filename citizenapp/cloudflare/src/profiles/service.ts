import type {
  AuthorContentFormat,
  AuthorPostCategory,
  CitizenProfileDoc,
  Env,
  UserProfileResponse
} from '../types';
import {
  HttpError,
  jsonResponse,
  maybeSession,
  parsePositiveInt,
  readJson,
  requireSession
} from '../shared/http';
import { assertOwnerAccount } from '../shared/ids';
import { nowMs } from '../shared/time';
import { profileAssetPrefix } from '../storage/r2_keys';
import {
  countUserStats,
  defaultProfileDoc,
  isFollowing,
  listAuthorPosts,
  listFollows,
  readLatestCidNumber,
  readProfileDoc,
  writeProfileDoc
} from './repository';

const DISPLAY_NAME_MAX = 40;
const BIO_MAX = 160;
const DEFAULT_AUTHOR_POST_LIMIT = 20;

interface ProfileUpdateRequest {
  display_name?: unknown;
  bio?: unknown;
  avatar_object_key?: unknown;
  avatar_content_hash?: unknown;
  banner_object_key?: unknown;
  banner_content_hash?: unknown;
}

/// GET /v1/square/users/:account —— 公开可读；带登录态时附带 is_following。
export async function getUserProfileRoute(
  request: Request,
  env: Env,
  accountRaw: string
): Promise<Response> {
  const ownerAccount = parseAccount(accountRaw);
  const viewer = await maybeSession(request, env);
  const profile = await buildProfileResponse(env, ownerAccount, viewer?.owner_account ?? null);
  return jsonResponse({ ok: true, profile });
}

/// GET 与 PUT 共用同一份主页响应装配：profile 文档 + 计数 + 认证 + is_following。
async function buildProfileResponse(
  env: Env,
  ownerAccount: string,
  viewerAccount: string | null
): Promise<UserProfileResponse> {
  const [doc, counts, cidNumber, following] = await Promise.all([
    readProfileDoc(env, ownerAccount),
    countUserStats(env, ownerAccount),
    readLatestCidNumber(env, ownerAccount),
    isFollowing(env, viewerAccount, ownerAccount)
  ]);

  const profile = doc ?? defaultProfileDoc(ownerAccount);
  return {
    owner_account: ownerAccount,
    display_name: profile.display_name,
    bio: profile.bio,
    avatar_object_key: profile.avatar_object_key,
    banner_object_key: profile.banner_object_key,
    cid_number: cidNumber,
    is_certified: cidNumber !== null,
    counts,
    is_following: following,
    updated_at: profile.updated_at
  };
}

/// GET /v1/square/users/:account/posts?category=&content_format=&limit=&cursor= —— 按作者分页。
export async function getUserPostsRoute(
  request: Request,
  env: Env,
  accountRaw: string
): Promise<Response> {
  const ownerAccount = parseAccount(accountRaw);
  const url = new URL(request.url);
  const category = parseCategory(url.searchParams.get('category'));
  const contentFormat = parseContentFormat(url.searchParams.get('content_format'));
  const limit = parsePositiveInt(
    url.searchParams.get('limit') ?? undefined,
    DEFAULT_AUTHOR_POST_LIMIT
  );
  const cursor = parseCursor(url.searchParams.get('cursor'));

  const posts = await listAuthorPosts(
    env,
    ownerAccount,
    category,
    contentFormat,
    limit,
    cursor
  );
  const nextCursor =
    posts.length >= limit ? posts[posts.length - 1]?.created_at ?? null : null;

  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    category,
    content_format: contentFormat,
    posts,
    next_cursor: nextCursor
  });
}

/// GET /v1/square/users/:account/follows?type=following|followers —— 关注/粉丝列表分页。
export async function getUserFollowsRoute(
  request: Request,
  env: Env,
  accountRaw: string
): Promise<Response> {
  const ownerAccount = parseAccount(accountRaw);
  const url = new URL(request.url);
  const type = url.searchParams.get('type') === 'followers' ? 'followers' : 'following';
  const limit = parsePositiveInt(
    url.searchParams.get('limit') ?? undefined,
    DEFAULT_AUTHOR_POST_LIMIT
  );
  const cursor = parseCursor(url.searchParams.get('cursor'));

  const accounts = await listFollows(env, ownerAccount, type, limit, cursor);
  const nextCursor =
    accounts.length >= limit ? accounts[accounts.length - 1]?.created_at ?? null : null;

  return jsonResponse({ ok: true, type, accounts, next_cursor: nextCursor });
}

/// PUT /v1/square/profile —— 仅本人可写；owner 从 session 派生。
export async function putProfileRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<ProfileUpdateRequest>(request);
  const existing = (await readProfileDoc(env, session.owner_account)) ??
    defaultProfileDoc(session.owner_account);

  const assetPrefix = profileAssetPrefix(session.owner_account);
  const next: CitizenProfileDoc = {
    schema: 'citizenapp.square.profile.v1',
    owner_account: session.owner_account,
    display_name: normalizeText(body.display_name, existing.display_name, DISPLAY_NAME_MAX),
    bio: normalizeText(body.bio, existing.bio, BIO_MAX),
    avatar_object_key: normalizeAssetKey(
      body.avatar_object_key,
      existing.avatar_object_key,
      assetPrefix
    ),
    avatar_content_hash: normalizeOptional(body.avatar_content_hash, existing.avatar_content_hash),
    banner_object_key: normalizeAssetKey(
      body.banner_object_key,
      existing.banner_object_key,
      assetPrefix
    ),
    banner_content_hash: normalizeOptional(body.banner_content_hash, existing.banner_content_hash),
    updated_at: nowMs()
  };

  await writeProfileDoc(env, next);
  // 返回与 GET 一致的完整主页响应（本人视角 is_following=false），让客户端单一解析。
  const profile = await buildProfileResponse(env, session.owner_account, session.owner_account);
  return jsonResponse({ ok: true, profile });
}

function parseAccount(accountRaw: string): string {
  try {
    return assertOwnerAccount(decodeURIComponent(accountRaw));
  } catch {
    throw new HttpError(400, 'invalid_owner_account', '钱包账户格式不合法');
  }
}

function parseCategory(value: string | null): AuthorPostCategory {
  if (value === 'normal' || value === 'campaign') {
    return value;
  }
  return 'all';
}

function parseContentFormat(value: string | null): AuthorContentFormat {
  if (value === 'normal' || value === 'article') {
    return value;
  }
  return 'all';
}

function parseCursor(value: string | null): number | null {
  if (!value) {
    return null;
  }
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}

/// 文本字段：未提供沿用旧值；提供则 trim 并强制长度上限，超限直接拒绝而非静默截断。
function normalizeText(value: unknown, fallback: string, max: number): string {
  if (value === undefined) {
    return fallback;
  }
  if (typeof value !== 'string') {
    throw new HttpError(400, 'invalid_profile_field', '资料字段必须是文本');
  }
  const trimmed = value.trim();
  if (trimmed.length > max) {
    throw new HttpError(400, 'profile_field_too_long', `资料字段超过 ${max} 字上限`);
  }
  return trimmed;
}

/// 头像/背景对象 key：未提供沿用旧值；提供则必须落在本人 profile 前缀下，杜绝越权写他人对象。
function normalizeAssetKey(
  value: unknown,
  fallback: string | null,
  assetPrefix: string
): string | null {
  if (value === undefined) {
    return fallback;
  }
  if (value === null || value === '') {
    return null;
  }
  if (typeof value !== 'string' || !value.startsWith(assetPrefix)) {
    throw new HttpError(400, 'invalid_asset_key', '资源对象不属于本账户目录');
  }
  return value;
}

function normalizeOptional(value: unknown, fallback: string | null): string | null {
  if (value === undefined) {
    return fallback;
  }
  if (value === null || value === '') {
    return null;
  }
  return typeof value === 'string' ? value : fallback;
}
