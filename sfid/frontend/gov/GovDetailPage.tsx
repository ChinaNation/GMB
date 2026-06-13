// 中文注释:机构详情页(调度器)。各类机构统一使用左侧导航详情布局;
// 私权机构仍由 PrivateDetailLayout 承接本模块独有编辑逻辑。

import React, { useCallback, useEffect, useState } from 'react';
import { Button, Card, Col, Descriptions, Row, Tag, Typography } from 'antd';
import { INSTITUTION_CODE_LABEL, ORG_CODE_LABEL } from '../subjects/labels';
import { getInstitution, type InstitutionDetail } from './api';
import { deleteAccount } from '../accounts/api';
import {
  generateCpmsInstallQr,
  getCpmsSiteByInstitution,
  type CpmsSiteRow,
} from '../cpms/api';
import type { AdminAuth } from '../auth/types';
import { AccountList } from '../accounts/AccountList';
import { notice } from '../utils/notice';
import { CpmsSitePanel } from '../cpms/CpmsSitePanel';
import { CreateAccountModal } from '../accounts/CreateAccountModal';
import { PrivateDetailLayout } from '../private/PrivateDetailLayout';
import { DocumentLibrary } from '../docs/DocumentLibrary';
import {
  commitAdminAction,
  getPasskeyAssertion,
  prepareAdminAction,
  type AdminActionType,
  type AdminSecurityGrantOutput,
} from '../admins/admin_security_api';
import { parseSignedReceiptPayload } from '../utils/parseSignedPayload';
import { WuminSignatureModal } from '../core/WuminSignatureModal';
import {
  institutionDetailCacheKey,
  readCachedInstitutionDetail,
  writeCachedInstitutionDetail,
} from '../china/metaCache';
import { InstitutionDetailNavLayout } from '../core/InstitutionDetailNavLayout';
import { OperationRecords } from './OperationRecords';

interface Props {
  auth: AdminAuth;
  sfidNumber: string;
  canWrite: boolean;
  /** 不传则隐藏返回按钮(注册局 tab 里市管理员直接进详情、或联邦注册局无上一级时)。 */
  onBack?: () => void;
  /** 返回按钮文案,默认「返回列表」。 */
  backLabel?: string;
  /**
   * 详情数据加载覆盖。不传则走默认 getInstitution(auth, sfidNumber)(带 scope 校验)。
   * 联邦注册局走 scope-bypass 的 getFederalRegistry,通过此 prop 注入。
   */
  loadDetail?: () => Promise<InstitutionDetail>;
  /** 注册局机构详情页:有管理员数据时显示“管理员列表”tab;普通机构不传。 */
  adminListSection?: React.ReactNode;
}

const SUBJECT_STATUS_LABEL: Record<string, string> = {
  ACTIVE: '正常',
  REVOKED: '已注销',
};

type SecurityModalState = {
  actionId: string;
  signRequest: string;
  passkeyAssertion: unknown;
  resolve: (value: AdminSecurityGrantOutput) => void;
  reject: (reason?: unknown) => void;
};

