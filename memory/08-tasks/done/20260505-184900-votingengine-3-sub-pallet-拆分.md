# votingengine 拆 3 sub-pallet(internal-vote / joint-vote / citizen-vote)

- **日期**: 2026-05-05
- **模块**: Blockchain Agent — `citizenchain/runtime/votingengine`
- **优先级**: 高
- **依赖**: PR-X(votingengine 提级 + lib.rs 拆分)、org-manage 重命名、citizen 目录骨架已就位

## 背景

当前 votingengine 是单一 pallet,内部把内部投票/联合投票/公民投票(空骨架)三种模式塞在同一个 `impl<T:Config> Pallet<T>` 下。新增的公民投票要接更多模式(election / referendum / approval / RCV / ...),边界扩张会进一步污染当前已实现稳定的 internal/joint。

参考 `transaction/{duoqian-transfer,institution-asset,onchain-transaction,offchain-transaction}` 的设计风格 —— 每种交易一个独立 pallet —— 把三种投票模式拆为 3 个独立 pallet,边界硬隔离。

## 决策(已与 user 拍板)

1. **拆分形态**:`runtime/votingengine/` 父目录承载 1 引擎核心 + 3 sub-pallet
   - `votingengine/`(Cargo.toml + src/):引擎核心,所有共用基础设施
   - `internal-vote/`:内部投票
   - `joint-vote/`:联合投票(`jointinternal.rs` 管理员阶段 + `jointreferendum.rs` 全民兜底)
   - `citizen-vote/`:公民投票(空骨架,Phase 3 实现细节)
2. **核心 storage 仍在 votingengine**:`Proposals<T>` / 4 反向索引 / `AdminSnapshot` / `InternalProposalMutexes` / `ActiveProposalsByInstitution` / `ProposalIdCounter` 全部留在引擎核心,sub-pallet 通过 trait 接口读写
3. **mode 私有 storage**:各 sub-pallet 自己的 storage(internal_vote::InternalThresholdSnapshot,joint_vote::JointBallotBox 等)
4. **votingengine alias / storage prefix 不变**:链上数据 0 影响
5. **snapshot.rs / mutex.rs 不合并**:职责不同(快照 vs 锁)
6. **chainspec 不重生成**:无 fresh genesis
7. **spec_version**:本 PR 完成全部改造、tests 全绿后由 user 决定升级时机

## 范围内

- 链端 votingengine 重构 + 3 sub-pallet 建立
- 业务 pallet 调用面切换(7 pallet 共 ~11 处)
- citizen-vote 空骨架(无业务逻辑)
- wuminapp / Tauri 客户端 extrinsic 路径同步
- joint-vote 内部 `joint.rs` → `jointinternal.rs`(管理员阶段)
- 历史 extrinsic `votingengine.citizen_vote` 改名 `joint_vote.cast_referendum`(实质是联合公投兜底)

## 范围外

- ❌ citizen-vote 业务模式实现(election / referendum / approval / RCV)→ Phase 3
- ❌ 公权 / 其他机构注册路径 → Phase 2
- ❌ ProposalData enum 结构调整 → 维持现状
- ❌ wumin 公民钱包 decoder 重构 → 客户端 PR

## 7 个 trait 接口(votingengine/src/traits.rs)

```rust
pub trait VotingEngineApi<AccountId, BlockNumber> {
    // ID 服务
    fn next_proposal_id() -> u64;
    fn allocate_display_id(year: u32, kind: ProposalKind) -> ProposalDisplayId;
    // 提案存储
    fn write_proposal(id: u64, proposal: ProposalEnvelope) -> DispatchResult;
    fn read_proposal(id: u64) -> Option<ProposalEnvelope>;
    fn finalize_proposal(id: u64, status: ProposalStatus) -> DispatchResult;
    // 反向索引
    fn register_proposal_indexes(id: u64, owner: ProposalOwner, year: u32) -> DispatchResult;
    // 管理员快照
    fn write_admin_snapshot(id: u64, institution: InstitutionPalletId, admins: Vec<AccountId>) -> DispatchResult;
    fn read_admin_snapshot(id: u64, institution: InstitutionPalletId) -> Option<Vec<AccountId>>;
    fn is_admin_in_snapshot(id: u64, institution: InstitutionPalletId, who: &AccountId) -> bool;
    // 互斥锁
    fn acquire_proposal_mutex(id: u64, org: u8, institution: InstitutionPalletId, kind: MutexKind) -> DispatchResult;
    fn release_proposal_mutexes(id: u64) -> DispatchResult;
    // 活跃上限
    fn try_add_active_proposal(institution: InstitutionPalletId, id: u64) -> DispatchResult;
    // 清理调度
    fn schedule_cleanup(id: u64, at: BlockNumber, callback: CleanupCallback) -> DispatchResult;
}
```

