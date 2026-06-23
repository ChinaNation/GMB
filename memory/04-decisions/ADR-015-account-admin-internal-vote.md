# ADR-015 账户级内部投票管理员模型

- 状态:Accepted
- 决议日期:2026-05-07
- 关联前置:ADR-010(AccountId 协议)、ADR-011(onchain-issuance Plain FT,预留 0x04)
- 关联任务卡:`memory/08-tasks/open/20260507-173627-账户级内部投票管理员模型第0步设计-所有可操作账户使用内部投票-质押账户永不可操作-治理机构账户共.md`

## 背景

原内部投票模型以“机构主体”为主要治理对象。新需求要求所有可操作账户都使用内部投票,且注册个人账户、注册机构账户、公权机构账户、其他机构账户都必须支持账户级管理员集合。

链上资金与权限最终落到账户,机构只是账户的归属分组。为了避免机构级管理员池与账户级子集形成双重真源,本 ADR 将内部投票治理对象收敛为“账户治理主体”。

## 决议

### 1. 账户范围

“账户”指省储行永久质押账户之外的全部机构账户:

- 主账户
- 费用账户
- 安全基金账户
- 用户自定义账户
- 其他后续明确可操作的机构账户

省储行永久质押账户永远不可操作:

- 不进入内部投票主体
- 不允许支出
- 不允许注销
- 不允许管理员治理
- 不允许作为业务提案资金源

### 2. 治理机构账户

治理机构包括国储会、省储会、省储行等内置治理主体。

- 治理机构所有可操作账户共享同一套管理员。
- 治理机构管理员数量固定。
- 治理机构阈值固定。
- 治理机构只允许等长更换管理员。
- 治理机构不允许增加管理员、删除管理员或修改阈值。
- 治理机构内置主体永远不能关闭。

当前固定规则继续沿用:

| 主体 | 管理员数量 | 阈值 |
|---|---:|---:|
| 国储会(NRC) | 19 | 13 |
| 省储会(PRC) | 9 | 6 |
| 省储行(PRB) | 9 | 6 |

### 3. 注册个人账户

注册个人账户只有一个账户,该账户独立持有管理员集合。

- 最少管理员数量:2
- 最多管理员数量:64
- 注册创建必须由拟定管理员全员投票通过。
- 注销关闭必须由当前管理员全员投票通过。
- 普通业务提案按动态阈值通过。
- 管理员集合变更由当前管理员按动态阈值通过。

### 4. 注册机构账户

注册机构、公权机构注册账户、其他机构注册账户统一按账户级治理。

- 一个机构可以下挂多个账户。
- 每个账户独立持有管理员集合。
- 每个账户只由自己的管理员管理。
- 主账户不管理其他账户。
- 任一账户注册创建必须由该账户拟定管理员全员投票通过。
- 任一账户注销关闭必须由该账户当前管理员全员投票通过。
- 注册机构账户最少管理员数量:2
- 注册机构账户最多管理员数量:1989

### 5. 动态阈值规则

注册个人账户和注册机构账户不再由用户输入阈值。阈值由管理员数量自动派生:

```text
admins_len == 2: threshold = 2
admins_len >= 3: threshold = ceil(admins_len / 2)
```

链端必须拒绝以下情况:

- 管理员数量小于 2
- 注册个人账户管理员数量大于 64
- 注册机构账户管理员数量大于 1989
- 管理员列表有重复
- 2 个管理员时继续删除管理员
- 管理员集合变更后与旧集合完全一致

### 6. 管理员集合变更提案

动态账户只保留一个管理员集合变更提案。

提案输入目标管理员集合,链端对比旧集合与新集合,自行识别增加、删除、更换或组合变更。阈值不作为用户输入字段,永远按动态阈值规则计算。

管理员集合变更执行前:

- 当前账户必须为 Active。
- 发起人必须是当前账户管理员。
- 投票主体为当前账户。
- 投票阈值使用当前账户管理员数量派生出的当前阈值。
- 同一账户的管理员集合变更提案必须与普通活跃提案互斥。

管理员集合变更执行后:

- 写入新管理员集合。
- 新阈值按新管理员数量派生。
- `updated_at` 更新。

### 7. 创建和注销全员规则

账户注册创建和注销关闭是生命周期操作,不使用普通动态阈值:

- 创建账户:阈值 = 拟定管理员数量。
- 注销账户:阈值 = 当前管理员数量。

