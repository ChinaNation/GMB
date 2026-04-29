---
title: SFID 机构账户按角色分流 op_tag（新增 OP_INSTITUTION=0x05）
status: done
owner: Blockchain Agent + SFID Agent + Mobile Agent
created: 2026-04-21
completed: 2026-04-21
---

# 执行结果（2026-04-21）

- **primitives**：[core_const.rs](citizenchain/runtime/primitives/src/core_const.rs) 新增 `OP_INSTITUTION = 0x05` 常量，注释约束为 "SFID 机构自定义命名账户"
- **链端枚举 + 派生**（[duoqian-manage/src/lib.rs](citizenchain/runtime/transaction/duoqian-manage/src/lib.rs)）：
  - 新增 `InstitutionAccountRole<'a>` 枚举（`Main` / `Fee` / `Named(&'a [u8])`）+ 保留名常量 `RESERVED_NAME_MAIN` / `RESERVED_NAME_FEE`
  - 旧 `derive_duoqian_address_from_sfid_id(sfid_id, name)` **重构为** `derive_institution_address(sfid_id, role)`
  - 新增 `role_from_name(name)` 翻译辅助："主账户"→Main, "费用账户"→Fee, 其他非空→Named(name), 空→`EmptySfidName`
  - 新增 `ReservedAccountName` 错误变体，拦截 `Role::Named("主账户")` / `Role::Named("费用账户")`
- **调用点同步**：`register_sfid_institution` 用 `role_from_name` + `derive_institution_address` 两步；[benchmarks.rs](citizenchain/runtime/transaction/duoqian-manage/src/benchmarks.rs) 改用 `Role::Main`
- **SFID 后端**：[service.rs](sfid/backend/src/institutions/service.rs) CLEARING_BANK_NAMES 注释更新，[INSTITUTIONS_TECHNICAL.md](memory/05-modules/sfid/backend/institutions/INSTITUTIONS_TECHNICAL.md) 补 2026-04-21 段
- **TS 镜像**：[deriveDuoqianAddress.ts](sfid/frontend/src/utils/deriveDuoqianAddress.ts) 重写为 3 路派生（按 name 值路由到 Main/Fee/Named）
- **Dart 镜像**：wuminapp 只有 OP_PERSONAL 派生，本次无改动；wumin 无本地派生
- **文档**：[BLAKE2_ADDRESS_DERIVATION.md](memory/05-modules/citizenchain/runtime/primitives/BLAKE2_ADDRESS_DERIVATION.md)、[DUOQIAN_TECHNICAL.md](memory/05-modules/citizenchain/runtime/transaction/duoqian-manage/DUOQIAN_TECHNICAL.md)、[feedback_sfid_sheng_signing_keyring.md](memory/feedback_sfid_sheng_signing_keyring.md)、[20260405-sfid-多签地址派生加入name字段](memory/08-tasks/open/20260405-sfid-多签地址派生加入name字段.md) 全部加 2026-04-21 尾注
- **验证**：10 个 pallet `cargo check` 全通过；`cargo test -p primitives` 7/7 通过；全仓库活代码 `rg derive_duoqian_address_from_sfid_id` 零残留

# 最终账户派生公式对齐

| 账户类型 | op_tag | preimage |
|---|---|---|
| 宪法机构主账户（NRC/PRC/PRB） | `OP_MAIN = 0x00` | `ss58 \|\| shenfen_id` |
| SFID 机构主账户（清算行/公安局/其他） | **`OP_MAIN = 0x00`** | **`ss58 \|\| sfid_id`**（与宪法公式完全一致，仅 sfid_id 命名空间不同） |
| 宪法机构费用账户 | `OP_FEE = 0x01` | `ss58 \|\| shenfen_id` |
| SFID 机构费用账户 | **`OP_FEE = 0x01`** | **`ss58 \|\| sfid_id`**（与宪法公式完全一致） |
| 省储行质押账户（仅 PRB） | `OP_STAKE = 0x02` | `ss58 \|\| citizens_number_u64_le` |
| 国储会安全基金（仅 NRC） | `OP_AN = 0x03` | `ss58 \|\| NRC_shenfen_id` |
| 个人多签 | `OP_PERSONAL = 0x04` | `ss58 \|\| creator_32 \|\| name` |
| **SFID 机构自定义命名账户**（临时/工资/运营/备用金...） | **`OP_INSTITUTION = 0x05`** | **`ss58 \|\| sfid_id \|\| name`** |

每个 op_tag 单一派生公式，无"if name empty"分支。保留名铁律：`"主账户"` / `"费用账户"` 链端强制走 `Role::Main` / `Role::Fee`，不能作为 `Role::Named` 参数（返回 `ReservedAccountName` 错误）。

