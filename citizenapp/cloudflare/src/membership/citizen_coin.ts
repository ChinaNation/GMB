import type { Env } from "../types";
import { HttpError, jsonResponse, readJson, requireSession } from "../shared/http";
import { sha256Hex } from "../shared/hash";
import { nowMs } from "../shared/time";
import {
  bindFinalizedTransactionConfirmation,
  readSubscriptionAtBlock,
  updateChainClock,
  verifyFinalizedSubscriptionTransaction,
  type ChainSubscriptionState,
  type FinalizedTransactionProofInput,
  type PlatformLevel,
  type SubscriptionBusinessAction,
  type VerifiedFinalizedTransaction,
} from "../chain/subscription";
import { assertMembershipLevel } from "./plans";

/**
 * 平台订阅 BFF 只接收 CitizenApp 已完成的一次账户签名交易证明。Worker 校验完整交易属于
 * 当前钱包、位于 finalized 主链、调用参数与动作一致，再读取同一区块订阅状态并镜像。
 * 任何 HTTP 重试都只使用 Bearer 会话，不产生第二次账户或设备签名。
 */

type PlatformAction = "subscribe" | "cancel" | "change";

interface PlatformConfirmBody {
  tx_hash?: unknown;
  block_hash?: unknown;
  signed_extrinsic_hex?: unknown;
  action?: unknown;
  membership_level?: unknown;
}

export interface PlatformSubscriptionConfirmDeps {
  verifyTransaction: (
    env: Env,
    ownerAccount: string,
    expectedAction: SubscriptionBusinessAction,
    proof: FinalizedTransactionProofInput,
  ) => Promise<VerifiedFinalizedTransaction>;
  readSubscriptionAtBlock: (
    env: Env,
    ownerAccount: string,
    blockHash: string,
  ) => Promise<ChainSubscriptionState | null>;
}

const defaultConfirmDeps: PlatformSubscriptionConfirmDeps = {
  verifyTransaction: verifyFinalizedSubscriptionTransaction,
  readSubscriptionAtBlock: (env, ownerAccount, blockHash) =>
    readSubscriptionAtBlock(env, ownerAccount, { kind: "platform" }, blockHash),
};

/** POST /v1/square/membership/confirm —— finalized 平台订阅镜像（严格幂等）。 */
export async function platformSubscriptionConfirmRoute(
  request: Request,
  env: Env,
  deps: PlatformSubscriptionConfirmDeps = defaultConfirmDeps,
): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<PlatformConfirmBody>(request);
  const action = platformAction(body.action);
  const membershipLevel =
    action === "cancel" ? null : assertMembershipLevel(body.membership_level);
  const expectedAction = expectedPlatformAction(action, membershipLevel);
  const proof = transactionProof(body);
  const transaction = await deps.verifyTransaction(
    env,
    session.owner_account,
    expectedAction,
    proof,
  );
  const state = await deps.readSubscriptionAtBlock(
    env,
    session.owner_account,
    transaction.blockHash,
  );
  assertPlatformStateMatches(state, action, membershipLevel);

  const confirmedAt = nowMs();
  const requestHash = await sha256Hex(
    JSON.stringify({ action, membership_level: membershipLevel }),
  );
  await bindFinalizedTransactionConfirmation(
    env,
    session.owner_account,
    transaction,
    requestHash,
    confirmedAt,
  );
  await updateChainClock(env, {
    chainTimestamp: transaction.chainTimestamp,
    blockNumber: transaction.blockNumber,
    blockHash: transaction.blockHash,
    observedAt: confirmedAt,
  });
  await mirrorPlatformState(env, session.owner_account, state!, transaction, confirmedAt);
  return jsonResponse({
    ok: true,
    subscription_status: state!.status,
    membership_level: state!.plan.kind === "platform" ? state!.plan.membershipLevel : null,
    paid_until: state!.paidUntil,
  });
}

function platformAction(value: unknown): PlatformAction {
  if (value === "subscribe" || value === "cancel" || value === "change") return value;
  throw new HttpError(400, "invalid_subscription_action", "平台订阅操作不合法");
}

