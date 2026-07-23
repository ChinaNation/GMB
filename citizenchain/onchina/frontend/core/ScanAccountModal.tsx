// 通用“扫码识别账户”弹窗。扫描 QR_V1 用户码后把 body.address(SS58)
// 立即转换成规范 account_id，再回填给业务表单。
// 使用统一的 BarcodeDetector 方案(cameraScanner.ts),与登录和扫码签名场景一致。

import { useCallback, useEffect, useRef, useState } from 'react';
import { Button, Modal, Typography } from 'antd';
import { decodeSs58 } from '../utils/ss58';
import { parseQrEnvelope, QrParseError } from './citizenQr';
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
          if (env.kind !== 'user_contact' && env.kind !== 'user_transfer') {
            setError('不是用户码二维码');
            return;
          }
          const addr = String((env.body as { address?: string }).address || '').trim();
          if (!addr) {
            setError('用户码未携带 address 字段');
            return;
          }
          let account_id: string;
          try {
            account_id = decodeSs58(addr);
          } catch (e) {
            setError(e instanceof Error ? e.message : 'SS58 校验失败');
            return;
          }
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
      title={<div style={{ textAlign: 'center', width: '100%' }}>扫描用户码</div>}
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
