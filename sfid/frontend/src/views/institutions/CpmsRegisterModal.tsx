// 中文注释:CPMS 注册弹窗(摄像头扫码版)。
//
// 流程:CPMS 设备扫 QR1 安装后会显示一个 QR2 二维码,管理员在 sfid 前端
// 打开本弹窗用摄像头扫 QR2,后端校验 install_token 后 RSA 盲签名返回
// QR3 anonymous cert payload,弹窗里展示 QR3 二维码给 CPMS 设备扫描完成激活。
//
// 任务卡 `20260408-sfid-public-security-cpms-embed`:
// 老 App.tsx 里的"扫码注册机构"入口搬到这里,必须保持摄像头扫描,
// **不能**是粘贴文本。

import React, { useEffect, useRef, useState } from 'react';
import { Button, message, Modal, QRCode, Space, Typography } from 'antd';
import { QrcodeOutlined } from '@ant-design/icons';
import { registerCpms, type AdminAuth } from '../../api/client';
import { startCameraScanner } from '../../utils/cameraScanner';

interface Props {
  auth: AdminAuth;
  open: boolean;
  onClose: () => void;
  /** 注册成功拿到 QR3 payload 后的回调(由父组件触发刷新) */
  onRegistered: (qr3Payload: string) => void;
}

export const CpmsRegisterModal: React.FC<Props> = ({ auth, open, onClose, onRegistered }) => {
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const cleanupRef = useRef<(() => void) | null>(null);
  const [scannerReady, setScannerReady] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [qr3Text, setQr3Text] = useState('');

  // 中文注释:open=true 打开弹窗时自动启动摄像头;关闭或拿到 QR3 时停止。
  useEffect(() => {
    if (!open || qr3Text) {
      stopScanner();
      return;
    }
    // 等 video 元素挂载完
    const t = window.setTimeout(() => {
      if (!videoRef.current) return;
      cleanupRef.current = startCameraScanner(
        videoRef.current,
        (raw) => void onDetected(raw),
        () => setScannerReady(true),
        (msg) => message.error(msg),
      );
    }, 50);
    return () => {
      window.clearTimeout(t);
      stopScanner();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, qr3Text]);

  // 弹窗关闭时重置
  useEffect(() => {
    if (!open) {
      setQr3Text('');
      setSubmitting(false);
      setScannerReady(false);
    }
  }, [open]);

  const stopScanner = () => {
    if (cleanupRef.current) {
      cleanupRef.current();
      cleanupRef.current = null;
    }
    setScannerReady(false);
  };

  const onDetected = async (raw: string) => {
    stopScanner();
    setSubmitting(true);
    try {
      const result = await registerCpms(auth, { qr_payload: raw });
      setQr3Text(result.qr3_payload);
      message.success('CPMS 注册成功,已生成 QR3 匿名证书');
      onRegistered(result.qr3_payload);
    } catch (err) {
      message.error(err instanceof Error ? err.message : 'CPMS 注册失败');
      // 失败后不自动重开扫码;关闭弹窗用户重试
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal
      title="CPMS 站点注册(扫 QR2 → 返回 QR3)"
      open={open}
      onCancel={onClose}
      footer={[
        <Button key="close" onClick={onClose}>
          关闭
        </Button>,
      ]}
      destroyOnClose
      width={520}
    >
      {!qr3Text && (
        <>
          <Typography.Paragraph type="secondary" style={{ marginBottom: 12 }}>
            请将摄像头对准 CPMS 设备显示的 QR2 二维码,扫描后后端自动返回 QR3
            匿名证书二维码,请由 CPMS 设备扫描完成站点激活。
          </Typography.Paragraph>

          <div
            style={{
              width: '84%',
              maxWidth: 320,
              aspectRatio: '1 / 1',
              margin: '0 auto',
              background: 'linear-gradient(145deg, #0f172a, #1e293b)',
              borderRadius: 16,
              overflow: 'hidden',
              position: 'relative',
              border: '2px solid #334155',
              boxShadow: 'inset 0 2px 8px rgba(0,0,0,0.3)',
            }}
          >
            <div style={{ position: 'absolute', top: 8, left: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderTopLeftRadius: 4, zIndex: 2 }} />
            <div style={{ position: 'absolute', top: 8, right: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderTopRightRadius: 4, zIndex: 2 }} />
            <div style={{ position: 'absolute', bottom: 8, left: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderBottomLeftRadius: 4, zIndex: 2 }} />
            <div style={{ position: 'absolute', bottom: 8, right: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderBottomRightRadius: 4, zIndex: 2 }} />
            <video
              ref={videoRef}
              style={{ width: '100%', height: '100%', objectFit: 'cover' }}
              muted
              playsInline
            />
            {!scannerReady && !submitting && (
              <div
                style={{
                  position: 'absolute',
                  inset: 0,
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: 8,
                }}
              >
                <QrcodeOutlined style={{ fontSize: 32, color: 'rgba(255,255,255,0.25)' }} />
                <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
                  摄像头初始化中...
                </Typography.Text>
              </div>
            )}
            {submitting && (
              <div
                style={{
                  position: 'absolute',
                  inset: 0,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  background: 'rgba(0,0,0,0.5)',
                }}
              >
                <Typography.Text style={{ color: '#fff' }}>提交注册中...</Typography.Text>
              </div>
            )}
          </div>
        </>
      )}

      {qr3Text && (
        <Space direction="vertical" size={12} style={{ width: '100%', alignItems: 'center' }}>
          <Typography.Text strong>QR3 匿名证书(请由 CPMS 设备扫描)</Typography.Text>
          <div style={{ padding: 12, background: '#fff', borderRadius: 8 }}>
            <QRCode value={qr3Text} size={240} bordered={false} />
          </div>
        </Space>
      )}
    </Modal>
  );
};
