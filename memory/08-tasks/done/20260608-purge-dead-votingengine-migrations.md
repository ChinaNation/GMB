# 任务卡:清除 votingengine 三个死 migration + 全量残留清理

## 任务需求

预上线全新创世(`fuwuqi.sh q`),`runtime/src/lib.rs` 的 `Migrations` 元组仍挂着上一阶段 sub-pallet 拆分/双层 ID 的 3 个 `OnRuntimeUpgrade`,对新链是死代码。按"预上线不留任何旧残留"铁律全部清除,并连带清理一切引用、benchmark、测试、自相矛盾的 stale 注释;清完复查 runtime 无残留、无遗漏、无漏洞。

所属模块:citizenchain/runtime(Blockchain Agent)

## 必须遵守

- 不动 `spec_version`(已归零=1)与 STORAGE_VERSION(布局语义元数据,非 migration 残留)
- 不动 `genesis/src/lib.rs`(其 OnRuntimeUpgrade 注释指另一个 genesis pallet,无关)
- 不改链上行为契约;全新创世下本就是空转,删除后等价
- 不搞兼容/保留(feedback_no_compatibility)

## 清除清单

删除文件(6):
- internal-vote/src/migrations/{v1.rs,mod.rs}
- joint-vote/src/migrations/{v1.rs,mod.rs}
- votingengine/src/migrations/{v1.rs,mod.rs}

删除/收敛(benchmark + 死测试):
- internal-vote/src/benchmarks.rs 收敛为空骨架(原仅 migration benchmark),对齐 votingengine 约定
- joint-vote/src/benchmarks.rs 同上
- joint-vote/src/tests/ 整目录删除(仅为 migration 测试提供 mock,无自有 #[test]);移除 lib.rs `mod tests;`
- internal-vote/src/tests/dual_id.rs 删 `migration_v1_backfills_*` 单个测试(保留其余 5 个)

移除模块声明:
- internal-vote/lib.rs `pub mod migrations;`
- joint-vote/lib.rs `pub mod migrations;` + `mod tests;`
- votingengine/lib.rs `pub mod migrations;`

runtime 接线:
- runtime/src/lib.rs `Migrations` 元组 → `()`,更新注释
- runtime/src/benchmarks.rs 注释掉 `[internal_vote, InternalVote]` / `[joint_vote, JointVote]`(无 benchmark fn 后会编译失败),对齐 votingengine 注释

stale 注释/文档修正(自相矛盾的遗漏):
- configs/mod.rs:86-94 "MigrateToV1 未注册/spec_version 0" → 改为全新创世无运行时迁移
- votingengine/src/id.rs:9-16 删 migration 回填叙述,改为创世直写双层 ID
- votingengine/src/lib.rs:182-187 STORAGE_VERSION 注释删 migration 回填引用

## 验收标准

- `cargo check -p internal-vote -p joint-vote -p votingengine` 通过
- `cargo check -p <runtime>` 通过(含 `--features runtime-benchmarks`)
- `cargo test` 三个 sub-pallet 全过
- 全仓 Grep:MigrateV0ToV1 / MigrateToV1 / move_prefix / 死 migration 引用 零残留
- 无新 clippy / 编译告警(死代码、unused import)
- 文档/注释与代码一致,无自相矛盾

## 执行记录(2026-06-08 完成)

改动:8 改 + 7 删(6 migration 文件 + joint-vote 死 tests),净 -909 行。

- [x] 删 3 sub-pallet migrations 目录 + mod 声明
- [x] internal/joint benchmarks.rs 收敛空骨架;runtime benchmarks.rs 注释掉两条注册
- [x] runtime `Migrations` → `()`;configs `SingleBlockMigrations` stale 注释改正
- [x] id.rs / votingengine lib.rs / configs 三处 stale 注释(MigrateToV1 未注册/spec_version 0 与实际矛盾)修正
- [x] internal-vote dual_id 删 1 个 migration 测试 + 清 unused import + rustfmt
- [x] 验证:sub-pallet `cargo check` 0 警;`cargo test` 87 passed 0 failed;runtime `cargo check`(默认 + runtime-benchmarks)exit 0;全仓残留 grep 零;改动文件 rustfmt clean
- [x] 行为中性确认:旧链上这 3 个 migration 在创世已是 noop(守卫 on_chain>=1,创世 StorageVersion 2/2/1),删除后等价,无回归

遗留(非本卡):
- configs/mod.rs 4 处 unused import(Encode/UnfilteredDispatchable/ResolutionIssuance/sr25519_verify/blake2_256)+ genesis.rs:52 rustfmt drift,均预存、与本次无关,按 no-scope-expansion 不动。
- 部署口:`fuwuqi.sh` 走 deb 内嵌的冻结 `node/chainspecs/citizenchain.raw.json`(创世 `:code`=该文件内 WASM,文件日期 5/7)。本次 runtime 改动要真正进创世,需先重生该冻结 chainspec 再出 deb,否则创世跑的是旧 WASM。待 user 决策。
