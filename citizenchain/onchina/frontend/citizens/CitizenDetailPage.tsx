// 中文注释:公民详情页承接单个公民档案展示与链上身份推送。
// 本地建档不要求钱包;只有本页推送链上身份时才录入钱包并要求目标钱包签名。

import { useMemo, useState } from 'react';
import { Alert, Button, Card, Descriptions, Form, Input, Modal, QRCode, Space, Tag, Typography } from 'antd';
import {
  ArrowLeftOutlined,
  CloudUploadOutlined,
  QrcodeOutlined,
  ScanOutlined,
  WalletOutlined,
} from '@ant-design/icons';

import type { AdminAuth } from '../auth/types';
import { glassCardHeadStyle, glassCardStyle } from '../core/cardStyles';
import { CitizenSignatureModal } from '../core/CitizenSignatureModal';
import { ScanAccountModal } from '../core/ScanAccountModal';
import { notice } from '../utils/notice';
import {
  completeCitizenOnchainSignature,
  prepareCitizenOnchainSignature,
  type CitizenRow,
  type CompleteCitizenOnchainResult,
  type PrepareCitizenOnchainResult,
} from './api';

type Props = {
  auth: AdminAuth;
  citizen: CitizenRow;
  canWrite: boolean;
  provinceName?: string | null;
  cityName?: string | null;
  onBack: () => void;
  onUpdated: (next: CitizenRow) => void;
};

type OnchainForm = {
  wallet_account: string;
};

function makeCitizenName(row: Pick<CitizenRow, 'citizen_family_name' | 'citizen_given_name'>) {
  return `${row.citizen_family_name ?? ''}${row.citizen_given_name ?? ''}`.trim() || '-';
}

function formatDate(value?: string) {
  if (!value) return '-';
  const parts = value.split('-');
  if (parts.length !== 3) return value;
  return `${parts[0]}年${parts[1]}月${parts[2]}日`;
}

function formatDateRange(from?: string, until?: string) {
  if (!from || !until) return '-';
  return `${formatDate(from)}-${formatDate(until)}`;
}

function sexText(sex?: string) {
  if (sex === 'MALE') return '男';
  if (sex === 'FEMALE') return '女';
  return '-';
}

function statusTag(status?: string) {
  if (status === 'NORMAL') return <Tag color="green">正常</Tag>;
  if (status === 'REVOKED') return <Tag color="red">注销</Tag>;
  return <Tag>-</Tag>;
}

function statusText(status?: string) {
  if (status === 'NORMAL') return '正常';
  if (status === 'REVOKED') return '注销';
  return '-';
}

function areaText(province?: string, city?: string, town?: string) {
  return [province, city, town].filter((v) => v?.trim()).join(' / ') || '-';
}

function calculateAgeYears(birthDate?: string) {
  if (!birthDate) return null;
  const birth = new Date(`${birthDate}T00:00:00`);
  if (Number.isNaN(birth.getTime())) return null;
  const today = new Date();
  let age = today.getFullYear() - birth.getFullYear();
  const beforeBirthday =
    today.getMonth() < birth.getMonth() ||
    (today.getMonth() === birth.getMonth() && today.getDate() < birth.getDate());
  if (beforeBirthday) age -= 1;
  return age;
}

