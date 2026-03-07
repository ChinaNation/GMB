import { useState } from 'react';
import { api } from '../../api';
import type { RewardWallet } from '../../types';

type Props = {
  wallet: RewardWallet;
  onUpdated: (next: RewardWallet) => void;
  disabled: boolean;
};

export function WalletSection({ wallet, onUpdated, disabled }: Props) {
  const [input, setInput] = useState(wallet.address ?? '');
  const [showPasswordModal, setShowPasswordModal] = useState(false);
  const [unlockPassword, setUnlockPassword] = useState('');
  const [pendingAddress, setPendingAddress] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const hasBoundAddress = Boolean(wallet.address);
  const actionText = hasBoundAddress ? '变更地址' : '绑定地址';

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
            const nextAddress = input.trim();
            if (!nextAddress) {
              setError('请输入手续费收款钱包地址');
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
                onClick={async () => {
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
                    setUnlockPassword('');
                    setPendingAddress(null);
                    setError(null);
                  } catch (e) {
                    setError(e instanceof Error ? e.message : String(e));
                  } finally {
                    setSaving(false);
                  }
                }}
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
