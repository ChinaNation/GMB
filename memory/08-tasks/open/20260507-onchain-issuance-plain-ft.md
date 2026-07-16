# 任务卡:链上发行代币功能(onchain-issuance Plain FT)框架搭建

- 创建日期:2026-05-07
- 入口:GMB Claude 主聊天入口(本会话)
- 关联 ADR:
  - ADR-010(AdminAccountKind 协议规范)— 本任务追加 0x04 = asset_id 资产编号 段
  - ADR-011(onchain-issuance Plain FT 协议位 + 监管六条 + 计费)— 本任务新建

## 任务需求

在区块链中引入「除 GMB 主币外其他人发行代币」的功能。

发行人范围:**CID 注册机构 + personal-manage 注册个人多签**(裸账户禁止)。
代币种类:**第一期只 FT(同质化代币),Plain 类(无锚定,Pegged 留 Phase 2)**。
治理铁律:**GMB 是唯一计费/治理代币**,所有用户代币操作只收 GMB。

## 所属模块

- citizenchain runtime(主战场)
- primitives(协议位扩展)
- citizenapp(用户钱包资产视图)
- citizenwallet(冷钱包 QR decoder)

不涉及:node/frontend(矿工端,`feedback_desktop_is_miner`)、cid-frontend(Phase 1 不进监管入口)、chainspec.json(由用户单独处理重新创世)。

## 输入文档

- memory/04-decisions/ADR-010-subject-id-protocol.md(AccountId 协议)
- memory/CLAUDE.md / memory/AGENTS.md / memory/07-ai/chat-protocol.md
- memory/MEMORY.md 中已锁定的相关 feedback / project 条目:
  - feedback_no_compatibility(死规则:绝不搞兼容/保留/过渡)
  - feedback_pubkey_format_rule(内部 0x 小写 hex / 前端 SS58)
  - feedback_extrinsic_submit_must_watch(submit-only)
  - feedback_user_naming_literal(命名字面照抄)
  - feedback_desktop_is_miner(桌面端是矿工)
  - project_unified_voting_entry / project_unified_voting_entry_phase4(业务 pallet 不暴露 wrapper extrinsic,前端直调 VotingEngine)
  - project_qr_signing_two_color(冷钱包两色识别铁律,禁止盲签)
  - project_account_id_protocol_2026_05_06(永久 ABI 锁定)
  - project_pallet_tests_restructured_2026_05_07(tests 搬入 src/tests/{mod,cases}.rs 样板)
  - project_fee_policy_unified(全链费率单一权威源)

## 必须遵守

- 不可突破模块边界(链端 / citizenapp / citizenwallet 三段各管各)
- 不可绕过既有契约(VotingEngine 单一入口、AccountId 协议、fee_policy 权威源、QR 两色识别)
- 不可擅自修改安全红线(BaseCallFilter 屏蔽 pallet_assets 原生 extrinsic 不可松开)
- pallet_assets 仅作内核,对外只能通过 OnchainIssuance pallet 包装入口暴露
- decimals 区间强制 `0..=18`,链端硬校验(用户已确认)
- monitor 主体强制 NRC(国家级清算行 / 国家储委会),链端在 GenesisConfig 写入,extrinsic 入参不接受
- 不设资产创建押金或专属创建费；公开发行 call 在业务落地前保持 `FeeRoute::Reject`，落地后必须进入全链五类费用协议
- 不写 storage migration(用户单独处理重新创世)
- 不动 chainspec.json(用户单独处理)
- 不清楚逻辑时先沟通(本会话已在第 1~7 轮完成需求对齐)

## 决议汇总(7 轮对话已确认)

| 项 | 决议 |
|---|---|
| 谁能发 | CID 机构 + personal-manage |
| 资产种类 | 第一期 FT only,Plain only |
| GMB 唯一性 | 计费 / 治理 / gas 统一 GMB,用户代币禁用 |
| decimals | 用户自定义,范围 0..=18 |
| 创建费 | 无专属费用或押金；机构发起业务操作按五类协议由 actor CID 费用账户支付链上操作费 |
| mint/transfer 计费 | 机构发起操作由 actor CID 费用账户支付，实际 `cast` 由投票签名者支付 1 元 |
| burn / monitor 监管动作 | 外层机构操作收费；仅 Root/投票引擎内部回调免费 |
| 业务审批 | 多签内部执行(InternalVote 收 admin 签名),无外部审批层 |
| 监管动作 | JointVote(管理员多签 + 全民兜底),NRC 主体调用 |
| monitor 主体 | NRC |
| 字符串黑名单 | 法币词 / 锚定词 / 权威词 / 数字货币词 |
| 资产关闭 | 发行方 InternalVote 关闭,余额销毁不退 GMB |
| 强制封禁 | NRC 走 JointVote,30 天冻结期后销毁 |
| spec_version | 不加(随重新创世带过去) |
| migration | 不写 |

