---
title: 宪法机构主账户字段改名 duoqian_address → main_address
status: done
owner: Blockchain Agent
created: 2026-04-20
completed: 2026-04-20
---

# 执行结果（2026-04-20）

- primitives/china/ 8 文件完成 `duoqian_address → main_address` + `CHINA_RESERVED_DUOQIAN_ADDRESSES → CHINA_RESERVED_MAIN_ADDRESSES` + `is_reserved_duoqian_address → is_reserved_main_address` + 测试函数 `all_china_ch_duoqian_addresses_are_unique → all_china_ch_main_addresses_are_unique`
- runtime 调用方：configs/mod.rs（选择性保留 duoqian_manage_pow::propose_close pattern bindings）、genesis_config_presets.rs、runtime/src/lib.rs、resolution-issuance-gov、resolution-destro、shengbank-stake-interest、offchain-transaction-pos、onchain-transaction-pow、duoqian-transfer-pow、institution-asset-guard 全部更新
- 节点 UI：node/src/ui/governance/ mod.rs + types.rs + signing.rs；node/frontend/governance/ 4 份 TS 文件 `duoqianAddress → mainAddress`
- 文档：11 份 memory/05-modules 技术文档 + GMB_WHITEPAPER + 3 份任务卡注释更新
- 工具：tools/duoqian.py 扫描正则、生成模板同步；Rust 引用函数名 `derive_duoqian_address_from_sfid_id` 保留
- 验证：`cargo check -p primitives ...` 9 crate 全通过；`cargo test -p primitives` 7/7 测试通过（含 `all_china_ch_main_addresses_are_unique`）；node frontend `tsc --noEmit` 零错误

# 保留清单（注册多签概念）

- duoqian-manage-pow 全部（`action.duoqian_address`、`DuoqianAccounts`、`PersonalDuoqianInfo`、`CreateDuoqianAction`、`DuoqianAddressValidator`、`derive_duoqian_address_from_sfid_id`）
- SFID 后端 `MultisigAccount.duoqian_address`、SFID 前端 `deriveDuoqianAddress.ts`
- wuminapp/wumin 的 `duoqianAddress` Dart 字段
- tools/duoqian.py 文件名

# 背景

`primitives/china/` 下 8 个宪法机构常量文件的 `duoqian_address` 字段语义错位：这个字段代表的是机构的**主账户**（与 `fee_address` 费用账户同级），但字面却叫"duoqian（多签）"，而"多签"在本系统里是**更大的范畴**（包含主账户、费用账户、注册机构多签、注册个人多签四类都是多签账户）。

为消除歧义，将宪法机构常量及其直接读取方的 `duoqian_address` 统一改名为 `main_address`。注册多签概念（duoqian-manage-pow / SFID 注册机构 / wuminapp 注册机构）保持 `duoqian_address` 不动。

# 铁律

- 字节值不变，仅字段名变，不触 chainspec 冻结、不需要链重启
- 注册多签（`duoqian-manage-pow::action.duoqian_address`、`DuoqianAccounts`、`PersonalDuoqianInfo`、`DuoqianAddressValidator`、`CreateDuoqianAction`、`derive_duoqian_address_from_sfid_id`）全部保留
- SFID 后端 `MultisigAccount.duoqian_address`、前端 `deriveDuoqianAddress.ts` 保留
- wuminapp `duoqianAddress` 字段保留；只改指向 primitives 的注释
- wumin `payload_decoder.dart`、`tools/duoqian.py` 文件名保留；工具产出模板同步更新

# 执行清单

## 第 1 步：primitives/china/ 源头（8 文件）

- [ ] china_cb.rs / china_ch.rs / china_zf.rs / china_jc.rs / china_lf.rs / china_sf.rs / china_jy.rs：字段 `duoqian_address` → `main_address`
- [ ] china_zb.rs：常量 `CHINA_RESERVED_DUOQIAN_ADDRESSES` → `CHINA_RESERVED_MAIN_ADDRESSES`、函数 `is_reserved_duoqian_address` → `is_reserved_main_address`、顶部注释
- [ ] china_ch.rs：测试函数 `all_china_ch_duoqian_addresses_are_unique` → `all_china_ch_main_addresses_are_unique`
- [ ] china_cb.rs：顶部注释"duoqian_address 和 fee_address" → "main_address 和 fee_address"

