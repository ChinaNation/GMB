// 中文注释:通用机构 tab 视图。公安局/公权机构/私权机构三个 tab 共享同一套 UI,
// 仅通过 `category` 参数切换数据过滤 + 锁字段规则。
//
// 视觉完全复用"注册局"的 Card 样式(glassCardStyle / glassCardHeadStyle 直接从 App.tsx 导入),
// 保证横线颜色、毛玻璃底、绿色左竖条与注册局完全一致。
//
// Card title 布局:
//   - 左侧绝对定位返回按钮(可选)
//   - 中间绝对居中标题
//   - 右侧由 Card.extra 承载(机构表格页的"+ 新增")

import React, { useState } from 'react';
import { Button, Card, Input } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import { ProvinceGrid } from '../common/ProvinceGrid';
import { CityGrid } from '../common/CityGrid';
import { InstitutionListTable } from './InstitutionListTable';
import { CreateInstitutionModal } from './CreateInstitutionModal';
import { InstitutionDetailPage } from './InstitutionDetailPage';
import { useScope } from '../../hooks/useScope';
import type { AdminAuth, SfidMetaResult } from '../../api/client';
import type { InstitutionCategory } from '../../api/institution';
import { glassCardStyle, glassCardHeadStyle } from '../../components/App';

interface Props {
  auth: AdminAuth;
  category: InstitutionCategory;
  sfidMeta: SfidMetaResult | null;
}

/** 可复用的 Card title 布局:左(可选返回) + 中间绝对居中标题 */
function makeCenteredTitle(center: React.ReactNode, back?: () => void, backLabel?: string) {
  return (
    <div style={{ position: 'relative', display: 'flex', alignItems: 'center', minHeight: 32 }}>
      {back && (
        <Button type="link" style={{ paddingLeft: 0 }} onClick={back}>
          ← {backLabel ?? '返回'}
        </Button>
      )}
      <span style={{ position: 'absolute', left: '50%', transform: 'translateX(-50%)' }}>
        {center}
      </span>
    </div>
  );
}

