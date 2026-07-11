// 机构详情页(调度器)。各类机构统一使用左侧导航详情布局;
// 私权机构仍由 PrivateDetailLayout 承接本模块独有编辑逻辑。

import React, { useCallback, useEffect, useState } from 'react';
import { Button, Card, Col, Descriptions, Popconfirm, Row, Space, Tag, Typography } from 'antd';
import { EDUCATION_TYPE_LABEL } from '../subjects/labels';
import { useInstitutionCodeLabels } from '../subjects/institutionLabels';
import { getInstitution, type InstitutionDetail } from './api';
import { deleteAccount } from '../accounts/api';
import type { AdminAuth } from '../auth/types';
import { AccountList } from '../accounts/AccountList';
import { notice } from '../utils/notice';
import { CreateAccountModal } from '../accounts/CreateAccountModal';
import { PrivateDetailLayout } from '../private/PrivateDetailLayout';
import { DocsLibrary } from '../docs/DocsLibrary';
import {
  commitAdminAction,
  prepareAdminAction,
  type AdminActionType,
  type AdminSecurityGrantOutput,
} from '../admins/securityApi';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';
import { CitizenSignatureModal } from '../core/CitizenSignatureModal';
import {
  institutionDetailCacheKey,
  readCachedInstitutionDetail,
  writeCachedInstitutionDetail,
} from '../china/metaCache';
import { InstitutionDetailNavLayout } from '../core/InstitutionDetailNavLayout';
import { OperationRecords } from './OperationRecords';

interface Props {
  auth: AdminAuth;
  cidNumber: string;
  canWrite: boolean;
  /** 不传则隐藏返回按钮(注册局 tab 里市注册局管理员直接进详情、或联邦注册局无上一级时)。 */
  onBack?: () => void;
  /** 返回按钮文案,默认「返回列表」。 */
  backLabel?: string;
  /**
   * 详情数据加载覆盖。不传则走默认 getInstitution(auth, cidNumber)(带 scope 校验)。
   * 联邦注册局走 scope-bypass 的 getFederalRegistry,通过此 prop 注入。
   */
  loadDetail?: () => Promise<InstitutionDetail>;
  /** 注册局机构详情页:有管理员数据时显示“管理员列表”tab;普通机构不传。 */
  adminListSection?: React.ReactNode;
  /** 注册局管理员进入详情页时可默认打开管理员列表。 */
  initialActiveKey?: string;
}

const SUBJECT_STATUS_LABEL: Record<string, string> = {
  ACTIVE: '正常',
  REVOKED: '已注销',
};

type SecurityModalState = {
  actionId: string;
  signRequest: string;
  payloadHash: string;
  resolve: (value: AdminSecurityGrantOutput) => void;
  reject: (reason?: unknown) => void;
};

