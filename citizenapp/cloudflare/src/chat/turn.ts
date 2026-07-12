import type { Env } from '../types';
import { HttpError, jsonResponse, requireSession } from '../shared/http';
import { nowMs } from '../shared/time';

const TURN_TTL_SECONDS = 300;

interface TurnResponse {
  iceServers?: Array<{
    urls: string[];
    username?: string;
    credential?: string;
  }>;
}

/** 为当前钱包设备签发短期 TURN 凭证；附件字节只经 TURN 中继，不进入 Worker。 */
export async function createTurnCredentials(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  if (!env.TURN_KEY_ID || !env.TURN_API_TOKEN) {
    throw new HttpError(503, 'turn_unavailable', 'TURN 服务尚未配置');
  }
  await env.DB.prepare(`DELETE FROM chat_turn_credentials WHERE expires_at <= ?`)
    .bind(nowMs())
    .run();
  const response = await fetch(
    `https://rtc.live.cloudflare.com/v1/turn/keys/${encodeURIComponent(env.TURN_KEY_ID)}/credentials/generate-ice-servers`,
    {
      method: 'POST',
      headers: {
        authorization: `Bearer ${env.TURN_API_TOKEN}`,
        'content-type': 'application/json',
      },
      body: JSON.stringify({ ttl: TURN_TTL_SECONDS }),
    },
  );
  if (!response.ok) {
    throw new HttpError(502, 'turn_credential_failed', 'TURN 凭证生成失败');
  }
  const payload = (await response.json()) as TurnResponse;
  const username = payload.iceServers?.find((server) => server.username)?.username;
  if (!username) {
    throw new HttpError(502, 'turn_credential_invalid', 'TURN 凭证响应不完整');
  }
  const createdAt = nowMs();
  await env.DB.prepare(
    `INSERT INTO chat_turn_credentials (owner_account, username, expires_at, created_at)
      VALUES (?, ?, ?, ?)`,
  )
    .bind(session.owner_account, username, createdAt + TURN_TTL_SECONDS * 1000, createdAt)
    .run();
  return jsonResponse({ ok: true, ice_servers: payload.iceServers ?? [], expires_at: createdAt + TURN_TTL_SECONDS * 1000 });
}

/** 注销时先撤销全部活动凭证，再删除本地索引。 */
export async function revokeOwnerTurn(env: Env, ownerAccount: string): Promise<number> {
  const rows = await env.DB.prepare(
    `SELECT username FROM chat_turn_credentials WHERE owner_account = ? AND expires_at > ?`,
  )
    .bind(ownerAccount, nowMs())
    .all<{ username: string }>();
  let revoked = 0;
  if (env.TURN_KEY_ID && env.TURN_API_TOKEN) {
    for (const row of rows.results ?? []) {
      const response = await fetch(
        `https://rtc.live.cloudflare.com/v1/turn/keys/${encodeURIComponent(env.TURN_KEY_ID)}/credentials/${encodeURIComponent(row.username)}/revoke`,
        { method: 'POST', headers: { authorization: `Bearer ${env.TURN_API_TOKEN}` } },
      );
      if (response.ok || response.status === 404) revoked += 1;
    }
  }
  await env.DB.prepare(`DELETE FROM chat_turn_credentials WHERE owner_account = ?`)
    .bind(ownerAccount)
    .run();
  return revoked;
}
