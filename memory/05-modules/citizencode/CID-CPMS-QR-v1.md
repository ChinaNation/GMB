# CID_CPMS_V1

> 两码协议：CID 安装码 + CPMS 公民档案码。

## 设计目标

1. CID 能确认档案码是否由已授权 CPMS 签发，未授权 CPMS 或伪造码必须拒绝。
2. CID 能知道档案号属于哪个公安局机构 `cid_number`，并按 `geo_seal` 中的代码记录居住地投票范围、出生地参选范围和市/镇精度。
3. 档案号 `archive_no` 本体不携带省、市、机构、日期等可识别信息。
4. 其他 CPMS、其他人和普通扫码方不能从档案码明文看出签发城市或机构。
5. 只有签发该档案号的 CPMS 本机，以及 CID 系统，知道档案号归属。

## 协议常量

- 协议名固定为 `CID_CPMS_V1`，不得新建或派生其他协议名。
- 机构 CID 字段固定为 `cid_number`，对外协议不得使用其他命名。
- 二维码类型只允许：
  - `INSTALL`：CID 签发给 CPMS，用于安装初始化。
  - `ARCHIVE`：CPMS 签发给 CID，用于公民电子护照绑定。

## INSTALL - CID 到 CPMS

```json
{
  "proto": "CID_CPMS_V1",
  "type": "INSTALL",
  "cid_number": "GD001-GZF06-123456789-2026",
  "province_name": "广东省",
  "city_name": "广州市",
  "install_secret": "0x...",
  "sig": "0x..."
}
```

| 字段 | 说明 |
|---|---|
| `proto` | 固定 `CID_CPMS_V1` |
| `type` | 固定 `INSTALL` |
| `cid_number` | 市公安局机构 CID 号，省市代码由该字段解码 |
| `province_name` | CPMS 离线显示用省名，来自 CID 省市真源 |
| `city_name` | CPMS 离线显示用市名，来自 CID 省市真源 |
| `install_secret` | 每个 CPMS 安装独有密钥材料，CPMS 必须本机安全保存 |
| `sig` | CID 主密钥对 INSTALL 核心字段签名 |

INSTALL 安装码不再额外加密。CPMS 初始化只做本地防误装校验：协议类型、字段格式、
`cid_number` 省市代码、`province_name/city_name` 与 CID 行政区真源一致性，以及
`install_secret` 格式。未授权或伪造 CPMS 签出的 ARCHIVE 由 CID 在档案绑定阶段最终拒绝。

INSTALL 签名原文：

```text
cid-cpms-v1|install|{cid_number}|{province_name}|{city_name}|{install_secret_hash}
```

`install_secret_hash = blake2b_256(install_secret)`。

CPMS 离线安装时保存 INSTALL 安装材料；档案码真实性由 CID 在 ARCHIVE 验真阶段通过授权记录、`geo_seal` 和 CPMS 本机签名闭环确认。

## ARCHIVE - CPMS 到 CID

```json
{
  "proto": "CID_CPMS_V1",
  "type": "ARCHIVE",
  "archive_no": "K8M4ZP7W2Q1C9T6R5N3X8V2Y1A-7H",
  "citizen_status": "NORMAL",
  "voting_eligible": true,
  "valid_from": "2026-05-24",
  "valid_until": "2036-05-23",
  "status_updated_at": 1779580800,
  "cpms_pubkey": "0x...",
  "geo_seal": "g1.<nonce_hex>.<cipher_hex>",
  "wallet_address": "5...",
  "wallet_pubkey": "0x...",
  "wallet_sig_alg": "sr25519",
  "sig": "0x..."
}
```

| 字段 | 说明 |
|---|---|
| `archive_no` | 公民档案号，必须全局唯一且不编码归属信息 |
| `citizen_status` | CPMS 公民状态，`NORMAL` 或 `REVOKED` |
| `voting_eligible` | CPMS 选举资格，`true` 表示有选举权利 |
| `valid_from` | 档案所属电子护照有效期开始日期，格式 `YYYY-MM-DD` |
| `valid_until` | 档案所属电子护照有效期截止日期，格式 `YYYY-MM-DD` |
| `status_updated_at` | CPMS 公民状态更新时间，Unix 秒 |
| `cpms_pubkey` | CPMS 本机签发公钥 |
| `geo_seal` | 只有 CID 能按安装授权解开的归属密文 |
| `wallet_address` | citizenapp 钱包地址，由 CPMS 扫 citizenapp 钱包二维码保存 |
| `wallet_pubkey` | `wallet_address` 对应的 0x 公钥 |
| `wallet_sig_alg` | 钱包签名算法，固定 `sr25519` |
| `sig` | CPMS 本机私钥对档案核心字段签名 |

