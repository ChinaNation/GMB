# ADR-024 账户地址派生统一为唯一真源

- 状态：Tier 1/2 + `gmb.py` 改名 已实现并提交于 HEAD `4ab8efe6 "统一机构命名"`（2026-06-22，与并行 T3/T4 同提交混合）；合并态全量验证通过（链端 `cargo check --workspace` + 派生 22 测试 + 金标 1；后端 71；citizenapp Dart 28 + 跨语言金标逐字节对齐）；行为中性、地址逐字节零变化已证。Tier 3（域 `DUOQIAN→GMB`）+ 账户重生 仍 gated 在 `20260622-cid-classification-unify-t3t4` Phase 3 之后
- 关联：[[ADR-021]] 单源思想；任务卡 `20260622-account-derive-single-source.md`；并入 `20260622-cid-classification-unify-t3t4` 末尾创世
- 取代：原 `20260622-derive-domain-rename-gmb-op-name`（改名收为本 ADR Tier 3）

## 背景（问题）

底层哈希算法（`core_const::derive_account` = `blake2_256(域 ‖ op_tag ‖ ss58_le ‖ payload)`）已是单源，但哈希**之上**的 op_tag / 5 保留名 / 路由（name→op_tag）/ payload 字段拼装散落 4 处，且已漂移：

| 要素 | 散落处 |
|---|---|
| op_tag | `core_const.rs:40-46` + citizenapp dart `kOp*` |
| 5 保留名 | `core_const`(&[u8]) + 后端 `accounts/derive.rs`(&str) + citizenapp + citizenwallet（**4 份**） |
| 路由 | `organization-manage`(3 tag) + 后端 `derive.rs`(6 tag) + citizenapp dart(6 tag) + `china/mod.rs`·`personal-manage` 内联（**≥4 表**） |
| `isForbidden` | core_const + citizenapp + citizenwallet（**3 份，2 种行为**） |

🔴 漂移 bug：`isForbidden` 链端/冷钱包判 3 名不 trim，citizenapp 判 5 名 + `trim()` → `"  主账户  "` 两端结论相反。

三种 payload schema（异构是账户种类本质，**不抹平**）：
- 机构 主/费/质押/安全/两和 → `cid_number`
- 机构自定义 → `cid_number ‖ account_name`
- 个人多签 → `creator(32B) ‖ account_name`

## 决策

### 原则（用户锁定）
1. 账户派生 op_tag + 保留名 + 路由 + payload 字段拼装收敛到**唯一一处**。
2. payload schema **不改**（只统一定义源，Tier 1/2 行为中性、地址不变）。
3. Dart 跨语言防漂移 = **金标向量 fixture**（不 codegen）。
4. 改名 `DUOQIAN→GMB` + `OP_INSTITUTION→OP_NAME` 并入 Tier 3，字节值随 T3/T4 末尾创世。
5. 只改账户派生域常量 + op_tag，不碰同名无关模块（duoqian-transfer / chain_duoqian_info / DUOQIAN_PALLET_INDEX）。

### Tier 1 — Rust 单源：新建 `primitives::account_derive`

新模块持有：op_tag（从 core_const 迁入，`OP_INSTITUTION`→`OP_NAME`，值 0x06 不变）+ 5 保留名 + `is_forbidden_account_name` + `AccountKind` 枚举（op_tag↔payload 唯一映射）+ 路由 `institution_kind_by_name` + 注册策略 `is_registrable_custom_name` + 唯一派生入口 `AccountKind::derive`。`core_const` 只留：域分隔符 `DUOQIAN`（签名亦共用）+ `SS58_FORMAT` + 签名 op_tag（0x10-0x1F）。删 core_const 的账户 op_tag / 保留名 / `derive_account` / `is_forbidden`。