export const GovDetailPage: React.FC<Props> = ({ auth, sfidNumber, canWrite, onBack, backLabel, loadDetail, adminListSection }) => {
  const detailCacheKey = institutionDetailCacheKey(auth, sfidNumber);
  const [detail, setDetail] = useState<InstitutionDetail | null>(() =>
    readCachedInstitutionDetail(detailCacheKey),
  );
  const [loading, setLoading] = useState(false);
  const [createAccountOpen, setCreateAccountOpen] = useState(false);

  // ── CPMS 站点状态(仅公安局机构使用) ──
  const [cpmsSite, setCpmsSite] = useState<CpmsSiteRow | null>(null);
  const [cpmsBusy, setCpmsBusy] = useState(false);
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
    const fetchDetail = loadDetail ?? (() => getInstitution(auth, sfidNumber));
    fetchDetail()
      .then((next) => {
        setDetail(next);
        writeCachedInstitutionDetail(detailCacheKey, next);
      })
      .catch(() => { /* 静默：后台刷新失败不弹窗 */ })
      .finally(() => {
        if (!cached) setLoading(false);
      });
  }, [auth.access_token, detailCacheKey, sfidNumber, loadDetail]);

  useEffect(() => {
    load();
  }, [load]);

  const runPasskeyChallengeGrant = async (
    actionType: AdminActionType,
    payload: unknown,
  ): Promise<AdminSecurityGrantOutput> => {
    const prepared = await prepareAdminAction(auth, actionType, payload);
    if (prepared.auth_type !== 'PASSKEY_CHALLENGE' || !prepared.sign_request) {
      throw new Error('该操作缺少公民钱包签名请求');
    }
    const passkeyAssertion = await getPasskeyAssertion(prepared.webauthn_options);
    return new Promise<AdminSecurityGrantOutput>((resolve, reject) => {
      setSecurityModal({
        actionId: prepared.action_id,
        signRequest: prepared.sign_request || '',
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
      notice.error(err, '');
    } finally {
      setSecurityCommitLoading(false);
    }
  }, [auth, securityModal]);

  const inst = detail?.institution;
  const accounts = detail?.accounts || [];
  const canManageCpms = canWrite && auth.role === 'FEDERAL_ADMIN';
  const administrativeArea = inst
    ? [inst.province, inst.city, inst.town].filter(Boolean).join('/') || '-'
    : '-';

  const loadCpms = useCallback(
    (instSfidNumber: string) => {
      getCpmsSiteByInstitution(auth, instSfidNumber)
        // 中文注释：后端「尚未生成」返回 200 + data:null，这里 row=null 即正常未生成态。
        .then((row) => setCpmsSite(row))
        .catch((err) => {
          // 真错误（403/500/网络）才到这里：提示而非静默吞成 null，
          // 避免把「加载失败」误判为「未生成」而显示重新生成按钮；同时不清空已加载的二维码。
          notice.error(err, 'CPMS 安装码加载失败');
        });
    },
    [auth.access_token]
  );

  useEffect(() => {
    if (inst && inst.category === 'PUBLIC_SECURITY') {
      loadCpms(inst.sfid_number);
    } else {
      setCpmsSite(null);
    }
  }, [inst?.sfid_number, inst?.category, loadCpms]);

  const onDeleteAccount = async (accountName: string) => {
    try {
      const grant = await runPasskeyChallengeGrant('INSTITUTION_DELETE_ACCOUNT', {
        target: sfidNumber,
        sfid_number: sfidNumber,
        account_name: accountName,
      });
      await deleteAccount(auth, sfidNumber, accountName, grant);
      notice.success(`账户 "${accountName}" 已删除`);
      load();
    } catch (err) {
      notice.error(err, '');
    }
  };

  const onGenerateCpms = async () => {
    if (!inst) return;
    setCpmsBusy(true);
    try {
      // 中文注释：grant 签名内容保持 {province,city,institution} 不变（不动冷钱包解码）；
      // 生成请求额外带机构自身 sfid_number，后端以它为写入键（= 详情页读取键），根治再次进入二维码丢失。
      const grantPayload = {
        province: inst.province,
        city: inst.city,
        institution: inst.institution_code,
      };
      const grant = await runPasskeyChallengeGrant('CPMS_ISSUE_INSTALL_CODE', grantPayload);
      const result = await generateCpmsInstallQr(
        auth,
        { ...grantPayload, sfid_number: inst.sfid_number },
        grant,
      );
      setCpmsSite({
        sfid_number: result.sfid_number,
        install_token_status: 'PENDING',
        status: 'PENDING',
        version: 1,
        qr1_payload: result.qr1_payload,
        admin_province: inst.province,
        city_name: inst.city,
        institution_code: inst.institution_code,
        institution_name: inst.institution_name ?? '',
        created_by: auth.admin_pubkey,
        created_at: new Date().toISOString(),
      });
      notice.success('CPMS 安装码已生成');
      // 中文注释:安装码直接交给 CPMS 初始化,不再经过中间注册回传。
      load();
      loadCpms(result.sfid_number);
    } catch (err) {
      notice.error(err, '');
    } finally {
      setCpmsBusy(false);
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
        extra={(() => {
          if (inst.category !== 'PUBLIC_SECURITY' || !canManageCpms) return null;
          if (!cpmsSite) {
            return (
              <Button type="primary" onClick={onGenerateCpms} loading={cpmsBusy}>
                生成 CPMS 安装码
              </Button>
            );
          }
          return null;
        })()}
      >
        <Row gutter={24}>
          <Col xs={24} md={cpmsSite ? 12 : 24}>
            <Descriptions column={1} size="small">
              <Descriptions.Item label="身份ID">
                <Typography.Text style={{ fontSize: 12, wordBreak: 'break-all' }}>
                  {inst.sfid_number}
                </Typography.Text>
              </Descriptions.Item>
              <Descriptions.Item label="全称">{inst.sfid_name || '-'}</Descriptions.Item>
              <Descriptions.Item label="简称">{inst.short_name || inst.institution_name || '-'}</Descriptions.Item>
              <Descriptions.Item label="行政区">{administrativeArea}</Descriptions.Item>
              <Descriptions.Item label="机构类型">
                {INSTITUTION_CODE_LABEL[inst.institution_code] || inst.institution_code}
                {inst.org_code ? ` / ${ORG_CODE_LABEL[inst.org_code] || inst.org_code}` : ''}
              </Descriptions.Item>
              <Descriptions.Item label="状态">
                <Tag color={inst.status === 'ACTIVE' ? 'green' : 'red'}>
                  {SUBJECT_STATUS_LABEL[inst.status] || inst.status}
                </Tag>
              </Descriptions.Item>
              <Descriptions.Item label="法定代表人姓名">
                {inst.legal_rep_name || <span style={{ color: '#999' }}>(未填写)</span>}
              </Descriptions.Item>
              <Descriptions.Item label="法定代表人身份ID">
                {inst.legal_rep_sfid_number ? (
                  <Typography.Text style={{ fontSize: 12, wordBreak: 'break-all' }}>
                    {inst.legal_rep_sfid_number}
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
          {inst.category === 'PUBLIC_SECURITY' && cpmsSite && (
            <Col xs={24} md={12}>
              <CpmsSitePanel
                auth={auth}
                site={cpmsSite}
                canWrite={canManageCpms}
                onChanged={() => loadCpms(inst.sfid_number)}
              />
            </Col>
          )}
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
          title={inst.institution_name ?? inst.short_name ?? '(未命名机构)'}
          subtitle={`身份ID：${inst.sfid_number}`}
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
                <DocumentLibrary
                  auth={auth}
                  sfidNumber={inst.sfid_number}
                  canWrite={canWrite}
                  createPasskeyChallengeGrant={runPasskeyChallengeGrant}
                />
              ),
            },
            {
              key: 'operations',
              label: '操作记录',
              content: <OperationRecords auth={auth} sfidNumber={inst.sfid_number} />,
            },
          ]}
        />

        <CreateAccountModal
          auth={auth}
          sfidNumber={inst.sfid_number}
          institutionName={inst.institution_name ?? ''}
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
              createPasskeyChallengeGrant={runPasskeyChallengeGrant}
              onBack={onBack}
              backLabel={backLabel}
            />
          ) : renderOfficialDetail()}
        </>
      )}
      <WuminSignatureModal
        title="公民钱包签名确认"
        open={!!securityModal}
        onCancel={() => {
          securityModal?.reject(new Error('已取消签名确认'));
          setSecurityModal(null);
          setSecurityCommitLoading(false);
        }}
        qrTitle="签名二维码"
        qrValue={securityModal?.signRequest}
        qrHint="使用联邦管理员冷钱包扫码签名"
        scannerHint="扫描冷钱包生成的签名回执二维码"
        scannerDisabled={securityCommitLoading}
        scannerLoading={securityCommitLoading}
        onDetected={handleSecuritySignedResponse}
        onScannerError={(msg) => notice.error(msg)}
      />
    </div>
  );
};
