# ADR-022:GMB 抗量子签名升级 + 统一加密方案（真源 v5）

状态:Accepted(2026-06-18,v5 2026-06-21)。**本 ADR 是 GMB 抗量子方案的唯一真源**,取代并删除一切旧 PQC 迁移方案文档。
关联:[[feedback_no_compatibility]]、[[feedback_chainspec_frozen]]、[[feedback_pubkey_format_rule]]、[[project_pqc_unified_adr022]]
v4 变更:并入 8 项已拍板决策(见 §15)+ 7 视角对抗审查的 13 BLOCKER + 关键 HIGH 修正。
v5 变更(2026-06-21,对抗复审 28 项确认后定稿):
- 🔴 **`extra` 必带 `account`**(§3/§5):ML-DSA **无公钥恢复**(no ecrecover),General Transaction origin=None 下链端必须靠 extra 内 `account` 查 `AccountPqcKey` 才能取验签公钥;account 是查表 hint,随后被 ML-DSA 签名 + payload 内 account 双重绑定,攻击者填他人 account 也伪造不出签名。
- 🔴 **fail-safe 拆三语境**(§3/§4):创世初值=phase=A / 正常 Phase B 运营值 / decode 失败 fallback;decode 失败=**sr25519 不冻结 + PQC/bootstrap 关**(保守且瞬态,非永久锁人),不再笼统说"fail-open 到 phase=B"。
- 🔴 **创世前对 PQC 零改动(2026-06-21 进一步精简,覆盖"烘骨架"口径)**(§16):当前 runtime 已支持创世后 setCode 加 TxExtension/pallet,故 GmbPqcAuth/account-keys/transaction_version 全部后置;**`transaction_version` 0→1 bump 发生在"启用 PQC 的那次 setCode"**(TxExtension 结构变更),不在创世;spec_version 每次 setCode 递增。
- domain 字面量全文统一 `GMB_PQC_TX_MLDSA65_V1` / `GMB_PQC_BOOTSTRAP_MLDSA65_V1`;`extra=None` 透传澄清(weight 不读 storage,validate 对 sr25519 signed 仍读 AccountPqcKey/PqcPolicy 判已绑定拒);bootstrap 前置 providers 机制纠正(CheckNonce 只查 nonce,费由 ChargeTransactionPayment 拒);txpool DoS 补具体节点级机制;L3/CID 补真实代码行号;HKDF info 字节锁定;sign_response 补定义;seal 明确排除创世冻结。

---

## 0. 方案目标 + 钉死点

GMB **当前继续用 sr25519 签名**(链上已有真实用户);**未来通过链 runtime 升级 + citizenapp/citizenwallet 客户端升级,在位切换到 PQC 签名**。用户不换助记词/账户/地址/余额。**核心原则:AccountId 是账户身份锚点,签名算法只是该账户的授权方式。**

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
- HKDF-SHA512(RFC5869)Extract-then-Expand;`PRK=HKDF-Extract(salt=空, IKM=AccountSeedV1[32B])`,域分离全靠 `info`。
- 🔴 **`info` 是字面 ASCII 字符串 `"GMB/account/ml-dsa-65/seed32/v1"`(无 null 结尾、无独立长度前缀字节;串内 `seed32` 只是输出长度的字面标识,不是要额外塞一个长度域字节)**。Rust 传 `&[u8]`,Dart 按 UTF-8 编码;golden vector 锁死这串的精确字节序列(31 字节),三端(Rust 二进制 / iOS-Android FFI / Dart)必须逐字节一致,否则 KDF 输出发散→跨端公钥不同。
- ML-DSA-65:`HKDF-Expand(PRK, info, L=32)` → 32B ξ,**直接当 ξ** 喂 FIPS204 `KeyGen_internal(ξ)`。
- **ML-DSA-87(algo 0x03 预留)未来如启用,须用独立 info(如 `"GMB/account/ml-dsa-87/.../v1"`)+ 独立 AccountPqcKey schema/version,不复用 65 的 info,不塞进 `BoundedVec<2048>`。**
- 🔴 **(B8)强制选用暴露 `KeyGen_internal(ξ)` seed-API 的库**(fips204 锁定版本);**删除一切 DRBG fallback**——库不暴露 ξ-API 则换库,不接受 DRBG 替代(否则同助记词跨端派生不同公钥)。库名+版本+API 名钉进 card1 spike。
- **golden vector** 必须含中间量 ξ 与最终 {sr25519 SS58, ML-DSA-65 公钥},供冷/热/链/后端逐字节对拍;库升级须重跑。

