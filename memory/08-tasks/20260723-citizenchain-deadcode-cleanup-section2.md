# CitizenChain 死代码清理（全仓扫描第 2 节）

任务需求：清理 citizenchain 扫描出的编译器静音、QR 协议孤儿字段标签、`GMB_ROLE_V1` 域分隔符与墓碑注释；对「SCALE 字段序占位」「生成物」「活不变量注释」不做删除。
所属模块：citizenchain（node / onchina / runtime / crates/qr-protocol）+ 生成物同步到 citizenapp、citizenwallet + `memory/` 规则文档。
兄弟卡：[20260723-citizenapp-deadcode-cleanup-section3.md](20260723-citizenapp-deadcode-cleanup-section3.md)（同批扫描第 3 节）。

## 复核结论（推翻扫描的两个前提）

1. **QR 字段死活不能靠 grep `.rs` 判定**。decoder 在 Dart 端（`citizenwallet/lib/signer/payload_decoder.dart`，3832 行），链端 `.rs` 根本不消费 `field_key`。扫描列出的 25 条候选里，`valid_range`、`residence`、`birth_place`、`wasm_hash`、`create_threshold`、`governance_detail`、`asset_*`、`amount_raw`、`bank_cid_number`、`article_count`、`chapter_count`、`cid_count`、`fee_payer_description` 等**全是 `actions.yaml` 的活 `required_fields`**，删掉会让 `required_fields_all_have_chinese_labels` 立即失败并让对应动作扫码红拒。真实死字段 **14 条**。
2. **`institution_read/chain.rs` 的 9 处 `#[allow(dead_code)]` 不是问题区**。它们是 `#[derive(Decode)]` 镜像结构的字段占位，与 `constitution/mod.rs` 同类，**删字段会破坏 SCALE 解码对齐**。真正的盲区是 **14 个文件级 `#![allow(dead_code)]` 整片静音**。

## 定稿（用户 2026-07-23 逐条确认）

1. **`registry_org_code` 删除**。live code 零命中；同批改写 AGENTS.md / agent-rules.md / onchina README 三处当前规则载体，收敛为 `institution_code + workspace` 单源；ADR-025 只加「后续变更」指引，不改历史决策正文。
2. **`GMB_ROLE_V1` 删除，全仓只留 `QR_V1`**。用户不接受「派生域不受签名规则约束」的边界主张。替代方案：并入既有 `MODULE_TAG` 惯例（`personal-manage/close.rs:112`、`create.rs:81` 的唯一写法），不造任何新常量。
3. `GMB_CHAT_V1` 同批清掉（仅 4 行注释，从不上线，proto 用 `package gmb.chat.v1` + 数字 `protocol_version`）。
4. runtime 二次确认：**A+B 全部授权**（2026-07-23）。

## 检测方法学修正（写入本卡防复发）

- **QR 字段判据是三层集合差，不是 grep**：`死字段 = fields.yaml(113) − actions.yaml required_fields(95) − payload_decoder.dart 实际发射的 key`。`fields.yaml` 是中文名字典，`actions.yaml` 是「谁用它」，Dart decoder 是「运行时发射谁」。
- **`#[allow(dead_code)]` 必须先分类再动**：生成物（`weights.rs`，benchmark 重跑会写回）/ SCALE 字段序占位（删字段破坏解码）/ 文件级整片静音（真盲区）/ 单点（逐条判定）。四类处置完全不同。
- **墓碑注释判据**：注释约束「未来不能做什么」→ 保留，是防回归不变量；只记录「过去删了什么」→ 删，是残留。`runtime/src/lib.rs:375`（pallet index 32 永久留空）、`primitives/src/sign.rs` 的 op_tag 说明属前者。
- **citizenchain CI 无任何 `cargo test` / `cargo clippy` job**。唯一 Rust 编译门禁是 `citizenchain-ci.yml` 的 `cargo tauri build`（仅 node）与 `citizenchain-wasm.yml` 的 `cargo build`（仅手动触发）。摘除抑制后无自动兜底，判定必须靠本地全量编译。

