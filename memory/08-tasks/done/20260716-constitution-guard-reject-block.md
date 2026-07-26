# 宪法守卫非法区块拒绝与旧 Runtime 保持

## 任务需求

- 启动时继续检查链上 block#0 宪法基准是否合法。
- 运行期间只把篡改不可修改条款、manifest、历史版本或修宪凭据的区块判为非法。
- 非法区块统一返回 `KnownBad`，不得进入内层导入器、数据库或最佳链。
- 非法 runtime 升级不得改变父区块状态和旧 runtime，也不得终止正在运行的节点。
- 第一步只处理 ConstitutionGuard；NodeGuard、PoW、CID、费率留到第二步。

## 输入文档

- `memory/00-vision/trust-boundary.md`
- `memory/03-security/security-rules.md`
- `memory/04-decisions/ADR-027-legislation-yuan.md`
- `memory/05-modules/citizenchain/node/constitution-guard/CONSTITUTION_GUARD_TECHNICAL.md`
- `memory/07-ai/workflow.md`
- `memory/07-ai/definition-of-done.md`

## 修改边界

- 修改 `citizenchain/node/src/core/constitution/` 的原生宪法判定与导入编排。
- 修改 `citizenchain/node/src/core/service/p2p_bad_block_tests.rs`，只增加真实 client/backend 的拒块回归测试。
- 只在必要时修改 `citizenchain/node/src/core/service.rs` 的网络/挖矿装配注释或错误传播。
- 不修改 `citizenchain/runtime/`。
- 不修改 NodeGuard、PoW、CID 或费率逻辑。
- 不新增 ConstitutionGuard 平行导入器或兼容分支。

## 验收标准

- block#0 链上宪法基准仍被完整检查。
- 四类非法宪法变化全部返回 `KnownBad`，内层导入次数为零。
- 连续拒绝非法区块后，合法区块仍可导入。
- 网络导入和本地挖矿使用同一 ConstitutionGuard。
- 非法升级后最佳块与 runtime 保持父状态，节点进程和 RPC 继续运行。
- Rust 测试、编译、格式检查及真实双节点验收通过。
- 文档、中文注释和旧错误口径完成收口。

## 当前状态

- 已完成，待统一归档。

## 执行结果

- ConstitutionGuard 已收口为 `Ok` 合法 / `Err` 非法二态；非法统一由导入闸门返回 `KnownBad`。
- 父状态中已经存在的历史版本和两类修宪凭据完成逐字冻结，禁止修改、删除及事后补写。
- 不可修改条款、manifest、历史版本、修宪凭据四类拒绝后，内层导入次数均为零；后续合法块仍可导入。
- 启动期 block#0 宪法基准检查、网络/挖矿包装顺序均保持不变。
- 未修改 `citizenchain/runtime/`，未修改 NodeGuard、PoW、CID 或费率规则。

## 验收记录

- 带当前源码 WASM 执行 `cargo test --manifest-path citizenchain/node/Cargo.toml constitution --no-fail-fast`：44/44；
  其中服务级测试实际构造 client/backend，拒绝 manifest 恶意 delta 后继续接受合法 delta。
- `cargo test --manifest-path citizenchain/node/Cargo.toml node_guard --no-fail-fast`：74/74，含真实双节点 P2P 拒块。
- node 定向格式检查和 `cargo check`：通过。
- 当前源码 WASM fresh 节点真实启动成功；block#0 为
  `0x8347f61bd28c93c4ce6d6b98f4b5a70f185841e0ac87b0bab9eb8c6caf8375ed`，RPC 健康且宪法正文可读。
- 验收节点正常退出，验收临时目录和误启动进程均已清理；未新增兼容分支、临时源码或 runtime diff。
