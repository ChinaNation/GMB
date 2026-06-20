# ADR-022:GMB 抗量子签名升级 + 统一加密方案（真源）

状态:Accepted(2026-06-18)。**本 ADR 是 GMB 抗量子方案的唯一真源**,取代并删除一切旧 PQC 迁移方案文档。
关联:[[feedback_no_compatibility]]、[[feedback_chainspec_frozen]]、[[feedback_pubkey_format_rule]]、[[project_pqc_unified_adr022]]

---

## 0. 方案目标

GMB **当前继续用 sr25519 签名**(链上已有真实用户);**未来通过链 runtime 升级 + wuminapp/wumin 客户端升级,在位切换到 PQC 签名**。用户**不换助记词、不换钱包、不换账户、不换地址、不换余额归属**,日常仍是"打开钱包、确认交易、冷钱包扫码签名"。

**核心原则一句话:AccountId 是账户身份锚点,签名算法只是该账户的授权方式。**

> 这是**活链在位升级**,不是重新创世造新链,不是无数据冷启。链已有真实用户用 sr25519,所以需要一座"旧地址账户无痛进入 PQC"的授权桥(见 §3 bootstrap),不是保留旧流程双轨。

## 1. 外部标准依据(锁版本 + 测试向量)

- **签名 = ML-DSA-65**,NIST **FIPS 204**。最终标准已发布但有 errata/修订提示 → **必须锁库版本 + 钉测试向量**。
- **KEM = ML-KEM-768**,NIST **FIPS 203**。
- **IM/TLS 混合 KEM 参考 X-Wing**,但 X-Wing 仍是 **draft 且不是认证 KEM**——**不能当身份认证**;身份仍绑账户签名 / MLS credential。
- TLS:**rustls 已支持 X25519MLKEM768**,但需处理互通失败 + provider 选择。

## 2. 账户与地址模型(铁律)

当前地址规则**保持不变**:
```
助记词 → 当前 wuminapp/wumin 已用的 miniSecretFromEntropy → AccountSeedV1 → sr25519 public key → AccountId/钱包地址
```

**铁律:**
- `AccountSeedV1` 必须等于现有 32B mini secret,**不能为 PQC 改变**。
- 当前 sr25519 地址派生路径**不改**,地址逐字节不变。
- `AccountId` 永远等于当前 sr25519 锚点账户。
- 未来 PQC 私钥从同一 `AccountSeedV1` 派生,但**不参与生成地址**。
- **不允许**用 ML-DSA 公钥 / ML-DSA 公钥哈希 / KEM 公钥重新生成新账户地址。

**派生规则(关键:sr25519 分支不套 HKDF):**
```
AccountSeedV1 = 当前 miniSecretFromEntropy 输出, 32B, 绝密

sr25519 地址锚点:
  current_sr25519_from_seed(AccountSeedV1) -> AccountId    ← 沿用现有直接派生, 不经 HKDF

PQC 签名密钥:
  HKDF-SHA512(AccountSeedV1, info="GMB/account/ml-dsa-65/v1")
    -> ML-DSA-65 确定性 keygen seed -> ML-DSA-65 keypair

PQC/KEM 加密密钥:
  HKDF-SHA512(AccountSeedV1, info="GMB/account/ml-kem-768/v1")
    -> ML-KEM-768 keypair
```
> ⚠️ **sr25519 分支绝不写成 `HKDF("GMB/sr25519/v1")`**,除非 golden vector 证明地址逐字节一致——否则 `HKDF(seed)≠seed`,`fromSeed` 输入变→地址变→破"不换地址"。当前真实路径(已核验 `wallet_manager.dart:541`)是 `miniSecret → sr25519.fromSeed` 直接,无 HKDF。