这样在“一人一票一笔交易”模型下,创建和注销等价于全部管理员逐票上链通过。

### 8. 账户治理主体 ID

实现阶段必须保证每个可操作账户能映射到唯一内部投票主体。

推荐映射:

- 治理机构账户:继续映射到现有 `AdminAccountKind::Builtin` 主体,同一治理机构所有账户共享一个主体。
- 注册个人账户:继续映射到现有 `AdminAccountKind::PersonalAccount` 主体。
- 注册机构账户:新增账户级主体类型,推荐 `AdminAccountKind::InstitutionAccount = 0x05`,payload 使用账户 `AccountId` 的 32 字节值并右填零。

说明:

- `0x04` 已由 ADR-011 预留给 `asset_id 资产编号`。
- 本 ADR 只确定业务模型;执行阶段必须同步更新 ADR-010、`unified-protocols.md`、runtime primitive、citizenapp 和 citizenwallet 解码契约。

## 影响

- `admins-change` 从“机构管理员真源”升级为“账户管理员真源”。
- `internal-vote` 继续保持一人一票一笔交易,但创建提案时读取账户级管理员快照与阈值快照。
- `personal-manage` 的 `threshold` 输入应被移除或忽略,改由链端派生。
- `organization-manage` 必须为每个可操作账户创建独立管理员主体;机构只作为账户归属分组。
- `multisig-transfer` 等业务模块必须绑定具体账户治理主体。
- `citizenapp` 需要按账户展示管理员、阈值、待投提案和投票进度。
- `citizenwallet` 公民钱包需要按账户级 AccountId 展示待签交易。

## 备选方案

### 机构统一管理员池 + 账户子集

放弃。该方案会产生机构管理员池和账户管理员子集两层真源,管理员增删、阈值变化和账户注销时容易出现权限漂移。

### 每个账户完全独立管理员主体

采用。该方案与链上“账户是资金与权限落点”的事实一致,边界最清晰。

## 后续动作

1. 第 1 步改造 `admins-change`:账户级主体、动态阈值、管理员集合变更提案。已于 2026-05-08 完成。
2. 第 2 步改造 `internal-vote`:账户级快照、全员生命周期阈值、动态普通阈值。已于 2026-05-08 完成。
3. 第 3 步改造 `personal-manage`:注册个人账户管理员上限 64,阈值链端派生。已于 2026-05-08 完成。
4. 第 4 步改造 `organization-manage`:注册机构账户管理员上限 1989,每账户独立主体。
5. 第 5 步改造 `multisig-transfer`、`citizenapp`、`citizenwallet`。已于 2026-05-08 先行完成转账链路接入；第 4 步仍需随后补齐机构管理创建/注销全流程。

## 第 1 步执行结果

2026-05-08 已完成：

- `core_const::AdminAccountKind` 新增 `InstitutionAccount = 0x05`。
- `account_id_from_institution_account(account)` 已落地，payload 为账户 `AccountId` 前 32 字节并右填零。
- `admins-change` 新增统一动态阈值工具：`dynamic_threshold` / `derived_threshold`。
- `admins-change` 增加 `MaxPersonalAccountAdmins` 配置项，runtime 设置为 64。
- `admins-change::MaxAdminsPerInstitution` runtime 设置为 1989，用作注册机构账户管理员上限和物理 `BoundedVec` 上限。
- 旧 `propose_admin_replacement` 已替换为 `propose_admin_set_change`。
- 提案数据从 `AdminReplacementAction` 改为 `AdminSetChangeAction<AdminsOf<T>>`。
- `MODULE_TAG` 从 `b"adm-rep-v1"` 改为 `b"adm-set-v1"`。
- `cargo test --manifest-path citizenchain/Cargo.toml -p primitives --lib`：24 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib`：41 passed。

说明：`注册机构归属关系 = 0x02` 继续保留，用于机构归属和检索；转账和端侧发现已切到 `InstitutionAccount = 0x05`，后续第 4 步需要把 `organization-manage` 的新增/注销机构账户全流程也统一到 `0x05`。

## 第 2 步执行结果

2026-05-08 已完成：

- `votingengine::snapshot_institution_admins` 写入 `AdminSnapshot` 前拒绝空管理员列表和重复管理员。
- `internal-vote` 新增 `InvalidThresholdSnapshot`，统一表达阈值与管理员快照人数不匹配。
- Pending 注册创建提案强制全员阈值：`threshold == AdminSnapshot.len()`。
- 显式阈值内部提案强制全员阈值，用于注册账户注销关闭等生命周期操作。
- 普通内部提案和管理员集合变更提案仍使用普通动态阈值，但创建时必须满足 `0 < threshold <= AdminSnapshot.len()`。
- `joint-vote` 单测构建残留已清理：test-only 发起人机构解析补入 `codec::Encode` 导入。
- `cargo test --manifest-path citizenchain/Cargo.toml -p internal-vote --lib`：86 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p votingengine --lib`：0 tests passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p joint-vote --lib`：5 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib`：41 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib`：23 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p organization-manage --lib`：24 passed。

