# App.tsx 拆分方案(精确版)

> 配套任务卡:`20260408-sfid-frontend-app-tsx-split.md`
> 本文件是**执行方案**,列出精确的状态/handler/JSX 归属。

## 当前现状(2026-04-08)

- `sfid/frontend/src/components/App.tsx` = **3431 行 / 1 个 React 组件**
- 60+ 个 `useState` / 10+ 个 `useRef` / 5+ 个 `Form.useForm` 全部在顶层组件里
- 11 个 `capabilities.*` 分支 + 8 个 `activeView` 分支堆在同一个 return
- `views/` 目录目前只有:
  - `institutions/`(任务卡 3 已拆)
  - `common/`(只有 `ProvinceGrid.tsx` / `CityGrid.tsx`)
  - 任务卡 `20260408-sfid-public-security-cpms-embed` 刚新增的 `CpmsSitePanel.tsx` / `CpmsRegisterModal.tsx` / `InstitutionDetailPage.tsx` 也在 institutions/

## 目标结构

```
src/
├── components/
│   └── App.tsx                          # ≤300 行:Layout 壳子 + 登录守卫 + tab 路由
├── contexts/
│   └── AuthContext.tsx                  # 新增:全局 auth 状态 Provider
├── hooks/
│   ├── useAuth.ts                       # 已存在,整合 AuthContext
│   ├── useCapabilities.ts               # 已存在
│   ├── useScope.ts                      # 已存在
│   └── useSfidMeta.ts                   # 新增:sfidMeta + cities 共享
├── utils/
│   ├── cameraScanner.ts                 # 已存在
│   ├── downloadQr.ts                    # 新增:提取 downloadQrFromRef
│   └── storedAuth.ts                    # 新增:localStorage auth 读写
├── views/
│   ├── auth/
│   │   ├── LoginView.tsx                # 二维码登录 + 挑战应答
│   │   └── login-qr-hooks.ts
│   ├── citizens/
│   │   ├── CitizensView.tsx             # 注册局:citizen 列表 + 搜索
│   │   ├── BindModal.tsx                # 有账户绑档案 + 有档案绑账户
│   │   ├── UnbindModal.tsx              # 解绑
│   │   └── citizen-columns.tsx          # 表格列定义
│   ├── operators/
│   │   ├── OperatorsView.tsx            # ShiAdmin 市管理员 CRUD
│   │   ├── CreateOperatorModal.tsx
│   │   └── OperatorScanner.tsx          # QR 扫码注册市管理员
│   ├── sheng-admins/
│   │   ├── ShengAdminsView.tsx          # 省管理员列表 + 替换
│   │   └── ReplaceShengAdminModal.tsx
│   ├── keyring/
│   │   ├── KeyringView.tsx              # KeyAdmin 密钥轮换
│   │   └── KeyringRotateModal.tsx
│   ├── multisig-legacy/
│   │   ├── LegacyMultisigView.tsx       # 老表(保留作兜底,后续可删)
│   │   └── GenerateMultisigModal.tsx
│   ├── system-settings/
│   │   └── SystemSettingsView.tsx       # 系统设置
│   ├── institutions/                    # 已存在,保持
│   └── common/                          # 已存在,保持
└── api/
    └── client.ts                        # 保持,只是使用方变化
```

## 模块状态 / handler 归属清单

### 模块 1:`auth/` — 登录鉴权

**State**:
- `auth` / `setAuth`(**提升到 AuthContext**)
- `bootstrapping` / `pendingQrLogin` / `challengeLoading`
- `videoRef` / `scannerRef` / `videoMounted` / `scannerActive` / `scannerReady` / `scanSubmitting`
- `error`

**Handlers**:
- `onCreateQrLogin` / `stopScanner` / `onCompleteSignedLogin` / `onToggleScanner` / `onLogout`
- 对应 useEffect:`[scannerActive, pendingQrLogin]` / QR 轮询 `[auth, pendingQrLogin]`

**保留在 App.tsx**:`auth` 本身 → 移到 `AuthContext`,App.tsx 只消费 `useAuth()`

---

### 模块 2:`citizens/` — 注册局(最大的一块)

**State**:
- `rows` / `loading` / `binding`(citizen 列表)
- **绑定流程**:`bindModalOpen` / `bindTargetPubkey` / `bindMode` / `bindTargetRecord` / `bindChallenge` / `bindChallengeLoading` / `bindQr4Payload` / `bindQr4ScanLoading` / `bindSignature` / `bindStep` / `bindNewPubkey` / `bindScannerActive` / `bindScannerReady` / `bindVideoRef` / `bindScanCleanupRef`
- **解绑流程**:`unbindModalOpen` / `unbindTarget` / `unbindChallenge` / `unbindChallengeLoading` / `unbindScannerActive` / `unbindScannerReady` / `unbindSubmitting` / `unbindStep` / `unbindVideoRef` / `unbindScanCleanupRef`

