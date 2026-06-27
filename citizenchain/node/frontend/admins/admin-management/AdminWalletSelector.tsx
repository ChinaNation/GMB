import { hexToSs58 } from '../../shared/ss58';
import type { AdminWalletMatch } from '../../governance/types';

type Props = {
  wallets: AdminWalletMatch[];
  value: AdminWalletMatch | null;
  disabled?: boolean;
  onChange: (wallet: AdminWalletMatch | null) => void;
};

export function AdminWalletSelector({ wallets, value, disabled, onChange }: Props) {
  return (
    <div className="wallet-form-field">
      <label>发起管理员</label>
      <select
        value={value?.pubkeyHex || ''}
        disabled={disabled || wallets.length <= 1}
        onChange={(e) => onChange(wallets.find((w) => w.pubkeyHex === e.target.value) || null)}
      >
        {wallets.length === 0 && <option value="">无已激活管理员</option>}
        {wallets.length > 1 && <option value="">请选择…</option>}
        {wallets.map((wallet) => (
          <option key={wallet.pubkeyHex} value={wallet.pubkeyHex}>
            {wallet.walletLabel || hexToSs58(wallet.pubkeyHex)}
          </option>
        ))}
      </select>
    </div>
  );
}
