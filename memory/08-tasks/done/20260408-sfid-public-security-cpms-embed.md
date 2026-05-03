# 任务卡:公安局 CPMS 安装流程接入市公安局详情页

- **任务 ID**: `20260408-sfid-public-security-cpms-embed`
- **模块**: sfid-backend + sfid-frontend
- **状态**: 已完成
- **完成日期**: 2026-04-08

## 目标

把"给公安局安装 CPMS 系统"的整套 QR1/QR2/QR3 流程,从 sfid 前端老入口
(`components/App.tsx` 密钥管理 tab)整体搬迁到**每个市公安局的机构详情页**,
详情页拆上下两部分:上 = CPMS 站点管理,下 = 现有账户列表。老位置 UI 全部删除,
不保留全局总览视图。

## 后端改动

### 新增 HTTP 路由
- `GET /api/v1/admin/cpms-keys/by-institution/:sfid_id`
  - Handler: `sheng_admins::get_cpms_site_by_institution`
  - 输入:`multisig_institutions.sfid_id`(机构主键,**不是 CPMS site_sfid**)
  - 逻辑:读机构拿 `(province, city, institution_code)`,线性扫描
    `cpms_site_keys.values()` 找匹配三元组的首条记录,返回 `CpmsSiteKeysListRow` 或 `null`

### 新增启动钩子
- `app_core::runtime_ops::cleanup_orphan_cpms_sites(state)`
  - 构建全部机构的 `(province, city, institution_code)` 合法集合
  - 扫 `cpms_site_keys.values()` 硬删所有元组不在集合中的孤儿站点
  - `main.rs` 启动流程追加调用,排在
    `backfill_and_reconcile_public_security` 之后,确保 reconcile
    删除的市公安局同时带走其 CPMS 站点

### 关键约束
- **两个 sfid_id 不同值**:`multisig_institutions.sfid_id` 是两层机构模型主键,
  `cpms_site_keys.site_sfid` 是生成 QR1 时随机派生,**两者不等价**,只能用
  `(province, city, institution_code)` 元组关联。公安局每市唯一保证一一对应。
- 见 `feedback_cpms_institution_tuple_match.md`

### 改动文件
- `sfid/backend/src/sheng-admins/institutions.rs`(新增 handler)
- `sfid/backend/src/sheng-admins/mod.rs`(re-export)
- `sfid/backend/src/main.rs`(路由 + 启动钩子调用)
- `sfid/backend/src/app_core/runtime_ops.rs`(新增 `cleanup_orphan_cpms_sites`)

## 前端改动

### 新增文件
- `sfid/frontend/src/views/institutions/CpmsSitePanel.tsx`(~300 行)
  - 公安局详情页上半部分主组件
  - 无站点态:显示"生成 CPMS 安装二维码"按钮
  - 有站点态:Descriptions 展示 site_sfid、安装令牌状态(Tag)、站点状态(Tag)、
    版本、时间戳;QR1 payload 折叠展开 + 复制
  - 操作按钮(`canWrite` 门禁):扫描 QR2 注册、重发安装令牌、禁用、吊销、删除
  - 复用 `api/client.ts` 里已有的 `generateCpmsInstitutionSfid` /
    `reissueInstallToken` / `disableCpmsKeys` / `revokeCpmsKeys` / `deleteCpmsKeys`
  - 调新增 API `getCpmsSiteByInstitution`

- `sfid/frontend/src/views/institutions/CpmsRegisterModal.tsx`(~105 行)
  - QR2 payload 粘贴输入 → `registerCpms` → 展示 QR3 payload + 复制
  - 本次不做摄像头扫码(老 App.tsx 里的扫码实现后续拆分任务再搬)

### 改动文件
- `sfid/frontend/src/api/institution.ts`
  - 新增 `getCpmsSiteByInstitution(auth, sfidId) → Promise<CpmsSiteRow | null>`
  - 从 `client.ts` re-export `CpmsSiteRow` 类型

- `sfid/frontend/src/views/institutions/InstitutionDetailPage.tsx`
  - 仅 `inst.category === 'PUBLIC_SECURITY'` 时在账户列表 Card **之上**
    插入 `<CpmsSitePanel>`
  - 其他机构类别完全不渲染 Panel
  - 下半部分(账户列表 + 新建账户)保持不变

### 删除范围 — `components/App.tsx`(**3769 行 → 3431 行**,删除 338 行)
- imports:移除 `CpmsSiteRow`, `GenerateCpmsInstitutionSfidResult`,
  `deleteCpmsKeys`, `disableCpmsKeys`, `generateCpmsInstitutionSfid`,
  `listCpmsSites`, `registerCpms`(**保留 `scanCpmsStatusQr`**,QR4 citizen
  档案扫描仍在用)
- state:`cpmsSites` / `cpmsSitesLoading` / `institutionSfidOpen` /
  `institutionSfidLoading` / `institutionSfidResult` / `institutionQrPreview`
- forms/refs:`institutionSfidForm`, `institutionQrRef`, `institutionQrPreviewRef`
- handler:`refreshCpmsSites`, `onDisableCpmsSite`, `onDeleteCpmsSite`,
  `openRegisterScanner`, `openInstitutionSfidModal`,
  `onGenerateInstitutionSfid`, `onFinishInstitutionSfid`,
  `downloadQrFromRef`, `onDownloadInstitutionSfid`, `onDownloadInstitutionPreview`
- JSX:"新增公安局" Modal(QR1 生成)、"身份识别码二维码" 预览 Modal
- 其他:`const institutionRows = cpmsSites;` 死代码、`setCpmsSites([])` 登出清理、
  `onHandleOperationQr` 的 `register` 分支简化为只处理 `status`(QR4)分支
- **保留**:`scanCpmsStatusQr`(QR4 citizen 档案扫描)、`importArchive`(档案导入)、
  业务员绑定流程、登录/密钥轮换/操作员/Dashboard 全部其他功能

## 验收命令

```bash
# 后端
cd sfid/backend && cargo check
# → 37 条既有 warnings(不相关),无新增错误

# 前端
cd sfid/frontend && npx tsc --noEmit   # EXIT=0
npm run build                           # 成功, 1.69s
```

## 孤儿清理语义

**开发期铁律**(`feedback_chain_in_dev.md`):发现 `cpms_site_keys` 中存在
找不到对应 `multisig_institutions` 的记录,直接硬删。启动日志示例:

```
INFO sfid_backend::app_core::runtime_ops: cleaned up orphan CPMS sites (no matching institution)
  count=3 sample=["SFID-XXX", "SFID-YYY", "SFID-ZZZ"]
```

## 不做的事

- 不动 citizenchain 链码
- 不动 `operate/cpms_qr.rs`(QR 协议签名/验签核心)
- 不动数据库 schema / 任何现有迁移文件
- 不动 QR4 citizen 档案绑定流程(跟本任务无关,归属业务员模块)
- 不动摄像头扫码组件(后续 App.tsx 拆分任务搬)

## 相关文档

- 前置讨论:省级管理员一主两备方案 A(修改 citizenchain runtime 支持每省 signer)
- 后续任务:`20260408-sfid-frontend-app-tsx-split` — App.tsx 彻底拆分
  (3431 行 → ≤300 行,按 auth/dashboard/registration/binding/operators/key-management
  分六个子目录)