```rust
// primitives/src/account_derive.rs（核心）
use crate::core_const::DUOQIAN;            // 域共享（签名也用），留在 core_const
use sp_core::hashing::blake2_256;
use sp_std::vec::Vec;

pub const OP_MAIN: u8 = 0x00;  pub const OP_FEE: u8 = 0x01;
pub const OP_STAKE: u8 = 0x02; pub const OP_AN: u8 = 0x03; pub const OP_HE: u8 = 0x04;
pub const OP_PERSONAL: u8 = 0x05;
pub const OP_NAME: u8 = 0x06;  // 原 OP_INSTITUTION

pub const RESERVED_NAME_MAIN: &[u8] = "主账户".as_bytes();
pub const RESERVED_NAME_FEE: &[u8] = "费用账户".as_bytes();
pub const RESERVED_NAME_STAKE: &[u8] = "永久质押".as_bytes();
pub const RESERVED_NAME_ANQUAN: &[u8] = "安全基金".as_bytes();
pub const RESERVED_NAME_HE: &[u8] = "两和基金".as_bytes();
pub const RESERVED_ACCOUNT_NAMES: [&[u8]; 5] =
    [RESERVED_NAME_MAIN, RESERVED_NAME_FEE, RESERVED_NAME_STAKE, RESERVED_NAME_ANQUAN, RESERVED_NAME_HE];

/// 制度专属「禁止注册」名（质押/安全/两和）。主/费不在此列（强制默认）。不 trim。
pub fn is_forbidden_account_name(name: &[u8]) -> bool {
    name == RESERVED_NAME_STAKE || name == RESERVED_NAME_ANQUAN || name == RESERVED_NAME_HE
}

/// op_tag + payload 字段 schema 的唯一权威映射。
#[derive(Clone, Copy, Debug)]
pub enum AccountKind<'a> {
    InstitutionMain { cid_number: &'a [u8] },
    InstitutionFee { cid_number: &'a [u8] },
    InstitutionStake { cid_number: &'a [u8] },
    InstitutionAnquan { cid_number: &'a [u8] },
    InstitutionHe { cid_number: &'a [u8] },
    InstitutionNamed { cid_number: &'a [u8], account_name: &'a [u8] },
    Personal { creator: &'a [u8; 32], account_name: &'a [u8] },
}

impl<'a> AccountKind<'a> {
    pub const fn op_tag(&self) -> u8 { /* 7 分支 → OP_* */ }
    fn payload(&self) -> Vec<u8> {        // 字段拼装唯一处
        match self {
            InstitutionMain|Fee|Stake|Anquan|He { cid_number } => cid_number.to_vec(),
            InstitutionNamed { cid_number, account_name } => cid ‖ name,
            Personal { creator, account_name } => creator ‖ name,
        }
    }
    /// 唯一派生入口：preimage = DUOQIAN ‖ op_tag ‖ ss58_le(2B) ‖ payload → blake2_256。
    pub fn derive(&self, ss58: u16) -> [u8; 32] { /* 拼 preimage + blake2_256 */ }
}

/// 唯一路由表：name → AccountKind（主/费/质押/安全/两和 各自；其他非空 → Named；空 → None）。
/// 只做派生路由，不做「能否注册」校验。
pub fn institution_kind_by_name<'a>(cid: &'a [u8], name: &'a [u8]) -> Option<AccountKind<'a>> { ... }

/// 注册策略（非派生）：空/主/费/制度专属 一律不可作自定义名。
pub fn is_registrable_custom_name(name: &[u8]) -> bool {
    !name.is_empty() && name != RESERVED_NAME_MAIN && name != RESERVED_NAME_FEE
        && !is_forbidden_account_name(name)
}
```

