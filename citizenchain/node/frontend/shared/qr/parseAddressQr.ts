// 解析收款地址二维码。
//
// 唯一事实源:memory/05-architecture/qr-protocol-spec.md
// 只接受 WUMIN_QR_V1 envelope,kind ∈ { user_contact, user_transfer, user_duoqian }。
// 其他 kind(login_*、sign_*)不是收款码,直接报错。
// 裸 SS58 地址和 gmb://account/<addr> 仍然支持(非二维码协议的本地输入兜底)。

import { parseQrEnvelope, QrParseError, type UserContactBody, type UserTransferBody, type UserDuoqianBody } from './wuminQr';

export type AddressScanResult = {
  address: string;
  amount?: number;
  memo?: string;
};

const SS58_RE = /^[1-9A-HJ-NP-Za-km-z]{30,80}$/;
const GMB_ACCOUNT_RE = /^gmb:\/\/account\/([1-9A-HJ-NP-Za-km-z]{30,80})$/;

export function parseAddressQr(raw: string): AddressScanResult {
  const trimmed = raw.trim();

  // 1. WUMIN_QR_V1 envelope
  if (trimmed.startsWith('{')) {
    let env;
    try {
      env = parseQrEnvelope(trimmed);
    } catch (e) {
      if (e instanceof QrParseError) {
        throw new Error(`二维码解析失败: ${e.message}`);
      }
      if (e instanceof SyntaxError) {
        // 不是 JSON,继续尝试 gmb:// 或裸地址
        env = null;
      } else {
        throw e;
      }
    }

    if (env) {
      switch (env.kind) {
        case 'user_contact':
        case 'user_duoqian': {
          const body = env.body as UserContactBody | UserDuoqianBody;
          if (!SS58_RE.test(body.address)) {
            throw new Error('用户码中地址格式无效');
          }
          return { address: body.address };
        }
        case 'user_transfer': {
          const body = env.body as UserTransferBody;
          if (!SS58_RE.test(body.address)) {
            throw new Error('收款码中地址格式无效');
          }
          const result: AddressScanResult = { address: body.address };
          if (body.amount) {
            const amt = Number(body.amount);
            if (!isNaN(amt) && amt > 0) result.amount = amt;
          }
          if (body.memo) {
            result.memo = body.memo;
          }
          return result;
        }
        default:
          throw new Error('该二维码不是收款码');
      }
    }
  }

  // 2. gmb://account/<address>
  const gmbMatch = GMB_ACCOUNT_RE.exec(trimmed);
  if (gmbMatch) {
    return { address: gmbMatch[1] };
  }

  // 3. 裸 SS58 地址
  if (SS58_RE.test(trimmed)) {
    return { address: trimmed };
  }

  throw new Error('无法识别的二维码');
}
