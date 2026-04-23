# 任务卡：投票引擎统一入口整改 · Phase 3

- 状态：done(2026-04-22),与 Phase 1+2 一起移 done
- 归属：Blockchain Agent(runtime spec_version + node 内置 Tauri UI) + Mobile Agent(wumin 冷钱包 + wuminapp 热钱包)
- 承接：`20260422-unified-voting-entry-phase1.md` / `20260422-unified-voting-entry-phase2.md`

## Phase 3 目标

把所有"管理员一人一票"动作收敛到客户端单一入口：

```
(pallet=9, call=0)  VotingEngineSystem::internal_vote(proposal_id: u64, approve: bool)
```

`joint_vote` / `citizen_vote` 路径保留，但 call_index 同步 Phase 2 新编号
(joint 3→1 / citizen 4→2 / finalize 新增 3)。

## 范围校准（关键）

Explore 扫描确认真实客户端分布:

- **sfid-backend 不参与本 Phase**。sfid 只给"公民投票"签凭证
  (`build_vote_credential` / `OP_SIGN_VOTE` 0x11),管理员治理投票从不经过 sfid。
- **sfid-frontend 不参与本 Phase**。前端无治理投票 UI。
- 真正客户端 = **三处**:
  1. `citizenchain/node/src/ui/governance/` + `citizenchain/node/frontend/`
     — 节点自带 Tauri UI,单独一套 signing.rs + api.ts + VoteSigningFlow.tsx
  2. `wumin/lib/signer/` — 冷钱包(payload 解码 + 离线签名)
  3. `wuminapp/lib/governance/` — 热钱包(构造 extrinsic + 提交)

## 改动清单

### citizenchain/node · 节点自带 Tauri 治理 UI

| 文件 | 改动 |
|---|---|
| `src/ui/governance/signing.rs` | `build_vote_sign_request`:call data 从 `[19][1]` 改为 `[9][0]`(internal_vote),display action `vote_transfer → internal_vote`;`build_joint_vote_sign_request`:call_index 3→1;`build_propose_safety_fund_sign_request`:call_index 3→1;`build_propose_sweep_sign_request`:call_index 5→2;删除 `build_safety_fund_vote_sign_request` / `build_sweep_vote_sign_request` 两个函数 |
| `src/ui/governance/mod.rs` | 删除 `build_safety_fund_vote_request` / `build_sweep_vote_request` 两个 Tauri command;`submit_propose_sweep` 内 call_index 5→2;`submit_propose_safety_fund` 内 call_index 3→1 |
| `src/ui/mod.rs` | `invoke_handler!` 移除上述两个 command 注册 |
| `frontend/api.ts` | 删除 `buildSafetyFundVoteRequest` / `buildSweepVoteRequest` |
| `frontend/governance/VoteSigningFlow.tsx` | 移除 `useSafetyFundVote` / `useSweepVote` props + 对应分支;`buildVoteCallDataHex` 改为 `buildInternalVoteCallDataHex`(`[9][0]` 编码);`buildJointVoteCallDataHex` 内 call_index 3→1;删除 `buildSweepVoteCallDataHex` / `buildSafetyFundVoteCallDataHex` |
| `frontend/governance/ProposalDetailPage.tsx` | 移除向 `VoteSigningFlow` 传的 `useSafetyFundVote` / `useSweepVote` |

### wumin · 冷钱包 payload 解码与 call_index

| 文件 | 改动 |
|---|---|
| `lib/signer/pallet_registry.dart` | VotingEngineSystem 新增 `internalVoteCall=0` / `finalizeProposalCall=3`;jointVote 3→1 / citizenVote 4→2;删除业务 pallet 的投票 call 常量(voteAdminReplacementCall / voteDestroyCall / voteCloseCall(17) / voteKeyChangeCall) |
| `lib/signer/payload_decoder.dart` | 新增 `_decodeInternalVote`(pallet=9 call=0);重写 `_decodeJointVote`(call 3→1) / `_decodeCitizenVote`(call 4→2);删除 `_decodeVoteProposal` 及其 dispatch 表 |
| `lib/signer/action_labels.dart` | 删 `vote_admin_replacement / vote_destroy / vote_close / vote_key_change / vote_create / vote_transfer / vote_safety_fund_transfer / vote_sweep_to_main` 标签;新增 `internal_vote` / `finalize_proposal` 标签 |

### wuminapp · 热钱包治理服务

