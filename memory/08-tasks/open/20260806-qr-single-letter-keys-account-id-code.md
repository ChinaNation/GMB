# 任务卡：二维码字段全单字母键 + 钱包码改名账户码(全仓统一)

状态：进行中(2026-08-06)

## 任务需求(用户确认的四点)

1. **删除 `display_name`**(用户码)与 **`recipient_name`**(收款码)——本机可随意改写的
   字符串一旦进码就会被扫码端当对方公开身份显示,是冒名风险;真实昵称应由扫码端按
   `cid_number` 从服务端拉取。
2. **收款码不带 `cid_number`**。
3. **「钱包码」改名「账户码」**:钱包由多个账户组成,**钱包没有码,账户才有码**。
   枚举 `walletCode` → `accountIdCode`(用户写 `accountidCode`,按仓库死规则
   「账户全仓统一 `account_id`/`accountId`」归位 `Id` 大小写,词根未改)。
4. **body 键全部统一为单字母**(此前 k=1/k=2 单字母、k=3/4/5 snake_case 长键,两套风格)。
   同时 `ss58_address` → `account_id`:SS58 是给人看的,机器一律用 `account_id`。

全仓库统一、彻底执行,冷热两端(CitizenApp / CitizenWallet)同一次改完。

## 单字母全局注册表(一字母 = 一含义,跨所有码型唯一)

信封层:`p` 协议 / `k` 码型 / `i` 临时码 id / `e` 过期毫秒 / `b` body

body 层:

| 键 | 含义 | 编码 | 出现于 |
|---|---|---|---|
| `a` | 业务动作码 | int | k=1 |
| `g` | 签名算法(1=sr25519) | int | k=1 |
| `u` | 签名者公钥 | base64url | k=1, k=2 |
| `d` | 审阅载荷 | base64url | k=1 |
| `s` | 签名 | base64url | k=2 |
| `o` | 换绑时当前账户 | base64url | k=2 |
| `r` | 换绑时当前账户签名 | base64url | k=2 |
| `c` | cid_number 身份主键 | 文本 | k=3 |
| `n` | account_id 账户标识 | `0x` 小写 64 hex | k=3, k=4, k=5 |
| `v` | 金额 | 文本 | k=4 |
| `t` | 币种 | 文本 | k=4 |
| `m` | 备注 | 文本 | k=4 |
| `l` | 收款方清算行 CID | 文本 | k=4 |

**`u` 与 `n` 都是 32 字节公钥但必须两个字母**:编码不同(`u` base64url 压体积、
`n` `0x` 小写 hex 走 `isAccountIdText` 单源校验)。同一字母两种编码 = 解析器按码型
猜格式,是明确埋雷。连编码一起统一会作废冷热两端全部签名金标向量,不在本卡范围。

## 五种码定稿

- **k=1 签名请求**(临时,不改):`a` `g` `u` `d`
- **k=2 签名响应**(临时,不改):`u` `s` `o?` `r?`(`o`/`r` 换绑时成对出现)
- **k=3 用户码**(人,固定):`c` `n` —— 唯一能写入通讯录的码
- **k=4 收款码**(一笔收款,临时):`n` `v` `t` `m` `l`
- **k=5 账户码**(账户,固定):`n` —— 冷热两端都能生成

## 命名改动

| | 旧 | 新 |
|---|---|---|
| 中文 | 钱包码 | 账户码 |
| 枚举 | `walletCode` | `accountIdCode` |
| body 类 | `WalletCodeBody` | `AccountIdCodeBody` |
| 文件 | `wallet_code_body.dart` | `account_id_code_body.dart` |

## 破坏性说明

旧码全部作废。冷热两端 body 定义各一份镜像,必须同一次改完(双端一致铁律)。
按仓库「开发期零用户、绝不兼容」规矩:直接改、不留兼容分支、不做双读。

## 验收

- [x] 五种码 body 键全单字母,与注册表逐字一致
- [x] `display_name` / `recipient_name` / `ss58_address` 在 QR 协议内**零残留**
      (`QR_TECHNICAL.md` 仅剩一处「旧键必须拒绝」的刻意说明)
- [x] `walletCode` / `WalletCodeBody` / `wallet_code` / 中文「钱包码」**三端零残留**
- [x] `n` 字段全部经 `isAccountIdText` 校验(热端 `account_derivation.dart`、
      冷端新建 `util/account_id_text.dart`,均为单源)
- [x] 冷热两端 body 逐字一致(归一化 import 前缀后 md5 相同)
- [x] 测试:**热端 1146、冷端 288、onchina 9/9 全绿**;两端 analyze 零问题;
      `cargo check -p onchina` 通过
- [x] Golden fixtures 更新(`wallet_code.json` → `account_id_code.json`,
      三份 body 全改单字母);协议 spec 第 6/7/8 节重写
- [x] 文档:`qr-protocol-spec.md`、`QR_TECHNICAL.md`、`unified-protocols.md`、
      `unified-naming.md`、`citizenapp-vs-citizenwallet.md`、`NODE_TECHNICAL.md`、
      `qr-action-registry.md`

## 实施记录(与原计划的偏差)

1. **第三端**:除冷热两个 Flutter App,`citizenchain/onchina`(Rust)也解析 k=5
   (管理员扫码登录第 1 步)。已同步改造:`WalletCodeEnvelope/Body` →
   `AccountIdCodeEnvelope/Body`、`parse_wallet_code_account_id` →
   `parse_account_id_code`、serde 键 `account_id` → `#[serde(rename = "n")]`、
   9 个单测夹具全改。**只改 Dart 两端会造成跨语言协议漂移**。
2. **冷端补单源**:冷端没有 `isAccountIdText`,`^0x[0-9a-f]{64}$` 正则散落 3+ 处。
   新建 `citizenwallet/lib/util/account_id_text.dart` 作冷端单源并在 QR body 使用;
   按「同形异语义禁并入」,未动 `payload_decoder`(其 `_isCanonicalHex32` 语义待确认)
   与 `secret_cipher`(存储键名模式,语义不同)。
3. **`qr_action_registry.g.dart` 里的 `'account_id'`/`'cid_number'` 不是 body 键**,
   而是 k=1 审阅载荷的**中文字段标签**(另一命名空间),按设计保持不变。
4. **测试失败误报排查**:全量回归时 41 个钱包测试报
   `Failed to lookup symbol 'smoldot_client_init'` —— 真因是先前跑
   `citizenapp-run.sh` 时脚本内的 `cargo clean` 清掉了宿主 macOS dylib,
   与本次改动无关;`build-smoldot-native.sh macos` 重建后全绿。
