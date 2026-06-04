// 中文注释:机构详情页(调度器)。
// 按 category 分派给不同布局模块:
//   - PRIVATE_INSTITUTION → PrivateDetailLayout(三板块:机构信息+账户列表+资料库)
//   - PUBLIC_SECURITY / GOV_INSTITUTION → 默认布局(机构信息+CPMS+账户列表)
// 修改某类机构的布局只需改对应模块,不影响其他类型。

import React, { useCallback, useEffect, useState } from 'react';
import { Button, Card, Col, Descriptions, message, Row, Tag, Typography } from 'antd';
import { A3_LABEL, INSTITUTION_CODE_LABEL } from '../subjects/labels';
import { getInstitution, type InstitutionDetail } from './api';
import { deleteAccount } from '../accounts/api';
import {
  generateCpmsInstallQr,
  getCpmsSiteByInstitution,
  type CpmsSiteRow,
} from '../cpms/api';
import type { AdminAuth } from '../auth/types';
import { AccountList } from '../accounts/AccountList';
import { CpmsSitePanel } from '../cpms/CpmsSitePanel';
import { CreateAccountModal } from '../accounts/CreateAccountModal';
import { PrivateDetailLayout } from '../private/PrivateDetailLayout';
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

interface Props {
  auth: AdminAuth;
  sfidNumber: string;
  canWrite: boolean;
  onBack: () => void;
}

const INSTITUTION_CHAIN_STATUS_LABEL: Record<string, string> = {
  NOT_REGISTERED: '未注册',
  REGISTERED: '已注册',
  REVOKED_ON_CHAIN: '已注销',
};

type SecurityModalState = {
  actionId: string;
  signRequest: string;
  passkeyAssertion: unknown;
  resolve: (value: AdminSecurityGrantOutput) => void;
  reject: (reason?: unknown) => void;
};

export const GovDetailPage: React.FC<Props> = ({ auth, sfidNumber, canWrite, onBack }) => {
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
    getInstitution(auth, sfidNumber)
      .then((next) => {
        setDetail(next);
        writeCachedInstitutionDetail(detailCacheKey, next);
      })
      .catch(() => { /* 静默：后台刷新失败不弹窗 */ })
      .finally(() => {
        if (!cached) setLoading(false);
      });
  }, [auth.access_token, detailCacheKey, sfidNumber]);

  useEffect(() => {
    load();
  }, [load]);

  const runPasskeyChallengeGrant = async (
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

  const inst = detail?.institution;
  const accounts = detail?.accounts || [];
  const canManageCpms = canWrite && auth.role === 'SHENG_ADMIN';

  const loadCpms = useCallback(
    (instSfidNumber: string) => {
      getCpmsSiteByInstitution(auth, instSfidNumber)
        .then((row) => setCpmsSite(row))
        .catch(() => {
          // 静默：后台刷新失败不弹窗（404 是正常场景——尚未生成）
          setCpmsSite(null);
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
      message.success(`账户 "${accountName}" 已删除`);
      load();
    } catch (err) {
      message.error(err instanceof Error ? err.message : '删除账户失败');
    }
  };

  const onGenerateCpms = async () => {
    if (!inst) return;
    setCpmsBusy(true);
    try {
      const payload = {
        province: inst.province,
        city: inst.city,
        institution: inst.institution_code,
      };
      const grant = await runPasskeyChallengeGrant('CPMS_ISSUE_INSTALL_CODE', payload);
      const result = await generateCpmsInstallQr(auth, payload, grant);
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
      message.success('CPMS 安装码已生成');
      // 中文注释:安装码直接交给 CPMS 初始化,不再经过中间注册回传。
      load();
      loadCpms(result.sfid_number);
    } catch (err) {
      message.error(err instanceof Error ? err.message : '生成失败');
    } finally {
      setCpmsBusy(false);
    }
  };

  return (
    <div>
      <div style={{ marginBottom: 12 }}>
        <Button type="link" onClick={onBack} style={{ paddingLeft: 0 }}>
          ← 返回机构列表
        </Button>
      </div>

      {loading && !inst && <Typography.Text type="secondary">加载中...</Typography.Text>}

      {inst && detail && (
        <>
          {/* ── 私权机构:三板块布局(独立模块) ── */}
          {inst.category === 'PRIVATE_INSTITUTION' ? (
            <PrivateDetailLayout
              auth={auth}
              detail={detail}
              canWrite={canWrite}
              loading={loading}
              onReload={load}
              onDeleteAccount={onDeleteAccount}
              createPasskeyChallengeGrant={runPasskeyChallengeGrant}
            />
          ) : (
            <>
              {/* ── 公安局 / 公权机构:默认布局 ── */}
              <Card
                title={
                  <span style={{ fontSize: 18, fontWeight: 600 }}>
                    {inst.institution_name ?? '(未命名机构)'}
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
                style={{ marginBottom: 16 }}
              >
                <Row gutter={24}>
                  <Col xs={24} md={cpmsSite ? 12 : 24}>
                    <Descriptions column={1} size="small">
                      <Descriptions.Item label="机构 SFID">
                        <Typography.Text code style={{ fontSize: 12, wordBreak: 'break-all' }}>
                          {inst.sfid_number}
                        </Typography.Text>
                      </Descriptions.Item>
                      <Descriptions.Item label="省份">{inst.province}</Descriptions.Item>
                      <Descriptions.Item label="城市">{inst.city}</Descriptions.Item>
                      <Descriptions.Item label="A3 类型">{inst.a3}/{A3_LABEL[inst.a3] || inst.a3}</Descriptions.Item>
                      <Descriptions.Item label="机构代码">{inst.institution_code}/{INSTITUTION_CODE_LABEL[inst.institution_code] || inst.institution_code}</Descriptions.Item>
                      <Descriptions.Item label="链上状态">
                        <Tag>{INSTITUTION_CHAIN_STATUS_LABEL[inst.chain_status] || inst.chain_status}</Tag>
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
          )}
        </>
      )}
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
        qrHint="使用省管理员冷钱包扫码签名"
        scannerHint="扫描冷钱包生成的签名回执二维码"
        scannerDisabled={securityCommitLoading}
        scannerLoading={securityCommitLoading}
        onDetected={handleSecuritySignedResponse}
        onScannerError={(msg) => message.error(msg)}
      />
    </div>
  );
};
