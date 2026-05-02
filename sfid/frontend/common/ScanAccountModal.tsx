// 通用"扫码识别账户"弹窗。调用摄像头扫描 WUMIN_QR_V1 用户码,
// 从中提取 body.address(SS58)回填。供 密钥管理 / 新增市级管理员 / 更换省级管理员 等场景复用。
// 使用统一的 BarcodeDetector 方案(cameraScanner.ts),与登录/绑定/CPMS 等场景一致。

import { useCallback, useEffect, useRef, useState } from 'react';
import { Button, Modal, Typography } from 'antd';
import { decodeSs58 } from '../utils/ss58';
import { parseQrEnvelope, QrParseError } from '../qr/wuminQr';
import { startCameraScanner } from '../utils/cameraScanner';

export function ScanAccountModal(props: {
  open: boolean;
  onClose: () => void;
  onResolved: (ss58Address: string) => void;
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
          if (env.kind !== 'user_contact' && env.kind !== 'user_transfer' && env.kind !== 'user_duoqian') {
            setError('不是用户码二维码');
            return;
          }
          const addr = String((env.body as { address?: string }).address || '').trim();
          if (!addr) {
            setError('用户码未携带 address 字段');
            return;
          }
          try {
            decodeSs58(addr);
          } catch (e) {
            setError(e instanceof Error ? e.message : 'SS58 校验失败');
            return;
          }
          // 识别成功,停止扫描
          if (cleanupRef.current) {
            cleanupRef.current();
            cleanupRef.current = null;
          }
          props.onResolved(addr);
        } catch (e) {
          if (e instanceof QrParseError) {
            setError(e.message);
          } else {
            setError('二维码不是有效 WUMIN_QR_V1 格式');
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
