---
title: 全仓库统一签名/派生域为 DUOQIAN_V1 + op_tag
status: done
owner: Blockchain Agent + SFID Agent + Mobile Agent
created: 2026-04-20
completed: 2026-04-20
---

# 执行结果（2026-04-20）

- **primitives 常量**：`primitives/src/core_const.rs` 新增 `DUOQIAN_DOMAIN = b"DUOQIAN_V1"`（10B）+ `OP_MAIN/FEE/STAKE/AN/PERSONAL`（0x00-0x04 地址派生）+ `OP_SIGN_BIND/VOTE/POP/INST/CREATE/TRANSFER`（0x10-0x15 签名 payload）
- **地址哈希重算**：`tools/duoqian.py` 重写为四合一生成器，跑 `--apply` 一次性重算：
  - 277 个 `main_address`（cb/ch/zf/jc/lf/sf/jy 7 文件）
  - 87 个 `fee_address`（cb+ch）
  - 43 个 `stake_address`（ch，按 `citizens_number_u64_le` 输入）
  - 1 个 `NRC_ANQUAN_ADDRESS`
  - `CHINA_RESERVED_MAIN_ADDRESSES` 汇总重建（408 个唯一地址）
- **链上派生函数**：
  - `derive_duoqian_address_from_sfid_id` → `DUOQIAN_DOMAIN + OP_MAIN`
  - `derive_personal_duoqian_address` → `DUOQIAN_DOMAIN + OP_PERSONAL`
- **configs/mod.rs 4 个 verifier 11 处**：BIND/VOTE/POP/INST 各自拼对应 op_tag
- **SFID 后端**：`sfid/backend/src/chain/runtime_align.rs` 5 个旧 `*_DOMAIN` 统一成 `DUOQIAN_DOMAIN + OP_SIGN_*`
- **前端镜像**：`deriveDuoqianAddress.ts` + `personal_duoqian_create_page.dart` 重写 preimage 组装
- **Benchmarks**：`sfid-code-auth/src/benchmarks.rs` 同步
- **注释清理**：`china_cb.rs` / `china_ch.rs` 顶部注释、`sfid/backend/src/institutions/service.rs` 注释
- **验证**：9 个 pallet `cargo check` 全通过；`cargo test -p primitives` 7/7 通过（含 `all_china_ch_main_addresses_are_unique`）

# 彻底退役字符串（零保留）

以下旧 domain 前缀在活代码和 05-modules 技术文档中**零出现**，仅在历史任务卡 / ADR 中作为回顾记录保留（都已加"2026-04-20 彻底退役"尾注）：

- `DUOQIAN_SFID_V1`
- `DUOQIAN_PERSONAL_V1`
- `FEIYONG_SFID_V1`
- `ANQUAN_SFID_V1`
- `GMB_SFID_V1`
- `GMB_SFID_BIND_V3` / `GMB_SFID_VOTE_V3` / `GMB_SFID_POPULATION_V3`
- `GMB_SFID_INSTITUTION_V1` / `GMB_SFID_INSTITUTION_V2`

# 机构账户与签名域的最终模型

| 用途 | preimage（domain + op_tag + payload） | 输入 payload |
|---|---|---|
| 机构主账户（含注册多签） | `DUOQIAN_V1 + 0x00 + ss58 + sfid_id [+ name]` | SFID 身份码 |
| 机构费用账户 | `DUOQIAN_V1 + 0x01 + ss58 + shenfen_id` | 机构身份 ID |
| 省储行质押账户 | `DUOQIAN_V1 + 0x02 + ss58 + citizens_number_u64_le` | 省人口数 |
| 国储会安全基金 | `DUOQIAN_V1 + 0x03 + ss58 + NRC_shenfen_id` | NRC 身份 |
| 个人多签账户 | `DUOQIAN_V1 + 0x04 + ss58 + creator_32 + name` | 创建人+名称 |
| 公民身份绑定签名 | `(DUOQIAN_V1, 0x10, genesis, who, binding_id, nonce)` | SCALE tuple |
| 公民投票签名 | `(DUOQIAN_V1, 0x11, genesis, who, binding_id, proposal_id, nonce)` | SCALE tuple |
| 人口快照签名 | `(DUOQIAN_V1, 0x12, genesis, who, eligible_total, nonce)` | SCALE tuple |
| SFID 机构登记签名 | `(DUOQIAN_V1, 0x13, genesis, sfid_id, name, nonce [, province])` | SCALE tuple |

# 背景

当前 5 个域前缀（`DUOQIAN_SFID_V1` / `DUOQIAN_PERSONAL_V1` / `FEIYONG_SFID_V1` / `ANQUAN_SFID_V1` / `GMB_SFID_V1`）功能分散，其中 `GMB_SFID_V1` 已在 Phase 1.B 把 BIND/VOTE/INSTITUTION/POPULATION 合并，但地址派生（DUOQIAN/FEIYONG/ANQUAN/PERSONAL）仍分散。