调用方改造（全部改调新源，删本地重复）：
- `organization-manage`：删 `InstitutionAccountRole` 枚举 + `address.rs`；`derive_institution_account`/`role_from_account_name` 合成一个薄适配 `derive_registered_account(cid, name)`：空→EmptyAccountName、`is_forbidden`→ReservedAccountName，否则 `institution_kind_by_name` 派生（只会得 Main/Fee/Named）。register.rs / accounts.rs / close.rs / benchmarks / tests 改调。
- `personal-manage::derive_personal_account`：`creator.encode()`→`[u8;32]`，走 `AccountKind::Personal`。
- `china/mod.rs`（创世）：内联 `derive_account(OP_MAIN..)`→`AccountKind::InstitutionMain{cid}.derive(SS58_FORMAT)`。
- 后端 `accounts/derive.rs`：删 5 个 `&str` 保留名 + 路由，`derive_account(cid,name)`→`institution_kind_by_name(...).map(|k| hex::encode(k.derive(SS58_FORMAT)))`；`RESERVED_ACCOUNT_NAMES` 改 re-export 新源（&[u8]）。调用方 `admins/actions.rs:909`、`accounts/handler.rs:86`、`subjects/service.rs:344`、`citizenapp/public_institution.rs:400`（&str→&[u8] 适配）。
- **`scripts/duoqian.py`（第 5 个派生镜像，创世账户烘焙器，474 行）**：read `china_*.rs` 的 `cid_number` + regex 读 `core_const.rs` 的 `DUOQIAN`/`SS58_FORMAT`/`OP_*` → 算 main/fee/stake 写回 china_*.rs。本 ADR 把 op_tag 迁到 `account_derive.rs`（Tier 1）+ 域改 `GMB: &[u8;3]`（Tier 3）→ **它的 regex 必断**，须同步改其读取路径（op_tag 改读 account_derive.rs、域改读 `GMB`/`&\[u8; 3\]`），并纳入金标覆盖。它**只读 cid 不生成 cid**。

### Tier 2 — 跨语言金标对齐

- citizenapp 保留**唯一** Dart 镜像 `account_derivation.dart`；citizenwallet 只共享保留名。
- Rust 导出金标 fixture（canonical 向量），Dart 测试逐字节断言；CI 守卫两份副本一致。
- 修 citizenapp 漂移：`isForbiddenAccountName`→3 名 + 不 trim（对齐链端）；自定义名拒绝逻辑改用「registrable」判据（空/主/费/制度专属 拒绝），trim 只允许在 UI 输入层、绝不进派生/校验。

fixture 格式 `account_derive_vectors.json`：
```json
{ "ss58_format": 2027, "domain": "DUOQIAN",
  "vectors": [
    {"kind":"InstitutionMain","cid_number":"LN001-GCB05-944805165-2026","address_hex":"..."},
    {"kind":"InstitutionFee","cid_number":"...","address_hex":"..."},
    {"kind":"InstitutionNamed","cid_number":"...","account_name":"工资账户","address_hex":"..."},
    {"kind":"InstitutionStake","cid_number":"...","address_hex":"..."},
    {"kind":"Personal","creator_hex":"<64hex>","account_name":"我的多签","address_hex":"..."}
  ] }
```
- канon 路径 `citizenchain/runtime/primitives/tests/fixtures/account_derive_vectors.json`；Dart 副本 `citizenapp/test/governance/shared/fixtures/`。
- Rust 测试：`ACCOUNT_DERIVE_UPDATE=1` 时写文件，否则读取断言。Dart 测试读副本断言。CI：脚本 `tools/sync_account_derive_vectors.sh` 重生 + `git diff --exit-code` 两份。
- **行为中性回归证明**：先用**当前 DUOQIAN 值**生成 fixture 提交，PR-1/2/3 后断言地址不变；Tier 3（GMB）才允许 fixture 改变。

### Tier 3 — 改名（gated 在 T3/T4 末尾创世）

- `core_const::DUOQIAN`(b"DUOQIAN",`&[u8;7]`)→`GMB`(b"GMB",`&[u8;3]`)：account_derive import + 签名 `configs/mod.rs:798,855,952,1020,1099` + 后端 `core/chain_runtime.rs` + 热钱包 `account_derivation.dart:45 _domain` 锁步；`china/{cb,ch,zb}.rs` 创世地址重算。
- `OP_INSTITUTION`→`OP_NAME` 值中性（Tier 1 已落地）。
- 冷钱包 citizenwallet 签名链路审计（确认是否含域）。

