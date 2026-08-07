# 任务卡：正式创世（2026-08-07）

任务需求：
执行 CitizenChain 正式创世：`spec_version` 归零、WASM CI 冻结 runtime、bake 正式 chainspec 并同步五处 SSOT、三端 CI 产出安装包供 44 节点部署。

所属模块：citizenchain（runtime / node / scripts）、citizenapp（创世资产）、CI

必须遵守：
- chainspec 创世后永久冻结，不得再改
- 创世 WASM 只能来自 GitHub WASM CI artifact，禁止本地编译产物
- 节点重部署前必须清除旧链数据库，否则创世哈希不匹配拒绝同步

执行记录：
1. `spec_version` 1 → 0，配套 `runtime_version_and_block_types_are_sane` 精确断言同步（提交 `44b5961a9`）
2. WASM CI 原有 `spec_version 必须大于 0` 守卫按创世场景移除——该守卫按 runtime 升级写死，与从 0 起版的正式创世冲突（提交 `a00f19f09`）
3. WASM CI run `31157796731` 成功，artifact `citizenchain-wasm` id `8986853331`，zip `sha256:2d89a519…`（下载后本地重算一致，非本地编译）
4. `bake-chainspec.sh --finalize` 完成：首启物化 131s，公权机构 43 省 49,593 个；五处 SSOT 同步并提交（`937889e86`）
5. 创世锚点回写 `memory/05-modules/citizenchain/node/NODE_TECHNICAL.md`
6. `check-ai-guardrails.sh` 文档门禁豁免补入创世冻结产物：`node/chainspecs/*`、`citizenapp/assets/chainspec.json`、`light_sync_state.json`、`public_institutions/*`。
   这四项是 `bake-chainspec.sh` 机器生成的冻结产物，与既有豁免的 `dist/`、`target/`、`build/` 同类；
   此前每次创世提交纯生成物都会被误判为「代码变更缺文档」，是该门禁反复失败的根因

创世锚点（当前唯一正式基线）：
- `spec_version=0`，runtime 源提交 `a00f19f09c91b6f22a2b01cc1a48bb82b76ca4eb`，冻结资产提交 `937889e86`
- `genesis_hash=0x18847a5dfd263272f2e7727836fe6582f8c4463ff48609df7b96d5e4d9dd24dd`
- `state_root=0x1f74a2ca094fc3ebb2143f504d807a6b4f4f9b0a3d13ac808ae84efc7cb12111`
- `runtime_wasm_hash=3d17b089dd8051be3aee9cd53d50fe7a67f81ba3c9027c37cc066e5836f60257`
- `chainspec_hash=312a64a1b4b8abda6b688a8d127c14abe4ef410c74623fc524f9b710b1e234ab`
- `light_sync_state_hash=014802836a0f6e01a9f1bf7173b8e04c9df8fc3f057565f855abdccdc7361ab6`
- `public_institution_root=c21f99f5bd40bc3c9fcee9439de9f6902c98212b2510dd7440c9630284ab939f`

待办：
- 三端 CI（mode=ci）产出安装包
- 44 节点清库后按新创世重部署
- Cloudflare 按新创世重部署

验收标准：
- 三端 CI 全绿并产出可部署安装包
- 44 节点创世哈希与上述锚点一致
- 全网从块 0 正常出块

## 三端 CI 修复记录（2026-08-07）

创世资产推送后三端 CI 全部失败，逐轮定位并修复：

1. **文档门禁两级规则**：先查"有无文档更新"，再查"每个改动文件有无对应模块回写"
   （`.github/workflows/*` → `memory/07-ai/`、`citizenapp/**` → `memory/05-modules/citizenapp/`、
   `citizenchain/**` → `memory/05-modules/citizenchain/`）；改守卫脚本等真实开发变更还要求任务卡。
   创世冻结产物（chainspec、light_sync_state、公权机构缓存）已加入豁免——它们是 bake 生成物，
   与 `dist/`、`target/` 同类，此前每次创世都被误判为"代码变更缺文档"。
2. **Rust 格式**：`cargo fmt --all` 必须全 workspace 跑；只格式化本次改动会把既有偏差留到下次。
3. **clippy `-D warnings`**：`qr-protocol/tests/golden_fixtures.rs` 缺
   `#![allow(clippy::expect_used, clippy::unwrap_used)]`（`repo_guard.rs` 一直有）。
   本地不带 `-D warnings` 跑不暴露，必须用 CI 同款命令验证。