**KDF 精确定义(钉死,直接决定冷热 + 链 + 后端 golden vector):**
- 算法:**HKDF-SHA512**(RFC 5869),Extract-then-Expand。
- `PRK = HKDF-Extract(salt, IKM = AccountSeedV1[32B])`,**salt = 空(零长度)**(RFC5869 默认按 HashLen 个 0 字节处理);**域分离全靠 `info`,不靠 salt**。
- `info` = 标签的 **ASCII 字节,无 null 结尾**,例:`b"GMB/account/ml-dsa-65/v1"`、`b"GMB/account/ml-kem-768/v1"`。
- 输出长度 + 喂入:
  - ML-DSA-65:`OKM = HKDF-Expand(PRK, info_mldsa, L=32)` → 32B 种子 ξ → FIPS 204 `KeyGen_internal(ξ)`。🔴 **强制选用暴露 FIPS internal seed API(ξ / (d,z))的库**(如 fips204/fips203 的 `_internal`),由构造保证确定性;**禁止"喂模糊 RNG"的 fallback**——若实在要 DRBG,必须钉死具体算法(指定 `SHAKE256(seed=OKM)` XOF 按字节序取流)+ 锁库版本,任何库升级须重跑 golden vector 校验。
  - ML-KEM-768:`OKM = HKDF-Expand(PRK, info_mlkem, L=64)` → 64B `(d‖z)` → FIPS 203 `KeyGen_internal(d,z)`。
- **golden vector**:固定一条助记词 → AccountSeedV1 → {sr25519 地址, ML-DSA-65 公钥, ML-KEM-768 公钥},作为冷/热/链/后端跨端逐字节一致性基准。

## 3. 链上签名策略 + 无感 bootstrap 绑定

**当前阶段:** 普通 extrinsic 继续走 MultiSignature/sr25519;AccountId/SS58/余额/权限/治理身份全部现状不变。代码/文档去除"sr25519 = 账户身份"表述,改为"sr25519 是**当前**签名算法"。

**未来 runtime 升级后(最终定稿 = 路线 A,无二选一):**
- **不扩展 `MultiSignature`**(避免改 extrinsic 线格式和账户派生模型)。`MultiSignature` 是上游类型(`lib.rs:130/134`),加 ML-DSA 变体=改线格式≠setCode。
- **链上授权 = General Transaction + 自定义 `GmbPqcAuth` TransactionExtension**(最终路线;**废弃"pqc_dispatch pallet call 作为主路径"的说法**)。
  - `GmbPqcAuth` 放在**交易扩展流水线最前面**:验 PQC(ML-DSA-65)通过后,把这笔 General Transaction 的 origin **转成 `Signed(account)`**,后续 `CheckNonce`、`ChargeTransactionPayment` 等**走系统标准逻辑**(不自管 nonce/扣费)。
  - 🔴 **tuple 12 上限约束(落地硬性)**:当前 `TxExtension` 已 **12 项**(`lib.rs:164-176`,Polkadot SDK tuple 实现上限也是 12)→ **不能加第 13 项**。用**嵌套 tuple `(GmbPqcAuth, AuthorizeCall)` 合成第一项**(占原 `AuthorizeCall` 槽位),outer 仍 12 项。
  - 🔴 **`GmbPqcAuth` 同时负责"已绑定账户拒 sr25519"**(读 `AccountPqcKey`+`PqcPolicy`):**不再单独加拒绝扩展**——既避免 tuple 超限,又因同一扩展天然不会误伤它自己授权过的 PQC 交易(无需额外 marker 防误伤)。
  - **PQC proof 放在扩展 `extra`**(不在 call 里):ML-DSA 签名 / 公钥(bootstrap)/ `auth_mode` / `key_version`;**签名 preimage 排除签名字节本身**;扩展校验 `call_hash == blake2_256(body.call)`,bootstrap 再校验 `pqc_pubkey_hash == blake2_256(body.pqc_pubkey)`。
  - 🔴 **`GmbPqcAuth.extra` 必须有 `None`/`Disabled` 变体**:Phase A/B 的普通 sr25519 signed extrinsic **也会过这个扩展**(它在 tuple 里,每笔交易都过),`extra` 要能表达"本交易不是 PQC 授权"。形如 `extra = { None | Pqc{sig,auth_mode,key_version} | Bootstrap{pqc_pubkey,sig,bootstrap_sig,...} }`。
  - 🔴 **无 PQC proof 时透明放行,不得拒所有 General Transaction**:`extra=None` 且 call 是 runtime 支持的 authorized call 时,`GmbPqcAuth` 必须**原样把 origin 放行给后面的 `AuthorizeCall` 继续处理**(否则误伤现有 `#[pallet::authorize]` 机制);仅对 `extra=None` 的 **sr25519 signed origin** 才按 `PqcPolicy.reject_sr25519_when_bound` 做"已绑定拒 sr25519"判断。