# 背景

统一 DUOQIAN_V1 方案落地后，仍有一处语义不一致：`OP_MAIN = 0x00` 同时背负两个职责：
- name 空 → 宪法机构主账户（input = sfid_id）
- name 非空 → SFID 登记机构的任意命名账户（input = sfid_id + name）

`一 op_tag 双公式` 导致"费用账户"在宪法（OP_FEE）和 SFID（OP_MAIN+name="费用账户"）走不同路径，派生公式分裂。

# 目标

- 每个 op_tag **单一派生公式**（不再有"if name empty"分支）
- 宪法机构和 SFID 机构的 Main/Fee 账户使用**相同的派生公式**
- SFID 机构的自定义命名账户（临时/工资/运营...）独占 `OP_INSTITUTION = 0x05`

# 最终模型

| op_tag | 值 | 适用 | preimage |
|---|---|---|---|
| OP_MAIN | 0x00 | 所有机构主账户（宪法 + SFID） | ss58 ‖ sfid_id |
| OP_FEE | 0x01 | 所有机构费用账户（宪法 + SFID） | ss58 ‖ sfid_id |
| OP_STAKE | 0x02 | 仅 PRB 质押账户 | ss58 ‖ citizens_number_u64_le |
| OP_AN | 0x03 | 仅 NRC 安全基金 | ss58 ‖ NRC_sfid_id |
| OP_PERSONAL | 0x04 | 个人多签（creator + name） | ss58 ‖ creator_32 ‖ name |
| **OP_INSTITUTION** | **0x05** | **SFID 机构自定义命名账户** | **ss58 ‖ sfid_id ‖ name** |

# 校验规则（链端新增）

SFID 机构创建 `Named(name)` 账户时：
1. name 非空
2. name ≠ `"主账户"` 且 name ≠ `"费用账户"`（保留名必须走 OP_MAIN / OP_FEE，不走 OP_INSTITUTION）
3. name 长度 ≤ MaxSfidNameLength

# 执行清单

## 第 1 步：primitives 常量
- [ ] `primitives/src/core_const.rs` 加 `OP_INSTITUTION = 0x05`（注释说明适用范围）

## 第 2 步：链端 pallet 重构
- [ ] `duoqian-manage/src/lib.rs`：
  - 定义 `InstitutionAccountRole` 枚举（`Main` / `Fee` / `Named(BoundedVec<u8>)`）
  - `derive_duoqian_address_from_sfid_id(sfid_id, name)` → `derive_institution_address(sfid_id, role)`
  - 保留名校验：reject `Named("主账户")` / `Named("费用账户")`
  - 更新 `register_sfid_institution` extrinsic 参数（name → role 或内部翻译）
  - 更新 `AddressRegisteredSfid` / `SfidRegisteredAddress` 存储键值（按 role 归一化）

## 第 3 步：SFID 后端
- [ ] `sfid/backend/src/institutions/service.rs`：`CLEARING_BANK_NAMES` 改 `DEFAULT_CLEARING_BANK_ROLES`
- [ ] `sfid/backend/src/institutions/handler.rs`：创建账户调用点按 role 分流
- [ ] `sfid/backend/src/institutions/chain.rs`：链上 extrinsic 提交按 role

## 第 4 步：前端镜像
- [ ] `sfid/frontend/src/utils/deriveDuoqianAddress.ts`：3 种派生（Main / Fee / Named）
- [ ] `sfid/frontend/src/views/institutions/CreateAccountModal.tsx`：表单先选 role
- [ ] wuminapp Dart 镜像同步

## 第 5 步：验证
- [ ] `cargo check -p primitives -p duoqian-manage`
- [ ] `cargo test -p primitives`
- [ ] `cargo test -p duoqian-manage`（若有单元测试）

## 第 6 步：文档
- [ ] `primitives/BLAKE2_ADDRESS_DERIVATION.md` 加 OP_INSTITUTION 段
- [ ] `duoqian-manage/DUOQIAN_TECHNICAL.md` 更新派生公式表
- [ ] `sfid/backend/institutions/INSTITUTIONS_TECHNICAL.md` 更新账户模型
- [ ] `feedback_sfid_sheng_signing_keyring.md` 更新 op_tag 清单
- [ ] 历史任务卡 `20260408-sfid-机构账户两层模型-任务卡2` 加尾注

# 铁律

- fresh genesis（按 `feedback_chain_in_dev.md`）
- 不做兼容过渡（按 `feedback_no_compatibility.md`）
- 保留名列表硬编码在链上 extrinsic 校验层，后端/前端不能绕过