## 实施步骤

参见对话技术方案 10 步。

## 验收清单(实施完成 2026-05-05)

- [x] votingengine 三 sub-crate 建立(internal-vote / joint-vote / citizen-vote)
- [x] joint-vote/src/jointinternal.rs(re-export 原 joint.rs)与 jointreferendum.rs 就位
- [x] internal-vote 实现 InternalVoteEngine trait(委派 votingengine::Pallet::do_X)
- [x] joint-vote 实现 JointVoteEngine trait + cast_admin / cast_referendum 两 extrinsic
- [x] citizen-vote 空骨架(Phase 3 业务待接入)
- [x] votingengine 删除 internal_vote / joint_vote / citizen_vote 三 extrinsic
- [x] 业务 pallet wiring 切到 sub-pallet:`type InternalVoteEngine = InternalVote`(原 VotingEngine);`type JointVoteEngine = JointVote`
- [x] construct_runtime! 注册 InternalVote(22) / JointVote(23) / CitizenVote(24)
- [x] benchmarks.rs 加 3 行
- [x] votingengine alias / Proposals storage prefix 不变
- [x] 链端 11 pallet cargo test 全过(228 passed,0 failed)
- [x] wumin 公民钱包 105 tests 全过(pallet_registry + decoder + offline_sign + fixture 全部更新)
- [x] cargo check --workspace 通过
- [x] grep 零残留(votingengine.internal_vote / .joint_vote / .citizen_vote 全栈消失)
- [ ] wuminapp flutter analyze lib(本任务范围内 0 改动,未跑)
- [ ] Tauri tsc(VoteSigningFlow.tsx 已更新 pallet/call index,未跑)

## 关键决策(实施过程中)

1. **shell pallet 路由策略**:sub-pallet extrinsic 委派给 `votingengine::Pallet::do_X` 而非完全搬迁 storage/event/error;原因:storage 与 lifecycle helper 紧耦合,完全拆分需重写 ~1500 行;shell 模式实现"3 pallet 边界",代价小、风险可控
2. **InternalVoteEngine/JointVoteEngine impl 双份**:votingengine::Pallet 保留 trait impl(测试用),sub-pallet 也 impl(runtime 用);两个 impl 内容一致(都委派到 do_X);这样 votingengine 的 80 个测试 mock 不需要重做
3. **测试 helper 加 with_transaction wrap**:从 extrinsic 路径变成直接 do_X 调用后,需手动 with_transaction 还原 dispatch 隐式事务语义,否则 callback Err 不会回滚
4. **wumin pallet_registry 双 pallet**:internalVotePallet=22 / jointVotePallet=23 / votingEnginePallet=9 三个常量并存;decoder 按新 pallet 编码识别投票 extrinsic,votingengine 仅匹配 finalize/retry/cancel
5. **fixture 更新**:test/fixtures/step2d_credential_payload.json 中 citizen_vote 的 expected_call_data_hex 前两字节 `0x09 0x02` → `0x17 0x01`(JointVote(23).cast_referendum)
6. **VotingEngine 历史 call_index 留洞**:votingengine 内部 0/1/2 不再使用,从 3=finalize_proposal 起编号保持;新 sub-pallet 从 0 重新编号

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| trait 抽象层性能损耗 | 全部 monomorphize,wasm 编译期展开 |
| mode pallet test mock 复杂度 | 写共用 MockVotingEngine,11 pallet 复用 |
| 业务 pallet wiring 漏改 | cargo check --workspace 强制覆盖 |
| 历史 citizen_vote extrinsic 改名导致客户端硬编码失效 | grep citizen_vote / citizenVote 全栈替换 |

## 工时估算

约 10h(单 PR 完成全部链端 + 客户端)

## 上下文加载

- memory/05-modules/citizenchain/runtime/votingengine/VOTINGENGINE_TECHNICAL.md
- memory/08-tasks/done/20260504-100000-citizenchain-votingengine-提级与目录拆分.md(PR-X)
- memory/08-tasks/done/20260505-100000-投票引擎与机构管理模块重构.md(citizen 目录骨架建立)
