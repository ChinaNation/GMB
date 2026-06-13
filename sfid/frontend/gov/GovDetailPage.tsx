// 中文注释:机构详情页(调度器)。
// 按 category 分派给不同布局模块:
//   - PRIVATE_INSTITUTION → PrivateDetailLayout(三板块:机构信息+账户列表+资料库)
//   - PUBLIC_SECURITY / GOV_INSTITUTION → 默认布局(机构信息+CPMS+账户列表)
// 修改某类机构的布局只需改对应模块,不影响其他类型。

import React, { useCallback, useEffect, useState } from 'react';
import { Button, Card, Col, Descriptions, Row, Table, Tag, Typography } from 'antd';
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
import { tryEncodeSs58 } from '../utils/ss58';
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
import { adminRequest } from '../utils/http';

interface Props {
  auth: AdminAuth;
  sfidNumber: string;
  canWrite: boolean;
  /** 不传则隐藏返回按钮(注册局 tab 里市管理员直接进详情、或联邦注册局无上一级时)。 */
  onBack?: () => void;
  /** 返回按钮文案,默认「返回机构列表」。 */
  backLabel?: string;
  /**
   * 详情数据加载覆盖。不传则走默认 getInstitution(auth, sfidNumber)(带 scope 校验)。
   * 联邦注册局走 scope-bypass 的 getFederalRegistry,通过此 prop 注入。
   */
  loadDetail?: () => Promise<InstitutionDetail>;
  /**
   * 注册局机构详情页:在机构信息卡与账户列表卡之间内嵌的管理员列表。
   * 普通机构不传 → 零行为变化。
   */
  adminListSection?: React.ReactNode;
}

const SUBJECT_STATUS_LABEL: Record<string, string> = {
  ACTIVE: '正常',
  REVOKED: '已注销',
};

type AuditLogEntry = {
  seq: number;
  action: string;
  actor_pubkey: string;
  target_pubkey?: string | null;
  /** 结构化事实字段(后端 append_audit_log 只存事实);旧文本行/异常值回退 string */
  detail: Record<string, unknown> | string;
  created_at: string;
};

// 中文注释:审计日志操作类型中文映射(代码不上前端)。
// 单一来源 = 后端 append_audit_log 各调用点的 action 字面量(共 10 个);
// 后端新增 action 必须同步补这里,未知值回退显示原标识兜底。
const AUDIT_ACTION_LABEL: Record<string, string> = {
  CPMS_INSTALL_QR_GENERATE: '生成 CPMS 安装码',
  CPMS_INSTALL_QR_REISSUE: '重新生成 CPMS 安装码',
  CPMS_KEYS_STATUS_UPDATE: 'CPMS 授权状态变更',
  CPMS_KEYS_DELETE: '删除 CPMS 授权',
  CPMS_STATUS_EXPORT_IMPORT: '导入 CPMS 年度报告',
  CPMS_ARCHIVE_VERIFY: 'CPMS 档案码核验',
  CITIZEN_BIND: '公民身份ID绑定',
  PUBLIC_IDENTITY_SEARCH: '公开身份查询',
  APP_VOTERS_COUNT: 'App 选民人数查询',
  APP_VOTE_CREDENTIAL: 'App 投票凭证签发',
};

// 中文注释:审计详情"事实字段"的人话翻译(代码不上前端)。
// 后端 detail 只存结构化事实(键小写蛇形,值为系统原值),展示翻译全在这里;
// 后端新增字段须同步补键名映射,未知键回退「键名: 值」兜底。
const AUDIT_DETAIL_KEY_LABEL: Record<string, string> = {
  city: '市',
  institution: '机构',
  archive_no: '档案号',
  found: '查询命中',
  request_id: '请求ID',
  actor_ip: '来源IP',
  eligible_total: '选民总数',
  mode: '绑定方式',
  sfid_number: '身份ID',
  proposal_id: '提案ID',
  eligible: '有选举权',
  year: '年度',
  batch: '批次',
  result: '结果',
  message: '说明',
  status: '状态',
  reason: '原因',
  updates: '更新条数',
  wallet_replaced: '更换投票账户数',
  releases: '解除绑定数',
  unmatched_bindings: '未匹配绑定数',
  unmatched_releases: '未匹配解除数',
};

