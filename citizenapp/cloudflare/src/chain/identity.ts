import { bytesToHex, hexToBytes } from "../shared/signing_message";
import type { Env } from "../types";
import { decodeAccountId, storageMapKey } from "./storage_key";
import { nowMs } from "../shared/time";
import { putKvJson } from "../limits/storage";
import { fetchChainStorage, fetchFinalizedHead } from "./rpc";

/// 链上身份档位（电子护照真源，与会员档位彻底解耦，ADR-036）：
/// visitor 未认证 / voting 认证投票公民 / candidate 认证竞选公民。
export type IdentityLevel = "visitor" | "voting" | "candidate";

export interface ChainIdentityState {
  account_id: string;
  identity_level: IdentityLevel;
  has_voting_identity: boolean;
  has_candidate_identity: boolean;
  cid_number: string | null;
  checked_at: number;
}

interface VotingIdentity {
  passport_valid_from: number;
  passport_valid_until: number;
  citizen_status: "normal" | "revoked";
}

interface CandidateIdentity {
  birth_date: number;
}

/// 身份读取的 KV 短缓存 TTL（秒）。护照有效期按「日」判定，当天内结果稳定。
const IDENTITY_CACHE_TTL_SECONDS = 45;

function visitorIdentityState(accountId: string): ChainIdentityState {
  return {
    account_id: accountId,
    identity_level: "visitor",
    has_voting_identity: false,
    has_candidate_identity: false,
    cid_number: null,
    checked_at: nowMs(),
  };
}

/// 带 KV 短缓存 + 失败软降级的身份读取。
///
/// 主页/feed 等展示路径用它，绝不因链上 RPC 未配置/超时/失败而阻塞渲染：
/// 命中缓存直接返回；未命中读链并回写 KV；读链失败软降级为访客（未认证），不抛错。
export async function fetchChainIdentityStateCached(
  env: Env,
  accountId: string,
): Promise<ChainIdentityState> {
  const cacheKey = `square_identity:${accountId}`;
  try {
    const cached = await env.SQUARE_CACHE.get(cacheKey);
    if (cached) {
      return JSON.parse(cached) as ChainIdentityState;
    }
  } catch {
    // 缓存读失败忽略，继续读链。
  }
  try {
    const state = await fetchChainIdentityState(env, accountId);
    try {
      await putKvJson(env, cacheKey, state, "identity_cache", {
        expirationTtl: IDENTITY_CACHE_TTL_SECONDS,
      });
    } catch {
      // 缓存写失败忽略。
    }
    return state;
  } catch {
    // 链上 RPC 未配置/超时/失败：软降级为访客，展示未认证，不阻塞主页。
    return visitorIdentityState(accountId);
  }
}