## 3. 链上签名策略(路线 A = GmbPqcAuth,定稿)

**当前**:普通 extrinsic 继续 MultiSignature/sr25519,全部现状不变。

**未来 runtime 升级后:**
- **不扩展 `MultiSignature`**;**链上授权 = General Transaction + 自定义 `GmbPqcAuth` TransactionExtension**(废弃 pqc_dispatch pallet call 主路径)。
- `GmbPqcAuth` 放扩展流水线**最前**:`validate` 验 ML-DSA-65 后**返回 `Signed(account)` origin**(`prepare` 不改 origin);后续 `CheckNonce`/`ChargeTransactionPayment` 走系统标准逻辑。
- 🔴 **tuple 12 上限**:`TxExtension` 已 12 项(`lib.rs:164-176`)→ 嵌套 `(GmbPqcAuth, AuthorizeCall)` 占第一项槽位,不加第 13 项。
- 🔴 **(B12 v5 修正)transaction_version 在"启用 PQC 的 setCode"时 0→1 bump,非创世设定**:创世前不加 GmbPqcAuth,`transaction_version` 保持 0;**启用 PQC 的那次 setCode 把 TxExtension 第一项改成嵌套 `(GmbPqcAuth, AuthorizeCall)` → `transaction_version` 0→1++ 且 `spec_version++`**,`CheckTxVersion` 据此拒未升级旧客户端(那一刻推送 PQC 版客户端,计划内)。详见 §16(创世前对 PQC 零改动)。
- 🔴 **`extra` 必有变体 `{ None | Pqc{account,sig,key_version} | Bootstrap{account,pqc_pubkey,ml_dsa_sig,sr25519_bootstrap_sig,key_version} }`**(v5:`Pqc/Bootstrap` 必带 `account`):Phase A/B 普通 sr25519 交易也过此扩展,`None` 表达"非 PQC 授权"。
  - 🔴 **为何 extra 必带 `account`**:ML-DSA **无公钥恢复机制**(不同于 sr25519/ECDSA 的 ecrecover),General Transaction origin=None 下无外层 signer → `GmbPqcAuth.validate` 在验签前**必须先有 `account`** 才能查 `AccountPqcKey[account].pubkey` 取得验签公钥(否则只能遍历全部账户公钥试验签=性能灾难+DoS)。`account` 仅是查表 hint,**不被信任直到验签通过**:ML-DSA 签名须对 `AccountPqcKey[account].pubkey` 验通,且签名 payload 内含 `account`(§5)、`call_hash` 绑定 call → 攻击者填他人 `account` 也伪造不出签名。
- 🔴 **`extra=None` 透传语义(v5 澄清)**:`extra=None` 时 `GmbPqcAuth` **不改 origin、不写 storage**;但对 **sr25519 signed origin**,`validate` **仍需读 `AccountPqcKey[signer]`+`PqcPolicy`** 判"已绑定账户拒 sr25519"(故 validate 读 storage 是允许的,见下 weight 区分);origin=None 的 authorized general call(无 signer)直接透传给 `AuthorizeCall` 不被误伤。
- 🔴 **授权模式与 origin 互斥**:`extra=Pqc|Bootstrap` 只允许 General Transaction(origin=None);sr25519 signed extrinsic 携带 PQC proof 直接 `BadProof`。
- 🔴 **(H1)单测**:① `None`+authorized call → 原 origin 透传 → AuthorizeCall 产 Authorized;② `Pqc` → Signed → AuthorizeCall 不二次授权;③ `None`+sr25519 signed+已绑定账户 → 按 phase 拒/放(覆盖 validate 读 storage 路径)。
- 🔴 **(M2 v5 澄清)`weight()` 纯 `self.extra` 路由 card1 benchmark 常量、严禁读 storage**(weight 必须在 PqcPolicy decode 之前可算);**`validate()` 可读 storage**(查 AccountPqcKey/PqcPolicy 判绑定与 phase)——二者分工不矛盾。Bootstrap weight 取最坏(1952B 写入+双验签)。
- 🔴 **(M10 v5 修正)fail-safe 拆三语境**:
  - **建表初值**(启用 PQC 的那次 setCode 建 PqcPolicy 表时):`phase=A`/`reject_sr25519_when_bound=false`/`allow_bootstrap_unbound=false`/`deadline=None`——刚 setCode 加完验签逻辑、治理尚未推进,起步 phase=A。
  - **正常运营值**(治理逐阶段设):见 §6(Phase B 起 已绑定账户 reject=true、未绑定 allow_bootstrap=true)。
  - **decode 失败 / storage 缺失 fallback**:等价 phase=A 安全态 = **sr25519 不冻结 + PQC/bootstrap 拒**;**绝不 fail-closed 冻结全链,也绝不 fail-open 打开 bootstrap**。因 PqcPolicy 是简单结构、修复(setCode)后即恢复,此 fallback 是**瞬态**,不会像 Phase D 那样永久锁人。