- **`account-keys` pallet 只负责** `AccountPqcKey` / `PqcPolicy` 的**存储、查询、事件、密钥轮换**(及治理改 `PqcPolicy`);**不承载主交易派发**——真正业务 call 仍是原业务 call,由 `GmbPqcAuth` 授权后正常 dispatch。

**无感绑定(bootstrap):** 链无法从 sr25519 地址推出 ML-DSA 公钥,需一次"链上知道该账户 PQC 公钥"。设计为**首次 PQC 交易自动绑定并执行**,用户无感:
1. 用户升级 wuminapp/wumin。
2. 客户端从同一助记词派生当前地址 + ML-DSA-65 公钥。
3. 用户照常发起一笔交易。
4. 若链上还没有该账户 PQC 公钥 → 客户端构造**首笔 bootstrap General Transaction**(`GmbPqcAuth` 扩展授权)。
5. 冷/热钱包**一次确认**,内部同时生成:**sr25519 bootstrap 证明**(证"我是这个旧地址的主人")+ **ML-DSA 交易签名**(证"这个 PQC 公钥控制本次交易")。
6. `GmbPqcAuth` 验序 `pqc_pubkey_hash → sr25519 bootstrap sig → ML-DSA tx sig` 通过 → 交易以 `Signed(account)` 进入 nonce/扣费/业务 dispatch;**绑定写入 `AccountPqcKey` 放在 `post_dispatch`**(此时 nonce 与扣费已跑过)。
7. 后续交易只用 ML-DSA。

> 对用户是"第一次升级后照常交易",不是"去绑定新账户"。桥**只用于未绑定账户首次进入 PQC**,非长期双轨。
>
> 🔴 **bootstrap 失败语义(钉死)**:绑定写入在 `post_dispatch`(nonce/扣费已跑过),**即使内层业务 call 失败,绑定仍保留、内层 call 失败照常收费**(不让首笔业务失败逼用户重做 bootstrap)。钱包/用户提示按此口径统一。
>
> **当前"固定 sr25519"是 Phase A 实现真相,不是旧残留**:QR 硬拒非 sr25519(`wuminapp/lib/qr/bodies/sign_request_body.dart:37`、`wumin/lib/qr/bodies/sign_request_body.dart:40`)、CPMS/SFID `wallet_sig_alg=='sr25519'` 硬编码,**不要简单删除**;应改成"Phase A 当前只收 sr25519 → PQC 阶段按本 ADR 分流"。
>
> 🔴 **所有用户钱包签名域必须统一升级(不只普通链上交易)**:除普通 PQC 交易(`GmbPqcAuth` 授权)外,以下用户钱包 sr25519 授权面**全部**按 `sig_alg`/`auth_mode`/`AccountPqcKey` 统一升级——① L3/offchain payment 支付签名(`offchain settlement.rs` payer_sig);② SFID 绑定的钱包证明(如 super-admin 绑定);③ 治理 / 扫码签名;④ CPMS 钱包证明(ARCHIVE `wallet_sig`)。各域分别落 card2-5,本 ADR 在此统一声明,**杜绝"只改普通交易"漏面**。

## 4. 链上存储

```
AccountPqcKey[AccountId] = {
  alg: 0x02,                  // ML-DSA-65 (算法档; 0x03=ML-DSA-87)
  key_version: u32,           // 绑定代次, 换算法/轮换时 ++
  pubkey: BoundedVec<u8>,     // 完整 ML-DSA-65 公钥(~1952B)
  bound_at: BlockNumber,
  bootstrap_mode: AutoBound,
}
```
存完整公钥(非 hash):后续验签需完整公钥,每笔带公钥会让交易体积长期膨胀。