export async function fetchChainIdentityState(
  env: Env,
  accountId: string,
): Promise<ChainIdentityState> {
  const accountIdBytes = decodeAccountId(accountId);
  // 同一次身份判断的五项 storage 必须锚定同一个 finalized 区块，禁止混读 best head。
  const finalizedHead = await fetchFinalizedHead(env);
  const cidByWalletKey = storageMapKey(
    "CitizenIdentity",
    "CidByWalletAccount",
    accountIdBytes,
  );
  const cidHex = await fetchChainStorage(
    env,
    `0x${bytesToHex(cidByWalletKey)}`,
    finalizedHead,
  );
  const cidNumber = cidHex ? decodeCidNumber(hexToBytes(cidHex)) : null;
  if (!cidNumber) return visitorIdentityState(accountId);

  const cidScale = encodeBoundedBytes(new TextEncoder().encode(cidNumber));
  const walletByCidKey = storageMapKey(
    "CitizenIdentity",
    "WalletAccountByCid",
    cidScale,
  );
  const cidRegistryKey = storageMapKey(
    "CitizenIdentity",
    "CidRegistry",
    cidScale,
  );
  const votingKey = storageMapKey(
    "CitizenIdentity",
    "VotingIdentityByCid",
    cidScale,
  );
  const candidateKey = storageMapKey(
    "CitizenIdentity",
    "CandidateIdentityByCid",
    cidScale,
  );
  const [walletHex, cidRecordHex, votingHex, candidateHex] = await Promise.all([
    fetchChainStorage(env, `0x${bytesToHex(walletByCidKey)}`, finalizedHead),
    fetchChainStorage(env, `0x${bytesToHex(cidRegistryKey)}`, finalizedHead),
    fetchChainStorage(env, `0x${bytesToHex(votingKey)}`, finalizedHead),
    fetchChainStorage(env, `0x${bytesToHex(candidateKey)}`, finalizedHead),
  ]);

  const walletBinding = walletHex ? hexToBytes(walletHex) : null;
  const cidRecord = cidRecordHex ? hexToBytes(cidRecordHex) : null;
  if (
    !walletBinding ||
    !sameBytes(walletBinding, accountIdBytes) ||
    !cidRecordIsActive(cidRecord)
  ) {
    return visitorIdentityState(accountId);
  }

  const votingIdentity = votingHex
    ? decodeVotingIdentity(hexToBytes(votingHex))
    : null;
  const hasVotingIdentity = votingIdentity
    ? votingIdentityIsActive(votingIdentity)
    : false;
  const candidateIdentity = candidateHex
    ? decodeCandidateIdentity(hexToBytes(candidateHex))
    : null;
  const hasCandidateIdentity = hasVotingIdentity && candidateIdentity !== null;
  const identityLevel: IdentityLevel = hasCandidateIdentity
    ? "candidate"
    : hasVotingIdentity
      ? "voting"
      : "visitor";

  return {
    account_id: accountId,
    identity_level: identityLevel,
    has_voting_identity: hasVotingIdentity,
    has_candidate_identity: hasCandidateIdentity,
    cid_number: hasVotingIdentity ? cidNumber : null,
    checked_at: nowMs(),
  };
}

export function decodeVotingIdentity(data: Uint8Array): VotingIdentity | null {
  try {
    let offset = 0;
    if (offset + 4 + 4 + 1 > data.length) return null;
    const passportValidFrom = readU32Le(data, offset);
    offset += 4;
    const passportValidUntil = readU32Le(data, offset);
    offset += 4;
    const statusByte = data[offset];
    if (statusByte !== 0 && statusByte !== 1) return null;
    offset += 1;
    offset = readCompactBytes(data, offset, 16).nextOffset;
    offset = readCompactBytes(data, offset, 16).nextOffset;
    offset = readCompactBytes(data, offset, 16).nextOffset;
    if (offset + 4 !== data.length) return null;
    if (
      !isValidDateInt(passportValidFrom) ||
      !isValidDateInt(passportValidUntil)
    ) {
      return null;
    }
    return {
      passport_valid_from: passportValidFrom,
      passport_valid_until: passportValidUntil,
      citizen_status: statusByte === 0 ? "normal" : "revoked",
    };
  } catch {
    return null;
  }
}

export function decodeCandidateIdentity(
  data: Uint8Array,
): CandidateIdentity | null {
  try {
    let offset = 0;
    offset = readCompactBytes(data, offset, 16).nextOffset;
    offset = readCompactBytes(data, offset, 16).nextOffset;
    offset = readCompactBytes(data, offset, 16).nextOffset;
    const familyName = readCompactBytes(data, offset, 128);
    offset = familyName.nextOffset;
    const givenName = readCompactBytes(data, offset, 128);
    offset = givenName.nextOffset;
    if (familyName.value.length === 0 || givenName.value.length === 0)
      return null;
    if (offset + 1 + 4 + 4 !== data.length) return null;
    const citizenSex = data[offset];
    if (citizenSex !== 0 && citizenSex !== 1) return null;
    offset += 1;
    const birthDate = readU32Le(data, offset);
    if (!isValidDateInt(birthDate)) return null;
    return { birth_date: birthDate };
  } catch {
    return null;
  }
}

