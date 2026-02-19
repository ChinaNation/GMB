import { invoke } from '@tauri-apps/api/core';

type CryptoKind = 'sr25519' | 'ed25519';

export async function verifyLoginSignature(input: {
  payload: string;
  signature: string;
  publicKey: string;
  crypto?: CryptoKind;
}): Promise<boolean> {
  return invoke<boolean>('verify_login_signature', {
    payload: input.payload,
    signature: input.signature,
    publicKey: input.publicKey,
    crypto: input.crypto
  });
}
