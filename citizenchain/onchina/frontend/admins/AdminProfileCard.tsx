import type { ReactNode } from 'react';
import { tryEncodeSs58 } from '../utils/ss58';

export type AdminProfileLike = {
  admin_account: string;
  admin_cid_number?: string | null;
  name?: string | null;
  admin_role?: string | null;
  term_start?: number | null;
  term_end?: number | null;
  source_label?: string | null;
  balance_fen?: string | null;
};

type Props = {
  profile: AdminProfileLike;
  index?: number;
  action?: ReactNode;
  status?: ReactNode;
};

function formatDay(day?: number | null): string {
  if (!day) return '';
  const date = new Date(day * 86400 * 1000);
  const year = date.getUTCFullYear();
  const month = `${date.getUTCMonth() + 1}`.padStart(2, '0');
  const dayOfMonth = `${date.getUTCDate()}`.padStart(2, '0');
  return `${year}-${month}-${dayOfMonth}`;
}

function termText(profile: AdminProfileLike): string {
  const start = formatDay(profile.term_start);
  const end = formatDay(profile.term_end);
  if (!start && !end) return '';
  return `${start} ~ ${end}`;
}

function formatBalanceFen(fen?: string | null): string {
  if (fen == null) return '';
  try {
    const value = BigInt(fen);
    const negative = value < 0n;
    const abs = negative ? -value : value;
    const yuan = abs / 100n;
    const cents = abs % 100n;
    const yuanText = yuan.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
    return `${negative ? '-' : ''}${yuanText}.${cents.toString().padStart(2, '0')} 元`;
  } catch {
    return '';
  }
}

function Field({ label, value }: { label: string; value: string; wide?: boolean }) {
  return (
    <div style={{
      display: 'grid',
      gridTemplateColumns: 'max-content minmax(0, 1fr)',
      gap: 4,
      alignItems: 'start',
      minHeight: 20,
    }}>
      <span style={{ color: '#93a4b8', fontSize: 12, lineHeight: 1.45, whiteSpace: 'nowrap' }}>{label}:</span>
      <span style={{
        color: '#e5edf6',
        fontSize: 12,
        lineHeight: 1.45,
        wordBreak: 'break-all',
        minHeight: 18,
        fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Consolas, monospace',
      }}>
        {value}
      </span>
    </div>
  );
}

export function AdminProfileCard({ profile, index, action, status }: Props) {
  const topAction = action ?? status;
  const accountText = profile.admin_account ? tryEncodeSs58(profile.admin_account) : '';
  // 字段标签固定显示;字段值为空时只保留空值区域,不显示兜底文案。
  return (
    <div style={{
      display: 'grid',
      gap: 8,
      minWidth: 0,
      padding: 12,
      borderRadius: 8,
      border: '1px solid rgba(148, 163, 184, 0.28)',
      background: 'rgba(15, 23, 42, 0.92)',
      boxShadow: '0 8px 20px rgba(15, 23, 42, 0.12)',
    }}>
      <div style={{ display: 'flex', minHeight: 28, alignItems: 'center', justifyContent: 'space-between', gap: 10 }}>
        {index != null ? (
          <span style={{
            display: 'inline-flex',
            minWidth: 28,
            height: 28,
            alignItems: 'center',
            justifyContent: 'center',
            borderRadius: 8,
            background: 'rgba(96, 165, 250, 0.14)',
            color: '#bfdbfe',
            fontSize: 12,
            fontWeight: 700,
          }}>
            {index}
          </span>
        ) : <span />}
        <span style={{ display: 'inline-flex', minHeight: 28, alignItems: 'center', justifyContent: 'flex-end', marginLeft: 'auto' }}>
          {topAction}
        </span>
      </div>
      <div style={{ display: 'grid', gap: 6 }}>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, minmax(0, 1fr))', gap: 10 }}>
          <Field label="姓名" value={profile.name ?? ''} />
          <Field label="职务" value={profile.admin_role ?? ''} />
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, minmax(0, 1fr))', gap: 10 }}>
          <Field label="任期" value={termText(profile)} />
          <Field label="来源" value={profile.source_label ?? ''} />
        </div>
        <Field label="身份CID" value={profile.admin_cid_number ?? ''} wide />
        <Field label="账户" value={accountText} wide />
        <Field label="余额" value={formatBalanceFen(profile.balance_fen)} wide />
      </div>
    </div>
  );
}
