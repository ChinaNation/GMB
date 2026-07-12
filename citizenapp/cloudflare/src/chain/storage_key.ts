import { blake2AsU8a, decodeAddress, xxhashAsU8a } from '@polkadot/util-crypto';
import { HttpError } from '../shared/http';

/// 链上 storage 读取的共用底座：SS58 地址解码 + `Blake2_128Concat` map storage key 拼装。
/// 由 chain/identity（护照身份）与 chain/wallet（账户余额门禁）共用，避免各自复制。

/** SS58 钱包地址 → 32 字节 AccountId；格式不合法抛 400。 */
export function decodeOwnerAccount(ownerAccount: string): Uint8Array {
  try {
    return decodeAddress(ownerAccount);
  } catch {
    throw new HttpError(400, 'invalid_owner_account', '钱包账户地址不合法');
  }
}

/**
 * 拼 `Blake2_128Concat` 单键 map 的完整 storage key：
 * xxhash128(pallet) ++ xxhash128(storage) ++ blake2_128(key) ++ key。
 */
export function storageMapKey(
  palletName: string,
  storageName: string,
  keyData: Uint8Array
): Uint8Array {
  const palletHash = xxhashAsU8a(palletName, 128);
  const storageHash = xxhashAsU8a(storageName, 128);
  const keyHash = blake2AsU8a(keyData, 128);
  return concat([palletHash, storageHash, keyHash, keyData]);
}

/** 顺序拼接多段字节。 */
export function concat(chunks: Uint8Array[]): Uint8Array {
  const length = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const out = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.length;
  }
  return out;
}
