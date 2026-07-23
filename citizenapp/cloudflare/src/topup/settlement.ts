import type { Env } from '../types';
import { HttpError, jsonResponse, readJson } from '../shared/http';
import { nowMs } from '../shared/time';
import { isEvmTxHash, railRpcUrl, topupMinConfirmations, topupRail, topupRecvAddress } from './config';
import type { TopupToken } from './config';
import { findOrderById, statusLabel, type TopupOrderRow } from './orders';
import { verifyErc20Payment } from './evm_verify';

/// 结算接口(供本地部署控制台发币端调用):拉待发币队列、回写已支付/异常。
///
/// 鉴权:仅凭 TOPUP_SETTLE_TOKEN(只放 Worker Secret),非广场会话。控制台是四方之一;
/// Worker 回写「已支付」前会独立复核 EVM 到账仍成立(Cloudflare 角的四方对账)。

const PENDING_QUERY_LIMIT = 50;

interface SettledBody {
  gmb_tx_hash?: unknown;
}

interface ExceptionBody {
  reason?: unknown;
}

/// 常量时间比对 Bearer 令牌,校验失败一律 401(缺 secret 则 503)。
export function requireSettleAuth(request: Request, env: Env): void {
  const expected = env.TOPUP_SETTLE_TOKEN;
  if (!expected) {
    throw new HttpError(503, 'topup_settle_unconfigured', '结算接口未配置');
  }
  const authorization = request.headers.get('authorization') ?? '';
  const token = authorization.startsWith('Bearer ') ? authorization.slice('Bearer '.length).trim() : '';
  if (!token || !timingSafeEqual(token, expected)) {
    throw new HttpError(401, 'topup_settle_unauthorized', '结算令牌校验失败');
  }
}

/// GET /v1/square/topup/settlement/pending — 待发币队列(台账 pending,按到账时间升序)。
export async function topupPendingRoute(request: Request, env: Env): Promise<Response> {
  requireSettleAuth(request, env);
  const rows = await env.DB.prepare(
    `SELECT * FROM topup_orders WHERE status = 'pending' ORDER BY confirmed_at ASC LIMIT ?`
  )
    .bind(PENDING_QUERY_LIMIT)
    .all<TopupOrderRow>();
  return jsonResponse({
    ok: true,
    orders: (rows.results ?? []).map((row) => ({
      order_id: row.order_id,
      chain_id: row.chain_id,
      token: row.token,
      token_contract: row.token_contract,
      evm_tx_hash: row.evm_tx_hash,
      payer_address: row.payer_address,
      recv_address: row.recv_address,
      pay_amount: row.pay_amount,
      account_id: row.account_id,
      coin_fen: row.coin_fen,
      package_id: row.package_id,
      confirmed_at: row.confirmed_at,
    })),
  });
}

/// POST /v1/square/topup/settlement/:orderId/settled — 控制台发币完成回写。
/// Worker 侧独立复核 EVM 到账后才置「已支付」:
/// - 复核 confirmed → 置 paid(记 gmb_tx_hash),幂等。
/// - 复核 rejected(如 reorg 移除到账) → 真实不一致,置 exception。
/// - 复核 pending / RPC 抖动 → 不改状态,回 409 让控制台稍后重试(不误判异常)。
export async function topupSettledRoute(request: Request, env: Env, orderId: string): Promise<Response> {
  requireSettleAuth(request, env);
  const body = await readJson<SettledBody>(request);
  const gmbTxHash = typeof body.gmb_tx_hash === 'string' ? body.gmb_tx_hash.trim().toLowerCase() : '';
  if (!isEvmTxHash(gmbTxHash)) {
    throw new HttpError(400, 'topup_gmb_txhash_invalid', '公民币发币交易哈希不合法');
  }

  const order = await requireOrder(env, orderId);
  if (order.status === 'paid') {
    return jsonResponse({ ok: true, status: 'paid', status_label: statusLabel('paid'), order_id: orderId, deduplicated: true });
  }
  if (order.status !== 'pending') {
    throw new HttpError(409, 'topup_order_not_pending', '订单不处于待支付,无法结算');
  }

  const rail = topupRail(env, order.token as TopupToken);
  const outcome = await verifyErc20Payment({
    rail,
    rpcUrl: railRpcUrl(env, rail),
    txHash: order.evm_tx_hash,
    expectedRecv: topupRecvAddress(env),
    minAmount: BigInt(order.pay_amount),
    expectedPayer: order.payer_address ?? undefined,
    minConfirmations: topupMinConfirmations(env),
  });

  if (outcome.status === 'rejected') {
    await markException(env, orderId, `settle_recheck_rejected:${outcome.reason}`);
    throw new HttpError(409, 'topup_settle_recheck_rejected', '结算复核发现到账不一致,已置异常');
  }
  if (outcome.status === 'pending') {
    throw new HttpError(409, 'topup_settle_recheck_pending', '结算复核到账未确认,请稍后重试');
  }

  const updated = await env.DB.prepare(
    `UPDATE topup_orders SET status = 'paid', gmb_tx_hash = ?, settled_at = ?
      WHERE order_id = ? AND status = 'pending'`
  )
    .bind(gmbTxHash, nowMs(), orderId)
    .run();
  if ((updated.meta?.changes ?? 0) !== 1) {
    // 并发下已被结算:回读为准。
    const latest = await findOrderById(env, orderId);
    return jsonResponse({ ok: true, status: latest?.status ?? 'paid', status_label: statusLabel(latest?.status ?? 'paid'), order_id: orderId, deduplicated: true });
  }
  return jsonResponse({ ok: true, status: 'paid', status_label: statusLabel('paid'), order_id: orderId });
}

/// POST /v1/square/topup/settlement/:orderId/exception — 控制台报异常(发币失败/验不过),交人工。
export async function topupExceptionRoute(request: Request, env: Env, orderId: string): Promise<Response> {
  requireSettleAuth(request, env);
  const body = await readJson<ExceptionBody>(request);
  const reason = typeof body.reason === 'string' && body.reason.trim() !== '' ? body.reason.trim().slice(0, 200) : 'unspecified';

  const order = await requireOrder(env, orderId);
  if (order.status === 'exception') {
    return jsonResponse({ ok: true, status: 'exception', status_label: statusLabel('exception'), order_id: orderId, deduplicated: true });
  }
  if (order.status !== 'pending') {
    throw new HttpError(409, 'topup_order_not_pending', '订单不处于待支付,无法置异常');
  }
  await markException(env, orderId, reason);
  return jsonResponse({ ok: true, status: 'exception', status_label: statusLabel('exception'), order_id: orderId });
}

async function markException(env: Env, orderId: string, reason: string): Promise<void> {
  await env.DB.prepare(
    `UPDATE topup_orders SET status = 'exception', exception_reason = ?, settled_at = ?
      WHERE order_id = ? AND status = 'pending'`
  )
    .bind(reason, nowMs(), orderId)
    .run();
}

async function requireOrder(env: Env, orderId: string): Promise<TopupOrderRow> {
  const order = await findOrderById(env, orderId);
  if (!order) {
    throw new HttpError(404, 'topup_order_not_found', '充值订单不存在');
  }
  return order;
}

function timingSafeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let index = 0; index < a.length; index += 1) {
    diff |= a.charCodeAt(index) ^ b.charCodeAt(index);
  }
  return diff === 0;
}
