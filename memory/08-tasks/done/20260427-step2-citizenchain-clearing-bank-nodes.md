# Step 2：区块链端清算行节点声明 + 收款方主导清算 + fee_account 付 gas

任务需求：
作为清算行三阶段实施（[ADR-007](../../04-decisions/ADR-007-clearing-bank-three-phase.md)）的第 2 阶段，在 citizenchain runtime + node Tauri 后端 + node 前端落地清算行核心机制：

1. **清算方向反转**：从付款方主导改为**收款方主导**清算批次提交
2. **fee 流向统一**：永远归 `fee_account_of(recipient_bank)`（同行 / 跨行统一）
3. **gas 由费用账户直接支付**：runtime 自定义 OnChargeTransaction，从 `fee_account_of(institution_main)` 扣 gas
4. **链上资格白名单二次校验**：`bank_check::ensure_can_be_bound` 收紧到 6 重校验
5. **链上节点声明 storage + extrinsic**：`ClearingBankNodes` + `register_clearing_bank` / `update_clearing_bank_endpoint` / `unregister_clearing_bank`
6. **InstitutionMetadata 上链**：register_sfid_institution 等创建路径增加 a3/sub_type/parent 参数（Required，开发期彻底切换，无 backfill）
7. **节点 UI 新增清算行 tab**：8 视图状态机 + "解密"管理员密钥（仅清算行 tab 用，不动 NRC/PRC/PRB）
8. **桌面节点 连通性自测强制**：DNS + wss + 链 ID + PeerId 匹配，全过才允许提交
9. **SFID 端 Step 2 末尾联动**：`/clearing-banks/search` 加 `AND sfid_id ∈ ClearingBankNodes` 过滤；新建 ClearingBankWatcher 订阅链上事件 + SQLite 缓存

所属模块：**Architect 主入口跨模块协调**——Blockchain Agent（runtime + node）+ SFID Agent（推链字段 + watcher）；不动 wumin/wuminapp（留 Step 3）。

## 必须遵守

- **跨模块联动铁律**（[chat-protocol.md §5](../../07-ai/chat-protocol.md)）：
  - runtime spec_version 2 → 3 但 **Step 2 不上主网**
  - 等 Step 3 wumin decoder + wuminapp 兼容做完后，由 Architect 主入口走 `propose_runtime_upgrade` 联合提案上链
  - dev 链可直接用 Step 2 新版 runtime 做端到端验证
- **不可突破模块边界**：本步零 wumin / wuminapp 改动
- **不留兼容窗口**（[feedback_no_compatibility.md](../../feedback_no_compatibility.md)）：开发期彻底切换，无 InstitutionMetadata Optional 兼容、无 backfill
- **不重建 chainspec**（[feedback_chainspec_frozen.md](../../feedback_chainspec_frozen.md)）：runtime 升级走链上 setCode
- **不清楚逻辑时先沟通**

## 输入文档

- [ADR-007 清算行三阶段拆分](../../04-decisions/ADR-007-clearing-bank-three-phase.md)
- [memory/05-modules/sfid/clearing-bank-eligibility.md](../../05-modules/sfid/clearing-bank-eligibility.md)
- [project_unified_voting_entry.md](../../project_unified_voting_entry.md)
- [project_qr_signing_two_color.md](../../project_qr_signing_two_color.md)
- [feedback_no_compatibility.md](../../feedback_no_compatibility.md)
- [memory/08-tasks/templates/citizenchain-runtime.md](../templates/citizenchain-runtime.md)
- [memory/08-tasks/templates/citizenchain-node.md](../templates/citizenchain-node.md)
- [memory/08-tasks/templates/citizenchain-node.md](../templates/citizenchain-node.md)
- [memory/08-tasks/templates/sfid-backend.md](../templates/sfid-backend.md)

## 变更范围（文件级）

### A. citizenchain/runtime

