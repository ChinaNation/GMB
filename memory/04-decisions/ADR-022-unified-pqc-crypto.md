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

## 3. 链上签名策略 + 无感 bootstrap 绑定

**当前阶段:** 普通 extrinsic 继续走 MultiSignature/sr25519;AccountId/SS58/余额/权限/治理身份全部现状不变。代码/文档去除"sr25519 = 账户身份"表述,改为"sr25519 是**当前**签名算法"。

**未来 runtime 升级后(C1 守则):**
- **不扩展 `MultiSignature`**(避免改 extrinsic 线格式和账户派生模型)。`MultiSignature` 是上游类型(`lib.rs:130/134`),加 ML-DSA 变体=改线格式≠setCode。
- 新增 `account-keys` pallet(或等价 runtime 授权模块),新增 general-transaction 授权路径 **`pqc_dispatch`**。
- `pqc_dispatch` 验 ML-DSA-65 签名后,以 `RawOrigin::Signed(account)` 派发内层 call。
- **费用仍从 account 扣;nonce 用 `frame_system::AccountInfo.nonce`,不另造一套用户 nonce**(见 §11 spike 验证)。

**无感绑定(bootstrap):** 链无法从 sr25519 地址推出 ML-DSA 公钥,需一次"链上知道该账户 PQC 公钥"。设计为**首次 PQC 交易自动绑定并执行**,用户无感:
1. 用户升级 wuminapp/wumin。
2. 客户端从同一助记词派生当前地址 + ML-DSA-65 公钥。
3. 用户照常发起一笔交易。
4. 若链上还没有该账户 PQC 公钥 → 客户端构造 `bootstrap_pqc_dispatch`。
5. 冷/热钱包**一次确认**,内部同时生成:**sr25519 bootstrap 证明**(证"我是这个旧地址的主人")+ **ML-DSA 交易签名**(证"这个 PQC 公钥控制本次交易")。
6. runtime 验两签后写 `AccountPqcKey[account]`,**立即派发内层交易**。
7. 后续交易只用 ML-DSA。

> 对用户是"第一次升级后照常交易",不是"去绑定新账户"。桥**只用于未绑定账户首次进入 PQC**,非长期双轨。

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

**绑定规则:**
- 未绑定账户:允许 `bootstrap_pqc_dispatch`。
- 已绑定账户:**拒绝再次用 sr25519 bootstrap 覆盖** PQC 公钥(first-bind-wins;bootstrap 只能由持 sr25519 私钥者发起,故不弱于现状)。
- 换算法版本:必须由**当前有效 PQC 私钥**授权 + `key_version++`。
- 绑定后,普通 sr25519 用户交易按链上策略逐步拒绝(见 §6 Phase D)。

## 5. 交易载荷(反重放域)

**`pqc_dispatch` payload(`GMB_PQC_TX_V1`):**
`genesis_hash`、`spec_version`、`transaction_version`、`ss58_format`、`account`、`nonce`、`era_or_deadline`、`tip`、`call_hash`、`sig_alg`、`key_version`、`auth_mode`。

**`bootstrap_pqc_dispatch` 额外(`GMB_PQC_BOOTSTRAP_V1`):**
`genesis_hash`、**`spec_version`**(防跨升级重放)、`account`、`pqc_pubkey_hash`、`sig_alg`、`key_version`、`nonce`、`call_hash`。

**验签规则:**
- `sr25519_signature` 只验 bootstrap challenge;`ml_dsa_signature` 验正式交易 payload。
- `account` 必须 = sr25519 公钥派生出的当前 AccountId。
- txpool 用 `(account, nonce)` 做 `provides` 防重复入池。
- `validate_unsigned`/general-tx 验证**轻量无副作用**;写 `AccountPqcKey` + 派发 call 在**执行阶段**完成。

## 6. 签名策略阶段(不要求用户手动参与)

| 阶段 | 链策略 | 用户体验 |
|---|---|---|
| **A 当前** | 只用 sr25519 | 完全不变 |
| **B 预埋** | runtime 支持 PQC 自动绑定,sr25519 仍可用 | 升级客户端后下一笔交易可自动绑定 |
| **C PQC 主用** | 已绑定账户只收 ML-DSA;未绑定旧账户允许 bootstrap 后立即转 PQC | 照常交易,客户端自动处理 |
| **D 收紧** | 长期未绑定账户按治理策略处理;已绑定账户彻底拒绝 sr25519 用户交易 | 大多数用户无感 |

> 🔴 **安全硬约束**:bootstrap 靠 sr25519 证明所有权,**强度等于 sr25519**。sr25519 一旦被量子破,攻击者可伪造 bootstrap 把自己的 ML-DSA 绑到受害者账户并锁死原主。**因此 bootstrap 窗口(Phase D 对长期未绑定账户的治理截止)必须赶在 sr25519 被量子破之前关闭**——这是无感桥的代价,非可选项。

## 7. wuminapp(热钱包)

- 导入/创建仍显示**同一地址**;内部保存 `AccountSeedV1` 语义,不改助记词恢复结果。
- 增 Rust FFI:`ml_dsa65_public_from_seed`、`ml_dsa65_sign`(经 `gmb-pqc`)。
- 查 runtime 签名策略:`Sr25519Only`→构造普通 sr25519 extrinsic;`PqcPrepared/PqcPrimary`→优先 `pqc_dispatch`;**账户未绑定**→构造 `bootstrap_pqc_dispatch`。
- UI **不展示** "PQC 公钥""绑定状态机""换账户";交易记录仍按原地址归集。
- 恢复:同一助记词恢复同一地址 + 同一 PQC 密钥(确定性派生)。