- 🔴 **(M3 修正)metadata 绑定不夸大**:`CheckMetadataHash` 全链保持 **Disabled**(决策7),其 implicit=None;runtime/版本隔离实际靠 `spec_version`+`transaction_version`+`genesis_hash`+`call_hash`,非 metadata hash。
- **`account-keys` pallet(idx=27)只承载** `AccountPqcKey`/`PqcPolicy` 的存储/查询/事件/密钥轮换;**不承载主交易派发**。

**无感首笔绑定(bootstrap):**
1-3. 用户升级 → 同助记词派生当前地址 + ML-DSA-65 公钥 → 照常发起一笔交易。
4. 链上无该账户 PQC 公钥 → 客户端构造**首笔 bootstrap General Transaction**(`GmbPqcAuth` 扩展授权)。
5. 冷/热钱包一次确认,同时出 **sr25519 bootstrap 证明** + **ML-DSA 交易签名**。
6. `GmbPqcAuth.validate` 按 §5 验序通过 → 返回 `Signed(account)` → nonce/扣费/业务 dispatch;**绑定写 `AccountPqcKey` 在 `post_dispatch`**(nonce/扣费已跑)。
7. 后续只用 ML-DSA。
> 🔴 **(H1/post_dispatch)失败语义**:绑定写在 post_dispatch,内层 call 失败绑定仍保留、照常收费;**post_dispatch 绝不返回 Err**(返回 Err 会作废整个区块=远程 DoS)——冲突判定(已绑定不同值)前移到 `validate` 拒,post_dispatch 任何情况返回 `Ok`(未绑定→写、已绑定→no-op)。
> 🔴 **(H2 v5 修正)bootstrap 账户须有 `AccountInfo`(有余额即 providers>0,nonce 可追踪)且余额≥手续费**。机制纠正:**`CheckNonce` 只查 nonce 窗口,不查 providers/余额**;手续费不足由 **`ChargeTransactionPayment`** 拒(非 CheckNonce)。客户端构造 bootstrap 前必须检查账户存在性 + 可付余额;provider=0(从未收过资产)账户的 bootstrap 须给明确错误语义 + 单测(provider=0 但余额充足应过 CheckNonce、在 ChargeTransactionPayment 生效)。

> **当前固定 sr25519 是 Phase A 真相不是旧残留**(QR/CPMS/CID `wallet_sig_alg=='sr25519'` 硬编码)→ 改"Phase A 只收 sr25519 → 分流",不删。

## 4. 链上存储

```
AccountPqcKey[AccountId] = { alg:0x02(ML-DSA-65), key_version:u32, pubkey:BoundedVec<u8,ConstU32<2048>>, bound_at:BlockNumber }
PqcPolicy = { phase, bootstrap_deadline:Option<BlockNumber>, reject_sr25519_when_bound:bool, allow_bootstrap_unbound:bool }
```
- 删除 `bootstrap_mode` 字段(M15:无第二变体、无消费方、疑似违反 per-account-state 禁令)。
- `account-keys` **pallet_index=27**(契约真源在此登记;已核实当前 runtime 用 0..=26,27 空闲;**pallet 集属永久地基,创世前冻结**)。
- 🔴 **(v5)PqcPolicy 建表初值 = `phase=A`/`reject_sr25519_when_bound=false`/`allow_bootstrap_unbound=false`/`bootstrap_deadline=None`**(account-keys pallet 在"启用 PQC 的 setCode"才加入,建表即起步 phase=A;创世前根本无此 pallet)。**该 setCode 后治理推进**才进入 Phase B 运营值(`allow_bootstrap_unbound=true`、已绑定账户 `reject_sr25519_when_bound=true`,见 §6)。**decode 失败/storage 缺失的瞬态 fallback 同 phase=A 安全态**(§3 M10)。`2048` 上限只覆盖 ML-DSA-65,ML-DSA-87 须新 alg+新 schema(§2)。
- **绑定规则**:未绑定→允许首笔 bootstrap;已绑定→拒再次 sr25519 覆盖(first-bind-wins,冲突在 validate 拒)。
- 🔴 **(H3)密钥轮换双签**:轮换 = ① 当前 PQC 私钥授权 + ② **新 PQC 私钥对 `(新公钥+新 key_version+account+genesis_hash)` 自签(PoP)**,两签皆过才 `key_version++` 写新 `AccountPqcKey`。
- 🔴 **(决策1/2)无恢复通道**:绑定后授权**只认 ML-DSA**,无 sr25519 锚点回退;**ML-DSA 私钥泄露/丢失无代绑恢复**(用户须妥善备份助记词——助记词在即可确定性重派生同一 ML-DSA 私钥;但若 ML-DSA 私钥被泄露并被攻击者抢先轮换,无回退,账户即失陷)。Phase D 关窗后未绑定老用户=资产终态锁定(见 §6)。

