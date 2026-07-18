import type { Env } from '../types';
import { HttpError, jsonResponse } from '../shared/http';
import { readJson } from '../shared/http';
import { createId, assertOwnerAccount } from '../shared/ids';
import { nowMs } from '../shared/time';
import {
  findPackage,
  isEvmAddress,
  isEvmTxHash,
  isTopupToken,
  railRpcUrl,
  topupMinConfirmations,
  topupNetwork,
  topupPackages,
  topupRail,
  topupRails,
  topupRecvAddress,
} from './config';
import { verifyErc20Payment } from './evm_verify';

/// 充值订单三态台账(仅此三种,与用户口径一一对应):
/// pending=待支付(已收稳定币未发币) / paid=已支付(成功) / exception=异常(失败,交人工)。
/// 未支付订单不入表(无第四态);'confirming'/'not_found' 只是轮询过渡响应,不落库。
export type TopupOrderStatus = 'pending' | 'paid' | 'exception';

export interface TopupOrderRow {
  order_id: string;
  chain_id: number;
  token: string;
  token_contract: string;
  evm_tx_hash: string;
  payer_address: string | null;
  recv_address: string;
  pay_amount: string;
  gmb_address: string;
  coin_fen: string;
  package_id: string;
  status: TopupOrderStatus;
  gmb_tx_hash: string | null;
  exception_reason: string | null;
  confirmed_at: number;
  settled_at: number | null;
}

/// 台账状态 → 用户可读中文标签。
export function statusLabel(status: TopupOrderStatus): string {
  return status === 'pending' ? '待支付' : status === 'paid' ? '已支付' : '异常';
}

interface SubmitBody {
  token?: unknown;
  package_id?: unknown;
  gmb_address?: unknown;
  evm_tx_hash?: unknown;
  payer_address?: unknown;
}

/// GET /v1/square/topup/config — 可购币轨 + 套餐 + 收款地址(配置驱动,公开只读)。
export async function topupConfigRoute(_request: Request, env: Env): Promise<Response> {
  const rails = topupRails(env);
  if (rails.length === 0) {
    throw new HttpError(503, 'topup_unconfigured', '充值渠道尚未配置');
  }
  const recvAddress = topupRecvAddress(env);
  return jsonResponse({
    ok: true,
    network: topupNetwork(env),
    recv_address: recvAddress,
    rails: rails.map((rail) => ({
      token: rail.token,
      chain_id: rail.chain_id,
      token_contract: rail.token_contract,
      token_decimals: rail.token_decimals,
      label: rail.label,
    })),
    packages: topupPackages(),
  });
}

