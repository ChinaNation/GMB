# ADR-007 清算行三阶段拆分与"视图归类"模型

- 状态：accepted
- 日期：2026-04-24
- 决策人：Architect 主入口（Claude）+ 用户

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

### Step 2：区块链端（citizenchain）

- runtime: `bank_check::ensure_can_be_bound` 收紧到资格白名单（链上二次保险）
- runtime: 新增 `ClearingBankNodes: Map<sfid_id, ClearingBankNodeInfo>` storage 和 `NodePeerToInstitution` 反向索引
- runtime: 新增 `register_clearing_bank` / `update_clearing_bank_endpoint` / `unregister_clearing_bank` extrinsic
- runtime: spec_version 升级（2 → 3）走链上 `propose_runtime_upgrade`
- citizenchain/node UI：新增"清算行"顶级 tab（首页/挖矿/国储会/省储会/省储行/清算行/白皮书/公民宪法/设置 = 9 个 tab）
- NodeUI: 空页搜索 SFID 候选（调 Step 1 新增的 eligible-search） → 创建机构提案 / 已存在直接展示 → 声明本机节点为该机构清算节点
- NodeUI: clearing_nodes 字段从硬编码 0 改为读 ClearingBankNodes 长度
- 末尾联动：SFID 后端 `/clearing-banks/search` 加 `AND sfid_id ∈ ClearingBankNodes` 过滤

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

## 资金模型确认（与本 ADR 一致）

- 用户 `bind_clearing_bank` = 在该清算行开户（无预存费）
- 充值：用户钱包 → 清算行主账户（Currency 真转），同时 `DepositBalance[bank][user] += amount`
- 同行支付：DepositBalance 内部轧差，主账户内 fee 流转
- 跨行支付：链上原子 2 次 Currency::transfer（本金 + 手续费）+ 双方 DepositBalance 同步
- 用户单笔签名：sr25519 签 PaymentIntent 的 `blake2_256("GMB_L3_PAY_V1" || SCALE(intent))`
- 链上 `submit_offchain_batch_v2` 整批原子（with_transaction），失败全回滚

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