## 输出物(本任务卡覆盖范围)

### 链端

- `citizenchain/runtime/issuance/onchain-issuance/` 新建 crate(Cargo.toml + 11 个 src 文件骨架)
- `citizenchain/runtime/primitives/src/derive.rs` 加 `asset_id 资产编号 = 0x04` + parse 分支 + helper
- `citizenchain/runtime/primitives/src/fee_policy.rs` 只复用全链 `FeeRoute`、`ONCHAIN_MIN_FEE` 与统一收费执行器，不增加资产专属费用常量
- `citizenchain/Cargo.toml` workspace.members 加 `runtime/issuance/onchain-issuance`
- `citizenchain/Cargo.toml` workspace.dependencies 加 `pallet-assets`
- `citizenchain/runtime/Cargo.toml` 加 `onchain-issuance` + `pallet-assets` 依赖与 features 传播
- `citizenchain/runtime/src/lib.rs` `construct_runtime` 加 `Assets`(pallet_index=26)+ `OnchainIssuance`(pallet_index=25)
- `citizenchain/runtime/src/configs.rs` 配 `pallet_assets::Config` + `onchain_issuance::Config`；公开占位 call 在真正创建投票和执行资产业务前由 `RuntimeFeeRouter` 显式 `Reject`，`RuntimeCallFilter` 屏蔽 `pallet-assets` 原生 extrinsic。
- `citizenchain/runtime/src/genesis_config_presets.rs` 注入 OnchainIssuance GenesisConfig(黑名单初始词表 + decimals 边界 + NRC subject)

### 客户端

- `citizenapp/lib/asset/` 新建模块(shared / entity / pages / widgets 骨架文件)
- `citizenwallet/lib/qr/bodies/onchain_asset_*_body.dart` 10 个文件骨架

### 文档

- `memory/04-decisions/ADR-011-onchain-issuance-plain-ft.md` 新建
- `memory/04-decisions/ADR-010-subject-id-protocol.md` 追加 0x04 段
- `memory/MEMORY.md` 加 project_onchain_issuance_phase1 索引行
- `memory/08-tasks/index.md` 加本任务卡索引行

## 验收标准(本框架阶段)

- 链端 `cargo check -p onchain-issuance` 通过
- 链端 `cargo check -p citizenchain` 通过
- workspace `cargo check --workspace` 不引入新警告
- citizenapp / citizenwallet Dart 骨架文件 `flutter analyze` 不报错(允许 unused warning)
- ADR-011 落盘,关键决议齐全
- ADR-010 增量段写明 0x04 协议位
- MEMORY.md / 08-tasks/index.md 索引同步

## 残留清理

- 不留 `// TODO: implement business logic` 之外的 TODO
- 所有占位文件必须有中文模块说明
- 不引入未使用的 import / dependency

## 后续(超出本任务卡)

业务逻辑实装(extrinsic / 投票回调 / 黑名单数据 / 测试 / benchmarks)拆为后续任务卡:

- 子任务 A:onchain-issuance pallet 业务实装(issue/mint/burn/close/transfer)
- 子任务 B:onchain-issuance pallet 监管实装(NRC freeze/unfreeze/confiscate/forceTransfer/forceClose)
- 子任务 C:citizenapp 资产视图业务实装
- 子任务 D:citizenwallet 公民钱包 QR decoder 10 个 ACTION 解码逻辑
- 子任务 E:端到端联调验证(发行 / 转账 / 监管 / 关闭)

---

## v2 修订记录(2026-05-07,review 后)

本会话内 review 识别 15 项设计漏洞 / 残留 / 遗漏(6 🔴 严重 + 5 🟡 中等 + 4 🟢 轻微),用户决议「一起修复」,全部纳入 v2:

| # | 类别 | 修订 | 影响文件 |
|---|---|---|---|
| 1 | ✅ | 发行机构身份改为 `actor_cid_number`，具体资产执行账户改为 `execution_account`；费用账户由后续统一费用路由按 CID 解析 | lib.rs / configs.rs |
| 2 | 🔴 | propose origin 校验铁律写入 ADR-011 5.4 / 5.6 节 + proposal.rs doc + Error::ProposeOriginNotAllowed | ADR-011 / lib.rs / proposal.rs |
| 3 | ✅ | 删除资产创建押金、专属收费存储、文件和事件；不得恢复专属收费真源 | fee_policy.rs / lib.rs |
| 4 | ✅ | 计费表订正：机构发起操作由 actor CID 费用账户支付链上操作费，只有管理员实际 `cast` 才由签名者支付 1 元 | fee_policy.rs / configs.rs |
| 5 | 🔴 | ForceCloseSchedule storage(BlockNumber → Vec<asset_id>) + on_finalize O(1) take + MaxScheduledPerBlock | ADR-011 5.6 / 8 节 / lib.rs / monitor.rs |
| 6 | 🔴 | onchain-issuance/Cargo.toml `try-runtime` feature 传播给 frame/balances/assets/votingengine | onchain-issuance/Cargo.toml |
| 7 | 🟡 | AccountId 0x04 payload 简化:8B+4B+35B → 4B+43B(去 issuer_subject_short) | ADR-010 / ADR-011 2 节 / derive.rs / citizenapp codec |
| 8 | 🟡 | 哈希算法依赖随 #7 自动消失 | derive.rs |
| 9 | 🟡 | OnchainAssetMeta 去 `monitor_account_id` 字段(NRC 全局,非每条) | types.rs / citizenapp query |
| 10 | 🟡 | OnchainAssetMeta 去 `asset_id` 字段(AccountId byte[1..5] 即可反推),保留 AssetIdIndex | types.rs |
| 11 | 🟡 | metadata 永久不可改铁律写入 ADR-011 5.7 节 + Error::MetadataImmutable | ADR-011 / lib.rs |
| 12 | ✅ | 删除未实现的资产专属收费实现，收费统一进入全链执行器 | lib.rs / fee_policy.rs |
| 13 | 🟢 | onchain-issuance vs onchain-transaction 命名说明加在 ADR-011 顶部 | ADR-011 |
| 14 | 🟢 | pallet_assets Freezer/Holder=() 注解 | ADR-011 8 节 |
| 15 | 🟢 | OnchainIssuance::Assets ↔ pallet_assets::Asset 双轨同步铁律 | ADR-011 8.1 节 |

### v2 验收

- `cargo check -p citizenchain` 0 警告(WASM_FILE=/tmp/stub.wasm)
- `cargo test -p primitives --lib derive` 11/11 ok(含 4 个新 onchain_asset 测试)
- `cargo test -p onchain-issuance --lib` 8/8 ok
- `flutter analyze lib/asset/`(citizenapp)0 issue
- `flutter analyze lib/qr/bodies/`(citizenwallet)0 issue

### 顺手修复(超出 review 15 项,但属测试基础设施)

- `primitives/derive.rs` 既存测试 bug 2 处:
  - `cid_number_starts_with_0x02`:错调 `account_id_from_cid_number(&str)`,改为 `account_id_from_registered_cid_number(&[u8])`
  - `builtin_id_starts_with_0x01`:硬编码 `id[1..34]` 与 29B 实际字符串长度不匹配,改用动态 `n = cid_number.len()`
- 这 2 处既存 bug 阻塞 `cargo test -p primitives`,与 v2 修订无直接关系但作为验证基础设施顺手修;原本派出的 spawn_task chip(修复同一 bug)可作废,用户可 dismiss

---

## v3 修订记录(2026-05-07,模块编号 / call 同步对齐)

用户问"模块编号和 call 都同步了吗?区块链和 citizenwallet 公民钱包等端"时识别出 v2 严重对齐错位:

**根本错误**:v2 把 unified_voting_entry phase 4 铁律("业务 pallet wrapper extrinsic 全删")扩展过头,误把 propose_X 也归入"不暴露",与 GMB 现有架构(duoqian-transfer / personal-manage / organization-manage 等业务 pallet **都暴露 propose_X extrinsic**)严重背离。phase 4 实际删除的只是 execute/cancel wrapper(由 VotingEngine 9.4/9.5 统一承载),不是 propose_X。