function expectedPlatformAction(
  action: PlatformAction,
  membershipLevel: PlatformLevel | null,
): SubscriptionBusinessAction {
  if (action === "cancel") return { kind: "platform_cancel" };
  if (!membershipLevel) throw new HttpError(400, "invalid_request", "平台会员档位缺失");
  return action === "subscribe"
    ? { kind: "platform_subscribe", membershipLevel }
    : { kind: "platform_change", membershipLevel };
}

function transactionProof(body: PlatformConfirmBody): FinalizedTransactionProofInput {
  if (
    typeof body.tx_hash !== "string" ||
    typeof body.block_hash !== "string" ||
    typeof body.signed_extrinsic_hex !== "string"
  ) {
    throw new HttpError(400, "invalid_transaction_proof", "finalized 交易证明不完整");
  }
  return {
    txHash: body.tx_hash,
    blockHash: body.block_hash,
    signedExtrinsicHex: body.signed_extrinsic_hex,
  };
}

function assertPlatformStateMatches(
  state: ChainSubscriptionState | null,
  action: PlatformAction,
  requestedLevel: PlatformLevel | null,
): void {
  if (state === null || state.plan.kind !== "platform") {
    throw new HttpError(409, "subscription_state_not_finalized", "链上平台订阅状态尚未最终确认");
  }
  if (action === "cancel") {
    if (state.status !== "cancelled") {
      throw new HttpError(409, "subscription_state_not_finalized", "链上取消状态尚未最终确认");
    }
    return;
  }
  // 换挡即时生效（无 pending）：确认后链上 plan 已是目标档。
  if (
    state.status !== "active" ||
    state.plan.membershipLevel !== requestedLevel
  ) {
    throw new HttpError(409, "subscription_state_not_finalized", "链上平台订阅或换档状态尚未最终确认");
  }
}

async function mirrorPlatformState(
  env: Env,
  ownerAccount: string,
  state: ChainSubscriptionState,
  transaction: VerifiedFinalizedTransaction,
  verifiedAt: number,
): Promise<void> {
  if (state.plan.kind !== "platform") {
    throw new HttpError(409, "subscription_state_not_finalized", "链上平台订阅计划不合法");
  }
  const lastChargedPriceFen = safePrice(state.lastChargedPriceFen);
  const entitlementLapsedAt = state.status === "active" ? null : state.paidUntil;
  await env.DB.prepare(
    `INSERT INTO square_memberships
      (owner_account, membership_level, started_at,
       last_charged_at, last_charged_price_fen, paid_until, subscription_status,
       finalized_block_number, finalized_block_hash, verified_at,
       entitlement_lapsed_at, last_tx_hash)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      ON CONFLICT(owner_account) DO UPDATE SET
        membership_level = excluded.membership_level,
        started_at = excluded.started_at,
        last_charged_at = excluded.last_charged_at,
        last_charged_price_fen = excluded.last_charged_price_fen,
        paid_until = excluded.paid_until,
        subscription_status = excluded.subscription_status,
        finalized_block_number = excluded.finalized_block_number,
        finalized_block_hash = excluded.finalized_block_hash,
        verified_at = excluded.verified_at,
        entitlement_lapsed_at = excluded.entitlement_lapsed_at,
        last_tx_hash = excluded.last_tx_hash
      WHERE excluded.finalized_block_number >= square_memberships.finalized_block_number`,
  )
    .bind(
      ownerAccount,
      state.plan.membershipLevel,
      state.startedAt,
      state.lastChargedAt,
      lastChargedPriceFen,
      state.paidUntil,
      state.status,
      transaction.blockNumber,
      transaction.blockHash,
      verifiedAt,
      entitlementLapsedAt,
      transaction.txHash,
    )
    .run();
}

function safePrice(value: bigint): number {
  if (value <= 0n || value > BigInt(Number.MAX_SAFE_INTEGER)) {
    throw new HttpError(502, "subscription_price_out_of_range", "链上订阅价格超出边缘服务范围");
  }
  return Number(value);
}
