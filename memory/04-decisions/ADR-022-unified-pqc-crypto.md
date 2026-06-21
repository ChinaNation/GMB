# ADR-022:GMB 抗量子签名升级 + 统一加密方案（真源 v4）

状态:Accepted(2026-06-18)。**本 ADR 是 GMB 抗量子方案的唯一真源**,取代并删除一切旧 PQC 迁移方案文档。
关联:[[feedback_no_compatibility]]、[[feedback_chainspec_frozen]]、[[feedback_pubkey_format_rule]]、[[project_pqc_unified_adr022]]
v4 变更:并入 8 项已拍板决策(见 §15)+ 7 视角对抗审查的 13 BLOCKER + 关键 HIGH 修正。

---

## 0. 方案目标 + 钉死点

GMB **当前继续用 sr25519 签名**(链上已有真实用户);**未来通过链 runtime 升级 + wuminapp/wumin 客户端升级,在位切换到 PQC 签名**。用户不换助记词/账户/地址/余额。**核心原则:AccountId 是账户身份锚点,签名算法只是该账户的授权方式。**

- **一条链活链在位升级**(非新链/非重新创世)。
- **sr25519 分两段**:现在 sr25519 签名 → PQC 上链后 ML-DSA-65 签名;AccountId(=sr25519 公钥)全程不变。
- **账户密钥只派生 sr25519 + ML-DSA-65**(决策3:账户**不派生** ML-KEM;机密性 KEM 在 IM/TLS 层独立,见 §10)。

## 1. 外部标准依据(锁版本 + 测试向量)

- 签名 = **ML-DSA-65**(NIST FIPS 204,有 errata → **锁库版本 + 钉测试向量/golden vector**)。
- 机密性 KEM = **ML-KEM-768**(NIST FIPS 203),**仅用于 IM/TLS 层,不进账户密钥体系**。
- IM/TLS 混合 KEM 参考 X-Wing(draft 且非认证 KEM → **不能当身份认证**)。
- TLS:rustls X25519MLKEM768(混合密钥交换,非身份来源)。

## 2. 账户与地址模型 + KDF(铁律)

```
助记词 → miniSecretFromEntropy → AccountSeedV1(32B 绝密) → sr25519 public key → AccountId/地址
```
- `AccountSeedV1` = 现有 32B mini secret,不为 PQC 改变;sr25519 派生路径**不改**,地址逐字节不变;`AccountId` 永远=sr25519 锚点;不允许用 ML-DSA 公钥/哈希生成新地址。

**派生规则(只两支;sr25519 不套 HKDF):**
```
sr25519 地址锚点: sr25519.fromSeed(AccountSeedV1) -> AccountId        ← 现有直接派生, 绝不经 HKDF
ML-DSA-65 签名:   HKDF-SHA512(AccountSeedV1, info="GMB/account/ml-dsa-65/seed32/v1") -> ξ(32B) -> KeyGen_internal(ξ)
```
> ⚠️ sr25519 分支**绝不套 HKDF**(`HKDF(seed)≠seed`→地址变);账户**不再派生 ML-KEM**(决策3)。

**KDF 精确定义(钉死 golden vector):**
- HKDF-SHA512(RFC5869)Extract-then-Expand;`PRK=HKDF-Extract(salt=空, IKM=AccountSeedV1[32B])`,域分离全靠 `info`(ASCII 无 null,**含长度域** `seed32`)。
- ML-DSA-65:`HKDF-Expand(PRK, info, L=32)` → 32B ξ,**直接当 ξ** 喂 FIPS204 `KeyGen_internal(ξ)`。
- 🔴 **(B8)强制选用暴露 `KeyGen_internal(ξ)` seed-API 的库**(fips204 锁定版本);**删除一切 DRBG fallback**——库不暴露 ξ-API 则换库,不接受 DRBG 替代(否则同助记词跨端派生不同公钥)。库名+版本+API 名钉进 card1 spike。
- **golden vector** 必须含中间量 ξ 与最终 {sr25519 SS58, ML-DSA-65 公钥},供冷/热/链/后端逐字节对拍;库升级须重跑。

