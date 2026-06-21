// 注册局机构详情视图(两个顶级 tab 各一个):
//   - CityRegistryView(市注册局 tab):联邦注册局管理员 城市表格→点市→该市注册局机构详情页;
//                                      市注册局管理员 直接进本市注册局机构详情页。
//   - FederalRegistryView(联邦注册局 tab):全国唯一联邦注册局机构详情页。
// 两者 leaf 都是「机构详情页 + 管理员列表 tab」(GovDetailPage.adminListSection),
// 管理员列表由详情页左侧导航显示。数据由 RegistryAdminsView 统一加载。

import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { Button, Card, Space, Table, Tag, Typography } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { useAuth } from '../hooks/useAuth';
import { useScope } from '../hooks/useScope';
import type { AdminAuth } from '../auth/types';
import type { CidCityItem } from '../china/api';
import type { CityRegistryAdminRow } from './city_registry_admins_api';
import { tryEncodeSs58 } from '../utils/ss58';
import { glassCardStyle, glassCardHeadStyle } from '../core/cardStyles';
import { MAX_CITY_REGISTRY_ADMINS_PER_CITY, sameHexAccount } from './adminUtils';
import type { RegistryAdminsSharedState } from './adminUtils';
import { AddCityRegistryAdminModal } from './AddCityRegistryAdminModal';
import { FederalRegistryAdminSubTab } from './FederalRegistryAdminSubTab';
import { Passkey } from './Passkey';
import { GovDetailPage } from '../gov/GovDetailPage';
import { getFederalRegistry, listOfficialInstitutions } from '../gov/api';
import type { InstitutionListRow } from '../subjects/api';

interface RegistryViewProps {
  state: RegistryAdminsSharedState;
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

// ── 联邦注册局 tab:机构详情页 + 本省联邦注册局管理员列表 ──

export function FederalRegistryView({ state }: RegistryViewProps) {
  const { auth } = useAuth();
  const scope = useScope(auth);
  const {
    federalRegistryAdmins,
    federalRegistryAdminsLoading,
    selectedFederalRegistry,
    federalRegistryDetail,
    federalRegistryLoading,
  } = state;

  // 稳定引用:复用 RegistryAdminsView 预加载的 detail,避免 GovDetailPage 重复触发 load。
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
      cidNumber={federalRegistryDetail.institution.cid_number}
      canWrite={scope.canWrite && auth.registry_org_code === 'FEDERAL_REGISTRY'}
      loadDetail={loadFederalRegistry}
      adminListSection={selectedFederalRegistry ? (
        <FederalRegistryAdminSubTab
          selectedFederalRegistry={selectedFederalRegistry}
          federalRegistryAdmins={federalRegistryAdmins}
          federalRegistryAdminsLoading={federalRegistryAdminsLoading}
          refreshFederalRegistryAdmins={state.refreshFederalRegistryAdmins}
          runSecuredAction={state.runSecuredAction}
        />
      ) : null}
    />
  );
}

// ── 市注册局 tab:城市表格(联邦) / 机构详情页 + 本市市注册局管理员列表 ──

export function CityRegistryView({ state }: RegistryViewProps) {
  const { auth } = useAuth();
  const scope = useScope(auth);
  const {
    selectedFederalRegistry,
    selectedCity,
    setSelectedCity,
    cityRegistryAdmins,
    cityRegistryAdminsLoading,
    cityRegistryAdminListPage,
    setCityRegistryListPage,
    cityRegistryAdminCities,
    cityRegistryAdminCitiesLoading,
    setAddCityRegistryOpen,
    onUpdateCityRegistry,
    onDeleteCityRegistry,
    cityRegistryCid,
  } = state;

  if (!auth) return null;

  const effectiveProvince = scope.lockedProvinceName;
  const effectiveCity = selectedCity ?? scope.lockedCityName;
  const cityRegistryAdminsForProvince = selectedFederalRegistry ? cityRegistryAdmins : [];
  // 市注册局管理员只读;联邦注册局管理员可增删改(后端按登录省域二次校验)。
  const canEditCityRegistryAdmins = scope.canWrite && auth.registry_org_code === 'FEDERAL_REGISTRY';

  let body: React.ReactNode;
  if (!effectiveCity) {
    // 联邦注册局管理员未选市 → 城市表格(套 Card 显示省份标题)
    body = (
      <Card
        title={makeCenteredTitle(effectiveProvince ?? '市注册局')}
        bordered={false}
        style={glassCardStyle}
        headStyle={glassCardHeadStyle}
      >
        <CityRegistryListTable
          auth={auth}
          province_name={effectiveProvince ?? ''}
          cities={cityRegistryAdminCities.filter((c) => c.code !== '000')}
          citiesLoading={cityRegistryAdminCitiesLoading || (!selectedFederalRegistry && cityRegistryAdminCities.length === 0)}
          cityRegistryAdmins={cityRegistryAdminsForProvince}
          cityRegistryAdminsLoading={cityRegistryAdminsLoading}
          onSelectCity={setSelectedCity}
        />
      </Card>
    );
  } else if (cityRegistryCid) {
    // 选中市(联邦) / 锁定市(市注册局管理员) → 市注册局机构详情页 + 本市市注册局管理员列表
    const canGoBack = !scope.skipCityList;
    body = (
      <GovDetailPage
        auth={auth}
        cidNumber={cityRegistryCid}
        canWrite={scope.canWrite}
        onBack={canGoBack ? () => { setSelectedCity(null); setCityRegistryListPage(1); } : undefined}
        backLabel="返回列表"
        adminListSection={
          <CityRegistryAdminsView
            canEditCityRegistryAdmins={canEditCityRegistryAdmins}
            cityRegistryAdmins={cityRegistryAdminsForProvince.filter((op) => op.city_name === effectiveCity)}
            cityRegistryAdminsLoading={cityRegistryAdminsLoading}
            cityRegistryAdminListPage={cityRegistryAdminListPage}
            setCityRegistryListPage={setCityRegistryListPage}
            setAddCityRegistryOpen={setAddCityRegistryOpen}
            onUpdateCityRegistry={onUpdateCityRegistry}
            onDeleteCityRegistry={onDeleteCityRegistry}
          />
        }
      />
    );
  } else {
    // effectiveCity 必有对应市注册局(CITY_TEMPLATES 必生成);cid 解析失败会单独 toast 报错。
    body = (
      <Card bordered={false} style={glassCardStyle} headStyle={glassCardHeadStyle}>
        <Typography.Text type="secondary">加载中...</Typography.Text>
      </Card>
    );
  }

  return (
    <>
      {body}
      <AddCityRegistryAdminModal state={state} />
    </>
  );
}

