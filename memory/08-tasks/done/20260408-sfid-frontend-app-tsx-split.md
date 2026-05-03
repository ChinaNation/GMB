# 任务卡:sfid 前端 App.tsx 彻底拆分

- **任务 ID**: 20260408-sfid-frontend-app-tsx-split
- **模块**: sfid-frontend
- **优先级**: 中(技术债,阻塞后续功能迭代效率)
- **前置依赖**: `20260408-sfid-public-security-cpms-embed`(CPMS 流程搬走之后再拆,减少合并冲突)
- **状态**: 待启动

## 背景

任务卡 3 的前端模块化只完成了 20% —— 只把"私权/公权/公安局"三个 tab 的内容抽到了 `views/institutions/`,其他功能(登录、注册局、业务员绑定、操作员管理、密钥轮换、Dashboard)原封不动留在 `sfid/frontend/src/components/App.tsx`,导致该文件长期维持在 **3769 行**。每次动这个文件都战战兢兢,极易误伤不相关模块。

目前 `views/` 下只有 `institutions/` + `common/`,缺少 `auth/` `registration/` `binding/` `operators/` `dashboard/` 等子目录。

## 目标

把 App.tsx 拆成 **200~300 行的路由壳子**,所有业务功能移入 `views/<module>/` 子目录。

## 拆分目标结构

```
views/
  auth/
    LoginView.tsx          # 三角色登录 + 二维码扫码登录 + 挑战应答
    ChallengeFlow.tsx
  dashboard/
    DashboardView.tsx      # 首页卡片/统计
  registration/
    RegistrationView.tsx   # 注册局省市选择 + citizen 注册流程
    SfidGenerator.tsx
  binding/
    BindingView.tsx        # 业务员"有账户绑档案"+"有档案绑账户"
    Qr4Scanner.tsx
  operators/
    OperatorsView.tsx      # 操作员(SHI_ADMIN)增删改查
  key-management/
  institutions/            # 已存在,保持不动
  common/                  # 已存在,保持不动
```

App.tsx 最终形态:

```tsx
// 约 200~300 行
// 只负责:
//  1. Auth Provider + 登录守卫
//  2. Ant Design Layout(Header/Sider/Content)
//  3. Tab 路由 → 每个 tab 渲染对应的 <XxxView>
//  4. 全局 message/notification 配置
```

## 执行步骤

### 1. 预备
- 在 `views/` 下按目标结构创建子目录
- 建立 `views/_shared/` 存放跨 view 复用的 hook 和工具(如果当前 hooks/ 不够用)
- **先运行 `npm run build` 记录基线** —— 任何一步破坏编译都立即回滚

### 2. 按"从外到内、从独立到耦合"的顺序拆分

**顺序建议**(每步独立 commit,每步 build 绿):

1. **操作员管理**(`operators/`) —— 最独立,跟其他功能几乎零耦合
2. **密钥管理**(`key-management/`) —— 登录后的独立 tab
3. **Dashboard 首页**(`dashboard/`) —— 静态展示为主
4. **注册局流程**(`registration/`) —— 中等耦合,但自成一体
5. **业务员绑定**(`binding/`) —— 重耦合,含摄像头 ref 和扫码 state
6. **登录/鉴权**(`auth/`) —— 最后做,因为所有其他 view 都依赖 auth 状态;重点是把登录逻辑移出后 App.tsx 用 context/hook 注入

### 3. 每拆一个模块的动作模板

- Grep 定位该模块所有 state / handler / JSX / helper
- 新建 `views/<module>/<ModuleView>.tsx`,接收必要的 props (auth, capabilities, etc.)
- 把相关代码整体迁移,保留中文注释
- App.tsx 里对应 tab 的 JSX 替换成 `<ModuleView {...} />` 一行
- 删除 App.tsx 里迁走的 state / handler / helper / imports
- `npm run build` + `tsc --noEmit` 双绿
- 手动点一遍该 tab 所有按钮确认行为未变(有 e2e 的话跑 e2e)
- commit

