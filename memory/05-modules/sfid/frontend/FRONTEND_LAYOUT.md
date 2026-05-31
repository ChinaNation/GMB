# SFID 前端目录布局

- 最后更新:2026-05-31
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
  - `memory/08-tasks/open/20260530-sfid-admins-module-unify.md`
  - `memory/08-tasks/open/20260530-sfid-province-admin-governance-passkey.md`
  - `memory/08-tasks/done/20260530-sfid-admin-permission-step2.md`
  - `memory/08-tasks/done/20260531-sfid-admin-ui-closeout.md`
  - `memory/08-tasks/done/20260531-sfid-admin-model-no-status.md`
  - `memory/08-tasks/open/20260531-sfid-signature-modal-stack.md`

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
├── common/                    # 跨业务复用组件,含 WUMIN_QR_V1 签名面板/弹窗
├── cpms/                      # CPMS 系统管理组件和 cpms/api.ts
├── hooks/                     # useAuth / useScope / useSfidMeta 等
├── institutions/              # 机构本地管理页面、institutions/api.ts、chain_duoqian_info.ts
├── qr/
├── sfid/                      # SFID 元数据 API,如省市/A3/机构类型选项
├── admins/                    # 省/市管理员页面、Passkey.tsx、operators_api.ts、admin_security_api.ts
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
- 公民电子护照绑定和 CPMS 状态扫码接口放 `citizens/api.ts`。
- 省/市管理员本地后台接口统一放 `admins/`;省管理员目录接口放 `admins/api.ts`,
  市管理员列表接口放 `admins/operators_api.ts`,Passkey 更新工具放 `admins/Passkey.tsx`。
- `common/WuminSignaturePanel.tsx` 与 `common/WuminSignatureModal.tsx` 是统一冷钱包签名 UI;
  登录页、Passkey 更新和管理员重要操作都复用登录页同款“左二维码 + 右扫码窗口”布局。
- `common/modalStack.ts` 是 SFID 前端弹窗层级唯一入口。普通业务弹窗固定在业务层,
  扫码账户弹窗在其上,Passkey 后续冷钱包签名弹窗固定在最高安全层。
- 管理端权限类型统一为 `LOGIN_STATE / PASSKEY / PASSKEY_CHALLENGE`;前端类型必须与后端
  `admins/operation_auth.rs` 对齐,不得恢复旧二级权限命名。
- `PASSKEY` 业务写操作不得直接裸调用 CRUD 端点;必须先通过
  `admins/admin_security_api.ts` 触发浏览器 Passkey 并取得一次性 grant。
- `PASSKEY_CHALLENGE` 写操作必须通过 `admins/admin_security_api.ts` 的 Passkey +
  `WUMIN_QR_V1` 冷钱包签名流程取得一次性 grant。
- `PASSKEY_CHALLENGE` 写操作触发 Passkey + 冷钱包签名时,不得为了规避遮挡而关闭编辑、新增或删除确认弹窗。
  正确顺序是:底层业务弹窗保持打开并进入 loading/禁用状态,浏览器 Passkey 原生验证完成后,
  `WuminSignatureModal` 以最高安全层展示在所有业务弹窗前面;签名成功后先关闭签名弹窗,
  再关闭或刷新原业务弹窗。失败或取消时底层业务弹窗保留,方便用户修改后重试。
- 签名弹窗扫码按钮不得复用底层业务 loading。底层业务 loading 只负责防止重复提交;
  扫码按钮只在已经识别到签名回执并提交 `commitAdminAction` 时进入 loading/禁用,
  Passkey 完成后刚打开签名弹窗时必须允许用户点击“开启扫码”。
- Passkey 更新流程固定为 `start -> confirm -> complete`:先扫描冷钱包签名请求并确认当前管理员,
  再调用浏览器 WebAuthn 创建凭据,最后提交后端落库;不得恢复先注册浏览器凭据再冷钱包确认的流程。

## 公民绑定弹窗 UI 口径

- `citizens/BindModal.tsx` 只保留单一绑定流程：扫描/上传 CPMS 档案码、展示 wuminapp `sign_request`、扫描 wuminapp `sign_response`、提交 SFID 绑定。
- 扫码框提示统一为“点击扫描档案码”；签名回执页提示为“点击扫描签名回执”。
- 进入签名二维码展示步骤后，弹窗标题切换为“wuminapp 签名”；进入签名回执扫描页后，弹窗标题切换为“扫描签名回执”。
- 绑定签名回执的 `sign_request.id` 必须与后端保存的 `challenge_id` 完全一致;
  不得给公民绑定挑战额外添加 `bind-` 前缀,否则 SFID 后端会查不到 challenge。
- “扫描档案码”步骤同时支持摄像头扫码和上传二维码图片;上传入口只在本地用
  `utils/cameraScanner.ts` 的 `BarcodeDetector` 解析图片,解析出的二维码原文继续走同一条档案码绑定流程,
  不把图片文件上传到后端。