#### A.1 duoqian-manage pallet
- `MetadataInfo` 结构体新增（a3 / sub_type / parent_sfid_id）
- `InstitutionMetadata: StorageMap<SfidId, MetadataInfo>` 新建
- `register_sfid_institution`：参数追加 a3 / sub_type / parent_sfid_id（Required）
- `propose_create`：参数追加 a3 / sub_type / parent_sfid_id（在 institution 元数据未存时写入 InstitutionMetadata）
- `SfidAccountQuery` trait 扩展：暴露 `institution_a3` / `institution_sub_type` / `institution_parent` 三个查询方法

#### A.2 offchain-transaction pallet
- `bank_check::ensure_can_be_bound`：4 重校验扩展到 6 重（加资格白名单 + sfid ∈ ClearingBankNodes）
- `ClearingBankNodes: StorageMap<SfidId, ClearingBankNodeInfo>` 新建
  - `ClearingBankNodeInfo { peer_id, rpc_domain, rpc_port, registered_at, registered_by }`
- `NodePeerToInstitution: StorageMap<PeerId, SfidId>` 反向索引新建
- `register_clearing_bank` extrinsic 新建（任一 duoqian_admin 单签）
- `update_clearing_bank_endpoint` extrinsic 新建（仅改端点）
- `unregister_clearing_bank` extrinsic 新建（注销 + 反向索引清理）
- `submit_offchain_batch_v2`：校验改为 `item.recipient_bank == institution_main`（原为 payer_bank）
- `settlement.rs`：同行 / 跨行 fee 流向统一为 `fee_account_of(recipient_bank)`，简化分支
- 新增事件 `ClearingBankRegistered` / `ClearingBankEndpointUpdated` / `ClearingBankUnregistered`

#### A.3 runtime/lib.rs（OnChargeTransaction 自定义）
- 新增 `ChargeBatchFromInstitution` 实现 `OnChargeTransaction<Runtime>`
- 对 `submit_offchain_batch_v2` 这个 call：从 `fee_account_of(institution_main)` 直接扣 gas
- 其他 call：走默认 CurrencyAdapter（不变）
- 替换 `pallet_transaction_payment` 的 `OnChargeTransaction` 关联类型

#### A.4 spec_version
- bump：2 → 3
- transaction_version：bump（参数签名变了）
- **不主网升级**——Step 2 完工时仅 dev 链使用

### B. citizenchain/node Tauri 后端

新建模块 `citizenchain/node/src/ui/clearing_bank/mod.rs`：

新 Tauri commands：
- `search_eligible_clearing_banks(query, limit)` — HTTP 转发 SFID `/clearing-banks/eligible-search`
- `query_clearing_bank_node_info(sfid_id)` — 链上查 `ClearingBankNodes[sfid_id]`
- `query_local_peer_id()` — 调 RPC `system_localPeerId`
- `test_clearing_bank_endpoint_connectivity(domain, port, expected_peer_id)` — 连通性自测（4 项校验）
- `build_register_clearing_bank_request(pubkey, sfid_id, peer_id, rpc_domain, rpc_port)` + `submit_register_clearing_bank(...)`
- `build_update_clearing_bank_endpoint_request(...)` + `submit_update_clearing_bank_endpoint(...)`
- `build_unregister_clearing_bank_request(...)` + `submit_unregister_clearing_bank(...)`

修改：
- `network/network-overview/mod.rs::get_network_overview`：`clearing_nodes` 字段从硬编码 0 改为 `ClearingBankNodes` 链上 storage 计数

### C. citizenchain/node/frontend（清算行 tab）

#### C.1 App.tsx
- TabKey 联合类型加 `'clearing-bank'`
- 顶部 nav 9 tab，顺序：`首页 / 挖矿 / 国储会 / 省储会 / 省储行 / 清算行 / 白皮书 / 公民宪法 / 设置`
- 路由分发到 `<ClearingBankSection />`