export function CitizenDetailPage({
  auth,
  citizen,
  canWrite,
  provinceName,
  cityName,
  onBack,
  onUpdated,
}: Props) {
  const [form] = Form.useForm<OnchainForm>();
  const [current, setCurrent] = useState(citizen);
  const [scanOpen, setScanOpen] = useState(false);
  const [prepareLoading, setPrepareLoading] = useState(false);
  const [completeLoading, setCompleteLoading] = useState(false);
  const [prepared, setPrepared] = useState<PrepareCitizenOnchainResult | null>(null);
  const [chainRequest, setChainRequest] = useState<CompleteCitizenOnchainResult | null>(null);
  const [chainModalOpen, setChainModalOpen] = useState(false);

  const ageYears = useMemo(() => calculateAgeYears(current.citizen_birth_date), [current.citizen_birth_date]);
  const canPushOnchain =
    canWrite &&
    current.citizen_status === 'NORMAL' &&
    current.identity_status === 'NORMAL' &&
    current.voting_eligible &&
    typeof ageYears === 'number' &&
    ageYears >= 16;

  const titleText = provinceName && cityName ? `${provinceName} · ${cityName}` : '公民详情';

  const updateWalletAddress = (walletAddress: string) => {
    const next = { ...current, wallet_address: walletAddress };
    setCurrent(next);
    onUpdated(next);
  };

  const prepareOnchainSignature = async () => {
    const values = await form.validateFields();
    setPrepareLoading(true);
    setPrepared(null);
    setChainRequest(null);
    try {
      const output = await prepareCitizenOnchainSignature(
        auth,
        current.cid_number,
        values.wallet_account.trim(),
      );
      setPrepared(output);
      setChainRequest(null);
      setChainModalOpen(false);
      notice.success('公民钱包签名二维码已生成');
    } catch (err) {
      notice.error(err, '生成签名二维码失败');
    } finally {
      setPrepareLoading(false);
    }
  };

  const completeOnchainSignature = async (raw: string) => {
    const values = form.getFieldsValue();
    const walletAccount = values.wallet_account?.trim();
    if (!walletAccount) {
      notice.warning('请先录入钱包账户');
      return;
    }
    setCompleteLoading(true);
    try {
      const output = await completeCitizenOnchainSignature(
        auth,
        current.cid_number,
        walletAccount,
        raw,
      );
      setPrepared(null);
      setChainRequest(output);
      setChainModalOpen(true);
      updateWalletAddress(output.wallet_address);
      notice.success('公民钱包签名已验证,请继续提交链上交易');
    } catch (err) {
      notice.error(err, '公民钱包签名验证失败');
    } finally {
      setCompleteLoading(false);
    }
  };

  return (
    <>
      <Card
        title={
          <div style={{ position: 'relative', display: 'flex', alignItems: 'center', minHeight: 32 }}>
            <Button type="link" icon={<ArrowLeftOutlined />} style={{ paddingLeft: 0 }} onClick={onBack}>
              返回
            </Button>
            <span style={{ position: 'absolute', left: '50%', transform: 'translateX(-50%)' }}>
              {titleText}
            </span>
          </div>
        }
        bordered={false}
        style={glassCardStyle}
        headStyle={glassCardHeadStyle}
      >
        <Typography.Title level={4} style={{ marginTop: 0, marginBottom: 16 }}>
          公民详情
        </Typography.Title>

        <Descriptions column={1} size="small" bordered>
          <Descriptions.Item label="护照号">{current.passport_no || '-'}</Descriptions.Item>
          <Descriptions.Item label="身份CID">{current.cid_number || '-'}</Descriptions.Item>
          <Descriptions.Item label="姓名">{makeCitizenName(current)}</Descriptions.Item>
          <Descriptions.Item label="性别">{sexText(current.citizen_sex)}</Descriptions.Item>
          <Descriptions.Item label="出生日期">{formatDate(current.citizen_birth_date)}</Descriptions.Item>
          <Descriptions.Item label="年龄">{typeof ageYears === 'number' ? `${ageYears}周岁` : '-'}</Descriptions.Item>
          <Descriptions.Item label="投票账户">{current.wallet_address || '-'}</Descriptions.Item>
          <Descriptions.Item label="选举权利">{current.voting_eligible ? '有' : '无'}</Descriptions.Item>
          <Descriptions.Item label="公民状态">{statusTag(current.citizen_status)}</Descriptions.Item>
          <Descriptions.Item label="投票身份状态">{statusText(current.identity_status)}</Descriptions.Item>
          <Descriptions.Item label="居住地">
            {areaText(
              current.residence_province_name,
              current.residence_city_name,
              current.residence_town_name,
            )}
          </Descriptions.Item>
          <Descriptions.Item label="出生地">
            {areaText(current.birth_province_name, current.birth_city_name, current.birth_town_name)}
          </Descriptions.Item>
          <Descriptions.Item label="有效期">
            {formatDateRange(current.passport_valid_from, current.passport_valid_until)}
          </Descriptions.Item>
          <Descriptions.Item label="档案哈希">{current.archive_hash || '-'}</Descriptions.Item>
          <Descriptions.Item label="链上交易">{current.onchain_tx_hash || '-'}</Descriptions.Item>
        </Descriptions>

        <div style={{ marginTop: 20, borderTop: '1px solid #e5e7eb', paddingTop: 18 }}>
          <Typography.Title level={5} style={{ marginTop: 0 }}>
            链上身份推送
          </Typography.Title>
          {!canPushOnchain && (
            <Alert
              type="warning"
              showIcon
              style={{ marginBottom: 16 }}
              message="当前档案不能推送链上身份"
              description="公民必须年满16周岁、档案正常且具有选举资格。"
            />
          )}
          <Form
            form={form}
            layout="inline"
            initialValues={{ wallet_account: current.wallet_address ?? '' }}
            style={{ rowGap: 12 }}
          >
            <Form.Item
              name="wallet_account"
              rules={[{ required: canPushOnchain, message: '请输入钱包账户' }]}
              style={{ minWidth: 460, marginBottom: 0 }}
            >
              <Input
                prefix={<WalletOutlined />}
                placeholder="推送链上身份时录入公民钱包账户"
                disabled={!canPushOnchain || prepareLoading || completeLoading}
                allowClear
              />
            </Form.Item>
            <Form.Item style={{ marginBottom: 0 }}>
              <Button
                icon={<ScanOutlined />}
                disabled={!canPushOnchain || prepareLoading || completeLoading}
                onClick={() => setScanOpen(true)}
              >
                扫描钱包
              </Button>
            </Form.Item>
            <Form.Item style={{ marginBottom: 0 }}>
              <Button
                type="primary"
                icon={<QrcodeOutlined />}
                loading={prepareLoading}
                disabled={!canPushOnchain || completeLoading}
                onClick={prepareOnchainSignature}
              >
                生成签名二维码
              </Button>
            </Form.Item>
            {chainRequest && (
              <Form.Item style={{ marginBottom: 0 }}>
                <Button icon={<CloudUploadOutlined />} onClick={() => setChainModalOpen(true)}>
                  查看链上交易二维码
                </Button>
              </Form.Item>
            )}
          </Form>
        </div>
      </Card>

      <ScanAccountModal
        open={scanOpen}
        onClose={() => setScanOpen(false)}
        onResolved={(address) => {
          form.setFieldsValue({ wallet_account: address });
          setScanOpen(false);
        }}
      />

      <CitizenSignatureModal
        title="公民钱包签名确认"
        open={!!prepared}
        onCancel={() => setPrepared(null)}
        qrTitle="身份载荷签名二维码"
        qrValue={prepared?.sign_request}
        qrHint="使用该公民本人的公民钱包扫码签名"
        scannerHint="扫描该公民钱包生成的签名响应二维码"
        scannerDisabled={completeLoading}
        scannerLoading={completeLoading}
        onDetected={completeOnchainSignature}
        onScannerError={(msg) => notice.error(msg)}
      />

      <Modal
        title="公民身份上链交易"
        open={chainModalOpen && !!chainRequest}
        onCancel={() => setChainModalOpen(false)}
        footer={[
          <Button key="done" type="primary" onClick={() => setChainModalOpen(false)}>
            已提交链上交易
          </Button>,
        ]}
        destroyOnClose
        width={460}
      >
        <Space direction="vertical" align="center" style={{ width: '100%' }} size={14}>
          <QRCode
            value={chainRequest?.citizen_identity_chain_sign_request ?? 'CID_CITIZEN_CHAIN_PENDING'}
            size={280}
          />
          <Typography.Text type="secondary" style={{ textAlign: 'center' }}>
            使用当前注册局管理员的公民钱包扫码签名并提交
          </Typography.Text>
          <Typography.Text copyable={{ text: chainRequest?.call_data_hex ?? '' }} style={{ maxWidth: 380 }}>
            {chainRequest?.call_data_hex ?? ''}
          </Typography.Text>
        </Space>
      </Modal>
    </>
  );
}
