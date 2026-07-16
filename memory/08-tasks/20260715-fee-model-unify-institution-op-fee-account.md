# 任务卡：机构 CID 主键统一、五类费用、投票快照与五端同步

## 状态

- 当前阶段：第 1 步实施中
- 第 1 步方案确认：2026-07-15
- runtime 二次确认：已获得
- 开发方式：breaking runtime，重新创世，不做旧存储、旧 call、旧 payload 或旧命名兼容

## 最终目标

全仓库、全平台以 `cid_number` 作为机构唯一主键。机构下面可以有多个机构账户、多个 `admins`、多个岗位和多条岗位任职；机构账户无私钥，只有 `admins` 中的管理员钱包持有私钥并代表机构签署交易。

机构交易统一分为：

1. 账户型机构交易：`actor_cid_number + institution_account + origin 管理员签名`。
2. 非账户型机构交易：`actor_cid_number + origin 管理员签名`。
3. 实际投票 `cast_*`：管理员个人签名并由签名者支付 `VoteFlat`。

主账户只是一种协议账户，不得作为机构 ID、管理员根、投票阈值 key、提案发起机构或费用路由 key。

## 强制业务规则

### 机构账户

- 普通机构必须有：主账户、费用账户。
- 国储会必须有：主账户、费用账户、安全基金账户、两和基金账户。
- 省储行必须有：主账户、费用账户、永久质押账户。
- 其他特殊机构按 `primitives::institution_constraints` 的唯一制度规格确定。
- 每一种强制协议账户必须存在且只能存在一个。
- 一个机构可以有多个自定义命名账户，同一 CID 下 `account_name` 唯一。
- 所有协议账户永远不可关闭；只有 `InstitutionNamed` 可以关闭。
- 逻辑账户允许零余额；非零初始金额必须大于等于 ED。

### 管理员、岗位和阈值

- `PublicAdmins/PrivateAdmins::AdminAccounts[cid_number]` 是机构执行授权真源。
- 管理员唯一字段为 `admins`。
- 岗位和任职统一以 `(cid_number, role_code)` 组织；有效任职变化原子刷新同一 CID 的 `admins`。
- 机构动态阈值使用 `ActiveInstitutionThresholds[cid_number]`。
- 个人多签继续使用 `ActivePersonalThresholds[personal_account]`，不得伪造机构 CID。

### 签名与凭证

- 外层标准 extrinsic `origin` 是唯一交易授权；必须属于 `AdminAccounts[actor_cid_number].admins`。
- 不新增 SignedExtension 或第二套授权真源。
- 注册局业务凭证只表达跨机构业务背书，不能替代外层授权。
- runtime 与 OnChina 共同调用 `runtime/primitives/src/sign.rs` 的唯一消息构造函数。

### 费用最终规则

- 全链费用严格落入既定五类；没有 `WeightToFee` 费用。
- 机构操作由 `actor_cid_number` 对应费用账户支付，失败即失败，不回落签名者。
- `VoteFlat` 只用于实际投票等个人签名投票交易，由签名者支付。
- Fullnode 不是机构，不进入机构费用路由。
- 未分类 call 一律拒绝。

### 投票职责

- 机构提案发起方使用 `actor_cid_number`，不得使用主账户或通用账户上下文表示机构身份。
- 具体账户只允许作为 `execution_account`，并强制验证属于 `actor_cid_number`。
- 人口快照、投票资格、状态推进、计票、终态与提案清理统一归 votingengine。

## 分步实施

### 第 1 步：机构 CID、账户、admins、岗位和交易身份唯一真源

- 建立机构类型到强制协议账户集合的唯一函数。
- 删除 `CidRegisteredAccount`、机构/账户生命周期状态、`is_default`、`ProtectedGenesisAccounts`。
- `InstitutionAccounts[(cid_number, account_name)]` 为正向账户真源，`AccountRegisteredCid` 为反向索引。
- public/private admins 改为 CID key，删除主账户管理员根和机构管理员关闭流程。
- 机构阈值按 CID，个人阈值按个人账户。
- 机构提案增加 `actor_cid_number`；具体账户只作 `execution_account`。
- 立法、决议发行、互选、普选、机构治理和注册局管理统一使用 CID + 管理员。
- 机构转账和具体账户操作统一使用 CID + 账户 + 管理员。
- 删除重复 `register_cid_*` call；创建、批量新增、关闭账户统一命名。
- 允许零初始余额；非零初始余额校验 ED。
- 五端同步 runtime、node、OnChina、CitizenApp、CitizenWallet。
- 重新创世并做真实运行态验收。

