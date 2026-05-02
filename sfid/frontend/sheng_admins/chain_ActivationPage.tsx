// 中文注释:首登激活省级签名密钥页(ADR-008)。
// 三槽各自独立签名密钥;首次登录或本机检测到 signing 未激活时,引导调
// POST /api/v1/admin/sheng-signer/activate(推链段当前为 mock,phase7 切真)。

import React, { useState } from 'react';
import { Card, Button, Alert, message, Descriptions, Tag } from 'antd';
import type { AdminAuth } from '../auth/types';
import { activateSigner, type SignerActivateResult } from './chain_sheng_admins';
import { ShengSlotLabel } from './chain_sheng_admins_types';
import { glassCardStyle, glassCardHeadStyle } from '../common/cardStyles';

interface Props {
  auth: AdminAuth;
}

export const ActivationPage: React.FC<Props> = ({ auth }) => {
  const [submitting, setSubmitting] = useState(false);
  const [result, setResult] = useState<SignerActivateResult | null>(null);

  const handleActivate = async () => {
    setSubmitting(true);
    try {
      const r = await activateSigner(auth);
      setResult(r);
      const tag = r.chain_status === 'MOCKED' ? '(链上推送 mock,phase7 切真)' : '';
      message.success(`签名密钥已激活 ${tag}`);
    } catch (e) {
      message.error(e instanceof Error ? e.message : '激活失败');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Card title="激活本槽签名密钥" style={glassCardStyle} headStyle={glassCardHeadStyle}>
      <Alert
        type="info"
        showIcon
        style={{ marginBottom: 16 }}
        message="说明"
        description={
          <div>
            ADR-008 起每个省管理员槽(Main / Backup1 / Backup2)各自独立签名密钥。
            <br />
            首次登录该槽时,需主动激活才能签发业务凭证。激活会:
            <ol style={{ marginTop: 6 }}>
              <li>本地生成 sr25519 keypair,加密落盘到 SFID 后端</li>
              <li>推链 <code>activate_sheng_signing_pubkey</code>(Pays::No)登记签名公钥</li>
            </ol>
            <em>当前推链段为 mock,Step 2 区块链 extrinsic 上线后切真(phase7)。</em>
          </div>
        }
      />

      <Descriptions column={1} size="small" style={{ marginBottom: 16 }}>
        <Descriptions.Item label="省份">{auth.admin_province ?? '-'}</Descriptions.Item>
        <Descriptions.Item label="当前槽">
          {auth.unlocked_slot ? <Tag color="cyan">{ShengSlotLabel[auth.unlocked_slot]}</Tag> : <Tag>未识别</Tag>}
        </Descriptions.Item>
        <Descriptions.Item label="管理员公钥">
          <code style={{ fontSize: 12 }}>{auth.unlocked_admin_pubkey ?? auth.admin_pubkey}</code>
        </Descriptions.Item>
      </Descriptions>

      <Button type="primary" loading={submitting} onClick={handleActivate} disabled={!!result}>
        {result ? '已激活' : '激活签名密钥'}
      </Button>

      {result && (
        <Alert
          type="success"
          showIcon
          style={{ marginTop: 16 }}
          message="激活成功"
          description={
            <Descriptions column={1} size="small">
              <Descriptions.Item label="签名公钥">
                <code style={{ fontSize: 12 }}>{result.signing_pubkey}</code>
              </Descriptions.Item>
              <Descriptions.Item label="链上状态">
                <Tag color={result.chain_status === 'MOCKED' ? 'orange' : 'green'}>{result.chain_status}</Tag>
              </Descriptions.Item>
              {result.chain_tx_hash && (
                <Descriptions.Item label="交易哈希">
                  <code style={{ fontSize: 12 }}>{result.chain_tx_hash}</code>
                </Descriptions.Item>
              )}
            </Descriptions>
          }
        />
      )}
    </Card>
  );
};