export const InstitutionsView: React.FC<Props> = ({ auth, category, sfidMeta }) => {
  const scope = useScope(auth);
  const [selectedProvince, setSelectedProvince] = useState<string | null>(null);
  const [selectedCity, setSelectedCity] = useState<string | null>(null);
  const [selectedSfidId, setSelectedSfidId] = useState<string | null>(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [refreshKey, setRefreshKey] = useState(0);
  // 市详情页的机构列表搜索:输入不自动触发,点搜索图标提交 → committedSearch
  const [searchInput, setSearchInput] = useState('');
  const [committedSearch, setCommittedSearch] = useState('');

  const provinces = sfidMeta?.provinces || [];
  const lockedProvince = scope.lockedProvince;
  const lockedCity = scope.lockedCity;

  // 中文注释:公安局机构的数据由后端启动钩子 backfill_and_reconcile_public_security
  // 按 sfid 工具权威市清单一次性对齐好了,运行时静态不变。前端直接 list,不再每次进省
  // 页面 reconcile(那会触发全量 Store 持久化,500ms~2s 抖动)。
  // 当 sfid 工具的 city_codes 更新时发布新版 → 重启后端自动重跑启动对账。
  const isPublicSecurity = category === 'PUBLIC_SECURITY';

  // 机构详情页:InstitutionsView 本身不再渲染 Card,由 DetailPage 自己管
  if (selectedSfidId) {
    return (
      <InstitutionDetailPage
        auth={auth}
        sfidId={selectedSfidId}
        canWrite={scope.canWrite}
        onBack={() => {
          setSelectedSfidId(null);
          setRefreshKey((k) => k + 1);
        }}
      />
    );
  }

  const effectiveProvince = selectedProvince ?? lockedProvince;
  const effectiveCity = selectedCity ?? (scope.skipCityList ? lockedCity : null);

  // 按钮统一显示"新增",功能不变(弹 CreateInstitutionModal)
  const createLabel = '新增';

  const onSubmitSearch = () => {
    setCommittedSearch(searchInput.trim());
  };

  let title: React.ReactNode;
  let extra: React.ReactNode;
  let body: React.ReactNode;

  if (!effectiveProvince) {
    // ── KeyAdmin 首屏:省份列表 ──
    title = '省份列表';
    body = <ProvinceGrid provinces={provinces} onPick={(p) => setSelectedProvince(p)} />;
  } else if (isPublicSecurity) {
    // ── 任务卡 6:公安局省详情 = 该省所有公安局表格(跳过市卡片层,无"新增"按钮)──
    // 机构数据由后端 reconcile 按 sfid 工具市清单自动维护,前端不能新建。
    const canGoBack = !scope.skipProvinceList;
    title = makeCenteredTitle(
      effectiveProvince,
      canGoBack ? () => setSelectedProvince(null) : undefined,
      '返回省列表'
    );
    body = (
      <InstitutionListTable
        auth={auth}
        category={category}
        province={effectiveProvince}
        city=""
        refreshKey={refreshKey}
        onSelectInstitution={(sfidId) => setSelectedSfidId(sfidId)}
      />
    );
  } else if (!effectiveCity) {
    // ── 公权/私权的省详情 = 该省市列表 ──
    const canGoBack = !scope.skipProvinceList;
    title = makeCenteredTitle(
      effectiveProvince,
      canGoBack ? () => setSelectedProvince(null) : undefined,
      '返回省列表'
    );
    body = <CityGrid auth={auth} province={effectiveProvince} onPick={(c) => setSelectedCity(c)} />;
  } else {
    // ── 公权/私权的市详情 = 该市机构表格 ──
    // 不使用 Card 的 extra(否则 extra 占用 title 空间,中间标题会被挤到左侧)。
    // 整个 Card head 用三列 flex:左返回 + 中标题 + 右(搜索+新增),
    // 左右两列 flex:1 等宽,保证中间标题相对整个 Card 真正居中;
    // 搜索框 flex:1,自适应右列剩余宽度。
    const canGoBack = !scope.skipCityList;
    title = (
      <div style={{ display: 'flex', alignItems: 'center', width: '100%', gap: 16, minHeight: 32 }}>
        <div style={{ flex: '1 1 0', display: 'flex', alignItems: 'center' }}>
          {canGoBack && (
            <Button type="link" style={{ paddingLeft: 0 }} onClick={() => setSelectedCity(null)}>
              ← 返回
            </Button>
          )}
        </div>
        <div style={{ flex: '0 0 auto', fontWeight: 500 }}>
          {effectiveProvince} · {effectiveCity}
        </div>
        <div style={{ flex: '1 1 0', display: 'flex', alignItems: 'center', justifyContent: 'flex-end', gap: 8, minWidth: 0 }}>
          <Input
            value={searchInput}
            placeholder="请输入机构名称、机构SFID"
            allowClear
            style={{ flex: '1 1 auto', maxWidth: 360, minWidth: 0 }}
            onChange={(e) => {
              const v = e.target.value;
              setSearchInput(v);
              if (!v) setCommittedSearch('');
            }}
            onPressEnter={onSubmitSearch}
            suffix={
              <span
                style={{ cursor: 'pointer', color: '#1890ff' }}
                onClick={onSubmitSearch}
                title="搜索"
              >
                <SearchOutlined />
              </span>
            }
          />
          {scope.canWrite && (
            <Button type="primary" onClick={() => setCreateOpen(true)}>
              + {createLabel}
            </Button>
          )}
        </div>
      </div>
    );
    extra = null;
    body = (
      <InstitutionListTable
        auth={auth}
        category={category}
        province={effectiveProvince}
        city={effectiveCity}
        refreshKey={refreshKey}
        searchQuery={committedSearch}
        onSelectInstitution={(sfidId) => setSelectedSfidId(sfidId)}
      />
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

      <CreateInstitutionModal
        auth={auth}
        category={category}
        open={createOpen}
        lockedProvince={effectiveProvince}
        lockedCity={effectiveCity}
        onCancel={() => setCreateOpen(false)}
        onCreated={(result) => {
          setCreateOpen(false);
          setRefreshKey((k) => k + 1);
          // 两步式:私权机构生成 SFID 后直接跳详情页,让管理员补填名称/企业类型等信息
          if (category === 'PRIVATE_INSTITUTION' && result?.sfid_id) {
            setSelectedSfidId(result.sfid_id);
          }
        }}
      />
    </>
  );
};