## 3. 链上签名策略(路线 A = GmbPqcAuth,定稿)

**当前**:普通 extrinsic 继续 MultiSignature/sr25519,全部现状不变。

**未来 runtime 升级后:**
- **不扩展 `MultiSignature`**;**链上授权 = General Transaction + 自定义 `GmbPqcAuth` TransactionExtension**(废弃 pqc_dispatch pallet call 主路径)。
- `GmbPqcAuth` 放扩展流水线**最前**:`validate` 验 ML-DSA-65 后**返回 `Signed(account)` origin**(`prepare` 不改 origin);后续 `CheckNonce`/`ChargeTransactionPayment` 走系统标准逻辑。
- 🔴 **tuple 12 上限**:`TxExtension` 已 12 项(`lib.rs:164-176`)→ 嵌套 `(GmbPqcAuth, AuthorizeCall)` 占第一项槽位,不加第 13 项。
- 🔴 **(B12)本次升级必须 `spec_version++` 且 `transaction_version`(0→1)++**——改 TxExtension 结构属交易口径变更,让 `CheckTxVersion` 明确拒旧口径交易(而非模糊 BadProof)。
- **`extra` 必有变体 `{ None | Pqc{sig,key_version} | Bootstrap{pqc_pubkey,ml_dsa_sig,sr25519_bootstrap_sig,key_version} }`**:Phase A/B 普通 sr25519 交易也过此扩展,`None` 表达"非 PQC 授权"。
- 🔴 **`extra=None` 透明放行**:`extra=None` 时 `GmbPqcAuth` **返回未改动的原 origin**(零副作用)给 `AuthorizeCall` 接手——authorized general call 不被误伤;仅对 `extra=None` 的 **sr25519 signed origin** 才按 `PqcPolicy` 判"已绑定拒 sr25519"。
- 🔴 **授权模式与 origin 互斥**:`extra=Pqc|Bootstrap` 只允许 General Transaction(origin=None);sr25519 signed extrinsic 携带 PQC proof 直接 `BadProof`。
- 🔴 **(H1)单测**:① `None`+authorized call → 原 origin 透传 → AuthorizeCall 产 Authorized;② `Pqc` → Signed → AuthorizeCall 不二次授权。
- 🔴 **(M2)`weight()` 纯 `self.extra` 路由 card1 benchmark 常量,严禁读 storage**;Bootstrap 取最坏(1952B 写入+双验签)。
- 🔴 **(M10)fail-open**:`PqcPolicy` storage 缺失/解码失败时等价安全默认(phase=B、reject=false),**绝不 fail-closed 冻结全链**。
- 🔴 **(M3 修正)metadata 绑定不夸大**:`CheckMetadataHash` 全链保持 **Disabled**(决策7),其 implicit=None;runtime/版本隔离实际靠 `spec_version`+`transaction_version`+`genesis_hash`+`call_hash`,非 metadata hash。
- **`account-keys` pallet(idx=27)只承载** `AccountPqcKey`/`PqcPolicy` 的存储/查询/事件/密钥轮换;**不承载主交易派发**。

**无感首笔绑定(bootstrap):**
1-3. 用户升级 → 同助记词派生当前地址 + ML-DSA-65 公钥 → 照常发起一笔交易。
4. 链上无该账户 PQC 公钥 → 客户端构造**首笔 bootstrap General Transaction**(`GmbPqcAuth` 扩展授权)。
5. 冷/热钱包一次确认,同时出 **sr25519 bootstrap 证明** + **ML-DSA 交易签名**。
6. `GmbPqcAuth.validate` 按 §5 验序通过 → 返回 `Signed(account)` → nonce/扣费/业务 dispatch;**绑定写 `AccountPqcKey` 在 `post_dispatch`**(nonce/扣费已跑)。
7. 后续只用 ML-DSA。
> 🔴 **(H1/post_dispatch)失败语义**:绑定写在 post_dispatch,内层 call 失败绑定仍保留、照常收费;**post_dispatch 绝不返回 Err**(返回 Err 会作废整个区块=远程 DoS)——冲突判定(已绑定不同值)前移到 `validate` 拒,post_dispatch 任何情况返回 `Ok`(未绑定→写、已绑定→no-op)。
> 🔴 **(H2)bootstrap 账户须 providers/sufficients>0**(已有余额),否则标准 `CheckNonce` 先以 `Payment` 拒;provider=0 账户的 bootstrap 行为须在 GmbPqcAuth 给明确错误语义 + 单测。

