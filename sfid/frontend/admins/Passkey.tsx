// 中文注释:管理员 Passkey 更新工具。它只服务当前登录管理员本人;
// 生成/重新生成时必须先完成 WebAuthn,再用本人冷钱包 sr25519 签名确认。

import { useCallback, useEffect, useRef, useState } from 'react';
import { Button, Modal, QRCode, Space, Typography, message } from 'antd';
import type { ButtonProps } from 'antd';
import { useAuth } from '../hooks/useAuth';
import { writeStoredAuth } from '../utils/storedAuth';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';
import { startCameraScanner } from '../utils/cameraScanner';
import {
  attestPasskeyRegistration,
  completePasskeyRegistration,
  createPasskeyCredential,
  startPasskeyRegistration,
  type PasskeyAttestOutput,
} from './admin_security_api';

export interface PasskeyProps {
  buttonText?: string;
  disabled?: boolean;
  size?: ButtonProps['size'];
  type?: ButtonProps['type'];
  onCompleted?: () => void;
}

export function Passkey({
  buttonText = '更新密钥',
  disabled = false,
  size,
  type,
  onCompleted,
}: PasskeyProps) {
  const { auth, setAuth } = useAuth();
  const [passkeyAttest, setPasskeyAttest] = useState<PasskeyAttestOutput | null>(null);
  const [loading, setLoading] = useState(false);
  const [scannerActive, setScannerActive] = useState(false);
  const [scannerReady, setScannerReady] = useState(false);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const cleanupRef = useRef<(() => void) | null>(null);

  const stopScanner = useCallback(() => {
    if (cleanupRef.current) {
      cleanupRef.current();
      cleanupRef.current = null;
    }
    setScannerReady(false);
    setScannerActive(false);
  }, []);

  useEffect(() => () => stopScanner(), [stopScanner]);

  useEffect(() => {
    if (!scannerActive || !videoRef.current) return;
    cleanupRef.current = startCameraScanner(
      videoRef.current,
      (raw) => {
        stopScanner();
        void handleSignedResponse(raw);
      },
      () => setScannerReady(true),
      (msg) => {
        message.error(msg);
        stopScanner();
      },
    );
    return () => stopScanner();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scannerActive, stopScanner]);

  const openRegistration = async () => {
    if (!auth) return;
    setLoading(true);
    try {
      const start = await startPasskeyRegistration(auth);
      const credential = await createPasskeyCredential(start.public_key_options);
      const attest = await attestPasskeyRegistration(auth, start.registration_id, credential);
      setPasskeyAttest(attest);
    } catch (error) {
      message.error(error instanceof Error ? error.message : 'Passkey 更新失败');
    } finally {
      setLoading(false);
    }
  };

  const handleSignedResponse = async (raw: string) => {
    if (!auth || !passkeyAttest) return;
    setLoading(true);
    try {
      const signed = parseSignedReceiptPayload(raw, passkeyAttest.request_id);
      if (signed.challenge_id !== passkeyAttest.request_id) {
        throw new Error('签名回执与当前 Passkey 请求不匹配');
      }
      if (!signed.signer_pubkey || !signed.payload_hash) {
        throw new Error('签名回执缺少 signer_pubkey 或 payload_hash');
      }
      await completePasskeyRegistration(auth, {
        registration_id: passkeyAttest.registration_id,
        signer_pubkey: signed.signer_pubkey,
        signature: signed.signature,
        payload_hash: signed.payload_hash,
      });
      const nextAuth = { ...auth, passkey_bound: true };
      setAuth(nextAuth);
      writeStoredAuth(nextAuth);
      message.success('Passkey 已更新');
      setPasskeyAttest(null);
      onCompleted?.();
    } catch (error) {
      message.error(error instanceof Error ? error.message : 'Passkey 签名回执处理失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      <Button
        size={size}
        type={type}
        disabled={disabled}
        loading={loading}
        onClick={() => void openRegistration()}
      >
        {buttonText}
      </Button>

      <Modal
        title="Passkey 冷钱包确认"
        open={!!passkeyAttest}
        onCancel={() => {
          stopScanner();
          setPasskeyAttest(null);
        }}
        footer={null}
        destroyOnClose
      >
        {passkeyAttest ? (
          <Space direction="vertical" size={12} style={{ width: '100%', alignItems: 'center' }}>
            <Typography.Text type="secondary">使用当前管理员冷钱包扫码签名。</Typography.Text>
            <QRCode value={passkeyAttest.sign_request} size={260} color="#134e4a" />
            <div style={{ position: 'relative', width: '100%', aspectRatio: '4 / 3', background: '#0f172a', borderRadius: 8, overflow: 'hidden' }}>
              <video ref={videoRef} muted playsInline style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
              {!scannerReady ? (
                <div style={{ position: 'absolute', inset: 0, display: 'grid', placeItems: 'center', color: '#e5e7eb' }}>
                  {scannerActive ? '摄像头初始化中...' : '摄像头未开启'}
                </div>
              ) : null}
            </div>
            <Button onClick={() => setScannerActive((v) => !v)} loading={loading}>
              {scannerActive ? '停止扫码' : '开启扫码'}
            </Button>
          </Space>
        ) : null}
      </Modal>
    </>
  );
}