## 5. 交易载荷 + 验签(反重放域)

**`GmbPqcAuth.extra` 签名 payload(域标签 `DOMAIN_TX = b"GMB_PQC_TX_MLDSA65_V1"`,普通 PQC 交易):**
`DOMAIN_TX` ++ SCALE(`genesis_hash`、`spec_version`、`transaction_version`、`account`、`nonce`、`era_or_deadline`、`tip`、`call_hash`、`key_version`、`following_extensions_hash`)。
- 签名内含 `account` 与 extra 内 `account` 是同一值(§3:extra 内 account 是查表 hint,此处进 payload 让签名对 account 承诺,双重绑定)。
- `ss58_format` 为**纯展示字段**(L2:链上无对应扩展 implicit,不参与一致性比对,链域已由 genesis_hash 隐含)。
- `sig_alg`/`alg` 算法标识**进 DOMAIN 字面量**(`GMB_PQC_TX_MLDSA65_V1`),不另设字段(H7/rank8 域+算法隔离)。

**首笔 bootstrap(域标签 `DOMAIN_BOOTSTRAP = b"GMB_PQC_BOOTSTRAP_MLDSA65_V1"`):** 字段集**与 `GMB_PQC_TX_MLDSA65_V1` 对齐**(补 transaction_version/tip/era_or_deadline,H4),额外含 `pqc_pubkey_hash`。
- 🔴 **(v5)签名格式是算法相关结构,非裸字节**:sr25519 签名固定 64B,ML-DSA-65 签名 ~3309B,ML-DSA 公钥 ~1952B → bootstrap extrinsic body ≈ `sr25519_sig(64B)+ml_dsa_sig(3309B)+ml_dsa_pubkey(1952B)+call` ≈ **5.3KB+,hex 编码 ~10KB+**(QR 必须分片,见 §8 B11)。后续普通 PQC 交易**不带公钥**,只带 ml_dsa_sig(~3309B)。

🔴 **(B3)`following_extensions_hash` 口径 = SDK `inherited_implication` 精确递归编码,不是扁平拼接**:
`blake2_256( ImplicationParts{ base: TxBaseImplication((extension_version, call)), explicit:(following_explicit_tail, parent_explicit), implicit:(following_implicit_tail, parent_implicit) }.encode() )`,严格复刻 `runtime traits/transaction_extension/mod.rs:577-598`。嵌套 tuple `(GmbPqcAuth, AuthorizeCall)` 下 GmbPqcAuth 看到的 explicit/implicit tail 是"内层 tail 再与 outer tail 组对"的递归结构。card1 出该字节 golden vector,card2 单测断言链端 `inherited_implication.encode()` 与协议口径逐字节相等(参照 mod.rs:712-869)。
> 参与 following implicits 的扩展逐一列明:`CheckSpecVersion()`/`CheckTxVersion()`/`CheckNonStakeSender()`/`CheckGenesis(genesis_hash)`/`CheckMortality(immortal→genesis_hash)`/`CheckNonce()`/`CheckWeight()`/`ChargeTransactionPayment()`/`CheckMetadataHash(None,Disabled)`/`WeightReclaim()`。**(M16)immortal 下 CheckMortality.implicit 仍是 genesis hash,不可漏。**

🔴 **(B1)bootstrap challenge 字面构造(钉死)**:
`sr25519_bootstrap_signature = sr25519_sign(blake2_256(DOMAIN_BOOTSTRAP ++ SCALE(genesis_hash, spec_version, transaction_version, account, pqc_pubkey_hash, key_version, nonce, call_hash, following_extensions_hash)))`。
硬约束:① sr25519 签名**必须覆盖 `pqc_pubkey_hash`**("我授权这把 PQC 公钥");② 专用 DOMAIN_BOOTSTRAP 前缀与 CID/治理/L3 的 sr25519 签名域不可互换;③ ML-DSA 交易签名**反向覆盖** `blake2_256(sr25519_bootstrap_signature)`(双向交叉绑定,杜绝两签拼接)。card2 单测:挪用其它域 sr25519 签名构造 bootstrap 必拒。