## 执行进度（2026-07-23）

| Step | 状态 | 说明 |
|---|---|---|
| 1 第 4 项 | ✅ | 删 `governance_skeleton.rs:984` 多余 `#[test]`，文档注释上移 |
| 2 第 5 项 | ✅ | **由并发会话完成**，三处判据与本卡一致（保留活约束句，只删回指过去的句子），本卡不重复动 |
| 3 第 2 项 + 裁定 a | ✅ | 删 14 条 `field_key`（113→99，零误删）；删 2 条 Dart 断言；新增 `fields_yaml_has_no_orphan_entries`；两端生成物重生且逐字节一致；4 处规则文档改写 |
| 4 裁定 b | ✅ | `GMB_ROLE_V1` / `GMB_CHAT_V1` 全仓归零，全仓 `_V1` 只剩 `QR_V1`；派生域改 `MODULE_TAG`；13 处文档同步；新增协议版本标识硬规则 |
| 5 第 1 项 批 2(node) | ✅ | 6 处整片静音全摘，编译器只吐出 1 件事；B 类 9 行收敛为 3 行 + 中文理由 |
| 5 第 1 项 批 3(runtime) | ✅ | 6 处处置完毕：4 删 + 1 收窄 + 1 纯净移除 |
| 5 第 1 项 批 1(onchina) | ✅ | 摘 34 处静音,编译器暴露 84 条;删 3 整文件+约30死项,接线1守卫,保留12组契约字段;onchina 零警告 129 测试绿 |
| 6 门禁 | ✅ | `check-ai-guardrails.sh` 并入 `_V1` 与 `allow` 两条 diff 级门禁 |

### 第 1 项 批 3（runtime，6 处）判定结果

| 位置 | 结局 | 判据 |
|---|---|---|
| `primitives/cid/number.rs:1` 整片静音 | **纯净移除** | 摘除后零警告，原来什么都没藏 |
| `runtime/src/configs.rs:99` `allow(unused_parens)` | **删** | `type SingleBlockMigrations = ()` 是单元类型不是括号，抑制本身多余 |
| `runtime/src/configs.rs:58` `use sp_runtime::traits::Hash as _` | **删** | 编译器确认真·无用 import，全文件零 Hash 方法调用 |
| `offchain/src/solvency.rs` `emit_warning_if_low` | **删** | 空函数体 + 「本步不做」注释，纯占位残桩 |
| `offchain/src/tests/cases.rs` `_touch_encode` | **删** | 为保住 import 而写的 no-op，删后 `Encode` 仍被正常使用 |
| `onchain-issuance/src/lib.rs:58` 整个 pallet 模块静音 | **收窄** | 整片静音只藏了 FRAME 生成的 `deposit_event` 未被调用；收窄到事件定义处 + 中文理由 + 指向开放任务卡 `20260507-onchain-issuance-plain-ft` |

### 第 1 项 批 1（onchina）判定结果

摘除 34 处「无理由静音」→ 编译器暴露 **84 条**（远超 node+runtime 之和），历经 6 轮迭代编译收敛到零警告。