> **当前固定 sr25519 是 Phase A 真相不是旧残留**(QR/CPMS/SFID `wallet_sig_alg=='sr25519'` 硬编码)→ 改"Phase A 只收 sr25519 → 分流",不删。

## 4. 链上存储

```
AccountPqcKey[AccountId] = { alg:0x02(ML-DSA-65), key_version:u32, pubkey:BoundedVec<u8,ConstU32<2048>>, bound_at:BlockNumber }
PqcPolicy = { phase, bootstrap_deadline:Option<BlockNumber>, reject_sr25519_when_bound:bool, allow_bootstrap_unbound:bool }
```
- 删除 `bootstrap_mode` 字段(M15:无第二变体、无消费方、疑似违反 per-account-state 禁令)。
- `account-keys` **pallet_index=27**(契约真源在此登记;当前 runtime 最高 idx=26,27 空闲)。
- 🔴 **首次升级 PqcPolicy 安全默认**:`phase=B`/`reject_sr25519_when_bound=false`/`allow_bootstrap_unbound=true`/`bootstrap_deadline=None`。
- **绑定规则**:未绑定→允许首笔 bootstrap;已绑定→拒再次 sr25519 覆盖(first-bind-wins,冲突在 validate 拒)。
- 🔴 **(H3)密钥轮换双签**:轮换 = ① 当前 PQC 私钥授权 + ② **新 PQC 私钥对 `(新公钥+新 key_version+account+genesis_hash)` 自签(PoP)**,两签皆过才 `key_version++` 写新 `AccountPqcKey`。
- 🔴 **(决策1/2)无恢复通道**:绑定后授权**只认 ML-DSA**,无 sr25519 锚点回退;**ML-DSA 私钥泄露/丢失无代绑恢复**(用户须妥善备份助记词——助记词在即可确定性重派生同一 ML-DSA 私钥;但若 ML-DSA 私钥被泄露并被攻击者抢先轮换,无回退,账户即失陷)。Phase D 关窗后未绑定老用户=资产终态锁定(见 §6)。

## 5. 交易载荷 + 验签(反重放域)

**`GmbPqcAuth.extra` 签名 payload(`GMB_PQC_TX_V1`,普通 PQC 交易):**
域标签 `DOMAIN_TX` ++ SCALE(`genesis_hash`、`spec_version`、`transaction_version`、`account`、`nonce`、`era_or_deadline`、`tip`、`call_hash`、`key_version`、`following_extensions_hash`)。
- `ss58_format` 为**纯展示字段**(L2:链上无对应扩展 implicit,不参与一致性比对,链域已由 genesis_hash 隐含)。
- `sig_alg`/`alg` 算法标识**进 DOMAIN 字面量**(如 `DOMAIN_TX = b"GMB_PQC_TX_MLDSA65_V1"`),不另设字段(H7/rank8 域+算法隔离)。

**首笔 bootstrap(`GMB_PQC_BOOTSTRAP_V1`):** 字段集**与 GMB_PQC_TX_V1 对齐**(补 transaction_version/tip/era_or_deadline,H4),额外含 `pqc_pubkey_hash`;域标签 `DOMAIN_BOOTSTRAP = b"GMB_PQC_BOOTSTRAP_MLDSA65_V1"`。extrinsic body 携带完整 ML-DSA 公钥(~1952B)+ sr25519 签名(64B)+ ML-DSA 签名(~3309B);后续普通 PQC 交易不带公钥。

