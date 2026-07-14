// 临时 sr25519 钱包会员操作驱动（等价 CitizenApp 扫码签名，无任何真实密钥）。
// 被签消息严格复刻 Worker：message = blake2_256(GMB(0x47,0x4d,0x42) || 0x1D || signing_payload)。
//
// 用法(在 citizenapp/cloudflare 目录，或确保能解析 @polkadot)：
//   BASE=http://127.0.0.1:8787 SEED=<32B hex 可选> node membership_driver.mjs <op> [args...]
// op:
//   wallet                              仅打印 owner_account（配合 SEED 复用同一钱包）
//   subscribe   <level>                 卡订阅（新订阅→checkout_url / 换档→action）
//   prepaid     <level> <quarter|year>  USDC 预付购买 → checkout_url
//   change      <targetLevel>           USDC 换挡 → action / checkout_url
//   cancel                              退订 → cancel_kind
import {
  cryptoWaitReady, sr25519PairFromSeed, sr25519Sign,
  encodeAddress, randomAsU8a, blake2AsU8a
} from '@polkadot/util-crypto';
import { u8aToHex, hexToU8a } from '@polkadot/util';

const BASE = process.env.BASE || 'http://127.0.0.1:8787';
const SS58 = 2027;
const GMB = [0x47, 0x4d, 0x42];
const OP_SQUARE_ACTION = 0x1d;

// 被签消息 = blake2_256(GMB || op_tag || scalePayload)（逐字节对齐 signing_message.ts）。
function signingMessage(payloadU8a) {
  return blake2AsU8a(new Uint8Array([...GMB, OP_SQUARE_ACTION & 0xff, ...payloadU8a]), 256);
}

async function post(path, body) {
  const res = await fetch(`${BASE}${path}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body)
  });
  const text = await res.text();
  let json = null;
  try { json = JSON.parse(text); } catch { /* 非 JSON（如 Access 页/HTML） */ }
  return { status: res.status, json, text };
}

async function main() {
  await cryptoWaitReady();
  const op = process.argv[2];
  const seed = process.env.SEED ? hexToU8a(process.env.SEED) : randomAsU8a(32);
  const pair = sr25519PairFromSeed(seed);
  const owner = encodeAddress(pair.publicKey, SS58);

  if (op === 'wallet') {
    console.log(JSON.stringify({ owner, seed: u8aToHex(seed) }, null, 2));
    return;
  }

  // 1) 各操作对应的 挑战/确认 路径与 body。
  const level = process.argv[3];
  const duration = process.argv[4];
  const map = {
    subscribe: {
      cha: ['/v1/square/membership/subscribe/challenge', { owner_account: owner, membership_level: level }],
      con: (sig, cid) => ['/v1/square/membership/subscribe', { owner_account: owner, membership_level: level, challenge_id: cid, signature: sig }]
    },
    prepaid: {
      cha: ['/v1/square/membership/prepaid/challenge', { owner_account: owner, membership_level: level, duration }],
      con: (sig, cid) => ['/v1/square/membership/prepaid', { owner_account: owner, membership_level: level, duration, challenge_id: cid, signature: sig }]
    },
    change: {
      cha: ['/v1/square/membership/prepaid/change/challenge', { owner_account: owner, membership_level: level }],
      con: (sig, cid) => ['/v1/square/membership/prepaid/change', { owner_account: owner, membership_level: level, challenge_id: cid, signature: sig }]
    },
    cancel: {
      cha: ['/v1/square/membership/cancel/challenge', { owner_account: owner }],
      con: (sig, cid) => ['/v1/square/membership/cancel', { owner_account: owner, challenge_id: cid, signature: sig }]
    }
  };
  const plan = map[op];
  if (!plan) { console.log('unknown op:', op); process.exit(1); }

  console.log('owner_account =', owner, '| op =', op, level ? `| level=${level}` : '', duration ? `| ${duration}` : '');

  // 2) 取挑战。
  const cRes = await post(plan.cha[0], plan.cha[1]);
  if (cRes.status !== 200 || !cRes.json) {
    console.log('CHALLENGE FAIL', cRes.status, cRes.text.slice(0, 300));
    process.exit(2);
  }
  const { challenge_id, signing_payload_hex, op_tag } = cRes.json;
  console.log('challenge ok: op_tag =', op_tag, '| challenge_id =', challenge_id, '| preview =', JSON.stringify(cRes.json.preview ?? null));

  // 3) 签名：sr25519 签 signingMessage(payload)。
  const message = signingMessage(hexToU8a(signing_payload_hex));
  const sig = '0x' + u8aToHex(sr25519Sign(message, pair)).replace(/^0x/, '');

  // 4) 提交确认。
  const [conPath, conBody] = plan.con(sig, challenge_id);
  const fRes = await post(conPath, conBody);
  console.log('CONFIRM HTTP', fRes.status);
  console.log('CONFIRM body:', fRes.json ? JSON.stringify(fRes.json, null, 2) : fRes.text.slice(0, 400));
}

main().catch((e) => { console.log('ERR', e.message); process.exit(3); });