**删除（3 整文件 + 约 30 死项）**：
- 整文件死：`institution/admins/model.rs`（`InstitutionAdmin` 零调用）、`institution/admins/repo.rs`（8 函数零调用，auth 引的是 `city_registry_admins` 另一模块）、`scope/filter.rs`（`filter_by_scope`/`HasProvinceCity` 唯一调用者已死）。
- 机构分类残留（C 组）：`SubjectLegalKind`/`classify`/`legal_kind`/`category_name_zh`/`derive_category`。
- 账户 offchain 残留（B 组）：`can_delete_account`/`required_protocol_account_names(_for_institution)`/`build_default_accounts_for_names`/`build_required_protocol_accounts_for_institution`/`derive_account_id` + `ACCOUNT_NAME_FEE/STAKE/SAFETYFUND/HE`。
- 教育分类残留：`EDUCATION_TYPE_EARLY/PRIMARY/SECONDARY/UNIVERSITY`、`EDUCATION_SCHOOL_TYPES`、`EDUCATION_COMMITTEE_TYPES`、`is_education_school_type`、`AccountKey`/`account_key_to_string`/`account_key_from_string`。
- 零散（E 组）：`ParticipantDraft`、`http_security` 的 `require_public_search_auth`+`constant_time_eq`（onchina 无 public search 路由,非鉴权缺口）。
- `institution_call.rs` 12 处 allow **纯噪音**：摘除后零 warning,全部真实在用（`admins/mod.rs` 正 import）。

**接线（唯一行为变更）**：`assert_no_duplicate_chain_cids` 从零调用接到 `read_chain_projection` 收齐机构投影后、构建 scope 索引前——链上公权机构 CID 重复即拒绝写库。

**保留 + 中文理由（12 组契约字段,删了会引入回归）**：
- `InstitutionCategory` enum：**不是死代码**。编译器暂未判死是因它是 3 个活 DTO（`Institution`/`InstitutionListRow`/`ParentInstitutionRow`）的 `category` 字段类型 + serde,值从 DB `category` 列读（`sql_clause` 的 `'GOV_INSTITUTION'`）。真死的只是 Rust 侧计算分类的 `classify`/`derive_category`——分类已改为 DB 列直读。
- `ServiceError::{NotFound,Conflict}`：`service_error_to_response` 的 HTTP 状态码映射（404/409）完备性槽位,当前只产 BadInput 但 match 列举全部变体。
- `QrKind::UserTransfer`（k=4）：前端 `citizenQr.ts` 有完整解析,跨端协议码契约。
- `CreateInstitutionInput`（含 `subject_property` 第6步原子创建预留）：7 个私权创建 handler 反序列化的 serde DTO。
- `UpdateInstitutionInput.cid_short_name`：serde 编辑 DTO 契约字段。
- `TxRecordRow.{extrinsic_index,event_index}`：DB SELECT 列序投影（`row.get(索引)` 位置读入）。
- `VisibleScope.skip_{province,city,town}_list`：前端 tab 跳级预留字段。
- `chain_read_proposal.rs` 两个 SCALE 解码镜像 struct（9 行逐字段 allow → 2 个结构体级 allow）。
- `budget/model.rs`、`personnel/model.rs`：Phase 4 预留整模块（指向 legislation-console-framework）。
- `chain_vote.rs`：公投/行政签署/护宪终审 encode 函数预留。

**方法学修正（写入本卡防复发）**：
- **onchina 的 QR/scope 与 node 不同**：onchina 死代码判定靠编译器逐层暴露,不能靠单次 grep（类型引用会掩盖真死链）。
- **「活 DTO 的类型」不因构造者死而删**：`InstitutionCategory` 的唯一 Rust 构造者（`classify`）死,但它作为 serde DTO 字段类型仍活（值来自 DB 列反序列化）。误删会让 3 个活 DTO 的 `category` 字段 unresolved。
- **enum「never constructed」变体 ≠ 可删**：`ServiceError::NotFound/Conflict` 无构造点,但 `service_error_to_response` 的 match 消费它们;删变体破坏映射（我误删后 http.rs 4 处 E0599,已恢复+allow）。
- **「活 struct 的死字段」默认保留**：serde DTO 契约字段 / DB 列序投影 / UI 预留字段,删了改契约或改 SELECT,引入行为回归。

**F 组裁定落地**：
- `scope/filter.rs` 删 + **作废死规则**：更新私人记忆 `scope-auto-filter`（作用域过滤下沉 SQL 层,`*_in_scope` 查询按 `VisibleScope` 约束 WHERE）;订正 open 卡 `20260628-onchina-console-refactor.md` 步骤 08 的实现描述。
- `assert_no_duplicate_chain_cids` 接线（见上）。
- `http_security` 两函数删（已核实非鉴权缺口）。
- `UserTransfer` 保留 + 理由。