// ── 市注册局城市入口表格 ──

const CITY_REGISTRY_PAGE_SIZE = 20;

function areaText(row: InstitutionListRow | null, province_name: string, city_name: string) {
  if (!row) return [province_name, city_name].filter(Boolean).join('/') || '-';
  return [row.province_name, row.city_name, row.town_name].filter(Boolean).join('/') || '-';
}

function nameText(row: InstitutionListRow | null, city_name: string) {
  return row?.cid_full_name || row?.cid_short_name || `${city_name}注册局`;
}

function CityRegistryListTable({ auth, province_name, cities, citiesLoading, cityRegistryAdmins, cityRegistryAdminsLoading, onSelectCity }: {
  auth: AdminAuth;
  province_name: string;
  cities: CidCityItem[];
  citiesLoading: boolean;
  cityRegistryAdmins: CityRegistryAdminRow[];
  cityRegistryAdminsLoading: boolean;
  onSelectCity: (city_name: string) => void;
}) {
  const [registryRows, setRegistryRows] = useState<InstitutionListRow[]>([]);
  const [registryLoading, setRegistryLoading] = useState(false);
  const [page, setPage] = useState(1);

  useEffect(() => {
    if (!province_name) {
      setRegistryRows([]);
      setRegistryLoading(false);
      return;
    }
    let cancelled = false;
    setRegistryLoading(true);
    listOfficialInstitutions(auth, { province_name, org_code: 'CITY_REGISTRY', page_size: 300 })
      .then((res) => {
        if (!cancelled) {
          setRegistryRows(res.items);
        }
      })
      .catch(() => {
        if (!cancelled) setRegistryRows([]);
      })
      .finally(() => {
        if (!cancelled) setRegistryLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [auth.access_token, province_name]);

  useEffect(() => {
    setPage(1);
  }, [province_name, cities.length]);

  const rows = useMemo(() => {
    return cities.map((city_item) => {
      const registry = registryRows.find((row) => row.city_name === city_item.name) ?? null;
      const adminCount = cityRegistryAdmins.filter((admin) => admin.city_name === city_item.name).length;
      return { city_item, registry, adminCount };
    });
  }, [cities, registryRows, cityRegistryAdmins]);

  const totalPages = Math.max(1, Math.ceil(rows.length / CITY_REGISTRY_PAGE_SIZE));
  const displayRows = rows.slice(
    (page - 1) * CITY_REGISTRY_PAGE_SIZE,
    page * CITY_REGISTRY_PAGE_SIZE,
  );

  const columns = useMemo<ColumnsType<(typeof rows)[number]>>(() => [
    {
      title: '序号',
      width: 70,
      align: 'center',
      render: (_v, _row, index) => (page - 1) * CITY_REGISTRY_PAGE_SIZE + index + 1,
    },
    {
      title: '身份ID',
      width: 260,
      align: 'center',
      render: (_v, row) => row.registry?.cid_number ?? '-',
    },
    {
      title: '市注册局名称',
      width: 180,
      align: 'center',
      render: (_v, row) => nameText(row.registry, row.city_item.name),
    },
    {
      title: '所属行政区',
      width: 180,
      align: 'center',
      render: (_v, row) => areaText(row.registry, province_name, row.city_item.name),
    },
    {
      title: '管理员数',
      width: 150,
      align: 'center',
      render: (_v, row) => (
        <Tag color={row.adminCount >= MAX_CITY_REGISTRY_ADMINS_PER_CITY ? 'red' : 'teal'}>
          {row.adminCount} / {MAX_CITY_REGISTRY_ADMINS_PER_CITY}
        </Tag>
      ),
    },
  ], [page, province_name]);

  if (!citiesLoading && cities.length === 0) {
    return <Typography.Text type="secondary">暂无城市数据</Typography.Text>;
  }

  return (
    <div>
      <Table<(typeof rows)[number]>
        rowKey={(row) => row.city_item.code}
        loading={citiesLoading || registryLoading || cityRegistryAdminsLoading}
        dataSource={displayRows}
        pagination={false}
        onRow={(row) => ({
          onClick: () => row.registry && onSelectCity(row.city_item.name),
          style: row.registry ? { cursor: 'pointer' } : { color: '#94a3b8' },
        })}
        columns={columns}
      />
      <Space style={{ marginTop: 12 }} wrap>
        <Typography.Text type="secondary">
          共 {totalPages} 页 / 第 {page} 页
        </Typography.Text>
        <Typography.Text type="secondary">共 {rows.length} 条</Typography.Text>
        <Button disabled={citiesLoading || registryLoading || page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))}>
          上一页
        </Button>
        <Button disabled={citiesLoading || registryLoading || page >= totalPages} onClick={() => setPage((p) => Math.min(totalPages, p + 1))}>
          下一页
        </Button>
      </Space>
    </div>
  );
}

// ── 某市的市注册局管理员列表(显示在市注册局机构详情页的“管理员列表”tab) ──

function CityRegistryAdminsView({ canEditCityRegistryAdmins, cityRegistryAdmins, cityRegistryAdminsLoading, cityRegistryAdminListPage, setCityRegistryListPage, setAddCityRegistryOpen, onUpdateCityRegistry, onDeleteCityRegistry }: {
  canEditCityRegistryAdmins: boolean;
  cityRegistryAdmins: CityRegistryAdminRow[];
  cityRegistryAdminsLoading: boolean;
  cityRegistryAdminListPage: number;
  setCityRegistryListPage: (v: number) => void;
  setAddCityRegistryOpen: (v: boolean) => void;
  onUpdateCityRegistry: (row: CityRegistryAdminRow) => void;
  onDeleteCityRegistry: (row: CityRegistryAdminRow) => void;
}) {
  const { auth } = useAuth();
  // 中文注释:本列表已经按当前市过滤,所以长度就是该市市注册局管理员数量。
  const cityLimitReached = cityRegistryAdmins.length >= MAX_CITY_REGISTRY_ADMINS_PER_CITY;
  return (
    <Card
      type="inner"
      title="市注册局管理员列表"
      extra={
        <Space size="middle" align="center">
          <Typography.Text type="secondary" style={{ fontWeight: 400, fontSize: 13 }}>
            用户数：{cityRegistryAdmins.length} / {MAX_CITY_REGISTRY_ADMINS_PER_CITY}
          </Typography.Text>
          {canEditCityRegistryAdmins ? (
            <Button
              type="primary"
              disabled={cityLimitReached}
              title={cityLimitReached ? `本市市注册局管理员已满 ${MAX_CITY_REGISTRY_ADMINS_PER_CITY} 人` : undefined}
              onClick={() => setAddCityRegistryOpen(true)}
            >
              新增市注册局管理员
            </Button>
          ) : null}
        </Space>
      }
    >
      <Table<CityRegistryAdminRow>
        rowKey={(r) => `${r.id}-${r.admin_account}`}
        loading={cityRegistryAdminsLoading}
        dataSource={cityRegistryAdmins}
        pagination={{
          pageSize: 10, current: cityRegistryAdminListPage,
          onChange: (page) => setCityRegistryListPage(page),
          showSizeChanger: false,
          showTotal: (total) => `共 ${total} 条`,
        }}
        columns={[
          { title: '序号', width: 70, align: 'center', render: (_v, _row, index) => (cityRegistryAdminListPage - 1) * 10 + index + 1 },
          { title: '姓名', dataIndex: 'admin_display_name', align: 'center', width: 160 },
          { title: '账户', dataIndex: 'admin_account', align: 'center', render: (v: string) => tryEncodeSs58(v) },
          {
            title: '操作', width: 260, align: 'center' as const,
            render: (_v: unknown, row: CityRegistryAdminRow) => (
              <Space>
                {canEditCityRegistryAdmins ? <Button size="small" onClick={() => onUpdateCityRegistry(row)}>编辑</Button> : null}
                {canEditCityRegistryAdmins ? <Button size="small" danger onClick={() => onDeleteCityRegistry(row)}>删除</Button> : null}
                <Passkey
                  size="small"
                  disabled={!sameHexAccount(row.admin_account, auth?.admin_account)}
                />
              </Space>
            ),
          },
        ]}
      />
    </Card>
  );
}