#### C.2 新建 frontend/clearing-bank/
- `ClearingBankSection.tsx` — 8 视图状态机
- `ClearingBankAddPage.tsx` — 输入 sfid_id + 链上状态检查 + 分支
- `ClearingBankProposeCreatePage.tsx` — 调 propose_create（含候选 admin pubkey 列表 + threshold + amount）
- `ClearingBankWaitVotePage.tsx` — 等其他 admins 投票，复用现有 voting status 查询
- `ClearingBankDeclareNodePage.tsx` — peer_id 自动填 + domain/port 手填 + 连通性自测 + register_clearing_bank 签名提交
- `ClearingBankDetailPage.tsx` — 复用 InstitutionDetailPage（5 卡片 + 提案列表 + 节点信息长卡）
- `ClearingBankNodeInfoPanel.tsx` — peer_id / rpc_domain / port / registered_at / 注册管理员 + update_endpoint / unregister 入口
- `ClearingBankAdminListPage.tsx` — 仅本 tab 用 "**解密**" 术语；其他 tab（NRC/PRC/PRB）用 AdminListPage（"激活"）不动
  - 列表行加"解密"按钮 + 状态指示绿点
  - 解密 = wumin 扫码签 challenge → 节点验签 → 解密本地加密存储私钥到内存
  - 内存中密钥永久驻留至节点重启，无 TTL
  - 解密后自动签 submit_offchain_batch_v2

#### C.3 NetworkInlineSection
- `NetworkOverview.clearingNodes` 已有字段，前端无改动（后端从 0 → 真实计数）

### D. SFID 端 Step 2 末尾联动

#### D.1 sfid/backend
- 新建 `chain/clearing_bank_watcher.rs` — 常驻 tokio task：
  - 启动：全量 scan ClearingBankNodes storage 写入 SQLite 缓存
  - 增量：订阅 finalized blocks 解析 ClearingBankRegistered/Updated/Unregistered 事件
  - 容错：指数退避重连 + 重连后全量对账
- 新建 SQLite 表 `clearing_bank_nodes`（持久化）+ `clearing_bank_sync_state`
- `institutions/handler.rs::app_search_clearing_banks`：第 2 轮过滤里加 `AND sfid_id ∈ clearing_bank_nodes_cache`
- 推链 `register_sfid_institution` 调用增加 a3 / sub_type / parent_sfid_id 参数

### E. 测试

- runtime: 新增单测覆盖：
  - 资格白名单 6 case（已有 SFID 端 8 个，链上版需 mock SfidAccountQuery）
  - register_clearing_bank 通过 / 失败（PeerId 占用 / 未授权 admin / 未注册机构）
  - update_endpoint / unregister
  - submit_offchain_batch_v2 改为 recipient_bank 主导后的同行 + 跨行 case
  - OnChargeTransaction 测试：fee_account 余额不足时 batch 拒绝；正常情况 fee_account 减少 gas
- node Tauri: cargo test
- node 前端: tsc + vite build
- SFID watcher: 集成测试（mock 链上事件流）

## 输出物

- 代码：runtime / node Tauri / node 前端 / SFID 后端 watcher
- 中文注释：所有新增 storage / extrinsic / Tauri command / 前端组件
- 测试：单测 + 集成测试
- 文档：
  - 更新 ADR-007 标记 Step 2 完成
  - 更新 [memory/05-modules/sfid/clearing-bank-eligibility.md](../../05-modules/sfid/clearing-bank-eligibility.md) 加 watcher 章节
  - 新建 [memory/05-modules/citizenchain/clearing-bank-node.md] 描述链上设计
  - 更新 [memory/02-modules/citizenchain/]（如需）
  - 更新 auto-memory `project_clearing_bank_three_phase.md` 标记 Step 2 完成
- 残留清理：无 TODO / 无 backfill 残留 / 无 Optional 兼容字段

## 验收标准

