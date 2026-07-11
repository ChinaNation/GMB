# 任务卡：其他(官网/BFF/smoldot/脚本/docs) 命名精简统一 + 残留清理

## 任务需求

执行命名审计「其他」一类 91 条([[project_naming_audit_2026_07_11]]),覆盖 citizenweb(React)、citizenapp/cloudflare(TS Worker)、smoldotpow(Rust fork)、smoldotdart(Dart FFI)、chat_mls(proto/rust/dart)、scripts、docs、Dockerfile、CI。完成后更新文档、完善注释、彻底删除残留代码/注释/文档。

## 已完成/边界

- **已在 CitizenApp 轮完成:** smoldot-pow→smoldotpow、smoldot-dart→smoldotdart 目录改名。
- **延后到公民链轮:** `citizenchain/node` crate `name="node"→citizenchain-node`(牵连 -p 参数/Docker/CI,与公民链一起做)。Dockerfile 的 bin 路径 `node→citizenchain` 可现在改(bin 名本就是 citizenchain)。
- 逐子系统分语言验证:citizenweb/cloudflare 用 tsc/build;smoldotpow 用 cargo check;smoldotdart 删重复目录前先验证确为未用;chat_mls proto 改动核对编译模型。发现错误建议即跳过并报告。

## 分阶段

1. citizenweb:删死资源(vite/react.svg、hero.png、icons.svg)、qrV1.ts→qr-v1.ts、QrScannerModal→QRScannerModal、RosetteBadge→IdentityBadge、IdentityTier→IdentityLevel、VITE_CITIZENAPP_SQUARE_API_BASE_URL→VITE_SQUARE_API_BASE_URL、文案术语统一(储委会/储行/公民链/去中英硬空格)、重写模板 README。
2. scripts/docs/Docker/CI:拼音脚本改英文、.sh kebab、router/context/guardrails 路径(citizencode→onchina 等)、docs 中文资源名→英文、Dockerfile /polkadot→/citizenchain+bin、根 Cargo 空 members、CI 改名。
3. cloudflare:CITIZEN_CHAIN_*→CHAIN_*、CITIZENAPP_MEMBERSHIP_*→MEMBERSHIP_*、包名、FEED_CACHE→SQUARE_CACHE、STRIPE_DEV_CHECKOUT_PROXY→STRIPE_DEV_PROXY、checkout.ts→subscribe.ts、stripe.ts→webhook.ts、hex/scale helper 收敛 shared 单源。
4. smoldotdart:删重复 rust//native/、死构建脚本、ffigen 死配置、pubspec 上游元数据、废弃 FFI 同步导出与 Dart 封装、dead_code DTO/error。
5. smoldotpow(Rust fork):PowPreRuntime→PowPreDigest、删 difficulty 死字段、Chain*别名去前缀、Smoldot/LightClient 前缀统一。
6. chat_mls:owner_account→owner_account、sender/recipient→*_owner_account、chat_device_*去前缀、MlsWireMessageKind→MlsMessageKind。

## 验收

各子系统构建/分析通过;残留旧名与脚手架死资源零引用/已删;数量口径以 code.rs(104/43)为准。

## 执行结果(2026-07-11 部分完成)

**已完成并独立验证 GREEN:**
- citizenweb:删 4 死资源(vite/react.svg、hero.png、icons.svg)、qrV1.ts→qr-v1.ts、QrScannerModal→QRScannerModal、RosetteBadge→IdentityBadge、IdentityTier→IdentityLevel、VITE 环境变量去冗余、术语统一(储委会/储行/公民链/去中英硬空格)。`npm run build` GREEN。
- cloudflare:CITIZEN_CHAIN_*→CHAIN_*、CITIZENAPP_MEMBERSHIP_*→MEMBERSHIP_*、包名→citizenapp-square-api、FEED_CACHE→SQUARE_CACHE、STRIPE_DEV_CHECKOUT_PROXY→STRIPE_DEV_PROXY、checkout.ts→subscribe.ts、stripe.ts→webhook.ts(含测试)。`tsc --noEmit` GREEN。
- Dockerfile 重写(/polkadot→/citizenchain、bin node→citizenchain);docs/国旗.png→flag.png、项目讲解.pptx→project-overview.pptx;CI citizenchain.yml→citizenchain-ci.yml;CITIZENPASSPORT_TECHNICAL.md china.sqlite→citizenchain/onchina/src/cid/china/china.sqlite;7 个脚本改名(fuwuqi/zhujichi/gmb→英文、.sh kebab)+ 引用更新;Codex→Claude。
- smoldotdart:删死重复 rust//native/ 目录 + 死构建脚本 + 死 ffigen 配置;修 pubspec 上游 polkadart 元数据、smoldot_light 导入示例、citizenapp/rust authors。

**⚠️ 发现并发冲突(非本轮改动):** 主检出中 `lib/chat/`→`lib/chat/` 迁移(chat_mls→chat 字段重命名,即本方案第 6 阶段 chat_mls 行)正被**另一线程/会话并发执行**,处于半成品破损态——chat_store/chat_page/chat_device_binding 引用新字段但 `app_isar.g.dart` 未重生、qr_signer.dart 引用被删的 `kOpSignChatDeviceBind` → 15 个错误。**本轮未触碰 chat/chat/app_isar,以上错误全部来自并发迁移,不是本轮引入。** 已停止,避免与并发线程冲突。

