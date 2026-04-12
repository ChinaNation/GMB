// 链上多签地址派生（前端预览用）
// 公式与链端 DuoqianManagePow::derive_duoqian_address_from_sfid_id 完全一致:
//   input = "DUOQIAN_SFID_V1" ++ SS58_PREFIX_LE(2027) ++ sfid_id_bytes ++ name_bytes
//   address = blake2b_256(input)
//   展示用 SS58(2027) 编码

import { blake2b } from '@noble/hashes/blake2.js';
import { CITIZENCHAIN_SS58_PREFIX, tryEncodeSs58 } from './ss58';

const DUOQIAN_SFID_V1 = new TextEncoder().encode('DUOQIAN_SFID_V1');

/**
 * 从 sfid_id + account_name 派生多签地址（SS58 格式）。
 * 与链端逻辑完全一致，仅用于前端预览展示。
 */
export function deriveDuoqianAddress(sfidId: string, accountName: string): string | null {
  if (!sfidId.trim() || !accountName.trim()) return null;
  try {
    const sfidBytes = new TextEncoder().encode(sfidId);
    const nameBytes = new TextEncoder().encode(accountName);
    // SS58 prefix 2027 little-endian = [0xEB, 0x07]
    const prefixLe = new Uint8Array(2);
    prefixLe[0] = CITIZENCHAIN_SS58_PREFIX & 0xFF;
    prefixLe[1] = (CITIZENCHAIN_SS58_PREFIX >> 8) & 0xFF;
    // 拼接: DUOQIAN_SFID_V1 + prefix_le + sfid_id + name
    const input = new Uint8Array(DUOQIAN_SFID_V1.length + 2 + sfidBytes.length + nameBytes.length);
    input.set(DUOQIAN_SFID_V1, 0);
    input.set(prefixLe, DUOQIAN_SFID_V1.length);
    input.set(sfidBytes, DUOQIAN_SFID_V1.length + 2);
    input.set(nameBytes, DUOQIAN_SFID_V1.length + 2 + sfidBytes.length);
    // blake2b-256
    const digest = blake2b(input, { dkLen: 32 });
    // digest 就是 32 字节的 AccountId，转 hex 后用 SS58 编码
    const hex = Array.from(digest).map(b => b.toString(16).padStart(2, '0')).join('');
    return tryEncodeSs58(hex);
  } catch {
    return null;
  }
}
