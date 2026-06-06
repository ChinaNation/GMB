// 注册局视图 mode='system-settings'
// 和机构类省市视图保持一样的显示方式:
//   始终渲染一个 Card,按角色和选中状态切换内部 title/body/extra
//   用 useScope 自动处理角色锁定,不闪烁

import React, { useState } from 'react';
import { Button, Card, Space, Table, Tag, Typography } from 'antd';
import { useAuth } from '../hooks/useAuth';
import { useScope } from '../hooks/useScope';
import type { SfidCityItem } from '../china/api';
import type { OperatorRow } from './operators_api';
import { tryEncodeSs58 } from '../utils/ss58';
import { glassCardStyle, glassCardHeadStyle } from '../core/cardStyles';
import { MAX_SHI_ADMINS_PER_CITY, sameHexPubkey } from './shengAdminUtils';
import type { ShengAdminSharedState } from './shengAdminUtils';
import { AddOperatorModal } from './AddOperatorModal';
import { ShengAdminSubTab } from './ShengAdminSubTab';
import { Passkey } from './Passkey';

interface ProvinceDetailViewProps {
  state: ShengAdminSharedState;
}

function makeCenteredTitle(center: React.ReactNode, back?: () => void) {
  return (
    <div style={{ position: 'relative', display: 'flex', alignItems: 'center', minHeight: 32 }}>
      {back && (
        <Button type="link" style={{ paddingLeft: 0 }} onClick={back}>
          ← 返回
        </Button>
      )}
      <span style={{ position: 'absolute', left: '50%', transform: 'translateX(-50%)' }}>
        {center}
      </span>
    </div>
  );
}

