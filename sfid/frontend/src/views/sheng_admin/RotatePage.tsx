// 中文注释:省级签名密钥 rotate 页(ADR-008)。
// 在私钥泄漏 / 例行轮换场景下,生成新签名 keypair 并推链替换旧公钥。
// 后端 POST /api/v1/admin/sheng-signer/rotate(推链段当前为 mock,phase7 切真)。

import React, { useState } from 'react';
import { Card, Button, Alert, message, Descriptions, Tag, Modal } from 'antd';
import type { AdminAuth } from '../../api/client';
import { rotateSigner, type SignerRotateResult } from '../../api/sheng_signer';
import { ShengSlotLabel } from '../../types/slot';
import { glassCardStyle, glassCardHeadStyle } from '../../components/App';

interface Props {
  auth: AdminAuth;
}

export const RotatePage: React.FC<Props> = ({ auth }) => {
  const [submitting, setSubmitting] = useState(false);
  const [result, setResult] = useState<SignerRotateResult | null>(null);

  const handleRotate = () => {
    Modal.confirm({
      title: '确认 rotate 本槽签名密钥?',
      content: 'rotate 会:① 生成新 sr25519 keypair;② 推链 rotate_sheng_signing_pubkey 替换旧公钥;③ 旧 keypair 立即失效。该操作不可撤销。',
      okType: 'danger',
      okText: 'rotate',
      cancelText: '取消',
      onOk: async () => {
        setSubmitting(true);
        try {
          const r = await rotateSigner(auth);
          setResult(r);
          const tag = r.chain_status === 'MOCKED' ? '(链上推送 mock,phase7 切真)' : '';
          message.success(`签名密钥已替换 ${tag}`);
        } catch (e) {
          message.error(e instanceof Error ? e.message : 'rotate 失败');
        } finally {
          setSubmitting(false);
        }
      },
    });
  };

  return (
    <Card title="rotate 本槽签名密钥" style={glassCardStyle} headStyle={glassCardHeadStyle}>
      <Alert
        type="warning"
        showIcon
        style={{ marginBottom: 16 }}
        message="rotate 会替换签名密钥,旧密钥立即失效"
        description={
          <div>
            场景:私钥疑似泄漏 / 例行密钥轮换。该操作仅作用于当前登录的槽
            ({auth.unlocked_slot ? ShengSlotLabel[auth.unlocked_slot] : '未识别'}),
            其他槽不受影响。
            <br />
            <em>当前推链段为 mock,Step 2 区块链 extrinsic 上线后切真(phase7)。</em>
          </div>
        }
      />

      <Descriptions column={1} size="small" style={{ marginBottom: 16 }}>
        <Descriptions.Item label="省份">{auth.admin_province ?? '-'}</Descriptions.Item>
        <Descriptions.Item label="当前槽">
          {auth.unlocked_slot ? <Tag color="cyan">{ShengSlotLabel[auth.unlocked_slot]}</Tag> : <Tag>未识别</Tag>}
        </Descriptions.Item>
      </Descriptions>

      <Button danger type="primary" loading={submitting} onClick={handleRotate}>
        rotate 签名密钥
      </Button>

      {result && (
        <Alert
          type="success"
          showIcon
          style={{ marginTop: 16 }}
          message="rotate 成功"
          description={
            <Descriptions column={1} size="small">
              <Descriptions.Item label="旧签名公钥">
                <code style={{ fontSize: 12 }}>{result.old_signing_pubkey}</code>
              </Descriptions.Item>
              <Descriptions.Item label="新签名公钥">
                <code style={{ fontSize: 12 }}>{result.new_signing_pubkey}</code>
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
