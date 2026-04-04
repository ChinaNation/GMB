# BLAKE2-256 地址派生方案

## 概述

机构手续费账户地址和安全基金账户地址采用统一的确定性派生方案，基于 BLAKE2-256 哈希函数，从机构 shenfen_id 推导出唯一的链上地址。

## 派生公式

### 手续费账户（fee_address）

```
fee_address = BLAKE2-256("FEIYONG_SFID_V1" || SS58_PREFIX_LE || shenfen_id)
```

- `"FEIYONG_SFID_V1"`：域前缀字符串（UTF-8 字节）
- `SS58_PREFIX_LE`：SS58 地址格式前缀 2027 的小端字节表示（`[0xEB, 0x07]`）
- `shenfen_id`：机构身份标识字符串（UTF-8 字节）

### 安全基金账户（NRC_ANQUAN_ADDRESS）

```
NRC_ANQUAN_ADDRESS = BLAKE2-256("ANQUAN_SFID_V1" || SS58_PREFIX_LE || 国储会 shenfen_id)
```

使用不同的域前缀 `"ANQUAN_SFID_V1"`，其余逻辑相同。

## 设计理由

### 1. 抗碰撞性

BLAKE2-256 输出 256 位，碰撞概率为 2^(-128)，满足链上地址安全要求。没有对应的私钥，资金只能通过 pallet 治理逻辑（如 sweep、safety fund transfer）操作。

### 2. 链域隔离

SS58_PREFIX_LE 参与哈希输入，保证不同链（不同 SS58_FORMAT）派生出不同地址。CitizenChain 的 SS58_FORMAT = 2027，即使 shenfen_id 相同，与其他链的地址也不会冲突。

### 3. 用途隔离

手续费账户使用 `FEIYONG_SFID_V1` 前缀，安全基金使用 `ANQUAN_SFID_V1` 前缀，保证同一机构的不同用途账户地址不同。

### 4. 确定性

地址在编译期计算并硬编码为 `[u8; 32]` 常量，无运行时开销。所有节点使用相同地址，无需链上注册。

## 覆盖范围

### 国储会 + 省储委会（china_cb.rs）

- 1 个国储会 + 43 个省储委会 = **44 个 fee_address**
- 1 个安全基金地址 NRC_ANQUAN_ADDRESS
- 结构体：`ChinaCb { shenfen_id, fee_address, duoqian_address, ... }`

### 省储行（china_ch.rs）

- 43 个省储行 = **43 个 fee_address**
- 结构体：`ChinaCh { shenfen_id, fee_address, duoqian_address, ... }`

### 合计

- 手续费账户：44（CB）+ 43（CH）= **87 个 fee_address**
- 安全基金账户：**1 个 NRC_ANQUAN_ADDRESS**

## 核心常量

| 常量 | 值 | 定义位置 |
|------|------|----------|
| SS58_FORMAT | 2027 | core_const.rs |
| NRC_ANQUAN_ADDRESS | `045bdb35...884a37` | china_cb.rs |

## shenfen_id 编码

`shenfen_id_to_fixed48` 将 shenfen_id 字符串编码为固定 48 字节（右侧补零），用于链上 `InstitutionPalletId` 标识。此编码仅用于 pallet 存储键，与地址派生中的原始字节拼接无关。

## 示例

以国储会 shenfen_id `GFR-LN001-CB0C-617776487-20260222` 为例：

```
fee_address = BLAKE2-256(
    b"FEIYONG_SFID_V1"
    ++ [0xEB, 0x07]
    ++ b"GFR-LN001-CB0C-617776487-20260222"
)
= 40c1532dc0071e2dfc59a8f273f4f893bf51c6311e8d718c24721677ce02d203
```

安全基金地址：

```
NRC_ANQUAN_ADDRESS = BLAKE2-256(
    b"ANQUAN_SFID_V1"
    ++ [0xEB, 0x07]
    ++ b"GFR-LN001-CB0C-617776487-20260222"
)
= 045bdb35046c60c1346ba48e1e79049519edf4c009e40c7ecead1bebd1884a37
```

## 源码位置

- `citizenchain/runtime/primitives/china/china_cb.rs` - 国储会+省储委会常量（含 NRC_ANQUAN_ADDRESS）
- `citizenchain/runtime/primitives/china/china_ch.rs` - 省储行常量
- `citizenchain/runtime/primitives/src/core_const.rs` - SS58_FORMAT 定义
