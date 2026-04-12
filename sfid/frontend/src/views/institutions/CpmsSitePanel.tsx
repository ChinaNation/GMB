// CPMS 站点管理面板 —— 按站点状态显示不同的二维码和操作按钮。
//
// 状态对应:
//   PENDING  → 显示 QR1(安装码),按钮:扫描 QR2 注册(由父组件 Card.extra 提供)
//   ACTIVE   → 显示 QR3(匿名证书),按钮:禁用、吊销、下载
//   DISABLED → 显示 QR3 灰显+禁用图标,按钮:启用、吊销
//   REVOKED  → 灰色二维码+红色禁止图标,按钮:重发令牌

import React, { useRef, useState } from 'react';
import { Button, Input, message, Modal, Popconfirm, QRCode, Tag, Typography } from 'antd';
import {
  disableCpmsKeys,
  enableCpmsKeys,
  reissueInstallToken,
  revokeCpmsKeys,
  type AdminAuth,
  type CpmsSiteRow,
} from '../../api/client';
import { downloadQr } from '../../utils/downloadQr';

interface Props {
  auth: AdminAuth;
  site: CpmsSiteRow;
  canWrite: boolean;
  onChanged: () => void;
}

type CpmsSiteStatus = 'PENDING' | 'ACTIVE' | 'DISABLED' | 'REVOKED';

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
  const status = (site.status || 'PENDING') as CpmsSiteStatus;

  const onReissue = async () => {
    setBusy(true);
    try {
      await reissueInstallToken(auth, site.site_sfid);
      message.success('已重发安装令牌');
      onChanged();
    } catch (err) {
      message.error(err instanceof Error ? err.message : '重发失败');
    } finally { setBusy(false); }
  };

  const onDisable = async () => {
    const reason = await askReason('请输入禁用原因(可选)');
    if (reason === null) return;
    setBusy(true);
    try {
      await disableCpmsKeys(auth, site.site_sfid, reason || undefined);
      message.success('已禁用');
      onChanged();
    } catch (err) {
      message.error(err instanceof Error ? err.message : '禁用失败');
    } finally { setBusy(false); }
  };

  const onEnable = async () => {
    setBusy(true);
    try {
      await enableCpmsKeys(auth, site.site_sfid);
      message.success('已启用');
      onChanged();
    } catch (err) {
      message.error(err instanceof Error ? err.message : '启用失败');
    } finally { setBusy(false); }
  };

  const onRevoke = async () => {
    const reason = await askReason('请输入吊销原因(可选)');
    if (reason === null) return;
    setBusy(true);
    try {
      await revokeCpmsKeys(auth, site.site_sfid, reason || undefined);
      message.success('已吊销');
      onChanged();
    } catch (err) {
      message.error(err instanceof Error ? err.message : '吊销失败');
    } finally { setBusy(false); }
  };

  const onDownload = () => {
    const label = status === 'PENDING' ? 'qr1' : 'qr3';
    downloadQr({
      container: qrRef.current,
      filename: `cpms-${label}-${site.site_sfid}`,
      onError: (msg) => message.error(msg),
    });
  };

  // 决定显示哪个二维码
  const qrPayload = status === 'PENDING' ? site.qr1_payload : site.qr3_payload;
  const qrLabel = status === 'PENDING' ? '安装码' : '匿名证书';
  const isDisabledOrRevoked = status === 'DISABLED' || status === 'REVOKED';

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'flex-start', gap: 12, justifyContent: 'flex-end' }}>
        {/* 左列:状态 */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8, paddingTop: 8 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, whiteSpace: 'nowrap' }}>
            <Typography.Text type="secondary" style={{ fontSize: 12 }}>站点状态</Typography.Text>
            {siteStatusTag(status)}
          </div>
          <Typography.Text type="secondary" style={{ fontSize: 11 }}>{qrLabel}</Typography.Text>
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
            {qrPayload ? (
              <QRCode value={qrPayload} size={160} bordered={false} />
            ) : (
              <div style={{ width: 160, height: 160, display: 'flex', alignItems: 'center', justifyContent: 'center', background: '#f0f0f0', borderRadius: 4 }}>
                <Typography.Text type="secondary" style={{ fontSize: 12 }}>无二维码</Typography.Text>
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
                    <Button size="small" danger loading={busy}>禁用</Button>
                  </Popconfirm>
                  <Popconfirm title="确认吊销?此操作不可逆" onConfirm={onRevoke}>
                    <Button size="small" danger loading={busy}>吊销</Button>
                  </Popconfirm>
                  <Button size="small" onClick={onDownload}>下载</Button>
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
    </div>
  );
};

function askReason(title: string): Promise<string | null> {
  return new Promise((resolve) => {
    let value = '';
    Modal.confirm({
      title,
      content: (
        <Input.TextArea defaultValue="" rows={3} onChange={(e) => { value = e.target.value; }} />
      ),
      onOk: () => resolve(value),
      onCancel: () => resolve(null),
    });
  });
}
