export type LoginCrypto = 'sr25519' | 'ed25519';

export type LoginChallenge = {
  nonce: string;
  issuedAt: number;
};

export type LoginChallengePayload = {
  type: 'citizennode.login.challenge';
  version: 1;
  issuedAt: number;
  nonce: string;
};

export type SignedLoginPayload = {
  type: 'citizennode.login.signature';
  version: 1;
  crypto: LoginCrypto;
  publicKey: string;
  nonce: string;
  signature: string;
};

function randomNonce(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

export function issueLoginChallenge(): LoginChallenge {
  return {
    nonce: randomNonce(),
    issuedAt: Date.now()
  };
}

export function toChallengePayload(challenge: LoginChallenge): LoginChallengePayload {
  return {
    type: 'citizennode.login.challenge',
    version: 1,
    issuedAt: challenge.issuedAt,
    nonce: challenge.nonce
  };
}

export function getChallengeMessage(challenge: LoginChallenge): string {
  return JSON.stringify(toChallengePayload(challenge));
}

export function parseSignedLoginPayload(raw: string): SignedLoginPayload | null {
  const text = raw.trim();
  if (!text) return null;

  try {
    const parsed = JSON.parse(text) as Partial<SignedLoginPayload>;
    if (parsed.type !== 'citizennode.login.signature') return null;
    if (parsed.version !== 1) return null;
    if (parsed.crypto !== 'sr25519' && parsed.crypto !== 'ed25519') return null;
    if (typeof parsed.publicKey !== 'string' || !parsed.publicKey.trim()) return null;
    if (typeof parsed.nonce !== 'string' || !parsed.nonce.trim()) return null;
    if (typeof parsed.signature !== 'string' || !parsed.signature.trim()) return null;

    return {
      type: parsed.type,
      version: parsed.version,
      crypto: parsed.crypto,
      publicKey: parsed.publicKey.trim(),
      nonce: parsed.nonce.trim(),
      signature: parsed.signature.trim()
    };
  } catch {
    return null;
  }
}