🔴 **(B3)`following_extensions_hash` 口径 = SDK `inherited_implication` 精确递归编码,不是扁平拼接**:
`blake2_256( ImplicationParts{ base: TxBaseImplication((extension_version, call)), explicit:(following_explicit_tail, parent_explicit), implicit:(following_implicit_tail, parent_implicit) }.encode() )`,严格复刻 `runtime traits/transaction_extension/mod.rs:577-598`。嵌套 tuple `(GmbPqcAuth, AuthorizeCall)` 下 GmbPqcAuth 看到的 explicit/implicit tail 是"内层 tail 再与 outer tail 组对"的递归结构。card1 出该字节 golden vector,card2 单测断言链端 `inherited_implication.encode()` 与协议口径逐字节相等(参照 mod.rs:712-869)。
> 参与 following implicits 的扩展逐一列明:`CheckSpecVersion()`/`CheckTxVersion()`/`CheckNonStakeSender()`/`CheckGenesis(genesis_hash)`/`CheckMortality(immortal→genesis_hash)`/`CheckNonce()`/`CheckWeight()`/`ChargeTransactionPayment()`/`CheckMetadataHash(None,Disabled)`/`WeightReclaim()`。**(M16)immortal 下 CheckMortality.implicit 仍是 genesis hash,不可漏。**

🔴 **(B1)bootstrap challenge 字面构造(钉死)**:
`sr25519_bootstrap_signature = sr25519_sign(blake2_256(DOMAIN_BOOTSTRAP ++ SCALE(genesis_hash, spec_version, transaction_version, account, pqc_pubkey_hash, key_version, nonce, call_hash, following_extensions_hash)))`。
硬约束:① sr25519 签名**必须覆盖 `pqc_pubkey_hash`**("我授权这把 PQC 公钥");② 专用 DOMAIN_BOOTSTRAP 前缀与 SFID/治理/L3 的 sr25519 签名域不可互换;③ ML-DSA 交易签名**反向覆盖** `blake2_256(sr25519_bootstrap_signature)`(双向交叉绑定,杜绝两签拼接)。card2 单测:挪用其它域 sr25519 签名构造 bootstrap 必拒。

**验签顺序(钉死,hash 全 `blake2_256`):**
- **bootstrap**:① `blake2_256(body.pqc_pubkey)==payload.pqc_pubkey_hash` → ② sr25519 bootstrap challenge(覆盖 pqc_pubkey_hash)→ ③ ML-DSA 交易签名(用 body 公钥,且覆盖 sr25519 签名 hash)→ 通过才 post_dispatch 写。
- **普通 PQC**:ML-DSA 签名验交易 payload,公钥从 `AccountPqcKey` 按 account 读;`alg` 必须等于 `AccountPqcKey.alg`(防降级)。
- `account` = sr25519 公钥派生的当前 AccountId;`call_hash` 与 following_extensions_hash 内 call 必须**同一份字节序列**(M6:最简——following 直接复用 call_hash)。
- **(L9)txpool `provides=(account,nonce)` 由标准 CheckNonce 在 Signed origin 下自动产生,GmbPqcAuth 不重复设**;GmbPqcAuth.validate 对 nonce 只做廉价窗口预检。
- 🔴 **(H12/B11)body 长度上限硬校验**(按最坏 ~10KB)+ 未绑定账户 bootstrap 按 (account,source) 限速。

🔴 **(B4)era 钉死 immortal(决策4,PoW 难度无问题)**:`era_or_deadline=immortal`,链域靠 genesis_hash,不带 checkpoint;CheckMortality.implicit 仍是 genesis hash(已纳入 following_extensions_hash)。

## 6. 签名策略阶段

| 阶段 | 链策略 | 用户体验 |
|---|---|---|
| A 当前 | 只用 sr25519 | 完全不变 |
| B 预埋 | 支持 PQC 自动绑定,sr25519 仍可用 | 升级后下一笔自动绑定 |
| C PQC 主用 | 已绑定只收 ML-DSA;未绑定 bootstrap 后转 | 照常交易 |
| D 收紧 | 关 bootstrap;已绑定彻底拒 sr25519 | 大多数无感 |