**统一目标**：一个 domain `DUOQIAN_V1` + 1 字节 `op_tag` 覆盖全部"地址派生 + 签名 payload"。

# 统一方案

```rust
// primitives/src/core_const.rs
pub const DUOQIAN_DOMAIN: &[u8; 10] = b"DUOQIAN_V1";

// 地址派生 op_tag (0x00-0x0F)
pub const OP_MAIN:     u8 = 0x00;  // ss58 || shenfen_id [|| name]
pub const OP_FEE:      u8 = 0x01;  // ss58 || shenfen_id
pub const OP_STAKE:    u8 = 0x02;  // ss58 || citizens_number_u64_le
pub const OP_AN:       u8 = 0x03;  // ss58 || NRC_shenfen_id
pub const OP_PERSONAL: u8 = 0x04;  // ss58 || creator_32 || name

// 签名 payload op_tag (0x10-0x1F)
pub const OP_SIGN_BIND:     u8 = 0x10;
pub const OP_SIGN_VOTE:     u8 = 0x11;
pub const OP_SIGN_POP:      u8 = 0x12;
pub const OP_SIGN_INST:     u8 = 0x13;
pub const OP_SIGN_CREATE:   u8 = 0x14;  // 预留：注册多签离线聚合创建
pub const OP_SIGN_TRANSFER: u8 = 0x15;  // 预留：多签转账离线聚合
```

# 执行清单

## 第 1 步：primitives 新增常量
- [ ] `primitives/src/core_const.rs`：新增 `DUOQIAN_DOMAIN` + `OP_*` 常量

## 第 2 步：重写生成器 + 重算 773 个哈希
- [ ] `tools/duoqian.py` 重写为四合一生成器（main + fee + stake + anquan）
- [ ] 运行 `--apply` 重算：
  - 277 main_address（7 文件）
  - 87 fee_address（2 文件）
  - 43 stake_address（china_ch，按 citizens_number u64_le 输入）
  - 1 NRC_ANQUAN_ADDRESS
  - 365 `CHINA_RESERVED_MAIN_ADDRESSES`（排序后重建）

## 第 3 步：链上派生函数
- [ ] `duoqian-manage-pow/src/lib.rs` 的 `derive_duoqian_address_from_sfid_id`：`b"DUOQIAN_SFID_V1"` → `DUOQIAN_DOMAIN + OP_MAIN`
- [ ] `derive_personal_duoqian_address`：`b"DUOQIAN_PERSONAL_V1"` → `DUOQIAN_DOMAIN + OP_PERSONAL`

## 第 4 步：configs 4 个 verifier
- [ ] `configs/mod.rs` 11 处 `b"GMB_SFID_V1"` → `DUOQIAN_DOMAIN + OP_SIGN_*`（按 BIND/VOTE/POP/INST 匹配 tag）

## 第 5 步：SFID 后端
- [ ] `sfid/backend/src/chain/runtime_align.rs` 5 个常量 + verifier 逻辑

## 第 6 步：TS/Dart 镜像
- [ ] `sfid/frontend/src/utils/deriveDuoqianAddress.ts`
- [ ] `wuminapp/lib/governance/personal_duoqian_create_page.dart`

## 第 7 步：bench + 注释
- [ ] `sfid-code-auth/src/benchmarks.rs:41`
- [ ] `china_cb.rs` / `china_ch.rs` 顶部注释

## 第 8 步：验证
- [ ] `cargo check -p primitives -p duoqian-manage-pow -p sfid-code-auth -p citizenchain-runtime --offline`
- [ ] `cargo test -p primitives`

## 第 9 步：文档彻底清理（V2/V3 名字零保留）
- [ ] BLAKE2_ADDRESS_DERIVATION.md 整篇重写
- [ ] DUOQIAN_TECHNICAL.md、GOVERNANCE_TECHNICAL.md、SIGNER_TECHNICAL.md、SFIDCODEAUTH_TECHNICAL.md、CHAIN_TECHNICAL.md、SAFETY_FUND_GOVERNANCE.md
- [ ] GMB_WHITEPAPER.md（stake_address 派生说明）
- [ ] ADR-005 加尾注 "Superseded by DUOQIAN_V1 unification"
- [ ] feedback_sfid_sheng_signing_keyring.md 更新铁律
- [ ] 4 份历史任务卡更新

# 铁律

- 旧 V3/V2/_SFID_V1/_PERSONAL_V1 字符串**零保留**
- 按 `feedback_chain_in_dev.md` 走 fresh genesis
- 按 `feedback_no_compatibility.md` 不做兼容过渡