**Handlers**:
- `refreshList` / `onSearch`
- `openBindModal` / `onScanBindQr4` / `onBindPubkeyNext` / `onScanBindSignature` / `stopBindScanner` / `onToggleBindScanner`
- `openUnbindModal` / `stopUnbindScanner` / `onUnbindGenerateChallenge` / `onScanUnbindSignature`

**文件拆分**:
- `views/citizens/CitizensView.tsx` — 顶层容器,持有 `rows` / `loading`,渲染 Table + 搜索栏 + BindModal + UnbindModal
- `views/citizens/BindModal.tsx` — 纯 Modal 组件,接收 `record` + `onDone`,内部持有 bind 相关所有 state
- `views/citizens/UnbindModal.tsx` — 同上,unbind state

---

### 模块 3:`operators/` — 市管理员

**State**:
- `operators` / `operatorsLoading` / `operatorPage` / `operatorListPage`
- `addOperatorOpen` / `addOperatorLoading` / `addOperatorForm`
- `operatorCities` / `operatorCitiesLoading`
- `opScanOpen` / `opScanType` / `opScannerReady` / `opScanSubmitting` / `opVideoRef` / `opScanCleanupRef`

**Handlers**:
- `refreshOperators` / `onCreateOperator` / `onToggleOperatorStatus` / `onUpdateOperator` / `onDeleteOperator`
- `stopOpScanner` / `onHandleOperationQr`

**注意**:`onHandleOperationQr` 本次 CPMS 清理后只剩 QR4 citizen 状态扫描路径,严格说这属于 citizens 模块。拆分时要把它挪到 `citizens/` 而不是 `operators/`。

**文件**:
- `views/operators/OperatorsView.tsx`
- `views/operators/CreateOperatorModal.tsx`

---

### 模块 4:`sheng-admins/` — 省管理员

**State**:
- `shengAdmins` / `shengAdminsLoading` / `selectedShengAdmin` / `adminDetailTab`
- `replaceSuperLoading` / `replaceSuperForm`

**Handlers**:
- `refreshShengAdmins` / `onReplaceShengAdmin`

**文件**:
- `views/sheng-admins/ShengAdminsView.tsx`
- `views/sheng-admins/ReplaceShengAdminModal.tsx`

---

### 模块 5:`keyring/` — 密钥管理员轮换

**State**:
- `keyringState` / `keyringLoading` / `keyringActionLoading` / `keyringChallenge` / `keyringSignedPayload`
- `keyringScannerActive` / `keyringScannerReady` / `keyringScanSubmitting` / `keyringCommitLoading`
- `keyringVideoRef` / `keyringScanCleanupRef` / `keyringForm`
- `keyringScanAccountOpen` / `accountScanTarget`

**Handlers**:
- `refreshKeyringState` / `stopKeyringScanner` / `onCreateKeyringRotateChallenge` / `onCompleteKeyringRotate` / `onToggleKeyringScanner`

**文件**:
- `views/keyring/KeyringView.tsx`
- `views/keyring/KeyringRotateModal.tsx`

---

### 模块 6:`multisig-legacy/` — **删除死代码**(不拆分)

**背景重判**:`activeView === 'multisig'` 分支实际渲染的是 `<InstitutionsView category="PRIVATE_INSTITUTION">`(App.tsx L2334-2335),**所有 multisig 相关 state / handler / Modal 均不可达**,是任务卡 2 两层模型切换后遗留的死代码。

**动作**:整块删除,不抽取到新 view。
- State 删:`multisigRows` / `multisigLoading` / `multisigModalOpen` / `multisigGenerating` / `multisigPage` / `multisigA3` / `multisigForm`
- Handler 删:`refreshMultisigSfids` / `onDeleteMultisigSfid` / `onGenerateMultisigSfid` / `openMultisigModal`
- JSX 删:L3155-3245 的"生成机构SFID" Modal(整块不可达)
- Import 删:`GenerateMultisigSfidResult` / `MultisigSfidRow` / `deleteMultisigSfid` / `generateMultisigSfid` / `listMultisigSfids`
- `activeView === 'multisig'` 分支**保留**,继续渲染 `<InstitutionsView category="PRIVATE_INSTITUTION">`

**api/client.ts 连带清理**:`listMultisigSfids` / `deleteMultisigSfid` / `generateMultisigSfid` / `MultisigSfidRow` / `GenerateMultisigSfidResult`(整仓库 Grep 确认无其他消费者)。

---

### 模块 7:`system-settings/` — **不存在,跳过**

**背景重判**:`mainAccountBalance` 实际属于 keyring 模块(密钥管理页面展示主账户链上余额);`activeView === 'system-settings'` 分支渲染的是 KeyAdmin 的省管理员列表 / 详情页,归属 operators + sheng-admins 模块。**本方案不创建 system-settings view**。

---

### 共享 hooks / contexts

**`contexts/AuthContext.tsx`**:
```tsx
interface AuthContextValue {
  auth: AdminAuth | null;
  setAuth: (next: AdminAuth | null) => void;
  capabilities: RoleCapabilities;
  logout: () => void;
}
```
App.tsx 外层 Provider 包裹,内层所有 view 用 `useAuth()` 拿。