## 第 2 步：Rust runtime 直接读取方

- [ ] `runtime/src/configs/mod.rs`（选择性：保留 duoqian_manage_pow pallet call 参数 `duoqian_address`；rename `n.duoqian_address` + `is_reserved_duoqian_account` 包装 + 注释中的 const 名）
- [ ] `runtime/src/genesis_config_presets.rs`（全量）
- [ ] `runtime/src/lib.rs`（2 条注释）
- [ ] `runtime/governance/resolution-issuance-gov/` lib.rs + benchmarks.rs（全量）
- [ ] `runtime/governance/resolution-destro/src/lib.rs`（全量）
- [ ] `runtime/issuance/shengbank-stake-interest/src/lib.rs`（全量）
- [ ] `runtime/transaction/offchain-transaction-pos/src/lib.rs`（1 处）
- [ ] `runtime/transaction/onchain-transaction-pow/src/lib.rs` + benches（全量）
- [ ] `runtime/transaction/duoqian-transfer-pow/src/lib.rs`（选择性：rename primitives 访问，保留 `DuoqianAddressValidator` trait 使用）
- [ ] `runtime/transaction/institution-asset-guard/src/lib.rs`（2 条注释）

## 第 3 步：节点 UI（宪法机构展示）

- [ ] `node/src/ui/governance/mod.rs`（InstitutionEntry + 90 条硬编码数据）
- [ ] `node/src/ui/governance/types.rs`（InstitutionDetail / InstitutionListItem）
- [ ] `node/src/ui/governance/signing.rs`
- [ ] `node/frontend/governance/governance-types.ts` + InstitutionDetailPage.tsx + CreateProposalPage.tsx + GovernanceSection.tsx（TS 侧 `duoqianAddress` → `mainAddress`）

## 第 4 步：文档/注释

- [ ] memory/05-modules/citizenchain/runtime/primitives/PRIMITIVES_TECHNICAL.md、BLAKE2_ADDRESS_DERIVATION.md
- [ ] memory/05-modules/citizenchain/runtime/issuance/shengbank-stake-interest/SHENGBANK_TECHNICAL.md
- [ ] memory/05-modules/citizenchain/runtime/governance/resolution-destro/RESOLUTIONDESTRO_TECHNICAL.md
- [ ] memory/05-modules/citizenchain/runtime/governance/voting-engine/VOTINGENGINE_TECHNICAL.md
- [ ] memory/05-modules/citizenchain/runtime/transaction/institution-asset-guard/INSTITUTION_ASSET_GUARD_TECHNICAL.md
- [ ] memory/05-modules/citizenchain/runtime/transaction/duoqian-transfer-pow/DUOQIAN_TRANSFER_TECHNICAL.md + SWEEP_TECHNICAL.md（指向 primitives 的段落）
- [ ] memory/00-vision/GMB_WHITEPAPER.md
- [ ] wuminapp 若干 .dart 注释行（"primitives 中的 duoqian_address 字段"）
- [ ] `tools/duoqian.py`：更新扫描正则 + 输出模板（scan `main_address:`，emit `CHINA_RESERVED_MAIN_ADDRESSES` / `is_reserved_main_address`）

## 第 5 步：验证

- [ ] `cargo check -p primitives -p gmb-runtime -p gmb-node`（或相应 workspace 命令）
- [ ] `cargo test -p primitives -- all_china_ch_main_addresses_are_unique`
- [ ] 前端 `tsc --noEmit`（node/frontend）
- [ ] 回写索引 memory/08-tasks/index.md，迁移至 done/

# 验收

全仓库 `rg "duoqian_address|CHINA_RESERVED_DUOQIAN_ADDRESSES|is_reserved_duoqian"` 仅保留：注册多签 pallet、SFID、wuminapp、wumin、tools/duoqian.py 内部残留（符合边界设计）。
