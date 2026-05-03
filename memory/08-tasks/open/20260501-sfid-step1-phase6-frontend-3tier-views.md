# SFID Step 1 / Phase 6:前端 3-tier 视图重构

- 状态:open
- 创建日期:2026-05-01
- 模块:`sfid/frontend`
- 上游:`memory/08-tasks/open/20260501-sfid-step1-sheng-admin-3tier.md`(主卡)
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier.md`
- 前置依赖:Phase 4+5 卡(后端 API endpoint 必须落地)

## 任务需求


## 建议模块

- `sfid/frontend/src/views/`:删 keyring + 重构 sheng-admins → sheng_admin + 新页面
- `sfid/frontend/src/api/`:新增 sheng_admin / sheng_signer 客户端

## 影响范围(文件级)

### 删除

| 路径 | 说明 |
|---|---|
| Header 角色切换 KEY 选项 | 改为只 SHENG / SHI 两选项 |

### 重命名

| 路径 | 原 |
|---|---|
| `views/sheng-admins/` → `views/sheng_admin/` | 对齐后端目录命名 |
| `views/operators/` → `views/shi_admin/` | 命名统一(原 operators 即市管理员) |

### 新增页面

| 路径 | 职责 |
|---|---|
| `views/sheng_admin/RosterPage.tsx` | 名册管理:展示 3 slot,main 可加/删 backup admin pubkey |
| `views/sheng_admin/ActivationPage.tsx` | 首登检测 `signing_pending_activation` flag → 引导激活(调 `POST /sheng-signer/activate`) |
| `views/sheng_admin/RotatePage.tsx` | rotate 签名密钥(调 `POST /sheng-signer/rotate`) |
| `views/institution/policies/PrivateForm.tsx` | 私权机构表单 |
| `views/institution/policies/GovForm.tsx` | 公权机构表单 |
| `views/institution/policies/PublicSecurityForm.tsx` | 公安局表单 |
| `views/institution/CreateForm.tsx` | 框架,根据 category 渲染上面三个 |

### 改造

| 路径 | 改造 |
|---|---|
| `views/sheng_admin/DashboardPage.tsx` | 加全局视图(43 省可看)+ 跨省按钮置灰 + 当前登录 slot 显示 |
| `views/shi_admin/DashboardPage.tsx` | 同上(全局视图,跨省置灰) |
| `App.tsx` | 路由表:删 keyring 路由 + 加 roster/activation/rotate 三路由;角色守卫只剩 SHENG/SHI |
| `components/Header.tsx` | 角色切换删 KEY 选项 |
| `api/sheng_admin.ts`(新建) | `getRoster`、`addBackup`、`removeBackup` |
| `api/sheng_signer.ts`(新建) | `activate`、`rotate` |
| `api/institution.ts` | 加 policy 类型字段 |

### types/

| 文件 | 内容 |
|---|---|
| `types/role.ts` | AdminRole 删 KEY |
| `types/slot.ts`(新建) | `ShengSlot = 'Main' \| 'Backup1' \| 'Backup2'` |
| `types/session.ts` | session 加 `unlockedAdminPubkey` / `unlockedSlot` |

## 主要风险点

- **session 字段变更影响所有受 session 守卫的页面**:必须同步更新所有 `useSession()` 消费方。
- **跨省按钮置灰逻辑分散**:每个写操作按钮都要 `disabled={session.province !== row.province}` + tooltip;考虑抽 `<ProvinceWriteGuard>` 组件统一。
- **机构创建表单 3 类策略**:三类字段集合差异大(尤其公安局编码 + 公权机构层级),先确认字段清单(已有 `institutions/policies/{private,gov,public_security}.rs` 后端策略类比对前端字段)。
- **浏览器扩展签名兼容**:登录挑战签名 + activation/rotate 签名都靠浏览器签名扩展,新接口的挑战格式必须兼容现有扩展。
- **删 keyring 后路由重定向**:已登录用户若旧 URL `/keyring` → 404 vs redirect to `/sheng-admin/dashboard`;开发期可硬 404,但 nice-to-have 是兜底重定向。

## 是否需要先沟通

- **是 1 项**:三类机构策略字段清单是否已经在后端 `institutions/policies/` 中定型?若未定型,前端表单字段无法确定。
- 其余按方案直接执行

## 建议下一步

1. 删 `views/keyring/` 整目录
2. 重命名 `views/sheng-admins/` → `views/sheng_admin/`,更新 import 路径
3. 重命名 `views/operators/` → `views/shi_admin/`
4. 新建 `views/sheng_admin/RosterPage.tsx`,对接 `GET /sheng-admin/roster` + `POST /sheng-admin/roster/add-backup`
5. 新建 `views/sheng_admin/ActivationPage.tsx`,对接 `POST /sheng-signer/activate`
6. 新建 `views/sheng_admin/RotatePage.tsx`,对接 `POST /sheng-signer/rotate`
7. 新建 `views/institution/policies/{Private,Gov,PublicSecurity}Form.tsx`
8. 改造 Dashboard 加全局视图 + 跨省置灰
9. `tsc --noEmit` + `npm run lint` 全绿
10. 浏览器手动:main 登录 → activation → roster 加 backup_1 → backup_1 登录 → activation
11. **更新文档**:`memory/05-modules/sfid/frontend/` 加新视图说明
12. **完善注释**:新页面顶部 1-3 行中文用途

## 验收清单

- `tsc --noEmit` + `npm run lint` + `npm run build` 全绿
- 浏览器 e2e:三 slot admin 登录 + 激活 + 加 backup + rotate
- 全局视图 + 跨省置灰按钮符合预期

## 工作量预估

- 净改动:~+900 行新增,~-800 行删除(keyring + operators 旧视图)
- 工时:~2d 集中开发 + 0.5d 验证 + 0.5d 文档/残留

## 提交策略

- feature branch:`sfid-step1-phase6-frontend-3tier-views`
- 单 PR 落地,commit message 引用任务卡 + ADR-008
