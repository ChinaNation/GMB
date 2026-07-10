import { describe, expect, it } from 'vitest';
import {
  assertP256PublicKeyHex,
  buildDeviceBindingSigningMessage,
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
  owner_account: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
  p256_pubkey: '04' + 'ab'.repeat(64),
  issued_at: 1_700_000_000_000
};
const DEVICE_BIND_GOLDEN_HEX =
  'e9e25da7159f23e174b3c1cfc214ab41c4ea6fa413844e0e89656e8d24166c31';

describe('buildDeviceBindingSigningMessage', () => {
  it('is signing_message(OP_SIGN_SQUARE_DEVICE_BIND, owner ‖ pubkey ‖ issued_at)', () => {
    const message = buildDeviceBindingSigningMessage(DEVICE_BIND_INPUT);
    expect(message.length).toBe(32);
    // 字段顺序锁：owner → p256_pubkey → issued_at。
    const expected = signingMessage(
      OP_SIGN_SQUARE_DEVICE_BIND,
      concatBytes(
        scaleString(DEVICE_BIND_INPUT.owner_account),
        scaleString(DEVICE_BIND_INPUT.p256_pubkey),
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
  it('accepts a 65-byte uncompressed point and strips 0x', () => {
    const hex = '04' + 'a'.repeat(128);
    expect(assertP256PublicKeyHex('0x' + hex.toUpperCase())).toBe(hex);
  });

  it('rejects wrong length or prefix', () => {
    expect(() => assertP256PublicKeyHex('05' + 'a'.repeat(128))).toThrow();
    expect(() => assertP256PublicKeyHex('04' + 'a'.repeat(120))).toThrow();
    expect(() => assertP256PublicKeyHex(123)).toThrow();
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
    // 0x 前缀两端都接受
    expect(await verifyP256Signature(message, '0x' + sigHex, '0x' + pubHex)).toBe(true);
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