ARCHIVE 签名原文：

```text
cid-cpms-v1|archive|{archive_no}|{citizen_status}|{voting_eligible}|{valid_from}|{valid_until}|{status_updated_at}|{cpms_pubkey}|{geo_seal_hash}|{wallet_address}|{wallet_pubkey}
```

ARCHIVE 不包含 `code_id` 或 `usage_limit`。档案码不是一次性票据；CID 以 `archive_no / cid_number / wallet_pubkey` 三者一对一关系防止重复绑定。

`geo_seal` 明文结构：

```json
{
  "cid_number": "GD001-GZF06-123456789-2026",
  "residence": {
    "province_code": "GD",
    "city_code": "001",
    "town_code": null
  },
  "birthplace": {
    "province_code": "GD",
    "city_code": "001",
    "town_code": null
  },
  "election_scope_level": "CITY"
}
```

`residence / birthplace` 只允许行政区代码，不允许中文名称。`election_scope_level` 只允许：

- `PROVINCE`：只携带居住省、出生省；市镇为空。
- `CITY`：携带居住省市、出生省市；镇为空。
- `TOWN`：携带居住省市镇、出生省市镇；注册镇时必须同时注册市。

`geo_seal` 使用 AES-256-GCM：

- 密钥：`blake2b_256(install_secret)`
- AAD：`cid-cpms-v1|geo-seal|{archive_no}|{cpms_pubkey}`

## 全局唯一性

- `install_secret` 每个 CPMS 安装独有，参与 `geo_seal` 密钥派生。
- CPMS 生成 `archive_no` 时必须使用安全随机数。
- CID 绑定 ARCHIVE 时必须以 `archive_no` 做全局唯一检查；已有档案号必须拒绝。

## CPMS_STATUS_EXPORT - CPMS 到 CID

CPMS 每年通过离线 JSON 文件向 CID 更新本 CPMS 内档案号对应的公民状态和投票资格。该文件由管理员手工导出和导入，不改变 CPMS 永不联网边界。

```json
{
  "proto": "CID_CPMS_V1",
  "type": "CPMS_STATUS_EXPORT",
  "version": 1,
  "export_year": 2026,
  "cid_number": "GD001-GZF06-123456789-2026",
  "cpms_pubkey": "0x...",
  "export_batch_id": "cse_...",
  "exported_at": 1780185600,
  "citizen_binding_records_count": 1,
  "binding_release_records_count": 1,
  "records_hash": "0x...",
  "citizen_binding_records": [
    {
      "archive_no": "K8M4ZP7W2Q1C9T6R5N3X8V2Y1A-7H",
      "wallet_address": "5...",
      "wallet_pubkey": "0x...",
      "wallet_sig_alg": "sr25519",
      "wallet_bound_at": 1780185600,
      "citizen_status": "NORMAL",
      "voting_eligible": true,
      "status_updated_at": 1780185600
    }
  ],
  "binding_release_records": [
    {
      "archive_no": "OLDARCHIVE",
      "released_at": 4933872000,
      "release_reason": "ARCHIVE_HARD_DELETED_AFTER_100_YEARS"
    }
  ],
  "sig": "0x..."
}
```

状态规则：

