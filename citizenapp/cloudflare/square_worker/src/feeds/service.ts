import type { Env, FeedKind } from '../types';
import { jsonResponse, requireSession } from '../shared/http';
import { listFeedPosts } from '../posts/repository';

function parseLimit(url: URL): number {
  const value = Number.parseInt(url.searchParams.get('limit') ?? '20', 10);
  return Number.isFinite(value) ? value : 20;
}

export async function feedRoute(
  request: Request,
  env: Env,
  feedKind: FeedKind
): Promise<Response> {
  const url = new URL(request.url);
  const session = await maybeSession(request, env);
  const posts = await listFeedPosts(env, feedKind, session?.owner_account ?? null, parseLimit(url));

  return jsonResponse({
    ok: true,
    feed_kind: feedKind,
    posts
  });
}

async function maybeSession(request: Request, env: Env) {
  const authorization = request.headers.get('authorization');
  if (!authorization?.startsWith('Bearer ')) {
    return null;
  }
  return requireSession(request, env);
}