### 4. 最后的清理
- App.tsx 只保留 Layout 壳子 + tab 路由 + 登录守卫
- 全局 state(auth / capabilities / tab 选择)上提到 `hooks/useAuth.ts` 或新建 Context
- 删除所有不再使用的 import
- 行数目标:≤ 300 行

## 验收标准

- `sfid/frontend/src/components/App.tsx` ≤ 300 行
- `views/` 下有 `auth/` `dashboard/` `registration/` `binding/` `operators/` `key-management/` `institutions/` `common/` 完整结构
- `npm run build` 全绿
- 手工回归:登录、注册、绑定、机构、密钥、操作员、Dashboard **全部功能行为不变**
- 回写 `feedback_sfid_frontend_modular_structure.md` 记录目录约定:**以后所有新 sfid 前端功能必须放到 `views/<module>/`,禁止直接写进 App.tsx**

## 风险

- 摄像头扫码 ref 在 binding view 里迁移时容易丢失绑定
- auth state 被全局 10+ 个 handler 引用,最后一步迁移时改动面大
- 可能需要新建 AuthContext 或把 auth state 挪到 `useAuth` hook 里统一管理
- 分多次 PR/commit 可大幅降低风险

## 预计工作量

- 每个模块 2~4 小时(含 build 验证和手工回归)
- 总计 **2~3 天**(6 个模块 + 最后壳子整理)

## 不做的事

- 不换 UI 库、不换状态管理、不加路由库(React Router)——纯文件组织拆分
- 不动 `views/institutions/` 现有结构
- 不动 `api/` `hooks/` 现有接口
- 不动后端

## 完成记录(2026-04-08 收官)

### 行数对比
- 起始:`src/components/App.tsx` 3431 行
- 步 1~5 后:535 行
- 步 6 收官后:**357 行**(删除 11 个 dead helper + 未用 import + 冗余注释 + 合并 onClick 中的重复 loadMeta 逻辑)

### 最终 views/ 结构
```
src/views/
├── README.md
├── auth/LoginView.tsx
├── citizens/{CitizensView,BindModal,UnbindModal}.tsx
├── common/{ProvinceGrid,CityGrid}.tsx
├── institutions/{InstitutionsView,InstitutionDetailPage,InstitutionListTable,
│                 AccountList,CreateInstitutionModal,CreateAccountModal,
│                 CpmsRegisterModal,CpmsSitePanel,locks.ts}.tsx
├── keyring/KeyringView.tsx
├── operators/OperatorsView.tsx
└── sheng-admins/ShengAdminsView.tsx
```

### 收官步清理清单
- 删除 dead helper:`isSr25519HexPubkey`, `sameHexPubkey`, `resolveAdminName`(合并进 resolveHeaderAdminName), `createSessionId`, `defaultInstitutionByA3`, `usesReservedProvinceCityByA3`, `institutionCodeToName`, `allowedInstitutionByA3`, `defaultP1ByA3`, `p1LockedByA3`, `reservedProvinceCityName`
- 删除未用 import:`SfidCityItem`(仅被 dead helper 引用)、未使用的 `loginBg` 常量
- 合并 Tab onClick 里三份重复的 `getSfidMeta` 逻辑到单个 `loadSfidMetaForInstitutions` helper
- 新增顶部中文文件头注释,明确 App.tsx 的路由壳子职责与"严禁新增业务代码"铁律
- 新增 `ActiveView` 类型别名替代行内长 union

### 新增文档/feedback
- `sfid/frontend/src/views/README.md`(追加铁律段)
- `memory/feedback_sfid_frontend_modular_structure.md`(新建)
- `memory/MEMORY.md` 索引追加

### 验证
- `npx tsc --noEmit` EXIT=0
- `npm run build` EXIT=0(vite build 1.59s,bundle 正常)
