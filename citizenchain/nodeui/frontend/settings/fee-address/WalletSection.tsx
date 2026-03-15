import { useState, useEffect, useCallback } from 'react';
import { blake2b } from '@noble/hashes/blake2.js';
import { api, sanitizeError } from '../../api';
import type { RewardWallet } from '../../types';

const SS58_PREFIX = 2027;
const BASE58_ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
const BASE58_INDEX = new Map<string, number>(
  [...BASE58_ALPHABET].map((ch, idx) => [ch, idx])
);

function decodeBase58(input: string): Uint8Array {
  if (!input) {
    throw new Error('SS58 地址为空');
  }
  let leadingZeros = 0;
  while (leadingZeros < input.length && input[leadingZeros] === '1') {
    leadingZeros += 1;
  }

  const bytes: number[] = [0];
  for (const ch of input) {
    const val = BASE58_INDEX.get(ch);
    if (val === undefined) {
      throw new Error('SS58 地址解码失败');
    }
    let carry = val;
    for (let i = 0; i < bytes.length; i += 1) {
      const x = bytes[i] * 58 + carry;
      bytes[i] = x & 0xff;
      carry = x >> 8;
    }
    while (carry > 0) {
      bytes.push(carry & 0xff);
      carry >>= 8;
    }
  }

  const out = new Uint8Array(leadingZeros + bytes.length);
  out.fill(0, 0, leadingZeros);
  for (let i = 0; i < bytes.length; i += 1) {
    out[out.length - 1 - i] = bytes[i];
  }
  return out;
}

function decodeSs58Prefix(data: Uint8Array): { prefix: number; prefixLen: number } {
  if (data.length === 0) {
    throw new Error('SS58 地址为空');
  }
  const first = data[0];
  if (first <= 63) {
    return { prefix: first, prefixLen: 1 };
  }
  if (first <= 127) {
    if (data.length < 2) {
      throw new Error('SS58 地址格式无效');
    }
    const second = data[1];
    const prefix = ((first & 0x3f) << 2) | (second >> 6) | ((second & 0x3f) << 8);
    return { prefix, prefixLen: 2 };
  }
  throw new Error('SS58 地址格式无效');
}

function normalizeWalletAddressClient(input: string): string {
  const value = input.trim();
  if (!value) {
    throw new Error('请输入手续费收款钱包地址');
  }
  if (value.startsWith('0x')) {
    const raw = value.slice(2);
    if (!/^[0-9a-fA-F]{64}$/.test(raw)) {
      throw new Error('十六进制钱包地址格式无效，应为 0x + 64 位十六进制');
    }
    return `0x${raw.toLowerCase()}`;
  }

  const data = decodeBase58(value);
  const { prefix, prefixLen } = decodeSs58Prefix(data);
  if (prefix !== SS58_PREFIX) {
    throw new Error('SS58 地址前缀无效，必须为 2027');
  }
  if (data.length < prefixLen + 32 + 2) {
    throw new Error('SS58 地址长度无效');
  }
  const payloadLen = data.length - prefixLen - 2;
  if (payloadLen !== 32) {
    throw new Error('SS58 地址账户长度无效，必须是 32 字节账户地址');
  }

  // Blake2b-512 校验和验证（Substrate 标准）
  const withoutChecksum = data.slice(0, data.length - 2);
  const actualChecksum = data.slice(data.length - 2);
  const ss58Pre = new TextEncoder().encode('SS58PRE');
  const preimage = new Uint8Array(ss58Pre.length + withoutChecksum.length);
  preimage.set(ss58Pre);
  preimage.set(withoutChecksum, ss58Pre.length);
  const hash = blake2b(preimage, { dkLen: 64 });
  if (actualChecksum[0] !== hash[0] || actualChecksum[1] !== hash[1]) {
    throw new Error('SS58 地址校验和无效');
  }

  return value;
}

type Props = {
  wallet: RewardWallet;
  onUpdated: (next: RewardWallet) => void;
  disabled: boolean;
};

