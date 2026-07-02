import { useState } from 'react';
import { AdminProfileCard } from './AdminProfileCard';
import type { AdminProfileInfo } from './types';

type Props = {
  admins: string[];
  profiles?: AdminProfileInfo[];
  balances?: Record<string, string | null | undefined>;
  disabled?: boolean;
  onChange: (admins: string[]) => void;
};

const normalizeHex = (value: string) => value.trim().replace(/^0x/i, '').toLowerCase();

function accountOnlyProfile(account: string): AdminProfileInfo {
  return {
    account,
    adminCidNumber: '',
    name: '',
    adminRole: '',
    termStart: 0,
    termEnd: 0,
    source: 255,
    sourceLabel: '',
  };
}

export function AdminSetEditor({ admins, profiles = [], balances = {}, disabled, onChange }: Props) {
  const [draft, setDraft] = useState('');
  const profileByAccount = new Map(profiles.map((profile) => [profile.account.toLowerCase(), profile]));

  const removeAdmin = (admin: string) => {
    onChange(admins.filter((item) => item !== admin));
  };

  const addAdmin = () => {
    const clean = normalizeHex(draft);
    if (!clean) return;
    if (clean.length !== 64 || !/^[0-9a-f]+$/.test(clean)) return;
    if (admins.some((item) => item.toLowerCase() === clean)) return;
    onChange([...admins, clean]);
    setDraft('');
  };

  return (
    <div className="admin-set-editor">
      <div className="admin-set-list">
        {admins.map((admin, index) => (
          <div className="admin-set-card-row" key={admin}>
            <AdminProfileCard
              profile={profileByAccount.get(admin.toLowerCase()) ?? accountOnlyProfile(admin)}
              index={index + 1}
              balanceFen={balances[normalizeHex(admin)] ?? null}
            />
            <button type="button" disabled={disabled} onClick={() => removeAdmin(admin)}>
              移除
            </button>
          </div>
        ))}
      </div>
      <div className="admin-set-add-row">
        <input
          value={draft}
          disabled={disabled}
          onChange={(e) => setDraft(e.target.value)}
          placeholder="输入 64 位管理员公钥 hex"
        />
        <button type="button" disabled={disabled || !draft.trim()} onClick={addAdmin}>
          添加
        </button>
      </div>
    </div>
  );
}
