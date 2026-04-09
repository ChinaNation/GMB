// 中文注释:机构详情页。从市详情页的机构列表点一行进入。
// 展示:机构头信息(含公安局 CPMS 站点管理)+ 账户列表 + "+ 新建账户"按钮。
//
// 任务卡 `20260408-sfid-public-security-cpms-embed`:
// 公安局机构的 CPMS state/handler 由本组件持有,`CpmsSitePanel` 只做展示。
// "生成 CPMS 安装二维码"按钮挂在机构信息 Card 的 `extra` 右侧,仅当无站点时显示。

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

  // 中文注释:公安局机构加载后再拉 CPMS 站点。404/空站点静默降级,
  // 不弹 toast,用户看到空右侧 + 标题右边的"生成"按钮即可。
  const loadCpms = useCallback(
    (instSfidId: string) => {
      getCpmsSiteByInstitution(auth, instSfidId)
        .then((row) => setCpmsSite(row))
        .catch((err) => {
          // eslint-disable-next-line no-console
          console.warn('getCpmsSiteByInstitution failed:', err);
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
      // 中文注释:用生成响应里的 qr1_payload 直接本地构造 CpmsSiteRow,
      // 立即展示二维码,不依赖 by-institution 端点(后端可能还没重启)。
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
      // 后台刷新拿完整字段(若后端 by-institution 可用)
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

      {inst && (
        <>
          {/* 任务卡 `20260408-sfid-public-security-cpms-embed`:
              机构信息 Card 采用左右分栏:左 = 机构字段 Descriptions,
              右 = CPMS 站点管理(仅公安局且已有站点)。
              "生成 CPMS 安装二维码"按钮挂在 Card.extra,仅公安局+无站点+可写时显示。 */}
          <Card
            title={
              <span style={{ fontSize: 18, fontWeight: 600 }}>{inst.institution_name}</span>
            }
            extra={(() => {
              // 中文注释:Card.extra 按状态渲染唯一主操作按钮:
              //   - 无站点 → 生成 CPMS 安装二维码
              //   - 有站点且 PENDING + 令牌未吊销 → 扫描 QR2 注册
              //   - 其他状态 → 不显示(操作按钮在右侧 Panel 下半部分)
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
                    {new Date(inst.created_at).toLocaleString()}
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
    </div>
  );
};