export const GovDetailPage: React.FC<Props> = ({ auth, cidNumber, canWrite, onBack, backLabel, loadDetail, adminListSection, initialActiveKey }) => {
  const detailCacheKey = institutionDetailCacheKey(auth, cidNumber);
  const [detail, setDetail] = useState<InstitutionDetail | null>(() =>
    readCachedInstitutionDetail(detailCacheKey),
  );
  const [loading, setLoading] = useState(false);
  const [createAccountOpen, setCreateAccountOpen] = useState(false);

  const [securityCommitLoading, setSecurityCommitLoading] = useState(false);
  const [securityModal, setSecurityModal] = useState<SecurityModalState | null>(null);

  const load = useCallback(() => {
    const cached = readCachedInstitutionDetail(detailCacheKey);
    if (cached) {
      setDetail(cached);
      setLoading(false);
    } else {
      setDetail(null);
      setLoading(true);
    }
    const fetchDetail = loadDetail ?? (() => getInstitution(auth, cidNumber));
    fetchDetail()
      .then((next) => {
        setDetail(next);
        writeCachedInstitutionDetail(detailCacheKey, next);
      })
      .catch(() => { /* 静默：后台刷新失败不弹窗 */ })
      .finally(() => {
        if (!cached) setLoading(false);
      });
  }, [auth.access_token, detailCacheKey, cidNumber, loadDetail]);

  useEffect(() => {
    load();
  }, [load]);

  const runScanSignGrant = async (
    actionType: AdminActionType,
    payload: unknown,
  ): Promise<AdminSecurityGrantOutput> => {
    const prepared = await prepareAdminAction(auth, actionType, payload);
    if (prepared.auth_type !== 'PASSKEY_COLD_SIGN' || !prepared.sign_request) {
      throw new Error('该操作缺少公民钱包签名请求');
    }
    return new Promise<AdminSecurityGrantOutput>((resolve, reject) => {
      setSecurityModal({
        actionId: prepared.action_id,
        signRequest: prepared.sign_request || '',
        payloadHash: prepared.payload_hash,
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
        throw new Error('签名响应与当前请求不匹配');
      }
      if (!signed.signer_pubkey) {
        throw new Error('签名响应缺少 signer_pubkey');
      }
      const grant = await commitAdminAction<AdminSecurityGrantOutput>(auth, {
        action_id: securityModal.actionId,
        signer_pubkey: signed.signer_pubkey,
        signature: signed.signature,
        payload_hash: securityModal.payloadHash,
      });
      securityModal.resolve(grant);
      setSecurityModal(null);
    } catch (err) {
      securityModal.reject(err);
      notice.error(err, '');
    } finally {
      setSecurityCommitLoading(false);
    }
  }, [auth, securityModal]);

  const inst = detail?.institution;
  const accounts = detail?.accounts || [];
  const institutionLabels = useInstitutionCodeLabels();
  const administrativeArea = inst
    ? [inst.province_name, inst.city_name, inst.town_name].filter(Boolean).join('/') || '-'
    : '-';

  const onDeleteAccount = async (accountName: string) => {
    try {
      const grant = await runScanSignGrant('INSTITUTION_DELETE_ACCOUNT', {
        target: cidNumber,
        cid_number: cidNumber,
        account_name: accountName,
      });
      await deleteAccount(auth, cidNumber, accountName, grant);
      notice.success(`账户 "${accountName}" 已删除`);
      load();
    } catch (err) {
      notice.error(err, '');
    }
  };

  // 注册局注销整个机构——走 PASSKEY_COLD_SIGN 最严档,后端校验通过后签发注销凭证
  // (整机构 scope);机构管理员再拉 /deregistration-info 构造 propose_close 上链(见 ADR-023 §6.3)。
  // 创世/治理机构由后端 is_genesis_protected/org 闸权威拒,前端按 created_by 隐藏入口。
  const onDeregisterInstitution = async () => {
    try {
      await runScanSignGrant('INSTITUTION_DEREGISTER', {
        target: cidNumber,
        cid_number: cidNumber,
      });
      notice.success('已签发机构注销凭证,由机构管理员上链注销(将关闭其全部账户)');
      load();
    } catch (err) {
      notice.error(err, '');
    }
  };

  const renderOfficialDetail = () => {
    if (!inst || !detail || inst.category === 'PRIVATE_INSTITUTION') return null;

    const institutionInfoSection = (
      <Card
        title={
          <span style={{ fontSize: 18, fontWeight: 600 }}>
            机构信息
          </span>
        }
        extra={
          <Space>
            {canWrite && inst.status === 'ACTIVE' && inst.created_by !== 'SYSTEM' && (
              <Popconfirm
                title="注销整个机构"
                description="将关闭该机构的全部账户(余额转入指定 beneficiary),需机构管理员上链确认。"
                okText="确认注销"
                okButtonProps={{ danger: true }}
                cancelText="取消"
                onConfirm={onDeregisterInstitution}
              >
                <Button danger>注销机构</Button>
              </Popconfirm>
            )}
          </Space>
        }
      >
        <Row gutter={24}>
          <Col xs={24} md={24}>
            <Descriptions column={1} size="small">
              <Descriptions.Item label="身份ID">
                <Typography.Text style={{ fontSize: 12, wordBreak: 'break-all' }}>
                  {inst.cid_number}
                </Typography.Text>
              </Descriptions.Item>
              <Descriptions.Item label="全称">{inst.cid_full_name || '-'}</Descriptions.Item>
              <Descriptions.Item label="简称">{inst.cid_short_name || inst.cid_full_name || '-'}</Descriptions.Item>
              <Descriptions.Item label="行政区">{administrativeArea}</Descriptions.Item>
              <Descriptions.Item label="机构类型">
                {institutionLabels[inst.institution_code] || inst.institution_code}
              </Descriptions.Item>
              {inst.education_type && (
                <Descriptions.Item label="教育分类">
                  {EDUCATION_TYPE_LABEL[inst.education_type] || inst.education_type}
                </Descriptions.Item>
              )}
              <Descriptions.Item label="状态">
                <Tag color={inst.status === 'ACTIVE' ? 'green' : 'red'}>
                  {SUBJECT_STATUS_LABEL[inst.status] || inst.status}
                </Tag>
              </Descriptions.Item>
              <Descriptions.Item label="法定代表人姓名">
                {inst.legal_rep_name || <span style={{ color: '#999' }}>(未填写)</span>}
              </Descriptions.Item>
              <Descriptions.Item label="法定代表人身份ID">
                {inst.legal_rep_cid_number ? (
                  <Typography.Text style={{ fontSize: 12, wordBreak: 'break-all' }}>
                    {inst.legal_rep_cid_number}
                  </Typography.Text>
                ) : (
                  <span style={{ color: '#999' }}>(未填写)</span>
                )}
              </Descriptions.Item>
              <Descriptions.Item label="法定代表人证件照">
                {inst.legal_rep_photo_name || <span style={{ color: '#999' }}>(未上传)</span>}
              </Descriptions.Item>
              <Descriptions.Item label="创建时间">
                {new Date(inst.created_at).toLocaleString('zh-CN')}
              </Descriptions.Item>
            </Descriptions>
          </Col>
        </Row>
      </Card>
    );

    const accountListSection = (
      <Card
        type="inner"
        title={`账户列表(${accounts.length})`}
        extra={
          canWrite && (
            <Button type="primary" onClick={() => setCreateAccountOpen(true)}>
              + 新建账户
            </Button>
          )
        }
      >
        <AccountList
          accounts={accounts}
          loading={loading}
          canDelete={canWrite}
          onDelete={onDeleteAccount}
        />
      </Card>
    );

    return (
      <>
        <InstitutionDetailNavLayout
          backAction={onBack ? { label: backLabel ?? '返回列表', onClick: onBack } : undefined}
          title={inst.cid_full_name ?? inst.cid_short_name ?? '(未设置全称)'}
          subtitle={`身份ID：${inst.cid_number}`}
          status={
            <Tag color={inst.status === 'ACTIVE' ? 'green' : 'red'}>
              {SUBJECT_STATUS_LABEL[inst.status] || inst.status}
            </Tag>
          }
          items={[
            { key: 'info', label: '机构信息', content: institutionInfoSection },
            ...(adminListSection
              ? [{ key: 'admins', label: '管理员列表', content: adminListSection }]
              : []),
            { key: 'accounts', label: '账户列表', badge: accounts.length, content: accountListSection },
            {
              key: 'documents',
              label: '资料库',
              content: (
                <DocsLibrary
                  auth={auth}
                  cidNumber={inst.cid_number}
                  canWrite={canWrite}
                  createScanSignGrant={runScanSignGrant}
                />
              ),
            },
            {
              key: 'operations',
              label: '操作记录',
              content: <OperationRecords auth={auth} cidNumber={inst.cid_number} />,
            },
          ]}
          initialActiveKey={initialActiveKey}
        />

        <CreateAccountModal
          auth={auth}
          cidNumber={inst.cid_number}
          cidFullName={inst.cid_full_name ?? ''}
          existingAccounts={accounts}
          open={createAccountOpen}
          onCancel={() => setCreateAccountOpen(false)}
          onCreated={() => {
            setCreateAccountOpen(false);
            load();
          }}
        />
      </>
    );
  };

  return (
    <div>
      {loading && !inst && <Typography.Text type="secondary">加载中...</Typography.Text>}

      {inst && detail && (
        <>
          {/* ── 私权机构:保留独立编辑逻辑,接入共享左侧导航布局。 ── */}
          {inst.category === 'PRIVATE_INSTITUTION' ? (
            <PrivateDetailLayout
              auth={auth}
              detail={detail}
              canWrite={canWrite}
              loading={loading}
              onReload={load}
              onDeleteAccount={onDeleteAccount}
              createScanSignGrant={runScanSignGrant}
              onBack={onBack}
              backLabel={backLabel}
            />
          ) : renderOfficialDetail()}
        </>
      )}
      <CitizenSignatureModal
        title="公民钱包签名确认"
        open={!!securityModal}
        onCancel={() => {
          securityModal?.reject(new Error('已取消签名确认'));
          setSecurityModal(null);
          setSecurityCommitLoading(false);
        }}
        qrTitle="签名二维码"
        qrValue={securityModal?.signRequest}
        qrHint="使用联邦注册局管理员冷钱包扫码签名"
        scannerHint="扫描冷钱包生成的签名响应二维码"
        scannerDisabled={securityCommitLoading}
        scannerLoading={securityCommitLoading}
        onDetected={handleSecuritySignedResponse}
        onScannerError={(msg) => notice.error(msg)}
      />
    </div>
  );
};