**验签顺序(钉死,hash 全 `blake2_256`):**
- **bootstrap**:① `blake2_256(body.pqc_pubkey)==payload.pqc_pubkey_hash` → ② sr25519 bootstrap challenge(覆盖 pqc_pubkey_hash)→ ③ ML-DSA 交易签名(用 body 公钥,且覆盖 sr25519 签名 hash)→ 通过才 post_dispatch 写。
- **普通 PQC**:ML-DSA 签名验交易 payload,公钥从 `AccountPqcKey` 按 account 读;`alg` 必须等于 `AccountPqcKey.alg`(防降级)。
- `account` = sr25519 公钥派生的当前 AccountId;`call_hash` 与 following_extensions_hash 内 call 必须**同一份字节序列**(M6:最简——following 直接复用 call_hash)。
- **(L9)txpool `provides=(account,nonce)` 由标准 CheckNonce 在 Signed origin 下自动产生,GmbPqcAuth 不重复设**;GmbPqcAuth.validate 对 nonce 只做廉价窗口预检。
- 🔴 **(H12/B11 v5)txpool DoS 需具体节点级机制,不能只写"限速"**:Substrate txpool **无内建 per-account 限速**。防线分层:① validate 内 cheap-checks **在 ML-DSA 验签之前**全部跑完(body 长度上限 ~10KB 硬校验 / BoundedVec decode / `AccountPqcKey[account]` 存在性 / `alg` 匹配 / nonce 廉价窗口预检)→ 挡掉绝大多数垃圾;② 仍可能有"合法 cheap-check + 错误 ML-DSA sig"灌池逼链端做昂贵验签 → 需 **node 层(citizenchain/node)实装 `(account,source)` 失败签名滑窗计数 + 未绑定 bootstrap 类交易在池中占比上限 + 超限淘汰**;③ 单测:构造 100 笔同账户错误签名 bootstrap,验证 txpool 不被拖垮。card2 必须给出具体节点级实现(非一句"限速")。

🔴 **(B4)era 钉死 immortal(决策4,PoW 难度无问题)**:`era_or_deadline=immortal`,链域靠 genesis_hash,不带 checkpoint;CheckMortality.implicit 仍是 genesis hash(已纳入 following_extensions_hash)。

### 5.1 L3 / offchain payer 授权(资金侧 BLOCKER,真实代码行号)
L3 批量支付的 payer/batch 授权**必须与链上账户签名同源**,否则主链 Phase C/D 拒 sr25519 后,L3 仍是 sr25519 花钱后门。落地点(card2 §8):
- 🔴 **`configs/mod.rs:1261` `MaxBatchSignatureLength = ConstU32<128>` → `ConstU32<4096>`**(128B 装不下 ML-DSA ~3309B)。
- 🔴 **payer_sig / batch_signature 改带 sig_alg 标签的变长结构** `{ sig_alg:u8, key_version:u32, sig:BoundedVec<u8,4096> }`(对齐 §14 六处签名上限放宽)。
- 🔴 **`settlement.rs:169` 当前 `Sr25519Signature::try_from(&item.payer_sig[..])` 硬编 sr25519 → `verify_by_algo` 按 `AccountPqcKey[payer]` 路由**;`lib.rs:644-663` 的 `sr25519_pubkey_from_account`("公钥可从 account 反推")假设作废——ML-DSA 公钥不可由 account 反推,只能查 AccountPqcKey。
- 🔴 **`settlement.rs` 当前无 `PqcPolicy` phase 检查 → 必须加**:已绑定账户在 Phase C/D 拒 sr25519 payer、Phase D 后未绑定不可继续 sr25519 L3 花费——**这是真正堵后门的一步**,只改签名长度不够。offchain worker 预聚合阶段须同步感知 PqcPolicy(或 settlement 提交前再验一次),避免 L3/L2 phase 视图不一致导致批次行为不确定。

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

## 7. citizenapp(热钱包)

同一地址;Rust FFI(gmb-pqc)`ml_dsa65_public_from_seed`/`ml_dsa65_sign`;查 `PqcPolicy` 分流(Sr25519Only→sr25519;PqcPrepared/PqcPrimary→PQC General Transaction;未绑定→首笔 bootstrap);UI 不展示 PQC 公钥/绑定过程/换账户;同助记词恢复同地址+同 ML-DSA 密钥。

## 8. citizenwallet(冷钱包)+ QR

