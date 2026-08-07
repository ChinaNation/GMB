// 解析「扫码识别账户」二维码,用于治理提案的收款地址、手续费地址与安全基金地址。
//
// 唯一事实源:memory/01-architecture/qr/qr-protocol-spec.md
// 这里要的是「一个账户」,因此只接受 `k=5 account_id_code` 账户码。
// 用户码(k=3)表达的是「人」、收款码(k=4)表达的是「一笔收款请求」,都不是账户声明,一律拒绝。
// 裸 SS58 地址和 gmb://account/<addr> 仍然支持(非二维码协议的本地输入兜底)。

import { accountIdToSs58 } from '../ss58';
import { parseQrEnvelope, QrParseError, type AccountIdCodeBody } from './citizenQr';

export type AddressScanResult = {
  ss58_address: string;
};

const SS58_RE = /^[1-9A-HJ-NP-Za-km-z]{30,80}$/;
const GMB_ACCOUNT_RE = /^gmb:\/\/account\/([1-9A-HJ-NP-Za-km-z]{30,80})$/;

export function parseAddressQr(raw: string): AddressScanResult {
  const trimmed = raw.trim();

  // 1. QR_V1 envelope
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
      if (env.kind !== 'account_id_code') {
        throw new Error('请扫描账户码（钱包 → 账户详情右上角二维码）');
      }
      const body = env.body as AccountIdCodeBody;
      const ss58Address = accountIdToSs58(body.account_id);
      if (!SS58_RE.test(ss58Address)) {
        throw new Error('账户码中账户格式无效');
      }
      return { ss58_address: ss58Address };
    }
  }

  // 2. gmb://account/<address>
  const gmbMatch = GMB_ACCOUNT_RE.exec(trimmed);
  if (gmbMatch) {
    return { ss58_address: gmbMatch[1] };
  }

  // 3. 裸 SS58 地址
  if (SS58_RE.test(trimmed)) {
    return { ss58_address: trimmed };
  }

  throw new Error('无法识别的二维码');
}