**`PqcPolicy`(链上全局策略真源,Phase B/C/D 单一来源):**
```
PqcPolicy = {
  phase: Phase,                            // A/B/C/D 当前阶段
  bootstrap_deadline: Option<BlockNumber>, // bootstrap 截止块高(D 关窗)
  reject_sr25519_when_bound: bool,         // 已绑定账户是否拒 sr25519
  allow_bootstrap_unbound: bool,           // 未绑定账户是否还能 bootstrap
}
```
- 由治理写入;**wuminapp/wumin 从它读取**做客户端分流(`Sr25519Only/PqcPrepared/PqcPrimary` 是它的客户端投影,**不是各端各写**)。
- 链端 §6 的拒绝 / 截止逻辑全部以 `PqcPolicy` 为准,杜绝多处硬编码阶段。
- 🔴 **首次 runtime 升级默认值必须安全**:`phase=B`、`reject_sr25519_when_bound=false`、`allow_bootstrap_unbound=true`、`bootstrap_deadline=None`;收紧(C/D)由后续治理推进。
- 🔴 **"不误拒"的准确口径**:把 `GmbPqcAuth` 加进 `TxExtension` 会改变 signed-extra / metadata 格式 → **未适配新 `TxExtension`/metadata 的旧客户端本就无法构造合法交易**(改扩展集的固有结果,非误拒)。安全默认保证的是:**不误拒"仍用 sr25519 算法、但已适配新 metadata/extra(`extra=None`)的客户端"**。这与"用户必须升级 wuminapp/wumin"的根需求一致。

**绑定规则:**
- 未绑定账户:允许首笔 bootstrap(`GmbPqcAuth` 验三签 → `post_dispatch` 写入 `AccountPqcKey`)。
- 已绑定账户:**拒绝再次用 sr25519 bootstrap 覆盖** PQC 公钥(first-bind-wins;bootstrap 只能由持 sr25519 私钥者发起,故不弱于现状)。
- 换算法版本:必须由**当前有效 PQC 私钥**授权 + `key_version++`。
- 绑定后,普通 sr25519 用户交易按链上策略逐步拒绝(见 §6 Phase D)。

## 5. 交易载荷(反重放域)

**`GmbPqcAuth` 扩展 `extra`(`GMB_PQC_TX_V1`,普通 PQC 交易):**
`genesis_hash`、`spec_version`、`transaction_version`、`ss58_format`、`account`、`nonce`、`era_or_deadline`、`tip`、`call_hash`、`sig_alg`、`key_version`、`auth_mode`。
> 🔴 **era / checkpoint 口径钉死**:**默认 immortal**(与当前 GMB sr25519 提交一致),`era_or_deadline = immortal`,链域绑定靠 `genesis_hash`,**不带 checkpoint block hash**。若改用 mortal era,payload **必须额外包含对应 mortal 锚点 checkpoint block hash**——二选一,不混用。

**首笔 bootstrap 额外(`GMB_PQC_BOOTSTRAP_V1`,扩展 `extra`):**
`genesis_hash`、**`spec_version`**(防跨升级重放)、`account`、`pqc_pubkey_hash`、`sig_alg`、`key_version`、`nonce`、`call_hash`。
> 🔴 **首笔 bootstrap 的 extrinsic body 必须携带完整 ML-DSA 公钥(~1952B)**(链上还没有,要写入 `AccountPqcKey`)+ sr25519 bootstrap 签名(64B)+ ML-DSA 交易签名(~3309B);后续 `pqc_dispatch` 不带公钥(链端按 account 读)。这决定 §8 的 QR 最坏体积。