- 🔴 **(H17)per-account 收紧解耦**:`reject_sr25519_when_bound` 对**已写 AccountPqcKey 的账户**可在 Phase B 即默认 true(个体一旦绑定即只收 ML-DSA),对未绑定旧账户保持 sr25519 可用直到 Phase D——杜绝 Phase B 的"已绑定账户双授权窗口"。
- "已绑定拒 sr25519"由 **GmbPqcAuth 同一扩展**负责(读 AccountPqcKey+PqcPolicy),全局一处不漏 call。
- 🔴 **安全硬约束 + 决策1(无恢复)**:bootstrap 强度=sr25519,窗口(`bootstrap_deadline`)必须赶在 sr25519 被量子破前关闭;**关窗后未绑定老用户无恢复通道 = 资产终态锁定**。因此 `bootstrap_deadline` 的治理设定 + **多轮充分公告**是硬前置(见 card7),且必须在宪法/白皮书层向用户提前告知"逾期不升级即永久锁定"。

## 7. wuminapp(热钱包)

同一地址;Rust FFI(gmb-pqc)`ml_dsa65_public_from_seed`/`ml_dsa65_sign`;查 `PqcPolicy` 分流(Sr25519Only→sr25519;PqcPrepared/PqcPrimary→PQC General Transaction;未绑定→首笔 bootstrap);UI 不展示 PQC 公钥/绑定过程/换账户;同助记词恢复同地址+同 ML-DSA 密钥。

## 8. wumin(冷钱包)+ QR

- 🔴 **(B9)冷钱包新建 Rust FFI 子工程**(wumin 现纯 Dart 无 rust/):对标 wuminapp/rust 的 cdylib/staticlib + Android/iOS target + cbindgen,把 gmb-pqc 编进冷热两端。
- 🔴 **(B10)离线 metadata 策略**:"按 metadata 重建 following_extensions_hash"整体留在在线热钱包;QR 携带重建所需最小要素(extension_version + 各后续扩展显式值 + 预算 hash),**冷钱包用 gmb-pqc 本地 SCALE 重算并比对**,且**自己从助记词派生 ML-DSA 公钥核对 `pqc_pubkey_hash`**(不盲信 QR 公钥),核对通过才出双签;**严禁退化成 wasm 式纯哈希盲签**(保持两色识别·decodeFailed 即拒)。
- 🔴 **(B11)QR 分片**:envelope 加 `chunk_index/chunk_total/total_hash`;渲染端多帧轮播(单帧字节预算按 ECC=M version≤40 反推);扫描端分片聚合状态机(按 id 归并/去重补帧/校验 total_hash);放开 32768 上限;最坏 bootstrap(~10KB+)真机实测。
- **(H18)QR body 四处(冷热 request/response)放开** `sig_alg(sr25519|ml-dsa-65)`+`auth_mode`+`key_version`+`chunk_*`,Phase A 仍只收 sr25519;进签名的 hash 一律 gmb-pqc blake2_256,**禁复用 qr_signer 的 sha256**。

## 9. SFID / CPMS 边界

- 不托管助记词/seed/私钥;`wallet_sig_alg` 放开枚举但不生成新地址。
- 🔴 **(决策8/B5/B7/H7)ML-DSA 钱包签名归属验证 = 统一查链 `AccountPqcKey[wallet_address]`**:
  - **唯一权威源 = 查链上 `AccountPqcKey`**(SFID 经 subxt 0.43.1 查,已具备能力);确认该 ML-DSA 公钥已绑到此 sr25519 锚点才接受;**未完成 bootstrap → 拒 ML-DSA 钱包签名**(退化策略)。SFID/CPMS **必须用同一选项**。
  - **(B5)SFID `citizens/binding.rs`(:79/340/878)是唯一真实落点**:ML-DSA 时 QR 单独带公钥(不再 ss58 反推)、查链验归属、verify_by_algo 验签;脱离 `[u8;32]`。card4 增列。
  - **(B7)CPMS 无链客户端**:CPMS **不验钱包签名**(本就不验,P-CRED-002),归属判定下沉 SFID;card5 删①②选项。
- 🔴 **(B6)SFID MAIN signer 迁 ML-DSA 必须与 card2 链端验签器原子同批上线**(或链端先双 algo 路由再切),否则机构注册/投票/人口快照全红。
- **(H9)系统签名原文 `build_signature_message` 必须含 sig_alg 进 preimage**(防算法混淆/降级)。
- 🔴 **(决策5)CPMS install_sig 补真实验证**:CPMS 启动用 SFID MAIN 公钥真正验 install_sig(顺带补"任何人投递 install QR 即可初始化 CPMS"的安全洞);card4/card5。

