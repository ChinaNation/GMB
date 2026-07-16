# 任务卡：机构 CID 主键统一、五类费用、投票快照与五端同步

## 状态

- 当前阶段：第 5 步创世准备、五端自动验收、preview 候选链和真实 OnChina
  运行态验收已完成；按用户要求未执行 CI、正式冻结、`--finalize` 或正式创世。
  Rust workspace 已不排除任何 crate 全量通过；真实管理员扫码签名仍待专用测试
  签名环境。
- 第 1 步方案确认：2026-07-15
- runtime 二次确认：已获得
- 开发方式：breaking runtime，重新创世，不做旧存储、旧 call、旧 payload 或旧命名兼容

## 最终目标

全仓库、全平台以 `cid_number` 作为机构唯一主键。机构下面可以有多个机构账户、多个 `admins`、多个岗位和多条岗位任职；机构账户无私钥，只有 `admins` 中的管理员钱包持有私钥并代表机构签署交易。

机构交易统一分为：

1. 账户型机构交易：`actor_cid_number + institution_account + origin 管理员签名`。
2. 非账户型机构交易：`actor_cid_number + origin 管理员签名`。
3. 实际投票 `cast_*`：管理员或公民个人签名并由签名者支付固定 1 元投票费。

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
- 固定 1 元投票费只用于实际投票等个人签名投票交易，由签名者支付。
- Fullnode 不是机构，不进入机构费用路由。
- 未分类 call 一律拒绝。

### 投票职责

- 机构提案发起方使用 `actor_cid_number`，不得使用主账户或通用账户上下文表示机构身份。
- 具体账户只允许作为 `execution_account`，并强制验证属于 `actor_cid_number`。
- 人口快照、投票资格、状态推进、计票、终态与提案清理统一归 votingengine。

## 分步实施

### 第 1 步：机构 CID、账户、admins、岗位和交易身份唯一真源

- 建立机构类型到强制协议账户集合的唯一函数。
- 删除旧的 CID 正向单账户映射、机构/账户生命周期状态、重复默认标记和额外创世保护表。
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
- `cast_*` 等真实投票统一路由为 `FeeRoute::Vote { payer: signer }`。
- Fullnode 保持非机构分类。

### 第 3 步：执行期直接扣费与 ED 规则统一

- 所有收费统一进入对应五类的执行器。
- 机构费用直接从费用账户扣除，余额不足交易失败。
- 普通支出统一校验 ED；显式账户关闭允许账户死亡。
- 不使用 `WeightToFee`、最坏路径权重费用或隐式 Substrate 交易费。

### 第 4 步：快照与提案清理收归投票引擎

- 删除业务 pallet 的 public `prepare_*_snapshot` extrinsic 和 pending snapshot 中转。
- 提案创建时由 votingengine 内部生成并锁定快照。
- 删除 public/private/personal 业务模块的手工清理入口和 pending 残留。
- votingengine 统一在终态、超时和执行失败路径清理。

### 第 5 步：全仓最终验收

- runtime、node、OnChina、CitizenApp、CitizenWallet 全量测试。
- 在 `target/` 生成不覆盖正式资产的 preview 创世候选，启动隔离 node、真实 OnChina
  数据库/API/页面；正式 CI、release 冻结和正式创世按用户要求留到后续阶段。
- 有专用管理员私钥时执行真实扫码签名交易；没有时必须保持阻塞，不得伪造私钥、
  临时管理员或费用回落路径。
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

第 1 步自动验收完成后，用户已明确要求继续执行第 2 步；缺少专用管理员私钥导致的真实签名验收不构成恢复旧协议或伪造私钥的授权。

## 第 1 步执行与验收记录（2026-07-15）

### 已完成