**验签规则(顺序钉死):**
- 🔴 **所有 hash 口径写死 `blake2_256`**(`call_hash` / `pqc_pubkey_hash` / challenge 一律 `blake2_256`,对齐链端 `sp_io::hashing::blake2_256`)。
- 🔴 **bootstrap 首笔(顺序不可乱)**:① **先校验 `blake2_256(body.pqc_pubkey) == payload.pqc_pubkey_hash`**(确保 body 携带的公钥就是被签名承诺的那把)→ ② 验 sr25519 bootstrap challenge(证旧地址主人)→ ③ 验 ML-DSA 交易签名(用 body 里那把公钥)→ 全过才写 `AccountPqcKey` 并派发。
- 普通 PQC 交易:`ml_dsa_signature` 验交易 payload,公钥从 `AccountPqcKey` 按 account 读(交易不带公钥)。
- `account` 必须 = sr25519 公钥派生出的当前 AccountId。
- txpool 用 `(account, nonce)` 做 `provides` 防重复入池。
- `GmbPqcAuth` 的 `validate` 轻量无副作用;**绑定写入 `AccountPqcKey` 在 `post_dispatch`**(nonce/扣费后);业务 call 正常 dispatch。

## 6. 签名策略阶段(不要求用户手动参与)

| 阶段 | 链策略 | 用户体验 |
|---|---|---|
| **A 当前** | 只用 sr25519 | 完全不变 |
| **B 预埋** | runtime 支持 PQC 自动绑定,sr25519 仍可用 | 升级客户端后下一笔交易可自动绑定 |
| **C PQC 主用** | 已绑定账户只收 ML-DSA;未绑定旧账户允许 bootstrap 后立即转 PQC | 照常交易,客户端自动处理 |
| **D 收紧** | 长期未绑定账户按治理策略处理;已绑定账户彻底拒绝 sr25519 用户交易 | 大多数用户无感 |

> 🔴 **安全硬约束**:bootstrap 靠 sr25519 证明所有权,**强度等于 sr25519**。sr25519 一旦被量子破,攻击者可伪造 bootstrap 把自己的 ML-DSA 绑到受害者账户并锁死原主。**因此 bootstrap 窗口(Phase D 对长期未绑定账户的治理截止)必须赶在 sr25519 被量子破之前关闭**——这是无感桥的代价,非可选项。
>
> 🔴 **Phase D 用户后果(收紧前必须定稿,否则"无痛升级"与"量子破前关窗"在产品层冲突)**:关闭 bootstrap 后,**未升级老用户无法再用 sr25519 证明账户归属**。必须明确:① 截止块高(= `PqcPolicy.bootstrap_deadline`);② 公告周期;③ 长期未绑定账户的冻结/恢复策略;④ 是否有线下恢复流程——若答案是"不能恢复",等同资产终态锁定,须提前充分告知。
>
> 🔴 **"已绑定账户拒 sr25519"的实现点(钉死)**:由 **`GmbPqcAuth` 同一扩展负责**(不单独加扩展):对 sr25519 signed origin 读 `AccountPqcKey[who]` + `PqcPolicy.reject_sr25519_when_bound`,已绑定即拒(`InvalidTransaction`)。**全局一处、不漏 call**,不逐 call 拒;所有阶段判断统一读 `PqcPolicy`(§4),不多处硬编码。

## 7. wuminapp(热钱包)

- 导入/创建仍显示**同一地址**;内部保存 `AccountSeedV1` 语义,不改助记词恢复结果。
- 增 Rust FFI:`ml_dsa65_public_from_seed`、`ml_dsa65_sign`(经 `gmb-pqc`)。
- 查 `PqcPolicy` 分流:`Sr25519Only`→构造普通 sr25519 extrinsic;`PqcPrepared/PqcPrimary`→构造 PQC General Transaction(`GmbPqcAuth` 扩展授权);**账户未绑定**→构造首笔 bootstrap(同一确认出 sr25519 bootstrap + ML-DSA 签名)。
- UI **不展示** "PQC 公钥""绑定状态机""换账户";交易记录仍按原地址归集。
- 恢复:同一助记词恢复同一地址 + 同一 PQC 密钥(确定性派生)。

