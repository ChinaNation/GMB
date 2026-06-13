// 中文注释:公民钱包签名弹窗外壳。内容统一交给 WuminSignaturePanel,
// 避免登录、Passkey、管理员动作各自维护不同的扫码界面。

import { Modal } from 'antd';
import type { WuminSignaturePanelProps } from './WuminSignaturePanel';
import { WuminSignaturePanel } from './WuminSignaturePanel';
import { SFID_MODAL_Z_INDEX } from './modalStack';

export interface WuminSignatureModalProps extends WuminSignaturePanelProps {
  open: boolean;
  title: string;
  onCancel: () => void;
  zIndex?: number;
}

export function WuminSignatureModal({
  open,
  title,
  onCancel,
  zIndex = SFID_MODAL_Z_INDEX.securitySignature,
  ...panelProps
}: WuminSignatureModalProps) {
  return (
    <Modal
      title={<span style={{ display: 'block', textAlign: 'center', fontWeight: 600 }}>{title}</span>}
      open={open}
      onCancel={onCancel}
      footer={null}
      destroyOnClose
      maskClosable={false}
      keyboard={false}
      width={760}
      zIndex={zIndex}
    >
      <WuminSignaturePanel {...panelProps} />
    </Modal>
  );
}
