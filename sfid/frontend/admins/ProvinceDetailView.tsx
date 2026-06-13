// 注册局机构详情视图(两个顶级 tab 各一个):
//   - CityRegistryView(市注册局 tab):联邦管理员 城市网格→点市→该市注册局机构详情页;
//                                      市管理员 直接进本市注册局机构详情页。
//   - FederalRegistryView(联邦注册局 tab):全国唯一联邦注册局机构详情页。
// 两者 leaf 都是「机构详情页 + 内嵌管理员列表」(GovDetailPage.adminListSection),
// 管理员列表渲染在机构信息卡与账户列表卡之间。数据由 FederalAdminsView 统一加载。

import React, { useCallback } from 'react';
import { Button, Card, Space, Table, Tag, Typography } from 'antd';
import { useAuth } from '../hooks/useAuth';
import { useScope } from '../hooks/useScope';
import type { SfidCityItem } from '../china/api';
import type { CityAdminRow } from './city_admins_api';
import { tryEncodeSs58 } from '../utils/ss58';
import { glassCardStyle, glassCardHeadStyle } from '../core/cardStyles';
import { MAX_CITY_ADMINS_PER_CITY, sameHexPubkey } from './adminUtils';
import type { FederalAdminSharedState } from './adminUtils';
import { AddCityAdminModal } from './AddCityAdminModal';
import { FederalAdminSubTab } from './FederalAdminSubTab';
import { Passkey } from './Passkey';
import { GovDetailPage } from '../gov/GovDetailPage';
import { getFederalRegistry } from '../gov/api';

interface RegistryViewProps {
  state: FederalAdminSharedState;
}

function makeCenteredTitle(center: React.ReactNode) {
  return (
    <div style={{ position: 'relative', display: 'flex', alignItems: 'center', minHeight: 32 }}>
      <span style={{ position: 'absolute', left: '50%', transform: 'translateX(-50%)' }}>
        {center}
      </span>
    </div>
  );
}

// ── 联邦注册局 tab:机构详情页 + 本省联邦管理员列表 ──

export function FederalRegistryView({ state }: RegistryViewProps) {
  const { auth } = useAuth();
  const {
    federalAdmins,
    federalAdminsLoading,
    selectedFederalAdmin,
    federalRegistryDetail,
    federalRegistryLoading,
  } = state;

  // 稳定引用:复用 FederalAdminsView 预加载的 detail,避免 GovDetailPage 重复触发 load。
  const loadFederalRegistry = useCallback(() => {
    if (federalRegistryDetail) return Promise.resolve(federalRegistryDetail);
    if (auth) return getFederalRegistry(auth);
    return Promise.reject(new Error('未登录'));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [federalRegistryDetail, auth?.access_token]);

  if (!auth) return null;

  if (!federalRegistryDetail) {
    return (
      <Card bordered={false} style={glassCardStyle} headStyle={glassCardHeadStyle}>
        <Typography.Text type="secondary">
          {federalRegistryLoading ? '加载中...' : '暂无联邦注册局数据'}
        </Typography.Text>
      </Card>
    );
  }

  return (
    <GovDetailPage
      auth={auth}
      sfidNumber={federalRegistryDetail.institution.sfid_number}
      canWrite={false}
      loadDetail={loadFederalRegistry}
      adminListSection={selectedFederalAdmin ? (
        <FederalAdminSubTab
          selectedFederalAdmin={selectedFederalAdmin}
          federalAdmins={federalAdmins}
          federalAdminsLoading={federalAdminsLoading}
          refreshFederalAdmins={state.refreshFederalAdmins}
          runSecuredAction={state.runSecuredAction}
        />
      ) : null}
    />
  );
}

// ── 市注册局 tab:城市网格(联邦) / 机构详情页 + 本市市管理员列表 ──

export function CityRegistryView({ state }: RegistryViewProps) {
  const { auth } = useAuth();
  const scope = useScope(auth);
  const {
    selectedFederalAdmin,
    selectedCity,
    setSelectedCity,
    cityAdmins,
    cityAdminsLoading,
    cityAdminListPage,
    setCityAdminListPage,
    cityAdminCities,
    cityAdminCitiesLoading,
    setAddCityAdminOpen,
    onUpdateCityAdmin,
    onDeleteCityAdmin,
    cityRegistrySfid,
  } = state;

  if (!auth) return null;

  const effectiveProvince = scope.lockedProvince;
  const effectiveCity = selectedCity ?? scope.lockedCity;
  const city_adminsForProvince = selectedFederalAdmin ? cityAdmins : [];
  // 市管理员只读;联邦管理员可增删改(后端按登录省域二次校验)。
  const canEditCityAdmins = scope.canWrite && auth.role === 'FEDERAL_ADMIN';

  let body: React.ReactNode;
  if (!effectiveCity) {
    // 联邦管理员未选市 → 城市网格(套 Card 显示省份标题)
    body = (
      <Card
        title={makeCenteredTitle(effectiveProvince ?? '市注册局')}
        bordered={false}
        style={glassCardStyle}
        headStyle={glassCardHeadStyle}
      >
        <CityGrid
          cities={cityAdminCities.filter((c) => c.code !== '000')}
          citiesLoading={cityAdminCitiesLoading || (!selectedFederalAdmin && cityAdminCities.length === 0)}
          cityAdmins={city_adminsForProvince}
          onSelectCity={setSelectedCity}
        />
      </Card>
    );
  } else if (cityRegistrySfid) {
    // 选中市(联邦) / 锁定市(市管理员) → 市注册局机构详情页 + 本市市管理员列表
    const canGoBack = !scope.skipCityList;
    body = (
      <GovDetailPage
        auth={auth}
        sfidNumber={cityRegistrySfid}
        canWrite={scope.canWrite}
        onBack={canGoBack ? () => { setSelectedCity(null); setCityAdminListPage(1); } : undefined}
        backLabel="返回"
        adminListSection={
          <CityCityAdminsView
            canEditCityAdmins={canEditCityAdmins}
            cityAdmins={city_adminsForProvince.filter((op) => op.city === effectiveCity)}
            cityAdminsLoading={cityAdminsLoading}
            cityAdminListPage={cityAdminListPage}
            setCityAdminListPage={setCityAdminListPage}
            setAddCityAdminOpen={setAddCityAdminOpen}
            onUpdateCityAdmin={onUpdateCityAdmin}
            onDeleteCityAdmin={onDeleteCityAdmin}
          />
        }
      />
    );
  } else {
    // effectiveCity 必有对应市注册局(CITY_TEMPLATES 必生成);sfid 解析失败会单独 toast 报错。
    body = (
      <Card bordered={false} style={glassCardStyle} headStyle={glassCardHeadStyle}>
        <Typography.Text type="secondary">加载中...</Typography.Text>
      </Card>
    );
  }

  return (
    <>
      {body}
      <AddCityAdminModal state={state} />
    </>
  );
}

// ── 市管理员列表的城市入口网格 ──

function CityGrid({ cities, citiesLoading, cityAdmins, onSelectCity }: {
  cities: SfidCityItem[];
  citiesLoading: boolean;
  cityAdmins: CityAdminRow[];
  onSelectCity: (city: string) => void;
}) {
  return citiesLoading ? (
    <Typography.Text type="secondary">加载中...</Typography.Text>
  ) : cities.length === 0 ? (
    <Typography.Text type="secondary">暂无城市数据</Typography.Text>
  ) : (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))', gap: 12 }}>
      {cities.map((city) => {
        const count = cityAdmins.filter((op) => op.city === city.name).length;
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
              color={count >= MAX_CITY_ADMINS_PER_CITY ? 'red' : 'teal'}
              style={{ marginTop: 6 }}
            >
              {count} / {MAX_CITY_ADMINS_PER_CITY}
            </Tag>
          </div>
        );
      })}
    </div>
  );
}

