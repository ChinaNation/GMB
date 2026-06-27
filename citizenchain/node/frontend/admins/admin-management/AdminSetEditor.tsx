import { useState } from 'react';
import { hexToSs58 } from '../../shared/ss58';

type Props = {
  admins: string[];
  disabled?: boolean;
  onChange: (admins: string[]) => void;
};

const normalizeHex = (value: string) => value.trim().replace(/^0x/i, '').toLowerCase();

export function AdminSetEditor({ admins, disabled, onChange }: Props) {
  const [draft, setDraft] = useState('');

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
          <div className="admin-set-row" key={admin}>
            <span>{index + 1}</span>
            <code>{hexToSs58(admin)}</code>
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

