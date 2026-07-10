import { describe, expect, it } from 'vitest';
import { bytesToHex, hexToBytes, signingMessage } from '../src/shared/signing_message';

// 金标向量副本，逐字节对齐 citizenchain
// runtime/primitives/tests/fixtures/signing_domain_vectors.json。
// 任一漂移即 worker 与链端签名消息不一致 —— 必须与该 fixture 同步更新。
const GOLDEN_VECTORS: Array<{ op_tag: number; scale_payload_hex: string; message_hex: string }> = [
  { op_tag: 0x10, scale_payload_hex: '0102030405060708', message_hex: '19e050b3476dfd7db0aae9d527e205da44b8f9d00e5ddf4f81f4830ab0c00568' },
  { op_tag: 0x13, scale_payload_hex: '3132333435363738', message_hex: 'd33919e352038d7bd62172b0530362fb6ef0da3990e27b56ea9325195fc6b1a6' },
  { op_tag: 0x14, scale_payload_hex: '4142434445464748', message_hex: '501d0b85cf1ac826d58c974337b824b202d105dd9928a79c423052b8bd976274' },
  { op_tag: 0x15, scale_payload_hex: '5152535455565758', message_hex: '4e4aece62f76d1e6e198cb6382f6bbe49d3d858d9c263d96662bf064c6fc36f0' },
  { op_tag: 0x16, scale_payload_hex: '6162636465666768', message_hex: '49f4eb5bdc4ce83b738568504e1df83e9ce08b39debee48977041da8f17b4af2' },
  { op_tag: 0x17, scale_payload_hex: '7172737475767778', message_hex: '8942b83ce5d46d2e1ca865fe4fbd699c86733551c83c771189aabf65978269da' },
  { op_tag: 0x1a, scale_payload_hex: '696d2d62696e6431', message_hex: 'ecfabbf1ad5cf526920af3e85dd129a45f45190e88fe33431f3e4d83f7a1167f' },
  { op_tag: 0x1b, scale_payload_hex: '73712d6c6f67696e', message_hex: '76a011b084004e797527ada19d740f6a0cf0b1c6d534fd92c08972aceddb642f' },
  { op_tag: 0x1c, scale_payload_hex: '73712d62696e64696e67', message_hex: '6ba60be1df51dbf63ff00a9f2ef838b44492b42933009fb2b48ec9cdd0c32ebc' },
  { op_tag: 0x1d, scale_payload_hex: '73712d616374696f6e', message_hex: '88a1c979a2018717db6313ae6ce8f5766cabd9760158317a557fded0c0119f6a' }
];

describe('signingMessage golden vectors (worker ⇔ citizenchain)', () => {
  for (const vector of GOLDEN_VECTORS) {
    it(`op_tag 0x${vector.op_tag.toString(16)} matches the chain golden message`, () => {
      const message = signingMessage(vector.op_tag, hexToBytes(vector.scale_payload_hex));
      expect(bytesToHex(message)).toBe(vector.message_hex);
    });
  }
});
