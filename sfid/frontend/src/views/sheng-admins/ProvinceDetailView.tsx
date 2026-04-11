// 省份详情视图 mode='system-settings'(从 ShengAdminsView.tsx 拆分)
// 包含:KeyAdmin 省份网格 / 机构详情页(sub-tab:市级管理员列表 / 省级管理员)

import { Button, Card, Space, Table, Typography } from 'antd';
import { useAuth } from '../../hooks/useAuth';
import type { OperatorRow } from '../../api/client';
import { tryEncodeSs58 } from '../../utils/ss58';
import { glassCardStyle, glassCardHeadStyle } from '../../components/App';
import { sameHexPubkey } from './shengAdminUtils';
import type { ShengAdminSharedState } from './shengAdminUtils';
import { AddOperatorModal } from './AddOperatorModal';
import { SuperAdminSubTab } from './SuperAdminSubTab';

interface ProvinceDetailViewProps {
  state: ShengAdminSharedState;
}

export function ProvinceDetailView({ state }: ProvinceDetailViewProps) {
  const { auth } = useAuth();
  const {
    shengAdmins,
    shengAdminsLoading,
    selectedShengAdmin,
    setSelectedShengAdmin,
    adminDetailTab,
    setAdminDetailTab,
    replaceSuperLoading,
    operators,
    operatorsLoading,
    operatorListPage,
    setOperatorListPage,
    setAddOperatorOpen,
    replaceSuperForm,
    onReplaceShengAdmin,
    onToggleOperatorStatus,
    onUpdateOperator,
    onDeleteOperator,
    setAccountScanTarget,
  } = state;

  return (
    <>
      {selectedShengAdmin ? (
        // ── 机构详情页(sub-tab:市级管理员列表 / 省级管理员) ──
        (() => {
          const isKeyAdmin = auth?.role === 'KEY_ADMIN';
          const isSelf = auth ? sameHexPubkey(selectedShengAdmin.admin_pubkey, auth.admin_pubkey) : false;
          const canEditOperators = isKeyAdmin || (auth?.role === 'SHENG_ADMIN' && isSelf);
          const canReplaceThisAdmin = isKeyAdmin;
          const operatorsForThisAdmin = operators.filter((op) =>
            sameHexPubkey(op.created_by, selectedShengAdmin.admin_pubkey),
          );
          const subTabs: Array<{ key: 'operators' | 'super-admin'; label: string }> = [
            { key: 'operators', label: '市级管理员列表' },
            { key: 'super-admin', label: '省级管理员' },
          ];
          return (
            <Card
              bordered={false}
              style={glassCardStyle}
              headStyle={glassCardHeadStyle}
              title={
                <div style={{ position: 'relative', display: 'flex', alignItems: 'center', minHeight: 32 }}>
                  {isKeyAdmin && (
                    <Button type="link" style={{ paddingLeft: 0 }} onClick={() => setSelectedShengAdmin(null)}>
                      ← 返回省列表
                    </Button>
                  )}
                  <span style={{ position: 'absolute', left: '50%', transform: 'translateX(-50%)' }}>
                    {selectedShengAdmin.province}
                  </span>
                </div>
              }
            >
              <div
                style={{
                  display: 'flex',
                  gap: 8,
                  padding: 6,
                  background: 'rgba(15,23,42,0.06)',
                  borderRadius: 10,
                  border: '1px solid rgba(15,23,42,0.12)',
                  width: 'fit-content',
                  marginBottom: 16,
                }}
              >
                {subTabs.map((t) => (
                  <button
                    key={t.key}
                    onClick={() => setAdminDetailTab(t.key)}
                    style={{
                      padding: '6px 18px',
                      borderRadius: 8,
                      border: 'none',
                      cursor: 'pointer',
                      fontSize: 13,
                      fontWeight: 500,
                      transition: 'all 0.2s ease',
                      ...(adminDetailTab === t.key
                        ? {
                            background: 'linear-gradient(135deg, #0d9488, #0f766e)',
                            color: '#fff',
                            boxShadow: '0 2px 6px rgba(13,148,136,0.35)',
                          }
                        : {
                            background: 'rgba(255,255,255,0.7)',
                            color: 'rgba(15,23,42,0.75)',
                          }),
                    }}
                  >
                    {t.label}
                  </button>
                ))}
              </div>

              {adminDetailTab === 'operators' ? (
                <OperatorsSubTab
                  canEditOperators={canEditOperators}
                  operatorsForThisAdmin={operatorsForThisAdmin}
                  operatorsLoading={operatorsLoading}
                  operatorListPage={operatorListPage}
                  setOperatorListPage={setOperatorListPage}
                  setAddOperatorOpen={setAddOperatorOpen}
                  onToggleOperatorStatus={onToggleOperatorStatus}
                  onUpdateOperator={onUpdateOperator}
                  onDeleteOperator={onDeleteOperator}
                />
              ) : (
                <SuperAdminSubTab
                  selectedShengAdmin={selectedShengAdmin}
                  canReplaceThisAdmin={canReplaceThisAdmin}
                  replaceSuperLoading={replaceSuperLoading}
                  replaceSuperForm={replaceSuperForm}
                  onReplaceShengAdmin={onReplaceShengAdmin}
                  setAccountScanTarget={setAccountScanTarget}
                />
              )}
            </Card>
          );
        })()
      ) : (
        // ── KeyAdmin:注册局省份列表 ──
        <ProvinceGrid
          shengAdmins={shengAdmins}
          shengAdminsLoading={shengAdminsLoading}
          setSelectedShengAdmin={setSelectedShengAdmin}
        />
      )}

      {/* 新增市级管理员 Modal + 扫码弹窗 */}
      <AddOperatorModal state={state} />
    </>
  );
}