- runtime、node、OnChina、CitizenApp、CitizenWallet 已统一机构 CID 主键；机构账户只作为同一 CID 下的具体执行账户，个人多签只使用 `personal_account`。
- `PublicAdmins/PrivateAdmins::AdminAccounts[cid_number]`、`ActiveInstitutionThresholds[cid_number]`、岗位与任职、提案主体和管理员快照均按 CID；个人管理员与阈值继续按个人账户。
- 普通机构、国家储委会、省储行的协议账户集合由 `institution_constraints` 单源确定；协议账户不可关闭，自定义账户可关闭；零初始余额允许，非零低于 ED 拒绝。
- 删除旧单账户映射、机构主账户身份/授权字段、机构生命周期与重复默认标记；数据库启动代码不保留旧列迁移兼容。
- 机构账户派生统一命名为 `derive_institution_account`；runtime `MODULE_TAG` 和 owner data 统一为 `multisig`，旧标签只在拒绝测试中作为非法输入出现。
- 本记录生成时第 2 步尚未实施；其后第 2 至第 4 步均已按下方执行记录完成。

### 自动验收

- runtime 及相关 pallet 全量回归通过；最终复验 `citizenchain` 42、`multisig` 24、`offchain` 24、`public-manage` 15、`private-manage` 10，均 0 失败。
- `cargo test -p node`：270 通过、0 失败。
- `cargo test -p onchina`：131 通过、0 失败；OnChina 前端 production build 通过。
- CitizenApp：666 通过、5 项环境性跳过；`flutter analyze` 0 问题。
- CitizenWallet：165 通过；`flutter analyze` 0 问题。
- node 前端 production build、`runtime-benchmarks` 编译、`cargo fmt --all -- --check`、普通 clippy 和 `git diff --check` 均通过。
- 旧 key、旧授权方法、旧动态阈值、旧主账户身份字段与旧账户派生接口残留扫描为 0；不把第 4 步快照入口误删。

### 真实重创世状态验收

- 使用最终源码重新构建 runtime WASM，导出新 chainspec 并在独立临时目录启动真实本地链；最终临时链 genesis hash 为 `0x2c5b44639235e88e602a9bf88b0473e6fe8f6e9a72b8b4a9a38fd303c212ad91`，节点和 node guard 创世完整性检查通过。
- 新创世状态共 49,593 个机构、99,231 个机构账户，正向账户记录与反向索引数量完全一致。
- 国家储委会 1 个，协议账户精确为主账户、费用账户、安全基金账户、两和基金账户。
- 省储行 43 个，协议账户精确为主账户、费用账户、永久质押账户。
- 其余机构协议账户精确为主账户、费用账户；协议集合错误数为 0；旧正向单账户 storage key 数为 0。
- 临时验收节点已关闭，没有连接相同创世网络、没有提交交易、没有读取或使用用户私钥。

### 尚未完成的真实验收

- 尚未执行“注册局管理员真实扫码签名后发起创建/低于 ED/非管理员/CID 与账户不匹配/账户关闭”的端到端交易与真实页面联动。
- 原因：当前创世管理员只有固定公钥，仓库和自动化环境没有对应私钥；不得伪造管理员、读取用户 Keychain 或要求用户在聊天中提供私钥。
- 完成方式：由用户在本机解锁并授权一个专用测试管理员钱包参与扫码，或另行明确批准仅用于验收的测试创世管理员方案。用户已明确要求在不伪造私钥的前提下继续第 2 步，因此该环境限制保留为最终真实验收项。

## 第 2 步执行记录（2026-07-15）

### 已完成