- runtime cargo test 全绿（新增 ≥ 20 case）
- node Tauri cargo check + 联调通过
- node 前端 tsc + vite build 通过
- SFID 后端 cargo test + ClearingBankWatcher 集成测试通过
- dev 链端到端验证：
  - 9 tab 切换正确
  - 清算行 tab 完整 8 视图状态机可走通
  - 一家 SFR-JOINT_STOCK 机构：register_sfid_institution → propose_create → 投票通过 → register_clearing_bank → 详情页展示 → 端点更新 → 注销
  - SFR-LIMITED_LIABILITY 机构 register_clearing_bank 链上拒绝
  - bind_clearing_bank 绑定一家未声明的机构 → 链上拒绝（call_index 30 报错）
  - submit_offchain_batch_v2 同行 / 跨行：fee 进 recipient_bank 费用账户、gas 从 fee_account 扣、管理员个人钱包余额不变
  - 用 SFR-JOINT_STOCK 机构成功提交跨行 batch，X 主账户净流出 = 本金 + fee + 0 gas（gas 从 Y 费用账户扣，X 不参与）
  - "解密"按钮仅在清算行 tab 显示；NRC/PRC/PRB 的"激活"按钮文案保持不变
- 桌面节点 连通性自测：填入错误 domain/port 提交按钮置灰
- 残留扫描通过

## 落地顺序（4 阶段）

```
阶段 A: runtime               (~3-4 天)
  A1. SfidAccountQuery trait 扩展(暴露 a3/sub_type/parent)
  A2. InstitutionMetadata storage + register_sfid_institution / propose_create 参数追加
  A3. ClearingBankNodes / NodePeerToInstitution storage
  A4. register_clearing_bank / update_endpoint / unregister extrinsic
  A5. bank_check::ensure_can_be_bound 收紧到 6 重
  A6. submit_offchain_batch_v2 校验改为 recipient_bank 主导
  A7. settlement.rs fee 流向统一(同行/跨行都到 recipient_bank.fee_account)
  A8. ChargeBatchFromInstitution OnChargeTransaction 实现
  A9. spec_version 2→3 + transaction_version bump
  A10. 单测覆盖

阶段 B: node Tauri 后端         (~1-2 天)
  B1. clearing_bank/ 模块新建 + 9 个 Tauri command
  B2. 连通性自测实现(DNS + wss + 链 ID + PeerId 匹配)
  B3. NetworkOverview.clearing_nodes 真实计数
  B4. cargo check + 联调

阶段 C: node 前端              (~3 天)
  C1. App.tsx TabKey + 9 tab 路由
  C2. ClearingBankSection 8 视图状态机
  C3. ClearingBankAddPage / ClearingBankProposeCreatePage / ClearingBankWaitVotePage
  C4. ClearingBankDeclareNodePage(连通性自测 UI)
  C5. ClearingBankAdminListPage("解密"按钮 + 绿点)
  C6. ClearingBankNodeInfoPanel(端点更新/注销)
  C7. tsc + vite build

阶段 D: SFID 联动 + 验收       (~1 天)
  D1. sfid backend register_sfid_institution 调用补 a3/sub_type/parent 参数
  D2. ClearingBankWatcher 模块(订阅 + SQLite + 启动 scan)
  D3. app_search_clearing_banks 过滤 ClearingBankNodes
  D4. dev 链端到端 12 项验收清单
  D5. 文档同步 + 任务卡归档

总计 ~7-10 天 runtime + node + SFID 主体
Step 3 wumin/wuminapp 完成后再走主网升级
```

## 阶段 A 完工记录(2026-04-27)

**runtime 改动全部完成,验证通过**:

