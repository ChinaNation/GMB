// 中文注释:管理员 Passkey 更新工具。它只服务当前登录管理员本人;
// 生成/重新生成时先由本人冷钱包确认,再创建浏览器 Passkey 并落库。

import { useCallback, useState } from 'react';
import { Badge, Button, message } from 'antd';
import type { ButtonProps } from 'antd';
import { useAuth } from '../hooks/useAuth';
import { writeStoredAuth } from '../utils/storedAuth';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';
import { WuminSignatureModal } from '../core/WuminSignatureModal';
import {
  confirmPasskeyRegistration,
  completePasskeyRegistration,
  createPasskeyCredential,
  startPasskeyRegistration,
  type PasskeyStartOutput,
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
  const [passkeyStart, setPasskeyStart] = useState<PasskeyStartOutput | null>(null);
  const [loading, setLoading] = useState(false);
  // 中文注释:首次登录且未绑定 Passkey 时,只在“本人”的可点击更新密钥按钮上提示红点。
  const showRequiredBadge = auth?.passkey_bound === false && !disabled;
  const requiredBadgeDot = (
    <span
      style={{
        display: 'block',
        width: 12,
        height: 12,
        borderRadius: '50%',
        background: '#ff4d4f',
        boxShadow: '0 0 0 2px #fff',
      }}
    />
  );

  const openRegistration = async () => {
    if (!auth) return;
    setLoading(true);
    try {
      const start = await startPasskeyRegistration(auth);
      setPasskeyStart(start);
    } catch (error) {
      message.error(error instanceof Error ? error.message : 'Passkey 更新失败');
    } finally {
      setLoading(false);
    }
  };

  const handleSignedResponse = useCallback(async (raw: string) => {
    if (!auth || !passkeyStart) return;
    setLoading(true);
    try {
      const signed = parseSignedReceiptPayload(raw, passkeyStart.request_id);
      if (signed.challenge_id !== passkeyStart.request_id) {
        throw new Error('签名回执与当前 Passkey 请求不匹配');
      }
      if (!signed.signer_pubkey || !signed.payload_hash) {
        throw new Error('签名回执缺少 signer_pubkey 或 payload_hash');
      }
      const confirmed = await confirmPasskeyRegistration(auth, {
        registration_id: passkeyStart.registration_id,
        signer_pubkey: signed.signer_pubkey,
        signature: signed.signature,
        payload_hash: signed.payload_hash,
      });
      const credential = await createPasskeyCredential(confirmed.public_key_options);
      await completePasskeyRegistration(auth, {
        registration_id: confirmed.registration_id,
        credential,
      });
      const nextAuth = { ...auth, passkey_bound: true };
      setAuth(nextAuth);
      writeStoredAuth(nextAuth);
      message.success('Passkey 已更新');
      setPasskeyStart(null);
      onCompleted?.();
    } catch (error) {
      message.error(error instanceof Error ? error.message : 'Passkey 签名回执处理失败');
    } finally {
      setLoading(false);
    }
  }, [auth, onCompleted, passkeyStart, setAuth]);

  const updateButton = (
    <Button
      size={size}
      type={type}
      disabled={disabled}
      loading={loading}
      onClick={() => void openRegistration()}
    >
      {buttonText}
    </Button>
  );

  return (
    <>
      {showRequiredBadge ? (
        <Badge count={requiredBadgeDot} offset={[-2, 2]}>
          {updateButton}
        </Badge>
      ) : updateButton}
      <WuminSignatureModal
        title="Passkey 冷钱包确认"
        open={!!passkeyStart}
        onCancel={() => {
          setPasskeyStart(null);
        }}
        qrTitle="Passkey 签名二维码"
        qrValue={passkeyStart?.sign_request}
        qrHint={
          passkeyStart
            ? `有效期至 ${new Date(passkeyStart.expires_at * 1000).toLocaleTimeString()}`
            : '请先发起 Passkey 更新'
        }
        scannerHint="使用当前管理员冷钱包扫码签名后，再扫描签名回执"
        scannerDisabled={loading}
        scannerLoading={loading}
        onDetected={handleSignedResponse}
        onScannerError={(msg) => message.error(msg)}
      />
    </>
  );
}