- 在 `primitives::fee_policy` 建立唯一 `FeeRoute<AccountId, Balance>`，把五类费用与确切付款账户合并为同一协议类型；删除历史费种、付款方并行类型和 runtime 分类/付款提取双真源。
- `RuntimeFeeRouter` 对 `RuntimeCall` 穷尽映射；各 pallet 隐藏 `__Ignore` 分支只允许显式 `Reject`，新增 runtime pallet 仍需在外层 match 中显式分类。
- 机构操作严格使用 `actor_cid_number + admins 外层签名 + InstitutionFee`。费用账户、公私权归属、正反索引、具体机构账户任一不一致即 `Reject`，不允许改扣管理员。
- 机构发起提案统一为最低链上交易费 0.1 元；只有实际 `cast_*` / 表决动作由签名者支付固定 1 元投票费。
- Fullnode 绑定/重绑奖励钱包保持非机构操作，由 Fullnode 自己的签名账户支付 0.1 元。
- 链下清算批次按多付款人模型归为 `OffchainFeePayer::BatchItemPayers`：每个付款公民从其 L2 存款支付 item 费用；收款方清算行费用账户是收款账户，不再错误标成单一付款账户，也不另收链上 gas。
- `TRANSACTION_TIP` 固定为零；Rust 统一签名构造、CitizenApp 热签和 CitizenWallet 冷签尾一致，runtime 与公民钱包均拒绝非零 tip。
- `WeightToFee`、`LengthToFee` 固定为零；node `fee_blockFees` 只统计 `FeePaid.fee`，不再把 FRAME tip 事件拼接为第二口径。
- `developer_direct_upgrade` 显式增加 `actor_cid_number`，runtime 与 node call 编码、构建、提交和测试同步；管理员签名仍是标准 extrinsic origin，不建立第二授权真源。
- CitizenApp 已把清算行绑定错误的 1 元提示修正为普通链上操作 0.1 元，并修正资产关闭提案/实际投票的费用说明；CitizenWallet 增加非零 tip 拒签回归。
- `onchain-issuance` 的 10 个公开 `propose_*` 仍是直接返回成功的业务占位，统一路由为 `Reject`；在发行模块真正创建投票并实现资产执行前，禁止扣费后返回无业务结果。
- 更新 primitives、onchain、跨模块、runtime-upgrade、node、统一协议、统一命名文档，并清理旧费用类型和回落描述。

### 自动验收

- Rust 核心回归：`chain-signing` 4、`citizenchain` 43、`onchain` 20、`offchain` 24、`primitives` 71 加 2 组金标、`runtime-upgrade` 19，均 0 失败。
- `cargo test -p node -p onchina`：退出码 0；node 270、OnChina 131，均 0 失败。
- CitizenApp：666 通过、5 项环境性跳过，0 失败；`flutter analyze` 0 问题。
- CitizenWallet：166 通过、0 失败；`flutter analyze` 0 问题。
- runtime `runtime-benchmarks` 特性编译、node 前端 production build、OnChina 前端 production build、Rust 格式与 `git diff --check` 均通过。
- Rust 整体普通 Clippy 通过；`-D warnings` 被未改动的 `runtime/build.rs` 4 处既有 `expect()` 拦截，本步没有把无关构建脚本纳入修改范围。本步新增代码自身发现的常量断言和返回类型复杂度告警已清理。
- 可执行源码中的旧费用类型、双分类器、付款方回落、治理统一 1 元、免费可收 tip、RPC 叠加 FRAME tip 等残留扫描为 0；生产 runtime 与 onchain 测试 mock 的 `WeightToFee`、`LengthToFee` 均固定为零；白皮书已同步并重新生成节点本地文档产物。

### 真实运行态边界

- runtime 测试状态机已直接验证：机构操作只扣费用账户、管理员余额不变；删除费用账户映射后返回 `InvalidTransaction::Call`，即使管理员有余额也不回落。
- 真实管理员扫码交易仍受第 1 步所述私钥环境限制；不得以伪造管理员或临时兼容路径替代。

## 第 3 步执行与验收记录（2026-07-16）

### 已完成

- `primitives::fee_policy` 增加唯一链上执行期收费接口；`onchain` 实现通用执行收费器，与外层 `FeeRoute` 共用同一费率常量、80/10/10 分账和 `FeePaid` 事件，不存在第二套费种分类。
- 公权/私权机构创建 ABI 增加显式 `funding_account`：非零初始本金只能从 actor CID 下符合用途的确切机构账户支出，零初始本金必须不传资金账户；签名原文、OnChina 构建、CitizenWallet 解码已同步。
- 公权/私权机构创建、关闭，机构多签转账、安全基金转账、费用账户归集和决议销毁的执行费，全部只从 actor CID 的确切费用账户收取；本金仍只从载荷指定的所属机构账户支出，任一账户缺失、归属不一致或余额不足均原子失败，管理员钱包无代付路径。
- 个人多签创建的本金和执行费由创建者支付；关闭执行费从个人多签账户收取。关闭的重复固定余额门槛已删除，提案和执行均按统一费率公式 + 链上 ED 动态校验。
- 协议账户仍不允许关闭，只有 `InstitutionNamed` 可关闭；普通支出和收费统一 `KeepAlive`，显式账户关闭转出才使用 `AllowDeath`。
- 删除 `onchain-issuance` 已废弃的 1000 GMB 资产创建押金、单独扣费/退费存储和本地分账实现；未实装的公开占位 call 继续 `Reject`，禁止扣费后无业务结果。
- CitizenApp 已分开检查机构本金账户、机构费用账户、外层操作费、后续执行费和 ED；个人多签关闭页不再错误写死 1.11 元。CitizenWallet 已显示本金账户与费用付款账户。
- 清理 public/private 重复收费 helper、onchain-issuance 废弃收费文件、历史费用类型/金额提取命名、过期文档和错误费用文案。

