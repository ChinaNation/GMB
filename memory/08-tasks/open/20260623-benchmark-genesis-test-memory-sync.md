# 任务卡：benchmark 权重回填 + 创世测试 + memory 卡同步

创建：2026-06-23　承接「重新创世条件检查报告」执行项

## 任务需求

用户在确认链端已具备重新创世条件后，下达三项收尾：

1. **跑一遍 benchmark 权重并回填**：用本地源码生成各 pallet 的 `weights.rs` 真实权重并提交。
2. **跑一遍链端创世相关 cargo test**：确认创世余额/发行量断言全绿（闭环上轮报告黄灯「未跑创世单测」）。
3. **处理 3 张滞后的 memory 卡**：把实际已落地、但描述仍写「待实施/阻塞」的卡更新为真实状态。

## 预计修改目录

- `citizenchain/runtime/**/weights.rs`：benchmark.sh 回填（11 个 pallet，基于本机硬件权重）。
- `~/.claude/.../memory/`：更新 3 张 project 卡 + MEMORY.md 索引行。
- `memory/08-tasks/`：本卡执行记录。
- 链端代码与创世数据：**不改**（仅回填 weights）。

## 执行边界

- benchmark 权重基于**本机硬件**，非生产服务器；如需生产权重须在目标机重跑。本轮按用户「跑一遍并回填」执行，结果如实标注。
- benchmark.sh 已显式排除 stub/空 benchmark 模块（organization_manage 部分覆盖 / personal_manage / offchain_transaction / votingengine / internal_vote / joint_vote / onchain_issuance / genesis_pallet）。
- 创世数据、china_*.rs、chainspec 本轮不动。

## 验收标准

- [ ] `scripts/benchmark.sh` 跑通，11 个 pallet weights.rs 回填，`cargo check --workspace` 仍 0 error。
- [ ] 创世 7 个断言测试全绿。
- [ ] 3 张 memory 卡描述与真实代码状态一致。

## 执行记录

- [x] 任务2：创世 cargo test ✅ `cargo test -p citizenchain --lib genesis` **8 passed / 0 failed**（genesis 7 断言 + test_genesis_config_builds；含 NRC/省储行/两和基金 balances.len、总发行量、grandpa 公钥校验）。无需 WASM_FILE。
- [x] 任务3：3 张 memory 卡 + MEMORY.md 索引已更新为真实态（T3/T4 代码全落地、派生域=GMB 已落地、china_zf 空 admin 已补）。
- [x] 任务1：`scripts/benchmark.sh` ✅ **11 个 pallet 全回填**（provincialbank_interest/fullnode_issuance/citizen_issuance/resolution_issuance/cid_system/pow_difficulty/admins_change/resolution_destro/grandpakey_change/multisig_transfer/runtime_upgrade），DATE=2026-06-23，本机 MacBook Pro 硬件权重，11 weights.rs 改动 185/185 行。
- [x] 回填后 `cargo check --workspace` 复验 ✅ **0 error**（11.35s，含 node）。三项验收标准全过。

## 注意

- benchmark 权重基于**本机 MacBook Pro**（`HOSTNAME: rhettdeMacBook-Pro.local`, `CPU:<UNKNOWN>`），**非生产服务器硬件**。上生产前应在目标机重跑 benchmark.sh 覆盖。
- 排除项不变：organization_manage(部分覆盖)/personal_manage/offchain_transaction/votingengine/internal_vote/joint_vote/onchain_issuance(stub)/genesis_pallet(无 extrinsic) 仍用手写 WeightInfo。