## 10. IM / 传输加密(独立线,不阻塞;账户不派生 KEM)

- IM/MLS 机密性 = X-Wing/ML-KEM-768 混合(换 libcrux provider);TLS = X25519MLKEM768;对称 AES-256/ChaCha20-256。**KEM 不当身份认证**。
- 🔴 **(决策3)账户不派生 ML-KEM**:IM/TLS 的 ML-KEM 密钥是 **MLS 设备/会话密钥 + 传输层密钥,与 `AccountSeedV1` 无关**,走前向保密 rekey;绝不复用账户密钥做 IM 会话。
- 🔴 **(决策6)card0 不动 IM**:IM ciphersuite(AES-128→256 + X-Wing)**一次性由 card6 完成**,card0 不碰 IM,避免 MLS 套件二次变更破坏已有会话。

## 11. 必须先做的技术 spike(card1 闸门,绿了才进 card2/card3)

1. **fips204/ML-DSA-65 no_std/WASM + iOS/Android 移动端**编译 + 体积 + 验签 weight;**锁定暴露 `KeyGen_internal(ξ)` 的库版本**(B8)。
2. 单次 ML-DSA 验签 weight;区块容量。
3. 🔴 **(B2/B3)客户端 + 链端 General Transaction 机制**:① polkadart 0.7.1 **不能编 General Transaction**(只 legacy 0x84)→ fork/patch 或自写 v5 SCALE 编码器 + extension_version + 嵌套 extra;② 链端 `GmbPqcAuth.validate` 转 `Signed(account)`、嵌套 tuple `(GmbPqcAuth,AuthorizeCall)` 编译+按序执行;③ **嵌套下 `inherited_implication` 真含 outer 全部后续扩展 implicit、Dart 重建值与链端逐字节一致**(最高风险点);④ post_dispatch 写绑定可行且幂等。
4. **(H14)枚举当前 runtime 所有 `#[pallet::authorize]` call**,确认不成为已绑定账户绕过 PQC 强制的旁路。
5. QR 分片真机稳定(最坏 bootstrap ~10KB+)。
6. seal=ML-DSA 评估(见 card7,独立)。

## 12. 验收标准(节选)

- 地址逐字节不变、golden vector(含 ξ)跨端一致;升级后余额/权限/治理身份不变。
- 嵌套 tuple `(GmbPqcAuth,AuthorizeCall)` 编译+按序;`following_extensions_hash` 链端/钱包逐字节一致;`CheckNonce`/`ChargeTransactionPayment` 在 Signed origin 下真实生效。
- 未绑定首次 bootstrap+execute 成功(post_dispatch 写、内层失败绑定仍留、**冲突不作废区块**);已绑定后续 PQC 成功;已绑定 sr25519 被拒、PQC 不误伤;`None`+authorized call 透传成功。
- 挪用其它域 sr25519 签名构造 bootstrap 被拒;sr25519 signed 夹带 PQC proof 被拒;算法降级(改 alg)被拒。
- L3/批签:已绑定账户拒 sr25519 payer、ML-DSA 走 AccountPqcKey,未绑定 sr25519 仍可用;6 处签名长度上限放宽容纳 ML-DSA(~3309B)。
- SFID 查链 AccountPqcKey 验 ML-DSA 钱包签名归属;CPMS install_sig 被真实验证;transaction_version 已 bump 旧口径交易被明确拒。
- 全签名面(见 §14)无遗漏;真实运行态验收(非仅编译/单测)。

## 13. 静态加密 + KDF 卫生

