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
