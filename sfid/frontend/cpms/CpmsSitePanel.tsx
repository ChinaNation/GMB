// CPMS 安装授权管理面板 —— 两码方案下只展示安装码和授权状态。
//
// 状态对应:
//   PENDING  → 显示安装码
//   ACTIVE   → 已经绑定 CPMS 本机签发公钥,不再展示安装码
//   DISABLED → 暂停接收该授权签发的档案码
//   REVOKED  → 不再接收该授权签发的档案码

import React, { useCallback, useRef, useState } from 'react';
import { Button, Input, message, Modal, Popconfirm, QRCode, Tag, Typography } from 'antd';
import {
  disableCpmsKeys,
  enableCpmsKeys,
  reissueInstallToken,
  revokeCpmsKeys,
  type CpmsSiteRow,
} from './api';
import type { AdminAuth } from '../auth/types';
import { downloadQr } from '../utils/downloadQr';
import {
  commitAdminAction,
  getPasskeyAssertion,
  prepareAdminAction,
  type AdminActionType,
  type AdminSecurityGrantOutput,
} from '../admins/admin_security_api';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';
import { WuminSignatureModal } from '../core/WuminSignatureModal';
import { SFID_MODAL_Z_INDEX } from '../core/modalStack';

interface Props {
  auth: AdminAuth;
  site: CpmsSiteRow;
  canWrite: boolean;
  onChanged: () => void;
}

type CpmsSiteStatus = 'PENDING' | 'ACTIVE' | 'DISABLED' | 'REVOKED';

type SecurityModalState = {
  actionId: string;
  signRequest: string;
  payloadHash: string;
  passkeyAssertion: unknown;
  resolve: (value: AdminSecurityGrantOutput) => void;
  reject: (reason?: unknown) => void;
};

function siteStatusTag(status: CpmsSiteStatus | undefined) {
  switch (status) {
    case 'PENDING': return <Tag color="gold">待激活</Tag>;
    case 'ACTIVE': return <Tag color="green">运行中</Tag>;
    case 'DISABLED': return <Tag color="orange">已禁用</Tag>;
    case 'REVOKED': return <Tag color="red">已吊销</Tag>;
    default: return <Tag>未知</Tag>;
  }
}

