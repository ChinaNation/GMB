# SFID-CPMS QR v1

> 省级可见、站点匿名、档案号不透明

## 设计目标

1. SFID 必须能判断档案号是否由合法 CPMS 签发，非法直接拒绝
2. SFID 只知道档案号属于哪个省，不知道哪个市、哪台 CPMS；其他系统、其他人、其他 CPMS 也不知道；只有签发该档案号的 CPMS 本机知道是自己签发的

## 原则

- `site_sfid` 只用于 SFID 实名管理 CPMS
- `archive_no` 本体不带省、市、站点信息
- `province_code` 由档案二维码携带，并由匿名证书绑定
- CPMS 本地管理员密钥只在本机使用，不回传 SFID
- SFID 通过"省级匿名证书"判断合法性，不通过实名站点公钥判断

## 密码学方案

| 用途 | 算法 | 说明 |
|------|------|------|
| QR1 签名 | sr25519 | SFID 主密钥签 |
| 盲签名（匿名证书） | RSABSSA-SHA384-PSS-Randomized（RFC 9474） | SFID 单独维护 RSA 密钥对 |
| QR4 archive_sig | sr25519 | CPMS 本地 anon_keypair 签 |

## 省代码

全协议统一使用两位字母省码（与 `province.rs` 中的 `ProvinceCode.code` 一致）：`GD`、`LN`、`ZS`……

`site_sfid` 中 r5 段的前两位即为省码，与 `anon_cert.province_code`、`qr.province_code` 同一口径。

## 时间格式

全部使用 UTC RFC3339，例如 `2026-04-30T23:59:59Z`。

## 签名原文固定格式

### QR1 安装授权签名原文（sr25519）

```
sfid-cpms-install-v1|{site_sfid}|{install_token}
```

### anon_cert 证书签名原文（RSABSSA）

```
sfid-anon-cert-v1|{province_code}|{anon_pubkey}
```

### QR4 档案签名原文（sr25519）

```
cpms-archive-qr-v1|{province_code}|{archive_no}|{citizen_status}|{voting_eligible}
```

## 扫码总流程

```
QR1: SFID → CPMS    安装授权二维码
QR2: CPMS → SFID    注册请求二维码
QR3: SFID → CPMS    匿名证书二维码
QR4: CPMS → SFID    档案业务二维码
```

初始化只扫前 3 张。日常每条档案只扫第 4 张。

## QR1 — SFID → CPMS：安装授权

```json
{
  "ver": 1,
  "qr_type": "SFID_CPMS_INSTALL",
  "site_sfid": "GFR-GD001-ZG0X-123456789-20260330",
  "install_token": "b1c8f0c2e0d84a5cb7c0d0e2f9a8c1d3",
  "signature": "..."
}
```

| 字段 | 说明 |
|------|------|
| ver | 协议版本 |
| qr_type | 固定 `SFID_CPMS_INSTALL` |
| site_sfid | SFID 分配给该 CPMS 的实名身份号 |
| install_token | 一次性安装令牌，只能成功使用一次 |
| signature | SFID 主密钥 sr25519 签名，原文 `sfid-cpms-install-v1\|{site_sfid}\|{install_token}` |

CPMS 收到后先用预置的 SFID 公钥验签名，通过才接受安装。

## QR2 — CPMS → SFID：注册请求

```json
{
  "ver": 1,
  "qr_type": "CPMS_REGISTER_REQ",
  "site_sfid": "GFR-GD001-ZG0X-123456789-20260330",
  "install_token": "b1c8f0c2e0d84a5cb7c0d0e2f9a8c1d3",
  "blind_anon_req": "..."
}
```

| 字段 | 说明 |
|------|------|
| ver | 协议版本 |
| qr_type | 固定 `CPMS_REGISTER_REQ` |
| site_sfid | 本机初始化时读取到的实名站点号 |
| install_token | 安装令牌，回传给 SFID 校验 |
| blind_anon_req | 匿名公钥盲签申请，SFID 只负责签，不知道最终匿名公钥是谁 |

CPMS 初始化时本地生成：

- `local_admin_keypair`：本地管理员密钥，只本机使用
- `anon_keypair`：匿名签发密钥（sr25519），用于签档案二维码

两把私钥都不出机。

## QR3 — SFID → CPMS：匿名证书

```json
{
  "ver": 1,
  "qr_type": "SFID_ANON_CERT",
  "province_code": "GD",
  "blind_anon_sig": "..."
}
```

| 字段 | 说明 |
|------|------|
| ver | 协议版本 |
| qr_type | 固定 `SFID_ANON_CERT` |
| province_code | 该 CPMS 所属省代码，由 SFID 根据 site_sfid 自动确定 |
| blind_anon_sig | SFID 对盲请求的 RSABSSA 签名结果 |