// ── 市级管理员列表 sub-tab ──

interface OperatorsSubTabProps {
  canEditOperators: boolean;
  operatorsForThisAdmin: OperatorRow[];
  operatorsLoading: boolean;
  operatorListPage: number;
  setOperatorListPage: (v: number) => void;
  setAddOperatorOpen: (v: boolean) => void;
  onToggleOperatorStatus: (row: OperatorRow) => Promise<void>;
  onUpdateOperator: (row: OperatorRow) => void;
  onDeleteOperator: (row: OperatorRow) => void;
}

function OperatorsSubTab({
  canEditOperators,
  operatorsForThisAdmin,
  operatorsLoading,
  operatorListPage,
  setOperatorListPage,
  setAddOperatorOpen,
  onToggleOperatorStatus,
  onUpdateOperator,
  onDeleteOperator,
}: OperatorsSubTabProps) {
  return (
    <Card
      type="inner"
      title="市级管理员列表"
      extra={
        canEditOperators ? (
          <Button type="primary" onClick={() => setAddOperatorOpen(true)}>
            新增市级管理员
          </Button>
        ) : null
      }
    >
      <Table<OperatorRow>
        rowKey={(r) => `${r.id}-${r.admin_pubkey}`}
        loading={operatorsLoading}
        dataSource={operatorsForThisAdmin}
        pagination={{
          pageSize: 10,
          current: operatorListPage,
          onChange: (page) => setOperatorListPage(page),
          showSizeChanger: false,
          showTotal: (total) => `共 ${total} 条`,
        }}
        columns={[
          {
            title: '序号',
            width: 70,
            align: 'center',
            render: (_v, _row, index) => (operatorListPage - 1) * 10 + index + 1,
          },
          { title: '市', dataIndex: 'city', align: 'center', width: 120 },
          { title: '姓名', dataIndex: 'admin_name', align: 'center', width: 160 },
          {
            title: '账户',
            dataIndex: 'admin_pubkey',
            align: 'center',
            render: (v: string) => tryEncodeSs58(v),
          },
          { title: '状态', dataIndex: 'status', align: 'center', width: 100 },
          ...(canEditOperators
            ? [
                {
                  title: '操作',
                  width: 220,
                  align: 'center' as const,
                  render: (_v: unknown, row: OperatorRow) => (
                    <Space>
                      <Button size="small" onClick={() => onUpdateOperator(row)}>
                        修改
                      </Button>
                      <Button size="small" onClick={() => onToggleOperatorStatus(row)}>
                        {row.status === 'ACTIVE' ? '停用' : '启用'}
                      </Button>
                      <Button size="small" danger onClick={() => onDeleteOperator(row)}>
                        删除
                      </Button>
                    </Space>
                  ),
                },
              ]
            : []),
        ]}
      />
    </Card>
  );
}

// ── KeyAdmin 省份网格 ──

interface ProvinceGridProps {
  shengAdmins: ShengAdminSharedState['shengAdmins'];
  shengAdminsLoading: boolean;
  setSelectedShengAdmin: ShengAdminSharedState['setSelectedShengAdmin'];
}

function ProvinceGrid({ shengAdmins, shengAdminsLoading, setSelectedShengAdmin }: ProvinceGridProps) {
  return (
    <Card title="省份列表" bordered={false} style={glassCardStyle} headStyle={glassCardHeadStyle}>
      {shengAdminsLoading ? (
        <Typography.Text type="secondary">加载中...</Typography.Text>
      ) : (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))', gap: 12 }}>
          {shengAdmins.map((row) => (
            <div
              key={`${row.province}-${row.admin_pubkey}`}
              onClick={() => setSelectedShengAdmin(row)}
              style={{
                padding: 18, borderRadius: 12,
                border: '1px solid rgba(15,23,42,0.22)',
                background: 'rgba(226,232,240,0.55)',
                boxShadow: '0 2px 8px rgba(0,0,0,0.08)',
                cursor: 'pointer', transition: 'all 0.2s ease',
                textAlign: 'center' as const,
              }}
              onMouseEnter={(e) => {
                (e.currentTarget as HTMLDivElement).style.background = 'rgba(13,148,136,0.22)';
                (e.currentTarget as HTMLDivElement).style.borderColor = 'rgba(13,148,136,0.55)';
              }}
              onMouseLeave={(e) => {
                (e.currentTarget as HTMLDivElement).style.background = 'rgba(226,232,240,0.55)';
                (e.currentTarget as HTMLDivElement).style.borderColor = 'rgba(15,23,42,0.22)';
              }}
            >
              <div style={{ fontSize: 16, fontWeight: 600, color: '#0f172a', textAlign: 'center' }}>
                {row.province}
              </div>
            </div>
          ))}
        </div>
      )}
    </Card>
  );
}
