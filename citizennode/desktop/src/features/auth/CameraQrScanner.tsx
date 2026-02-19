import { BrowserQRCodeReader, type IScannerControls } from '@zxing/browser';
import { Button, Space, Typography } from 'antd';
import { useEffect, useRef, useState } from 'react';

type CameraQrScannerProps = {
  onDetected: (payload: string) => void;
  enabled: boolean;
};

export function CameraQrScanner({ onDetected, enabled }: CameraQrScannerProps) {
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const controlsRef = useRef<IScannerControls | null>(null);
  const readerRef = useRef<BrowserQRCodeReader | null>(null);

  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const mediaDevicesSupported =
    typeof navigator !== 'undefined' &&
    typeof navigator.mediaDevices !== 'undefined' &&
    typeof navigator.mediaDevices.getUserMedia === 'function';

  const stop = () => {
    controlsRef.current?.stop();
    controlsRef.current = null;
    readerRef.current = null;

    const video = videoRef.current;
    if (video) {
      video.srcObject = null;
    }

    setRunning(false);
  };

  const start = async () => {
    const video = videoRef.current;
    if (!video) return;
    if (!mediaDevicesSupported) {
      setError('当前运行环境不支持摄像头接口（navigator.mediaDevices 不可用），请使用手动粘贴签名 JSON。');
      return;
    }

    try {
      setError(null);
      const reader = new BrowserQRCodeReader();
      readerRef.current = reader;
      const controls = await reader.decodeFromVideoDevice(undefined, video, (result) => {
        const text = result?.getText?.().trim();
        if (text) {
          onDetected(text);
          stop();
        }
      });
      controlsRef.current = controls;
      setRunning(true);
    } catch (e) {
      stop();
      setError(e instanceof Error ? e.message : '无法打开摄像头或识别二维码');
    }
  };

  useEffect(() => {
    if (!enabled) {
      stop();
      setError(null);
    }
    return () => {
      stop();
    };
  }, [enabled]);

  return (
    <Space direction="vertical" size={8} style={{ width: '100%' }}>
      <Space size={10}>
        <Typography.Text type="secondary">摄像头扫码输入签名</Typography.Text>
        {!running ? (
          <Button onClick={() => void start()} disabled={!enabled || !mediaDevicesSupported}>
            启动扫码
          </Button>
        ) : (
          <Button type="text" onClick={stop}>
            停止
          </Button>
        )}
      </Space>

      <video
        ref={videoRef}
        style={{ width: 280, height: 180, borderRadius: 8, background: '#0b1014' }}
        muted
      />

      {error ? <Typography.Text type="warning">{error}</Typography.Text> : null}
    </Space>
  );
}
