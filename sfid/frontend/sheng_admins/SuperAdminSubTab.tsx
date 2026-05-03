// 中文注释:注册局-省级管理员页。一主两备名册和本人签名密钥操作统一在这里展示。

import React, { useCallback, useEffect, useRef, useState } from 'react';
import { Button, Modal, QRCode, Space, Tag, Typography, message } from 'antd';
import { useAuth } from '../hooks/useAuth';
import { serializeQrEnvelope } from '../qr/wuminQr';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';
import { startCameraScanner } from '../utils/cameraScanner';
import { tryEncodeSs58 } from '../utils/ss58';
import type { ShengAdminSharedState } from './shengAdminUtils';
import {
  getRoster,
  type RosterEntry,
  type ShengAdminRoster,
} from './roster_api';
import {
  prepareSignerOperation,
  submitSignerOperation,
  type SigningOperation,
  type SignerPrepareResult,
} from './signing_keys_api';
import type { ShengSlot } from './types';

interface SuperAdminSubTabProps {
  selectedShengAdmin: NonNullable<ShengAdminSharedState['selectedShengAdmin']>;
}

type SigningModalState = {
  operation: SigningOperation;
  prepare: SignerPrepareResult;
  step: 'show_qr' | 'scan_response';
};

const slotTitle: Record<ShengSlot, string> = {
  Main: '主管理员',
  Backup1: '备用管理员 1',
  Backup2: '备用管理员 2',
};

export function SuperAdminSubTab({
  selectedShengAdmin,
}: SuperAdminSubTabProps) {
  const { auth } = useAuth();
  const [roster, setRoster] = useState<ShengAdminRoster | null>(null);
  const [loading, setLoading] = useState(false);
  const [operationLoading, setOperationLoading] = useState(false);
  const [signingModal, setSigningModal] = useState<SigningModalState | null>(null);
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

  const reload = useCallback(async () => {
    if (!auth) return;
    setLoading(true);
    try {
      const data = await getRoster(auth, selectedShengAdmin.province);
      setRoster(data);
    } catch (error) {
      message.error(error instanceof Error ? error.message : '省管理员名册加载失败');
    } finally {
      setLoading(false);
    }
  }, [auth, selectedShengAdmin.province]);

  useEffect(() => {
    void reload();
  }, [reload]);

  useEffect(() => () => stopScanner(), [stopScanner]);

  useEffect(() => {
    if (!scannerActive || !videoRef.current) {
      return;
    }
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
  }, [scannerActive, stopScanner]);

  const openSigningModal = async (operation: SigningOperation) => {
    if (!auth) return;
    setOperationLoading(true);
    try {
      const prepare = await prepareSignerOperation(auth, operation);
      setSigningModal({ operation, prepare, step: 'show_qr' });
    } catch (error) {
      message.error(error instanceof Error ? error.message : '签名请求生成失败');
    } finally {
      setOperationLoading(false);
    }
  };

  const handleSignedResponse = async (raw: string) => {
    if (!auth || !signingModal) return;
    try {
      const signed = parseSignedReceiptPayload(raw, signingModal.prepare.request_id);
      if (signed.challenge_id !== signingModal.prepare.request_id) {
        message.error('签名回执与当前请求不匹配');
        return;
      }
      setOperationLoading(true);
      await submitSignerOperation(auth, {
        operation: signingModal.operation,
        payload_hex: signingModal.prepare.payload_hex,
        signature: signed.signature,
        signer_pubkey: signed.signer_pubkey,
      });
      message.success(signingModal.operation === 'GENERATE' ? '签名密钥已生成' : '签名密钥已更换');
      setSigningModal(null);
      await reload();
    } catch (error) {
      message.error(error instanceof Error ? error.message : '签名回执处理失败');
    } finally {
      setOperationLoading(false);
    }
  };

  const entries = roster?.entries ?? fallbackEntries(selectedShengAdmin);

  return (
    <>
      <div style={{ display: 'grid', gap: 12 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <Typography.Text type="secondary">
            {roster?.province ?? selectedShengAdmin.province}
            {roster?.current_slot ? <Tag color="cyan" style={{ marginLeft: 8 }}>当前槽位：{slotTitle[roster.current_slot]}</Tag> : null}
          </Typography.Text>
          <Button onClick={() => void reload()} loading={loading}>刷新</Button>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: 12 }}>
          {entries.map((entry) => (
            <AdminSlotPanel
              key={entry.slot}
              entry={entry}
              loading={operationLoading}
              onGenerate={() => void openSigningModal('GENERATE')}
              onReplace={() => void openSigningModal('REPLACE')}
            />
          ))}
        </div>
      </div>

      <Modal
        title={signingModal?.operation === 'REPLACE' ? '签名更换确认' : '签名生成确认'}
        open={!!signingModal}
        onCancel={() => {
          stopScanner();
          setSigningModal(null);
        }}
        footer={null}
        destroyOnClose
      >
        {signingModal && auth ? (
          signingModal.step === 'show_qr' ? (
            <Space direction="vertical" size={12} style={{ width: '100%', alignItems: 'center' }}>
              <Typography.Text type="secondary">
                用当前省管理员账户扫码签名，签名通过后才会{signingModal.operation === 'REPLACE' ? '更换' : '生成'}本人的签名密钥。
              </Typography.Text>
              <QRCode value={buildSignerQr(auth.admin_pubkey, signingModal.prepare)} size={260} color="#134e4a" />
              <Typography.Text type="secondary">
                有效期至：{new Date(signingModal.prepare.expires_at * 1000).toLocaleTimeString('zh-CN')}
              </Typography.Text>
              <Button
                type="primary"
                loading={operationLoading}
                onClick={() => {
                  setSigningModal({ ...signingModal, step: 'scan_response' });
                  setScannerActive(true);
                }}
              >
                下一步：扫描签名回执
              </Button>
            </Space>
          ) : (
            <Space direction="vertical" size={12} style={{ width: '100%' }}>
              <Typography.Text type="secondary">扫描钱包生成的签名回执二维码。</Typography.Text>
              <div style={{ position: 'relative', width: '100%', aspectRatio: '4 / 3', background: '#0f172a', borderRadius: 8, overflow: 'hidden' }}>
                <video ref={videoRef} muted playsInline style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
                {!scannerReady ? (
                  <div style={{ position: 'absolute', inset: 0, display: 'grid', placeItems: 'center', color: '#e5e7eb' }}>
                    {scannerActive ? '摄像头初始化中...' : '摄像头未开启'}
                  </div>
                ) : null}
              </div>
              <Button onClick={() => setScannerActive((v) => !v)} loading={operationLoading}>
                {scannerActive ? '停止扫码' : '开启扫码'}
              </Button>
            </Space>
          )
        ) : null}
      </Modal>
    </>
  );
}