### 第 1 项 批 2（node）判定结果

- **C 类 6 处整片静音全部摘除**。`offchain/rpc.rs`、`settlement/{listener,reserve,signer,submitter}.rs` 摘除后**零警告**——整片静音什么都没藏，属纯噪音。
- 唯一真命中：`settlement/packer.rs:151` 的 `OffchainPacker::new`。**它不是死代码**——bin 目标没人用，但同文件测试用了 4 次（bin 与 test 是两个编译目标，`--all-targets` 分别报）。且它把 `batch_seq` 写死 0，生产误用会打断「从链上 `LastClearingBatchSeq` 续跑」不变量。**改 `#[cfg(test)]`**，比 `allow` 更准且挡住生产误用。
- **D 类 12 处一律保留**：全部已带中文理由且属「未接线的成品」（`save_to_disk`、`accept_payment`、`FeeRate` 分支、`is_expected_rpc_node`、`scan_keystore_files`、`start_clearing_bank_components_with_noop`、`ledger` 字段），另 `governance/signing.rs` 两处是 **serde 反序列化目标结构**——字段由 serde 读取，`dead_code` 看不见，与 SCALE 占位同类。
- **B 类收敛**：`institution_read/chain.rs` 逐字段 9 行 `allow` → 3 个结构体级 1 行 + 统一中文理由，字段与顺序一字未动。

### 门禁两处修正（本轮自测发现）

1. **理由位置**：仓库惯例把理由写在 `allow` **上一行**（`proposal.rs:226`、`ledger.rs:263`），初版只认同行会误伤合规写法 → 改为取 `allow` 行连同前两行整块判定。
2. **`grep -v '^\+\+\+'` 在 BRE 下把 `\+` 当重复算子**（严格 grep 直接报错，GNU grep 宽容通过）→ 改用 `-E`。
3. 版本门禁加代码文件过滤：规则文档必须能点名被禁标识符（「禁止 `GMB_ROLE_V1`」），否则规则正文自己触发门禁。

用本轮真实改动逆向验证：两条门禁对全部改动**零误报**，对无理由 `allow` 与新增 `GMB_ROLE_V1` 反例均正确拦截。

**验证记录**

- 编译（全量收尾）：**`cargo check --workspace --all-targets` 零警告零错误**——node + runtime + onchina + 全 pallet + qr-protocol + primitives 整体编译干净,无跨 crate 破坏。（起始基线为全 workspace 唯一 1 条警告 + 大量被 `allow` 静音的隐藏死代码。）
- Rust 测试：qr-protocol 8 · entity-primitives 11 · public-manage 19 · private-manage 23 · citizenchain(runtime) 48 · **node 291** · offchain 28 · onchain-issuance 8 · primitives 74 · **onchina 129**（+ `account_derive_golden` / `sign_golden` 各 1）——**全绿，0 失败**。
- Dart 测试：citizenwallet signer 154 绿；citizenapp qr+signer 53 绿；两端 `.g.dart` 逐字节一致。
- 关键用例：`dynamic_role_code_uses_exact_domain_and_never_accepts_lowercase`（含新增的 `MODULE_TAG 变化必须生成不同岗位码`）、`dynamic_role_lifecycle_persists_permissions_and_never_reuses_code`、`genesis_citizenchain_foundation_is_complete_and_protected`、`fields_yaml_has_no_orphan_entries`、`generated_dart_registries_are_current`。
- 抑制总量：**62 文件 / 114 处 → 39 文件 / 72 处**。node / runtime / onchina 三层的「无理由文件级整片静音」全部清零；剩余 72 处 = 38 处 benchmark `weights.rs` 生成物（按定稿不动）+ 约 34 处带中文理由的保留项（SCALE 镜像 / serde DTO 契约 / DB 列序 / Phase 预留 / HTTP 映射完备性 / 跨端协议码）。
- 门禁自检：用本轮真实改动逆向验证 `check-ai-guardrails.sh` 两条新门禁,对全部改动零误报。

