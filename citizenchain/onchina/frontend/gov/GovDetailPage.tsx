// 机构详情页(调度器)。各类机构统一使用左侧导航详情布局;
// 私权机构仍由 PrivateDetailLayout 承接本模块独有编辑逻辑。

import React, { useCallback, useEffect, useState } from 'react';
import { Alert, Button, Card, Checkbox, Col, Descriptions, Divider, Form, Input, Row, Select, Space, Tag, Typography } from 'antd';
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
import { submitChainSign, useChainSign } from '../core/useChainSign';
import {
  prepareInstitutionGovernance,
  prepareRegisterInstitutionAdmins,
  type InstitutionGovernanceAdminInput,
  type InstitutionGovernanceAssignmentChangeInput,
  type InstitutionGovernanceRoleChangeInput,
} from '../admins/api';
import { isSubordinateRegistry, isTier1Registry } from '../platform/registryTier';

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

type GovernanceFormValues = {
  admins_text?: string;
  role_code?: string;
  role_name?: string;
  term_required?: boolean;
  role_status?: 'ACTIVE' | 'INACTIVE';
  assignments_text?: string;
  legal_representative_cid_number?: string;
  clear_legal_representative?: boolean;
};

function generateShortRoleCode(): string {
  const alphabet = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
  const bytes = new Uint8Array(4);
  crypto.getRandomValues(bytes);
  return `R${Array.from(bytes, (byte) => alphabet[byte % alphabet.length]).join('')}`;
}

function parseAdminsText(text?: string): InstitutionGovernanceAdminInput[] {
  return (text ?? '')
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const [name, account] = line.split(/[,，]/).map((part) => part.trim());
      if (!name || !account) {
        throw new Error('管理员集合每行格式必须是：姓名,账户');
      }
      return { admin_name: name, admin_account: account };
    });
}

function parseAssignmentsText(text?: string): InstitutionGovernanceAssignmentChangeInput[] {
  const byRole = new Map<string, InstitutionGovernanceAssignmentChangeInput>();
  for (const raw of (text ?? '').split(/\r?\n/)) {
    const line = raw.trim();
    if (!line) continue;
    const [roleCode, account, termStartRaw = '0', termEndRaw = '0'] = line
      .split(/[,，]/)
      .map((part) => part.trim());
    if (!roleCode || !account) {
      throw new Error('任职每行格式必须是：岗位码,管理员账户,任期开始,任期结束');
    }
    const termStart = Number(termStartRaw || 0);
    const termEnd = Number(termEndRaw || 0);
    if (!Number.isInteger(termStart) || !Number.isInteger(termEnd) || termStart < 0 || termEnd < 0) {
      throw new Error('任期必须是非负整数日序');
    }
    const row = byRole.get(roleCode) ?? { role_code: roleCode, assignments: [] };
    row.assignments.push({ admin_account: account, term_start: termStart, term_end: termEnd });
    byRole.set(roleCode, row);
  }
  return Array.from(byRole.values());
}