**连锁错误**:citizenwallet 把 10 个 ACTION 实现为 `lib/qr/bodies/onchain_asset_*_body.dart` QR envelope 顶层 body,与 citizenwallet "sign_request envelope 中 payload_hex 走 SCALE RuntimeCall 解码" 机制完全脱节(应该在 `payload_decoder.dart` 加 OnchainIssuance(25) 路由分支)。

| # | v3 修订 | 文件 |
|---|---|---|
| 1 | onchain-issuance lib.rs `#[pallet::call]` 实装 10 个 propose_X extrinsic 框架(call_index 0..=4 业务 / 10..=14 监管),不再为空 | [lib.rs](citizenchain/runtime/issuance/onchain-issuance/src/lib.rs) |
| 2 | `configs.rs` 保持 `OnchainIssuance` 占位 call 为 `Reject`；实装后必须按 actor CID 费用账户支付链上操作费，实际 `cast` 才由投票签名者支付投票费 | [configs.rs](citizenchain/runtime/src/configs.rs) |
| 3 | RuntimeCallFilter:OnchainIssuance 走默认 true(propose_X 是合法入口),Assets 仍全 reject | [configs/mod.rs](citizenchain/runtime/src/configs/mod.rs) |
| 4 | citizenwallet `pallet_registry.dart` 加 `onchainIssuancePallet = 25` + 10 个 call_index 常量 | [citizenwallet/lib/signer/pallet_registry.dart](citizenwallet/lib/signer/pallet_registry.dart) |
| 5 | citizenwallet `payload_decoder.dart` 加 OnchainIssuance(25) 路由分支 + 10 个 `_decodeOnchainAssetPlaceholder` 占位(框架阶段返回 action/summary,业务字段解码任务卡 D 实装) | [citizenwallet/lib/signer/payload_decoder.dart](citizenwallet/lib/signer/payload_decoder.dart) |
| 6 | **删除** `citizenwallet/lib/qr/bodies/onchain_asset_*_body.dart` 10 个错位文件 | citizenwallet/lib/qr/bodies/ |
| 7 | citizenapp `onchain_asset_constants.dart` 加 pallet_index / 10 个 call_index 常量 | [citizenapp/lib/asset/shared/onchain_asset_constants.dart](citizenapp/lib/asset/shared/onchain_asset_constants.dart) |
| 8 | ADR-011 v3:第 5.4 节订正 propose_X 暴露铁律 + 第 6 节计费表订正 + 第 9 节加 call_index 分配表 + 第 10 节加 citizenwallet 路由要求 | [ADR-011](memory/04-decisions/ADR-011-onchain-issuance-plain-ft.md) |

### v3 验收

- `WASM_FILE=/tmp/stub.wasm cargo check -p citizenchain` 0 警告
- `cargo test -p onchain-issuance --lib` **8/8** ok
- `cargo test -p primitives --lib derive` **11/11** ok
- `flutter analyze lib/asset/`(citizenapp)0 issue
- `flutter analyze lib/signer/ lib/qr/`(citizenwallet)0 issue
- `flutter test`(citizenwallet 完整套件)**96/96 全过**(含 PalletRegistry 索引唯一性测试,确认 onchainIssuancePallet=25 不冲突)

### 模块编号 / call 同步矩阵

| 角色 | 同步项 | 状态 |
|---|---|---|
| 链端 construct_runtime | OnchainIssuance idx=25 / Assets idx=26 | ✅ |
| 链端 RuntimeCall | OnchainIssuance 10 个 propose_X(0..=4 / 10..=14)| ✅ |
| 链端费用路由 | `OnchainIssuance(_)` 占位 call → `Reject`，禁止扣费后无业务结果 | ✅ |
| 链端 RuntimeCallFilter | Assets 全 reject / OnchainIssuance 默认通过 | ✅ |
| citizenwallet PalletRegistry | onchainIssuancePallet=25 + 10 call_index | ✅ |
| citizenwallet payload_decoder | OnchainIssuance(25) 路由 + 10 占位解码器 | ✅ |
| citizenapp constants | onchainIssuancePalletIndex=25 + 10 call 常量 | ✅ |
| ADR-011 v3 | 9 / 10 节完整 call_index 分配表 + 客户端硬编码同步要求 | ✅ |
