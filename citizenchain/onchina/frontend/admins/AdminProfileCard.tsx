import type { ReactNode } from 'react';
import { Descriptions, Typography } from 'antd';
import { tryEncodeSs58 } from '../utils/ss58';

export type AdminProfileLike = {
  admin_account: string;
  province_name?: string | null;
  city_name?: string | null;
  institution_code?: string | null;
  admin_cid_number?: string | null;
  admin_name?: string | null;
  name?: string | null;
  admin_role?: string | null;
  term_start?: number | null;
  term_end?: number | null;
  origin_label?: string | null;
  balance_fen?: string | null;
  built_in?: boolean | null;
  created_by_name?: string | null;
  created_at?: string | null;
  updated_at?: string | null;
};

type Props = {
  profile: AdminProfileLike;
  index?: number;
  action?: ReactNode;
  status?: ReactNode;
  actionPlacement?: 'top' | 'balance-row';
};

function formatDay(day?: number | null): string {
  if (!day) return '';
  const date = new Date(day * 86400 * 1000);
  const year = date.getUTCFullYear();
  const month = `${date.getUTCMonth() + 1}`.padStart(2, '0');
  const dayOfMonth = `${date.getUTCDate()}`.padStart(2, '0');
  return `${year}-${month}-${dayOfMonth}`;
}

export function adminTermText(profile: AdminProfileLike): string {
  const start = formatDay(profile.term_start);
  const end = formatDay(profile.term_end);
  if (!start && !end) return '';
  return `${start} ~ ${end}`;
}

export function formatAdminBalanceFen(fen?: string | null): string {
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

export function adminDisplayName(profile: AdminProfileLike): string {
  return profile.name?.trim() || profile.admin_name?.trim() || '';
}

function Field({ label, value, trailing }: { label: string; value: string; wide?: boolean; trailing?: ReactNode }) {
  return (
    <div style={{
      display: 'grid',
      gridTemplateColumns: trailing ? 'max-content minmax(0, 1fr) auto' : 'max-content minmax(0, 1fr)',
      gap: trailing ? 8 : 4,
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
      {trailing ? (
        <span style={{ display: 'inline-flex', justifyContent: 'flex-end', alignItems: 'center' }}>
          {trailing}
        </span>
      ) : null}
    </div>
  );
}

export function AdminProfileCard({ profile, index, action, status, actionPlacement = 'top' }: Props) {
  const topAction = actionPlacement === 'top' ? action ?? status : status;
  const balanceAction = actionPlacement === 'balance-row' ? action : null;
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
          <Field label="任期" value={adminTermText(profile)} />
          <Field label="来源" value={profile.origin_label ?? ''} />
        </div>
        <Field label="身份CID" value={profile.admin_cid_number ?? ''} wide />
        <Field label="账户" value={accountText} wide />
        <Field label="余额" value={formatAdminBalanceFen(profile.balance_fen)} wide trailing={balanceAction} />
      </div>
    </div>
  );
}

export function AdminProfileDetails({
  profile,
  areaLabel,
  areaValue,
}: {
  profile: AdminProfileLike;
  areaLabel?: string;
  areaValue?: string | null;
}) {
  const ss58 = profile.admin_account ? tryEncodeSs58(profile.admin_account) : '';
  const shownName = adminDisplayName(profile);
  const createdAt = profile.created_at ? new Date(profile.created_at).toLocaleString('zh-CN') : '';
  const updatedAt = profile.updated_at ? new Date(profile.updated_at).toLocaleString('zh-CN') : '';

  return (
    <Descriptions column={1} size="small" bordered>
      {areaLabel ? <Descriptions.Item label={areaLabel}>{areaValue || '-'}</Descriptions.Item> : null}
      <Descriptions.Item label="姓名">{shownName || '-'}</Descriptions.Item>
      <Descriptions.Item label="职务">{profile.admin_role || '-'}</Descriptions.Item>
      <Descriptions.Item label="任期">{adminTermText(profile) || '-'}</Descriptions.Item>
      <Descriptions.Item label="来源">{profile.origin_label || '-'}</Descriptions.Item>
      <Descriptions.Item label="身份CID">
        {profile.admin_cid_number ? (
          <Typography.Text style={{ wordBreak: 'break-all' }}>{profile.admin_cid_number}</Typography.Text>
        ) : '-'}
      </Descriptions.Item>
      <Descriptions.Item label="账户">
        {ss58 ? <Typography.Text style={{ wordBreak: 'break-all' }}>{ss58}</Typography.Text> : '-'}
      </Descriptions.Item>
      <Descriptions.Item label="账户Hex">
        {profile.admin_account ? (
          <Typography.Text style={{ wordBreak: 'break-all' }}>{profile.admin_account}</Typography.Text>
        ) : '-'}
      </Descriptions.Item>
      <Descriptions.Item label="余额">{formatAdminBalanceFen(profile.balance_fen) || '-'}</Descriptions.Item>
      {profile.institution_code ? (
        <Descriptions.Item label="机构码">{profile.institution_code}</Descriptions.Item>
      ) : null}
      {profile.created_by_name ? (
        <Descriptions.Item label="登记管理员">{profile.created_by_name}</Descriptions.Item>
      ) : null}
      {profile.built_in != null ? (
        <Descriptions.Item label="内置管理员">{profile.built_in ? '是' : '否'}</Descriptions.Item>
      ) : null}
      {createdAt ? <Descriptions.Item label="创建时间">{createdAt}</Descriptions.Item> : null}
      {updatedAt ? <Descriptions.Item label="更新时间">{updatedAt}</Descriptions.Item> : null}
    </Descriptions>
  );
}
