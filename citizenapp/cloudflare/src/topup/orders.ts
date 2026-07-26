import type { Env } from '../types';
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import { createId } from '../shared/ids';
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
  type TopupToken,
} from './config';
import { verifyErc20Payment } from './evm_verify';

/// 充值订单三态台账(仅此三种):
/// pending=稳定币已确认、公民币待发 / paid=公民币已发 / exception=异常、交人工。
export type TopupOrderStatus = 'pending' | 'paid' | 'exception';

export interface TopupOrderRow {
  order_id: string;
  intent_id: string;
  chain_id: number;
  token: string;
  token_contract: string;
  evm_tx_hash: string;
  payer_address: string;
  recv_address: string;
  pay_amount: string;
  account_id: string;
  coin_fen: string;
  package_id: string;
  status: TopupOrderStatus;
  settlement_claim_id: string | null;
  settlement_claimed_at: number | null;
  gmb_tx_hash: string | null;
  gmb_block_hash: string | null;
  gmb_extrinsic_index: number | null;
  exception_reason: string | null;
  confirmed_at: number;
  settled_at: number | null;
}

interface IntentBody {
  token?: unknown;
  package_id?: unknown;
  payer_address?: unknown;
}

interface ConfirmBody {
  payment_intent?: unknown;
  evm_tx_hash?: unknown;
}

interface PaymentIntent {
  intent_id: string;
  account_id: string;
  payer_address: string;
  token: TopupToken;
  package_id: string;
  chain_id: number;
  token_contract: string;
  recv_address: string;
  pay_amount: string;
  coin_fen: string;
  issued_at: number;
  expires_at: number;
}

const INTENT_TTL_MS = 10 * 60 * 1000;

/// 台账状态 → 用户可读中文标签。
export function statusLabel(status: TopupOrderStatus): string {
  return status === 'pending' ? '待支付' : status === 'paid' ? '已支付' : '异常';
}

/// GET /v1/square/topup/config — 公开报价。付款前的最终报价以后续签名意图为准。
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

/// POST /v1/square/topup/intent — 钱包连接后、付款前创建短期付款意图。
/// account_id 只取 Bearer 会话；客户端不能在请求体中指定或更换充值目标。
export async function topupIntentRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<IntentBody>(request);
  if (!isTopupToken(body.token)) {
    throw new HttpError(400, 'topup_token_invalid', '不支持的充值币种');
  }
  const packageId = typeof body.package_id === 'string' ? body.package_id : '';
  const pkg = findPackage(packageId);
  if (!pkg) {
    throw new HttpError(400, 'topup_package_invalid', '充值套餐不存在');
  }
  const payerAddress = normalizeAddress(body.payer_address);
  const rail = topupRail(env, body.token);
  const issuedAt = nowMs();
  const intent: PaymentIntent = {
    intent_id: createId('tpi'),
    account_id: session.account_id,
    payer_address: payerAddress,
    token: rail.token,
    package_id: pkg.package_id,
    chain_id: rail.chain_id,
    token_contract: rail.token_contract,
    recv_address: topupRecvAddress(env),
    pay_amount: pkg.pay_amount,
    coin_fen: pkg.coin_fen,
    issued_at: issuedAt,
    expires_at: issuedAt + INTENT_TTL_MS,
  };
  return jsonResponse({
    ok: true,
    payment_intent: await signIntent(env, intent),
    expires_at: intent.expires_at,
  });
}

