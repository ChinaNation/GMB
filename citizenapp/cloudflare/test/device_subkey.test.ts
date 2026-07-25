import { describe, expect, it } from 'vitest';
import {
  assertP256PublicKeyHex,
  buildDeviceBindingSigningMessage,
  normalizeP256SignatureHex,
  verifyP256Signature
} from '../src/auth/device_subkey';
import {
  OP_SIGN_SQUARE_DEVICE_BIND,
  bytesToHex,
  concatBytes,
  scaleString,
  signingMessage,
  u64Le
} from '../src/shared/signing_message';

function toHex(buf: ArrayBuffer): string {
  return [...new Uint8Array(buf)]
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

// 设备绑定是唯一「客户端 + Worker 双侧各自 SCALE 编码」的流，须逐字节对齐。
// 该 golden hex 必须与 App 端 test/signer/device_binding_golden_test.dart 完全一致。
const DEVICE_BIND_INPUT = {
  account_id: '0x1111111111111111111111111111111111111111111111111111111111111111',
  p256_public_key: '04' + 'ab'.repeat(64),
  issued_at: 1_700_000_000_000
};
const DEVICE_BIND_GOLDEN_HEX =
  '0089e293c8ef5c4d7bb5820e18dcb0bdac4eb374eaf6675c1bc2e53e50c3b960';

describe('buildDeviceBindingSigningMessage', () => {
  it('is signing_message(OP_SIGN_SQUARE_DEVICE_BIND, accountId ‖ pubkey ‖ issued_at)', () => {
    const message = buildDeviceBindingSigningMessage(DEVICE_BIND_INPUT);
    expect(message.length).toBe(32);
    // 字段顺序锁：accountId → p256_public_key → issued_at。
    const expected = signingMessage(
      OP_SIGN_SQUARE_DEVICE_BIND,
      concatBytes(
        scaleString(DEVICE_BIND_INPUT.account_id),
        scaleString(DEVICE_BIND_INPUT.p256_public_key),
        u64Le(DEVICE_BIND_INPUT.issued_at)
      )
    );
    expect(bytesToHex(message)).toBe(bytesToHex(expected));
  });

  it('matches the cross-language golden hex (App ⇔ Worker)', () => {
    expect(bytesToHex(buildDeviceBindingSigningMessage(DEVICE_BIND_INPUT))).toBe(
      DEVICE_BIND_GOLDEN_HEX
    );
  });
});

describe('assertP256PublicKeyHex', () => {
  it('accepts only the canonical lowercase 0x-prefixed 65-byte point and returns bare (ADR-041)', () => {
    const bare = '04' + 'a'.repeat(128);
    // 跨端文本须带 0x；返回值 strip 为裸供内部 SCALE/存储/hash 使用。
    expect(assertP256PublicKeyHex('0x' + bare)).toBe(bare);
    expect(() => assertP256PublicKeyHex(bare)).toThrow(); // 裸 → 拒
    expect(() => assertP256PublicKeyHex('0x' + bare.toUpperCase())).toThrow(); // 大写 → 拒
  });

  it('rejects wrong length or prefix', () => {
    expect(() => assertP256PublicKeyHex('0x05' + 'a'.repeat(128))).toThrow();
    expect(() => assertP256PublicKeyHex('0x04' + 'a'.repeat(120))).toThrow();
    expect(() => assertP256PublicKeyHex(123)).toThrow();
  });
});

describe('normalizeP256SignatureHex', () => {
  it('accepts only the canonical 0x-prefixed 64-byte signature and returns bare (ADR-041)', () => {
    const bare = 'a'.repeat(128);
    expect(normalizeP256SignatureHex('0x' + bare)).toBe(bare);
    expect(normalizeP256SignatureHex(bare)).toBeNull(); // 裸 → null
    expect(normalizeP256SignatureHex('0x' + bare.toUpperCase())).toBeNull(); // 大写 → null
    expect(normalizeP256SignatureHex('0x' + 'a'.repeat(120))).toBeNull(); // 错长 → null
    expect(normalizeP256SignatureHex(123)).toBeNull();
  });
});

describe('verifyP256Signature', () => {
  it('accepts a valid ES256 signature over the message digest and rejects tampering', async () => {
    const keyPair = await crypto.subtle.generateKey(
      { name: 'ECDSA', namedCurve: 'P-256' },
      true,
      ['sign', 'verify']
    );
    const pubHex = toHex(await crypto.subtle.exportKey('raw', keyPair.publicKey));
    const message = signingMessage(0x1b, scaleString('login-challenge'));
    const sigHex = toHex(
      await crypto.subtle.sign(
        { name: 'ECDSA', hash: 'SHA-256' },
        keyPair.privateKey,
        message
      )
    );

    expect(await verifyP256Signature(message, sigHex, pubHex)).toBe(true);
    // verifyP256Signature 是内部裸函数：0x 前缀须由边界（normalizeP256SignatureHex /
    // assertP256PublicKeyHex）先 strip；函数本身拒绝任何带 0x 的输入（ADR-041）。
    expect(await verifyP256Signature(message, '0x' + sigHex, '0x' + pubHex)).toBe(false);
    // 篡改 message → 拒
    const tampered = signingMessage(0x1b, scaleString('login-challenge-x'));
    expect(await verifyP256Signature(tampered, sigHex, pubHex)).toBe(false);
  });

  it('rejects malformed signature or pubkey', async () => {
    const message = new Uint8Array(32).fill(7);
    expect(await verifyP256Signature(message, 'zz', '04' + '0'.repeat(128))).toBe(false);
    expect(
      await verifyP256Signature(message, '0'.repeat(128), '05' + '0'.repeat(128))
    ).toBe(false);
  });
});