export function ProvinceDetailView({ state }: ProvinceDetailViewProps) {
  const { auth } = useAuth();
  const scope = useScope(auth);
  const {
    shengAdmins,
    shengAdminsLoading,
    selectedShengAdmin,
    setSelectedShengAdmin,
    selectedCity,
    setSelectedCity,
    adminDetailTab,
    setAdminDetailTab,
    operators,
    operatorsLoading,
    operatorListPage,
    setOperatorListPage,
    operatorCities,
    operatorCitiesLoading,
    setAddOperatorOpen,
    onUpdateOperator,
    onDeleteOperator,
  } = state;

  // scope 自动锁定省和市
  const [pickedProvince, setPickedProvince] = useState<string | null>(null);
  const effectiveProvince = pickedProvince ?? scope.lockedProvince;
  const effectiveCity = selectedCity ?? scope.lockedCity;

  // 点击省份时同步 selectedShengAdmin
  const onPickProvince = (provinceName: string) => {
    const row = shengAdmins.find((r) => r.province === provinceName);
    if (row) {
      setSelectedShengAdmin(row);
      setPickedProvince(provinceName);
    }
  };

  // 当前省的管理员
  const operatorsForProvince = selectedShengAdmin ? operators : [];

  // 中文注释:后端按登录省域二次校验;前端只负责把联邦管理员入口打开。
  const canEditOperators = scope.canWrite && auth?.role === 'FEDERAL_ADMIN';
  // sub-tab(仅在省详情内显示)
  const subTabs: Array<{ key: 'operators' | 'sheng-admin'; label: string }> = [
    { key: 'operators', label: '市级管理员列表' },
    { key: 'sheng-admin', label: '联邦管理员列表' },
  ];

  // ── 决定 title / body / extra ──
  let title: React.ReactNode;
  let extra: React.ReactNode = null;
  let body: React.ReactNode;

  if (!effectiveProvince) {
    // ── 全国省份网格(ADR-008 全局视图,跨省按钮置灰) ──
    title = '省份列表';
    body = shengAdminsLoading ? (
      <Typography.Text type="secondary">加载中...</Typography.Text>
    ) : (
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))', gap: 12 }}>
        {shengAdmins.map((row) => (
          <div
            key={`${row.province}-${row.admin_pubkey}`}
            onClick={() => onPickProvince(row.province)}
            style={{
              padding: 18, borderRadius: 12,
              border: '1px solid rgba(15,23,42,0.22)',
              background: 'rgba(226,232,240,0.55)',
              boxShadow: '0 2px 8px rgba(0,0,0,0.08)',
              cursor: 'pointer', transition: 'all 0.2s ease',
              textAlign: 'center',
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
            <div style={{ fontSize: 16, fontWeight: 600, color: '#0f172a' }}>{row.province}</div>
          </div>
        ))}
      </div>
    );
  } else if (!effectiveCity) {
    // ── 省详情:市级管理员列表入口 + sub-tab ──
    const canGoBack = !scope.skipProvinceList;
    title = makeCenteredTitle(
      effectiveProvince,
      canGoBack ? () => { setPickedProvince(null); setSelectedShengAdmin(null); } : undefined,
    );
    body = (
      <>
        <SubTabBar tabs={subTabs} active={adminDetailTab} onChange={(key) => {
          setAdminDetailTab(key);
          if (key === 'operators') setSelectedCity(null);
        }} />
        {adminDetailTab === 'operators' ? (
          <CityGrid
            cities={operatorCities.filter((c) => c.code !== '000')}
            citiesLoading={operatorCitiesLoading || (!selectedShengAdmin && operatorCities.length === 0)}
            operators={operatorsForProvince}
            onSelectCity={setSelectedCity}
          />
        ) : selectedShengAdmin ? (
          <ShengAdminSubTab
            selectedShengAdmin={selectedShengAdmin}
            shengAdmins={shengAdmins}
            shengAdminsLoading={shengAdminsLoading}
            refreshShengAdmins={state.refreshShengAdmins}
            runSecuredAction={state.runSecuredAction}
          />
        ) : null}
      </>
    );
  } else {
    // ── 市详情:该市市级管理员列表 + sub-tab ──
    const canGoBack = !scope.skipCityList;
    title = makeCenteredTitle(
      `${effectiveProvince} · ${effectiveCity}`,
      canGoBack ? () => { setSelectedCity(null); setOperatorListPage(1); } : undefined,
    );
    body = (
      <>
        <SubTabBar tabs={subTabs} active={adminDetailTab} onChange={(key) => {
          setAdminDetailTab(key);
          // 中文注释:联邦管理员列表属于省级页,从市详情点击时必须回到省级上下文。
          if (key === 'sheng-admin') setSelectedCity(null);
          if (key === 'operators' && !scope.skipCityList) setSelectedCity(null);
        }} />
        {adminDetailTab === 'operators' ? (
          <CityOperatorsView
            canEditOperators={canEditOperators}
            operators={operatorsForProvince.filter((op) => op.city === effectiveCity)}
            operatorsLoading={operatorsLoading}
            operatorListPage={operatorListPage}
            setOperatorListPage={setOperatorListPage}
            setAddOperatorOpen={setAddOperatorOpen}
            onUpdateOperator={onUpdateOperator}
            onDeleteOperator={onDeleteOperator}
          />
        ) : selectedShengAdmin ? (
          <ShengAdminSubTab
            selectedShengAdmin={selectedShengAdmin}
            shengAdmins={shengAdmins}
            shengAdminsLoading={shengAdminsLoading}
            refreshShengAdmins={state.refreshShengAdmins}
            runSecuredAction={state.runSecuredAction}
          />
        ) : null}
      </>
    );
  }

  return (
    <>
      <Card
        title={title}
        extra={extra}
        bordered={false}
        style={glassCardStyle}
        headStyle={glassCardHeadStyle}
      >
        {body}
      </Card>
      <AddOperatorModal state={state} />
    </>
  );
}

// ── Sub-tab 按钮条 ──

function SubTabBar({ tabs, active, onChange }: {
  tabs: Array<{ key: string; label: string }>;
  active: string;
  onChange: (key: 'operators' | 'sheng-admin') => void;
}) {
  return (
    <div style={{
      display: 'flex', gap: 8, padding: 6,
      background: 'rgba(15,23,42,0.06)', borderRadius: 10,
      border: '1px solid rgba(15,23,42,0.12)',
      width: 'fit-content', marginBottom: 16,
    }}>
      {tabs.map((t) => (
        <button
          key={t.key}
          onClick={() => onChange(t.key as 'operators' | 'sheng-admin')}
          style={{
            padding: '6px 18px', borderRadius: 8, border: 'none',
            cursor: 'pointer', fontSize: 13, fontWeight: 500,
            transition: 'all 0.2s ease',
            ...(active === t.key
              ? { background: 'linear-gradient(135deg, #0d9488, #0f766e)', color: '#fff', boxShadow: '0 2px 6px rgba(13,148,136,0.35)' }
              : { background: 'rgba(255,255,255,0.7)', color: 'rgba(15,23,42,0.75)' }),
          }}
        >
          {t.label}
        </button>
      ))}
    </div>
  );
}