/// POST /v1/square/topup/confirm — 付款后提交交易哈希。
/// Worker 同时核验会话、HMAC 意图和最终 EVM 事实后才创建三态订单。
export async function topupConfirmRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<ConfirmBody>(request);
  const encodedIntent = typeof body.payment_intent === 'string' ? body.payment_intent.trim() : '';
  const intent = await verifyIntent(env, encodedIntent);
  if (intent.account_id !== session.account_id) {
    throw new HttpError(403, 'topup_intent_account_mismatch', '付款意图不属于当前钱包');
  }
  if (intent.expires_at <= nowMs()) {
    throw new HttpError(409, 'topup_intent_expired', '付款意图已过期，请重新发起');
  }
  assertIntentStillMatchesConfig(env, intent);

  const txHash = typeof body.evm_tx_hash === 'string' ? body.evm_tx_hash.trim().toLowerCase() : '';
  if (!isEvmTxHash(txHash)) {
    throw new HttpError(400, 'topup_txhash_invalid', 'EVM 交易哈希不合法');
  }

  const existing = await findOrderByTx(env, intent.chain_id, txHash);
  if (existing) {
    if (existing.intent_id !== intent.intent_id || existing.account_id !== session.account_id) {
      throw new HttpError(409, 'topup_txhash_claimed', '该链上付款已绑定其它付款意图');
    }
    return orderResponse(existing, true);
  }

  const rail = topupRail(env, intent.token);
  const outcome = await verifyErc20Payment({
    rail,
    rpcUrl: railRpcUrl(env, rail),
    txHash,
    expectedRecv: intent.recv_address,
    minAmount: BigInt(intent.pay_amount),
    expectedPayer: intent.payer_address,
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
      (order_id, intent_id, chain_id, token, token_contract, evm_tx_hash, payer_address,
       recv_address, pay_amount, account_id, coin_fen, package_id, status, confirmed_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'pending', ?)`,
  )
    .bind(
      orderId,
      intent.intent_id,
      intent.chain_id,
      intent.token,
      intent.token_contract,
      txHash,
      outcome.payer,
      intent.recv_address,
      intent.pay_amount,
      intent.account_id,
      intent.coin_fen,
      intent.package_id,
      nowMs(),
    )
    .run();

  if ((inserted.meta?.changes ?? 0) !== 1) {
    const raced = await findOrderByTx(env, intent.chain_id, txHash)
      ?? await findOrderByIntent(env, intent.intent_id);
    if (raced && raced.intent_id === intent.intent_id && raced.account_id === session.account_id) {
      return orderResponse(raced, true);
    }
    throw new HttpError(409, 'topup_payment_already_bound', '付款或付款意图已被处理');
  }
  return jsonResponse({
    ok: true,
    status: 'pending',
    status_label: statusLabel('pending'),
    order_id: orderId,
  });
}

/// GET /v1/square/topup/status/:orderId — 只允许订单所属账户查询。
export async function topupStatusRoute(
  request: Request,
  env: Env,
  orderId: string,
): Promise<Response> {
  const session = await requireSession(request, env);
  if (!/^top_[0-9a-f]{32}$/.test(orderId)) {
    throw new HttpError(400, 'topup_order_id_invalid', '充值订单 ID 不合法');
  }
  const order = await findOrderById(env, orderId);
  if (!order || order.account_id !== session.account_id) {
    throw new HttpError(404, 'topup_order_not_found', '充值订单不存在');
  }
  return orderResponse(order, false);
}

export async function findOrderByTx(
  env: Env,
  chainId: number,
  txHash: string,
): Promise<TopupOrderRow | null> {
  return env.DB.prepare('SELECT * FROM topup_orders WHERE chain_id = ? AND evm_tx_hash = ?')
    .bind(chainId, txHash)
    .first<TopupOrderRow>();
}

export async function findOrderByIntent(env: Env, intentId: string): Promise<TopupOrderRow | null> {
  return env.DB.prepare('SELECT * FROM topup_orders WHERE intent_id = ?')
    .bind(intentId)
    .first<TopupOrderRow>();
}

export async function findOrderById(env: Env, orderId: string): Promise<TopupOrderRow | null> {
  return env.DB.prepare('SELECT * FROM topup_orders WHERE order_id = ?')
    .bind(orderId)
    .first<TopupOrderRow>();
}

function orderResponse(order: TopupOrderRow, deduplicated: boolean): Response {
  return jsonResponse({
    ok: true,
    status: order.status,
    status_label: statusLabel(order.status),
    order_id: order.order_id,
    gmb_tx_hash: order.gmb_tx_hash,
    coin_fen: order.coin_fen,
    ...(deduplicated ? { deduplicated: true } : {}),
  });
}

function normalizeAddress(value: unknown): string {
  const address = typeof value === 'string' ? value.trim().toLowerCase() : '';
  if (!isEvmAddress(address)) {
    throw new HttpError(400, 'topup_payer_invalid', '付款地址不合法');
  }
  return address;
}

function assertIntentStillMatchesConfig(env: Env, intent: PaymentIntent): void {
  const rail = topupRail(env, intent.token);
  const pkg = findPackage(intent.package_id);
  if (
    !pkg ||
    rail.chain_id !== intent.chain_id ||
    rail.token_contract !== intent.token_contract ||
    topupRecvAddress(env) !== intent.recv_address ||
    pkg.pay_amount !== intent.pay_amount ||
    pkg.coin_fen !== intent.coin_fen
  ) {
    throw new HttpError(409, 'topup_intent_config_changed', '充值配置已变化，请重新发起');
  }
}

async function signIntent(env: Env, intent: PaymentIntent): Promise<string> {
  const secret = requireIntentSecret(env);
  const payload = base64UrlEncode(new TextEncoder().encode(JSON.stringify(intent)));
  const signature = await crypto.subtle.sign('HMAC', await importIntentKey(secret), new TextEncoder().encode(payload));
  return `${payload}.${base64UrlEncode(new Uint8Array(signature))}`;
}

async function verifyIntent(env: Env, token: string): Promise<PaymentIntent> {
  const parts = token.split('.');
  if (parts.length !== 2 || !parts[0] || !parts[1]) {
    throw new HttpError(400, 'topup_intent_invalid', '付款意图不合法');
  }
  let signature: Uint8Array;
  let payloadBytes: Uint8Array;
  try {
    signature = base64UrlDecode(parts[1]);
    payloadBytes = base64UrlDecode(parts[0]);
  } catch {
    throw new HttpError(400, 'topup_intent_invalid', '付款意图不合法');
  }
  const valid = await crypto.subtle.verify(
    'HMAC',
    await importIntentKey(requireIntentSecret(env)),
    Uint8Array.from(signature),
    new TextEncoder().encode(parts[0]),
  );
  if (!valid) {
    throw new HttpError(400, 'topup_intent_invalid', '付款意图签名不合法');
  }
  let decoded: unknown;
  try {
    decoded = JSON.parse(new TextDecoder().decode(payloadBytes));
  } catch {
    throw new HttpError(400, 'topup_intent_invalid', '付款意图内容不合法');
  }
  if (!isPaymentIntent(decoded)) {
    throw new HttpError(400, 'topup_intent_invalid', '付款意图内容不合法');
  }
  return decoded;
}

function requireIntentSecret(env: Env): string {
  const secret = env.TOPUP_INTENT_SECRET?.trim();
  if (!secret || secret.length < 32) {
    throw new HttpError(503, 'topup_intent_unconfigured', '充值付款意图密钥未配置');
  }
  return secret;
}

function importIntentKey(secret: string): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    'raw',
    new TextEncoder().encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign', 'verify'],
  );
}

function isPaymentIntent(value: unknown): value is PaymentIntent {
  if (!value || typeof value !== 'object') return false;
  const intent = value as Record<string, unknown>;
  return (
    typeof intent.intent_id === 'string' &&
    /^tpi_[0-9a-f]{32}$/.test(intent.intent_id) &&
    typeof intent.account_id === 'string' &&
    /^0x[0-9a-f]{64}$/.test(intent.account_id) &&
    typeof intent.payer_address === 'string' &&
    isEvmAddress(intent.payer_address) &&
    isTopupToken(intent.token) &&
    typeof intent.package_id === 'string' &&
    typeof intent.chain_id === 'number' &&
    Number.isSafeInteger(intent.chain_id) &&
    typeof intent.token_contract === 'string' &&
    isEvmAddress(intent.token_contract) &&
    typeof intent.recv_address === 'string' &&
    isEvmAddress(intent.recv_address) &&
    typeof intent.pay_amount === 'string' &&
    /^\d+$/.test(intent.pay_amount) &&
    typeof intent.coin_fen === 'string' &&
    /^\d+$/.test(intent.coin_fen) &&
    typeof intent.issued_at === 'number' &&
    Number.isSafeInteger(intent.issued_at) &&
    typeof intent.expires_at === 'number' &&
    Number.isSafeInteger(intent.expires_at)
  );
}

function base64UrlEncode(bytes: Uint8Array): string {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/, '');
}

function base64UrlDecode(value: string): Uint8Array {
  if (!/^[A-Za-z0-9_-]+$/.test(value)) throw new Error('invalid base64url');
  const padded = value.replaceAll('-', '+').replaceAll('_', '/') + '='.repeat((4 - value.length % 4) % 4);
  const binary = atob(padded);
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}
