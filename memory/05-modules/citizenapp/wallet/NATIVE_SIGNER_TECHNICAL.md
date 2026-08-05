# CitizenApp 原生 sr25519 签名（schnorrkel FFI）技术说明

> 全仓 sr25519 **唯一实现**。纯 Dart `package:sr25519` 已彻底移除，禁止再引入第二套。

## 1. 为什么必须原生

纯 Dart `sr25519` 走 `BigInt` 软算标量乘，真机（OnePlus 6T）实测一次「派生 + 签名」
**8220 ms**：用户按完指纹要干等 8 秒，且该 CPU 曾留在 UI isolate 上，把主线程一次卡死
超过 Android ANR 阈值（5 s），发送交易必崩。

换成 schnorrkel（Substrate 全家官方实现）后，同一台设备实测 **14~18 ms**，约 500 倍。

| 阶段 | 纯 Dart | 原生 schnorrkel |
|------|---------|-----------------|
| 生物识别（交互，不可省） | ~2.3 s | ~2.3 s |
| **派生 + 签名** | **8220 ms** | **14~18 ms** |

原生库无需新增：`smoldotpow/lib` 早已依赖 `schnorrkel 0.11.2`（其源码注释还留着
`TODO: necessary for signing`），本次只是把它暴露给 Dart，**不新增 `.so`**。

## 2. 口径（错一处，同一助记词会派生出另一个账户）

| 项 | 取值 | 对齐对象 |
|----|------|----------|
| 扩展模式 | `ExpansionMode::Ed25519` | Dart `MiniSecretKey.expandEd25519()` / Substrate `Pair::from_seed` |
| 签名上下文 | `b"substrate"` | Dart `Sr25519.sign` 的 `newSigningContext` |
| 硬派生 chaincode | **由 Dart 侧算好传入** | `SecretUri.fromStr('//index').junctions` |

junction 解析属 SCALE 编码而非密码学、不慢，且已被金标测试守着；留在 Dart 可把
**口径漂移面压到最小**——原生只接管慢的密码学部分。

## 3. 接口

`rust/src/signer.rs` 导出 4 个 C 函数，`lib/wallet/core/native_sr25519.dart` 一一绑定
（复用现成的 `SmoldotPlatform.loadLibrary()`）：

| 函数 | 语义 |
|------|------|
| `gmb_sr25519_derive_hard(seed32, cc32) → child32` | 一层硬派生；多层由调用方按 junction 顺序逐层调用 |
| `gmb_sr25519_public_key(child32) → public32` | child mini-secret → 公钥（= AccountId32） |
| `gmb_sr25519_sign(child32, msg) → sig64` | 签名 |
| `gmb_sr25519_verify(public32, sig64, msg) → i32` | 验签 |

## 4. 安全

- **私钥材料**：Rust 侧一律 `Zeroizing`（作用域结束即擦除；禁止手写清零，会被优化掉）；
  Dart 侧出入参缓冲在 `finally` 里**先清零再释放**。
- **绝不 panic 跨边界**：4 个入口全部 `catch_unwind`，内部异常只回错误码
  （`GMB_SIGNER_ERR_*`）——原生 panic 会 `abort` 掉整个 App，代价与收益完全不成比例。
- **fail-closed**：空指针、长度非法一律拒绝；Dart 侧错误码一律上抛，
  **绝不静默兜底成"空签名"或"验签通过"**。
- **不改密钥来源**：仍从硬件金库 `readAccountKey` 取（生物识别、`biometricOnly` 不变）。
- **不需要 Isolate**：原生毫秒级，直接在调用线程完成；纯 Dart 时代为躲 ANR 而加的
  `Isolate.run` 已移除（换原生后它只剩开销）。

## 5. 验收（切换实现的安全闸门）

1. **金标测试** `test/wallet/derivation_golden_test.dart`：原生派生 + 原生公钥
   逐字节对拍 **Substrate 官方权威向量**（硬编码 `//Alice`、`//0 //1 //2` 的
   accountId / ss58 / childMiniSecret）——这是"生产实现 vs 外部真值"的直接对拍，
   比"两份 Dart 实现互证"更硬；
2. Rust 单测：派生确定性、签名→验签闭环、篡改必须验不过、空指针必拒；
3. 真机：交易正常签发并最终性上链，账户无漂移，`[Sign-Diag]` 显示 14~18 ms。

> 切换期间曾有一份 `native_sr25519_parity_test.dart` 做原生 ↔ 纯 Dart 对拍；原生真机
> 验收通过、纯 Dart 移除后该文件一并删除（不保留两套实现）。

## 6. 现存边界

- `polkadart_keyring` 仍保留，**仅用于 SS58 地址编解码**（base58 + 校验和，非密码学），
  全 app 统一走它；`sr25519` 因此仍是它的传递依赖，但 `pubspec.yaml` 已移除直接依赖，
  **禁止再直接 import**。
- **CitizenWallet（冷钱包）尚未原生化**：仍是纯 Dart `sr25519`，且该项目无原生层。
  第 2 步计划复用本文的同一份 Rust 实现，统一冷热两端。
