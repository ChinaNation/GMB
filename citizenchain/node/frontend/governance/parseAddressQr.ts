// 解析收款地址二维码，对齐 QR_TECHNICAL.md 中 QrRouter 路由优先级。

export type AddressScanResult = {
  address: string;
  amount?: number;
  memo?: string;
};

const SS58_RE = /^[1-9A-HJ-NP-Za-km-z]{30,80}$/;
const GMB_ACCOUNT_RE = /^gmb:\/\/account\/([1-9A-HJ-NP-Za-km-z]{30,80})$/;

export function parseAddressQr(raw: string): AddressScanResult {
  const trimmed = raw.trim();

  // 1. 尝试 JSON 解析
  if (trimmed.startsWith('{')) {
    try {
      const obj = JSON.parse(trimmed);
      const proto = obj.proto ?? obj.type;

      if (proto === 'WUMIN_USER_V1.0.0') {
        if (obj.purpose === 'transfer') {
          // 收款码：to 必填，amount/memo 可选
          const to = obj.to;
          if (typeof to !== 'string' || !SS58_RE.test(to)) {
            throw new Error('收款码中地址格式无效');
          }
          const result: AddressScanResult = { address: to };
          if (obj.amount != null && obj.amount !== '') {
            const amt = Number(obj.amount);
            if (!isNaN(amt) && amt > 0) result.amount = amt;
          }
          if (typeof obj.memo === 'string' && obj.memo) {
            result.memo = obj.memo;
          }
          return result;
        }
        // 用户码（contact 或无 purpose）：address 字段
        const addr = obj.address;
        if (typeof addr !== 'string' || !SS58_RE.test(addr)) {
          throw new Error('用户码中地址格式无效');
        }
        return { address: addr };
      }

      // 旧版用户码兼容
      if (proto === 'WUMINAPP_USER_CARD_V1') {
        const pk = obj.account_pubkey;
        if (typeof pk === 'string' && SS58_RE.test(pk)) {
          return { address: pk };
        }
        throw new Error('旧版用户码中地址格式无效');
      }

      // 其他协议（login/sign）不是收款码
      if (proto) {
        throw new Error('该二维码不是收款码');
      }
    } catch (e) {
      if (e instanceof SyntaxError) {
        // JSON 解析失败，继续尝试其他格式
      } else {
        throw e;
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