## 前置依赖与创世内顺序（2026-06-22 代码核查）

**核查结论：T3/T4 机构码重构当前不具备直接在本 ADR 内重生所有机构账户。** 账户派生吃 `cid_number`；创世机构 cid_number 来自 `china_*.rs`，而 china_*.rs 仍是**旧格式死码**：

| 环节 | 状态 |
|---|---|
| `number/code.rs` 新码 | ✅ 就位（NRC/PRB/PRS/CGOV/SFGT/CTZN/FRG） |
| `gov/service.rs` 模板 | ✅ 新码（PDF/PHS/CGOV…） |
| `china_*.rs` 创世 CID | ❌ 旧格式（`GCB05`/`GZF02`/`GJC`/`GLF`/`GSF`/`SCH`；旧段二码 ZF/JC/LF/SF/JY/CB 已从 code.rs 删除=死码） |
| china 账户字面值 `main/fee_account` | ❌ 基于旧 CID + 旧域 DUOQIAN，双重过期 |

旧码 CID 是 `&'static str`，**编译不报错 → 静默过期**。

**正确创世内顺序（全在最后一次 re-genesis）：**
1. **T3/T4 Phase 3**：重烤 `china_*.rs` cid_number → 新码（前置，**未做**）。
2. **ADR-024 Tier 3**：域 `DUOQIAN→GMB`（core_const）。
3. 跑 `scripts/duoqian.py`（已改读取路径）：用「新 CID + GMB」重算 china 账户字面值。
4. `citizenchain/scripts/bake-chainspec.sh` + 重跑 `citizenapp/tools/generate_public_institution_bundle.mjs`。

**结论对实施的约束：** Tier 1/2 行为中性、不碰 china/创世/cid，**可现在做**；**Tier 3 + 账户重生 硬 gated 在 T3/T4 Phase 3 之后**（缺最终态 cid_number，ADR-024 内不能独立闭环）。

**决策 B（2026-06-22，用户拍板）：** china 机构 CID 重烤（=T3/T4 Phase 3，含 federal 常量 CID 的 account_pubkey 确定性种子约定）**留在 T3/T4 线程**，本线程不吸收，避免两线程同改 `china_*.rs` 分叉。本线程现在做：Tier 1 + Tier 2 + `duoqian.py→gmb.py` 改名（regex 读取路径 op_tag 部分改到 `account_derive.rs`，域 GMB 部分留 Tier 3）。等 T3/T4 出新码 CID，本线程再 Tier 3（域 GMB）+ 跑 `gmb.py` 账户重生 + 一次创世。
（种子约定备注：CID 的 N9 = `blake2b(account_pubkey ‖ 机构码 ‖ 省 ‖ 市 ‖ 年)[:4] % 1e9`，account_pubkey 是同 (码,省,市,年) 桶内的去重熵；动态注册用随机 UUID+1000 重试，创世须确定性种子，gov 模板已用 `GOV-{scope}-…`，federal 常量种子由 T3/T4 定。）

**新增守卫测试**（防静默过期）：每条 china `cid_number` 必须 `InstitutionCode::from_str` 过新码表，否则 CI 红。

## 实施顺序（PR 切分）

- **PR-1** Tier 1 链端：建 `account_derive` 模块 + 迁移 + organization-manage/personal-manage/china/mod 改调 + 删 core_const 旧项。`cargo test`（primitives/organization-manage/personal-manage/runtime）行为中性。
- **PR-2** Tier 1 后端：`accounts/derive.rs` 委托 + 删重复 + 调用方适配。`cargo test`。
- **PR-3** Tier 2：Rust golden 导出 + citizenapp golden 测试 + 修 isForbidden 漂移 + citizenwallet 保留名对齐。`flutter analyze` + dart test。
- **PR-4** Tier 3：改名 + 签名锁步 + china_*.rs 重烤；并入 T3/T4 Phase 3 创世。

