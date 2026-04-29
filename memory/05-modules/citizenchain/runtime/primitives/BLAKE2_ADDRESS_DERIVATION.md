# 统一 BLAKE2-256 地址派生方案（DUOQIAN_V1 + op_tag）

## 概述

`citizenchain` 所有链上保留账户地址均通过 BLAKE2-256 确定性派生，统一使用单一 domain `DUOQIAN_V1` 加 1 字节 `op_tag` 子命名空间区分用途。

```
address = BLAKE2-256(
    DUOQIAN_DOMAIN (10B "DUOQIAN_V1")
 || op_tag (1B)
 || SS58_PREFIX_LE (2B, [0xEB, 0x07] = 2027)
 || payload（按 op_tag 规范）
)
```

常量定义见 [primitives/src/core_const.rs](../../../../../citizenchain/runtime/primitives/src/core_const.rs)。

## op_tag 分配

### 地址派生（0x00 - 0x0F）

| op_tag | 常量名 | 用途 | payload | 生成 |
|---|---|---|---|---|
| `0x00` | `OP_MAIN` | **所有机构**主账户（宪法 + SFID 登记） | `sfid_id` | `tools/duoqian.py` / 链上 `derive_institution_address(sfid_id, Main)` |
| `0x01` | `OP_FEE` | **所有机构**费用账户（宪法 + SFID 登记） | `sfid_id` | `tools/duoqian.py` / 链上 `derive_institution_address(sfid_id, Fee)` |
| `0x02` | `OP_STAKE` | 省储行永久质押账户（仅 PRB） | `citizens_number_u64_le` | `tools/duoqian.py` |
| `0x03` | `OP_AN` | 国储会安全基金账户（仅 NRC） | NRC `sfid_id` | `tools/duoqian.py` |
| `0x04` | `OP_PERSONAL` | 个人多签 | `creator_32 || name` | 链上 `derive_personal_duoqian_address` |
| `0x05` | `OP_INSTITUTION` | **仅 SFID 机构的自定义命名账户**（临时/工资/运营...） | `sfid_id || account_name` | 链上 `derive_institution_address(sfid_id, Named(account_name))` |

#### OP_MAIN / OP_FEE / OP_INSTITUTION 的语义分工

SFID 机构的账户名被链端硬翻译成 `InstitutionAccountRole`：

- `"主账户"` → `Role::Main` → `OP_MAIN`（preimage 不含 name）
- `"费用账户"` → `Role::Fee` → `OP_FEE`（preimage 不含 name）
- 其他非空 account_name → `Role::Named(account_name)` → `OP_INSTITUTION`（preimage 含 account_name）

保留名 `"主账户"` / `"费用账户"` 不允许作为 `Role::Named` 的自定义参数（链端返回 `ReservedAccountName` 错误）。这样保证同一角色在"宪法机构"和"SFID 登记机构"之间**派生公式完全一致**。

### 签名 payload（0x10 - 0x1F）

| op_tag | 常量名 | 用途 |
|---|---|---|
| `0x10` | `OP_SIGN_BIND` | 公民身份绑定签名 |
| `0x11` | `OP_SIGN_VOTE` | 公民投票签名 |
| `0x12` | `OP_SIGN_POP` | 人口快照签名 |
| `0x13` | `OP_SIGN_INST` | SFID 机构登记签名 |
| `0x14` | `OP_SIGN_CREATE` | 注册多签离线聚合创建（预留） |
| `0x15` | `OP_SIGN_TRANSFER` | 多签转账离线聚合（预留） |

## 设计理由

**统一域**：密码学上 `DUOQIAN_V1 || op_tag` 等价于 N 个独立 domain（BLAKE2 扩散性保证无碰撞），但代码层只维护一个 domain 常量，减少出错面。

**op_tag 分段**：低半段（0x00-0x0F）留给地址派生，高半段（0x10-0x1F）留给签名 payload，易读易审。

**链域隔离**：`SS58_PREFIX_LE` 参与 preimage，保证不同链（不同 SS58 format）派生出不同地址。

**确定性**：字段全部 bake 在 primitives/china/，节点无需链上注册。`tools/duoqian.py` 一次性生成并写回 `.rs` 源，禁止手改。

## 覆盖范围

| 类型 | 数量 | 文件 |
|---|---|---|
| main_address | 277 | china_cb(44) + china_ch(43) + china_zf(54) + china_jc(47) + china_lf(44) + china_sf(44) + china_jy(1) |
| fee_address | 87 | china_cb(44) + china_ch(43) |
| stake_address | 43 | china_ch |
| NRC_ANQUAN_ADDRESS | 1 | china_cb（全局唯一常量） |
| `CHINA_RESERVED_MAIN_ADDRESSES` 汇总 | 408 唯一项 | china_zb（由 tools/duoqian.py 生成） |