### 自动验收

- Rust 核心回归：`citizenchain` 43、`onchain` 21、`multisig` 25、`personal-manage` 23、`public-manage` 15、`private-manage` 10、`resolution-destroy` 16、`onchain-issuance` 8，均 0 失败。
- `cargo test -p node -p onchina --no-fail-fast`：退出码 0；node 270 项通过，OnChina 回归通过。OnChina 前端 production build 通过。
- CitizenApp：683 项通过、5 项环境性跳过；`flutter analyze` 0 问题。CitizenWallet：166 项通过；`flutter analyze` 0 问题。
- `cargo check -p citizenchain --features runtime-benchmarks`、当前源码 `cargo build -p node`、相关 crates 全目标 Clippy 和 `git diff --check` 均通过。Clippy 仍报告仓库既有的 `expect/unwrap`、大枚举和大错误类型等警告，无新增编译失败。
- 生产 runtime 和收费 mock 的 `WeightToFee` / `LengthToFee` 均为零；生产直接 `withdraw(FEE)` 只剩统一 `OnchainExecutionFeeCharger`，没有 pallet 各自的分账真源。

### 真实节点验收

- 使用当前源码重新编译 node/runtime WASM，导出 `citizenchain-fresh` 临时 chainspec，清空 bootnodes 后在独立 `--tmp` 目录启动真实节点。
- NodeGuard 通过；RPC 返回 `isSyncing=false`、`peers=0`、`shouldHavePeers=false`，链为 `CitizenChain`。最终源码重建复验的创世哈希为 `0x432f39f3b969f3ecf7c97062a2242f18b116020410fd9faba5c07ccbfab2ee4e`，状态根为 `0x06a8ced41198f9087b2054bdd6b1080c326de64bce6448d90707c0c70b7db05f`。
- 直接用当前冻结 chainspec 启动时，NodeGuard 按设计 fail-closed，报告 `FixedInstitutionMissing(NRC)`；说明正式冻结创世仍是旧状态。本步没有放宽守卫、没有覆盖冻结 SSOT；正式发布必须用同次成功 CI WASM 统一重烤 chainspec、创世状态包和 CitizenApp 轻客户端锚点。
- 验收节点已正常退出，临时 chainspec 已删除，没有连接现有网络、提交交易或使用任何用户私钥。

### 尚待专用签名环境的最终端到端项

- 注册局管理员真实扫码签名、机构费用账户真实扣款/余额不足失败、本金账户支出和五端 UI 联动，仍需用户在本机授权专用测试管理员钱包。
- 该环境限制不能通过伪造私钥、临时管理员、管理员代付或兼容路径绕过。

## 第 4 步执行与验收记录（2026-07-16）

### 已完成

- `citizen-identity` 删除公开人口快照 call，保留唯一内部
  `CitizenIdentityReader::create_population_snapshot`；call index 5 永久留洞。
- `joint-vote` 删除公开快照 call 和待消费快照存储；联合提案在同一事务中创建全国
  人口快照、写入提案并绑定 `snapshot_id`，人口为零或任一步失败时整体回滚；call
  index 2 永久留洞。
- `legislation-vote` 删除公开快照 call 和待消费快照存储；特别案只从已验证的
  `actor_cid_number` 唯一推导国家/省/市作用域并在建案事务内创建快照，普通案和
  重大案不创建；call index 0 永久留洞。
- CID 的 R5 省市作用域解析收敛到 `primitives::cid::number::cid_scope_codes`，runtime
  费用路由与立法引擎共同调用，不保留第二份字节切片规则。