### 第 2 步：费用分类与机构费用路由唯一真源

- RuntimeCall 穷尽分类到五类费用。
- 机构操作解析唯一 `actor_cid_number`。
- 机构费用账户由 CID + `InstitutionFee` 唯一解析。
- 不允许任何付费方回落。
- `cast_*` 等真实投票保持签名者 `VoteFlat`。
- Fullnode 保持非机构分类。

### 第 3 步：执行期直接扣费与 ED 规则统一

- 所有收费统一进入对应五类的执行器。
- 机构费用直接从费用账户扣除，余额不足交易失败。
- 普通支出统一校验 ED；显式账户关闭允许账户死亡。
- 不使用 `WeightToFee`、最坏路径权重费用或隐式 Substrate 交易费。

### 第 4 步：快照与提案清理收归投票引擎

- 删除业务 pallet 的 public `prepare_*_snapshot` extrinsic 和 pending snapshot 中转。
- 提案创建时由 votingengine 内部生成并锁定快照。
- 删除 public/private/personal 业务模块的 `cleanup_rejected_*` 和 pending 残留。
- votingengine 统一在终态、超时和执行失败路径清理。

### 第 5 步：全仓最终验收

- runtime、node、OnChina、CitizenApp、CitizenWallet 全量测试。
- 重新创世，启动真实 node、真实 OnChina 数据库/API/页面并执行真实扫码签名交易。
- 全仓搜索旧 key、旧 call、旧 payload、旧命名、旧文案、旧流程为零残留。

## 第 1 步预计修改范围

- `citizenchain/runtime/primitives/`：协议账户集合、CID 制度约束、地址派生和签名消息单源。
- `citizenchain/runtime/entity/`：机构、账户、岗位、任职、正反索引和生命周期清理。
- `citizenchain/runtime/admins/`：机构 admins 改 CID key。
- `citizenchain/runtime/votingengine/`：机构 actor CID、管理员快照和阈值路由。
- `citizenchain/runtime/governance/resolution-destroy/`、`grandpakey-change/`：机构治理发起方改 CID。
- `citizenchain/runtime/issuance/resolution-issuance/`、`onchain-issuance/`：机构身份与具体资产账户分离。
- `citizenchain/runtime/transaction/multisig/`：机构账户交易改 CID + 账户 + 管理员。
- `citizenchain/runtime/genesis/`：按机构制度校验完整协议账户集合。
- `citizenchain/runtime/src/`：runtime 聚合查询和授权；本步不改费用分类、付款方或 TxExtension。
- `citizenchain/node/src/`：RAW storage、node guard 和机构读取同步。
- `citizenchain/onchina/src/`、`frontend/`：CID 请求、凭证、账户页面和真实权限同步。
- `citizenapp/lib/`、`test/`：CID admins、阈值、提案和 storage 解码。
- `citizenwallet/lib/`、`test/`：call、QR、payload 解码和旧协议清理。
- `memory/05-modules/`：更新现有技术文档。

不新增文件或目录；如发现必须新增，先列明完整路径、用途、原因和 Git 跟踪状态并重新请求确认。

## 第 1 步验收

### 自动验收

- runtime 相关 crates 全量测试、clippy、benchmark/weights 更新。
- node 测试与构建。
- OnChina Rust 测试、前端测试和 build。
- CitizenApp/CitizenWallet `flutter test`、`flutter analyze`。
- 五端 SCALE call、storage key/value 和签名金标一致。

### 真实运行态验收

- 重新创世启动真实本地链和 node guard。
- 普通机构、国储会、省储行的协议账户集合逐项正确。
- 注册局管理员以 CID 发起真实机构创建，零初始余额成功，低于 ED 的非零金额失败。
- 非管理员失败；CID 与机构账户不匹配失败。
- 立法、决议发行、互选、普选等机构发起方按 CID。
- 所有协议账户关闭失败；自定义账户关闭后机构、协议账户、admins、岗位和阈值保持不变。
- OnChina 真实页面、CitizenApp 展示和 CitizenWallet 扫码解码全部与链上状态一致。

第 1 步全部通过后停止，等待用户确认再进入第 2 步。