## 8. wumin(冷钱包)

- 继续离线签名器;扫码按 `sig_alg`/`auth_mode` 识别:当前签 sr25519,未来签 ML-DSA,首次绑定时**一次扫码确认同时**出 sr25519 bootstrap 签名 + ML-DSA 交易签名。
- 展示仍是账户/收款方/金额/治理动作等用户语义,不展示多算法细节。
- **QR 升级:** `sig_alg: sr25519|ml-dsa-65`、`auth_mode: normal|pqc|bootstrap-pqc`、`key_version`、`payload_hash`、**`chunk_index/chunk_total` 大签名分片**。
- 🔴 **最坏体积按 bootstrap 设计(含公钥)**:首笔 bootstrap 二进制 ≈ sr25519 签名 64B + ML-DSA 签名 ~3309B + **ML-DSA 公钥 ~1952B** + call + 分片元数据,**hex 编码后再翻倍(≈10KB+)**。不能假设单张 QR 稳定可扫,分片必须按这个最坏值真机实测。

## 9. SFID / CPMS 边界

- SFID/CPMS **不托管**用户助记词、`AccountSeedV1`、sr25519 私钥、PQC 私钥。
- 只做两件:① 记录的钱包地址仍是原 `AccountId`;② 验钱包签名时按 `sig_alg` 分流(当前 sr25519,未来 ML-DSA)。
- `wallet_sig_alg` 字段**保留并放开枚举**,但**不允许生成新钱包地址**。
- 🔴 **ML-DSA 钱包签名的归属验证(P0)**:`wallet_sig_alg=ml-dsa-65` 时 `wallet_pubkey` 不能像 sr25519 那样反推 `wallet_address`。SFID/CPMS 验 ML-DSA 签名只能证"某 ML-DSA 公钥签了",**必须另有权威绑定来源证明"它属于这个原 sr25519 地址"**,三选一(实现期定一个):① 查链上 `AccountPqcKey[wallet_address]`;② 链上状态证明 / 可信缓存;③ 未完成链上 bootstrap 前**拒绝** ML-DSA 钱包签名。
- SFID/CPMS **自身系统签名**是否改 ML-DSA = 独立系统密钥升级,**不影响用户账户无痛升级主线**(单独推进:SFID MAIN signer / CPMS ARCHIVE 档案密钥)。

## 10. IM / 传输加密(与账户签名分开做,不阻塞)

- 账户签名 = ML-DSA-65;IM/MLS 机密性 = 未来 X-Wing/ML-KEM-768 混合 KEM;TLS = 评估 rustls X25519MLKEM768。
- **KEM 不能当身份认证**,身份仍绑账户签名或 MLS credential。
- **账户 ML-KEM 用途缩窄**:`AccountSeedV1` 派生的 ML-KEM-768 是**长期账户密钥**,最多用于身份 / 恢复 / 特定加密入口,**禁止直接用作 IM 消息会话密钥**;IM 必须走设备/会话密钥 + MLS rekey(前向保密),不复用账户 KEM。
- 此线不阻塞钱包签名升级。

## 11. 必须先做的技术 spike(写业务代码前)

1. ML-DSA-65 Rust crate 是否支持 no_std / WASM;runtime WASM 体积增量。
2. 单次 ML-DSA 验签 weight;一个区块最多容多少次 PQC 验签。
3. 🔴 **(硬闸门)验证路线 A `GmbPqcAuth` 落地机制**(路线已定 A,**不再二选一**):① `GmbPqcAuth` 在 `validate`/`prepare` 把 General Transaction origin 转成 `Signed(account)`,使其后 `CheckNonce`/`ChargeTransactionPayment` 正常生效;② **嵌套 tuple `(GmbPqcAuth, AuthorizeCall)` 作第一项能编译且按序执行**(outer 仍 12 项);③ 绑定写入放 `post_dispatch` 可行;④ txpool `provides=(account,nonce)`。**确认这套机制在当前 SDK 版本可行**(若 origin 转换受限,退而在 `GmbPqcAuth` 内自管 nonce/扣费,但路线仍是单一扩展授权,**不回退到 pallet call 主路径**)。
4. QR 分片真机稳定:按 bootstrap 最坏体积(sr25519 64B + ML-DSA 签名 ~3309B + ML-DSA 公钥 ~1952B + call,hex 翻倍 ≈10KB+)。
5. ML-DSA WASM 验签不可接受时再评估 host function;但 host function = 节点二进制边界变化,**不再是纯 setCode**(要 fork `ChinaNation/ss58-2027-fix`)。

