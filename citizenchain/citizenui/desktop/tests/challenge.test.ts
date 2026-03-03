import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import { issueLoginChallenge, parseSignedLoginPayload } from '../src/services/auth/challenge.js';

class MemoryStorage implements Storage {
  private readonly data = new Map<string, string>();

  get length(): number {
    return this.data.size;
  }

  clear(): void {
    this.data.clear();
  }

  getItem(key: string): string | null {
    return this.data.has(key) ? this.data.get(key)! : null;
  }

  key(index: number): string | null {
    return Array.from(this.data.keys())[index] ?? null;
  }

  removeItem(key: string): void {
    this.data.delete(key);
  }

  setItem(key: string, value: string): void {
    this.data.set(key, value);
  }
}

test('issueLoginChallenge keeps a stable persisted device origin', () => {
  const storage = new MemoryStorage();
  const original = Object.getOwnPropertyDescriptor(globalThis, 'localStorage');
  Object.defineProperty(globalThis, 'localStorage', {
    configurable: true,
    value: storage
  });

  try {
    const first = issueLoginChallenge();
    const second = issueLoginChallenge();
    assert.equal(first.origin, second.origin);
    assert.match(first.origin, /^[0-9a-f-]{16,}$/i);
    assert.equal(first.expiresAt - first.issuedAt, 90);
  } finally {
    if (original) {
      Object.defineProperty(globalThis, 'localStorage', original);
    } else {
      // Keep globals clean for following tests.
      Reflect.deleteProperty(globalThis, 'localStorage');
    }
  }
});

test('parseSignedLoginPayload validates payload shape', () => {
  const payload = parseSignedLoginPayload(
    JSON.stringify({
      proto: 'WUMINAPP_LOGIN_V1',
      request_id: 'abc',
      account: 'acc',
      pubkey: '0x1234',
      sig_alg: 'sr25519',
      signature: 'deadbeef',
      signed_at: 123
    })
  );
  assert.ok(payload);
  assert.equal(payload?.request_id, 'abc');

  assert.equal(parseSignedLoginPayload('{"proto":"wrong"}'), null);
});
