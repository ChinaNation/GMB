# ADR-022:GMB 统一抗量子加密方案

状态:Accepted(2026-06-18),**Supersedes ADR-016**
关联:[[ADR-016]](本 ADR 取代)、[[feedback_no_compatibility]]、[[feedback_chainspec_frozen]]、[[feedback_pubkey_format_rule]]、[[feedback_chain_in_dev]]

## 背景 / 问题

全系统(citizenchain / wuminapp / wumin / sfid / cpms)签名 100% sr25519,机密性(IM / 传输)100% 经典 X25519 + AES-128,均不抗量子。核验确认 **PQC 零代码**(全仓无 fips204 / ml-dsa / ml-kem / account-keys / argon2)。

ADR-016 只设计了"钱包签名域",且带完整 sr25519→PQC **迁移机器**(账户状态机 / hybrid 双签 / PqcOnly 收紧)。但:

1. ADR-016 未覆盖 **机密性域(ML-KEM)**、**静态加密**、**KDF 统一**——而机密性才是 harvest-now-decrypt-later 最不可逆的暴露面。
2. 全系统**无数据、上线前可重新创世**——迁移/兼容机器是多余的。

本 ADR **取代** ADR-016 + 计划中的"系统签名 PQC"任务卡 + 蓝图 `ACCOUNT_KEYS_PQC_TECHNICAL.md`,合并为单一统一方案。

### 核验抓出的两条 CRITICAL(已纠正)

- **C1:账户 PQC 不是透明 setCode。** 主交易签名是上游 `MultiSignature` 枚举(`runtime/src/lib.rs:130/134`),AccountId 由它派生。加 ML-DSA 变体=改 extrinsic 线格式≠setCode。→ 走 `pqc_dispatch` general-tx 旁路,**绝不扩 MultiSignature**。
- **C2:node 清算行落盘是假 AES-GCM。** `settlement/keystore.rs:181-220` 实为 `XOR(blake2 keystream)+截断 blake2 tag`,`derive_key` 只跑 100 轮 blake2(`PBKDF2_ITERATIONS/1000`),函数名和注释在说谎,node 无 aes-gcm 依赖。清算行私钥当前弱加密。

另修正一条 HIGH:**"SFID 省级 wrap HKDF" 代码里不存在**(`sfid/backend/Cargo.toml:42` hkdf 是死依赖,MAIN signer 是裸 seed 无 KDF);记忆 `feedback_sfid_sheng_signing_keyring` 已过时(级联重加密在 ADR-008 P23e 删除)。

## 决策

### 1. 统一四原语(全系统单一口径,收口 `gmb-pqc` 共享 crate)

| 用途 | 算法 | 标准 |
|---|---|---|
| 签名 | **ML-DSA-65** | FIPS 204,algo tag `0x02`,可升 `0x03`=ML-DSA-87 不换账户 |
| 机密性(KEM) | **ML-KEM-768**(混合) | FIPS 203;IM 用 X-Wing(X25519+ML-KEM-768),TLS 用 X25519MLKEM768 |
| 对称 AEAD | **AES-256-GCM** | IM 从 AES-128 升级;node 清算行从假 GCM 换真 GCM |
| KDF | **HKDF-SHA512**(高熵根种子) + **Argon2id**(用户口令) | 收敛三处异构 KDF |

### 2. 同源派生 + 地址锚点(option B)

```
助记词(BIP39) → miniSecretFromEntropy → AccountSeedV1(32B 算法中立根种子, 绝密, 不出本机)
  ├─ HKDF("GMB/sr25519/v1")    → sr25519 私钥 →(取公钥)→ AccountId(32B 永久地址锚点; sr25519 永不签名)
  ├─ HKDF("GMB/ML-DSA-65/v1")  → ML-DSA-65 私钥(签名)
  └─ HKDF("GMB/ML-KEM-768/v1") → ML-KEM-768 私钥(加密)
```

- **四不变**:不换助记词 / 账户 / 地址 / 余额。
- **option B**:AccountId = sr25519 公钥(沿用现派生),sr25519 退役为**纯地址锚点**,签名全 ML-DSA-65。地址永不变(同助记词跨链同地址)。
- **密钥隔离**:矿工 / 清算行 / SFID MAIN / CPMS-ARCHIVE 为独立密钥域(各自 OsRng / env seed,不走 AccountSeedV1),只共用算法不共用密钥。

### 3. 无迁移 / 无兼容 / 只用最新(因全系统无数据)

