# 节点桌面清算行 tab 技术说明

- 日期: 2026-05-02
- 任务卡:
  - `memory/08-tasks/done/20260501-node-clearing-bank-institution-detail-and-create.md`
  - `memory/08-tasks/done/20260502-node-offchain-duplicate-cleanup.md`
  - `memory/08-tasks/done/20260502-duoqian-registration-info-align.md`
- 承接: `20260501-cid-chain-folder-restructure.md`(CID 端 chain/ 目录重构)

## 0. 概览

节点桌面"清算行"tab 提供 3 类核心能力:

1. **添加清算行**:输入 CID → 链上判定多签是否存在 → 已存在显示详情 / 不存在进创建流程
2. **机构详情**:展示链上 `organization-manage::Institutions[cid_number]` 全部信息 + 折叠卡片入口(其他账户/管理员)+ 节点声明状态 + 提案列表
3. **创建机构多签**:拉 CID `registration-info` → 按 CID 返回的账户名称配置初始资金 + 管理员 + 阈值 → 冷钱包签 `propose_create_institution` extrinsic → 等其他管理员投票通过 → 进节点声明流程

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
| `citizenchain/node/frontend/offchain/section.tsx` | 状态机驱动;EmptyView / CheckMultisigView / WaitVoteView 子组件 |
| `citizenchain/node/frontend/governance/organization-manage/add-candidate.tsx` | ClearingBankAddPage:debounce 自动搜 CID 候选(2026-05-01 删"查询"按钮) |
| `citizenchain/node/frontend/governance/organization-manage/institution-detail.tsx` | ClearingBankInstitutionDetailPage:卡片栅格 + 折叠子页入口 + 节点信息 + 发起提案占位 + 提案列表 |
| `citizenchain/node/frontend/governance/organization-manage/other-accounts.tsx` | OtherAccountsListPage:其他账户列表子页 |
| `citizenchain/node/frontend/offchain/settlement/admin-unlock.tsx` | ClearingBankAdminListPage:管理员列表/解锁入口 |
| `citizenchain/node/frontend/governance/organization-manage/create-multisig.tsx` | CreateMultisigInstitutionPage:创建机构多签流程 |
| `citizenchain/node/frontend/offchain/offchain-transaction/node-register.tsx` | ClearingBankDeclareNodePage(声明本机为清算行节点)|
| `citizenchain/node/frontend/governance/organization-manage/api.ts` / `types.ts` | 机构多签 Tauri invoke / 类型 |
| `citizenchain/node/frontend/offchain/api.ts` / `types.ts` / `styles.css` | 清算行节点声明、解锁、连通性 invoke / 类型 / 样式 |

### 2026-05-02 删除清单

- `detail.tsx`(老 ClearingBankDetailPage):节点信息合入 `institution_detail.tsx` 内联卡片
- `admin.tsx`(老 ClearingBankAdminListPage):被 `settlement/admin-unlock.tsx` 取代
- `node.tsx`(老 ClearingBankNodeInfoPanel):合入 `institution_detail.tsx` 内联展示
- 根层 `cid.tsx` / `institution_detail.tsx` / `other_accounts.tsx` / `admin_list.tsx` / `create_multisig.tsx` / `register.tsx`:迁入业务子目录后删除,不再保留双份文件。

## 3. Tauri 命令目录

Tauri 命令按业务拆分:

| 目录 | 命令 | 用途 |
|---|---|
| `governance/organization-manage/commands.rs` | `search_eligible_clearing_banks` | 搜索清算行候选 |
| `governance/organization-manage/commands.rs` | `fetch_clearing_bank_institution_detail` | 链上按 finalized hash 查 `Institutions[cid_number]` + `InstitutionAccounts[cid_number, *]` + 各账户余额。`None` = 未创建,前端进 create 流程 |
| `governance/organization-manage/commands.rs` | `fetch_clearing_bank_institution_proposals` | 机构提案分页(占位:目前返回空列表,full scan 留 follow-up) |
| `governance/organization-manage/commands.rs` | `fetch_clearing_bank_institution_registration_info` | 调 CID `GET /api/v1/app/institutions/:cid_number/registration-info` 拉链上注册专用信息 |
| `governance/organization-manage/commands.rs` | `build_propose_create_institution_request` / `submit_propose_create_institution` | 公民钱包签名并提交 `propose_create_institution` |
| `offchain/offchain_transaction/commands.rs` | `query_clearing_bank_node_info` / `query_local_peer_id` / `test_clearing_bank_endpoint_connectivity` | 清算行节点声明和端点自测 |
| `offchain/offchain_transaction/commands.rs` | `build_register_*` / `submit_register_*` / `build_update_*` / `submit_update_*` / `build_unregister_*` / `submit_unregister_*` | 清算行节点注册、端点更新、注销 |
| `offchain/settlement/commands.rs` | `build_decrypt_admin_request` / `verify_and_decrypt_admin` / `list_decrypted_admins` / `lock_decrypted_admin` | 结算前管理员解锁 |

DTO 统一见 `offchain/common/types.rs`。

## 4. propose_create_institution(call_index 5)字节布局

链端 [`organization-manage::propose_create_institution`](citizenchain/runtime/governance/organization-manage/src/lib.rs) 11 入参:

```
[pallet_index=17][call_index=5]
cid_number: BoundedVec<u8>            = Compact(len) || bytes
cid_full_name: BoundedVec<u8>   = Compact(len) || bytes
accounts: BoundedVec<InstitutionInitialAccount>
                                    = Compact(N) || N × (account_name_compact || amount_u128_le)
org: u8                       = ORG_PUP(4) 或 ORG_OTH(5)
admins_len: u32                    = u32 LE
admins: BoundedVec<AccountId32>
                                    = Compact(N) || N × 32B
threshold: u32                      = u32 LE
register_nonce: BoundedVec<u8>      = Compact(len) || bytes
signature: BoundedVec<u8>(64)       = Compact(64) || 64B
province_name: Vec<u8>                   = Compact(len) || bytes
signer_pubkey: [u8; 32]       = 32B 原始公钥
```

