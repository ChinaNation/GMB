# CPMS ARCHIVE 投票账户

- 最后更新:2026-05-31
- 任务卡:`memory/08-tasks/done/20260531-cpms档案码生成完整性校验.md`

## 0. 行政区与档案基础字段

- 行政区数据唯一源是 SFID 系统 `sfid/backend/china/china.sqlite`，CPMS 源码树不得保存行政区第二份源码。
- CPMS 后端只允许通过 SFID 行政区真源导出的只读数据使用省市镇村信息；
  通用发行版只内置编译后的只读行政区数据。
- 一个 SFID INSTALL 安装码只对应一个市公安局；CPMS 初始化后只启用安装码 R5 段对应城市的
  镇/街道和村/路。
- 公民档案创建/编辑时，出生日期、性别、身高均为必填；出生日期使用 `YYYY-MM-DD`，
  身高必须在 `30-260 cm`。
- 选举资格必须同时满足公民状态 `NORMAL` 和已满 16 周岁；未满 16 周岁不得保存为有选举资格。
- CPMS 前端所有日期输入统一使用 `cpms/frontend/components/DateInput.tsx`；出生日期类输入
  默认最大日期为昨天，列表搜索、创建档案和编辑档案保持同一输入行为。
- 公民姓名字段统一为 `last_name / first_name`，数据库、后端 API 和前端类型保持一致。

## 0.1 管理员权限

CPMS 只有 `SUPER_ADMIN` 和 `OPERATOR_ADMIN` 两种管理员。`SUPER_ADMIN` 是上级角色，
可以执行所有档案业务操作；`OPERATOR_ADMIN` 可以执行日常档案业务，但不能查看系统设置，
也不能创建、编辑或删除管理员。初始化绑定的超级管理员不可删除并固定在管理员列表第一行；
后续新增的超级管理员可以删除。超级管理员总数最多 5 个，所有管理员只允许编辑姓名。管理员
删除是物理删除，并同步清理其本机会话。

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
→ CPMS 使用浏览器摄像头扫描投票账户地址二维码
→ CPMS 从 SS58 地址解析 wallet_pubkey
→ CPMS 保存 wallet_address / wallet_pubkey
→ 满足完整性门槛后生成 ARCHIVE 档案码
```

CPMS 只接受能解析出 AccountId32 的钱包地址；解析失败返回 `invalid wallet_address`。
同一个钱包账户在档案生命周期内只能绑定一个公民档案。软删除档案仍占用钱包账户、档案号和
护照号，满 100 年硬删除并物理删除档案后才释放；新增或回收复用号码时不得绕过该唯一约束。

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

无钱包地址时，CPMS 不允许生成 ARCHIVE。生成或打印 ARCHIVE 前，后端还必须确认姓氏、名字、
性别、身高、出生日期、护照号、有效期、省份、城市、公民状态、选举资格、投票账户、照片和
出生纸齐全。公民状态必须为 `NORMAL`，选举资格必须为 `true`，照片和出生纸各至少 1 张。

## 5. 档案码操作

档案码在保存投票账户并满足完整性门槛后才可签出。详情页按钮统一为:

```text
更新 / 下载 / 打印
```

- 更新：刷新当前 ARCHIVE 二维码内容。
- 下载：下载当前二维码图片。
- 打印：记录打印审计并调用浏览器打印；打印媒体只输出“公民档案详情”卡片，不输出页面导航
  或删除/编辑/返回列表/更换/更新/下载/打印等操作按钮。
- 编辑实名字段、修改公民状态、绑定/更换投票账户、上传资料或删除资料会清空旧 `archive_qr_payload`；
  管理员必须重新点击“更新”，后端重新校验完整性并签发新的 ARCHIVE。

## 6. 软删除

公民档案删除必须走 wumin 签名确认，不能物理删除。CPMS 创建 `WUMIN_QR_V1 / sign_request`
删除签名请求，当前登录管理员使用 wumin 签名后返回 `sign_response`。删除签名请求锁定当前
登录 CPMS 管理员的 `address / pubkey`，其中二维码 `body.pubkey` 和删除 payload 中的
`admin_pubkey` 必须是 `0x` + 64 位小写 hex。

删除 payload 固定为:

```text
CPMS_ARCHIVE_DELETE_V1|challenge_id|archive_id|archive_no|0x_admin_pubkey|expires_at
```

后端只接受当前登录管理员本人公钥签出的回执。完成接口先锁定删除 challenge 和档案行；
验签成功后在同一事务内消费 challenge、软删除档案并写入成功审计，避免重复提交:

```text
status = DELETED
citizen_status = REVOKED
voting_eligible = false
deleted_at = 当前时间
deleted_by = 当前登录管理员 user_id
delete_reason = wumin signed archive delete
```

列表默认不显示已删除档案；已删除档案禁止编辑、绑定/更换投票账户、更新、下载和打印。
challenge 不存在、已消费、过期、档案或管理员不匹配、签名人不匹配、payload hash
不一致、签名时间超出窗口或 sr25519 验签失败时，后端写入 `DELETE_ARCHIVE / FAILED`
审计；失败分支不消费 challenge，也不修改档案。