- **删除全部迁移机器**:账户状态机 `Sr25519Only→Bound→PqcOnly`、hybrid 双签 `bind_pqc_key`、`RejectSr25519WhenPqcOnly`、所有 storage 迁移、所有 re-wrap。
- 账户**出生即 PQC-native**;链上类型创世直接定义目标态(如 `ShengSigningPubkey` 直接 `BoundedVec`)。
- **始终一条链**:GMB 只有一条链。"重新创世"= 刷新**本链**创世(无数据可丢),**不是造新链、不是迁移**。伊犁省改名先在 sr25519 创世(PQC 未 built);PQC 就绪后,**同一条链**的下次创世/二进制直接带 ML-DSA-65,零迁移代码。

### 4. 账户 PQC 走 `pqc_dispatch`,不碰 `MultiSignature`(C1)

PQC 交易 = general-transaction(无外层 sr25519 签名)→ `account-keys.pqc_dispatch` → `#[pallet::authorize]` 阶段用 ML-DSA-65 验签(公钥从 `BoundPqcKey` 按 AccountId 读)→ 以 sr25519 锚点 AccountId 派发内层 call。全在 WASM 内。

### 5. seal 共识签名 ML-DSA 跟 PQC 二进制走,本链创世落地(无硬分叉)

seal 是原生二进制逻辑。本链处于开发期、无数据、本就随更新重刷创世+全网重装二进制,故 seal=ML-DSA-65 **免硬分叉**——跟 PQC 二进制一起,在**本链**带 PQC 的那次创世落地。`blake2_256` PoW 工作量哈希抗量子,不动。

### 6. 静态加密真伪修复(C2)

node 清算行 `keystore.rs:181-220` 换真 `Aes256Gcm` + `Argon2id`,改掉说谎的函数名 / 注释。可立即独立先行。

## 影响

- **删除**:`ADR-016`(转 Superseded 墓碑)、任务卡 `20260607-wallet-pqc-passkey` / `20260607-sfid-cpms-pqc-signing`、蓝图 `ACCOUNT_KEYS_PQC_TECHNICAL.md`。
- **新增**:`account-keys` pallet(idx=27),`spec_version` bump(本链带 PQC 的创世起);`gmb-pqc` 共享 crate(runtime no_std WASM + sfid/cpms/node 后端 + 钱包 FFI 共用)。
- **钱包**:Dart 侧 ML-DSA/ML-KEM 经 `gmb-pqc` FFI(无成熟纯 Dart 实现)。
- **IM**:换 libcrux/X-Wing-capable provider(openmls_rust_crypto 对 X-Wing `unimplemented!()`)。
- 不影响真实 `ADR-017`(finalized-unification,与 PQC 无关)。

## 备选方案(否决)

- **迁移模型(ADR-016 状态机)**:无数据下多余,否决。
- **扩 MultiSignature 加 ML-DSA 变体**:改 extrinsic 线格式,否决(走 pqc_dispatch)。
- **纯种子派生锚点(option A,彻底删 sr25519)**:更"删旧"但需重写钱包派生且 AccountId 不对应任何公钥,本期取 **option B**(sr25519 锚点)保地址工具兼容、可后续再评估。
- **Falcon / SLH-DSA**:侧信道 / 体积理由同 ADR-016。

## 后续动作

7 张任务卡(见 `memory/08-tasks/open/20260618-pqc-card0..6`):

| 卡 | 范围 | 域 |
|---|---|---|
| card0 | 卫生修复(清算行真 AES-GCM+Argon2id / App 锁 KDF 统一 / 热钱包 at-rest / IM AES-256) | Blockchain+Mobile |
| card1 | `gmb-pqc` 共享 crate + fips204 WASM spike + domain 常量 `[u8;N]` | Blockchain |
| card2 | account-keys pallet + 5 验签器 algo-tag + offchain L3/批量 + seal ML-DSA | Blockchain |
| card3 | 钱包 AccountSeedV1 HKDF 三分叉 + QR sig_alg + pqc_dispatch 构造 | Mobile |
| card4 | SFID MAIN signer ML-DSA-65 + KDF 收敛 + 激活死 hkdf 依赖 | SFID |
| card5 | CPMS ARCHIVE 签名 ML-DSA-65 + master/geo_seal KDF→HKDF + wallet_sig_alg 放开 | CPMS |
| card6 | 机密性:IM libcrux X-Wing(ML-KEM-768) + TLS X25519MLKEM768 | Blockchain+Mobile |

**先行 spike**:fips204 no_std WASM 编译 + 体积 + 权重 → 决定 WASM 内验签 vs host function(后者要 fork polkadot-sdk,非纯 setCode)。