- 🔴 **(B9)冷钱包新建 Rust FFI 子工程**(citizenwallet 现纯 Dart 无 rust/):对标 citizenapp/rust 的 cdylib/staticlib + Android/iOS target + cbindgen,把 gmb-pqc 编进冷热两端。
- 🔴 **(B10)离线 metadata 策略**:"按 metadata 重建 following_extensions_hash"整体留在在线热钱包;QR 携带重建所需最小要素(extension_version + 各后续扩展显式值 + 预算 hash),**冷钱包用 gmb-pqc 本地 SCALE 重算并比对**,且**自己从助记词派生 ML-DSA 公钥核对 `pqc_pubkey_hash`**(不盲信 QR 公钥),核对通过才出双签;**严禁退化成 wasm 式纯哈希盲签**(保持两色识别·decodeFailed 即拒)。
- 🔴 **(B11)QR 分片**:envelope 加 `chunk_index/chunk_total/total_hash`;渲染端多帧轮播(单帧字节预算按 ECC=M version≤40 反推);扫描端分片聚合状态机(按 id 归并/去重补帧/校验 total_hash);放开 32768 上限;最坏 bootstrap(~10KB+)真机实测。
- **(H18)QR body 四处(冷热 request/response)放开** `sig_alg(sr25519|ml-dsa-65)`+`auth_mode`+`key_version`+`chunk_*`,Phase A 仍只收 sr25519;进签名的 hash 一律 gmb-pqc blake2_256,**禁复用 qr_signer 的 sha256**。

## 9. CID / CPMS 边界

- 不托管助记词/seed/私钥;`wallet_sig_alg` 放开枚举但不生成新地址。
- 🔴 **(决策8/B5/B7/H7)ML-DSA 钱包签名归属验证 = 统一查链 `AccountPqcKey[wallet_address]`**:
  - **唯一权威源 = 查链上 `AccountPqcKey`**(CID 经 subxt 0.43.1 查,已具备能力);确认该 ML-DSA 公钥已绑到此 sr25519 锚点才接受;**未完成 bootstrap → 拒 ML-DSA 钱包签名**(退化策略)。CID/CPMS **必须用同一选项**。
  - **(B5)CID `citizens/binding.rs`(:79/340/878)是唯一真实落点**:ML-DSA 时 QR 单独带公钥(不再 ss58 反推)、查链验归属、verify_by_algo 验签;脱离 `[u8;32]`。card4 增列。
  - **(B7)CPMS 无链客户端**:CPMS **不验钱包签名**(本就不验,P-CRED-002),归属判定下沉 CID;card5 删①②选项。
- 🔴 **(B6)CID MAIN signer 迁 ML-DSA 必须与 card2 链端验签器原子同批上线**——允许序仅两种:**(A)链端先切 `verify_by_algo` 路由(支持双算法、仍默认 sr25519),再 CID 切 ML-DSA 发证**;**(B)链端验签器与 CID signer 同批发布**。**禁止 CID 先切 ML-DSA 而链端仍 sr25519_verify**,否则机构注册/投票/人口快照全红。
- 🔴 **(v5 真实代码行号)CID/CitizenPassport 三处硬编 sr25519 需 verify_by_algo / sig_alg 化(card4)**:
  - `citizencode/backend/citizenpassport/handler.rs:1017` `verify_sr25519_signature(&cpms_pubkey, ...)` 验 ARCHIVE 档案签名硬编 sr25519 → `verify_by_algo`(从 archive 协议串/元数据解析 sig_alg);否则 CPMS ARCHIVE 转 ML-DSA(card5)后 CID 端档案验签全红。
  - `citizencode/backend/core/qr/mod.rs:118-141` `build_signature_message`(格式 `QR_V1|kind|id|system|expires_at|principal`)**无 sig_alg** → 加 sig_alg 进 preimage(H9,防算法混淆/降级)。
  - 🔴 **治理签名有两套 builder**:上面 CID `qr/mod.rs` 一套 + `citizenchain/node/src/governance/signing.rs`(`sig_alg:String` 硬编 sr25519)一套,格式不一致 → unified-protocols 须定真源,两处都迁(最好抽单一 `build_signature_message` 共享)。
