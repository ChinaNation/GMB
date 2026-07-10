import { signatureVerify } from '@polkadot/util-crypto/signature/verify';

/// sr25519 主钥验签。[message] 为 `signing_message(op_tag)` 的 32 字节摘要
/// （不再是任何 GMB_*_V1 字符串原文）。签名/公钥非法一律返回 false。
export async function verifyWalletSignature(
  message: Uint8Array,
  signature: string,
  ownerAccount: string
): Promise<boolean> {
  try {
    return signatureVerify(message, signature, ownerAccount).isValid;
  } catch {
    return false;
  }
}