**`hooks/useSfidMeta.ts`**(新):
```tsx
function useSfidMeta(auth: AdminAuth | null) {
  // 懒加载 sfidMeta,共享给所有需要的 view
  // 返回 { meta, loadCities(province), cities, loading }
}
```

**`utils/storedAuth.ts`**(新):
- `readStoredAuth()` / `clearStoredAuth()` / `saveStoredAuth()`
- 从 App.tsx 里摘出来(目前应该就散在顶部)

**`utils/downloadQr.ts`**(新):
- `downloadQrFromRef(container, filename)` —— 复用已在 CpmsSitePanel 里写的一份,统一到这里
- CpmsSitePanel 改成 import

## 执行顺序(7 步,每步独立 commit)

| 步 | 内容 | 交付 | 验收 |
|---|---|---|---|
| **1** | **删除 multisig-legacy 死代码** | state / handler / Modal / api imports 整块删除;activeView === 'multisig' 分支保留 | tsc + build 绿;App.tsx 行数下降 |
| **2** | 拆 `operators/` + `sheng-admins/` | 两个管理员模块一起(原步 3) | build 绿 + 手工点增删改 |
| **3** | 拆 `keyring/` | 密钥轮换独立流程,有摄像头扫码(原步 4) | build 绿 + 手工发起一次轮换(不完成) |
| **4** | 拆 `citizens/` | **最大的一块**,含绑定/解绑双流程(原步 5) | build 绿 + 手工扫码绑定一次 |
| **5** | 拆 `auth/` (LoginView) | 独立登录 UI(原步 6) | build 绿 + 登出登录一次 |
| **6** | App.tsx 最终清理 | 删除所有已迁走的 state / handler / import(原步 7) | **App.tsx ≤ 300 行**;`npm run build` 绿 |

> system-settings 不单独成步,其 JSX 分支在步 2 随 operators + sheng-admins 一起迁走。
> 原"步 0 预备 + AuthContext"已在前序任务合入,本轮不再重复。

## 每一步的动作模板

```
1. 在目标子目录新建 <ModuleView>.tsx
2. 用 Grep 找该模块所有 state / handler / JSX 行号
3. 整体迁移代码到新文件,保留中文注释
4. 新 View 的 props 定义(一般是 auth + onLogout + capabilities + 回调)
5. App.tsx 对应 tab 的 JSX 替换成 <ModuleView {...} /> 一行
6. 删 App.tsx 里迁走的 state / handler / helper / imports
7. tsc --noEmit 必须绿
8. npm run build 必须绿
9. 手工点一遍该 tab 所有按钮
10. commit:"feat(sfid/frontend): extract <module> view from App.tsx"
```

## 风险清单

1. **摄像头 ref 迁移易失** —— 绑定/解绑/扫码登录/密钥轮换/操作员扫码 **5 处**摄像头 video ref。每处拆出去时要确保 `startCameraScanner` cleanup 正确(组件卸载时)。
2. **Auth state 改成 Context 后,useEffect 依赖更新可能触发多余重渲染** —— 跑一次登录完整流程确认没有无限循环。
3. **`sfidMeta` 被多处共享** —— operators / keyring / multisig-legacy / institutions 都用。改成 `useSfidMeta` hook 后首次加载时机要处理好(懒加载 + 缓存)。
4. **`onHandleOperationQr` 模块归属易混淆** —— 看起来在 operators 里(因为 `opScan*` 前缀),但实际路径只处理 QR4 citizen 状态,应归 `citizens/`。
5. **每次拆完必须 build 绿**,任何步骤 build 失败要立即回滚当前步,而不是硬改。
6. **commits 不合并** —— 每步一个 commit,便于 bisect 出问题的步骤。
7. **不做非拆分工作** —— 禁止顺手"优化"业务逻辑、改接口签名、改 UI 样式。只是文件搬家。

## 工作量

- 步 0:预备 + AuthContext 切换 — **1.5 小时**
- 步 1:multisig-legacy — **1 小时**
- 步 2:system-settings — **0.5 小时**
- 步 3:operators + sheng-admins — **2 小时**
- 步 4:keyring — **2 小时**
- 步 5:citizens(最大) — **4 小时**
- 步 6:auth(LoginView) — **1.5 小时**
- 步 7:App.tsx 最终清理 — **1 小时**

**合计约 13~14 小时,排在 2~3 天完成**(每天 5~6 小时实打实编辑 + 验证)。

## 不做的事(严格铁律)

- 不换 UI 库、不换状态管理库(不引入 redux/zustand)、**不引入 React Router**
- 不改 `api/` 任何接口签名
- 不动 `views/institutions/` 现有结构
- 不动后端
- 不趁机重构业务逻辑、不优化性能
- 不删老的 `views/common/` 里已有文件
- 不引入新的第三方依赖

## 完成记录(2026-04-08)

配套主任务卡 `20260408-sfid-frontend-app-tsx-split.md` 全部 6 步收官完成。
App.tsx 从 3431 行拆至 357 行,tsc + build 全绿。详见主任务卡的"完成记录"段。
