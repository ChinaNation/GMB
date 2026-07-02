// 扫码上链签名弹窗(发起/表决共用)。后端返回的 sign_request 渲染成二维码,
// 由公民钱包扫码冷签并提交上链;onchina 侧到"产出 QR"为止。

import React from 'react';
import { Modal, QRCode } from 'antd';

interface Props {
  /** 后端返回的 sign_request;为 null 时弹窗关闭。 */
  signRequest: string | null;
  onClose: () => void;
  title?: string;
}

export function SignRequestModal({ signRequest, onClose, title = '扫码签名并提交上链' }: Props) {
  return (
    <Modal
      open={signRequest !== null}
      title={title}
      onCancel={onClose}
      onOk={onClose}
      okText="完成"
      cancelButtonProps={{ style: { display: 'none' } }}
    >
      <div style={{ textAlign: 'center' }}>
        {signRequest !== null && <QRCode value={signRequest} size={240} />}
        <p style={{ marginTop: 12, color: 'rgba(0,0,0,0.55)' }}>
          用公民钱包扫描二维码,冷签后由钱包提交上链。
        </p>
      </div>
    </Modal>
  );
}
