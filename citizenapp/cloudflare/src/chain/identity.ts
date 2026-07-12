import { bytesToHex, hexToBytes } from '../shared/signing_message';
import type { Env } from '../types';
import { decodeOwnerAccount, storageMapKey } from './storage_key';
import { nowMs } from '../shared/time';
import { putKvJson } from '../limits/storage';
import { fetchChainStorage } from './rpc';
import type { RequiredIdentityLevel } from '../membership/plans';

export interface ChainIdentityState {
  owner_account: string;
  identity_level: RequiredIdentityLevel;
  has_voting_identity: boolean;
  has_candidate_identity: boolean;
  cid_number: string | null;
  checked_at: number;
}

interface VotingIdentity {
  cid_number: string;
  passport_valid_from: number;
  passport_valid_until: number;
  citizen_status: 'normal' | 'revoked';
}

/// 身份读取的 KV 短缓存 TTL（秒）。护照有效期按「日」判定，当天内结果稳定。
const IDENTITY_CACHE_TTL_SECONDS = 45;

function visitorIdentityState(ownerAccount: string): ChainIdentityState {
  return {
    owner_account: ownerAccount,
    identity_level: 'visitor',
    has_voting_identity: false,
    has_candidate_identity: false,
    cid_number: null,
    checked_at: nowMs()
  };
}

/// 带 KV 短缓存 + 失败软降级的身份读取。
///
/// 主页/feed 等展示路径用它，绝不因链上 RPC 未配置/超时/失败而阻塞渲染：
/// 命中缓存直接返回；未命中读链并回写 KV；读链失败软降级为访客（未认证），不抛错。
export async function fetchChainIdentityStateCached(
  env: Env,
  ownerAccount: string
): Promise<ChainIdentityState> {
  const cacheKey = `square_identity:${ownerAccount}`;
  try {
    const cached = await env.SQUARE_CACHE.get(cacheKey);
    if (cached) {
      return JSON.parse(cached) as ChainIdentityState;
    }
  } catch {
    // 缓存读失败忽略，继续读链。
  }
  try {
    const state = await fetchChainIdentityState(env, ownerAccount);
    try {
      await putKvJson(env, cacheKey, state, 'identity_cache', {
        expirationTtl: IDENTITY_CACHE_TTL_SECONDS
      });
    } catch {
      // 缓存写失败忽略。
    }
    return state;
  } catch {
    // 链上 RPC 未配置/超时/失败：软降级为访客，展示未认证，不阻塞主页。
    return visitorIdentityState(ownerAccount);
  }
}

export async function fetchChainIdentityState(
  env: Env,
  ownerAccount: string
): Promise<ChainIdentityState> {
  const accountId = decodeOwnerAccount(ownerAccount);
  const votingKey = storageMapKey('CitizenIdentity', 'VotingIdentityByAccount', accountId);
  const candidateKey = storageMapKey('CitizenIdentity', 'CandidateIdentityByAccount', accountId);
  const [votingHex, candidateHex] = await Promise.all([
    fetchChainStorage(env, `0x${bytesToHex(votingKey)}`),
    fetchChainStorage(env, `0x${bytesToHex(candidateKey)}`)
  ]);

  const votingIdentity = votingHex ? decodeVotingIdentity(hexToBytes(votingHex)) : null;
  const hasVotingIdentity = votingIdentity ? votingIdentityIsActive(votingIdentity) : false;
  const hasCandidateIdentity = hasVotingIdentity && Boolean(candidateHex);
  const identityLevel: RequiredIdentityLevel = hasCandidateIdentity
    ? 'candidate'
    : hasVotingIdentity
      ? 'voting'
      : 'visitor';

  return {
    owner_account: ownerAccount,
    identity_level: identityLevel,
    has_voting_identity: hasVotingIdentity,
    has_candidate_identity: hasCandidateIdentity,
    cid_number: hasVotingIdentity ? votingIdentity?.cid_number ?? null : null,
    checked_at: nowMs()
  };
}

function decodeVotingIdentity(data: Uint8Array): VotingIdentity | null {
  try {
    let offset = 0;
    const cid = readCompactBytes(data, offset, 32);
    offset = cid.nextOffset;
    if (offset + 4 + 4 + 1 > data.length) return null;
    const passportValidFrom = readU32Le(data, offset);
    offset += 4;
    const passportValidUntil = readU32Le(data, offset);
    offset += 4;
    const statusByte = data[offset];
    if (statusByte !== 0 && statusByte !== 1) return null;
    return {
      cid_number: utf8(cid.value).trim(),
      passport_valid_from: passportValidFrom,
      passport_valid_until: passportValidUntil,
      citizen_status: statusByte === 0 ? 'normal' : 'revoked'
    };
  } catch {
    return null;
  }
}

function votingIdentityIsActive(identity: VotingIdentity): boolean {
  if (!identity.cid_number || identity.citizen_status !== 'normal') {
    return false;
  }
  const today = dateInt(new Date(nowMs()));
  return today >= identity.passport_valid_from && today <= identity.passport_valid_until;
}

function dateInt(date: Date): number {
  const year = date.getUTCFullYear();
  const month = String(date.getUTCMonth() + 1).padStart(2, '0');
  const day = String(date.getUTCDate()).padStart(2, '0');
  return Number(`${year}${month}${day}`);
}

function readCompactBytes(
  data: Uint8Array,
  offset: number,
  maxLen: number
): { value: Uint8Array; nextOffset: number } {
  const [length, lengthSize] = readCompactU32(data, offset);
  if (length > maxLen) {
    throw new Error('compact bytes too long');
  }
  const start = offset + lengthSize;
  const end = start + length;
  if (end > data.length) {
    throw new Error('compact bytes out of range');
  }
  return {
    value: data.slice(start, end),
    nextOffset: end
  };
}

function readCompactU32(data: Uint8Array, offset: number): [number, number] {
  if (offset >= data.length) throw new Error('compact offset out of range');
  const first = data[offset];
  const mode = first & 0x03;
  if (mode === 0) return [first >> 2, 1];
  if (mode === 1) {
    if (offset + 1 >= data.length) throw new Error('compact mode1 out of range');
    return [(first >> 2) | (data[offset + 1] << 6), 2];
  }
  if (mode === 2) {
    if (offset + 3 >= data.length) throw new Error('compact mode2 out of range');
    return [
      (first >> 2) |
        (data[offset + 1] << 6) |
        (data[offset + 2] << 14) |
        (data[offset + 3] << 22),
      4
    ];
  }
  throw new Error('compact big integer mode is not supported');
}

function readU32Le(data: Uint8Array, offset: number): number {
  return new DataView(data.buffer, data.byteOffset + offset, 4).getUint32(0, true);
}

function utf8(bytes: Uint8Array): string {
  return new TextDecoder('utf-8', { fatal: false }).decode(bytes);
}
