import { signatureVerify } from '@polkadot/util-crypto/signature/verify';

export interface LoginPayloadInput {
  owner_account: string;
  challenge_id: string;
  expires_at: number;
}

export function buildLoginPayload(input: LoginPayloadInput): string {
  return [
    'GMB_SQUARE_LOGIN_V1',
    `owner_account:${input.owner_account}`,
    `challenge_id:${input.challenge_id}`,
    `expires_at:${input.expires_at}`
  ].join('\n');
}

export async function verifyWalletSignature(
  signingPayload: string,
  signature: string,
  ownerAccount: string
): Promise<boolean> {
  const result = signatureVerify(signingPayload, signature, ownerAccount);
  return result.isValid;
}
