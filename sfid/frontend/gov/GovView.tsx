// 中文注释:公权机构页面入口。公安局和普通公权机构都归 gov 目录维护。
//
// 视觉完全复用"注册局"的 Card 样式(glassCardStyle / glassCardHeadStyle),
// 保证横线颜色、毛玻璃底、绿色左竖条与注册局完全一致。
//
// Card title 布局:
//   - 左侧绝对定位返回按钮(可选)
//   - 中间绝对居中标题
//   - 右侧由 Card.extra 承载(机构表格页的"+ 新增")

import React, { useEffect, useState } from 'react';
import { Button, Card, Input } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import { ProvinceGrid } from '../core/ProvinceGrid';
import { CityGrid } from '../core/CityGrid';
import { GovListTable } from './GovListTable';
import { GovDetailPage } from './GovDetailPage';
import { useScope } from '../hooks/useScope';
import type { AdminAuth } from '../auth/types';
import type { SfidMetaResult } from '../china/api';
import type { GovCategory } from './api';
import { glassCardStyle, glassCardHeadStyle } from '../core/cardStyles';

interface Props {
  auth: AdminAuth;
  category: GovCategory;
  sfidMeta: SfidMetaResult | null;
  resetToken?: number;
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

export const GovView: React.FC<Props> = ({ auth, category, sfidMeta, resetToken = 0 }) => {
  const scope = useScope(auth);
  const [selectedProvince, setSelectedProvince] = useState<string | null>(null);
  const [selectedCity, setSelectedCity] = useState<string | null>(null);
  const [selectedSfidNumber, setSelectedSfidNumber] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  // 市详情页的机构列表搜索:输入不自动触发,点搜索图标提交 → committedSearch
  const [searchInput, setSearchInput] = useState('');
  const [committedSearch, setCommittedSearch] = useState('');

  useEffect(() => {
    // 中文注释:顶层 tab 切换必须打断详情页状态,否则公权机构和公安局复用 GovView 时会停留在旧详情。
    setSelectedProvince(null);
    setSelectedCity(null);
    setSelectedSfidNumber(null);
    setSearchInput('');
    setCommittedSearch('');
  }, [auth.admin_pubkey, category, resetToken]);

  const provinces = sfidMeta?.provinces || [];
  const lockedProvince = scope.lockedProvince;
  const lockedCity = scope.lockedCity;

  // 中文注释:公安局是确定性机构,不走普通公权机构搜索。
  // 前端进入 tab 后由 GovListTable 先读本地缓存,没有缓存才请求后端专用接口。
  const isPublicSecurity = category === 'PUBLIC_SECURITY';

  // 机构详情页由公权详情页组件接管,列表页自身只维护返回状态。
  if (selectedSfidNumber) {
    return (
      <GovDetailPage
        auth={auth}
        sfidNumber={selectedSfidNumber}
        canWrite={scope.canWrite}
        onBack={() => {
          setSelectedSfidNumber(null);
          setRefreshKey((k) => k + 1);
        }}
      />
    );
  }

  const effectiveProvince = selectedProvince ?? lockedProvince;
  const effectiveCity = selectedCity ?? (scope.skipCityList ? lockedCity : null);

  // 中文注释:公权机构全部由后端自动生成,本页无手动新增入口;
  // 教育委员会(JY)学校机构的新增/管理统一在教育机构 tab。
  const onSubmitSearch = () => {
    setCommittedSearch(searchInput.trim());
  };

  let title: React.ReactNode;
  let extra: React.ReactNode;
  let body: React.ReactNode;

  if (!effectiveProvince) {
    // ── 首屏:省份列表(SHENG/SHI 角色 effectiveProvince 一定有值,此分支留作未来扩展) ──
    title = '省份列表';
    body = <ProvinceGrid provinces={provinces} onPick={(p) => setSelectedProvince(p)} />;
  } else if (isPublicSecurity) {
    // ── 任务卡 6:公安局省详情 = 该省所有公安局表格(跳过市卡片层,无"新增"按钮)──
    // 机构数据由后端 reconcile 按 sfid 工具市清单自动维护,前端不能新建。
    const canGoBack = !scope.skipProvinceList;
    const titleText = scope.lockedCity
      ? `${effectiveProvince} · ${scope.lockedCity}`
      : effectiveProvince;
    title = makeCenteredTitle(
      titleText,
      canGoBack ? () => setSelectedProvince(null) : undefined,
      '返回省列表'
    );
    body = (
      <GovListTable
        auth={auth}
        category={category}
        province={effectiveProvince}
        city=""
        refreshKey={refreshKey}
        onSelectInstitution={(sfidNumber) => setSelectedSfidNumber(sfidNumber)}
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
            placeholder="请输入机构名称、身份ID"
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
        </div>
      </div>
    );
    extra = null;
    body = (
      <GovListTable
        auth={auth}
        category={category}
        province={effectiveProvince}
        city={effectiveCity}
        refreshKey={refreshKey}
        searchQuery={committedSearch}
        onSelectInstitution={(sfidNumber) => setSelectedSfidNumber(sfidNumber)}
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
    </>
  );
};