type BindStatus = null | 'binding' | 'success' | 'failed' | 'timeout';

export function WalletSection({ wallet, onUpdated, disabled }: Props) {
  const [input, setInput] = useState(wallet.address ?? '');
  const [showPasswordModal, setShowPasswordModal] = useState(false);
  const [unlockPassword, setUnlockPassword] = useState('');
  const [pendingAddress, setPendingAddress] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [bindStatus, setBindStatus] = useState<BindStatus>(null);
  const hasBoundAddress = Boolean(wallet.address);
  const actionText = hasBoundAddress ? '变更地址' : '绑定地址';

  // 监听后台链上绑定结果事件（仅在用户主动发起绑定后才响应）
  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlisten = await listen<{ status: string; detail: string }>(
          'reward-wallet-bind-result',
          (event) => {
            if (cancelled) return;
            setBindStatus((prev) => {
              if (prev !== 'binding') return prev;
              const { status } = event.payload;
              if (status === 'success') {
                return 'success';
              } else if (status === 'timeout') {
                setError('地址已保存，但链上绑定超时，将在下次启动时重试');
                return 'timeout';
              } else {
                setError(`地址已保存，但链上绑定失败：${event.payload.detail}`);
                return 'failed';
              }
            });
          },
        );
      } catch {
        // listen 不可用时静默降级
      }
    })();
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const onSubmit = useCallback(async () => {
    const password = unlockPassword.trim();
    if (!password) {
      setError('请输入设备开机密码');
      return;
    }
    if (!pendingAddress) {
      setError('地址为空，请重新输入');
      return;
    }
    setSaving(true);
    try {
      const next = await api.setRewardWallet(pendingAddress, password);
      onUpdated(next);
      setInput(next.address ?? '');
      setShowPasswordModal(false);
      setPendingAddress(null);
      setError(null);
      setBindStatus('binding');
    } catch (e) {
      setError(sanitizeError(e));
    } finally {
      setUnlockPassword('');
      setSaving(false);
    }
  }, [unlockPassword, pendingAddress, onUpdated]);

  const bindHint = bindStatus === 'binding'
    ? '链上绑定中，请稍候...'
    : bindStatus === 'success'
      ? '已绑定'
      : null;

  return (
    <section className="section settings-wallet-section">
      <div className="wallet-inline">
        <span className="wallet-current">
          手续费收款地址
          <span className="wallet-bind-state">{wallet.address ?? '未绑定'}</span>
        </span>
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="请输入手续费收款钱包地址"
          disabled={disabled || saving}
        />
        <button
          disabled={disabled || saving}
          onClick={() => {
            let nextAddress = '';
            try {
              nextAddress = normalizeWalletAddressClient(input);
            } catch (e) {
              setError(sanitizeError(e));
              return;
            }
            setError(null);
            setPendingAddress(nextAddress);
            setUnlockPassword('');
            setShowPasswordModal(true);
          }}
        >
          {saving ? '保存中...' : actionText}
        </button>
      </div>
      {bindHint ? <p className="section-inline-hint">{bindHint}</p> : null}
      {error ? <p className="section-inline-error">{error}</p> : null}

      {showPasswordModal ? (
        <div className="unlock-modal-mask" onClick={() => !saving && setShowPasswordModal(false)}>
          <div className="unlock-modal" onClick={(e) => e.stopPropagation()}>
            <h3>设备密码验证</h3>
            <input
              className="unlock-password-input"
              type="password"
              value={unlockPassword}
              onChange={(e) => setUnlockPassword(e.target.value)}
              placeholder="请输入设备开机密码"
              disabled={saving}
            />
            <div className="unlock-modal-actions">
              <button
                onClick={() => setShowPasswordModal(false)}
                disabled={saving}
              >
                取消
              </button>
              <button
                onClick={onSubmit}
                disabled={saving || disabled}
              >
                {saving ? '验证中...' : actionText}
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </section>
  );
}
