# node 管理员更换模块技术文档

最新更新：2026-05-09。

## 模块定位

节点桌面端管理员更换属于治理业务，但实现必须收口在独立目录，不再散落到机构管理或通用提案目录。

代码目录：

- `/Users/rhett/GMB/citizenchain/node/src/governance/admins_change/`：后端 Tauri 命令、管理员激活、链上 storage 解码、call data 构造、签名提交。
- `/Users/rhett/GMB/citizenchain/node/frontend/governance/admins_change/`：桌面前端管理员列表、管理员集合编辑、签名二维码流程。

边界：

- 不属于 `offchain/organization-manage`。机构管理只负责机构和机构账户的注册、注销、索引查询；其“换管理员”按钮只作为入口跳转到 `admins_change`。
- 不在 `frontend/governance/` 根目录继续堆管理员更换页面；根目录只保留页面路由入口。
- 管理员激活/已激活管理员查询的前端 API 只放在 `frontend/governance/admins_change/api.ts`，根 `governance/api.ts` 不再承载这些方法。
- `storage_keys.rs` 只保留通用哈希与 SubjectId 工具，`AdminsChange::Subjects` 专用读取在 `admins_change/storage.rs`。

## 后端结构

```text
citizenchain/node/src/governance/admins_change/
├── mod.rs              # 模块导出和边界说明
├── activation.rs       # 管理员激活：生成激活签名请求、验证签名、本地加密存储
├── types.rs            # AdminSubjectState DTO 与标签
├── subject_id.rs       # SubjectId / 管理员公钥 hex 规范化
├── codec.rs            # AdminSubject SCALE 解码与 BoundedVec<AccountId32> 编码
├── call_data.rs        # propose_admin_set_change call data 构造
├── validation.rs       # 桌面端前置校验
├── storage.rs          # AdminsChange::Subjects storage key 与 RPC 读取
├── signing.rs          # QR 签名请求构造、回执验证、交易提交
└── commands.rs         # Tauri 命令入口
```

Tauri 命令：

- `build_activate_admin_request`：验证链上管理员身份，构建本地激活签名请求。
- `verify_activate_admin`：验证冷钱包激活签名并写入本地激活记录。
- `get_activated_admins`：读取已激活管理员，并与链上当前管理员集合交叉校验。
- `deactivate_admin`：取消本地管理员激活。
- `get_admin_subject_state`：按 `sfidNumber` 或 `subjectIdHex` 读取管理员主体。
- `build_admin_set_change_request`：校验当前管理员身份和新管理员集合，构建冷钱包签名请求。
- `submit_admin_set_change`：复用签名时 nonce/block，验证冷钱包回执并提交 extrinsic。

链上 call data：

```text
[pallet=12][call=0][org:u8][subject_id:48][new_admins:Compact<Vec<AccountId32>>]
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

1. 治理机构详情页或 `offchain/organization-manage` 机构详情页点击“换管理员”。
2. `AdminSetChangePage` 读取 `AdminsChange::Subjects`。
3. 用户选择已激活管理员钱包，编辑完整的新管理员集合。
4. 后端构建 `propose_admin_set_change` 签名请求。
5. 前端展示 WUMIN_QR_V1 二维码，扫码签名回执后提交。
6. 成功后返回机构详情页。

## 校验规则

- 主体必须为 `Active`。
- 发起签名公钥必须是当前管理员。
- 新管理员公钥必须为 32 字节 hex，不能重复。
- 新集合不能与当前集合完全相同。
- 内置治理机构固定人数：NRC 19，PRC 9，PRB 9。
- 个人多签管理员数量：`2..=64`。
- 机构账户 / 过渡 SFID 机构主体管理员数量：`2..=1989`。

链端仍是最终裁判；桌面端校验只用于提前给出明确错误。
