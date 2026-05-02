# SFID 前端目录布局

- 最后更新:2026-05-02
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-duoqian-info-layout.md`
  - `memory/08-tasks/open/20260502-114447-按业务边界重新设计并落地-sfid-省管理员相关前后端与-runtime-目录结构.md`

## 当前边界

`sfid/frontend/src/` 与 `sfid/frontend/src/views/` 已删除。前端不再保留
“src + views” 这层空壳,所有页面、hook、通用组件和 API 都直接按业务目录放在
`sfid/frontend/` 下。

```text
sfid/frontend/
├── main.tsx
├── App.tsx
├── vite-env.d.ts
├── api/
│   ├── client.ts              # 共享 HTTP client / AdminAuth 类型
│   └── institution.ts         # 机构本地数据 API
├── auth/                      # 登录、AuthContext、登录态类型
├── citizens/                  # 公民首页、绑定弹窗
├── common/                    # 跨业务复用组件
├── hooks/                     # useAuth / useScope / useSfidMeta 等
├── institutions/              # 机构本地管理页面
├── qr/
├── sheng_admins/              # 非链上的省管理员本地业务页面
├── shi_admins/                # 市管理员页面
├── theme/
├── utils/
└── chain/
    ├── duoqian_info/          # 机构与 DUOQIAN 链交互 UI/API
    └── sheng_admins/          # 省管理员与 sfid-system 链交互 UI/API
```

## 省管理员目录规则

- `sheng_admins/`:放普通后台业务页面,例如省管理员列表、注册局视图、市管理员维护。
- `chain/sheng_admins/`:只放省管理员功能与链交互的页面、API 和类型:
  - `RosterPage.tsx`
  - `ActivationPage.tsx`
  - `RotatePage.tsx`
  - `api.ts`
  - `types.ts`
- 省管理员槽位 `ShengSlot` 属于链上 `ShengAdmins[Province][Slot]` 名册语义,
  因此放在 `chain/sheng_admins/types.ts`,不再放全局 `types/`。
- 登录角色和会话辅助类型放在 `auth/types.ts`。

## 链交互目录规则

前端 `chain/` 目录与后端 `sfid/backend/src/chain/`、runtime
`citizenchain/runtime/otherpallet/sfid-system/src/` 保持同名业务目录:

| 前端目录 | 后端目录 | runtime 目录 | 职责 |
|---|---|---|---|
| `chain/duoqian_info/` | `chain/duoqian_info/` | `sfid-system/src/duoqian_info/` | 机构备案、机构链信息状态 |
| `chain/sheng_admins/` | `chain/sheng_admins/` | `sfid-system/src/sheng_admins/` | 省管理员三槽名册、签名公钥激活/轮换 |

## TypeScript 覆盖

`sfid/frontend/tsconfig.json` 必须覆盖根层入口与一级业务目录:

```json
[
  "main.tsx",
  "App.tsx",
  "vite-env.d.ts",
  "api",
  "auth",
  "chain",
  "citizens",
  "common",
  "hooks",
  "institutions",
  "qr",
  "sheng_admins",
  "shi_admins",
  "theme",
  "utils"
]
```