## 8. wumin(冷钱包)

- 继续离线签名器;扫码按 `sig_alg`/`auth_mode` 识别:当前签 sr25519,未来签 ML-DSA,首次绑定时**一次扫码确认同时**出 sr25519 bootstrap 签名 + ML-DSA 交易签名。
- 展示仍是账户/收款方/金额/治理动作等用户语义,不展示多算法细节。
- **QR 升级:** `sig_alg: sr25519|ml-dsa-65`、`auth_mode: normal|pqc|bootstrap-pqc`、`key_version`、`payload_hash`、**`chunk_index/chunk_total` 大签名分片**。
- 🔴 **最坏体积按 bootstrap 设计**:首笔 bootstrap 同时带 sr25519(64B)+ML-DSA(~3.3KB) 两签,是最大 payload;ML-DSA 签名 3.3KB 不能假设单张 QR 永远稳定可扫,分片须按它实测。

## 9. SFID / CPMS 边界

- SFID/CPMS **不托管**用户助记词、`AccountSeedV1`、sr25519 私钥、PQC 私钥。
- 只做两件:① 记录的钱包地址仍是原 `AccountId`;② 验钱包签名时按 `sig_alg` 分流(当前 sr25519,未来 ML-DSA)。
- `wallet_sig_alg` 字段**保留并放开枚举**,但**不允许生成新钱包地址**。
- SFID/CPMS **自身系统签名**是否改 ML-DSA = 独立系统密钥升级,**不影响用户账户无痛升级主线**(单独推进:SFID MAIN signer / CPMS ARCHIVE 档案密钥)。

## 10. IM / 传输加密(与账户签名分开做,不阻塞)

- 账户签名 = ML-DSA-65;IM/MLS 机密性 = 未来 X-Wing/ML-KEM-768 混合 KEM;TLS = 评估 rustls X25519MLKEM768。
- **KEM 不能当身份认证**,身份仍绑账户签名或 MLS credential。
- 此线不阻塞钱包签名升级。

## 11. 必须先做的技术 spike(写业务代码前)

1. ML-DSA-65 Rust crate 是否支持 no_std / WASM;runtime WASM 体积增量。
2. 单次 ML-DSA 验签 weight;一个区块最多容多少次 PQC 验签。
3. QR 分片在真实手机摄像头上是否稳定(按 bootstrap 最坏体积测)。
4. `AuthorizeCall`/`TransactionExtension` 顺序能否让 `pqc_dispatch` **在收费前产出 signed origin**。
5. general-transaction txpool 防重放是否可靠;**复用 `frame_system::AccountInfo.nonce` 在 general-tx authorize 阶段是否可原子读/写且与 txpool `provides=(account,nonce)` 一致——不过就回退专用 nonce**。
6. ML-DSA WASM 验签不可接受时再评估 host function;但 host function = 节点二进制边界变化,**不再是纯 setCode**(要 fork `ChinaNation/ss58-2027-fix`)。

## 12. 验收标准

- 老客户端 sr25519 钱包地址不变;新客户端同助记词恢复地址**逐字节一致**。
- runtime 升级后原账户余额/权限/治理身份不变。
- 未绑定账户首次交易 **bootstrap + execute** 成功;已绑定账户后续 ML-DSA 交易成功。
- 已绑定账户普通 sr25519 用户交易按策略被拒。
- 冷钱包离线扫码完成 bootstrap + PQC 签名(含分片)。
- SFID/CPMS 验证同一地址下不同 `sig_alg` 的签名。
- 全仓 PQC 表述以本 ADR 为唯一真源;残留旧路线(单独绑定步 / per-account 状态切换 / 共识造新链)一律按本 ADR 收敛清理;真实运行态验收(非仅编译/单测)。

## 13. 静态加密 + KDF 卫生(随线带做,核验发现)

- 🔴 **node 清算行落盘是假 AES-GCM**:`settlement/keystore.rs:181-220` 实为 XOR(blake2 keystream)+截断 blake2 tag,`derive_key` 仅 100 轮 blake2,函数名/注释说谎,node 无 aes-gcm 依赖 → 换真 `Aes256Gcm`+`Argon2id`+改谎注释(独立可立即修)。
- KDF 收敛:CPMS master/geo_seal(`initialize/mod.rs:50`/`dangan/mod.rs:258` Blake2b 单次)→ HKDF-SHA512;口令场景(App 锁 PBKDF2 wumin 1M/wuminapp 100K 漂移)→ Argon2id。
- 对称统一 AES-256-GCM(IM 从 AES-128 升级)。
- `gmb-pqc` 共享 crate(runtime no_std WASM + sfid/cpms/node + 钱包 FFI 共用):HKDF 派生规则 + algo 常量 + domain 常量(**强制 `[u8;N]`**,修 `batch_item.rs:39/42` 的 `&[u8]`)+ `verify_by_algo` trait。

## 14. 任务卡

见 `memory/08-tasks/open/20260618-pqc-card*`(按"地址稳定签名升级"重拆):card0 卫生/card1 gmb-pqc+spike/card2 链端 account-keys+pqc_dispatch+bootstrap/card3 钱包派生+bootstrap+QR分片/card4 SFID 分流/card5 CPMS 分流/card6 IM-TLS 机密性。
