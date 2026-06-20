// 注册局机构详情视图(两个顶级 tab 各一个):
//   - CityRegistryView(市注册局 tab):联邦管理员 城市表格→点市→该市注册局机构详情页;
//                                      市管理员 直接进本市注册局机构详情页。
//   - FederalRegistryView(联邦注册局 tab):全国唯一联邦注册局机构详情页。
// 两者 leaf 都是「机构详情页 + 管理员列表 tab」(GovDetailPage.adminListSection),
// 管理员列表由详情页左侧导航显示。数据由 FederalAdminsView 统一加载。

import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { Button, Card, Space, Table, Tag, Typography } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { useAuth } from '../hooks/useAuth';
import { useScope } from '../hooks/useScope';
import type { AdminAuth } from '../auth/types';
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
import { getFederalRegistry, listOfficialInstitutions } from '../gov/api';
import type { InstitutionListRow } from '../subjects/api';

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
  const scope = useScope(auth);
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
      canWrite={scope.canWrite && auth.role === 'FEDERAL_ADMIN'}
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

// ── 市注册局 tab:城市表格(联邦) / 机构详情页 + 本市市管理员列表 ──

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
    // 联邦管理员未选市 → 城市表格(套 Card 显示省份标题)
    body = (
      <Card
        title={makeCenteredTitle(effectiveProvince ?? '市注册局')}
        bordered={false}
        style={glassCardStyle}
        headStyle={glassCardHeadStyle}
      >
        <CityRegistryListTable
          auth={auth}
          province={effectiveProvince ?? ''}
          cities={cityAdminCities.filter((c) => c.code !== '000')}
          citiesLoading={cityAdminCitiesLoading || (!selectedFederalAdmin && cityAdminCities.length === 0)}
          cityAdmins={city_adminsForProvince}
          cityAdminsLoading={cityAdminsLoading}
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
        backLabel="返回列表"
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

// ── 市注册局城市入口表格 ──

const CITY_REGISTRY_PAGE_SIZE = 20;

function areaText(row: InstitutionListRow | null, province: string, city: string) {
  if (!row) return [province, city].filter(Boolean).join('/') || '-';
  return [row.province_name, row.city_name, row.town_name].filter(Boolean).join('/') || '-';
}

function nameText(row: InstitutionListRow | null, city: string) {
  return row?.sfid_full_name || row?.sfid_short_name || `${city}注册局`;
}

function CityRegistryListTable({ auth, province, cities, citiesLoading, cityAdmins, cityAdminsLoading, onSelectCity }: {
  auth: AdminAuth;
  province: string;
  cities: SfidCityItem[];
  citiesLoading: boolean;
  cityAdmins: CityAdminRow[];
  cityAdminsLoading: boolean;
  onSelectCity: (city: string) => void;
}) {
  const [registryRows, setRegistryRows] = useState<InstitutionListRow[]>([]);
  const [registryLoading, setRegistryLoading] = useState(false);
  const [page, setPage] = useState(1);

  useEffect(() => {
    if (!province) {
      setRegistryRows([]);
      setRegistryLoading(false);
      return;
    }
    let cancelled = false;
    setRegistryLoading(true);
    listOfficialInstitutions(auth, { province_name: province, org_code: 'CITY_REGISTRY', page_size: 300 })
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
  }, [auth.access_token, province]);

  useEffect(() => {
    setPage(1);
  }, [province, cities.length]);

  const rows = useMemo(() => {
    return cities.map((city) => {
      const registry = registryRows.find((row) => row.city_name === city.name) ?? null;
      const adminCount = cityAdmins.filter((admin) => admin.city === city.name).length;
      return { city, registry, adminCount };
    });
  }, [cities, registryRows, cityAdmins]);

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
      render: (_v, row) => row.registry?.sfid_number ?? '-',
    },
    {
      title: '市注册局名称',
      width: 180,
      align: 'center',
      render: (_v, row) => nameText(row.registry, row.city.name),
    },
    {
      title: '所属行政区',
      width: 180,
      align: 'center',
      render: (_v, row) => areaText(row.registry, province, row.city.name),
    },
    {
      title: '管理员数',
      width: 150,
      align: 'center',
      render: (_v, row) => (
        <Tag color={row.adminCount >= MAX_CITY_ADMINS_PER_CITY ? 'red' : 'teal'}>
          {row.adminCount} / {MAX_CITY_ADMINS_PER_CITY}
        </Tag>
      ),
    },
  ], [page, province]);

  if (!citiesLoading && cities.length === 0) {
    return <Typography.Text type="secondary">暂无城市数据</Typography.Text>;
  }

  return (
    <div>
      <Table<(typeof rows)[number]>
        rowKey={(row) => row.city.code}
        loading={citiesLoading || registryLoading || cityAdminsLoading}
        dataSource={displayRows}
        pagination={false}
        onRow={(row) => ({
          onClick: () => row.registry && onSelectCity(row.city.name),
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

// ── 某市的市管理员列表(显示在市注册局机构详情页的“管理员列表”tab) ──

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