export const CpmsSitePanel: React.FC<Props> = ({ auth, site, canWrite, onChanged }) => {
  const [busy, setBusy] = useState(false);
  const qrRef = useRef<HTMLDivElement | null>(null);
  const [securityModal, setSecurityModal] = useState<SecurityModalState | null>(null);
  const [securityCommitLoading, setSecurityCommitLoading] = useState(false);
  const status = (site.status || 'PENDING') as CpmsSiteStatus;

  const runPasskeyChallengeAction = async (
    actionType: AdminActionType,
    payload: unknown,
  ): Promise<AdminSecurityGrantOutput> => {
    const prepared = await prepareAdminAction(auth, actionType, payload);
    if (prepared.auth_type !== 'PASSKEY_CHALLENGE' || !prepared.sign_request) {
      throw new Error('该操作缺少冷钱包签名请求');
    }
    const passkeyAssertion = await getPasskeyAssertion(prepared.webauthn_options);
    return new Promise<AdminSecurityGrantOutput>((resolve, reject) => {
      setSecurityModal({
        actionId: prepared.action_id,
        signRequest: prepared.sign_request || '',
        payloadHash: prepared.payload_hash,
        passkeyAssertion,
        resolve,
        reject,
      });
    });
  };

  const handleSecuritySignedResponse = useCallback(async (raw: string) => {
    if (!securityModal) return;
    setSecurityCommitLoading(true);
    try {
      const signed = parseSignedReceiptPayload(raw, securityModal.actionId);
      if (signed.challenge_id !== securityModal.actionId) {
        throw new Error('签名回执与当前请求不匹配');
      }
      const grant = await commitAdminAction<AdminSecurityGrantOutput>(auth, {
        action_id: securityModal.actionId,
        passkey_assertion: securityModal.passkeyAssertion,
        signer_pubkey: signed.signer_pubkey,
        signature: signed.signature,
        payload_hash: signed.payload_hash,
      });
      securityModal.resolve(grant);
      setSecurityModal(null);
    } catch (err) {
      securityModal.reject(err);
      message.error(err instanceof Error ? err.message : '签名回执处理失败');
    } finally {
      setSecurityCommitLoading(false);
    }
  }, [auth, securityModal]);

  const onReissue = async () => {
    setBusy(true);
    try {
      const grant = await runPasskeyChallengeAction('CPMS_REISSUE_INSTALL_TOKEN', { target: site.sfid_number });
      await reissueInstallToken(auth, site.sfid_number, grant);
      message.success('已重发安装令牌');
      onChanged();
    } catch (err) {
      message.error(err instanceof Error ? err.message : '重发失败');
    } finally { setBusy(false); }
  };

  const onDisable = async () => {
    const reason = await askReason('请输入禁用原因(可选)');
    if (reason === null) return;
    const normalizedReason = reason.trim();
    setBusy(true);
    try {
      const grant = await runPasskeyChallengeAction('CPMS_DISABLE_KEYS', {
        target: site.sfid_number,
        reason: normalizedReason,
      });
      await disableCpmsKeys(auth, site.sfid_number, normalizedReason || undefined, grant);
      message.success('已禁用');
      onChanged();
    } catch (err) {
      message.error(err instanceof Error ? err.message : '禁用失败');
    } finally { setBusy(false); }
  };

  const onEnable = async () => {
    setBusy(true);
    try {
      const grant = await runPasskeyChallengeAction('CPMS_ENABLE_KEYS', {
        target: site.sfid_number,
        reason: '',
      });
      await enableCpmsKeys(auth, site.sfid_number, grant);
      message.success('已启用');
      onChanged();
    } catch (err) {
      message.error(err instanceof Error ? err.message : '启用失败');
    } finally { setBusy(false); }
  };

  const onRevoke = async () => {
    const reason = await askReason('请输入吊销原因(可选)');
    if (reason === null) return;
    const normalizedReason = reason.trim();
    setBusy(true);
    try {
      const grant = await runPasskeyChallengeAction('CPMS_REVOKE_KEYS', {
        target: site.sfid_number,
        reason: normalizedReason,
      });
      await revokeCpmsKeys(auth, site.sfid_number, normalizedReason || undefined, grant);
      message.success('已吊销');
      onChanged();
    } catch (err) {
      message.error(err instanceof Error ? err.message : '吊销失败');
    } finally { setBusy(false); }
  };

  const onDownload = () => {
    const label = status === 'PENDING' ? 'install' : 'cpms';
    downloadQr({
      container: qrRef.current,
      filename: `cpms-${label}-${site.sfid_number}`,
      onError: (msg) => message.error(msg),
    });
  };

  // 中文注释:INSTALL 安装码完成首个 ARCHIVE 验真后即视为已使用,运行态不再提供下载。
  const installUsed = status === 'ACTIVE' && site.cpms_pubkey_bound;

  // 决定显示哪个二维码
  const qrPayload = status === 'PENDING' ? site.qr1_payload : undefined;
  const qrLabel = status === 'PENDING' ? '安装码' : installUsed ? '安装码已使用' : '无需展示二维码';
  const isDisabledOrRevoked = status === 'DISABLED' || status === 'REVOKED';

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'flex-start', gap: 12, justifyContent: 'flex-end' }}>
        {/* 左列:状态 */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8, paddingTop: 8 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, whiteSpace: 'nowrap' }}>
            <Typography.Text type="secondary" style={{ fontSize: 12 }}>授权状态</Typography.Text>
            {siteStatusTag(status)}
          </div>
          <Typography.Text type="secondary" style={{ fontSize: 11 }}>
            {site.cpms_pubkey_bound ? '签发公钥已绑定' : qrLabel}
          </Typography.Text>
        </div>

        {/* 右列:二维码 + 按钮 */}
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
          <div
            ref={qrRef}
            style={{
              padding: 8, background: '#fff', borderRadius: 8,
              position: 'relative',
              ...(isDisabledOrRevoked ? { filter: 'grayscale(1)', opacity: 0.5 } : {}),
            }}
          >
            {qrPayload && status === 'PENDING' ? (
              <QRCode value={qrPayload} size={160} bordered={false} />
            ) : (
              <div style={{ width: 160, height: 160, display: 'flex', alignItems: 'center', justifyContent: 'center', background: '#f0f0f0', borderRadius: 4 }}>
                <Typography.Text type="secondary" style={{ fontSize: 12 }}>{qrLabel}</Typography.Text>
              </div>
            )}
            {/* 禁用/吊销覆盖图标 */}
            {isDisabledOrRevoked && (
              <div style={{
                position: 'absolute', inset: 0,
                display: 'flex', alignItems: 'center', justifyContent: 'center',
              }}>
                <svg width="64" height="64" viewBox="0 0 64 64">
                  <circle cx="32" cy="32" r="28" fill="none" stroke={status === 'REVOKED' ? '#ef4444' : '#f97316'} strokeWidth="4" />
                  <line x1="12" y1="52" x2="52" y2="12" stroke={status === 'REVOKED' ? '#ef4444' : '#f97316'} strokeWidth="4" />
                </svg>
              </div>
            )}
          </div>

          {/* 操作按钮 */}
          {canWrite && (
            <div style={{ marginTop: 8, display: 'flex', justifyContent: 'center', gap: 8, flexWrap: 'wrap' }}>
              {status === 'ACTIVE' && (
                <>
                  <Popconfirm title="确认禁用 CPMS 站点?" onConfirm={onDisable}>
                    <Button
                      size="small"
                      loading={busy}
                      style={{ borderColor: '#f59e0b', color: '#b45309', background: '#fffbeb' }}
                    >
                      禁用
                    </Button>
                  </Popconfirm>
                  <Popconfirm title="确认吊销?此操作不可逆" onConfirm={onRevoke}>
                    <Button size="small" danger loading={busy}>吊销</Button>
                  </Popconfirm>
                </>
              )}
              {status === 'DISABLED' && (
                <>
                  <Button size="small" type="primary" onClick={onEnable} loading={busy}>启用</Button>
                  <Popconfirm title="确认吊销?此操作不可逆" onConfirm={onRevoke}>
                    <Button size="small" danger loading={busy}>吊销</Button>
                  </Popconfirm>
                </>
              )}
              {status === 'REVOKED' && (
                <Popconfirm title="重发后将重新走安装流程，确认？" onConfirm={onReissue}>
                  <Button size="small" type="primary" loading={busy}>重发令牌</Button>
                </Popconfirm>
              )}
              {status === 'PENDING' && (
                <Button size="small" onClick={onDownload}>下载</Button>
              )}
            </div>
          )}
        </div>
      </div>
      <WuminSignatureModal
        title="冷钱包签名确认"
        open={!!securityModal}
        onCancel={() => {
          securityModal?.reject(new Error('已取消签名确认'));
          setSecurityModal(null);
          setSecurityCommitLoading(false);
        }}
        qrTitle="签名二维码"
        qrValue={securityModal?.signRequest}
        qrHint="使用管理员冷钱包扫码签名"
        scannerHint="扫描冷钱包生成的签名回执二维码"
        scannerDisabled={securityCommitLoading}
        scannerLoading={securityCommitLoading}
        onDetected={handleSecuritySignedResponse}
        onScannerError={(msg) => message.error(msg)}
      />
    </div>
  );
};

function askReason(title: string): Promise<string | null> {
  return new Promise((resolve) => {
    let value = '';
    Modal.confirm({
      title,
      zIndex: SFID_MODAL_Z_INDEX.business,
      content: (
        <Input.TextArea defaultValue="" rows={3} onChange={(e) => { value = e.target.value; }} />
      ),
      onOk: () => resolve(value),
      onCancel: () => resolve(null),
    });
  });
}