**并发事实（写入本卡）**：本轮执行期间主检出有其它会话同时改动 citizenapp 第 3 节删除、`docs/citizenpassport/` 整目录删除、onchina 措辞清理等。第 1 项批 1 的 `institution_call.rs`(12 处 allow) / `admins/mod.rs` 正被对方持有，故押后。

## 分步计划

- **Step 1**：第 4 项，删重复 `#[test]`。清零警告基线，后续摘抑制才有对照。
- **Step 2**：第 5 项，墓碑注释。纯注释零风险。
- **Step 3**：第 2 项 + 裁定 a，删 14 条 `field_key` + 重生两端生成物 + 规则文档同批改写 + 加反向孤儿校验测试。
- **Step 4**：裁定 b，删 `GMB_ROLE_V1` / `GMB_CHAT_V1`，改 `MODULE_TAG` 域 + 12 处文档同步。
- **Step 5**：第 1 项，分 3 批摘抑制（onchina → node → runtime），逐条判定。
- **Step 6**：`check-ai-guardrails.sh` 一次性并入两条门禁（`allow` 需中文理由、`_V1` 只许 `QR_V1`）。

## Step 1 落点

`citizenchain/node/src/core/node_guard/governance_skeleton.rs:984` 是多余的 `#[test]`（正确的在 987）。删 984 行，两行 `///` 文档注释上移到剩下的 `#[test]` 之前。这是全 workspace 唯一编译器警告。

## Step 2 落点

**删（纯墓碑）**：

| 位置 | 删除内容 |
|---|---|
| `onchina/src/main.rs:1954` | 第 2–3 句「原平台签名钥 `ONCHINA_SIGNING_SEED_HEX` … 已随注销凭证链路整体删除」，保留第 1 句现行架构描述 |
| `onchina/src/core/chain_runtime.rs:12` | 后两行「原注销凭证签发链路 … 已整体删除」，保留前两行鉴权契约 |
| `onchina/src/crypto/mod.rs:6` | 「原 sr25519 种子派生工具 … 已随之整体删除」 |

**保留（活不变量，不是残留）**：`runtime/src/lib.rs:375`（pallet index 32 永久留空）、`runtime/primitives/src/sign.rs:52-58`（`OP_SIGN_INST` / `OP_SIGN_DEREGISTER` 为何仍在＝四端金标注册表成员）、3 处 `benchmarks.rs`（wrapper extrinsic 已统一到 VotingEngine，说明 benchmark 缺口原因）。

**附带发现（不在本卡范围）**：`OP_SIGN_INST`(0x13) / `OP_SIGN_DEREGISTER`(0x14) 已无 message 构造入口，是真死常量，但删除会扰动四端字节契约与金标向量。留待单独裁定。

## Step 3 落点

**删除的 14 条 `field_key`**（`crates/qr-protocol/registry/fields.yaml`）：

| 字段 | 判据 |
|---|---|
| `default_role`、`institution_governance_action`、`total_amount_yuan`、`registry_org_code` | 全仓（排除生成物）零命中 |
| `protocol_accounts` | 仅作 runtime 测试局部变量名，非 QR key |
| `funding_account_id` | 仅出现在 Dart 注释的 SCALE 布局说明 |
| `institution`、`signature`、`threshold` | 命中全是无关 Dart 标识符，decoder 从不发射 |
| `credential_issuer_cid_number`、`credential_signer_public_key` | 被 `citizenwallet/test/signer/field_labels_test.dart:87,89` 断言，**删 yaml 必须同删断言** |
| `register_nonce`、`scope_province_name`、`scope_city_name` | 链端/前端有同名**数据结构字段**，但从不作 QR 展示 key；**不动那些结构体字段** |