- 🔴 node 清算行假 AES-GCM(`keystore.rs:181-220` XOR+blake2,函数名说谎)→ 真 `Aes256Gcm`+`Argon2id`+改注释;**每次 12B CSPRNG 随机 nonce**(或 XChaCha20-Poly1305 24B nonce 规避 GCM nonce 悬崖);**Argon2id 参数(m/t/p)单一常量源跨 wumin/wuminapp/node 统一**;旧 XOR `.enc` 作废、重新 `save_signing_key` 导入。
- CPMS master/geo_seal(Blake2b 单次)→ HKDF-SHA512(info **带固定域前缀**,非裸 key_id);App 锁口令 → Argon2id;对称统一 AES-256-GCM。
- `gmb-pqc` crate:HKDF 派生表(仅 ML-DSA account domain)+ algo 常量 + **domain 常量强制 `[u8;N]`**(修 `batch_item.rs:39/42` 的 `&[u8]`)+ `verify_by_algo`。
- **(决策6)card0 不含 IM**:card0 = 清算行真 AES-GCM + App 锁 KDF + 热钱包 at-rest;IM 归 card6。

## 14. 全签名面穷尽清单(H15)+ 任务卡

**所有用户/系统签名面(逐项归属,杜绝漏面):**
① 普通链上交易(GmbPqcAuth, card2/3)② L3/offchain payer 支付(card2)③ SFID 钱包绑定证明 binding.rs(card4)④ 治理/扫码签名(card2/3)⑤ CPMS ARCHIVE/wallet_sig(card5)⑥ **IM 设备绑定 `GMB_IM_WALLET_BINDING_V1`(card6)**⑦ **登录回执 login_receipt(card3)**⑧ SFID MAIN/sheng 系统签名(card4+card2)⑨ **seal 共识签名(card7,独立)**。每项归属验证口径同 §9(ML-DSA 公钥需经 AccountPqcKey 证明属于该地址)。

| 卡 | 范围 |
|---|---|
| card0 | 卫生修复(清算行真 AES-GCM+Argon2id / App锁KDF / 热钱包at-rest)**不含 IM** |
| card1 | gmb-pqc crate(KDF 精确+锁库+golden vector 含ξ)+ §11 全部 spike 闸门 |
| card2 | GmbPqcAuth 扩展授权(嵌套tuple/following_extensions_hash/transaction_version++)+ account-keys(AccountPqcKey/PqcPolicy/轮换PoP)+ 5验签器algo-tag + 6签名长度上限放宽 + L3 PQC授权 |
| card3 | 钱包(sr25519直接派生)+ **冷钱包FFI** + **离线metadata策略** + **QR分片** + bootstrap + login_receipt |
| card4 | SFID MAIN signer ML-DSA(与card2原子上线)+ **binding.rs归属验证(查链)** + verify_cpms_archive_qr + build_signature_message含sig_alg |
| card5 | CPMS ARCHIVE签名(脱离[u8;32])+ wallet_sig_alg放开(符号定位)+ KDF + **install_sig验证下沉SFID/CPMS验** + 归属下沉SFID |
| card6 | IM X-Wing(ML-KEM-768,一次到位含AES-256)+ TLS X25519MLKEM768 + **GMB_IM_WALLET_BINDING_V1 签名升级** + 群rekey归属 |
| **card7(新)** | **seal 共识签名 ML-DSA-65 = 节点二进制协调升级**(非 setCode):激活高度 + 新旧节点验块共存窗口;与钱包 PQC 主线解耦;**Phase C/D 收紧治理**(bootstrap_deadline 设定 + 多轮公告 + 无恢复告知) |

## 15. 已拍板决策(2026-06-18)

1. **Phase D 关窗不保留恢复通道**——未绑定老用户逾期=资产终态锁定,靠提前充分公告。
2. **ML-DSA 私钥丢失/泄露不保留 sr25519 锚点回退**——绑定后只认 ML-DSA。
3. **账户不派生 ML-KEM**——机密性 KEM 仅在 IM/TLS 层,与账户密钥无关。
4. **PQC 交易 era 钉死 immortal**——PoW 难度无问题,不用 mortal/checkpoint。
5. **CPMS install_sig 补真实验证**。
6. **card0 不动 IM**——IM ciphersuite 一次性由 card6 完成。
7. **CheckMetadataHash 保持 Disabled**——不启用 metadata 绑定。
8. **ML-DSA 钱包签名归属验证统一查链 `AccountPqcKey`**——SFID 经 subxt 查,CPMS 下沉 SFID,未 bootstrap 前拒。