function InstitutionGovernancePanel({
  auth,
  cidNumber,
  canWrite,
  onSubmitted,
}: {
  auth: AdminAuth;
  cidNumber: string;
  canWrite: boolean;
  onSubmitted: () => void;
}) {
  const [form] = Form.useForm<GovernanceFormValues>();
  const [submitting, setSubmitting] = useState(false);
  const { signChain, chainSignModal } = useChainSign('机构治理链交易签名');
  const canRegistryRegister =
    isTier1Registry(auth.institution_code) || isSubordinateRegistry(auth.institution_code);

  useEffect(() => {
    form.setFieldsValue({
      role_code: generateShortRoleCode(),
      role_status: 'ACTIVE',
      term_required: false,
    });
  }, [form, cidNumber]);

  const submitPrepared = async (requestId: string, signRequest: string) => {
    const signed = await signChain(requestId, signRequest);
    const output = await submitChainSign(
      auth,
      requestId,
      signed.signer_pubkey,
      signed.signature,
    );
    notice.success(`链交易已提交：${output.tx_hash}`);
    onSubmitted();
  };

  const valuesToGovernancePayload = (values: GovernanceFormValues) => {
    const admins = parseAdminsText(values.admins_text);
    const roleChanges: InstitutionGovernanceRoleChangeInput[] = [];
    const roleCode = values.role_code?.trim() ?? '';
    const roleName = values.role_name?.trim() ?? '';
    if (roleCode || roleName) {
      if (!roleCode || !roleName) throw new Error('岗位码和岗位名称必须同时填写');
      roleChanges.push({
        role_code: roleCode,
        role_name: roleName,
        term_required: Boolean(values.term_required),
        role_status: values.role_status ?? 'ACTIVE',
      });
    }
    const legalRepresentativeCidNumber = values.legal_representative_cid_number?.trim() || undefined;
    const clearLegalRepresentative = Boolean(values.clear_legal_representative);
    if (legalRepresentativeCidNumber && clearLegalRepresentative) {
      throw new Error('任命/更换法定代表人和解除法定代表人不能同时提交');
    }
    return {
      cid_number: cidNumber,
      admins: admins.length ? admins : undefined,
      role_changes: roleChanges.length ? roleChanges : undefined,
      assignment_changes: parseAssignmentsText(values.assignments_text),
      legal_representative_cid_number: legalRepresentativeCidNumber,
      clear_legal_representative: clearLegalRepresentative || undefined,
    };
  };

  const onProposeGovernance = async () => {
    if (!canWrite) return;
    setSubmitting(true);
    try {
      const values = await form.validateFields();
      const prepared = await prepareInstitutionGovernance(auth, valuesToGovernancePayload(values));
      await submitPrepared(prepared.request_id, prepared.sign_request);
    } catch (err) {
      notice.error(err, '');
    } finally {
      setSubmitting(false);
    }
  };

  const onRegisterAdmins = async () => {
    if (!canWrite || !canRegistryRegister) return;
    setSubmitting(true);
    try {
      const values = await form.validateFields(['admins_text']);
      const admins = parseAdminsText(values.admins_text);
      if (admins.length < 2) throw new Error('注册局直接登记管理员至少需要 2 人');
      const prepared = await prepareRegisterInstitutionAdmins(auth, {
        cid_number: cidNumber,
        admins,
      });
      await submitPrepared(prepared.request_id, prepared.sign_request);
    } catch (err) {
      notice.error(err, '');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Card title="机构治理">
      <Alert
        type="info"
        showIcon
        style={{ marginBottom: 16 }}
        message="管理员是人，岗位是职位；本页面只构造链上治理交易，不本地改管理员真源。"
        description="管理员集合每行填“姓名,账户”。岗位码默认自动生成短码；任职每行填“岗位码,管理员账户,任期开始,任期结束”。法定代表人任命/更换只填公民 CID；解除则清空链上三字段。"
      />
      <Form form={form} layout="vertical" disabled={!canWrite || submitting}>
        <Form.Item label="管理员集合" name="admins_text">
          <Input.TextArea
            rows={4}
            placeholder={'张三,w5...\n李四,w5...'}
          />
        </Form.Item>
        <Divider orientation="left">岗位</Divider>
        <Row gutter={12}>
          <Col xs={24} md={8}>
            <Form.Item label="岗位码" name="role_code">
              <Input
                addonAfter={(
                  <Button
                    size="small"
                    type="link"
                    onClick={() => form.setFieldsValue({ role_code: generateShortRoleCode() })}
                  >
                    重生成
                  </Button>
                )}
              />
            </Form.Item>
          </Col>
          <Col xs={24} md={8}>
            <Form.Item label="岗位名称" name="role_name">
              <Input placeholder="例如：财务负责人" />
            </Form.Item>
          </Col>
          <Col xs={24} md={4}>
            <Form.Item label="岗位状态" name="role_status">
              <Select
                options={[
                  { label: '启用', value: 'ACTIVE' },
                  { label: '停用', value: 'INACTIVE' },
                ]}
              />
            </Form.Item>
          </Col>
          <Col xs={24} md={4}>
            <Form.Item name="term_required" valuePropName="checked" label="任期">
              <Checkbox>要求任期</Checkbox>
            </Form.Item>
          </Col>
        </Row>
        <Form.Item label="岗位任职" name="assignments_text">
          <Input.TextArea rows={4} placeholder={'RABCD,w5...,0,0'} />
        </Form.Item>
        <Form.Item label="法定代表人公民 CID" name="legal_representative_cid_number">
          <Input placeholder="只填公民 CID；姓名和钱包账户由后端读取公民档案" />
        </Form.Item>
        <Form.Item name="clear_legal_representative" valuePropName="checked">
          <Checkbox>解除法定代表人并清空链上三字段</Checkbox>
        </Form.Item>
        <Space wrap>
          <Button type="primary" loading={submitting} disabled={!canWrite} onClick={onProposeGovernance}>
            发起本机构治理
          </Button>
          <Button loading={submitting} disabled={!canWrite || !canRegistryRegister} onClick={onRegisterAdmins}>
            注册局直接登记管理员
          </Button>
        </Space>
      </Form>
      {chainSignModal}
    </Card>
  );
}

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

  const renderOfficialDetail = () => {
    if (!inst || !detail || inst.category === 'PRIVATE_INSTITUTION') return null;

    const institutionInfoSection = (
      <Card
        title={
          <span style={{ fontSize: 18, fontWeight: 600 }}>
            机构信息
          </span>
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
                {inst.legal_representative_name || <span style={{ color: '#999' }}>(未填写)</span>}
              </Descriptions.Item>
              <Descriptions.Item label="法定代表人身份ID">
                {inst.legal_representative_cid_number ? (
                  <Typography.Text style={{ fontSize: 12, wordBreak: 'break-all' }}>
                    {inst.legal_representative_cid_number}
                  </Typography.Text>
                ) : (
                  <span style={{ color: '#999' }}>(未填写)</span>
                )}
              </Descriptions.Item>
              <Descriptions.Item label="法定代表人证件照">
                {inst.legal_representative_photo_name || <span style={{ color: '#999' }}>(未上传)</span>}
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
            ...(canWrite
              ? [{
                  key: 'governance',
                  label: '机构治理',
                  content: (
                    <InstitutionGovernancePanel
                      auth={auth}
                      cidNumber={inst.cid_number}
                      canWrite={canWrite}
                      onSubmitted={load}
                    />
                  ),
                }]
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
        qrHint="使用当前注册局管理员公民钱包扫码签名"
        scannerHint="扫描公民钱包生成的签名响应二维码"
        scannerDisabled={securityCommitLoading}
        scannerLoading={securityCommitLoading}
        onDetected={handleSecuritySignedResponse}
        onScannerError={(msg) => notice.error(msg)}
      />
    </div>
  );
};
