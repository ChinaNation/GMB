import { useMemo, useState } from 'react';
import { Button, Card, Space, Typography, message } from 'antd';

import { FilingConfirmModal } from './FilingConfirmModal';
import { FilingStatusBadge } from './FilingStatusBadge';
import type { InstitutionFilingPayload, InstitutionFilingStatus } from './types';

interface InstitutionFilingPanelProps {
  sfidId: string;
  institutionName?: string | null;
  accountName?: string | null;
  status?: InstitutionFilingStatus;
  submitting?: boolean;
  onSubmit?: (payload: InstitutionFilingPayload) => Promise<void> | void;
}

export function InstitutionFilingPanel({
  sfidId,
  institutionName,
  accountName,
  status = 'NOT_FILED',
  submitting = false,
  onSubmit,
}: InstitutionFilingPanelProps) {
  const [confirmOpen, setConfirmOpen] = useState(false);
  const payload = useMemo<InstitutionFilingPayload | null>(() => {
    const name = institutionName?.trim();
    const account = accountName?.trim();
    if (!sfidId || !name || !account) return null;
    return {
      sfid_id: sfidId,
      institution_name: name,
      account_name: account,
    };
  }, [accountName, institutionName, sfidId]);

  const handleConfirm = async () => {
    if (!payload || !onSubmit) return;
    try {
      await onSubmit(payload);
      setConfirmOpen(false);
    } catch (err) {
      message.error(err instanceof Error ? err.message : '备案提交失败');
    }
  };

  return (
    <Card size="small" title="DUOQIAN 机构备案">
      <Space direction="vertical" size={12} style={{ width: '100%' }}>
        <Space>
          <Typography.Text type="secondary">备案状态</Typography.Text>
          <FilingStatusBadge status={status} />
        </Space>
        <Button
          type="primary"
          disabled={!payload || !onSubmit || status === 'FILED_ON_CHAIN'}
          loading={submitting}
          onClick={() => setConfirmOpen(true)}
        >
          备案上链
        </Button>
      </Space>
      <FilingConfirmModal
        open={confirmOpen}
        payload={payload}
        submitting={submitting}
        onCancel={() => setConfirmOpen(false)}
        onConfirm={handleConfirm}
      />
    </Card>
  );
}
