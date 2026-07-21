# 统一 BLAKE2-256 账户派生方案（GMB + op_tag）

## 概述

`citizenchain` 所有链上保留账户均通过 BLAKE2-256 确定性派生，统一使用单一 domain `GMB` 加 1 字节 `op_tag` 子命名空间区分用途。

```
address = BLAKE2-256(
    GMB (10B "GMB")
 || op_tag (1B)
 || SS58_PREFIX_LE (2B, [0xEB, 0x07] = 2027)
 || payload（按 op_tag 规范）
)
```

常量定义见 [primitives/src/core_const.rs](../../../../../citizenchain/runtime/primitives/src/core_const.rs)。

## op_tag 分配

### 账户派生（0x00 - 0x0F）

| op_tag | 常量名 | 用途 | payload | 生成 |
|---|---|---|---|---|
| `0x00` | `OP_MAIN` | **所有机构**主账户（宪法 + CID 登记） | `cid_number` | `scripts/multisig.py` / 链上 `derive_institution_account(cid_number, Main)` |
| `0x01` | `OP_FEE` | **所有机构**费用账户（宪法 + CID 登记） | `cid_number` | `scripts/multisig.py` / 链上 `derive_institution_account(cid_number, Fee)` |
| `0x02` | `OP_STAKE` | 省储行永久质押账户（仅 PRB） | `citizens_number_u64_le` | `scripts/multisig.py` |
| `0x03` | `OP_SAFETY` | 国家储委会安全基金账户（仅 NRC） | `cid_number`（国家储委会） | `scripts/multisig.py` |
| `0x04` | `OP_HE` | 国家储委会两和基金账户（仅 NRC） | `cid_number`（国家储委会） | `scripts/multisig.py` |
| `0x05` | `OP_PERSONAL` | 个人多签 | `creator_32 || name` | 链上 `derive_personal_account` |
| `0x06` | `OP_INSTITUTION` | **仅 CID 机构的自定义命名账户**（临时/工资/运营...） | `cid_number || account_name` | 链上 `derive_institution_account(cid_number, Named(account_name))` |

#### OP_MAIN / OP_FEE / OP_INSTITUTION 的语义分工

CID 机构的账户名被链端硬翻译成 `InstitutionAccountRole`：

- `"主账户"` → `Role::Main` → `OP_MAIN`（preimage 不含 name）
- `"费用账户"` → `Role::Fee` → `OP_FEE`（preimage 不含 name）
- 其他非空 account_name → `Role::Named(account_name)` → `OP_INSTITUTION`（preimage 含 account_name）

保留名 `"主账户"` / `"费用账户"` 不允许作为 `Role::Named` 的自定义参数（链端返回 `ReservedAccountName` 错误）。这样保证同一角色在"宪法机构"和"CID 登记机构"之间**派生公式完全一致**。

### 签名 payload（0x10 - 0x1F）

| op_tag | 常量名 | 用途 |
|---|---|---|
| `0x10` | `OP_SIGN_CITIZEN_IDENTITY` | 公民身份上链确认签名 |
| `0x13` | `OP_SIGN_INST` | 机构登记签名 |
| `0x14` | `OP_SIGN_DEREGISTER` | 机构/账户注销凭证签名 |
| `0x15` | `OP_SIGN_L3_PAY` | L3 支付签名 |
| `0x16` | `OP_SIGN_OFFCHAIN_BATCH` | 链下批次结算签名 |
| `0x17` | `OP_SIGN_L2_ACK` | L2 确认签名 |

## 设计理由

**统一域**：密码学上 `GMB || op_tag` 等价于 N 个独立 domain（BLAKE2 扩散性保证无碰撞），但代码层只维护一个 domain 常量，减少出错面。

**op_tag 分段**：低半段（0x00-0x0F）留给账户派生，高半段（0x10-0x1F）留给签名 payload，易读易审。

**链域隔离**：`SS58_PREFIX_LE` 参与 preimage，保证不同链（不同 SS58 format）派生出不同地址。

**确定性**：字段全部 bake 在 primitives/china/，节点无需链上注册。`scripts/multisig.py` 一次性生成并写回 `.rs` 源，禁止手改。