**任何字段顺序变更必须同步改 `governance/organization-manage/signing.rs::build_propose_create_institution_call_data`**,否则公民钱包签名 payload 与链上 call_data 不一致。

注册业务字段只允许来自 CID `registration-info` 的 `cid_number / cid_full_name / account_names[]`。
`subject_property / sub_type / parent_cid_number` 只属于 `eligible-search` 查询筛选和展示,不得进入注册 call_data。

## 5. 创建机构整体时序

```
[节点桌面] ① 选 candidate (cid_number) → check-multisig
          │
          ▼ ② Institutions[cid_number] = None
          │
          ▼ ③ fetch_clearing_bank_institution_registration_info(cid_number)
[CID 后端] ──→ ④ app_get_institution_registration_info 内部:
                  - 读机构数据(sharded_store)
                  - 取签发机构主账户和当前签名管理员公钥
                  - 生成 register_nonce = uuid_v4 字符串
                  - signature = IssuerAdminSigner.sign(blake2_256(scale_encode(
                        DUOQIAN ++ OP_SIGN_INST ++ genesis_hash
                        ++ cid_number ++ cid_full_name ++ account_names[]
                        ++ register_nonce
                        ++ issuer_cid_number ++ issuer_main_account ++ signer_pubkey
                        ++ scope_province_name ++ scope_city_name
                    )))
                  - 响应:cid_number + cid_full_name + account_names[] + credential
[节点桌面] ⑤ 用户按 account_names[] 填账户初始资金 + 扫码加管理员 + 设阈值 + 选冷钱包
          │
          ▼ ⑥ build_propose_create_institution_request(全部字段)
          │
          ▼ ⑦ 冷钱包扫 sign_request 两段握手 → response
          │
          ▼ ⑧ submit_propose_create_institution(extrinsic 上链)
          │
[chain runtime] ⑨ propose_create_institution:
                  - UsedRegisterNonce[hash(nonce)] 必须 false
                  - 读取 admins-change::AdminAccounts[issuer_main_account],确认 signer_pubkey 属于该机构 admins
                  - 重算 payload hash + sr25519_verify(signature, hash, pubkey)
                  - 通过 → Institutions[cid_number] = Pending,创建投票提案
                  - 失败 → DispatchError,extrinsic 回滚
[节点桌面] ⑩ wait-vote 视图轮询 fetchInstitutionDetail(cid_number).status
[其他管理员] ⑪ citizenwallet 公民钱包扫 vote 提案 → 投赞成
          │
          ▼ ⑫ 票数达 threshold → InternalVoteExecutor 自动执行 → status = Active
          │
[节点桌面] ⑬ 轮询发现 Active → 自动跳 declare-node
          │
          ▼ ⑭ 填本机 RPC 端点 + 自测 + 签名声明 → 链上 ClearingBankNodes[cid_number]
```

## 6. follow-up 任务卡

下面几项在本任务卡范围之外,后续单独开卡:

| # | 项 | 触发条件 |
|---|---|---|
| F1 | 机构提案列表 full scan(`fetch_institution_proposals`)| votingengine 提案存储扫描 + institution_hex 过滤 |
| F2 | 节点桌面"扫码添加管理员"接 citizenwallet user_contact / user_duoqian QR | 当前 create-multisig 用粘贴兜底 |
| F3 | 创建机构 extrinsic 提交后冷钱包两段握手实际接入(`VoteSigningFlow` 复用) | 当前 alert 占位 |
| F4 | citizenwallet decoder 加新版 `propose_create_institution` action 分支 | 已按 11 字段新布局同步，后续字段变更仍需三端同时更新 |
| F5 | 发起提案按钮组的具体提案类型(转账 / 关闭多签 / 换管理员 / 手续费划转) | 当前全部 disabled "即将上线" |
| F6 | 节点端"管理员激活"机制(冷钱包列表的来源)集成到 create-multisig 选签名钱包 | 当前 coldWallets={[]} 占位 |

## 7. 验收标准达成情况

- ✅ `cargo check -p organization-manage --tests` 通过
- ✅ `cargo check -p duoqian-transfer --tests` 通过
- ✅ `cargo check -p offchain-transaction --tests` 通过
- ✅ `cargo check -p node` 带 `WASM_FILE=target/ci-wasm/citizenchain.compact.compressed.wasm` 通过(仅既有 unsafe/dead_code 警告)
- ✅ `npm run build`(node frontend) 通过
- ✅ CID `registration-info` 返回 `cid_number / cid_full_name / account_names[] / credential`
- ✅ 节点桌面状态机重构,删除 register-cid / propose-create info 终态 + 老 detail.tsx + admin.tsx + node.tsx
- ✅ cid.tsx 删"查询"按钮,改 debounce 自动搜
- ✅ 4 个新页面:institution_detail / create_multisig / other_accounts / admin_list
- ⏳ 端到端冷钱包签 + 上链 + 等投票 + 声明节点(完整跑通需 citizenwallet decoder follow-up)

## 8. 变更记录

- 2026-05-01:首次落地。节点 Rust 加 4 个 Tauri 命令 + 5 个 chain/citizencode/signing helper;节点前端新建 4 页 + 状态机重构 + 删 3 个老文件。
- 2026-05-02:对齐 CID `registration-info`。创建机构多签注册 payload 收口为 `cid_number / cid_full_name / account_names[]`,移除 `subject_property/sub_type/parent_cid_number` 注册透传,补齐 `signer_pubkey`。
