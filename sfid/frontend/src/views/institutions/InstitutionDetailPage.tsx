// 中文注释:机构详情页(调度器)。
// 按 category 分派给不同布局模块:
//   - PRIVATE_INSTITUTION → PrivateInstitutionLayout(三板块:机构信息+账户列表+资料库)
//   - PUBLIC_SECURITY / GOV_INSTITUTION → 默认布局(机构信息+CPMS+账户列表)
// 修改某类机构的布局只需改对应模块,不影响其他类型。

import React, { useCallback, useEffect, useState } from 'react';
import { Button, Card, Col, Descriptions, message, Row, Typography } from 'antd';
import {
  deleteAccount,
  getCpmsSiteByInstitution,
  getInstitution,
  type InstitutionDetail,
} from '../../api/institution';
import {
  generateCpmsInstitutionSfid,
  type AdminAuth,
  type CpmsSiteRow,
} from '../../api/client';
import { AccountList } from './AccountList';
import { CpmsRegisterModal } from './CpmsRegisterModal';
import { CpmsSitePanel } from './CpmsSitePanel';
import { CreateAccountModal } from './CreateAccountModal';
import { PrivateInstitutionLayout } from './PrivateInstitutionLayout';

interface Props {
  auth: AdminAuth;
  sfidId: string;
  canWrite: boolean;
  onBack: () => void;
}

export const InstitutionDetailPage: React.FC<Props> = ({ auth, sfidId, canWrite, onBack }) => {
  const [detail, setDetail] = useState<InstitutionDetail | null>(null);
  const [loading, setLoading] = useState(false);
  const [createAccountOpen, setCreateAccountOpen] = useState(false);

  // ── CPMS 站点状态(仅公安局机构使用) ──
  const [cpmsSite, setCpmsSite] = useState<CpmsSiteRow | null>(null);
  const [cpmsBusy, setCpmsBusy] = useState(false);
  const [cpmsRegisterOpen, setCpmsRegisterOpen] = useState(false);

  const load = useCallback(() => {
    setLoading(true);
    getInstitution(auth, sfidId)
      .then(setDetail)
      .catch((err) => message.error(err instanceof Error ? err.message : '加载机构详情失败'))
      .finally(() => setLoading(false));
  }, [auth.access_token, sfidId]);

  useEffect(() => {
    load();
  }, [load]);

  const inst = detail?.institution;
  const accounts = detail?.accounts || [];

  const loadCpms = useCallback(
    (instSfidId: string) => {
      getCpmsSiteByInstitution(auth, instSfidId)
        .then((row) => setCpmsSite(row))
        .catch((err) => {
          // CPMS 站点不存在是正常情况(尚未生成),404 静默降级;其他错误提示用户
          const msg = err instanceof Error ? err.message : String(err);
          if (!msg.includes('404') && !msg.includes('not found')) {
            message.warning('CPMS 站点加载失败');
          }
          setCpmsSite(null);
        });
    },
    [auth.access_token]
  );

  useEffect(() => {
    if (inst && inst.category === 'PUBLIC_SECURITY') {
      loadCpms(inst.sfid_id);
    } else {
      setCpmsSite(null);
    }
  }, [inst?.sfid_id, inst?.category, loadCpms]);

  const onDeleteAccount = async (accountName: string) => {
    try {
      await deleteAccount(auth, sfidId, accountName);
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
      const result = await generateCpmsInstitutionSfid(auth, {
        province: inst.province,
        city: inst.city,
        institution: inst.institution_code,
        institution_name: inst.institution_name,
      });
      setCpmsSite({
        site_sfid: result.site_sfid,
        install_token_status: 'PENDING',
        status: 'PENDING',
        version: 1,
        qr1_payload: result.qr1_payload,
        admin_province: inst.province,
        city_name: inst.city,
        institution_code: inst.institution_code,
        institution_name: inst.institution_name,
        created_by: auth.admin_pubkey,
        created_at: new Date().toISOString(),
      });
      message.success('CPMS 安装二维码已生成');
      loadCpms(inst.sfid_id);
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
            <PrivateInstitutionLayout
              auth={auth}
              detail={detail}
              canWrite={canWrite}
              loading={loading}
              onReload={load}
              onDeleteAccount={onDeleteAccount}
            />
          ) : (
            <>
              {/* ── 公安局 / 公权机构:默认布局 ── */}
              <Card
                title={
                  <span style={{ fontSize: 18, fontWeight: 600 }}>{inst.institution_name}</span>
                }
                extra={(() => {
                  if (inst.category !== 'PUBLIC_SECURITY' || !canWrite) return null;
                  if (!cpmsSite) {
                    return (
                      <Button type="primary" onClick={onGenerateCpms} loading={cpmsBusy}>
                        生成 CPMS 安装二维码
                      </Button>
                    );
                  }
                  const tokenOk = cpmsSite.install_token_status !== 'REVOKED';
                  if (cpmsSite.status === 'PENDING' && tokenOk) {
                    return (
                      <Button type="primary" onClick={() => setCpmsRegisterOpen(true)}>
                        扫描 QR2 注册
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
                          {inst.sfid_id}
                        </Typography.Text>
                      </Descriptions.Item>
                      <Descriptions.Item label="省份">{inst.province}</Descriptions.Item>
                      <Descriptions.Item label="城市">{inst.city}</Descriptions.Item>
                      <Descriptions.Item label="A3 类型">{inst.a3}</Descriptions.Item>
                      <Descriptions.Item label="机构代码">{inst.institution_code}</Descriptions.Item>
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
                        canWrite={canWrite}
                        onChanged={() => loadCpms(inst.sfid_id)}
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
                sfidId={inst.sfid_id}
                institutionName={inst.institution_name}
                existingAccounts={accounts}
                open={createAccountOpen}
                onCancel={() => setCreateAccountOpen(false)}
                onCreated={() => {
                  setCreateAccountOpen(false);
                  load();
                }}
              />

              <CpmsRegisterModal
                auth={auth}
                open={cpmsRegisterOpen}
                onClose={() => setCpmsRegisterOpen(false)}
                onRegistered={() => {
                  setCpmsRegisterOpen(false);
                  loadCpms(inst.sfid_id);
                }}
              />
            </>
          )}
        </>
      )}
    </div>
  );
};
