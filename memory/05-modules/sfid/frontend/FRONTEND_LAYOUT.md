# SFID 前端目录布局

- 最后更新:2026-05-25
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-duoqian-info-layout.md`
  - `memory/08-tasks/open/20260502-114447-按业务边界重新设计并落地-sfid-省管理员相关前后端与-runtime-目录结构.md`
  - `memory/08-tasks/open/20260502-sfid-chain目录归并功能模块.md`
  - `memory/08-tasks/done/20260502-sfid-cpms-sheng目录整改.md`
  - `memory/08-tasks/done/20260502-sfid-frontend-api归并功能模块.md`
  - `memory/08-tasks/done/20260502-sfid-sheng-tabs.md`
  - `memory/08-tasks/done/20260502-sfid-sheng-backup-admin-ui.md`
  - `memory/08-tasks/done/20260525-sfid-cpms-archive-simplify.md`
  - `memory/08-tasks/done/20260525-sfid-bind-upload-qr.md`
  - `memory/08-tasks/done/20260525-sfid-bind-sign-request-wumin-scan.md`
  - `memory/08-tasks/done/20260525-sfid-bind-copy-myid-scan-square.md`
  - `memory/08-tasks/done/20260525-175008-sfid绑定签名回执与wuminapp启动anr修复.md`

## 当前边界

SFID 前端旧源码壳、旧 views 壳、旧全局业务 API 目录、旧全局链目录已删除。
前端不再保留“src + views”这层空壳,也不再维护全局业务 API 或全局链目录。
所有页面、hook、通用组件、业务 API 和链交互 API 都直接按业务目录放在 `sfid/frontend/` 下。

```text
sfid/frontend/
├── main.tsx
├── App.tsx
├── vite-env.d.ts
├── auth/                      # 登录、AuthContext、登录态类型、auth/api.ts
├── citizens/                  # 公民首页、绑定弹窗、citizens/api.ts
├── common/                    # 跨业务复用组件
├── cpms/                      # CPMS 系统管理组件和 cpms/api.ts
├── hooks/                     # useAuth / useScope / useSfidMeta 等
├── institutions/              # 机构本地管理页面、institutions/api.ts、chain_duoqian_info.ts
├── qr/
├── sfid/                      # SFID 元数据 API,如省市/A3/机构类型选项
├── sheng_admins/              # 省管理员/市管理员页面、roster_api.ts、signing_keys_api.ts
├── shi_admins/                # 市管理员页面、shi_admins/api.ts
├── theme/
└── utils/                     # 通用工具,http.ts 只放请求封装,不放业务 API
```

## API 目录规则

- 前端不再维护独立 `api/` 目录。某个功能需要后端 API 时,直接在所属功能目录新建 `api.ts`。
- `utils/http.ts` 只放 `request`、`adminRequest`、`adminHeaders` 和 401 拦截,不得放业务接口。
- `utils/http.ts` 收到 `401` 必须抛 `AuthExpiredError` 并触发全局退出;其他业务错误抛
  `ApiError`,页面按 `errorCode` 展示,不得返回 `undefined as T`。
- 登录/会话接口放 `auth/api.ts`;登录态和角色类型放 `auth/types.ts`。
- SFID 元数据接口放 `sfid/api.ts`,用于省份、市、A3、机构类型等跨页面选择项。
- 机构本地管理接口放 `institutions/api.ts`。机构与区块链交互继续放 `institutions/chain_duoqian_info.ts`。
- CPMS 系统管理接口放 `cpms/api.ts`;CPMS 组件放 `cpms/`。
- 公民绑定、解绑、推链绑定和 CPMS 状态扫码接口放 `citizens/api.ts`。
- 省管理员本地后台接口放 `sheng_admins/api.ts`;一主两备展示接口放
  `sheng_admins/roster_api.ts`;本人 signing seed 生成/更换接口放
  `sheng_admins/signing_keys_api.ts`。
- 市管理员操作员接口放 `shi_admins/api.ts`。

## 公民绑定弹窗 UI 口径

- `citizens/BindModal.tsx` 中新账户绑定档案的入口标题统一显示为“扫描档案码”。
- 新账户绑定档案时,弹窗不再展示模式说明和旧步骤标题。
- 扫码框提示统一为“点击扫描档案码”;签名环节继续显示签名二维码相关文案。
- 进入签名二维码展示步骤后,弹窗标题切换为“扫码签名”,不再展示“第二步：用公民钱包扫码签名”提示;
  进入签名回执扫描页后,弹窗标题切换为“扫描签名回执”,且不再展示“第三步：扫描签名结果二维码”。
  该口径只适用于签名回执扫描页,不得改动上一步“扫描档案码”。
- 绑定签名回执的 `sign_request.id` 必须与后端保存的 `challenge_id` 完全一致;
  不得给公民绑定挑战额外添加 `bind-` 前缀,否则 SFID 后端会查不到 challenge。
- “扫描档案码”步骤同时支持摄像头扫码和上传二维码图片;上传入口只在本地用
  `utils/cameraScanner.ts` 的 `BarcodeDetector` 解析图片,解析出的二维码原文继续走同一条档案码绑定流程,
  不把图片文件上传到后端。
- “上传二维码”按钮保持纯文字按钮;同一按钮组内的“开启扫码”没有图标,上传入口也不得额外增加图标。
- `citizens/CitizensView.tsx` 公民列表中 `sfid_code` 列标题显示为“身份ID”,不改变底层字段名。
- 绑定弹窗生成签名挑战时必须把当前钱包地址提交给 `citizens/api.ts` 的
  `citizenBindChallenge`,确保后端生成的 `sign_request.body.address/pubkey` 完整。
- `citizens/CitizensView.tsx` 的表格行点击只负责打开详情;操作栏按钮必须阻止事件冒泡,
  点击“绑定 / 确认 / 解绑”不得同时触发公民详情弹窗。
- 本 UI 边界只处理前端展示和组件内命名,不改变后端绑定协议字段。

## 省管理员目录规则

- `sheng_admins/`:放普通后台业务页面,例如省管理员列表、注册局视图、市管理员维护。
- 注册局-省级管理员页面由 `SuperAdminSubTab.tsx` 承接,竖向展示一主两备 3 个板块。
- 空备用管理员卡片只提供“扫码填入账户”的新增入口。
- `roster_api.ts` 做页面展示查询和本地备用管理员保存,不是链交互。
- `signing_keys_api.ts` 只做本人本地 signing seed 生成/更换,不是链交互。
- 省管理员只有“更换省管理员/主备交换”后续接入区块链时,才允许新增
  `chain_replace_admin.ts`。
- `省管理员名册`、`激活签名`、`rotate 签名` 不再作为 `App.tsx` 顶层 Tab 暴露,对应独立页面文件已删除。
- 省管理员槽位 `ShengSlot` 放在 `sheng_admins/types.ts`,不再放全局 `types/`。
- 登录角色和会话辅助类型放在 `auth/types.ts`。

## 链交互目录规则

前端不再维护独立 `chain/` 目录。只要某功能模块需要和区块链交互,就在该功能
模块目录中创建 `chain_` 开头的文件。

| 前端文件 | 后端文件 | 职责 |
|---|---|---|
| `institutions/chain_duoqian_info.ts` | `institutions/chain_duoqian_info.rs` | 机构查询、注册信息凭证、清算行信息 |

省管理员一主两备展示和本人 signing seed 生成/更换不列入链交互表。
CPMS 系统管理也不列入链交互表,归 `cpms/`。

### `institutions/chain_duoqian_info.ts` 边界

- 不放 SFID 内部机构创建/修改页面,这些仍归 `frontend/institutions/`。
- 不再提供“备案”按钮、备案弹窗或备案状态组件。
- 当前封装公开查询:
  - `getInstitutionInfo(sfidNumber)`:机构展示详情。
  - `getInstitutionRegistrationInfo(sfidNumber)`:链端注册信息凭证。
- 注册信息凭证的业务字段只有 `sfid_number / institution_name / account_names[]`;
  `credential` 下字段仅用于链端验签与防重放。

## TypeScript 覆盖

`sfid/frontend/tsconfig.json` 必须覆盖根层入口与一级业务目录:

```json
[
  "main.tsx",
  "App.tsx",
  "vite-env.d.ts",
  "auth",
  "citizens",
  "common",
  "cpms",
  "hooks",
  "institutions",
  "qr",
  "sfid",
  "sheng_admins",
  "shi_admins",
  "theme",
  "utils"
]
```
