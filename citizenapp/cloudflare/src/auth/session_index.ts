import type { Env } from '../types';
import { putKvJson } from '../limits/storage';
import { resourceLimit } from '../limits/catalog';

/// 会话按 token 为键存 KV（square_session:{token}），无法按账户枚举。
/// 这里额外维护「账户 → token 列表」索引，使注销时能定向失效该账户全部会话，
/// 不必等 token TTL 自然过期，满足「零残留」。
function accountSessionsKey(accountId: string): string {
  return `square_sessions_by_account_id:${accountId}`;
}

/// 登录成功后把新 token 记入账户索引。TTL 取至少一个会话周期，随每次登录续期。
/// 注：KV 读改写非原子，极端并发下个别 token 可能漏记 → 该 token 仍由自身 TTL 兜底过期。
export async function indexSessionToken(
  env: Env,
  accountId: string,
  token: string,
  sessionTtlSeconds: number
): Promise<void> {
  const key = accountSessionsKey(accountId);
  const existing = await env.SQUARE_CACHE.get(key);
  const tokens: string[] = existing ? (JSON.parse(existing) as string[]) : [];
  if (!tokens.includes(token)) {
    tokens.push(token);
  }
  const kept = tokens.slice(-(resourceLimit('session_index').max_count ?? 1));
  await putKvJson(env, key, kept, 'session_index', {
    expirationTtl: Math.max(sessionTtlSeconds, 3600)
  });
}

/// 注销时清空该账户全部会话 token 及索引本身。
export async function clearAccountSessions(env: Env, accountId: string): Promise<void> {
  const key = accountSessionsKey(accountId);
  const existing = await env.SQUARE_CACHE.get(key);
  if (existing) {
    const tokens = JSON.parse(existing) as string[];
    for (const token of tokens) {
      await env.SQUARE_CACHE.delete(`square_session:${token}`);
    }
  }
  await env.SQUARE_CACHE.delete(key);
}