- “上传二维码”按钮保持纯文字按钮;同一按钮组内的“开启扫码”没有图标,上传入口也不得额外增加图标。
- `citizens/CitizensView.tsx` 公民列表中 `sfid_code` 列标题显示为“身份ID”,不改变底层字段名。
- `citizens/CitizensView.tsx` 公民列表中 `wallet_address` 列标题显示为“投票账户”；列表状态列显示“投票状态”，由 `citizen_status + voting_eligible` 计算。
- 公民详情只展示“身份ID / 档案号 / 投票账户 / 绑定状态 / 选举权利 / 公民状态 / 有效期”，不得接收或展示签发地市归属。
- 公民身份列表右上角提供“导入年度报告”按钮，位于“新增身份ID绑定”左侧，开放给所有已登录管理员；导入弹窗只接收 CPMS 导出的 `CPMS_STATUS_EXPORT` JSON。
- 更换绑定弹窗的当前记录摘要只展示“档案号 / 身份ID / 投票账户”；签名请求摘要使用“选举权利 / 公民状态 / 投票账户”。
- 绑定弹窗生成签名挑战时只提交 `mode / archive_code_payload / citizen_id`；钱包字段只能来自 CPMS `ARCHIVE` 档案码。
- `citizens/CitizensView.tsx` 的表格行点击只负责打开详情;操作栏按钮必须阻止事件冒泡,
  点击“更换绑定”不得同时触发公民详情弹窗；顶部新增入口固定显示“新增身份ID绑定”。
- 本 UI 边界必须使用后端绑定协议字段：`wallet_pubkey / wallet_address / citizen_status / voting_eligible / vote_status / bind_status`。

## 管理员目录规则

- `admins/`:放省级管理员列表、注册局视图、市管理员维护和管理员安全写操作。
- 注册局-省管理员列表页面由 `ShengAdminSubTab.tsx` 承接,按“序号 / 姓名 / 账户 / 操作”表格展示。
- 省级管理员采用同级模型;每省最多 5 人,仅内置初始省级管理员拥有删除新增省级管理员的权限。
- 市级管理员列表必须显示 `本市市级管理员：x / 30`;市列表卡片显示该市 `x / 30`;
  达到 30 人的市禁用新增按钮和新增弹窗里的市选项,但最终上限仍以后端校验为准。
- `operators_api.ts` 保留市管理员列表读取和姓名登录态修改。
- `admin_security_api.ts` 负责 Passkey 注册、写操作 prepare/commit、浏览器 WebAuthn、
  `PASSKEY` grant、`PASSKEY_CHALLENGE` 冷钱包签名回执提交和管理员新增错误码文案转换。
- 管理员新增失败时，前端只能按 `ApiError.errorCode` 展示角色级重复、省级管理员上限和市级管理员上限提示，禁止解析后端
  `message`。
- 省级管理员和市级管理员都在管理员列表操作栏通过“更新密钥”使用 `Passkey.tsx`
  生成或重新生成本人 Passkey。
- 当 `auth.passkey_bound === false` 时,`Passkey.tsx` 只在当前登录管理员本人那一行的
  “更新密钥”按钮右上角显示红色角标;更新成功并刷新登录态后角标自动消失。
- 省/市管理员新增、删除必须走 `runSecuredAction`;编辑姓名只走登录态 PATCH 接口。
- 管理员列表不得展示状态栏,也不得保留启用/停用按钮。
- 编辑市管理员只允许调整管理员姓名;账户地址和市归属只读展示,不得在前端提交修改。
- 删除市管理员确认弹窗必须展示 SS58 地址,不直接展示 hex 公钥。
- 旧省管理员签名维护页不再作为 `App.tsx` 顶层 Tab 暴露,对应独立页面文件已删除。
- 登录角色和会话辅助类型放在 `auth/types.ts`。
- 本地开发的 Vite host 固定为 `localhost`;Passkey 开发配置依赖
  `http://localhost:5179`,不得用 `127.0.0.1` 或局域网 IP 打开前端注册 Passkey。
- `npm run dev` 使用 `vite preview --host localhost --port 5179 --strictPort`;端口被占用时
  必须失败,不得自动漂移到其它端口。

## 链交互目录规则

前端不再维护独立 `chain/` 目录。只要某功能模块需要和区块链交互,就在该功能
模块目录中创建 `chain_` 开头的文件。

| 前端文件 | 后端文件 | 职责 |
|---|---|---|
| `institutions/chain_duoqian_info.ts` | `institutions/chain_duoqian_info.rs` | 机构查询、注册信息凭证、清算行信息 |

省/市管理员治理 Passkey/冷钱包挑战不列入链交互表。
CPMS 系统管理也不列入链交互表,归 `cpms/`。

### `institutions/chain_duoqian_info.ts` 边界

- 不放 SFID 内部机构创建/修改页面,这些仍归 `frontend/institutions/`。
- 不再提供“备案”按钮、备案弹窗或备案状态组件。
- 机构列表的“清算行资格”列只在私权机构 Tab 显示;公安局和公权机构列表不得展示该列。
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
  "admins",
  "theme",
  "utils"
]
```