- `personal-manage`、`public-manage`、`private-manage` 删除公开拒绝清理 call 和旧
  权重/benchmark/钱包 action；否决、超时、执行失败统一由 votingengine 终态回调
  清除业务 pending，再由引擎维护管线按保留期清理提案、投票和人口快照。
- CitizenApp、CitizenWallet、OnChina 已删除独立快照与人工清理的 call 常量、QR
  映射、payload 解码、调用构造和旧文案；永久留洞不解码、不兼容。
- 更新投票引擎、公民身份、公私权机构、个人多签、CitizenApp、QR 和统一协议文档，
  并清理生产代码与当前协议文档中的旧流程残留。

### 权重与自动验收

- 使用当前源码构建 release benchmark runtime，以 50 steps / 20 repeat 真实重跑
  `runtime_upgrade` 与 `resolution_issuance`。联合提案权重已包含人口快照存储：分别
  测得 101 reads / 193 writes 与 102 reads / 192 writes，不使用自定义最坏路径费用。
- `cargo check -p citizenchain --features runtime-benchmarks` 通过。
- 核心回归通过：`citizen-identity` 27、`internal-vote` 95、`joint-vote` 10、
  `legislation-vote` 34、`personal-manage` 22、`public-manage` 16、`private-manage` 11、
  `runtime-upgrade` 19、`resolution-issuance` 18、`legislation-yuan` 32，均 0 失败。
- runtime 最终回归 `citizenchain` 43、`primitives` 72 加两组跨端金标，均 0 失败；
  相关全目标 Clippy 退出码 0，输出仅含仓库既有 lint 告警。
- Node 270、OnChina 131 项全量 Rust 测试通过。CitizenApp 689 项通过、5 项环境性
  跳过；CitizenWallet 166 项通过；两端 `flutter analyze` 均 0 问题。

### 真实节点验收

- 使用当前最终源码重建普通 debug node/runtime WASM，导出 fresh chainspec，清空
  bootnodes 和 telemetry 后在独立 `--tmp` 数据目录启动无挖矿节点。
- 首次临时启动时发现 fresh chainspec 仍携带默认 bootnode；对端创世不匹配，节点按
  设计立即断开并封禁该 peer。随即停止该实例、清空 bootnodes 后重新执行隔离验收；
  期间未同步区块、未广播或提交交易。
- RPC 返回 `peers=0`、`isSyncing=false`、`shouldHavePeers=false`；创世哈希为
  `0x07cf6b8c9592f8ec79b32b04be65311aa174bad6e5ebdf72765a1da950a2732b`，状态根为
  `0xf7489e817ea96b58578edf86f15b1eacd3f31402f2653fcb63f1502016121a5b`。
- 节点已正常关闭，临时 chainspec 已删除；未挖矿、未提交交易、未使用用户私钥。

### 保留边界

- 不迁移旧 pending storage、不保留旧 payload、不复用已删除 call index；开发期按
  当前 runtime 重新创世。
- 第 1 步真实管理员扫码签名的环境限制仍存在，不得用伪造私钥、管理员代付或兼容
  路径替代。

## 第 5 步创世准备与五端验收记录（2026-07-16）

### 创世准备工具链

- `bake-chainspec.sh` 删除外部公权机构 root 输入，统一从同一临时节点的块 0 生成
  plain spec、App 轻节点 chainspec、light sync checkpoint、43 个公权机构分片、
  Cloudflare 链身份配置和 genesis-state manifest，避免第二真源。
- 状态包清单新增 `artifact_stage=preview/release`：本地不带 `--finalize` 只能生成
  preview 且 CI provenance 留空；正式 `--finalize` 必须绑定 CI WASM、run id 和 commit。
- `prepack.sh` / `prepack.ps1` 在构建前拒绝 preview、缺失 release provenance 或
  chainspec hash 不一致的状态包；本次已真实验证 preview 被失败关闭。
- `check-chainspec-frozen.sh` 支持显式 staged 路径验收，同时继续默认校验仓库当前正式
  资产；node/App/checkpoint/43 分片/Cloudflare 锚点必须全部一致。
- Cloudflare bootstrap 删除默认 genesis/state root，缺失、非 32 字节或非 hex 时直接
  失败，不允许回落到旧冻结值。
