# 节点桌面清算行 tab 技术说明

> ⚠️ **过时警告(2026-06-29)**:本文描述 B0 重构前的 node 结构。B0 已把机构创建/管理下沉 onchina——
> 删除了 `node/.../governance/organization_manage/`(含 create-multisig 页与 build/submit_propose_create_institution 命令);
> node 现仅保留清算行所需的机构**只读**:链上直读在 `node/src/transaction/offchain/institution_read/`
> (按机构码路由 `PublicManage(30)`/`PrivateManage(31)` 前缀,取代已删的 `OrganizationManage`),前端在 `frontend/transaction/offchain/institution/`。
> 机构创建/关闭统一走 onchina 控制台 + 冷钱包。本文余下内容待按 B0 + 公私拆分重写。
>
> **2026-07-23 增补**:第 0/4/5 节与 Tauri 命令表已按 `registration-info` 死链路删除同步修正(命令表路径改真实目录、删已删命令行、时序节改写为节点声明时序)。
> 第 7「验收」与第 8「变更记录」是历史快照,按审计铁律不动。第 6 节 follow-up 表的 F3/F4 指向已随机构创建移出 node 的能力,待 owner 确认后处置。

- 日期: 2026-05-02
- 任务卡:
  - `memory/08-tasks/done/20260501-node-clearing-bank-institution-detail-and-create.md`
  - `memory/08-tasks/done/20260502-node-offchain-duplicate-cleanup.md`
  - `memory/08-tasks/done/20260502-multisig-registration-info-align.md`
- 承接: `20260501-cid-chain-folder-restructure.md`(OnChina 端 chain 目录重构)

## 0. 概览

节点桌面"清算行"tab 提供 3 类核心能力:

1. **添加清算行**:输入 CID → 链上判定多签是否存在 → 已存在显示详情 / 不存在进创建流程
2. **机构详情**:展示链上 `organization-manage::Institutions[cid_number]` 全部信息 + 折叠卡片入口(其他账户/管理员)+ 节点声明状态 + 提案列表
3. **节点声明**:机构多签已存在时,在本 tab 完成清算行节点注册、端点更新与注销。机构创建与机构治理归 OnChina 控制台,节点不承接(见第 4 节)

## 1. 状态机(`offchain/section.tsx`)

```
empty → add-input-cid (debounce 自动搜,无"查询"按钮)
      → 选 candidate → check-multisig (链上查 Institutions[cid_number])
        ├── 已存在 → institution-detail
        │   ├── other-accounts-list  (折叠子页)
        │   ├── admin-list           (折叠子页)
        │   ├── 节点已声明 → 内联展示节点信息
        │   └── 节点未声明 + status=Active → declare-node
        │
        └── 不存在 → create-multisig-institution
            └── 提案提交成功 → wait-vote (轮询 status === 'Active')
                └── Active → declare-node
```

## 2. 文件清单

| 文件 | 职责 |
|---|---|
| `citizenchain/node/frontend/transaction/offchain-transaction/section.tsx` | 状态机驱动;EmptyView / CheckMultisigView / WaitVoteView 子组件 |
| `citizenchain/node/frontend/governance/organization_manage/add-candidate.tsx` | ClearingBankAddPage:debounce 自动搜 CID 候选(2026-05-01 删"查询"按钮) |
| `citizenchain/node/frontend/governance/organization_manage/institution-detail.tsx` | ClearingBankInstitutionDetailPage:卡片栅格 + 折叠子页入口 + 节点信息 + 发起提案占位 + 提案列表 |
| `citizenchain/node/frontend/governance/organization_manage/other-accounts.tsx` | OtherAccountsListPage:其他账户列表子页 |
| `citizenchain/node/frontend/transaction/offchain-transaction/settlement/admin-unlock.tsx` | ClearingBankAdminListPage:管理员列表/解锁入口 |
| `citizenchain/node/frontend/governance/organization_manage/create-multisig.tsx` | CreateMultisigInstitutionPage:创建机构多签流程 |
| `citizenchain/node/frontend/transaction/offchain-transaction/offchain-transaction/node-register.tsx` | ClearingBankDeclareNodePage(声明本机为清算行节点)|
| `citizenchain/node/frontend/governance/organization_manage/api.ts` / `types.ts` | 机构多签 Tauri invoke / 类型 |
| `citizenchain/node/frontend/transaction/offchain-transaction/api.ts` / `types.ts` / `styles.css` | 清算行节点声明、解锁、连通性 invoke / 类型 / 样式 |

### 2026-05-02 删除清单

- `detail.tsx`(老 ClearingBankDetailPage):节点信息合入 `institution_detail.tsx` 内联卡片
- `admin.tsx`(老 ClearingBankAdminListPage):被 `settlement/admin-unlock.tsx` 取代
- `node.tsx`(老 ClearingBankNodeInfoPanel):合入 `institution_detail.tsx` 内联展示
- 根层 `cid.tsx` / `institution_detail.tsx` / `other_accounts.tsx` / `admin_list.tsx` / `create_multisig.tsx` / `register.tsx`:迁入业务子目录后删除,不再保留双份文件。

## 3. Tauri 命令目录

Tauri 命令按业务拆分:

| 目录 | 命令 | 用途 |
|---|---|
| `transaction/offchain/institution_read/commands.rs` | `search_eligible_clearing_banks` | 搜索清算行候选 |
| `transaction/offchain/institution_read/commands.rs` | `fetch_clearing_bank_institution_detail` | 链上按 finalized hash 查 `Institutions[cid_number]` + `InstitutionAccounts[cid_number, *]` + 各账户余额。`None` = 未创建 |
| `transaction/offchain/institution_read/commands.rs` | `fetch_clearing_bank_institution_proposals` | 机构提案分页(占位:目前返回空列表,full scan 留 follow-up) |
| `transaction/offchain/commands.rs` | `query_clearing_bank_node_info` / `query_local_peer_id` / `test_clearing_bank_endpoint_connectivity` | 清算行节点声明和端点自测 |
| `transaction/offchain/commands.rs` | `build_register_*` / `submit_register_*` / `build_update_*` / `submit_update_*` / `build_unregister_*` | 清算行节点注册、端点更新、注销 |
| `transaction/offchain/settlement/commands.rs` | `build_decrypt_admin_request` / `verify_and_decrypt_admin` / `list_decrypted_admins` / `lock_decrypted_admin` | 结算前管理员解锁 |

DTO 统一见 `transaction/offchain/institution_read/types.rs` 与 `transaction/offchain/types.rs`。

## 4. 机构创建入口状态

PublicManage/PrivateManage 的旧 call index 5 已永久关闭，Node 不再构造或提交旧机构直接创建载荷，`0x1e05/0x1f05` 也不再是合法 QR 动作。任务卡第 6 步将由独立机构创建业务模块原子覆盖 admins、LR、初始治理岗位、不可变权限、初始任职和投票规则，并按注册局有效 `RoleSubject` 授权；新载荷必须另行登记并同步全部生成端和解码端。

所有机构管理员记录统一为 `account_id + cid_number + family_name + given_name`。账户用于人员识别和签名，但账户本身没有机构业务权限；非空公民 CID 只引用 `citizen-identity` 真源。

`subject_property / sub_type / parent_cid_number` 只属于 `eligible-search` 查询筛选和展示,不得进入任何 call_data。

## 5. 节点声明整体时序

机构创建与机构治理归 OnChina 控制台，节点不参与；本 tab 只从机构已存在处接手。

```
[OnChina 控制台] 机构创建/治理提案 → 内部投票通过 → Institutions[cid_number] = Active
          │
[节点桌面] ① 选 candidate (cid_number) → fetch_clearing_bank_institution_detail
          │
          ▼ ② Institutions[cid_number] = None → 提示去 OnChina 控制台创建,节点流程终止
          │      Institutions[cid_number] = Active → 进 declare-node
          │
          ▼ ③ 填本机 RPC 端点 → test_clearing_bank_endpoint_connectivity 自测
          │
          ▼ ④ build_register_* → 冷钱包扫 sign_request 两段握手 → response
          │
          ▼ ⑤ submit_register_*(extrinsic 上链)
          │
[chain runtime] ⑥ 在 origin 处按机构 CID、岗位码和管理员账户 ID 三者鉴权
                  - 通过 → ClearingBankNodes[cid_number] 写入本机端点
                  - 失败 → DispatchError,extrinsic 回滚
[节点桌面] ⑦ 端点更新走 build_update_* / submit_update_*,注销走 build_unregister_*
```

## 6. follow-up 任务卡

下面几项在本任务卡范围之外,后续单独开卡:

| # | 项 | 触发条件 |
|---|---|---|
| F1 | 机构提案列表 full scan(`fetch_institution_proposals`)| votingengine 提案存储扫描 + institution_hex 过滤 |
| F2 | 节点桌面"扫码添加管理员"接 CitizenWallet user_contact / user_multisig QR | 当前 create-multisig 用粘贴兜底 |
| F3 | 创建机构 extrinsic 提交后冷钱包两段握手实际接入(`VoteSigningFlow` 复用) | 当前 alert 占位 |
| F4 | CitizenWallet decoder 加新版 `propose_create_institution` action 分支 | 已按 11 字段新布局同步，后续字段变更仍需三端同时更新 |
| F5 | 发起提案按钮组的具体业务类型（转账 / 关闭机构 / 手续费划转） | 按对应业务模块接口逐项接入；岗位任职变更不在此处直接实现 |
| F6 | 节点端"管理员激活"机制(冷钱包列表的来源)集成到 create-multisig 选签名钱包 | 当前 coldWallets={[]} 占位 |

## 7. 验收标准达成情况

- ✅ `cargo check -p organization-manage --tests` 通过
- ✅ `cargo check -p multisig-transfer --tests` 通过
- ✅ `cargo check -p offchain-transaction --tests` 通过
- ✅ `cargo check -p node` 带 `WASM_FILE=target/ci-wasm/citizenchain.compact.compressed.wasm` 通过(仅既有 unsafe/dead_code 警告)
- ✅ `npm run build`(node frontend) 通过
- ✅ CID `registration-info` 返回 `cid_number / cid_full_name / account_names[] / credential`
- ✅ 节点桌面状态机重构,删除 register-cid / propose-create info 终态 + 老 detail.tsx + admin.tsx + node.tsx
- ✅ cid.tsx 删"查询"按钮,改 debounce 自动搜
- ✅ 4 个新页面:institution_detail / create_multisig / other_accounts / admin_list
- ⏳ 端到端冷钱包签 + 上链 + 等投票 + 声明节点(完整跑通需 CitizenWallet decoder follow-up)

## 8. 变更记录

- 2026-05-01:首次落地。节点 Rust 加 4 个 Tauri 命令 + 5 个 chain/onchina/signing helper;节点前端新建 4 页 + 状态机重构 + 删 3 个老文件。
- 2026-05-02:对齐 CID `registration-info`。创建机构多签注册 payload 收口为 `cid_number / cid_full_name / account_names[]`,移除 `subject_property/sub_type/parent_cid_number` 注册透传,补齐 `signer_pubkey`。