**执行顺序（严格）**：删 yaml → 删 2 条 Dart 断言 → `cargo run -p qr-protocol --bin export_registry -- --dart <两端路径各一次>` → `cargo test -p qr-protocol`（`generated_dart_registries_are_current` 逐字节校验）→ 两端 `flutter test`。

**新增反向校验**：`crates/qr-protocol/tests/registry_consistency.rs` 加 `fields_yaml_has_no_orphan_entries` —— 每条 `field_key` 必须被某 action 的 `required_fields` 引用，或落在白名单（当前只有 `amount_` 动态前缀一条）。现有测试只做 actions→fields 单向校验，**缺反向**正是 14 条孤儿的成因。

**裁定 a 的文档同步**：

| 文件 | 性质 | 处置 |
|---|---|---|
| `memory/AGENTS.md:110`、`memory/07-ai/agent-rules.md:89` | 当前规则（同文两处，须逐字一致） | 删 `registry_org_code` 分支，收敛为 `institution_code + workspace` |
| `memory/01-architecture/onchina/README.md:55` | 当前架构 | 删「注册局机构归属仍只允许用 `registry_org_code`」整句 |
| `memory/04-decisions/ADR-025-*.md:36` | 历史决策记录 | 只加「后续变更」一行，**不改正文** |
| `ADR-030:13`、`08-tasks/done/20260621-*.md` | 历史叙述 / 已完结档案 | 不动 |

## Step 4 落点

**替代方案**：用既有 `MODULE_TAG` 惯例，不造新常量。`public-manage = b"pub-mgmt"`、`private-manage = b"pri-mgmt"`，域分隔强度不降反升（公权/私权进入不同域）。

```rust
// 删 pub const GMB_ROLE_V1
pub fn generate_dynamic_role_code(
    module_tag: &[u8],
    cid_number: &[u8], nonce: u64, proposal_id: u64,
) -> Vec<u8> {
    let hash = BlakeTwo256::hash_of(&(module_tag, cid_number, nonce, proposal_id));
    ...
}
```

**runtime 改动（8 文件，二次确认已获授权）**：`entity-primitives/src/institution_role.rs`（删第 30 行常量、改第 36–37 行签名与哈希材料、第 419 行测试）、`entity-primitives/src/lib.rs:42`（删 re-export）、`{public,private}-manage/src/institution/role.rs`（传 `crate::MODULE_TAG`）、`{public,private}-manage/src/tests/cases.rs`（4+3 处）、`runtime/src/tests/cases.rs`（2 处）。

**风险已实测**：全仓 `R_<32位大写十六进制>` 字面量为 0 → 无金标向量、无 chainspec 依赖；创世走固定岗位码不经本函数（`GENESIS_TECHNICAL.md:42`）；现有测试只断言形态与确定性，改域后照常通过。开发期已派生动态岗位码会变，链开发期零用户，重新创世即可。

**`GMB_CHAT_V1`（零风险）**：仅 4 行注释 —— `citizenapp/chat/proto/chat_envelope.proto:5`、`lib/chat/chat_flow.dart:161`、`lib/chat/crypto/mls_session.dart:58,132`、`lib/isar/app_isar.dart:575`。改措辞为 `gmb.chat.v1` proto 包 / `protocol_version` 字段，零行为、零 wire 改动。

**规则提醒注释升级（4 处保留但改措辞）**：`citizenapp/lib/signer/signing.dart:15`、`cloudflare/src/shared/signing_message.ts:8`、`cloudflare/src/auth/wallet_signature.ts:4`、`cloudflare/test/auth.test.ts:132` —— 由「禁 `GMB_*_V1`」升级为「全仓唯一版本化协议标识为 `QR_V1`」。

**新硬规则措辞（落 `unified-naming.md` + `AGENTS.md`）**：