// 枚举值翻译:按键名选择值映射,机构代码复用全局映射
const AUDIT_DETAIL_VALUE_LABEL: Record<string, Record<string, string>> = {
  institution: INSTITUTION_CODE_LABEL,
  status: { PENDING: '待安装', ACTIVE: '已启用', DISABLED: '已禁用', REVOKED: '已吊销' },
  mode: { create: '新增绑定', replace: '更换绑定' },
  result: { SUCCESS: '成功', FAILED: '失败' },
};

function formatAuditDetailValue(key: string, value: unknown): string | null {
  if (value === null || value === undefined || value === '') return null;
  if (typeof value === 'boolean') return value ? '是' : '否';
  const text = String(value);
  return AUDIT_DETAIL_VALUE_LABEL[key]?.[text] ?? text;
}

/** 结构化事实 → 人话(「市：锦程市；机构：政府」);旧文本行原样兜底。 */
function formatAuditDetail(detail: AuditLogEntry['detail']): string {
  if (typeof detail === 'string') return detail;
  if (!detail || typeof detail !== 'object') return '';
  const parts: string[] = [];
  for (const [key, value] of Object.entries(detail)) {
    const text = formatAuditDetailValue(key, value);
    if (text === null) continue;
    parts.push(`${AUDIT_DETAIL_KEY_LABEL[key] ?? key}：${text}`);
  }
  return parts.join('；');
}

const OperationRecords: React.FC<{ auth: AdminAuth; sfidNumber: string }> = ({ auth, sfidNumber }) => {
  const [rows, setRows] = useState<AuditLogEntry[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    adminRequest<AuditLogEntry[]>(
      `/api/v1/admin/audit-logs?keyword=${encodeURIComponent(sfidNumber)}&limit=20`,
      auth,
    )
      .then((next) => {
        if (!cancelled) setRows(next);
      })
      .catch(() => {
        if (!cancelled) setRows([]);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [auth.access_token, sfidNumber]);

  return (
    <Card type="inner" title={`操作记录(${rows.length})`}>
      <Table<AuditLogEntry>
        rowKey="seq"
        loading={loading}
        dataSource={rows}
        pagination={rows.length > 10 ? { pageSize: 10 } : false}
        columns={[
          {
            title: '操作',
            dataIndex: 'action',
            width: 160,
            render: (v: string) => AUDIT_ACTION_LABEL[v] || v,
          },
          {
            title: '操作者账户',
            dataIndex: 'actor_pubkey',
            width: 240,
            // 中文注释:公钥是系统的,SS58 地址才是给人看的;完整显示不截断,允许换行
            render: (v: string) => (
              <Typography.Text style={{ fontSize: 12, fontFamily: 'monospace', wordBreak: 'break-all' }}>
                {tryEncodeSs58(v) || v}
              </Typography.Text>
            ),
          },
          {
            title: '详情',
            dataIndex: 'detail',
            ellipsis: true,
            render: (v: AuditLogEntry['detail']) => formatAuditDetail(v),
          },
          {
            title: '时间',
            dataIndex: 'created_at',
            width: 170,
            render: (v: string) => new Date(v).toLocaleString('zh-CN'),
          },
        ]}
      />
    </Card>
  );
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

  return (
    <div>
      {onBack && (
        <div style={{ marginBottom: 12 }}>
          <Button type="link" onClick={onBack} style={{ paddingLeft: 0 }}>
            ← {backLabel ?? '返回机构列表'}
          </Button>
        </div>
      )}

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

              {/* 注册局机构详情页:管理员列表嵌在机构信息与账户列表之间 */}
              {adminListSection && (
                <div style={{ marginBottom: 16 }}>{adminListSection}</div>
              )}

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

	              <div style={{ marginTop: 16 }}>
	                <DocumentLibrary
	                  auth={auth}
	                  sfidNumber={inst.sfid_number}
	                  canWrite={canWrite}
	                  createPasskeyChallengeGrant={runPasskeyChallengeGrant}
	                />
	              </div>

	              <div style={{ marginTop: 16 }}>
	                <OperationRecords auth={auth} sfidNumber={inst.sfid_number} />
	              </div>

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