4. **CitizenWallet 宿主原生库**：`native_sr25519.dart` 的 host 路径硬编码 `.dylib`，
   Linux CI 产出 `.so` 永远找不到；且 CI 只编 Android ARM、没编宿主库。
   已改为按平台取扩展名，`build-signer-native.sh` 的 `macos` 目标泛化为 `host`，
   workflow 在 `flutter test` 前编宿主库。金标派生测试真跑不 skip。
5. **CitizenApp native skip 守卫**：`test/wallet/` 下 4 个 group + 金标测试全部用例接入
   `smoldotNativeSkipReason()`。验证用「移走 libsmoldot 跑一遍 + 放回去跑一遍」双环境确认。
6. **Worker 类型定义**：`wrangler.toml` 被 bake 改动后 `worker-configuration.d.ts` 过期，
   需 `npm run generate:types` 重新生成。

**验证纪律**：推 CI 前必须用与 CI 完全相同的命令本地跑通
（`BASE_REF=HEAD^ ./.github/scripts/check-ai-guardrails.sh`、
`cargo clippy --workspace --all-targets --locked -- -D warnings`、
`cargo fmt --all -- --check`、各端 `flutter test` 全量、`npm run types:check`），
禁止拿 CI 当试错场。

7. **CitizenApp iOS workspace 未入库**：`citizenapp/.gitignore` 整目录忽略了
   `ios/Runner.xcworkspace/`，CI 检出后无 workspace，`flutter build ios` 报
   `Xcode workspace not found`。CitizenWallet 未忽略该目录，所以它的 iOS job 一直是绿的。
   已改为只忽略 `xcuserdata/`，并补入 `contents.xcworkspacedata` 与
   `xcshareddata/{IDEWorkspaceChecks.plist,WorkspaceSettings.xcsettings}` 三个文件，
   与 CitizenWallet 结构对齐。

## 节点与公民云部署（2026-08-07，完成）

8. **三台节点清库重部署完成**，创世哈希全部为 `0x18847a5dfd263272…`，互联 peers=3、
   isSyncing=false、每台单进程单服务。矿工账户（keystore `powr` 公钥推导，SS58 前缀 2027）：
   - 国储会 `w5HCL4SgSNbCHxFMj3DFP9mryciFRin7DehYs99B3zg1Juknp`
   - 中枢省 `w5DAjQckXraS6XUGyW1Uwz3YkLg8YVg1wxqCX48qjbB1HUBpz`
   - 贵州省 `w5GshUYaTx3x5LnyG427b4uUKr1yCfm7mXAtV7Fsb98LjCRaM`
9. **旧部署残留清除**：三台各有一套按域名命名的旧服务与数据目录
   （`guo-node`+`/opt/guo` 1.2G、`prczss-node`+`/opt/prczss`、`prcgzs-node`+`/opt/prcgzs`），
   与 deploy 脚本认的 `citizenchain-node` 并存，旧进程抢占 9944 导致验收读到旧链应答。
   已全部停用、禁用、删单元文件并删数据目录。**cloudflared 与 nginx 是独立服务，未受影响**。
10. **deploy 脚本两处修复**（`citizenconsole/` 不入库，仅记录于此）：
    六个 `frozen_*` 冻结值更新到新创世（原值仍指旧链，会在校验处直接拒绝部署）；
    新版节点首启要在 base-path 下自建 TLS 目录 `n/`，而脚本只把子目录设为 `citizenchain`
    属主、base-path 仍是 root，导致 `创建 TLS 目录失败: Permission denied` 重启 37 次，
    已补 base-path 与 chains 两级属主。
11. **公民云按新创世重建**：`cloudflare.sh` 两个受审哈希更新（wrangler.toml 被 bake 改了
    链身份两行、schema 改了 `creatorPaused→issuerPaused`）；D1 删表重建（26 张业务表，
    另删一张 schema 已移除、代码零引用的残留表 `cid_data_roots`）；Worker 重新部署
    （version `5710750d`）后 `/api/chain/bootstrap` 实测返回新 genesis_hash 与 state_root。
    **vars 属于 wrangler.toml 而非 Secret，只重建 D1 不会更新它们，必须重新 deploy Worker**。
12. **旧链交易归档**：清库前导出全链交易，见
    `memory/05-modules/citizenchain/node/OLD_CHAIN_TRANSACTIONS_ARCHIVE.md`
    （37 笔转账、合计 19,900,023,035.25 元，含备注原文）。旧链数据已清，此为唯一留存。
