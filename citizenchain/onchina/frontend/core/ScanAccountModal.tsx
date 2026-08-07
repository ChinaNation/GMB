// 通用“扫码识别账户”弹窗。这里要的是「一个账户」，因此只接受 QR_V1 `k=5` 账户码，
// 其 body.account_id 本身就是规范 account_id，直接回填给业务表单。
// 用户码(k=3)表达「人」、收款码(k=4)表达「一笔收款请求」，都不是账户声明，一律拒绝。
// 使用统一的 BarcodeDetector 方案(cameraScanner.ts),与登录和扫码签名场景一致。

import { useCallback, useEffect, useRef, useState } from 'react';
import { Button, Modal, Typography } from 'antd';
import { parseQrEnvelope, QrParseError, type AccountIdCodeBody } from './citizenQr';
import { startCameraScanner } from '../utils/cameraScanner';
import { CID_MODAL_Z_INDEX } from './modalStack';

export function ScanAccountModal(props: {
  open: boolean;
  onClose: () => void;
  onResolved: (account_id: string) => void;
}) {
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const cleanupRef = useRef<(() => void) | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [videoMounted, setVideoMounted] = useState(false);
  const attachVideo = useCallback((el: HTMLVideoElement | null) => {
    videoRef.current = el;
    setVideoMounted(Boolean(el));
  }, []);

  useEffect(() => {
    if (!props.open) {
      setVideoMounted(false);
      setError(null);
    }
  }, [props.open]);

  useEffect(() => {
    if (!props.open || !videoMounted) return;
    const video = videoRef.current;
    if (!video) return;
    setError(null);

    const cleanup = startCameraScanner(
      video,
      (raw) => {
        try {
          const env = parseQrEnvelope(raw);
          if (env.kind !== 'account_id_code') {
            setError('请扫描账户码（钱包 → 账户详情右上角二维码）');
            return;
          }
          const account_id = (env.body as AccountIdCodeBody).account_id;
          // 识别成功,停止扫描
          if (cleanupRef.current) {
            cleanupRef.current();
            cleanupRef.current = null;
          }
          props.onResolved(account_id);
        } catch (e) {
          if (e instanceof QrParseError) {
            setError(e.message);
          } else {
            setError('二维码不是有效 QR_V1 格式');
          }
        }
      },
      () => {
        // camera ready — 无需额外操作
      },
      (msg) => {
        setError(msg);
      },
    );
    cleanupRef.current = cleanup;

    return () => {
      if (cleanupRef.current) {
        cleanupRef.current();
        cleanupRef.current = null;
      }
    };
  }, [props.open, videoMounted]);

  return (
    <Modal
      title={<div style={{ textAlign: 'center', width: '100%' }}>扫描账户码</div>}
      open={props.open}
      onCancel={props.onClose}
      footer={[
        <Button key="cancel" onClick={props.onClose}>
          取消
        </Button>,
      ]}
      destroyOnClose
      width={420}
      zIndex={CID_MODAL_Z_INDEX.accountScan}
    >
      <div
        style={{
          width: '100%',
          aspectRatio: '1 / 1',
          background: 'linear-gradient(145deg, #0f172a, #1e293b)',
          borderRadius: 12,
          overflow: 'hidden',
          position: 'relative',
        }}
      >
        <video
          ref={attachVideo}
          style={{ width: '100%', height: '100%', objectFit: 'cover' }}
          muted
          playsInline
        />
      </div>
      {error && (
        <Typography.Paragraph type="danger" style={{ marginTop: 12, marginBottom: 0 }}>
          {error}
        </Typography.Paragraph>
      )}
    </Modal>
  );
}