CPMS 扫描这张码后，本机解盲，得到最终匿名证书：

```json
{
  "province_code": "GD",
  "anon_pubkey": "0x...",
  "sfid_sig": "..."
}
```

`sfid_sig` 为 RSABSSA-SHA384-PSS-Randomized 签名，原文 `sfid-anon-cert-v1|GD|0x...`。

匿名证书长期有效，只证明两件事：

1. 这是 SFID 认证过的某个合法 CPMS
2. 它属于某个省

不暴露：哪个市、哪台 CPMS、哪个 site_sfid。

## QR4 — CPMS → SFID：档案业务

```json
{
  "ver": 1,
  "qr_type": "CPMS_ARCHIVE_QR",
  "province_code": "GD",
  "archive_no": "AR4-K8M4ZP7W2Q1C9T6R5N3X8V2Y1A-7H",
  "citizen_status": "NORMAL",
  "voting_eligible": true,
  "anon_cert": {
    "province_code": "GD",
    "anon_pubkey": "0x...",
    "sfid_sig": "..."
  },
  "archive_sig": "..."
}
```

| 字段 | 说明 |
|------|------|
| ver | 协议版本 |
| qr_type | 固定 `CPMS_ARCHIVE_QR` |
| province_code | 档案所属省代码，只精确到省 |
| archive_no | 不透明档案号，不含市和站点语义 |
| citizen_status | 公民状态 |
| voting_eligible | 是否具备投票资格 |
| anon_cert | SFID 签发的省级匿名证书（完整嵌入） |
| archive_sig | 匿名私钥（sr25519）对核心字段的签名 |

`archive_sig` 原文 `cpms-archive-qr-v1|GD|AR4-K8M4ZP7W2Q1C9T6R5N3X8V2Y1A-7H|NORMAL|true`。

## 档案号格式

```
AR4-<26位Base32随机体>-<2位校验>
```

- 随机体必须用安全随机数生成
- 不编码省、市、站点、日期
- 只有签发本机因为本地数据库里有这条 archive_no，才知道"这是我发的"
- 其他 CPMS、SFID、其他系统都不能通过号体判断来源

## SFID 验证 QR4 的顺序

1. 用 SFID 的 RSA 公钥验 `anon_cert.sfid_sig`，原文 `sfid-anon-cert-v1|{province_code}|{anon_pubkey}`
2. 验 `anon_cert.province_code == qr.province_code`
3. 用 `anon_cert.anon_pubkey`（sr25519）验 `archive_sig`，原文 `cpms-archive-qr-v1|{province_code}|{archive_no}|{citizen_status}|{voting_eligible}`
4. 验 `archive_no` 未录入过
5. 全通过，以 `anon_cert.province_code` 为准落库

## install_token 状态管理

| 状态 | 说明 |
|------|------|
| PENDING | 已生成，未使用 |
| USED | 注册成功，已消费 |
| REVOKED | 管理员手工作废 |

二维码泄露后由 SFID 管理员手工作废并重发。

## 重新发证场景

匿名证书长期有效，仅以下场景需重新走 QR1 → QR2 → QR3：

1. 首次安装
2. 换机器或重装
3. SFID 管理员手工作废该安装令牌后重装
4. SFID 匿名 RSA 密钥整体轮换

每次重新发证必须重新生成新的 `anon_keypair`，禁止复用旧 `anon_pubkey`。

## SFID 端存储

### cpms_sites（机构管理）

| 字段 | 说明 |
|------|------|
| site_sfid | PK，实名站点号 |
| install_token | 安装令牌 |
| install_token_status | PENDING / USED / REVOKED |
| province_code | 省代码（从 site_sfid 提取） |
| created_by | 创建人 |
| created_at | 创建时间 |

不存 anon_pubkey，不存任何能关联匿名身份的字段。

### imported_archives（档案录入）

| 字段 | 说明 |
|------|------|
| archive_no | PK |
| province_code | 以验签通过后的 anon_cert.province_code 为准 |
| anon_cert_hash | 匿名证书摘要，用于审计 |
| imported_at | 录入时间 |
| status | ACTIVE / REVOKED |

## CPMS 端存储

### config（本机配置）

| 字段 | 说明 |
|------|------|
| site_sfid | 实名站点号 |
| install_token | 安装令牌 |
| local_admin_pubkey | 本地管理员公钥 |
| anon_pubkey | 匿名签发公钥 |
| anon_cert | 完整匿名证书 JSON |
| installed_at | 安装时间 |

私钥只在密钥库中，不出现在任何 QR 码或传输中。

### archives（档案记录）

| 字段 | 说明 |
|------|------|
| archive_no | PK |
| payload_json | 完整业务数据 |
| created_at | 创建时间 |

本机靠本地库是否存在 archive_no 判断"是不是我发的"，不靠档案号解析判断。