| 文件 | 改动 |
|---|---|
| 新增 `lib/governance/internal_vote_service.dart` | 唯一投票入口 `InternalVoteService.submit(proposal_id, approve)`,构造 `(pallet=9, call=0)` extrinsic + 全流程 runtime_version / nonce / signing payload / sr25519 签名 / 提交 |
| `lib/governance/transfer_proposal_service.dart` | 删 `submitVoteTransfer` / `submitVoteSafetyFund` / `submitVoteSweep`(旧 pallet=19 call=1/4/6)+ 三个 `_buildVoteXCall` helper;`propose_safety_fund` call_index 3→1、`propose_sweep_to_main` call_index 5→2 |
| `lib/governance/duoqian_manage_service.dart` | 删 `submitVoteCreate` / `submitVoteClose`(旧 pallet=17 call=3/4)+ `_buildVoteCall` helper;`propose_create_personal` call_index 5→3 |
| `lib/governance/runtime_upgrade_service.dart` | `submitJointVote` 的 `_jointVoteCallIndex` 3→1(与 Phase 2 runtime 对齐);逻辑不动 |
| `lib/governance/transfer_proposal_detail_page.dart` | `_submitVote` 按 kind 分派改为单一 `InternalVoteService().submit()`;`_signAction` 恒为 `'internal_vote'`;`TransferProposalKind` 注释更新为 Phase 3 现状 |
| `lib/governance/duoqian_manage_detail_page.dart` | `_submitVote` 按 create/close 分派改为单一 `InternalVoteService().submit()`;QR 签名 action 恒为 `'internal_vote'` |

## call_index 变更对照(Phase 2 runtime 真值)

pallet=9 VotingEngineSystem:
- 0 `internal_vote(proposal_id, approve)` ← Phase 3 主入口
- 1 `joint_vote(proposal_id, institution_id, approve)` ← 原 3
- 2 `citizen_vote(proposal_id, binding_id, nonce, signature, approve)` ← 原 4
- 3 `finalize_proposal(proposal_id)` ← 新增

业务 pallet 投票 call 全部删除(Phase 2 已在链上完成),客户端必须同步删除。

## spec_version / genesis

- `citizenchain/runtime/src/lib.rs` 中 `spec_version: 1 → 2`(开发期,fresh genesis)
- `transaction_version` 保持 1(TxExtension 元组顺序未动)
- WASM CI 重新生成
- 按 `feedback_chain_in_dev` 开发期 fresh genesis,不走 setCode

## 验收

- ✅ `cargo check -p node`(+ stub `frontend/dist/index.html`)通过,0 error / 42 warning(preexisting)
- ✅ Rust 六个治理/业务 pallet 单测 **135/135** 通过(voting-engine 49 / admins 20 / resolution-destro 14 / grandpa-key 15 / duoqian-manage 17 / duoqian-transfer 20)
- ✅ wumin `flutter test test/signer/` **44/44** 通过(pallet_registry / payload_decoder / qr_signer / offline_sign_service)
- ✅ `cd citizenchain/node/frontend && npx tsc -b` 无错误
- ✅ wumin `flutter analyze lib/signer test/signer` 无 issue
- ✅ wuminapp `flutter analyze lib/governance` 无 issue
- ✅ runtime `spec_version: 1 → 2`,注释更新
- ✅ 全仓 grep 归零(仅保留 Phase 3 tombstone 注释 + 历史 memory 文档):
  - 客户端侧 `submitVoteTransfer / submitVoteSafetyFund / submitVoteSweep /
    submitVoteCreate / submitVoteClose / buildSweepVoteRequest /
    buildSafetyFundVoteRequest / useSafetyFundVote / useSweepVote /
    voteAdminReplacementCall / voteDestroyCall / voteKeyChangeCall /
    _voteCallIndex / _decodeVoteProposal` 全数清零
  - Rust 端 `cast_internal_vote(` 调用清零(仅保留 voting-engine-system 测试
    helper `cast_internal_vote_via_extrinsic`,名称保留但内部走
    `Pallet::internal_vote` extrinsic)
- ⏭️ 端到端联调(fresh genesis → 5 种提案各投一轮)留给运行时启动时执行

## 清理残留

- wumin/wuminapp 里 Phase 1/2 遗留的 "finalize_X 旧路径" 注释统一清理为"Phase 2 已删除,全部走 internal_vote"
- auto-memory:
  - `project_duoqian_finalize_create_step1.md` → 标 obsolete(Phase 2 已全部替换)
  - `project_duoqian_finalize_transfer_step2.md` → 标 obsolete
- Phase 1/2/3 三张任务卡 Phase 3 完工时一起移 `08-tasks/done/`

## 不在本 Phase 范围

- sfid-backend / sfid-frontend 不改(公民投票凭证路径与 Phase 2 不冲突)
- wumin/wuminapp 共享 Dart 包抽取(独立任务)