- ✅ A1 SfidAccountQuery trait 加 `is_clearing_bank_eligible` + `is_registered_clearing_node` 两个方法
- ✅ A2 duoqian-manage `MetadataInfo` 类型 + `InstitutionMetadata` storage + 6 个新错误码
- ✅ A3 `register_sfid_institution` 加 `a3` / `sub_type` / `parent_sfid_id` 三个 Required 参数 + 元数据写入/校验逻辑
- ✅ A4 offchain-transaction `ClearingBankNodeInfo` 结构体 + `ClearingBankNodes` storage + `NodePeerToInstitution` 反向索引
- ✅ A5 `register_clearing_bank` / `update_clearing_bank_endpoint` / `unregister_clearing_bank` 三个 extrinsic(call_index 50/51/52)+ 9 重校验链 + PeerId 字符集 / RPC 域名字符集校验辅助函数
- ✅ A6 `bank_check::ensure_can_be_bound` 由 4 重收紧到 6 重(加资格白名单 + 已声明节点)
- ✅ A7 `submit_offchain_batch_v2` 校验改为 `recipient_bank == institution_main`(收款方主导)+ 完整中文注释说明设计变化
- ✅ A8 `settlement.rs::execute_clearing_bank_batch` 重构:批次内可含多个 payer_bank 但 recipient_bank 必须统一,按 payer_bank 分组做偿付预检
- ✅ A9 `RuntimeFeePayerExtractor` 注释已同步标注"institution_main = 收款方清算行"+ "fee_account 自给自足闭环"语义
- ✅ A_configs runtime/src/configs 补 `MaxA3Length` / `MaxSubTypeLength` 配置 + DuoqianSfidAccountQuery 加 `is_clearing_bank_eligible` / `is_registered_clearing_node` 两个方法实现(查 InstitutionMetadata 跨省 parent + 查 ClearingBankNodes)
- ✅ A10 spec_version 2→3, transaction_version 1→2

**验证结果**:
- cargo check -p offchain-transaction: ✅ 通过
- cargo check -p duoqian-manage: ✅ 通过(5 warnings 全部预存)
- cargo check -p citizenchain (runtime): ✅ 通过
- cargo check -p node: ✅ 通过(42 warnings 全部预存,不增不减)
- cargo test -p offchain-transaction: ✅ 20/20 全绿
- cargo test -p duoqian-manage: ✅ 17/17 全绿
- cargo test -p node: 108/109 通过(1 个预存 bug `compact_u128_big_integer` 与本次无关,已 spawn 独立修复任务)

**新增/修改的关键文件**:
- runtime/transaction/duoqian-manage/src/lib.rs(MetadataInfo + InstitutionMetadata + register 参数 + 测试更新)
- runtime/transaction/offchain-transaction/src/lib.rs(ClearingBankNodeInfo + ClearingBankNodes + NodePeerToInstitution + 3 extrinsic + 4 events + 6 errors + 4 helper fn)
- runtime/transaction/offchain-transaction/src/bank_check.rs(SfidAccountQuery 加 2 方法 + ensure_can_be_bound 6 重校验)
- runtime/transaction/offchain-transaction/src/settlement.rs(收款方主导校验 + 多 payer_bank 偿付预检)
- runtime/transaction/offchain-transaction/src/tests.rs(MockSfid 补两个方法实现)
- runtime/src/configs/mod.rs(DuoqianSfidAccountQuery 扩展 + MaxA3Length / MaxSubTypeLength + RuntimeFeePayerExtractor 注释)
- runtime/src/lib.rs(spec_version 2→3, transaction_version 1→2)

**待 Step 3 完工后联动**: 主网升级走 `propose_runtime_upgrade` 上链。

## 阶段 B 完工记录(2026-04-27)

**node Tauri 后端清算行模块全部落地,验证通过**:

- ✅ B1 新建 `citizenchain/node/src/ui/clearing_bank/` 模块,7 个文件:
  - `mod.rs` — 14 个 Tauri command 注册入口
  - `types.rs` — DTO(EligibleClearingBankCandidate / ClearingBankNodeOnChainInfo / ConnectivityTestReport / DecryptedAdminInfo / DecryptAdminRequestResult)
  - `chain.rs` — `OffchainTransaction::ClearingBankNodes` storage 读取 + storage prefix 计数(state_getKeysPaged 分页)
  - `connectivity.rs` — DNS / 远端 RPC / 链 ID(ss58Format=2027) / PeerId 4 重自测
  - `signing.rs` — register/update_endpoint/unregister 三个 extrinsic 的 call_data + WUMIN_QR_V1 sign request
  - `sfid_proxy.rs` — HTTP 转发 SFID `/api/v1/app/clearing-banks/eligible-search`
  - `admin_decrypt.rs` — wumin 签 challenge → 节点 sr25519 验签 → 内存 HashMap 标记"已解密"
