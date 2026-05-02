import { Alert, Descriptions, Modal } from 'antd';

import type { InstitutionFilingPayload } from './types';

interface FilingConfirmModalProps {
  open: boolean;
  payload: InstitutionFilingPayload | null;
  submitting?: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}

export function FilingConfirmModal({
  open,
  payload,
  submitting = false,
  onCancel,
  onConfirm,
}: FilingConfirmModalProps) {
  return (
    <Modal
      title="确认备案上链"
      open={open}
      confirmLoading={submitting}
      okText="确认提交"
      cancelText="取消"
      onOk={onConfirm}
      onCancel={onCancel}
      okButtonProps={{ disabled: !payload }}
    >
      {payload && (
        <Descriptions column={1} size="small" bordered>
          <Descriptions.Item label="机构 SFID">{payload.sfid_id}</Descriptions.Item>
          <Descriptions.Item label="机构名称">{payload.institution_name}</Descriptions.Item>
          <Descriptions.Item label="机构账户名称">{payload.account_name}</Descriptions.Item>
        </Descriptions>
      )}
      <Alert
        style={{ marginTop: 16 }}
        type="info"
        showIcon
        message="备案只提交上述三项信息"
      />
    </Modal>
  );
}
