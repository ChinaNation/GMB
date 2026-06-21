# 统一 BLAKE2-256 账户派生方案（DUOQIAN + op_tag）

## 概述

`citizenchain` 所有链上保留账户均通过 BLAKE2-256 确定性派生，统一使用单一 domain `DUOQIAN` 加 1 字节 `op_tag` 子命名空间区分用途。

```
address = BLAKE2-256(
    DUOQIAN (10B "DUOQIAN")
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
| `0x00` | `OP_MAIN` | **所有机构**主账户（宪法 + CID 登记） | `cid_number` | `scripts/duoqian.py` / 链上 `derive_institution_account(cid_number, Main)` |
| `0x01` | `OP_FEE` | **所有机构**费用账户（宪法 + CID 登记） | `cid_number` | `scripts/duoqian.py` / 链上 `derive_institution_account(cid_number, Fee)` |
| `0x02` | `OP_STAKE` | 省储行永久质押账户（仅 PRB） | `citizens_number_u64_le` | `scripts/duoqian.py` |
| `0x03` | `OP_AN` | 国储会安全基金账户（仅 NRC） | `cid_number`（国储会） | `scripts/duoqian.py` |
| `0x04` | `OP_HE` | 国储会两和基金账户（仅 NRC） | `cid_number`（国储会） | `scripts/duoqian.py` |
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
| `0x10` | `OP_SIGN_BIND` | 公民身份绑定签名 |
| `0x11` | `OP_SIGN_VOTE` | 公民投票签名 |
| `0x12` | `OP_SIGN_POP` | 人口快照签名 |
| `0x13` | `OP_SIGN_INST` | CID 机构登记签名 |
| `0x14` | `OP_SIGN_CREATE` | 注册多签离线聚合创建（预留） |
| `0x15` | `OP_SIGN_TRANSFER` | 多签转账离线聚合（预留） |

## 设计理由

**统一域**：密码学上 `DUOQIAN || op_tag` 等价于 N 个独立 domain（BLAKE2 扩散性保证无碰撞），但代码层只维护一个 domain 常量，减少出错面。

**op_tag 分段**：低半段（0x00-0x0F）留给账户派生，高半段（0x10-0x1F）留给签名 payload，易读易审。

**链域隔离**：`SS58_PREFIX_LE` 参与 preimage，保证不同链（不同 SS58 format）派生出不同地址。

**确定性**：字段全部 bake 在 primitives/china/，节点无需链上注册。`scripts/duoqian.py` 一次性生成并写回 `.rs` 源，禁止手改。

## 覆盖范围

| 类型 | 数量 | 文件 |
|---|---|---|
| main_account | 277 | china_cb(44) + china_ch(43) + china_zf(54) + china_jc(47) + china_lf(44) + china_sf(44) + china_jy(1) |
| fee_account | 87 | china_cb(44) + china_ch(43) |
| stake_account | 43 | china_ch |
| NRC_ANQUAN_ACCOUNT | 1 | china_cb（全局唯一常量） |
| `CHINA_RESERVED_MAIN_ACCOUNTS` 汇总 | 408 唯一项 | china_zb（由 scripts/duoqian.py 生成） |

## 示例

国储会 `cid_number = "LN001-GCB05-944805165-2026"`：

```
// main_account
preimage = b"DUOQIAN" || 0x00 || [0xEB, 0x07] || b"LN001-GCB05-944805165-2026"
address  = BLAKE2-256(preimage)

// fee_account
preimage = b"DUOQIAN" || 0x01 || [0xEB, 0x07] || b"LN001-GCB05-944805165-2026"
address  = BLAKE2-256(preimage)

// NRC_ANQUAN_ACCOUNT
preimage = b"DUOQIAN" || 0x03 || [0xEB, 0x07] || b"LN001-GCB05-944805165-2026"
address  = 0x0521c1ef5fe34fab5353b6213a559c8ca1044cc1972977b648b84cc2d954e4f6
```

中枢省储行（`citizens_number = 10_913_902`）：

```
// stake_account
preimage = b"DUOQIAN" || 0x02 || [0xEB, 0x07] || u64_le(10913902)
address  = BLAKE2-256(preimage)
```

## 源码位置

- [primitives/src/core_const.rs](../../../../../citizenchain/runtime/primitives/src/core_const.rs) — `DUOQIAN` + `OP_*` 常量定义
- [primitives/china/china_cb.rs](../../../../../citizenchain/runtime/primitives/china/china_cb.rs) — 国储会 + 省储会常量（含 `NRC_ANQUAN_ACCOUNT`）
- [primitives/china/china_ch.rs](../../../../../citizenchain/runtime/primitives/china/china_ch.rs) — 省储行常量（含 `stake_account`）
- [primitives/china/china_zb.rs](../../../../../citizenchain/runtime/primitives/china/china_zb.rs) — 汇总保留名单 + `is_reserved_main_account()`
- [organization-manage](../../../../../citizenchain/runtime/governance/organization-manage/src/lib.rs) — 链上 `derive_institution_account(cid_number, role)` + `derive_personal_account(creator, account_name)` + `role_from_account_name` 辅助
- [scripts/duoqian.py](../../../../../scripts/duoqian.py) — 统一生成器

## 当前约束

账户派生统一使用 `DUOQIAN + op_tag`。任务卡：[20260420-unified-DUOQIAN-domain](../../../../08-tasks/done/20260420-unified-DUOQIAN-domain.md)。

`OP_INSTITUTION = 0x06` 专供 CID 机构自定义命名账户，`OP_MAIN` / `OP_FEE` 只走 `preimage = ss58 || cid_number`，宪法机构和 CID 登记机构的主/费用账户派生公式一致。`derive_institution_account(cid_number, role)` 与 `role_from_account_name` 是当前辅助接口。保留名 `"主账户"`/`"费用账户"` 强制走 `Role::Main`/`Role::Fee`。任务卡：[20260421-op-institution-role-split](../../../../08-tasks/done/20260421-op-institution-role-split.md)。

链端账户名称字段统一为 `account_name` / `AccountNameOf<T>` / `MaxAccountNameLength` / `EmptyAccountName`，与 CID 后端 `MultisigAccount.account_name` 对齐。任务卡：[20260421-name-to-account-name-rename](../../../../08-tasks/done/20260421-name-to-account-name-rename.md)。
