// 中文注释:身份ID绑定弹窗。新增和更换共用扫码签名步骤,但 mode 分别提交 create/replace。
// 扫 CPMS ARCHIVE 档案码 -> 展示 wuminapp sign_request -> 扫 sign_response -> 提交 SFID。

import { useEffect, useRef, useState } from 'react';
import { Button, Descriptions, Modal, QRCode, Typography, Upload, message } from 'antd';
import type { UploadProps } from 'antd';

import type { AdminAuth } from '../auth/types';
import {
  citizenBind,
  citizenBindChallenge,
  type CitizenBindChallengeResult,
  type CitizenRow,
} from './api';
import { decodeQrImageFile, startCameraScanner } from '../utils/cameraScanner';
import { ApiError } from '../utils/http';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';

type BindStep = 'scan_archive_code' | 'sign_challenge' | 'scan_signature';

interface BindModalProps {
  auth: AdminAuth | null;
  open: boolean;
  record: CitizenRow | null;
  onClose: () => void;
  onBound: () => Promise<void> | void;
}

function bindErrorMessage(err: unknown): string {
  if (err instanceof ApiError) {
    switch (err.errorCode) {
      case 'SFID_BIND_CHALLENGE_EXPIRED':
        return '签名请求已过期，请重新扫描档案码。';
      case 'SFID_BIND_SIGNATURE_VERIFY_FAILED':
        return '签名校验失败，请确认使用档案码绑定的钱包签名。';
      case 'SFID_BIND_WALLET_MISMATCH':
        return '签名钱包与本次绑定钱包不一致。';
      case 'SFID_BIND_ARCHIVE_ALREADY_BOUND':
        return '该档案已完成电子护照绑定。';
      case 'SFID_BIND_ARCHIVE_IMMUTABLE':
        return '档案号已和当前身份ID永久绑定，更换钱包必须扫描同一档案号的档案码。';
      case 'SFID_BIND_WALLET_ALREADY_BOUND':
        return '该钱包已绑定其他电子护照。';
      default:
        return err.message;
    }
  }
  return err instanceof Error ? err.message : '绑定失败';
}