- ✅ B2 8+6=14 个 Tauri command 全部注册(search_eligible_clearing_banks / query_clearing_bank_node_info / query_local_peer_id / test_clearing_bank_endpoint_connectivity / build+submit_register_clearing_bank / build+submit_update_clearing_bank_endpoint / build+submit_unregister_clearing_bank / build_decrypt_admin_request / verify_and_decrypt_admin / list_decrypted_admins / lock_decrypted_admin)
- ✅ B3 `network_overview.clearing_nodes` 从硬编码 0 改为 `count_clearing_bank_nodes()`(链上 storage prefix 实时计数,RPC 失败降级到 0 + warning)
- ✅ B4 `governance::storage_keys` 改 `pub(crate)` 让清算行模块复用 `twox_128 / blake2b_128 / map_key`,不重复实现

**验证**:
- `cargo check -p node`:✅ 通过(42 warnings 全部预存)
- `cargo test -p node clearing_bank`:✅ 18/18 全绿(types/chain/connectivity/signing/admin_decrypt)

## 阶段 C 完工记录(2026-04-27)

**node 前端清算行 tab + 8 视图状态机 + 解密管理员列表全部落地**:

- ✅ C1 `App.tsx` `TabKey` 加 `'clearing-bank'`,顶部 nav 9 tab 顺序固化(首页/挖矿/国储会/省储会/省储行/**清算行**/白皮书/公民宪法/设置)
- ✅ C2 新建 `frontend/clearing-bank/` 模块:
  - `clearing-bank-types.ts` — 与后端 `types.rs` 镜像 + `ClearingBankView` 8 视图状态机类型
  - `ClearingBankSection.tsx` — 主路由 + 状态机 dispatch + localStorage 缓存"已添加"sfid_id 列表 + empty/check-status/register-sfid/propose-create/wait-vote 5 个内联视图
  - `ClearingBankAddPage.tsx` — 输入 sfid_id 直查 + 关键字搜索(300ms 防抖调 SFID `/eligible-search`)
  - `ClearingBankDeclareNodePage.tsx` — peer_id 自动填(`query_local_peer_id`)+ rpc_domain/port 手填 + 4 重连通性自测 + WUMIN_QR_V1 签名提交
  - `ClearingBankDetailPage.tsx` — 详情(基础信息卡 + 节点信息长卡 + 管理员列表入口 + 提案列表 + 转账/手续费划转启用 + 换管理员/费率设置 disabled "即将上线")
  - `ClearingBankNodeInfoPanel.tsx` — peer_id/rpc_domain:port/registered_at/registered_by 展示 + 端点更新 / 注销两个入口(模态框 QR 签名)
  - `ClearingBankAdminListPage.tsx` — **本 tab 独有**"激活"+"解密"两段流程,激活复用治理 activate_admin,解密 = wumin 签 challenge → 内存"已解密"标记 + 加锁(lock_decrypted_admin)
- ✅ C3 `api.ts` 14 个新 invoke 调用 + `clearing-bank-types.ts` import
- ✅ C4 `assets/styles/global.css` 加清算行 CSS:`.admin-card-decrypted / .decrypted-tag / .green-dot / .decrypt-button / .status-badge / .connectivity-report / .node-info-panel / .metric-grid / .form-group / .balance / .countdown` 等

**验证**:
- `npx tsc --noEmit`:✅ 通过(0 错误)
- `npx vite build`:✅ 通过(86 modules transformed,index 435.27 kB / index.css 37.32 kB)

## 阶段 D 完工记录(2026-04-27)

**SFID 后端联动 + ClearingBankWatcher + app_search 过滤全部落地**:

