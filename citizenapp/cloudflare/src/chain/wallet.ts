import type { Env } from '../types';
import { HttpError } from '../shared/http';
import { bytesToHex, hexToBytes } from '../shared/signing_message';
import { decodeOwnerAccount, storageMapKey } from './storage_key';
import { fetchChainStorage } from './rpc';

/// 账户存在最低余额（ExistentialDeposit）。真源在链
/// `citizenchain/runtime/primitives/src/core_const.rs`（`ACCOUNT_EXISTENTIAL_DEPOSIT = 111`）。
/// 余额 < 111 分 → 链上账户被 reap 销毁，即"不是链上钱包"。该值恒定，永不改。
export const ACCOUNT_EXISTENTIAL_DEPOSIT_FEN = 111n;

/// `frame_system::AccountInfo` 前导 = nonce(u32) + consumers(u32) + providers(u32)
/// + sufficients(u32) = 16 字节；其后 `data.free`（u128 LE，pallet_balances::AccountData 首字段）。
const ACCOUNT_FREE_OFFSET = 16;

/**
 * 读链上 `System.Account[owner]` 的可用余额（free）。
 * 账户不存在（从未入金或已被 reap）返回 null；余额单位为分。
 */
export async function fetchAccountFreeBalance(
  env: Env,
  ownerAccount: string
): Promise<bigint | null> {
  const accountId = decodeOwnerAccount(ownerAccount);
  const key = storageMapKey('System', 'Account', accountId);
  const hex = await fetchChainStorage(env, `0x${bytesToHex(key)}`);
  if (!hex) return null;
  const data = hexToBytes(hex);
  if (data.length < ACCOUNT_FREE_OFFSET + 16) {
    throw new HttpError(502, 'chain_account_decode_failed', '链上账户数据无法解析');
  }
  return readU128Le(data, ACCOUNT_FREE_OFFSET);
}

/**
 * 会话签发门禁：签名钱包必须是链上活账户（free ≥ ED）。不满足抛 403。
 * 链 RPC 读不到（宕机/超时）时 `fetchAccountFreeBalance` 会向上抛 5xx —— fail-closed，同样拒发会话。
 */
export async function assertOnchainWallet(env: Env, ownerAccount: string): Promise<void> {
  const free = await fetchAccountFreeBalance(env, ownerAccount);
  if (free === null || free < ACCOUNT_EXISTENTIAL_DEPOSIT_FEN) {
    throw new HttpError(
      403,
      'not_onchain_wallet',
      '需链上钱包（余额≥1.11元）才能使用广场和聊天，请先为钱包充值'
    );
  }
}

function readU128Le(data: Uint8Array, offset: number): bigint {
  let result = 0n;
  for (let i = 15; i >= 0; i--) {
    result = (result << 8n) | BigInt(data[offset + i]);
  }
  return result;
}