## 第 3 步执行结果

2026-05-08 已完成：

- `personal-manage::propose_create` 已删除 `admins_len / threshold` 入参，创建 call 编码改为 `account_name + admins + amount`。
- 个人账户管理员数量由 `admins.len()` 派生，链端限制 `2..=64`。
- 普通阈值统一调用 `admins-change::derived_threshold` 派生：个人多签使用 `PersonalAccount + 个人多签码（is_personal_code，PMUL）`，机构账户使用 `InstitutionAccount + 机构账户码（is_institution_code）`，`注册机构归属关系` 不再作为管理员主体。（机构分类唯一真源 = CID 机构码 `institution_code`，见 [[ADR-025]]）
- 创建提案内部投票阈值为拟定管理员全员数量；关闭提案仍为当前管理员全员数量。
- `PersonalManage::PersonalAccounts` 不再镜像管理员列表、管理员数量和阈值，只保存 `creator / account_name / created_at / status`。
- `CreateMultisigAction` 不再保存管理员数量和阈值，但保存创建时 `fee` 快照。
- 提案通过后，同一执行事务内完成入金、激活 `admins-change` 主体、激活个人账户。
- `multisig-transfer` 的个人多签管理员查询已从 `admins-change` 读取。
- citizenapp 创建页、提案解码、账户查询和本地快照已切到新格式。
- citizenwallet 公民钱包 payload decoder 已按新格式展示派生日常阈值与创建全员阈值，并拒绝旧 `admins_len + threshold` 载荷。
- 本步骤未修改 `spec_version`。

回归结果：

- `cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib`：23 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib`：41 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p internal-vote --lib`：86 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p multisig-transfer --lib`：20 passed。
- `flutter test test/gmb/account_manage_service_test.dart test/gmb/gmb_storage_codec_test.dart test/gmb/organization_manage_storage_test.dart`：10 passed。
- `flutter test test/signer/payload_decoder_test.dart`：30 passed。

## 第 5 步执行结果

2026-05-08 已完成：

- `multisig-transfer::registered_account` 拆成 `PersonalAccount AccountId` 与 `InstitutionAccount AccountId` 两条账户级路径；`0x02 注册机构归属关系` 明确拒绝作为转账支出主体。
- 个人多签账户状态由 `PersonalQuery::is_active` 校验，机构账户状态由 `InstitutionQuery::is_active` 校验；管理员和阈值仍由投票引擎快照读取 `admins-change::Subjects`。
- citizenapp `institution_data.dart`、`gmb_storage_codec.dart`、`admin_institution_codec.dart` 已支持 `InstitutionAccount AccountId` 编码/解码。
- citizenapp 多签自动发现只把 `PersonalAccount AccountId` 与 `InstitutionAccount AccountId` 落为本地账户；`0x02 注册机构归属关系` 只作归属/检索。
- citizenapp 注册机构账户详情查询改为 `AccountRegisteredCid -> InstitutionAccounts -> AdminsChange::AdminAccounts[0x05]`，不再从 `0x02` 读取账户管理员。
- citizenwallet 公民钱包 `propose_transfer` 只接受 `0x01 / 0x03 / 0x05` 可支出主体，拒绝旧裸 cid 与 `0x02`；QR 展示字段新增 `institution`，显示内置机构名、个人多签短地址或机构账户短地址。
- 本步骤未修改 `spec_version`。

回归结果：

- `cargo test --manifest-path citizenchain/Cargo.toml -p multisig-transfer --lib`：22 passed。
- `flutter test test/gmb test/institution`：49 passed。
- `flutter test test/signer`：61 passed。