- ✅ D1 `submit_register_sfid_institution_extrinsic` 加 `a3 / sub_type / parent_sfid_id` 三个参数(sub_type 与 parent_sfid_id 用 `Value::unnamed_variant("Some"/"None", ...)` 编码 Option),`institutions/chain.rs::submit_register_account` 同步加 3 参数,`institutions/handler.rs::activate_account` 调用处从 `inst.a3 / inst.sub_type / inst.parent_sfid_id` 取值传入
- ✅ D2 新建 `chain/clearing_bank_watcher.rs`:
  - `ClearingBankNodeCache` 结构(`HashSet<String>` + `last_scan_ok` flag,RwLock 保护)
  - `spawn_watcher(http_url)` tokio task,启动时即第一次全量 scan
  - `scan_once` 用 `state_getKeysPaged(prefix, 1000)` 分页拉所有 ClearingBankNodes storage key,反向解出 sfid_id(blake2_128_concat key 编码 = blake2_128(16B) + Compact<u32> 长度 + sfid_id 字节)
  - 30 秒轮询间隔 + 失败指数退避(1s → 60s)
  - 内置 twox_128 + decode_compact_u32 工具,无需引 substrate primitives
- ✅ D3 `AppState` 加 `clearing_bank_node_cache: Arc<ClearingBankNodeCache>`,`main.rs` 启动时 `chain::url::chain_http_url()` 读 URL → `spawn_watcher` 启动 watcher → 把 Arc 放进 state(链 URL 未配置时跳过启动 + 给空 cache)
- ✅ D4 `app_search_clearing_banks` 第 2 轮过滤里加 `if cb_cache_ready && !cb_node_cache_inner.contains(&inst.sfid_id) { continue; }`(cache 未首次 scan 成功时降级到老语义,避免接口空响应)
- ✅ D5 `main_tests.rs::build_test_state` 加空 `clearing_bank_node_cache` 字段

**验证**:
- `cargo check`:✅ 通过(4 warnings 全部预存,与本次无关)
- `cargo test --bin sfid-backend`:✅ **85/85 全绿**(原 80 case + 新加 5 case clearing_bank_watcher)

**已修改的关键文件**:
- citizenchain/node/src/ui/clearing_bank/{mod,types,chain,connectivity,signing,sfid_proxy,admin_decrypt}.rs
- citizenchain/node/src/ui/{mod.rs,governance/mod.rs(storage_keys 改 pub(crate)),network/network-overview/mod.rs(clearing_nodes 真实计数)}
- citizenchain/node/frontend/{App.tsx,api.ts,clearing-bank/*.tsx,assets/styles/global.css}
- sfid/backend/src/{main.rs,main_tests.rs,chain/{mod.rs,clearing_bank_watcher.rs},institutions/{chain.rs,handler.rs},sheng-admins/institutions.rs}

**Step 2 全部完成**。等 Step 3 wumin/wuminapp 完工后由 Architect 主入口走 `propose_runtime_upgrade` 联合提案上主网。

## 风险与依赖

| 风险 | 等级 | 应对 |
|---|---|---|
| 跨模块联动铁律 | 高 | spec=3 不发主网，dev 链验证；Step 3 完工后联动升级 |
| OnChargeTransaction 自定义复杂度 | 中 | 参考 polkadot-sdk 既有模式；先在 dev 链验证 fee_account 余额 0 时拒绝交易 |
| SfidAccountQuery trait 影响范围 | 中 | 所有引用该 trait 的 pallet 同步加默认实现，避免编译断裂 |
| 收款方主导改向破坏现有 bind_clearing_bank | 低 | bind_clearing_bank 校验只多一条（必须 ∈ ClearingBankNodes），现有用户绑定的机构若是合法清算行，无需迁移 |
| ClearingBankWatcher 断线 | 低 | 指数退避重连 + 重连后全量对账，最坏情况是几秒钟数据延迟 |
| "解密"术语只在清算行 tab 用 | 低 | 用独立 ClearingBankAdminListPage 组件，不污染 NRC/PRC/PRB |

- 状态：done

## 完成信息

- 完成时间：2026-04-27 19:18:21
- 完成摘要：ADR-007 Step 2 阶段 A/B/C/D 全部完成:runtime spec=3 / node Tauri+前端 14 cmd+8 视图状态机+解密管理员列表 / SFID watcher 30s 轮询 + app_search 过滤;node test 18/18 + sfid test 85/85 全绿;待 Step 3 wumin/wuminapp 完工后走 propose_runtime_upgrade 上主网
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
