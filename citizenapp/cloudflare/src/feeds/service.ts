import type { Env, FeedKind } from '../types';
import { jsonResponse, requireSession } from '../shared/http';
import { listFeedPosts } from '../posts/repository';
import { addBrowseCount, assertBrowseAvailable, getBrowseState } from './browse';

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
  const session = await requireSession(request, env);
  const before = await getBrowseState(env, session.account_id);
  const limit = Math.min(parseLimit(url), assertBrowseAvailable(before));
  const posts = await listFeedPosts(env, feedKind, session.account_id, limit);
  const browse = await addBrowseCount(env, session.account_id, before, posts.length);

  return jsonResponse({
    ok: true,
    feed_kind: feedKind,
    posts,
    ...browse,
  });
}
