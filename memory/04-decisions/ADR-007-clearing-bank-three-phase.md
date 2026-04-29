# ADR-007 清算行三阶段拆分与"视图归类"模型

- 状态：accepted
- 日期：2026-04-24
- 决策人：Architect 主入口（Claude）+ 用户
- 进度:Step 1(2026-04-24)✅,Step 2 阶段 A+B+C+D(2026-04-27)✅,Step 3 待启动

## 上下文

GMB 区块链节点 UI 需要支持"清算行 tab"，让节点机构方在区块链软件上注册自己为清算行节点。在多轮需求澄清中明确了三个核心事实：

1. **清算行不是新机构类型**：链上现有机构类型枚举（国储会/省储会/省储行 = orgType 0/1/2）不增加新枚举值
2. **清算行是视图归类**：以"私法人股份公司 + 从属于私法人股份公司的非法人"作为资格白名单，把符合条件的现有机构在 NodeUI 上归类展示为"清算行"
3. **省储行的清算业务已废除**：43 个省储行不再做清算，旧 11 个清算 extrinsic（call_index 0/1/2/9/10/11/14-20/23）已在 Step 2b-iv-b 物理删除；清算业务由"注册清算行"的全节点组成清算网络

## 决策

把清算行体系实现拆为 **3 个独立阶段**，每阶段独立可发布、可验收：

### Step 1：SFID 端（本任务卡 20260424-step1-sfid-clearing-bank-eligibility）

**只动 sfid/backend + sfid/frontend**：
- 资格白名单判定函数：`is_clearing_bank_eligible(inst, parent) -> bool`
- 收紧 `GET /api/v1/app/clearing-banks/search` 到资格白名单（已激活的 SFR-JOINT_STOCK 与其下属 FFR）
- 新增 `GET /api/v1/app/clearing-banks/eligible-search`（NodeUI"添加清算行"用，含未激活机构）
- SFID 前端机构列表 / 详情页显示"可作为清算行"badge
- PrivateInstitutionLayout 选择 sub_type=JOINT_STOCK 时增加提示文案

**不做**：runtime 改动、wumin/wuminapp 改动、链上 ClearingBankNodes storage、PeerId 绑定。

### Step 2：区块链端（citizenchain，2026-04-27 完工）

#### 2.1 runtime 核心改动

**A. 清算方向反转：收款方主导清算**

- `submit_offchain_batch_v2` 校验改为 `item.recipient_bank == institution_main`（原为 `payer_bank`）
- batch 提交者 = 收款方清算行的某个 duoqian_admin（不再是付款方）
- fee 流向统一：永远归 `fee_account_of(recipient_bank)`（同行 / 跨行统一规则，简化 settlement.rs 分支逻辑）
- 链上验签流程不变：A 的 sr25519 签名 PaymentIntent 是核心授权，pallet 凭授权 mutate Currency，谁提交 batch 不影响安全

**B. gas 由 fee_account 直接支付**

- 新增 runtime 自定义 `OnChargeTransaction` 实现（`ChargeBatchFromInstitution`）
- 仅针对 `submit_offchain_batch_v2` 这个 call 特殊处理：从 `fee_account_of(institution_main)` 直接扣 gas
- 其他 call 走默认 CurrencyAdapter（从 origin 个人账户扣）
- 管理员个人钱包余额完全不动；清算行 fee 收入直接覆盖 gas 成本（fee 量级 vs gas 量级 ≈ 1000:1，盈余充足）

**C. 机构元数据上链（Required，开发期彻底切换）**

- 新增 `InstitutionMetadata: Map<SfidId, MetadataInfo>` storage，包含 `a3 / sub_type / parent_sfid_id`
- `register_sfid_institution` / `propose_create` 等创建路径增加 a3/sub_type/parent 参数（Required）
- **不做 backfill_institution_metadata**——开发期无旧数据，按 [feedback_chain_in_dev.md](../feedback_chain_in_dev.md) fresh genesis 重建

**D. 资格白名单链上二次校验**

- `bank_check::ensure_can_be_bound` 校验链由 4 重收紧到 6 重：
  - 原 1-4：AddressRegisteredSfid / account_name="主账户" / a3 ∈ {SFR,FFR} / DuoqianAccount.status=Active
  - **新 5**：资格白名单（SFR-JOINT_STOCK ∨ FFR-parent.SFR.JOINT_STOCK），通过 SfidAccountQuery trait 查 InstitutionMetadata
  - **新 6**：sfid_id ∈ ClearingBankNodes（必须是已声明的清算行节点）

**E. 清算行节点声明 storage + extrinsic**

- 新增 `ClearingBankNodes: Map<SfidId, ClearingBankNodeInfo>` storage（peer_id / rpc_domain / rpc_port / registered_at / registered_by）
- 新增 `NodePeerToInstitution: Map<PeerId, SfidId>` 反向索引（防 PeerId 冒名）
- 新增 `register_clearing_bank(sfid_id, peer_id, rpc_domain, rpc_port)` extrinsic（任一 duoqian_admin 单签即可，不走内部投票）
- 新增 `update_clearing_bank_endpoint(sfid_id, new_domain, new_port)`（仅改端点，不动 PeerId）
- 新增 `unregister_clearing_bank(sfid_id)`（注销 + 反向索引清理）