export function BindModal({ auth, open, record, onClose, onBound }: BindModalProps) {
  const [bindStep, setBindStep] = useState<BindStep>('scan_archive_code');
  const [bindChallenge, setBindChallenge] = useState<CitizenBindChallengeResult | null>(null);
  const [archiveCodeScanLoading, setArchiveCodeScanLoading] = useState(false);
  const [archiveCodeUploadLoading, setArchiveCodeUploadLoading] = useState(false);
  const [bindScannerActive, setBindScannerActive] = useState(false);
  const [bindScannerReady, setBindScannerReady] = useState(false);
  const bindVideoRef = useRef<HTMLVideoElement | null>(null);
  const bindScanCleanupRef = useRef<(() => void) | null>(null);

  const bindMode = record ? 'replace' : 'create';
  const modalTitle = record ? '更换绑定' : '新增身份ID绑定';

  const stopBindScanner = () => {
    if (bindScanCleanupRef.current) {
      bindScanCleanupRef.current();
      bindScanCleanupRef.current = null;
    }
    setBindScannerReady(false);
  };

  useEffect(() => {
    if (!open) return;
    setBindStep('scan_archive_code');
    setBindChallenge(null);
    setArchiveCodeUploadLoading(false);
    setBindScannerActive(false);
    stopBindScanner();
  }, [open, record?.id]);

  const onToggleBindScanner = () => {
    if (!open) return;
    if (bindScannerActive) {
      setBindScannerActive(false);
      stopBindScanner();
      return;
    }
    setBindScannerActive(true);
  };

  const onScanArchiveCode = async (qrPayload: string) => {
    if (!auth) return;
    if (!qrPayload.trim()) {
      message.error('二维码识别失败');
      return;
    }
    setArchiveCodeScanLoading(true);
    try {
      setBindScannerActive(false);
      stopBindScanner();
      const challenge = await citizenBindChallenge(auth, {
        mode: bindMode,
        archive_code_payload: qrPayload.trim(),
        citizen_id: record?.id,
      });
      setBindChallenge(challenge);
      setBindStep('sign_challenge');
    } catch (err) {
      message.error(bindErrorMessage(err));
    } finally {
      setArchiveCodeScanLoading(false);
    }
  };

  const onUploadArchiveCode: UploadProps['beforeUpload'] = async (file) => {
    if (!auth) {
      message.error('请先登录');
      return Upload.LIST_IGNORE;
    }
    setBindScannerActive(false);
    stopBindScanner();
    setArchiveCodeUploadLoading(true);
    try {
      const raw = await decodeQrImageFile(file as File);
      await onScanArchiveCode(raw);
    } catch (err) {
      message.error(err instanceof Error ? err.message : '二维码图片识别失败');
    } finally {
      setArchiveCodeUploadLoading(false);
    }
    return Upload.LIST_IGNORE;
  };

  const onScanBindSignature = async (raw: string) => {
    if (!auth || !bindChallenge) return;
    if (!raw.trim()) {
      message.error('签名二维码识别失败');
      return;
    }
    setArchiveCodeScanLoading(true);
    try {
      const payload = parseSignedReceiptPayload(raw.trim(), bindChallenge.challenge_id);
      if (!payload.signer_pubkey || !payload.payload_hash) {
        throw new Error('签名回执必须是 sign_response，并包含 pubkey 和 payload_hash');
      }
      setBindScannerActive(false);
      stopBindScanner();
      const result = await citizenBind(auth, {
        challenge_id: payload.challenge_id,
        pubkey: payload.signer_pubkey,
        signature: payload.signature,
        payload_hash: payload.payload_hash,
      });
      message.success(`${modalTitle}成功${result.sfid_code ? `，身份ID：${result.sfid_code}` : ''}`);
      onClose();
      await onBound();
    } catch (err) {
      message.error(bindErrorMessage(err));
    } finally {
      setArchiveCodeScanLoading(false);
    }
  };

  useEffect(() => {
    if (!open || !bindScannerActive || !bindVideoRef.current) {
      stopBindScanner();
      return;
    }
    const currentStep = bindStep;
    bindScanCleanupRef.current = startCameraScanner(
      bindVideoRef.current,
      (raw) => {
        setBindScannerActive(false);
        stopBindScanner();
        if (currentStep === 'scan_archive_code') {
          void onScanArchiveCode(raw);
        } else if (currentStep === 'scan_signature') {
          void onScanBindSignature(raw);
        }
      },
      () => setBindScannerReady(true),
      (msg) => {
        message.error(msg);
        setBindScannerActive(false);
      },
    );
    return () => stopBindScanner();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, bindScannerActive, bindStep]);

  const scannerBox = (label: string) => (
    <div
      style={{
        width: '84%',
        maxWidth: 320,
        aspectRatio: '1 / 1',
        background: 'linear-gradient(145deg, #0f172a, #1e293b)',
        borderRadius: 12,
        overflow: 'hidden',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        position: 'relative',
        margin: '14px auto 12px',
        border: '2px solid #334155',
        boxShadow: 'inset 0 2px 8px rgba(0,0,0,0.3)',
      }}
    >
      <div style={{ position: 'absolute', top: 8, left: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderTopLeftRadius: 4, zIndex: 2 }} />
      <div style={{ position: 'absolute', top: 8, right: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderTopRightRadius: 4, zIndex: 2 }} />
      <div style={{ position: 'absolute', bottom: 8, left: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderBottomLeftRadius: 4, zIndex: 2 }} />
      <div style={{ position: 'absolute', bottom: 8, right: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderBottomRightRadius: 4, zIndex: 2 }} />
      <video ref={bindVideoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
      {!bindScannerReady && (
        <div
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            gap: 8,
            cursor: bindScannerActive ? 'default' : 'pointer',
            userSelect: 'none',
          }}
          onClick={() => {
            if (!bindScannerActive) onToggleBindScanner();
          }}
        >
          <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>
            {bindScannerActive ? '摄像头初始化中...' : label}
          </Typography.Text>
        </div>
      )}
    </div>
  );

  return (
    <Modal
      title={
        <span style={{ display: 'block', fontSize: 20, fontWeight: 600, textAlign: 'center' }}>
          {modalTitle}
        </span>
      }
      open={open}
      footer={null}
      onCancel={() => {
        setBindScannerActive(false);
        stopBindScanner();
        onClose();
      }}
      destroyOnClose
      width={520}
    >
      {bindStep === 'scan_archive_code' && (
        <>
          {record && (
            <Descriptions column={1} size="small" bordered style={{ marginBottom: 12 }}>
              <Descriptions.Item label="当前档案号">{record.archive_no ?? '-'}</Descriptions.Item>
              <Descriptions.Item label="当前身份ID">{record.sfid_code ?? '-'}</Descriptions.Item>
              <Descriptions.Item label="当前钱包">{record.wallet_address ?? '-'}</Descriptions.Item>
            </Descriptions>
          )}
          {scannerBox('点击扫描档案码')}
          <div style={{ display: 'flex', justifyContent: 'center', gap: 12 }}>
            <Button onClick={onToggleBindScanner} loading={archiveCodeScanLoading}>
              {bindScannerActive ? '停止扫码' : '开启扫码'}
            </Button>
            <Upload
              accept="image/png,image/jpeg,image/webp,image/gif,image/bmp"
              beforeUpload={onUploadArchiveCode}
              showUploadList={false}
            >
              <Button loading={archiveCodeUploadLoading}>上传二维码</Button>
            </Upload>
          </div>
        </>
      )}

      {bindStep === 'sign_challenge' && bindChallenge && (
        <>
          <Descriptions column={1} size="small" bordered>
            <Descriptions.Item label="档案号">{bindChallenge.archive_no}</Descriptions.Item>
            <Descriptions.Item label="档案状态">{bindChallenge.archive_status}</Descriptions.Item>
            <Descriptions.Item label="绑定钱包">{bindChallenge.wallet_address}</Descriptions.Item>
          </Descriptions>
          <div style={{ display: 'flex', justifyContent: 'center', margin: '16px 0 12px' }}>
            <QRCode value={bindChallenge.sign_request} size={260} color="#134e4a" />
          </div>
          <Typography.Paragraph type="secondary" style={{ textAlign: 'center' }}>
            有效期至：{new Date(bindChallenge.expire_at * 1000).toLocaleTimeString()}
          </Typography.Paragraph>
          <div style={{ textAlign: 'center' }}>
            <Button type="primary" onClick={() => { setBindStep('scan_signature'); setBindScannerActive(true); }}>
              扫描签名回执
            </Button>
          </div>
        </>
      )}

      {bindStep === 'scan_signature' && (
        <>
          {scannerBox('点击扫描签名回执')}
          <div style={{ textAlign: 'center' }}>
            <Button onClick={onToggleBindScanner} loading={archiveCodeScanLoading}>
              {bindScannerActive ? '停止扫码' : '开启扫码'}
            </Button>
          </div>
        </>
      )}
    </Modal>
  );
}