## 示例

国储会 `shenfen_id = "GFR-LN001-CB0C-617776487-20260222"`：

```
// main_address
preimage = b"DUOQIAN_V1" || 0x00 || [0xEB, 0x07] || b"GFR-LN001-CB0C-617776487-20260222"
address  = BLAKE2-256(preimage)

// fee_address
preimage = b"DUOQIAN_V1" || 0x01 || [0xEB, 0x07] || b"GFR-LN001-CB0C-617776487-20260222"
address  = BLAKE2-256(preimage)

// NRC_ANQUAN_ADDRESS
preimage = b"DUOQIAN_V1" || 0x03 || [0xEB, 0x07] || b"GFR-LN001-CB0C-617776487-20260222"
address  = 0x0521c1ef5fe34fab5353b6213a559c8ca1044cc1972977b648b84cc2d954e4f6
```

中枢省储行（`citizens_number = 10_913_902`）：

```
// stake_address
preimage = b"DUOQIAN_V1" || 0x02 || [0xEB, 0x07] || u64_le(10913902)
address  = BLAKE2-256(preimage)
```

## 源码位置

- [primitives/src/core_const.rs](../../../../../citizenchain/runtime/primitives/src/core_const.rs) — `DUOQIAN_DOMAIN` + `OP_*` 常量定义
- [primitives/china/china_cb.rs](../../../../../citizenchain/runtime/primitives/china/china_cb.rs) — 国储会 + 省储会常量（含 `NRC_ANQUAN_ADDRESS`）
- [primitives/china/china_ch.rs](../../../../../citizenchain/runtime/primitives/china/china_ch.rs) — 省储行常量（含 `stake_address`）
- [primitives/china/china_zb.rs](../../../../../citizenchain/runtime/primitives/china/china_zb.rs) — 汇总保留名单 + `is_reserved_main_address()`
- [duoqian-manage](../../../../../citizenchain/runtime/transaction/duoqian-manage/src/lib.rs) — 链上 `derive_institution_address(sfid_id, role)` + `derive_personal_duoqian_address(creator, account_name)` + `role_from_account_name` 辅助
- [tools/duoqian.py](../../../../../tools/duoqian.py) — 统一生成器

## 历史遗留（已彻底退役）

**2026-04-20**：原来分离的 domain 前缀全部退役：`DUOQIAN_SFID_V1` / `DUOQIAN_PERSONAL_V1` / `FEIYONG_SFID_V1` / `ANQUAN_SFID_V1` / `GMB_SFID_V1` / `GMB_SFID_BIND_V3` / `GMB_SFID_VOTE_V3` / `GMB_SFID_POPULATION_V3` / `GMB_SFID_INSTITUTION_V1/V2` 统一合并到 `DUOQIAN_V1 + op_tag`。任务卡：[20260420-unified-DUOQIAN_V1-domain](../../../../08-tasks/done/20260420-unified-DUOQIAN_V1-domain.md)。

**2026-04-21**：消除 `OP_MAIN` 双语义残留（原 `OP_MAIN + sfid_id + name` 对 SFID 机构所有账户通吃）。新增 `OP_INSTITUTION = 0x05` 专供 SFID 机构自定义命名账户，`OP_MAIN` / `OP_FEE` 只走 `preimage = ss58 || sfid_id`，宪法机构和 SFID 登记机构的主/费用账户**派生公式彻底一致**。`derive_duoqian_address_from_sfid_id` 重构为 `derive_institution_address(sfid_id, role)` + `role_from_account_name` 辅助。保留名 `"主账户"`/`"费用账户"` 强制走 `Role::Main`/`Role::Fee`。按 `feedback_no_compatibility.md` 死规则，不留旧方案。任务卡：[20260421-op-institution-role-split](../../../../08-tasks/done/20260421-op-institution-role-split.md)。

**2026-04-21 第二轮**：链端字段名 `name` / `SfidNameOf<T>` / `MaxSfidNameLength` / `EmptySfidName` 同步重命名为 `account_name` / `AccountNameOf<T>` / `MaxAccountNameLength` / `EmptyAccountName`，与 SFID 后端 `MultisigAccount.account_name` 彻底对齐。Dart 侧 `submitProposeCreate({name})` → `({accountName})`，wumin `payload_decoder.dart` JSON key `'name'` → `'account_name'`。字节零影响（SCALE 按位置编码）。任务卡：[20260421-name-to-account-name-rename](../../../../08-tasks/done/20260421-name-to-account-name-rename.md)。
