# CPMS ARCHIVE 投票账户

- 最后更新:2026-05-27
- 任务卡:`memory/08-tasks/open/20260526-cpms-wallet-address-only.md`

## 0. 行政区与档案基础字段

- 行政区数据唯一源是 SFID 系统 `sfid/backend/sfid`，CPMS 源码树不得保存行政区第二份源码。
- CPMS 后端编译期直接引用 `sfid/backend/sfid/province.rs` 及其同目录 `city_codes/*.rs`；
  通用发行版只内置编译后的只读行政区数据。
- 一个 SFID INSTALL 安装码只对应一个市公安局；CPMS 初始化后只启用安装码 R5 段对应城市的
  镇/街道和村/路。
- 公民档案创建/编辑时，出生日期、性别、身高均为必填；出生日期使用 `YYYY-MM-DD`，
  身高必须在 `30-260 cm`。
- 公民姓名字段统一为 `last_name / first_name`，数据库、后端 API 和前端类型保持一致。

## 0.1 管理员权限

CPMS 只有 `SUPER_ADMIN` 和 `OPERATOR_ADMIN` 两种管理员。`SUPER_ADMIN` 是上级角色，
可以执行所有档案业务操作；`OPERATOR_ADMIN` 可以执行日常档案业务，但不能查看系统设置，
也不能创建、停用或删除操作员。

投票账户绑定/更换属于档案业务，允许 `SUPER_ADMIN` 和 `OPERATOR_ADMIN` 操作。
公民状态修改同样属于档案业务，两种管理员都可以操作。

## 1. 目标

CPMS 是线下实名档案系统，只负责确认用户现场出示的投票账户地址，并把该账户写入公民真实档案。
CPMS 不生成钱包签名 challenge，不保存钱包签名，不验证用户钱包私钥控制权。

钱包私钥控制权验证统一放在 SFID 绑定阶段：SFID 扫描 CPMS 出具的档案码后，要求 wumin 对 SFID
绑定 challenge 签名，并校验签名公钥等于档案码中的 `wallet_pubkey`。

## 2. 档案字段

`archives` 增加:

| 字段 | 含义 |
|---|---|
| `wallet_address` | 用户钱包 SS58 地址 |
| `wallet_pubkey` | 从 SS58 地址解析出的 32 字节公钥,`0x` hex |
| `wallet_sig_alg` | 固定 `sr25519` |
| `wallet_bound_at` | 钱包地址保存/更新时间 |
| `wallet_bound_by` | 执行保存的钱包管理员 user_id |

## 3. CPMS 保存流程

```text
创建档案
→ 用户在 wumin 电子护照页选择投票账户
→ wumin 展示投票账户地址二维码（WUMIN_QR_V1 / user_contact）
→ CPMS 档案详情页点击“投票账户”地址框右侧扫码图标或“更换”
→ CPMS 扫描投票账户地址二维码
→ CPMS 从 SS58 地址解析 wallet_pubkey
→ CPMS 保存 wallet_address / wallet_pubkey
→ 生成 ARCHIVE 档案码
```

CPMS 只接受能解析出 AccountId32 的钱包地址；解析失败返回 `invalid wallet_address`。

## 4. ARCHIVE 字段

`ARCHIVE` 必须包含:

```json
{
  "archive_no": "K8M4ZP7W2Q1C9T6R5N3X8V2Y1A-7H",
  "citizen_status": "NORMAL",
  "voting_eligible": true,
  "valid_from": "2026-05-24",
  "valid_until": "2036-05-23",
  "status_updated_at": 1779580800,
  "wallet_address": "5...",
  "wallet_pubkey": "0x...",
  "wallet_sig_alg": "sr25519"
}
```

CPMS 签名原文:

```text
sfid-cpms-v1|archive|{archive_no}|{citizen_status}|{voting_eligible}|{valid_from}|{valid_until}|{status_updated_at}|{cpms_pubkey}|{geo_seal_hash}|{wallet_address}|{wallet_pubkey}
```

无钱包地址时，CPMS 不允许生成 ARCHIVE。

## 5. 档案码操作

档案码在保存投票账户后即可签出。详情页按钮统一为:

```text
更新 / 下载 / 打印
```

- 更新：刷新当前 ARCHIVE 二维码内容。
- 下载：下载当前二维码图片。
- 打印：记录打印审计并调用浏览器打印。

## 6. 软删除

公民档案删除必须走 wumin 签名确认，不能物理删除。CPMS 创建 `WUMIN_QR_V1 / sign_request`
删除签名请求，当前登录管理员使用 wumin 签名后返回 `sign_response`。后端只接受当前登录
管理员本人公钥签出的回执，验签成功后设置:

```text
status = DELETED
deleted_at = 当前时间
deleted_by = 当前登录管理员 user_id
delete_reason = wumin signed archive delete
```

列表默认不显示已删除档案；已删除档案禁止编辑、绑定/更换投票账户、更新、下载和打印。
