import { AdminProfileCard } from './AdminProfileCard';
import type { AdminProfileInfo } from './types';

type Props = {
  currentAdmins: string[];
  admins: string[];
  currentProfiles?: AdminProfileInfo[];
  balances?: Record<string, string | null | undefined>;
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

export function AdminSetDiff({ currentAdmins, admins, currentProfiles = [], balances = {} }: Props) {
  const current = new Set(currentAdmins.map(normalizeHex));
  const next = new Set(admins.map(normalizeHex));
  const added = admins.filter((item) => !current.has(normalizeHex(item)));
  const removed = currentAdmins.filter((item) => !next.has(normalizeHex(item)));
  const profileByAccount = new Map(currentProfiles.map((profile) => [normalizeHex(profile.account), profile]));

  return (
    <div className="admin-set-diff">
      <div>
        <strong>新增</strong>
        <div className="admin-set-diff-list">
          {added.length === 0 ? <p>无</p> : added.map((item, index) => (
            <AdminProfileCard
              key={item}
              profile={accountOnlyProfile(item)}
              index={index + 1}
              balanceFen={balances[normalizeHex(item)] ?? null}
            />
          ))}
        </div>
      </div>
      <div>
        <strong>移除</strong>
        <div className="admin-set-diff-list">
          {removed.length === 0 ? <p>无</p> : removed.map((item, index) => (
            <AdminProfileCard
              key={item}
              profile={profileByAccount.get(normalizeHex(item)) ?? accountOnlyProfile(item)}
              index={index + 1}
              balanceFen={balances[normalizeHex(item)] ?? null}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
