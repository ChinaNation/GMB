import type { ReactNode } from 'react';

import { formatBalance } from '../../shared/format';
import { hexToSs58 } from '../../shared/ss58';
import type { AdminProfileInfo } from './types';

type Props = {
  profile: AdminProfileInfo;
  index?: number;
  balanceFen?: string | null;
  className?: string;
  action?: ReactNode;
  status?: ReactNode;
};

function formatDay(day: number): string {
  if (!day) return '';
  const date = new Date(day * 86400 * 1000);
  const year = date.getUTCFullYear();
  const month = `${date.getUTCMonth() + 1}`.padStart(2, '0');
  const dayOfMonth = `${date.getUTCDate()}`.padStart(2, '0');
  return `${year}-${month}-${dayOfMonth}`;
}

function termText(profile: AdminProfileInfo): string {
  const start = formatDay(profile.termStart);
  const end = formatDay(profile.termEnd);
  if (!start && !end) return '';
  return `${start} ~ ${end}`;
}

function ProfileField({
  label,
  value,
  wide,
}: {
  label: string;
  value: string;
  wide?: boolean;
}) {
  return (
    <div className={`admin-profile-field ${wide ? 'admin-profile-field-wide' : ''}`}>
      <span className="admin-profile-label">{label}:</span>
      <span className="admin-profile-value">{value}</span>
    </div>
  );
}

export function AdminProfileCard({
  profile,
  index,
  balanceFen,
  className,
  action,
  status,
}: Props) {
  // 中文注释:字段标签固定渲染;字段值为空时只让值区域留空,不隐藏标签。
  const balanceText = balanceFen != null ? formatBalance(balanceFen) : '';
  const topAction = action ?? status;
  const accountText = profile.account ? hexToSs58(profile.account) : '';

  return (
    <div className={`metric-card admin-profile-card ${className ?? ''}`}>
      <div className="admin-profile-top">
        {index != null ? <span className="admin-card-index">{index}</span> : null}
        <div className="admin-profile-top-action">{topAction}</div>
      </div>
      <div className="admin-profile-fields">
        <div className="admin-profile-field-pair">
          <ProfileField label="姓名" value={profile.name} />
          <ProfileField label="职务" value={profile.adminRole} />
        </div>
        <div className="admin-profile-field-pair">
          <ProfileField label="任期" value={termText(profile)} />
          <ProfileField label="来源" value={profile.sourceLabel} />
        </div>
        <ProfileField label="身份CID" value={profile.adminCidNumber} wide />
        <ProfileField label="账户" value={accountText} wide />
        <ProfileField label="余额" value={balanceText} wide />
      </div>
    </div>
  );
}
