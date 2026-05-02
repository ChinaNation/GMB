# SFID 前端目录布局

- 最后更新:2026-05-02
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-duoqian-info-layout.md`
  - `memory/08-tasks/open/20260502-114447-按业务边界重新设计并落地-sfid-省管理员相关前后端与-runtime-目录结构.md`
  - `memory/08-tasks/open/20260502-sfid-chain目录归并功能模块.md`
  - `memory/08-tasks/done/20260502-sfid-frontend-api归并功能模块.md`
  - `memory/08-tasks/done/20260502-sfid-sheng-tabs.md`

## 当前边界

`sfid/frontend/src/`、`sfid/frontend/src/views/`、`sfid/frontend/api/`、`sfid/frontend/chain/`
已删除。前端不再保留“src + views”这层空壳,也不再维护全局业务 API 或全局链目录。
所有页面、hook、通用组件、业务 API 和链交互 API 都直接按业务目录放在 `sfid/frontend/` 下。

```text
sfid/frontend/
├── main.tsx
├── App.tsx
├── vite-env.d.ts
├── auth/                      # 登录、AuthContext、登录态类型、auth/api.ts
├── citizens/                  # 公民首页、绑定弹窗、citizens/api.ts
├── common/                    # 跨业务复用组件
├── hooks/                     # useAuth / useScope / useSfidMeta 等
├── institutions/              # 机构本地管理页面、institutions/api.ts、chain_duoqian_info.ts
├── qr/
├── sfid/                      # SFID 元数据 API,如省市/A3/机构类型选项
├── sheng_admins/              # 省管理员业务页面、sheng_admins/api.ts、chain_* 链交互页面/API
├── shi_admins/                # 市管理员页面、shi_admins/api.ts
├── theme/
└── utils/                     # 通用工具,http.ts 只放请求封装,不放业务 API
```

## API 目录规则

- 前端不再维护独立 `api/` 目录。某个功能需要后端 API 时,直接在所属功能目录新建 `api.ts`。
- `utils/http.ts` 只放 `request`、`adminRequest`、`adminHeaders` 和 401 拦截,不得放业务接口。
- 登录/会话接口放 `auth/api.ts`;登录态和角色类型放 `auth/types.ts`。
- SFID 元数据接口放 `sfid/api.ts`,用于省份、市、A3、机构类型等跨页面选择项。
- 机构与 CPMS 站点管理接口放 `institutions/api.ts`。机构与区块链交互继续放 `institutions/chain_duoqian_info.ts`。
- 公民绑定、解绑、推链绑定和 CPMS 状态扫码接口放 `citizens/api.ts`。
- 省管理员本地后台接口放 `sheng_admins/api.ts`;省管理员链交互接口/页面保持 `chain_` 前缀。
- 市管理员操作员接口放 `shi_admins/api.ts`。

## 省管理员目录规则

- `sheng_admins/`:放普通后台业务页面,例如省管理员列表、注册局视图、市管理员维护。
- 省管理员功能与链交互的页面、API 和类型也放在 `sheng_admins/`,文件名必须以 `chain_` 开头:
  - `chain_RosterPage.tsx`
  - `chain_ActivationPage.tsx`
  - `chain_RotatePage.tsx`
  - `chain_sheng_admins.ts`
  - `chain_sheng_admins_types.ts`
- `省管理员名册`、`激活签名`、`rotate 签名` 不再作为 `App.tsx` 顶层 Tab 暴露。
  这些链交互页面/API 暂保留在 `sheng_admins/chain_*`,后续统一并入注册局页面承接。
- 省管理员槽位 `ShengSlot` 属于链上 `ShengAdmins[Province][Slot]` 名册语义,
  因此放在 `sheng_admins/chain_sheng_admins_types.ts`,不再放全局 `types/`。
- 登录角色和会话辅助类型放在 `auth/types.ts`。

## 链交互目录规则

前端不再维护独立 `chain/` 目录。只要某功能模块需要和区块链交互,就在该功能
模块目录中创建 `chain_` 开头的文件。

| 前端文件 | 后端文件 | 职责 |
|---|---|---|
| `institutions/chain_duoqian_info.ts` | `institutions/chain_duoqian_info*.rs` | 机构查询、注册信息凭证、清算行信息 |
| `sheng_admins/chain_*.tsx` / `chain_sheng_admins*.ts` | `sheng_admins/chain_*.rs` | 省管理员三槽名册、签名公钥激活/轮换 |

### `institutions/chain_duoqian_info.ts` 边界

- 不放 SFID 内部机构创建/修改页面,这些仍归 `frontend/institutions/`。
- 不再提供“备案”按钮、备案弹窗或备案状态组件。
- 当前封装公开查询:
  - `getInstitutionInfo(sfidId)`:机构展示详情。
  - `getInstitutionRegistrationInfo(sfidId)`:链端注册信息凭证。
- 注册信息凭证的业务字段只有 `sfid_id / institution_name / account_names[]`;
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