/// POST /v1/square/topup/submit — App 付款后上报 txHash。
/// 幂等:同一 (chain_id, evm_tx_hash) 已存在则直接回其当前状态,不重复验证/入账。
/// 验证结果:confirmed→落「待支付」入队;pending→'confirming'(不落库);rejected→拒绝。
export async function topupSubmitRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<SubmitBody>(request);

  if (!isTopupToken(body.token)) {
    throw new HttpError(400, 'topup_token_invalid', '不支持的充值币种');
  }
  const packageId = typeof body.package_id === 'string' ? body.package_id : '';
  const pkg = findPackage(packageId);
  if (!pkg) {
    throw new HttpError(400, 'topup_package_invalid', '充值套餐不存在');
  }
  const gmbAddress = assertGmbAddress(body.gmb_address);
  const txHash = typeof body.evm_tx_hash === 'string' ? body.evm_tx_hash.trim().toLowerCase() : '';
  if (!isEvmTxHash(txHash)) {
    throw new HttpError(400, 'topup_txhash_invalid', 'EVM 交易哈希不合法');
  }
  const payer = normalizeOptionalAddress(body.payer_address);

  const rail = topupRail(env, body.token);

  // 幂等前置:同链同 txHash 已入账 → 直接回状态(校验收款人一致,防冒领他人付款)。
  const existing = await findOrderByTx(env, rail.chain_id, txHash);
  if (existing) {
    if (existing.gmb_address !== gmbAddress) {
      throw new HttpError(409, 'topup_txhash_claimed', '该链上付款已绑定其它钱包');
    }
    return jsonResponse({ ok: true, status: existing.status, status_label: statusLabel(existing.status), order_id: existing.order_id });
  }

  const outcome = await verifyErc20Payment({
    rail,
    rpcUrl: railRpcUrl(env, rail),
    txHash,
    expectedRecv: topupRecvAddress(env),
    minAmount: BigInt(pkg.pay_amount),
    expectedPayer: payer ?? undefined,
    minConfirmations: topupMinConfirmations(env),
  });

  if (outcome.status === 'pending') {
    return jsonResponse({ ok: true, status: 'confirming' });
  }
  if (outcome.status === 'rejected') {
    throw new HttpError(400, 'topup_payment_invalid', `未确认到有效到账:${outcome.reason}`);
  }

  const orderId = createId('top');
  const inserted = await env.DB.prepare(
    `INSERT OR IGNORE INTO topup_orders
      (order_id, chain_id, token, token_contract, evm_tx_hash, payer_address, recv_address,
       pay_amount, gmb_address, coin_fen, package_id, status, confirmed_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'pending', ?)`
  )
    .bind(
      orderId,
      rail.chain_id,
      rail.token,
      rail.token_contract,
      txHash,
      outcome.payer,
      topupRecvAddress(env),
      pkg.pay_amount,
      gmbAddress,
      pkg.coin_fen,
      pkg.package_id,
      nowMs(),
    )
    .run();

  // 并发下另一个请求可能已抢先写入:回读为准,保证幂等。
  if ((inserted.meta?.changes ?? 0) !== 1) {
    const raced = await findOrderByTx(env, rail.chain_id, txHash);
    if (raced) {
      return jsonResponse({ ok: true, status: raced.status, status_label: statusLabel(raced.status), order_id: raced.order_id });
    }
  }

  return jsonResponse({ ok: true, status: 'pending', status_label: statusLabel('pending'), order_id: orderId });
}

/// GET /v1/square/topup/status?chain_id=&evm_tx_hash= — 查台账状态。
/// 未入账返回 'not_found'(过渡响应,非业务态)。
export async function topupStatusRoute(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url);
  const chainId = Number.parseInt(url.searchParams.get('chain_id') ?? '', 10);
  const txHash = (url.searchParams.get('evm_tx_hash') ?? '').trim().toLowerCase();
  if (!Number.isFinite(chainId) || chainId <= 0 || !isEvmTxHash(txHash)) {
    throw new HttpError(400, 'topup_query_invalid', '查询参数不合法');
  }
  const order = await findOrderByTx(env, chainId, txHash);
  if (!order) {
    return jsonResponse({ ok: true, status: 'not_found' });
  }
  return jsonResponse({
    ok: true,
    status: order.status,
    status_label: statusLabel(order.status),
    order_id: order.order_id,
    gmb_tx_hash: order.gmb_tx_hash,
    coin_fen: order.coin_fen,
  });
}

export async function findOrderByTx(
  env: Env,
  chainId: number,
  txHash: string,
): Promise<TopupOrderRow | null> {
  return env.DB.prepare(
    'SELECT * FROM topup_orders WHERE chain_id = ? AND evm_tx_hash = ?'
  )
    .bind(chainId, txHash)
    .first<TopupOrderRow>();
}

export async function findOrderById(env: Env, orderId: string): Promise<TopupOrderRow | null> {
  return env.DB.prepare('SELECT * FROM topup_orders WHERE order_id = ?')
    .bind(orderId)
    .first<TopupOrderRow>();
}

function assertGmbAddress(value: unknown): string {
  try {
    return assertOwnerAccount(value);
  } catch {
    throw new HttpError(400, 'topup_gmb_address_invalid', '公民链钱包地址不合法');
  }
}

function normalizeOptionalAddress(value: unknown): string | null {
  if (typeof value !== 'string' || value.trim() === '') return null;
  const address = value.trim().toLowerCase();
  if (!isEvmAddress(address)) {
    throw new HttpError(400, 'topup_payer_invalid', '付款地址不合法');
  }
  return address;
}