## 覆盖范围

| 类型 | 数量 | 文件 |
|---|---|---|
| main_account | 297 | 296 个公权内置机构 + 中国公民链技术发展基金会（私权非营利创世机构） |
| fee_account | 297 | 296 个公权内置机构 + 中国公民链技术发展基金会（私权非营利创世机构） |
| stake_account | 43 | china_ch |
| SAFETY_FUND_ACCOUNT | 1 | china_cb（全局唯一常量） |
| NRC_HE_ACCOUNT | 1 | china_cb（全局唯一常量） |
| `CHINA_RESERVED_MAIN_ACCOUNTS` 汇总 | 639 唯一项 | china_zb（内置主/费用账户、省储行质押账户、国家储委会安全基金和两和基金的排序去重表） |

## 示例

国家储委会 `cid_number = "LN001-NRC0G-944805165-2026"`：

```
// main_account
preimage = b"GMB" || 0x00 || [0xEB, 0x07] || b"LN001-NRC0G-944805165-2026"
address  = BLAKE2-256(preimage)

// fee_account
preimage = b"GMB" || 0x01 || [0xEB, 0x07] || b"LN001-NRC0G-944805165-2026"
address  = BLAKE2-256(preimage)

// SAFETY_FUND_ACCOUNT
preimage = b"GMB" || 0x03 || [0xEB, 0x07] || b"LN001-NRC0G-944805165-2026"
address  = 0xd78abac2e0a7772e72ba663313718e97288377d9ca2ca1467c710058f8b5effa
```

中枢省储行（`citizens_number = 10_913_902`）：

```
// stake_account
preimage = b"GMB" || 0x02 || [0xEB, 0x07] || u64_le(10913902)
address  = BLAKE2-256(preimage)
```

## 源码位置

- [primitives/src/core_const.rs](../../../../../citizenchain/runtime/primitives/src/core_const.rs) — `GMB` + `OP_*` 常量定义
- [primitives/china/china_cb.rs](../../../../../citizenchain/runtime/primitives/cid/china/china_cb.rs) — 国家储委会 + 省储委会常量（含 `SAFETY_FUND_ACCOUNT`）
- [primitives/china/china_ch.rs](../../../../../citizenchain/runtime/primitives/cid/china/china_ch.rs) — 省储行常量（含 `stake_account`）
- [primitives/china/china_zb.rs](../../../../../citizenchain/runtime/primitives/cid/china/china_zb.rs) — 汇总保留名单 + `is_reserved_main_account()`
- [public-manage](../../../../../citizenchain/runtime/entity/public-manage/src/lib.rs) / [private-manage](../../../../../citizenchain/runtime/entity/private-manage/src/lib.rs) — 链上公权/私权机构账户派生与 CID 注册账户登记
- [personal-manage](../../../../../citizenchain/runtime/entity/personal-manage/src/lib.rs) — 链上个人多签账户派生、创建、关闭与清理
- [scripts/multisig.py](../../../../../scripts/multisig.py) — 统一生成器

## 当前约束

账户派生统一使用 `GMB + op_tag`。任务卡：[20260420-unified-GMB-domain](../../../../08-tasks/done/20260420-unified-GMB-domain.md)。

`OP_INSTITUTION = 0x06` 专供 CID 机构自定义命名账户，`OP_MAIN` / `OP_FEE` 只走 `preimage = ss58 || cid_number`，宪法机构和 CID 登记机构的主/费用账户派生公式一致。`derive_institution_account(cid_number, role)` 与 `role_from_account_name` 是当前辅助接口。保留名 `"主账户"`/`"费用账户"` 强制走 `Role::Main`/`Role::Fee`。任务卡：[20260421-op-institution-role-split](../../../../08-tasks/done/20260421-op-institution-role-split.md)。

链端账户名称字段统一为 `account_name` / `AccountNameOf<T>` / `MaxAccountNameLength` / `EmptyAccountName`，与 OnChina 后端 `MultisigAccount.account_name` 对齐。任务卡：[20260421-name-to-account-name-rename](../../../../08-tasks/done/20260421-name-to-account-name-rename.md)。
