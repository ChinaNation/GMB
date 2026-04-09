// 中文注释:公安局详情页 CPMS 站点管理 —— 纯展示组件。
//
// 任务卡 `20260408-sfid-public-security-cpms-embed`:
// 嵌入"机构信息 Card"的右侧栏。布局:
//   上半:左 = 安装令牌 + 站点状态 Tag,右 = QR1 二维码图像
//   下半:操作按钮(扫描 QR2 / 重发 / 禁用 / 吊销 / 删除 / 下载)
// 数据抓取 / generate / 刷新逻辑全部由父 `InstitutionDetailPage` 持有。

import React, { useRef, useState } from 'react';
import {
  Button,
  Input,
  message,
  Modal,
  Popconfirm,
  QRCode,
  Tag,
  Typography,
} from 'antd';
import {
  deleteCpmsKeys,
  disableCpmsKeys,
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
  /** 任一操作(重发/禁用/吊销/删除)成功后回调父组件刷新。注册按钮已上提到父 Card.extra */
  onChanged: () => void;
}

type InstallTokenStatus = 'PENDING' | 'USED' | 'REVOKED';
type CpmsSiteStatus = 'PENDING' | 'ACTIVE' | 'DISABLED' | 'REVOKED';

function installTokenTag(status: InstallTokenStatus | undefined) {
  switch (status) {
    case 'PENDING':
      return <Tag color="blue">待安装</Tag>;
    case 'USED':
      return <Tag color="green">已使用</Tag>;
    case 'REVOKED':
      return <Tag color="red">已吊销</Tag>;
    default:
      return <Tag>未知</Tag>;
  }
}

function siteStatusTag(status: CpmsSiteStatus | undefined) {
  switch (status) {
    case 'PENDING':
      return <Tag color="gold">待激活</Tag>;
    case 'ACTIVE':
      return <Tag color="green">运行中</Tag>;
    case 'DISABLED':
      return <Tag color="orange">已禁用</Tag>;
    case 'REVOKED':
      return <Tag color="red">已吊销</Tag>;
    default:
      return <Tag>未知</Tag>;
  }
}

export const CpmsSitePanel: React.FC<Props> = ({ auth, site, canWrite, onChanged }) => {
  const [busy, setBusy] = useState(false);
  const qrRef = useRef<HTMLDivElement | null>(null);

  const status = site.status as CpmsSiteStatus | undefined;
  const tokenStatus = site.install_token_status as InstallTokenStatus | undefined;
  const canReissue = canWrite && (status === 'PENDING' || tokenStatus === 'REVOKED');
  const canDisable = canWrite && status === 'ACTIVE';
  const canRevoke = canWrite && (status === 'ACTIVE' || status === 'DISABLED');
  const canDelete = canWrite && status === 'PENDING';

  const onReissue = async () => {
    setBusy(true);
    try {
      await reissueInstallToken(auth, site.site_sfid);
      message.success('已重发安装令牌');
      onChanged();
    } catch (err) {
      message.error(err instanceof Error ? err.message : '重发失败');
    } finally {
      setBusy(false);
    }
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
    } finally {
      setBusy(false);
    }
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
    } finally {
      setBusy(false);
    }
  };

  const onDelete = async () => {
    setBusy(true);
    try {
      await deleteCpmsKeys(auth, site.site_sfid);
      message.success('已删除');
      onChanged();
    } catch (err) {
      message.error(err instanceof Error ? err.message : '删除失败');
    } finally {
      setBusy(false);
    }
  };

  // 中文注释:步 0 起统一走 utils/downloadQr,原本地实现已迁出。
  const onDownload = () => {
    downloadQr({
      container: qrRef.current,
      filename: `cpms-qr1-${site.site_sfid}`,
      onError: (msg) => message.error(msg),
    });
  };

  return (
    <div>
      {/* 布局:整组靠右显示;状态紧贴 QR 左侧,按钮紧贴 QR 下方居中 */}
      <div style={{ display: 'flex', alignItems: 'flex-start', gap: 12, justifyContent: 'flex-end' }}>
        {/* 左列:状态 */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8, paddingTop: 8 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, whiteSpace: 'nowrap' }}>
            <Typography.Text type="secondary" style={{ fontSize: 12 }}>
              安装令牌
            </Typography.Text>
            {installTokenTag(tokenStatus)}
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, whiteSpace: 'nowrap' }}>
            <Typography.Text type="secondary" style={{ fontSize: 12 }}>
              站点状态
            </Typography.Text>
            {siteStatusTag(status)}
          </div>
        </div>

        {/* 右列:QR + 按钮(按钮居中对齐在 QR 正下方) */}
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
          {site.qr1_payload && (
            <div
              ref={qrRef}
              style={{
                padding: 8,
                background: '#fff',
                borderRadius: 8,
              }}
            >
              <QRCode value={site.qr1_payload} size={160} bordered={false} />
            </div>
          )}
          {canWrite && (
            <div
              style={{
                marginTop: 8,
                display: 'flex',
                justifyContent: 'center',
                gap: 8,
                flexWrap: 'wrap',
              }}
            >
              {canReissue && (
                <Button size="small" onClick={onReissue} loading={busy}>
                  重发令牌
                </Button>
              )}
              {canDisable && (
                <Popconfirm title="确认禁用 CPMS 站点?" onConfirm={onDisable}>
                  <Button size="small" danger>
                    禁用
                  </Button>
                </Popconfirm>
              )}
              {canRevoke && (
                <Popconfirm title="确认吊销?此操作不可逆" onConfirm={onRevoke}>
                  <Button size="small" danger>
                    吊销
                  </Button>
                </Popconfirm>
              )}
              {canDelete && (
                <Popconfirm title="确认删除 CPMS 站点?" onConfirm={onDelete}>
                  <Button size="small" danger>
                    删除
                  </Button>
                </Popconfirm>
              )}
              <Button size="small" onClick={onDownload}>
                下载
              </Button>
            </div>
          )}
        </div>
      </div>

    </div>
  );
};

// 中文注释:简易原因输入提示框。返回 null 表示取消,字符串(可能为空)表示确认。
function askReason(title: string): Promise<string | null> {
  return new Promise((resolve) => {
    let value = '';
    Modal.confirm({
      title,
      content: (
        <Input.TextArea
          defaultValue=""
          rows={3}
          onChange={(e) => {
            value = e.target.value;
          }}
        />
      ),
      onOk: () => resolve(value),
      onCancel: () => resolve(null),
    });
  });
}