- 🔴 **(决策5 v5)CPMS install_sig 补真实验证 + 具体机制**:`citizencode/backend/citizenpassport/handler.rs:1343-1419` 当前 CID 端 `sign_with_main_key` **只签不验**,CPMS 无链访问无法独立验 → "任何人投递 install QR 即可初始化 CPMS" 安全洞未堵。card5 须给具体机制(install_sig 入 QR payload + CPMS 初始化时上传 + 向 CID 增 `/verify-install-sig` 端点回源验,或 CPMS 端 activated_at 防重放时间窗),非仅删旧表述。
> **(v5 已定位)代码分布**:登录/安装授权(CID/CitizenPassport)在 `citizencode/backend/citizenpassport/`(`handler.rs` build_install_sign_source:1336 / sign_with_main_key:1405 只签不验);**CPMS ARCHIVE/dangan 在 `citizenpassport/backend/dangan/mod.rs`**(ARCHIVE `[u8;32]` 撞穿点在此,card5 落点),与上是两个不同 `citizenpassport/` 根,勿混。

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
- CID 查链 AccountPqcKey 验 ML-DSA 钱包签名归属;CPMS install_sig 被真实验证;transaction_version 已 bump 旧口径交易被明确拒。
- 全签名面(见 §14)无遗漏;真实运行态验收(非仅编译/单测)。

## 13. 静态加密 + KDF 卫生

- 🔴 node 清算行假 AES-GCM(`keystore.rs:181-220` XOR+blake2,函数名说谎)→ 真 `Aes256Gcm`+`Argon2id`+改注释;**每次 12B CSPRNG 随机 nonce**(或 XChaCha20-Poly1305 24B nonce 规避 GCM nonce 悬崖);**Argon2id 参数(m/t/p)单一常量源跨 citizenwallet/citizenapp/node 统一**;旧 XOR `.enc` 作废、重新 `save_signing_key` 导入。
- CPMS master/geo_seal(Blake2b 单次)→ HKDF-SHA512(info **带固定域前缀**,非裸 key_id);App 锁口令 → Argon2id;对称统一 AES-256-GCM。
- `gmb-pqc` crate:HKDF 派生表(仅 ML-DSA account domain)+ algo 常量 + **domain 常量强制 `[u8;N]`**(修 `batch_item.rs:39/42` 的 `&[u8]`)+ `verify_by_algo`。
- **(决策6)card0 不含 IM**:card0 = 清算行真 AES-GCM + App 锁 KDF + 热钱包 at-rest;IM 归 card6。

## 14. 全签名面穷尽清单(H15)+ 任务卡

**所有用户/系统签名面(逐项归属,杜绝漏面):**
① 普通链上交易(GmbPqcAuth, card2/3)② L3/offchain payer 支付(card2 §5.1)③ CID 钱包绑定证明 binding.rs(card4)④ 治理/扫码签名(card2/3,两套 builder 见 §9)⑤ CPMS ARCHIVE 档案自签名 + wallet_sig 归属(card5)⑥ **IM 设备绑定 `GMB_IM_WALLET_BINDING_V1`(card6)**⑦ **登录签名响应 sign_response(card3)**⑧ CID MAIN/sheng 系统签名(card4+card2)⑨ **seal 共识签名(card7,独立)**。每项归属验证口径同 §9(ML-DSA 公钥需经 AccountPqcKey 证明属于该地址)。
> 注:⑤ CPMS **ARCHIVE 档案自签名**(机构自身密钥签档案,card5)与 **wallet_sig 归属**(用户钱包签名,归属下沉 CID)是两个面,勿混。⑦ **sign_response 定义**:钱包对登录会话(`session_id`/`device_fingerprint`/`timestamp`/专用 domain 标签)的签名,按 sig_alg 升 ML-DSA,归属同 §9;card3 给出精确字段+域标签(若评估后由 IM 设备绑定覆盖则显式删除,不留模糊)。

