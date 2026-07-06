import type { Env } from '../types';
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import { nowMs } from '../shared/time';

const signalWeights: Record<string, number> = {
  view: 0.1,
  open_detail: 0.3,
  like: 2,
  hide: -3,
  not_interested: -2,
  report: -5
};

interface SignalRequest {
  post_id?: unknown;
  signal_type?: unknown;
}

export async function signalRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<SignalRequest>(request);
  if (typeof body.post_id !== 'string' || !body.post_id.startsWith('sqp_')) {
    throw new HttpError(400, 'invalid_post_id', '动态编号不合法');
  }
  if (typeof body.signal_type !== 'string' || !(body.signal_type in signalWeights)) {
    throw new HttpError(400, 'invalid_signal_type', '用户行为类型不合法');
  }

  const weight = signalWeights[body.signal_type];
  await env.DB.prepare(
    `INSERT INTO square_user_signals
      (owner_account, post_id, signal_type, weight, created_at)
      VALUES (?, ?, ?, ?, ?)`
  )
    .bind(session.owner_account, body.post_id, body.signal_type, weight, nowMs())
    .run();

  return jsonResponse({
    ok: true,
    post_id: body.post_id,
    signal_type: body.signal_type,
    weight
  });
}

export async function reportRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<SignalRequest>(request);
  if (typeof body.post_id !== 'string' || !body.post_id.startsWith('sqp_')) {
    throw new HttpError(400, 'invalid_post_id', '动态编号不合法');
  }

  await env.DB.prepare(
    `INSERT INTO square_user_signals
      (owner_account, post_id, signal_type, weight, created_at)
      VALUES (?, ?, 'report', ?, ?)`
  )
    .bind(session.owner_account, body.post_id, signalWeights.report, nowMs())
    .run();

  return jsonResponse({
    ok: true,
    post_id: body.post_id,
    signal_type: 'report'
  });
}