- CPMS 从每年 UTC 1 月 1 日起允许管理员导出上一年度更新数据；若存在多年未导出，按最早未导出年度依次补导。
- 导出记录按 `export_year` 表示所属年度；`citizen_binding_records` 是导出时 CPMS 当前仍有钱包绑定的档案快照，`binding_release_records` 只包含该年度内硬删除释放时间落入范围的档案号释放记录。
- UTC 1 月 11 日起，如果存在超过 1 月 10 日仍未导出的年度报告，CPMS 锁定操作员登录和操作，管理员仍可登录补导。
- `citizen_status=NORMAL` 表示正常；只有 `voting_eligible=true` 的记录允许保留在 CID 公民库。
- `citizen_status=REVOKED` 表示注销；此时 `voting_eligible` 必须为 `false`。
- CPMS 软删除档案就是注销，用 `citizen_binding_records` 中的 `citizen_status=REVOKED / voting_eligible=false` 通知 CID 删除该档案号对应的本地公民记录。
- `citizen_status=NORMAL / voting_eligible=false` 表示该公民当前无投票资格；CID 导入年度报告时同样删除对应本地公民记录。
- `citizen_status=NORMAL / voting_eligible=true` 但钱包字段缺失、算法不为 `sr25519`、地址和公钥不匹配或时间字段无效时，CID 也按无效公民删除本地记录，不允许继续保留旧投票账户。
- CPMS 100 年硬删除后，使用 `binding_release_records` 告知 CID 删除该档案号与身份 ID、钱包地址的绑定关系；护照号属于 CPMS 内部号码，不导出给 CID。
- CID 导入前必须确认 `cid_number` 对应 CPMS 授权有效，且 `cpms_pubkey` 已通过 ARCHIVE 档案码验真绑定；未绑定公钥的年度报告不得作为首次信任来源。

签名原文：

```text
cid-cpms-v1|cpms-status-export|{cid_number}|{cpms_pubkey}|{export_batch_id}|{exported_at}|{records_hash}
```

`records_hash = blake2b_256(json({citizen_binding_records, binding_release_records}))`。

导出文件不得包含姓名、出生日期、地址、护照号等实名或 CPMS 内部号码；钱包地址和钱包公钥属于 CID 覆盖绑定状态所需字段。

## CID 验证 ARCHIVE 顺序

1. 校验 `proto=CID_CPMS_V1`、`type=ARCHIVE`。
2. 用已授权 CPMS 安装记录中的 `install_secret` 和 AAD 尝试解开 `geo_seal`。
3. 校验 `geo_seal.cid_number` 与 CID 授权记录一致。
4. 校验 `geo_seal.election_scope_level` 与 `residence / birthplace` 代码精度一致；未注册市镇时不得夹带市镇代码。
5. 从 `cid_number` 解码 CPMS 授权分区 `province_code / city_code`；该分区只表示档案码来自哪个公安局授权安装。
6. 校验 CPMS 本机签名 `sig`。
7. 首次验真成功时绑定 `cpms_pubkey_hash`；后续同一授权只能接受同一 CPMS 本机公钥。
8. CID 确认 `citizen_status=NORMAL`、`voting_eligible=true`，且 `wallet_address / wallet_pubkey / wallet_sig_alg` 已由 CPMS 档案码携带；否则不得录入 CID 公民库。
9. CID 根据 ARCHIVE 生成 citizenapp `sign_request`，并锁定 `wallet_address / wallet_pubkey`。
10. citizenapp 返回 `sign_response` 后，CID 校验签名和 `payload_hash`。
11. CID 检查 `archive_no / cid_number / wallet_pubkey` 三者唯一，并按 CPMS 授权分区写入 `citizens.province_code / city_code`；按 `geo_seal` 另存 `residence_* / birth_* / election_scope_level`。后续投票区域按居住地判断，参选区域按出生地判断，具体流程只由投票引擎实现。

## 授权状态

| 状态 | 说明 |
|---|---|
| `PENDING` | 已签发 INSTALL，等待 CPMS 首次提交有效 ARCHIVE |
| `ACTIVE` | 已绑定 CPMS 本机签发公钥，可继续接收档案码 |
| `DISABLED` | 管理员暂停接收 |
| `REVOKED` | 管理员吊销，不再接收 |

## 协议族关系

| 协议 | 用途 | 使用场景 |
|---|---|---|
| `CID_CPMS_V1` | CID 与 CPMS 业务交换 | `INSTALL` / `ARCHIVE` |
| `CITIZEN_QR_V1` | 扫码登录、离线签名、用户联系和收款 | CID/CPMS/citizenapp |