function AdminSlotPanel({
  entry,
  loading,
  onGenerate,
  onReplace,
}: {
  entry: RosterEntry;
  loading: boolean;
  onGenerate: () => void;
  onReplace: () => void;
}) {
  const status = renderStatus(entry);
  const canOperate = !!entry.can_operate_signing && entry.signing_status !== 'UNSET';
  const showGenerate = canOperate && (!entry.signing_pubkey || entry.signing_status === 'NOT_INITIALIZED');
  const showReplace = canOperate && !!entry.signing_pubkey && entry.signing_status !== 'NOT_INITIALIZED';

  return (
    <section style={{ border: '1px solid #e5e7eb', borderRadius: 8, padding: 14, background: 'rgba(255,255,255,0.72)' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: 8, marginBottom: 12 }}>
        <Typography.Text strong>{slotTitle[entry.slot]}</Typography.Text>
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: '76px 1fr', rowGap: 8, columnGap: 10, alignItems: 'center' }}>
        <Typography.Text type="secondary">状态</Typography.Text>
        <Space size={8} wrap>
          {status}
          {showGenerate ? <Button size="small" type="primary" loading={loading} onClick={onGenerate}>生成</Button> : null}
          {showReplace ? <Button size="small" loading={loading} onClick={onReplace}>更换</Button> : null}
        </Space>
        <Typography.Text type="secondary">生成时间</Typography.Text>
        <Typography.Text>{entry.signing_created_at ? new Date(entry.signing_created_at).toLocaleString('zh-CN') : '-'}</Typography.Text>
        <Typography.Text type="secondary">账户</Typography.Text>
        <Typography.Text code style={{ wordBreak: 'break-all' }}>
          {entry.admin_pubkey ? tryEncodeSs58(entry.admin_pubkey) : '-'}
        </Typography.Text>
      </div>
    </section>
  );
}

function renderStatus(entry: RosterEntry) {
  switch (entry.signing_status) {
    case 'UNSET':
      return <Tag>未设置</Tag>;
    case 'GENERATED':
      return <Tag color="green">已生成</Tag>;
    case 'GENERATED_NOT_LOADED':
      return <Tag color="orange">已生成 / 本机未加载</Tag>;
    case 'NOT_INITIALIZED':
    default:
      return <Tag color="blue">未初始化</Tag>;
  }
}

function fallbackEntries(selectedShengAdmin: NonNullable<ShengAdminSharedState['selectedShengAdmin']>): RosterEntry[] {
  return [
    {
      slot: 'Main',
      admin_pubkey: selectedShengAdmin.admin_pubkey,
      admin_name: selectedShengAdmin.admin_name,
      signing_status: selectedShengAdmin.signing_pubkey ? 'GENERATED_NOT_LOADED' : 'NOT_INITIALIZED',
      signing_pubkey: selectedShengAdmin.signing_pubkey ?? null,
      signing_created_at: selectedShengAdmin.signing_created_at ?? null,
      cache_loaded: false,
      can_operate_signing: false,
      can_manage_roster: false,
    },
    { slot: 'Backup1', admin_pubkey: null, signing_status: 'UNSET', can_operate_signing: false, can_manage_roster: false },
    { slot: 'Backup2', admin_pubkey: null, signing_status: 'UNSET', can_operate_signing: false, can_manage_roster: false },
  ];
}

function buildSignerQr(adminPubkey: string, prepare: SignerPrepareResult) {
  return serializeQrEnvelope({
    proto: 'WUMIN_QR_V1',
    kind: 'sign_request',
    id: prepare.request_id,
    issued_at: Math.floor(Date.now() / 1000),
    expires_at: prepare.expires_at,
    body: {
      address: tryEncodeSs58(adminPubkey),
      pubkey: adminPubkey,
      sig_alg: 'sr25519',
      payload_hex: prepare.payload_hex,
      spec_version: 0,
      display: {
        action: prepare.display_action,
        summary: prepare.display_summary,
        fields: prepare.display_fields,
      },
    },
  });
}
