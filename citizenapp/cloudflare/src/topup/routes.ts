import type { Env } from '../types';
import { HttpError } from '../shared/http';
import { topupConfigRoute, topupStatusRoute, topupSubmitRoute } from './orders';
import { topupExceptionRoute, topupPendingRoute, topupSettledRoute } from './settlement';

/// 稳定币充值(topup)子路由分派。挂在 `/v1/square/topup/` 前缀下。
/// App 端:config / submit / status(无广场会话,正确性来自链上真实到账)。
/// 控制台端:settlement/*(TOPUP_SETTLE_TOKEN 鉴权)。
const SETTLEMENT_PREFIX = '/v1/square/topup/settlement/';

export function isTopupPath(path: string): boolean {
  return path === '/v1/square/topup/config' || path.startsWith('/v1/square/topup/');
}

export async function routeTopup(request: Request, env: Env, path: string): Promise<Response> {
  if (request.method === 'GET' && path === '/v1/square/topup/config') {
    return topupConfigRoute(request, env);
  }
  if (request.method === 'POST' && path === '/v1/square/topup/submit') {
    return topupSubmitRoute(request, env);
  }
  if (request.method === 'GET' && path === '/v1/square/topup/status') {
    return topupStatusRoute(request, env);
  }
  if (request.method === 'GET' && path === '/v1/square/topup/settlement/pending') {
    return topupPendingRoute(request, env);
  }
  if (request.method === 'POST' && path.startsWith(SETTLEMENT_PREFIX) && path.endsWith('/settled')) {
    const orderId = path.slice(SETTLEMENT_PREFIX.length, -'/settled'.length);
    return topupSettledRoute(request, env, orderId);
  }
  if (request.method === 'POST' && path.startsWith(SETTLEMENT_PREFIX) && path.endsWith('/exception')) {
    const orderId = path.slice(SETTLEMENT_PREFIX.length, -'/exception'.length);
    return topupExceptionRoute(request, env, orderId);
  }
  throw new HttpError(404, 'route_not_found', '充值接口不存在');
}