**F. spec_version 2 → 3**

- runtime 代码 bump，dev 链直接用新版
- **主网升级**：Step 2 不上链；等 Step 3 wumin decoder + wuminapp 兼容做完后，由 Architect 主入口走 `propose_runtime_upgrade` 联合提案上链

#### 2.2 node Tauri 后端改动

新增 Tauri command（`citizenchain/node/src/offchain/commands.rs`，具体能力拆在 `offchain/{sfid,chain,health,signing,decrypt}.rs`）：

- `search_eligible_clearing_banks(query, limit)`：转发 SFID `/clearing-banks/eligible-search`
- `query_clearing_bank_node_info(sfid_id)`：链上查 `ClearingBankNodes[sfid_id]`
- `query_local_peer_id()`：调 RPC `system_localPeerId` 拿本机 PeerId
- `test_clearing_bank_endpoint_connectivity(domain, port, expected_peer_id)`：连通性自测（DNS + wss 连接 + 链 ID 匹配 + system_localPeerId 匹配）
- `build_register_clearing_bank_request` / `submit_register_clearing_bank`
- `build_update_clearing_bank_endpoint_request` / `submit_update_clearing_bank_endpoint`
- `build_unregister_clearing_bank_request` / `submit_unregister_clearing_bank`
- 修改 `get_network_overview`：`clearing_nodes` 字段从硬编码 0 改为 `ClearingBankNodes::iter().count()`

#### 2.3 node 前端改动（清算行 tab）

- `App.tsx` TabKey 加 `'clearing-bank'`，顶部 nav 9 tab：首页 / 挖矿 / 国储会 / 省储会 / 省储行 / **清算行** / 白皮书 / 公民宪法 / 设置
- 新建 `offchain/section.tsx`，状态机 8 视图：
  ```
  empty → add-input-sfid → check-status →
    ├─ register-sfid (链上未注册地址)
    ├─ propose-create (未创建多签账户)
    ├─ wait-vote     (等其他 admins 投票)
    ├─ declare-node  (Active 后,声明清算行节点)
    └─ detail        (完成,显示机构详情)
  ```
- 复用 `governance/InstitutionDetailPage` 展示 5 卡片 + 提案列表
- **管理员列表 UI 仅在清算行 tab 用"解密"术语**（NRC/PRC/PRB 沿用原"激活"不动）：
  - 列表行新增"解密"按钮 + 状态指示绿点
  - 解密 = wumin 扫码签 challenge → 节点验签 → 解密本地加密存储的私钥到内存
  - 内存中密钥永久驻留至节点重启，无时间限制
  - 解密后 packer 攒批可直接用内存中密钥签 `submit_offchain_batch_v2`
- 提案按钮：转账 / 手续费划转启用；换管理员 / 费率设置 disabled "即将上线"
- 新增"节点信息"长卡片：peer_id / rpc_domain:rpc_port / 注册管理员 + 端点更新/注销入口
- 提交 register_clearing_bank 前**强制 NodeUI 连通性自测**

#### 2.4 SFID 端 Step 2 末尾联动

- `sfid/backend/src/institutions/handler.rs::app_search_clearing_banks` 在第 2 轮跨省扫描里加过滤：
  - `AND sfid_id IN (SELECT sfid_id FROM clearing_bank_nodes_cache)`
- 新建 `sfid/backend/src/chain/clearing_bank_watcher.rs`：常驻 tokio task 订阅链上 `ClearingBankRegistered/Updated/Unregistered` 事件 + 全量启动 scan + SQLite 缓存（按 [feedback_no_dns_peerid_firewall](../feedback_no_dns_peerid_firewall.md) 不假设网络问题）
- SFID 后端推链 `register_sfid_institution` 等调用增加 a3/sub_type/parent 参数

### Step 3：手机端（wumin + wuminapp）

- wumin 冷钱包 decoder 补 `register_clearing_bank` / `update_clearing_bank_endpoint` / `unregister_clearing_bank` 扫码签名分支
- wumin pallet_registry action_labels 补对应中文标签
- wuminapp `bind_clearing_bank_page.dart` 调整：搜索来源切换为新 search API；绑定前查链上 ClearingBankNodes 取 RPC 域名+端口
- wuminapp `clearing_bank_settings_page.dart` 占位页落地（用户视角的"我的清算行配置"）
- 端到端验证清单（创建机构 → SFID 注册 → 链上注册清算行 → wuminapp 绑定 → 充值 → 跨行支付 → 提现）

## 链上准入设计（Step 2 锁定）

清算行准入用 **3 层卡口**：

