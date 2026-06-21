# node 管理员更换模块技术文档

最新更新：2026-05-10。

## 模块定位

节点桌面端管理员更换属于治理业务，但实现必须收口在独立目录，不再散落到机构管理或通用提案目录。

代码目录：

- `/Users/rhett/GMB/citizenchain/node/src/governance/admins_change/`：后端 Tauri 命令、管理员激活、链上 storage 解码、call data 构造、签名提交。
- `/Users/rhett/GMB/citizenchain/node/frontend/governance/admins_change/`：桌面前端管理员列表、管理员集合编辑、签名二维码流程。

边界：

- 不属于 `governance/organization-manage`。机构管理只负责机构和机构账户的注册、注销、索引查询；其“换管理员”按钮只作为入口跳转到 `admins_change`。
- 不在 `frontend/governance/` 根目录继续堆管理员更换页面；根目录只保留页面路由入口。
- 管理员激活/已激活管理员查询的前端 API 只放在 `frontend/governance/admins_change/api.ts`，根 `governance/api.ts` 不再承载这些方法。
- `storage_keys.rs` 只保留通用哈希与 AccountId 工具，`AdminsChange::AdminAccounts` 专用读取在 `admins_change/storage.rs`。

## 后端结构

```text
citizenchain/node/src/governance/admins_change/
├── mod.rs              # 模块导出和边界说明
├── activation.rs       # 管理员激活：生成激活签名请求、验证签名、本地加密存储
├── types.rs            # AdminAccountState DTO 与标签
├── account_id.rs       # AccountId / 管理员公钥 hex 规范化
├── codec.rs            # AdminAccount SCALE 解码与 BoundedVec<AccountId32> 编码
├── call_data.rs        # propose_admin_set_change call data 构造
├── validation.rs       # 桌面端前置校验
├── storage.rs          # AdminsChange::AdminAccounts storage key 与 RPC 读取
├── signing.rs          # QR 签名请求构造、回执验证、交易提交
└── commands.rs         # Tauri 命令入口
```

Tauri 命令：

- `build_activate_admin_request`：验证链上管理员身份，构建本地激活签名请求。
- `verify_activate_admin`：验证冷钱包激活签名并写入本地激活记录。
- `get_activated_admins`：按 subject 读取已激活管理员，并与链上当前管理员集合交叉校验；个人多签和机构账户必须附带 `accountIdHex + expectedOrg`。
- `deactivate_admin`：取消本地管理员激活。
- `get_admin_account_state`：按 `AdminAccountRef` 读取管理员主体。内置治理机构可用 `sfidNumber + expectedOrg`；个人多签和机构账户必须用 `accountIdHex + expectedOrg`。
- `build_admin_set_change_request`：校验当前管理员身份、主体 org 和新管理员集合，构建公民钱包签名请求。
- `submit_admin_set_change`：复用签名时 nonce/block，验证冷钱包回执并提交 extrinsic；提交前再次按同一 `AdminAccountRef` 读取主体。

管理员激活 payload：

```text
GMB_ACTIVATE_SUBJECT_V1
+ account_id(48)
+ org(u8)
+ kind(u8)
+ pubkey(32)
+ timestamp(u64 LE)
+ nonce(16)
```

激活 QR `display.action = activate_admin_account`，字段必须为 `org / subject / pubkey`，并与 wumin 公民钱包解码结果逐项一致。本地激活记录写入 `{app_data}/activated-admin-subjects.json`，只按 `accountIdHex / org / kind / pubkeyHex` 归档；旧 `activated-admins.json` 不读取、不迁移。

链上 call data：

```text
[pallet=12][call=0][org:u8][account_id:48][admins:Compact<Vec<AccountId32>>]
```

其中 `pallet=12` 对应 runtime `AdminsChange`，`call=0` 对应 `propose_admin_set_change`。

## 前端结构

```text
citizenchain/node/frontend/governance/admins_change/
├── index.ts
├── types.ts
├── api.ts
├── AdminListPage.tsx
├── AdminSetChangePage.tsx
├── AdminSetChangeSigningFlow.tsx
├── AdminWalletSelector.tsx
├── AdminSetEditor.tsx
├── AdminSetDiff.tsx
└── styles.css
```

页面流程：

1. 治理机构详情页或 `governance/organization-manage` 机构详情页点击“换管理员”。
2. `AdminSetChangePage` 读取 `AdminsChange::AdminAccounts`。
3. 用户选择已激活管理员钱包，编辑完整的新管理员集合。
4. 后端构建 `propose_admin_set_change` 签名请求。
5. 前端展示 WUMIN_QR_V1 二维码，扫码签名回执后提交。
6. 成功后返回机构详情页。

主体引用：

- `AdminAccountRef.sfidNumber`：仅用于 NRC / PRC / PRB 等内置治理机构，必须带 `org=0/1/2` 防止错主体。
- `AdminAccountRef.accountIdHex`：用于个人多签和机构账户，必须带 `org=3/4/5`。缺少 `accountIdHex` 时后端直接拒绝动态主体管理员激活和管理员更换。
- `offchain/organization-manage` 只提供页面入口和主账户 subject 元数据；管理员激活、更换读取、校验、QR 和提交仍全部走 `governance/admins_change`。

## 校验规则

- 主体必须为 `Active`。
- 发起签名公钥必须是当前管理员。
- 新管理员公钥必须为 32 字节 hex，不能重复。
- 新集合不能与当前集合完全相同。
- 内置治理机构固定人数：NRC 19，PRC 9，PRB 9。
- `注册机构归属关系` 只用于机构归属、检索、展示和反查，不允许作为管理员更换主体。
- 个人多签必须使用 `ORG_REN`，管理员数量：`2..=64`。
- 机构账户必须使用 `ORG_PUP / ORG_OTH`，管理员数量：`2..=1989`。
- 管理员激活 QR `display.fields` 必须与冷钱包解码保持一致：`org`、`subject`、`pubkey`。
- 管理员更换 QR `display.fields` 必须与冷钱包解码保持一致：`org`、`subject`、`admins`；`subject/admins` 使用 `0x` 小写 hex。

链端仍是最终裁判；桌面端校验只用于提前给出明确错误。