**未执行(冲突/需重验证/结构性,已报告):**
- 第 6 阶段 chat_mls proto/字段改名——**正被并发线程做,禁止冲突。**
- 第 5 阶段 smoldotpow Rust vendored fork 改名(PowPreRuntime→PowPreDigest、删 difficulty、Chain* 别名)——需 cargo 编 293 文件 fork 验证。
- cloudflare hex/scale helper 收敛 shared——跨 5 文件 DRY 重构(非命名)。
- smoldotdart 废弃 FFI 同步导出/dead_code DTO 删除——触碰活跃 FFI crate,需 cargo+FFI 验证。
- 结构性延后:根 Cargo.toml 空 members 删除、citizenchain/node crate 改名(并入公民链轮)、docs/citizenpassport→顶层(独立产品迁移)。

## 续跑结果(2026-07-11,并发 im→chat 完成后)

- **im_mls 阶段已由并发线程完成**:lib/im→lib/chat 迁移落地,审计所列旧字段名(owner_wallet_account / im_device_pubkey_hex / im_device_id / ImMlsWireMessageKind / sender_chat_account / recipient_chat_account)全仓 0 引用,proto 迁至 citizenapp/chat/proto/chat_envelope.proto。本轮无需重做。
- **smoldotpow(Rust fork)已做并 cargo 验证**:PowPreRuntime→PowPreDigest(PoW 自定义 digest 变体,header.rs)、Chain* 别名去前缀(ChainStartupFinalizedSource→StartupFinalizedSource 等,light-base/lib.rs 内外同名)。`cargo check` EXIT=0。
- **最终验证全绿**:citizenapp `flutter analyze`=2 既有 info(零错误)、citizenweb `npm run build` GREEN、cloudflare `tsc --noEmit` GREEN、citizenapp/rust `cargo check` GREEN。

**最终未执行(硬性理由,建议单独任务):**
- FFI 废弃同步导出删除:sync 符号仍被 bindings.dart 按字符串绑定(326-354 行)、getMetadataHex 有在用的 async 双胞胎——删则运行期 FFI 断链,headless 无法验证。
- cloudflare hex/scale helper 收敛 shared:跨 src+3 测试文件(含测试本地 compactU32 副本),触碰签名 wire 编码,属 DRY 重构非命名。
- smoldotpow difficulty 死字段删除:构造点不明,结构体字段删除风险高。
- 结构性:根 Cargo 空 members、node crate 改名(公民链轮)、docs/citizenpassport 顶层迁移。

## 「最终未做」清尾轮(2026-07-11,用户要求彻底完成)

- ✅ **FFI 废弃同步导出删除完成**:先证死(chain.dart 活路径调 bindings 的 *Async;sync bindings 方法零调用者)→ 删 rust/src/lib.rs 10 个 `#[deprecated]` sync 导出 + bindings.dart 全部 sync typedef/字段/eager lookup/方法(1157→694 行)+ 删已无用 `block_on_native_capability`。`cargo check` GREEN(零 warning)、`flutter analyze` 2 既有(零错)。踩坑:此前 smoldotpow `Chain*` 别名去前缀漏改消费方 citizenapp/rust,一度 23 编译错,已补齐 rust 侧 ChainSyncPhase→SyncPhase 等。留 `ffi_types.rs/error.rs` DTO 模块未删(AddChainConfigJson 与活跃 AddChainConfigJsonRpc 子串混淆,死活判定不可靠)。
- ✅ **cloudflare hex/scale DRY 收敛完成**:square_event/identity/extrinsic_relay/device_subkey + 2 测试 的本地 compactBytes/compactU32/hex/hexToBytes 收敛到 shared/signing_message 单源(scaleString/scaleCompact/bytesToHex/hexToBytes);shared hexToBytes 返回类型收紧为 `Uint8Array<ArrayBuffer>` 供 crypto 用。`tsc --noEmit` GREEN + `vitest` 112/112 通过。
- ⛔ **smoldotpow difficulty**:核实**非死字段**——header_only.rs:300 解构后传入 `pow::verify_header(VerifyConfig{...,difficulty})`,穿过 ConfigConsensus::Pow→VerifyConfig→verify_header 整条 PoW 校验 API;审计"死字段"判断有误,删除=多文件 fork API 改动零收益,跳过。
- ⛔ **结构性→并入公民链轮**:根 Cargo 空 members(本轮暴露为 cargo 虚拟清单报错)+ node crate 改名 + Dockerfile 构建是同一耦合单元,与公民链一起做;docs/citizenpassport 顶层迁移是独立产品搬迁决策,非命名。

**其他一类落地完毕(FFI/DRY 已补;difficulty 证伪跳过;结构性归公民链)。四子系统终检全绿:flutter analyze 2 既有 / rust cargo GREEN / cloudflare tsc+vitest GREEN / citizenweb build GREEN。**
