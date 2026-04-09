// 中文注释:从 App.tsx 迁移(任务卡 20260408-sfid-frontend-app-tsx-split 步 2)
// 通用"扫码识别账户"弹窗。调用摄像头扫描 WUMIN_USER_V1.0.0 用户码,
// 从中提取 `address` 字段(SS58)回填。供 密钥管理 / 新增市级管理员 / 更换省级管理员 等场景复用。

import { useCallback, useEffect, useRef, useState } from 'react';
import { Button, Modal, Typography } from 'antd';
import QrScanner from 'qr-scanner';
import { decodeSs58 } from '../utils/ss58';

/// 区块链全链统一的"用户协议"二维码 proto 标识。
/// 详见:wuminapp/lib/qr/qr_protocols.dart
const WUMIN_USER_PROTOCOL = 'WUMIN_USER_V1.0.0';

export function ScanAccountModal(props: {
  open: boolean;
  onClose: () => void;
  onResolved: (ss58Address: string) => void;
}) {
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const scannerRef = useRef<QrScanner | null>(null);
  const [error, setError] = useState<string | null>(null);
  // Antd Modal 的 destroyOnClose 会在 open=true 之后异步挂载内容,
  // 首次 useEffect 执行时 videoRef.current 可能仍为 null,
  // 所以用回调 ref + videoMounted state 等真实 <video> 元素挂上后再启动 scanner。
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
    let cancelled = false;
    const scanner = new QrScanner(
      video,
      (result) => {
        if (cancelled) return;
        try {
          const decoded = JSON.parse(result.data);
          if (decoded?.proto !== WUMIN_USER_PROTOCOL) {
            setError('不是用户协议二维码');
            return;
          }
          const addr = String(decoded.address || '').trim();
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
          scanner.stop();
          props.onResolved(addr);
        } catch {
          setError('二维码不是有效 JSON');
        }
      },
      { highlightScanRegion: true, highlightCodeOutline: true, returnDetailedScanResult: true },
    );
    scannerRef.current = scanner;
    scanner.start().catch((err) => {
      if (cancelled) return;
      setError(err instanceof Error ? err.message : '摄像头初始化失败');
    });
    return () => {
      cancelled = true;
      const s = scannerRef.current;
      if (s) {
        s.stop();
        s.destroy();
        scannerRef.current = null;
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
