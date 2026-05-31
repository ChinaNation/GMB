// 中文注释:冷钱包签名弹窗外壳。内容统一交给 WuminSignaturePanel,
// 避免登录、Passkey、管理员动作各自维护不同的扫码界面。

import { Modal } from 'antd';
import type { WuminSignaturePanelProps } from './WuminSignaturePanel';
import { WuminSignaturePanel } from './WuminSignaturePanel';

export interface WuminSignatureModalProps extends WuminSignaturePanelProps {
  open: boolean;
  title: string;
  onCancel: () => void;
}

export function WuminSignatureModal({
  open,
  title,
  onCancel,
  ...panelProps
}: WuminSignatureModalProps) {
  return (
    <Modal
      title={<span style={{ display: 'block', textAlign: 'center', fontWeight: 600 }}>{title}</span>}
      open={open}
      onCancel={onCancel}
      footer={null}
      destroyOnClose
      width={760}
    >
      <WuminSignaturePanel {...panelProps} />
    </Modal>
  );
}