1. **SFID 身份（Step 1 落地）**：机构必须在 SFID 后端注册成功（A3/机构码/sub_type/parent 校验）
2. **链上资格白名单（Step 2）**：`(SFR ∧ JOINT_STOCK) ∨ (FFR ∧ parent.SFR ∧ parent.JOINT_STOCK)`，链上 storage 自证不依赖中心化签名
3. **节点-机构绑定（Step 2）**：管理员私钥签名 + node PeerId 上链；同时配置 RPC 域名供 wuminapp 可达

PeerId 由节点 `base_path/node-key/secret_ed25519` 确定性生成，重启不变；域名作为辅助字段，可单独 update 不影响 PeerId 主键。

## 资金模型（2026-04-27 修订：收款方主导清算）

- 用户 `bind_clearing_bank` = 在该清算行开户（无预存费）
- 充值：用户钱包 → 清算行主账户（Currency 真转），同时 `DepositBalance[bank][user] += amount`
- 用户支付（核心修订）：
  - **wuminapp 把签名 PaymentIntent 发给收款方清算行的 wss 端口**（不再发给付款方）
  - 收款方清算行（Y）的 packer 攒批 → Y 的某个已解密管理员密钥自动签 `submit_offchain_batch_v2`
  - 链上验 A 签名 → 扣 X 主账户（A 的存款方） → 本金到 Y 主账户 + fee 到 Y 费用账户
- 同行支付：A、B 都在 X，X 自己作为收款方清算行清算；DepositBalance 内部轧差；fee 进 X 自己费用账户
- 跨行支付：A 在 X、B 在 Y，**Y 主导**链上原子 2 次 Currency::transfer（X主→Y主 本金 + X主→Y费用 fee）+ DepositBalance 双向同步
- 用户单笔签名：sr25519 签 PaymentIntent 的 `blake2_256("GMB_L3_PAY_V1" || SCALE(intent))`
- 链上 `submit_offchain_batch_v2` 整批原子（with_transaction），失败全回滚
- 2026-04-28 补齐：批次入口必须同时满足清算行管理员 batch 签名有效、
  `batch_seq == LastClearingBatchSeq[recipient_bank] + 1`、付款/收款双方
  `UserBank` 与 item 声明一致；成功 settlement 后才推进 `LastClearingBatchSeq`
- gas：**自定义 OnChargeTransaction 让 fee_account_of(institution_main) 直接付**，管理员个人钱包不参与
- 经济模型自洽：fee 量级 0.01%~0.1% × 交易金额，gas 量级约 fee × 0.1%，盈余约 99.9%

## 收款方主导的合理性

| 角度 | 说明 |
|---|---|
| 公平性 | 谁做事谁拿钱：Y 做了清算批次提交、本机账本管理、流动性承担，应得 fee |
| 业务一致性 | Y 是商户 B 的清算行，主导收款 = 现实金融中"商户银行"角色 |
| 同行/跨行统一 | 一套规则：永远收款方清算 + 收款方拿 fee + 收款方付 gas |
| 自给自足 | gas 由 fee_account 直接覆盖，运营闭环不依赖管理员个人垫付 |
| 安全模型 | A 的 sr25519 签名是核心授权（PaymentIntent 已含 payer_bank=X）；Y 提交时链上凭授权扣 X 主账户，与谁提交无关 |

## 后果

**优点**：
- 清算行接入门槛低（任意符合条件的私法人股份公司及其下属非法人都能加入），可扩到银行/分支行/第三方支付/大企业
- 不需要新增链上机构类型 / 新 orgType / 新 institution_code，runtime 改动最小化
- 资格判定在 SFID 后端 + 链上双层校验，单层故障不影响整体安全
- 三阶段独立发布，每阶段可验收，降低风险

**取舍**：
- SFID 后端的"清算行候选"语义会随 Step 2 完成而进一步收窄（加入"已加入清算网络"过滤），是预期演进
- 跨省 parent 查询需要二段读 shard，性能依赖 sharded_store 的并发读能力（已有）

## 与现存设计的兼容性

- [project_institution_create_rules.md](../project_institution_create_rules.md):56 的"清算行概念彻底废除"应理解为"`is_clearing_bank` 字段废除（省储行兼任清算行的旧模型废除）"，与本 ADR 的"清算行作为 SFR-JOINT_STOCK + FFR 子集的视图归类"不冲突
- [feedback_no_compatibility.md](../feedback_no_compatibility.md)：本 ADR 的 3 阶段切换不保留兼容窗口，每步直接切换
- [feedback_chainspec_frozen.md](../feedback_chainspec_frozen.md)：Step 2 的 spec_version 升级走链上 `propose_runtime_upgrade`，不重建 chainspec

## 引用

- 现有清算 pallet：[citizenchain/runtime/transaction/offchain-transaction-pos/src/lib.rs](../../citizenchain/runtime/transaction/offchain-transaction-pos/src/lib.rs)
- 现有 SFID 公开 API：[sfid/backend/src/institutions/handler.rs:1605](../../sfid/backend/src/institutions/handler.rs)（app_search_clearing_banks）
- ParentInstitutionRow 已含 sub_type 字段：[sfid/backend/src/institutions/model.rs:228-230](../../sfid/backend/src/institutions/model.rs)