## 验收

- 链端 + 后端全测过，**金标向量证明 PR-1/2/3 地址零变化**；后端 `chain_runtime` golden 重算对齐。
- 全仓残留=0：账户 op_tag / 保留名 / 路由仅 `account_derive`（Rust）+ `account_derivation.dart`（Dart 镜像）+ citizenwallet 保留名常量；旧 core_const 账户项、后端 &str 常量、第二路由表清零。
- 端到端：扫码签名 + 机构账户地址 ↔ 链上回执一致。

## 后果

- 正：单源；漂移 bug 根治；新增 op_tag/账户种类只改一处 + 金标自动拦截 Dart 漂移；OP_NAME 语义更准。
- 负：引入 `AccountKind` 枚举 + 金标 fixture 维护（CI sync 脚本）；Dart 仍是手写镜像（金标兜底，非编译期保证）。
- 风险：creator `encode()`→`[u8;32]` 转换需断言长度；no_std 下 `AccountKind::payload` 用 `sp_std::Vec`；Tier 3 域字节变更牵连签名，必须钱包锁步 + 创世重烤。

## 实施记录（2026-06-22，Tier 1/2 + gmb.py 已落地，行为中性）

**单源落地**：`citizenchain/runtime/primitives/src/account_derive.rs` = 全仓账户地址派生唯一真源（op_tag `OP_MAIN..OP_NAME`，`OP_NAME=0x06` = 原 `OP_INSTITUTION` 值不变；5 保留名；`is_forbidden_account_name`(3 名不 trim)；`AccountKind` 枚举 op_tag↔payload 唯一映射；`institution_kind_by_name` 路由；`is_registrable_custom_name`；唯一入口 `AccountKind::derive`）。`core_const` 账户 op_tag/保留名/`derive_account`/`is_forbidden` 已删，仅留域 `DUOQIAN`(派生+签名共用) + 签名 op_tag `OP_SIGN_*`，无任何兼容 re-export。调用方全部委托新源：后端 `accounts/derive.rs`、`organization-manage`(删 `address.rs`/`InstitutionAccountRole`)、`personal-manage`、`china/mod.rs`、`subjects/service.rs`。

**Dart 单源**：`citizenapp/lib/governance/shared/account_derivation.dart`(op_tag + 路由)+ `account_derivation` 调用方 + `reserved_account_names.dart`(citizenapp + citizenwallet 各一,只共享保留名)；citizenapp `isForbidden` 漂移已修(3 名 + 不 trim,对齐链端)。

**金标**：canonical `citizenchain/runtime/primitives/tests/fixtures/account_derive_vectors.json` + Dart 副本 `citizenapp/test/governance/shared/fixtures/`，导出测试 `tests/account_derive_golden.rs`，CI 守卫 `tools/sync_account_derive_vectors.sh`。`domain=DUOQIAN`/`ss58=2027` 基线，行为中性铁证 = `china_*.rs` 字面常量(NRC main/fee、ANQUAN、HE)与 fixture 逐字节一致。

**改名**：`scripts/duoqian.py → scripts/gmb.py`(op_tag 读取路径改到 `account_derive.rs`；域 `GMB` 部分留 Tier 3)。

**测试结果**：链端 `cargo test -p primitives -p organization-manage -p personal-manage` 全绿(organization-manage 29 + personal-manage 23 + primitives lib 20 + golden 1 + 3 doc-tests，0 failed)；后端 `cargo test` 71 + 5 integration passed(含 `accounts::derive::tests` 6 项)；金标二次 `ACCOUNT_DERIVE_UPDATE=1` 重跑后 fixture `git diff --stat` 为空 = 确定性/地址零变化。

**仍 gated（Tier 3，T3/T4 Phase 3 之后）**：域 `DUOQIAN→GMB`(字节变=地址变)+ china CID 重烤 + 跑 `gmb.py` 账户重生 + 一次创世 + 重跑 `generate_public_institution_bundle.mjs`。