export function votingIdentityIsActive(
  identity: VotingIdentity,
  now = new Date(nowMs()),
): boolean {
  if (identity.citizen_status !== "normal") {
    return false;
  }
  const today = dateInt(new Date(now.getTime() + 8 * 60 * 60 * 1000));
  return (
    today >= identity.passport_valid_from &&
    today <= identity.passport_valid_until
  );
}

export function decodeCidNumber(data: Uint8Array): string | null {
  try {
    const cid = readCompactBytes(data, 0, 32);
    if (cid.nextOffset !== data.length) return null;
    const value = utf8(cid.value).trim();
    return value || null;
  } catch {
    return null;
  }
}

export function cidRecordIsActive(data: Uint8Array | null): boolean {
  if (!data) return false;
  try {
    let offset = readCompactBytes(data, 0, 32).nextOffset;
    offset += 32;
    if (offset > data.length) return false;
    offset = readCompactBytes(data, offset, 16).nextOffset;
    offset = readCompactBytes(data, offset, 16).nextOffset;
    if (offset + 1 + 4 + 1 > data.length || data[offset] !== 0) return false;
    offset += 1 + 4;
    // Active 记录必须没有撤销块号；状态与 revoked_at 自相矛盾时 fail-closed。
    return data[offset] === 0 && offset + 1 === data.length;
  } catch {
    return false;
  }
}

export function encodeBoundedBytes(value: Uint8Array): Uint8Array {
  if (value.length === 0 || value.length > 32 || value.length >= 64) {
    throw new Error("CID 长度不合法");
  }
  return Uint8Array.from([value.length << 2, ...value]);
}

function sameBytes(left: Uint8Array, right: Uint8Array): boolean {
  if (left.length !== right.length) return false;
  return left.every((value, index) => value === right[index]);
}

function dateInt(date: Date): number {
  const year = date.getUTCFullYear();
  const month = String(date.getUTCMonth() + 1).padStart(2, "0");
  const day = String(date.getUTCDate()).padStart(2, "0");
  return Number(`${year}${month}${day}`);
}

function isValidDateInt(value: number): boolean {
  const year = Math.floor(value / 10000);
  const month = Math.floor((value % 10000) / 100);
  const day = value % 100;
  if (year < 1900 || month < 1 || month > 12 || day < 1 || day > 31)
    return false;
  const date = new Date(Date.UTC(year, month - 1, day));
  return (
    date.getUTCFullYear() === year &&
    date.getUTCMonth() === month - 1 &&
    date.getUTCDate() === day
  );
}

function readCompactBytes(
  data: Uint8Array,
  offset: number,
  maxLen: number,
): { value: Uint8Array; nextOffset: number } {
  const [length, lengthSize] = readCompactU32(data, offset);
  if (length > maxLen) {
    throw new Error("compact bytes too long");
  }
  const start = offset + lengthSize;
  const end = start + length;
  if (end > data.length) {
    throw new Error("compact bytes out of range");
  }
  return {
    value: data.slice(start, end),
    nextOffset: end,
  };
}

function readCompactU32(data: Uint8Array, offset: number): [number, number] {
  if (offset >= data.length) throw new Error("compact offset out of range");
  const first = data[offset];
  const mode = first & 0x03;
  if (mode === 0) return [first >> 2, 1];
  if (mode === 1) {
    if (offset + 1 >= data.length)
      throw new Error("compact mode1 out of range");
    return [(first >> 2) | (data[offset + 1] << 6), 2];
  }
  if (mode === 2) {
    if (offset + 3 >= data.length)
      throw new Error("compact mode2 out of range");
    return [
      (first >> 2) |
        (data[offset + 1] << 6) |
        (data[offset + 2] << 14) |
        (data[offset + 3] << 22),
      4,
    ];
  }
  throw new Error("compact big integer mode is not supported");
}

function readU32Le(data: Uint8Array, offset: number): number {
  return new DataView(data.buffer, data.byteOffset + offset, 4).getUint32(
    0,
    true,
  );
}

function utf8(bytes: Uint8Array): string {
  return new TextDecoder("utf-8", { fatal: false }).decode(bytes);
}