> 全仓唯一允许的版本化协议标识是 `QR_V1`（扫码协议）。签名域唯一走 `signing_message(op_tag)`；非签名哈希域一律用所属 pallet 的 `MODULE_TAG`。禁止任何新的 `*_V1` 常量。

**文档同步 12 处**：`CITIZENCHAIN_TECHNICAL.md:200`、`ADR-039:53`、onchina `BACKEND_TECHNICAL.md:162` / `FRONTEND_TECHNICAL.md:113`、`PRIVATE_MANAGE_TECHNICAL.md:17`、`PUBLIC_MANAGE_TECHNICAL.md:18`、`GENESIS_TECHNICAL.md:42`、`unified-protocols.md:33` 与 `:1570`、`unified-naming.md:144`、任务卡 `20260719-institution-role-permission-unify.md:18,77,111`。

**其中 `unified-naming.md:144` 必须优先改** —— 原文「域分隔符唯一命名为 `GMB_ROLE_V1`，不得另造别名」是正向强制该名字的硬规则，不改就是直接的规则-代码冲突。

## Step 5 落点

**四类分治（62 文件 / 114 处）**：

| 类 | 数量 | 处置 |
|---|---|---|
| A 生成物 | 19 文件 / 38 处 `weights.rs` | **一行不碰**，benchmark 重跑写回 |
| B SCALE 字段序占位 | 约 15 处（`constitution/mod.rs` 3、`institution_read/chain.rs` 9 等） | 保留语义，逐字段 allow 收敛为结构体级 1 行 + 统一中文理由注释，不改字段与顺序 |
| C 文件级整片静音 | 14 文件 | 逐个摘除，让编译器说话 |
| D 单点 allow | 约 15 处 | 逐条判定 |

**C 类清单**：`node/src/transaction/offchain/rpc.rs`、`settlement/{listener,packer,reserve,signer,submitter}.rs`、`onchina/src/cid/category.rs`、`domains/gov/handler.rs`、`domains/legislation/{budget/model,law/chain_vote,personnel/model}.rs`、`institution/{admins/model,admins/repo,subjects/model,subjects/service}.rs`、`scope/mod.rs`、`runtime/primitives/cid/number.rs`。

**三批执行**：批 1 `onchina`（9 个 C 类，独立 bin crate 编译最快）→ 批 2 `node`（6 个 C 类 + D 类）→ 批 3 `runtime`（6 处，已获二次确认）。

**判定只有三种结局**：接线（本该被调用）/ 删除（确认无用）/ 保留并写明中文理由。**不允许「先加回 allow 再说」**。

**批 2 特别注意**：`settlement/*` 属 ADR-007 L2 清算现行架构，报出的死函数须先判「未接线」还是「真死」，不得直接删。

## Step 6 落点

改现有 `.github/scripts/check-ai-guardrails.sh`（**不新建文件**），一次性并入两条 diff 级门禁：

1. 新增 `allow(dead_code)` / `allow(unused` 行必须同行或上一行带中文理由注释，否则拒。
2. 新增行出现 `_V1` 标识符且不是 `QR_V1`，拒。

**不上 `-D warnings` 的 clippy job**：citizenchain 现在连 `cargo test` 都没有，突然加全量 lint 会一次性红一大片。等 C/D 类清完再单独议。

## 验收

- Step 1：`cargo check --workspace --all-targets` 由 1 warning 变 0 warning。
- Step 3：`cargo test -p qr-protocol` 全绿（含新增反向校验）；两端 `flutter test` 全绿；两端 `.g.dart` 逐字节一致。
- Step 4：`cargo test -p citizenchain-runtime` 及两个 entity pallet 测试全绿；全仓 `_V1` 只剩 `QR_V1`。
- Step 5：每批 `cargo check -p <crate> --all-targets` 零警告 + `cargo test -p <crate>` 全绿；批 2 后本地起链跑通链下清算路径（真实运行态验收硬规则）。
- 全程：`/Users/rhett/GMB` 主检出操作，不碰任何 worktree 副本。
