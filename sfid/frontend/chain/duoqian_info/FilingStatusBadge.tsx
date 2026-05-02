import { Tag } from 'antd';

import type { InstitutionFilingStatus } from './types';

const STATUS_META: Record<InstitutionFilingStatus, { color: string; label: string }> = {
  NOT_FILED: { color: 'default', label: '未备案' },
  FILING_PENDING: { color: 'processing', label: '备案中' },
  FILED_ON_CHAIN: { color: 'success', label: '已备案' },
  FILING_FAILED: { color: 'error', label: '备案失败' },
};

export function FilingStatusBadge({ status }: { status: InstitutionFilingStatus }) {
  const meta = STATUS_META[status];
  return <Tag color={meta.color}>{meta.label}</Tag>;
}