## 12. 验收标准

- 老客户端 sr25519 钱包地址不变;新客户端同助记词恢复地址**逐字节一致**。
- runtime 升级后原账户余额/权限/治理身份不变。
- 未绑定账户首次交易 **bootstrap + execute** 成功;已绑定账户后续 ML-DSA 交易成功。
- 已绑定账户普通 sr25519 用户交易按策略被拒。
- 冷钱包离线扫码完成 bootstrap + PQC 签名(含分片)。
- SFID/CPMS 验证同一地址下不同 `sig_alg` 的签名,且 ML-DSA 签名经**权威绑定来源**(链上 `AccountPqcKey`/状态证明)证明归属于原地址,非仅"某公钥签了"。
- `GmbPqcAuth` 把 PQC General Transaction 转 `Signed(account)` 后 `CheckNonce`/`ChargeTransactionPayment` **真实跑通**;**嵌套 tuple `(GmbPqcAuth, AuthorizeCall)` 编译 + 按序执行**(outer 仍 12 项);bootstrap 绑定在 `post_dispatch`。
- `GmbPqcAuth.extra=None` 时:已适配新 metadata 的 sr25519 客户端正常发交易;**非 PQC 的 authorized general call 透明放行给 `AuthorizeCall`**(现有 `#[pallet::authorize]` 机制不被误伤)。
- bootstrap 绑定 + 内层 call 失败语义按定稿口径(绑定保留 / 内层失败收费)一致;Phase D 收紧前已定稿截止块高 / 公告周期 / 冻结恢复 / 线下流程。
- 全仓 PQC 表述以本 ADR 为唯一真源;残留旧路线(单独绑定步 / per-account 状态切换 / 共识造新链)一律按本 ADR 收敛清理;真实运行态验收(非仅编译/单测)。

## 13. 静态加密 + KDF 卫生(随线带做,核验发现)

- 🔴 **node 清算行落盘是假 AES-GCM**:`settlement/keystore.rs:181-220` 实为 XOR(blake2 keystream)+截断 blake2 tag,`derive_key` 仅 100 轮 blake2,函数名/注释说谎,node 无 aes-gcm 依赖 → 换真 `Aes256Gcm`+`Argon2id`+改谎注释(独立可立即修)。
- KDF 收敛:CPMS master/geo_seal(`initialize/mod.rs:50`/`dangan/mod.rs:258` Blake2b 单次)→ HKDF-SHA512;口令场景(App 锁 PBKDF2 wumin 1M/wuminapp 100K 漂移)→ Argon2id。
- 对称统一 AES-256-GCM(IM 从 AES-128 升级)。
- `gmb-pqc` 共享 crate(runtime no_std WASM + sfid/cpms/node + 钱包 FFI 共用):HKDF 派生规则 + algo 常量 + domain 常量(**强制 `[u8;N]`**,修 `batch_item.rs:39/42` 的 `&[u8]`)+ `verify_by_algo` trait。

## 14. 任务卡

见 `memory/08-tasks/open/20260618-pqc-card*`(按"地址稳定签名升级"重拆):card0 卫生/card1 gmb-pqc+spike(验 GmbPqcAuth 机制)/card2 链端 **GmbPqcAuth 扩展授权 + account-keys(存储/策略/查询/轮换)+ bootstrap(post_dispatch)**/card3 钱包派生+bootstrap+QR分片/card4 SFID 分流/card5 CPMS 分流/card6 IM-TLS 机密性。
