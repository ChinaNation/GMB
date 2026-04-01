# SFID_CPMS_V1

> 省级可见、站点匿名、档案号不透明

## 设计目标

1. SFID 必须能判断档案号是否由合法 CPMS 签发，非法直接拒绝
2. SFID 只知道档案号属于哪个省，不知道哪个市、哪台 CPMS；其他系统、其他人、其他 CPMS 也不知道；只有签发该档案号的 CPMS 本机知道是自己签发的

## 原则

- `sfid`（机构身份识别码）只用于 SFID 实名管理 CPMS
- `ano`（档案号）本体不带省、市、站点信息
- `prov`（省代码）由档案二维码携带，并由匿名证书绑定
- CPMS 本地管理员密钥只在本机使用，不回传 SFID
- SFID 通过"省级匿名证书"判断合法性，不通过实名站点公钥判断

## 密码学方案

| 用途 | 算法 | 说明 |
|------|------|------|
| QR1 签名 | sr25519 | SFID 主密钥签 |
| 盲签名（匿名证书） | RSABSSA-SHA384-PSS-Randomized（RFC 9474） | SFID 单独维护 RSA 密钥对 |
| QR4 sig | sr25519 | CPMS 本地 anon_keypair 签 |

## 省代码

全协议统一使用两位字母省码（与 `province.rs` 中的 `ProvinceCode.code` 一致）：`GD`、`LN`、`ZS`……

`sfid` 中 r5 段的前两位即为省码，与 `cert.prov`、QR4 的 `prov` 同一口径。

## 签名原文固定格式

### QR1 安装授权签名原文（sr25519）

```
sfid-cpms-v1|install|{sfid}|{token}
```

### anon_cert 证书签名原文（RSABSSA）

```
sfid-anon-cert-v1|{prov}|{pk}
```

### QR4 档案签名原文（sr25519）

```
sfid-cpms-v1|archive|{prov}|{ano}|{cs}|{ve}
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
  "proto": "SFID_CPMS_V1",
  "type": "INSTALL",
  "sfid": "GFR-GD001-ZG0X-123456789-20260330",
  "token": "b1c8f0c2e0d84a5cb7c0d0e2f9a8c1d3",
  "rsa": "MIIBIjANBg...",
  "sig": "0x..."
}
```

| 字段 | 说明 |
|------|------|
| proto | 协议标识 `SFID_CPMS_V1` |
| type | 固定 `INSTALL` |
| sfid | SFID 分配给该 CPMS 的实名身份号 |
| token | 一次性安装令牌，只能成功使用一次 |
| rsa | SFID RSA 公钥（base64 裸数据，无 PEM 头尾） |
| sig | SFID 主密钥 sr25519 签名，原文 `sfid-cpms-v1|install|{sfid}|{token}` |

CPMS 收到后先用预置的 SFID 公钥验签名，通过才接受安装。RSA 公钥需重建 PEM 信封后使用。

## QR2 — CPMS → SFID：注册请求

```json
{
  "proto": "SFID_CPMS_V1",
  "type": "REGISTER",
  "sfid": "GFR-GD001-ZG0X-123456789-20260330",
  "token": "b1c8f0c2e0d84a5cb7c0d0e2f9a8c1d3",
  "blind": "0x..."
}
```

| 字段 | 说明 |
|------|------|
| proto | 协议标识 |
| type | 固定 `REGISTER` |
| sfid | 本机初始化时读取到的实名站点号 |
| token | 安装令牌，回传给 SFID 校验 |
| blind | 匿名公钥盲签申请，SFID 只负责签，不知道最终匿名公钥是谁 |

CPMS 初始化时本地生成 `anon_keypair`（sr25519），私钥不出机。

## QR3 — SFID → CPMS：匿名证书

```json
{
  "proto": "SFID_CPMS_V1",
  "type": "CERT",
  "prov": "GD",
  "bsig": "0x..."
}
```

| 字段 | 说明 |
|------|------|
| proto | 协议标识 |
| type | 固定 `CERT` |
| prov | 该 CPMS 所属省代码，由 SFID 根据 sfid 自动确定 |
| bsig | SFID 对盲请求的 RSABSSA 签名结果 |

CPMS 扫描这张码后，本机解盲，得到最终匿名证书：

```json
{
  "prov": "GD",
  "pk": "0x...",
  "sig": "0x...",
  "mr": "0x..."
}
```

`sig` 为 RSABSSA-SHA384-PSS-Randomized 签名，原文 `sfid-anon-cert-v1|GD|0x...`。`mr` 为消息随机化因子（验签时需要）。

匿名证书长期有效，只证明两件事：

1. 这是 SFID 认证过的某个合法 CPMS
2. 它属于某个省

不暴露：哪个市、哪台 CPMS、哪个 sfid。

## QR4 — CPMS → SFID：档案业务

```json
{
  "proto": "SFID_CPMS_V1",
  "type": "ARCHIVE",
  "prov": "GD",
  "ano": "AR4-K8M4ZP7W2Q1C9T6R5N3X8V2Y1A-7H",
  "cs": "NORMAL",
  "ve": true,
  "cert": {
    "prov": "GD",
    "pk": "0x...",
    "sig": "0x...",
    "mr": "0x..."
  },
  "sig": "0x..."
}
```

| 字段 | 说明 |
|------|------|
| proto | 协议标识 |
| type | 固定 `ARCHIVE` |
| prov | 档案所属省代码，只精确到省 |
| ano | 不透明档案号，不含市和站点语义 |
| cs | 公民状态（NORMAL/ABNORMAL） |
| ve | 是否具备投票资格 |
| cert | SFID 签发的省级匿名证书（完整嵌入） |
| sig | 匿名私钥（sr25519）对核心字段的签名 |

`sig` 原文 `sfid-cpms-v1|archive|GD|AR4-K8M4ZP7W2Q1C9T6R5N3X8V2Y1A-7H|NORMAL|true`。

## 档案号格式

```
AR4-<26位Base32随机体>-<2位校验>
```

- 随机体必须用安全随机数生成
- 不编码省、市、站点、日期
- 只有签发本机因为本地数据库里有这条档案号，才知道"这是我发的"

## SFID 验证 QR4 的顺序

1. 用 SFID 的 RSA 公钥验 `cert.sig`，原文 `sfid-anon-cert-v1|{prov}|{pk}`
2. 验 `cert.prov == prov`
3. 用 `cert.pk`（sr25519）验 `sig`，原文 `sfid-cpms-v1|archive|{prov}|{ano}|{cs}|{ve}`
4. 验 `ano` 未录入过
5. 全通过，以 `cert.prov` 为准落库

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

每次重新发证必须重新生成新的 `anon_keypair`，禁止复用旧匿名公钥。

## 协议族关系

| 协议 | 用途 | 使用场景 |
|------|------|----------|
| `SFID_CPMS_V1` | 两系统间业务交换 | QR1/QR2/QR3/QR4 |
| `WUMIN_LOGIN_V1.0.0` | 扫码登录 + 管理员绑定 | SFID/CPMS 登录页 ↔ 手机 App |
| `WUMIN_SIGN_V1.0.0` | 离线签名 | 绑定/解绑/轮换 ↔ 冷钱包 |
| `WUMIN_USER_V1.0.0` | 用户信息 | 区块链付款码/名片 |