// ── 某市的市管理员列表(内嵌进市注册局机构详情页) ──

function CityCityAdminsView({ canEditCityAdmins, cityAdmins, cityAdminsLoading, cityAdminListPage, setCityAdminListPage, setAddCityAdminOpen, onUpdateCityAdmin, onDeleteCityAdmin }: {
  canEditCityAdmins: boolean;
  cityAdmins: CityAdminRow[];
  cityAdminsLoading: boolean;
  cityAdminListPage: number;
  setCityAdminListPage: (v: number) => void;
  setAddCityAdminOpen: (v: boolean) => void;
  onUpdateCityAdmin: (row: CityAdminRow) => void;
  onDeleteCityAdmin: (row: CityAdminRow) => void;
}) {
  const { auth } = useAuth();
  // 中文注释:本列表已经按当前市过滤,所以长度就是该市市管理员数量。
  const cityLimitReached = cityAdmins.length >= MAX_CITY_ADMINS_PER_CITY;
  return (
    <Card
      type="inner"
      title="市管理员列表"
      extra={
        <Space size="middle" align="center">
          <Typography.Text type="secondary" style={{ fontWeight: 400, fontSize: 13 }}>
            用户数：{cityAdmins.length} / {MAX_CITY_ADMINS_PER_CITY}
          </Typography.Text>
          {canEditCityAdmins ? (
            <Button
              type="primary"
              disabled={cityLimitReached}
              title={cityLimitReached ? `本市市管理员已满 ${MAX_CITY_ADMINS_PER_CITY} 人` : undefined}
              onClick={() => setAddCityAdminOpen(true)}
            >
              新增市管理员
            </Button>
          ) : null}
        </Space>
      }
    >
      <Table<CityAdminRow>
        rowKey={(r) => `${r.id}-${r.admin_pubkey}`}
        loading={cityAdminsLoading}
        dataSource={cityAdmins}
        pagination={{
          pageSize: 10, current: cityAdminListPage,
          onChange: (page) => setCityAdminListPage(page),
          showSizeChanger: false,
          showTotal: (total) => `共 ${total} 条`,
        }}
        columns={[
          { title: '序号', width: 70, align: 'center', render: (_v, _row, index) => (cityAdminListPage - 1) * 10 + index + 1 },
          { title: '姓名', dataIndex: 'admin_name', align: 'center', width: 160 },
          { title: '账户', dataIndex: 'admin_pubkey', align: 'center', render: (v: string) => tryEncodeSs58(v) },
          {
            title: '操作', width: 260, align: 'center' as const,
            render: (_v: unknown, row: CityAdminRow) => (
              <Space>
                {canEditCityAdmins ? <Button size="small" onClick={() => onUpdateCityAdmin(row)}>编辑</Button> : null}
                {canEditCityAdmins ? <Button size="small" danger onClick={() => onDeleteCityAdmin(row)}>删除</Button> : null}
                <Passkey
                  size="small"
                  disabled={!sameHexPubkey(row.admin_pubkey, auth?.admin_pubkey)}
                />
              </Space>
            ),
          },
        ]}
      />
    </Card>
  );
}