| 卡 | 范围 |
|---|---|
| card0 | 卫生修复(清算行真 AES-GCM+Argon2id / App锁KDF / 热钱包at-rest)**不含 IM** |
| card1 | gmb-pqc crate(KDF 精确+锁库+golden vector 含ξ)+ §11 全部 spike 闸门 |
| card2(创世后启用 PQC 的 setCode) | GmbPqcAuth 扩展授权(嵌套tuple/following_extensions_hash/**extra带account**/**transaction_version 0→1 bump @本次setCode**/**fail-safe三语境**/**extra=None validate读storage**/**txpool DoS节点级机制**)+ account-keys(AccountPqcKey/PqcPolicy/轮换PoP)+ 5验签器algo-tag + 6签名长度上限放宽 + **L3 PQC授权(§5.1:configs:1261 128→4096 / settlement:169 verify_by_algo / settlement加PqcPolicy phase检查 / sr25519_pubkey_from_account废)** |
| card3 | 钱包(sr25519直接派生)+ **冷钱包FFI** + **离线metadata策略** + **QR分片** + bootstrap + **sign_response(补精确字段+域标签)** |
| card4 | CID MAIN signer ML-DSA(与card2原子上线,序A/B)+ **binding.rs归属验证(查链)** + **handler.rs:1017 verify_by_algo** + **qr/mod.rs:118-141 build_signature_message含sig_alg** + **governance/signing.rs第二套builder同迁** |
| card5 | CPMS ARCHIVE自签名(脱离[u8;32])+ wallet_sig_alg放开(符号定位)+ KDF + **install_sig真实验证(handler.rs:1343-1419,给具体机制)** + 归属下沉CID + **先确认CPMS crate是否在本仓** |
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
8. **ML-DSA 钱包签名归属验证统一查链 `AccountPqcKey`**——CID 经 subxt 查,CPMS 下沉 CID,未 bootstrap 前拒。
9. **(2026-06-21 修正,覆盖 v4 旧口径)下一步是永久创世(此后不再清链);创世前对 PQC 零改动**——核验当前 runtime(`sp-runtime 45`/`AuthorizeCall`/`Preamble::General`/最终 sr25519 派生)即可在创世后用 **一次 setCode 接入全套 PQC**(含 GmbPqcAuth/account-keys/transaction_version 0→1),**无需重新创世**;v4 的"创世前烘骨架"口径**作废**(见 §16)。

## 16. 与永久创世的时序边界(创世前对 PQC 零改动 / 全套 PQC 创世后 setCode)

下一步将**清链重新创世,此后不再清链**=永久链。**核验结论(2026-06-21,v5):当前预创世 runtime(`sp-runtime/frame 45` + `AuthorizeCall` 在位 + `UncheckedExtrinsic = generic::UncheckedExtrinsic` 支持 `Preamble::General` + 地址派生已是最终 sr25519)原样即可在创世后用 setCode 接入全套 PQC——加 pallet、加 TxExtension(transaction_version bump)、存储迁移全部是标准 runtime 升级,均不触发重新创世。** 故**创世前对 PQC 零改动**;v4 的"只烘骨架"口径**已撤销**(它把最高风险的 S2/S3 spike + 客户端 wire 改动错误地前置到赶工的创世,且无实益——TxExtension 任何时候 setCode 加都一样无感)。

### 16.1 创世前(对 PQC 零改动,只读核验)
1. **核验 4 项(现状已全过,只读不改代码)**:① TxExtension 保留 `frame_system::AuthorizeCall`(General-tx 基础设施,`lib.rs:165`);② `UncheckedExtrinsic = generic::UncheckedExtrinsic`(支持 `Preamble::General`,`lib.rs:237`);③ 地址派生 = 最终 `miniSecretFromEntropy → sr25519.fromSeed`;④ `sp-runtime/frame=45` 足够新,后续 setCode 能加 TxExtension/pallet。
2. **用当前 runtime 重新创世**(为行政区/预发布做的那次),`transaction_version` 保持 **0**,**不加任何 PQC 物件**(无 gmb-pqc / 无 account-keys / 无 GmbPqcAuth)。
3. 不预留 pallet idx、不烘存储形状、不前置 spike——全部归"启用 PQC 前/创世后"。

### 16.2 创世后(全套 PQC 一律 setCode/app/治理/协调,永不再创世)
- **启用 PQC 的那次 setCode(card2)**:加 `account-keys` pallet(idx 取当时空位)+ `GmbPqcAuth` 进 TxExtension(嵌套 `(GmbPqcAuth, AuthorizeCall)`)+ ML-DSA 验签 + bootstrap → **该 setCode `spec_version++` 且 `transaction_version` 0→1++**(TxExtension 结构变更)。这一刻推送 PQC 版客户端(citizenapp/citizenwallet);未适配旧客户端发不出交易 = 计划内升级,**不影响任何账户/余额/地址/state**。
- **启用前先做(不卡创世)**:card1 gmb-pqc crate + golden vector + 🚦 **S2/S3 spike**(polkadart 能编 General Transaction / 嵌套 `following_extensions_hash` 逐字节对拍)——路线 A 闸门,**失败仍可改 setCode-able 的替代方案,绝不回滚重新创世**。
- 之后:治理翻 `PqcPolicy.phase` A→B→C→D;card3 钱包 PQC 路径 / card4 CID / card5 CPMS / card6 IM-TLS / card7 seal+治理 / card0 卫生(随时)。存储类型若需变 ML-DSA 形状(如系统签名公钥)走 `StorageVersion` 迁移,非创世前事。
- **seal** 共识签名在 node 二进制,改它=card7 协调二进制升级(激活高度+import 双接受窗口),与 setCode 无关、与创世无关、不烘骨架。