// ── 市级管理员列表的城市入口网格 ──

function CityGrid({ cities, citiesLoading, operators, onSelectCity }: {
  cities: SfidCityItem[];
  citiesLoading: boolean;
  operators: OperatorRow[];
  onSelectCity: (city: string) => void;
}) {
  return citiesLoading ? (
    <Typography.Text type="secondary">加载中...</Typography.Text>
  ) : cities.length === 0 ? (
    <Typography.Text type="secondary">暂无城市数据</Typography.Text>
  ) : (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))', gap: 12 }}>
      {cities.map((city) => {
        const count = operators.filter((op) => op.city === city.name).length;
        return (
          <div
            key={city.code}
            onClick={() => onSelectCity(city.name)}
            style={{
              padding: 18, borderRadius: 12,
              border: '1px solid rgba(15,23,42,0.22)',
              background: 'rgba(226,232,240,0.55)',
              boxShadow: '0 2px 8px rgba(0,0,0,0.08)',
              cursor: 'pointer', transition: 'all 0.2s ease',
              textAlign: 'center',
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
            <div style={{ fontSize: 16, fontWeight: 600, color: '#0f172a' }}>{city.name}</div>
            <Tag
              color={count >= MAX_SHI_ADMINS_PER_CITY ? 'red' : 'teal'}
              style={{ marginTop: 6 }}
            >
              {count} / {MAX_SHI_ADMINS_PER_CITY}
            </Tag>
          </div>
        );
      })}
    </div>
  );
}

// ── 某市的管理员列表 ──

function CityOperatorsView({ canEditOperators, operators, operatorsLoading, operatorListPage, setOperatorListPage, setAddOperatorOpen, onUpdateOperator, onDeleteOperator }: {
  canEditOperators: boolean;
  operators: OperatorRow[];
  operatorsLoading: boolean;
  operatorListPage: number;
  setOperatorListPage: (v: number) => void;
  setAddOperatorOpen: (v: boolean) => void;
  onUpdateOperator: (row: OperatorRow) => void;
  onDeleteOperator: (row: OperatorRow) => void;
}) {
  const { auth } = useAuth();
  // 中文注释:本列表已经按当前市过滤,所以长度就是该市市级管理员数量。
  const cityLimitReached = operators.length >= MAX_SHI_ADMINS_PER_CITY;
  return (
    <>
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: 12,
      }}>
        <Typography.Text type="secondary">
          本市市级管理员：{operators.length} / {MAX_SHI_ADMINS_PER_CITY}
        </Typography.Text>
        <div>
          {canEditOperators && (
            <Button
              type="primary"
              disabled={cityLimitReached}
              title={cityLimitReached ? `本市市级管理员已满 ${MAX_SHI_ADMINS_PER_CITY} 人` : undefined}
              onClick={() => setAddOperatorOpen(true)}
            >
              新增市级管理员
            </Button>
          )}
        </div>
      </div>
      <Table<OperatorRow>
        rowKey={(r) => `${r.id}-${r.admin_pubkey}`}
        loading={operatorsLoading}
        dataSource={operators}
        pagination={{
          pageSize: 10, current: operatorListPage,
          onChange: (page) => setOperatorListPage(page),
          showSizeChanger: false,
          showTotal: (total) => `共 ${total} 条`,
        }}
        columns={[
          { title: '序号', width: 70, align: 'center', render: (_v, _row, index) => (operatorListPage - 1) * 10 + index + 1 },
          { title: '姓名', dataIndex: 'admin_name', align: 'center', width: 160 },
          { title: '账户', dataIndex: 'admin_pubkey', align: 'center', render: (v: string) => tryEncodeSs58(v) },
          {
            title: '操作', width: 260, align: 'center' as const,
            render: (_v: unknown, row: OperatorRow) => (
              <Space>
                {canEditOperators ? <Button size="small" onClick={() => onUpdateOperator(row)}>编辑</Button> : null}
                {canEditOperators ? <Button size="small" danger onClick={() => onDeleteOperator(row)}>删除</Button> : null}
                <Passkey
                  size="small"
                  disabled={!sameHexPubkey(row.admin_pubkey, auth?.admin_pubkey)}
                />
              </Space>
            ),
          },
        ]}
      />
    </>
  );
}