- 公权机构 bundle 和宪法检查脚本补精确 SCALE 解码、自测与尾随字节拒绝；机构存在即
  active，不再读取已删除的机构 lifecycle/status 尾字段。

### Preview 候选（非冻结值）

- `artifact_stage=preview`，只保存在忽略目录 `citizenchain/target/chainspec/`；没有运行
  `--finalize`，没有覆盖正式 node chainspec、CitizenApp 资产、公权机构分片或
  Cloudflare `wrangler.toml`。
- `genesis_hash=0x8347f61bd28c93c4ce6d6b98f4b5a70f185841e0ac87b0bab9eb8c6caf8375ed`。
- `state_root=0x467996c0094900833e30ff0a11e668aaf234abc35acdb4917f858702642ee707`。
- `runtime_wasm_hash=c5333afdf66c5d60f58d9101c2dc49a50885773c7708dace7d64fd5f7a1079b5`。
- `chainspec_hash=0cfe7fa42d4afc34987c69357f593748ee6f4fc9d388378744ad2fa32c67ea8b`，
  `light_sync_state_hash=7caa134d4af22be0d214b383c0d0c6b8df995f5da0fcf2e2e63a8c8284034c92`。
- 43 个省级分片共 49,593 个机构，公权目录根为
  `ecff487ce7d2bac6cb89d064a456187b453acd27f4bee2b140f474a48d072682`；
  49,549 个普通机构各 2 个协议账户，43 个省储行各 3 个，国家储委会 4 个。
- 宪法 `law_id=0`、v1 生效版、不可变条款和 runtime `:code` 校验通过；物化耗时 50 秒。

### 真实运行态验收

- preview 状态包复制到仓库外临时目录后启动真实 node；RPC 返回同一块 0/state root、
  `CitizenChain`、`isSyncing=false`、`peers=0`，NodeGuard 未拒绝。
- 使用全新临时内嵌 PostgreSQL 启动当前源码重新构建的 OnChina；链上投影写入
  49,593 个机构和 99,231 个机构账户，33 项创世机构抽样对账通过。
- OnChina `/api/v1/health` 返回 `UP`；中枢省目录版本绑定候选 genesis/block#0，计数
  648；真实生产前端首页 HTTP 200 并显示“链上中国平台”。
- node、OnChina、PostgreSQL 均已停止，临时端口与仓库外目录已清理；未挖矿、未提交
  交易、未读取或伪造任何用户私钥。

### 自动验收结果

- Rust 格式检查通过；`cargo check -p citizenchain --features runtime-benchmarks` 通过；
  `cargo clippy --workspace --lib --bins` 退出码 0，仅有仓库既有 lint 告警。
- `citizen-issuance` 集成测试已删除旧 `u64` 注册局账户标识夹具，统一使用注册局机构
  CID 作为 `actor_cid_number`、管理员账户 100 作为外层签名 origin；定向 5 项测试通过。
- `cargo test --workspace --all-targets` 不再排除任何 crate，完整退出码 0；包括 runtime
  43、node 270、OnChina 131、投票引擎 95，以及全部费用、机构账户和跨模块集成测试。
- CitizenWallet 166 项测试和 analyze 全绿。CitizenApp Cloudflare 23 个文件共 168 项
  测试、typecheck 全绿；CitizenApp 除工作区另一个未提交且持续挂起的群聊页面测试外，
  其余 692 项通过、5 项环境性跳过，analyze 无问题。
- node 与 OnChina 前端 production build 通过；OnChina 后端已按当前源码重新构建并完成
  上述真实运行态验收。
- 已删除业务模块手工清理入口、公开预备快照 call、中转 storage 和钱包残留动作的精确
  旧名称；定向残留扫描为 0。

### 明确延后与阻塞项

- 按用户要求，本步不运行 GitHub CI、不执行 `--finalize`、不冻结新锚点、不打正式包、
  不发布或启动正式新创世；后续顺序固定为 CI WASM → release freeze → 软件 CI → 正式创世。
- 真实管理员扫码签名、机构费用账户真实扣 0.1 元、余额不足失败和真实投票扣 1 元仍需
  用户在本机解锁专用测试管理员/公民钱包；不得用临时管理员、假签名或回落付款替代。
