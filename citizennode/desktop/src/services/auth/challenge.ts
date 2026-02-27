export type LoginCrypto = 'sr25519' | 'ed25519';

export type LoginChallenge = {
  proto: 'WUMINAPP_LOGIN_V1';
  system: 'citizenchain';
  requestId: string;
  challenge: string;
  nonce: string;
  issuedAt: number; // epoch seconds
  expiresAt: number; // epoch seconds
  aud: string;
  origin: string;
};

export type LoginChallengePayload = {
  proto: 'WUMINAPP_LOGIN_V1';
  system: 'citizenchain';
  request_id: string;
  challenge: string;
  nonce: string;
  issued_at: number;
  expires_at: number;
  aud: string;
  origin: string;
};

export type SignedLoginPayload = {
  proto: 'WUMINAPP_LOGIN_V1';
  request_id: string;
  account: string;
  pubkey: string;
  sig_alg: LoginCrypto;
  signature: string;
  signed_at: number;
};

function randomNonce(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

function randomHex32(): string {
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

export function issueLoginChallenge(): LoginChallenge {
  const now = Math.floor(Date.now() / 1000);
  return {
    proto: 'WUMINAPP_LOGIN_V1',
    system: 'citizenchain',
    requestId: randomNonce(),
    challenge: randomHex32(),
    nonce: randomNonce(),
    issuedAt: now,
    expiresAt: now + 60,
    aud: 'citizenchain-front',
    origin: 'citizenchain-device-id'
  };
}

export function toChallengePayload(challenge: LoginChallenge): LoginChallengePayload {
  return {
    proto: challenge.proto,
    system: challenge.system,
    request_id: challenge.requestId,
    challenge: challenge.challenge,
    nonce: challenge.nonce,
    issued_at: challenge.issuedAt,
    expires_at: challenge.expiresAt,
    aud: challenge.aud,
    origin: challenge.origin
  };
}

export function getChallengeMessage(challenge: LoginChallenge): string {
  return JSON.stringify(toChallengePayload(challenge));
}

export function getSignMessage(challenge: LoginChallenge): string {
  return [
    challenge.proto,
    challenge.system,
    challenge.aud,
    challenge.origin,
    challenge.requestId,
    challenge.challenge,
    challenge.nonce,
    String(challenge.expiresAt)
  ].join('|');
}

export function parseSignedLoginPayload(raw: string): SignedLoginPayload | null {
  const text = raw.trim();
  if (!text) return null;

  try {
    const parsed = JSON.parse(text) as Partial<SignedLoginPayload>;
    if (parsed.proto !== 'WUMINAPP_LOGIN_V1') return null;
    if (typeof parsed.request_id !== 'string' || !parsed.request_id.trim()) return null;
    if (typeof parsed.account !== 'string' || !parsed.account.trim()) return null;
    if (typeof parsed.pubkey !== 'string' || !parsed.pubkey.trim()) return null;
    if (parsed.sig_alg !== 'sr25519' && parsed.sig_alg !== 'ed25519') return null;
    if (typeof parsed.signature !== 'string' || !parsed.signature.trim()) return null;
    if (typeof parsed.signed_at !== 'number') return null;

    return {
      proto: parsed.proto,
      request_id: parsed.request_id.trim(),
      account: parsed.account.trim(),
      pubkey: parsed.pubkey.trim(),
      sig_alg: parsed.sig_alg,
      signature: parsed.signature.trim(),
      signed_at: parsed.signed_at
    };
  } catch {
    return null;
  }
}
