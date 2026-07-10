# 任务卡：固定治理骨架守卫(档 A,admins-change 真源加锚)

## 任务需求

`admins-change`(`public-admins::AdminAccounts` + `FederalRegistryProvinceGroups`)是全部机构管理员角色的唯一真源,但节点层零守卫,一次 setCode/恶意 runtime 可任意改写。护宪大法官 4/7 终审(ADR-027 §6.3,已落地的 `ConstitutionGuardVoteProof` 只锚了「4」)其信任封顶在本真源完整性上——是本卡要加锚的更底层真源(见卡 `20260709-constitution-amend-tier-hardrule.md` 遗留 follow-up)。

档 A = 把「永不合法变更的结构骨架」冻到节点二进制 + 创世,setCode 也改不动:

- I1 固定机构(NRC/PRC/PRB/NJD)`AdminAccounts[主账户]` 恒存在;
- I2 `institution_code` 不变;I3 `kind==PublicInstitution`;I4 `status==Active`;
- I5 固定名额不变(NRC=19/PRC=9/PRB=9/NJD=15);
- I6 **NJD 护宪大法官恰 7 人**(补 4/7 里没锚的「7」);
- I7 FRG 43 省组 `FederalRegistryProvinceGroups[省码]` 恒存在、`code==FRG`、`Active`、`len==5`。

**边界(诚实)**:只冻结构,不冻成员身份(普选/互选换人照常放行,等长换座即过);不冻阈值(固定治理阈值是 `fixed_governance_pass_threshold` 计票逻辑、不落 state,守卫锚不到)。成员劫持(保持 7 人但整体换攻击者密钥)不在档 A 范围,留档 B(创世根验签链,缓做)。

## 预计修改目录

- `citizenchain/runtime/primitives/`
  - 用途:新增 `governance_skeleton.rs` 冻结规格单源(固定机构枚举 + 护宪座位常量 + 护宪 role 字面量),genesis/runtime/node 三端共读防漂移。
  - 边界:只加只读常量与枚举函数,不改既有派生/发行/费率/宪法原语。
- `citizenchain/runtime/admins/admin-primitives/`
  - 用途:`ADMIN_ROLE_CONSTITUTION_GUARD` 改为 re-export 自 primitives,消除两份字面量。
- `citizenchain/runtime/admins/public-admins/`
  - 用途:runtime 侧同步 I6——NJD 管理员集变更强制护宪恰 7,避免「runtime 放行、节点拒块」裂缝。
  - 边界:只对 NJD 加护宪计数校验,不改其它机构、不改投票流程。
- `citizenchain/node/src/core/`
  - 用途:新增 `governance_skeleton.rs` 骨架守卫(仿 `constitution.rs`:编译规格 + 创世双锚 + 逐块 `check` + warp 提交前校验 + fail-closed);`mod.rs` 注册;`service.rs` 两处导入栈串联。
  - 边界:只加守卫,不改导入链路业务逻辑。
- `memory/04-decisions/` `memory/05-modules/` `memory/08-tasks/`
  - 用途:ADR-027 补档 A 骨架守卫口径 + 模块文档 + 本卡验收。

## 遗留(另窗口,已 spawn)

节点管理员展示解码器(`node/src/admins/admin_management/codec.rs`、`node/src/transaction/offchain_transaction/institution_read/chain.rs` 的 `OnChainAdminAccount`)字段序与当前 `admin-primitives::AdminProfile/AdminAccount` 不一致(缺 `cid_number` + `role_code/role_name/admin_source_ref`)。非本卡引入,独立核对 deployed↔source 后对齐。

## 验收要求

- `cargo test -p primitives`(governance_skeleton 单测)
- `cargo test -p public-admins`
- `cargo check -p primitives --no-default-features`(no_std/WASM)
- `cargo test --manifest-path citizenchain/node/Cargo.toml governance_skeleton`
- `cargo fmt`(两侧)+ clippy 零新增告警
- 纯节点守卫 + primitives 只读常量 → 无需重新创世;若含 public-admins 校验改动按链开发期规则重新创世即可。

## 进度

- [x] 建卡
- [x] primitives 规格层 + 单测
- [x] admin-primitives re-export
- [x] public-admins I6 runtime 校验
- [x] node 骨架守卫 + 装配
- [x] 构建 + 测试
- [x] 更新文档 + 完善注释 + 清理残留

## 执行结果(2026-07-09)

设计优化:守卫基准**整份在二进制**(codes/counts/护宪 7 全是 `primitives` 编译常量),block#0 仅
用于启动双锚确认——比宪法守卫(条文字节只在链上、须从 block#0 派生)更自包含。**阈值移出冻结清单**:
核实固定治理阈值是 `internal-vote::pass_threshold` 对 `NRC|PRC|PRB|FRG|NJD` 直接返回
`fixed_governance_pass_threshold(code)` 计票常量、**不落 state**(不写 `ActiveDynamicThresholds`),
守卫锚不到,故 I 清单只保留 state 上可断言的 I1..I7。

落点:
- `primitives/src/governance_skeleton.rs`(新)+ `lib.rs`:`fixed_institutions()`/`frg_province_groups()`/
  `NJD_CONSTITUTION_GUARD_SEATS=7`/`KIND_PUBLIC_INSTITUTION`/`STATUS_ACTIVE`/`ROLE_CONSTITUTION_GUARD` + 4 单测。
- `admin-primitives/src/lib.rs`:`ADMIN_ROLE_CONSTITUTION_GUARD` re-export 自 primitives;+2 交叉钉死测试
  (kind/status 判别值 ↔ 声明序、护宪字面量单源)。
- `genesis/src/institution.rs`:`national_judicial_yuan_admin_role` 的护宪范围改引用常量(`i<guard_seats`,
  行为不变),单源掉硬编码 `0..=6`。
- `public-admins/src/lib.rs`:`ensure_court_composition`(NJD 护宪恰 7)接入 `propose_admin_set_change` +
  执行终态 `try_execute_set_change_from_action`;新 Error `InvalidCourtComposition`。
- `node/src/core/governance_skeleton.rs`(新,641 行)+ `mod.rs` + `service.rs`(两处导入栈串接):
  `GovernanceSkeletonGuard` 完全复刻 `ConstitutionGuard`(创世双锚 + 逐块 `check_skeleton_invariants` +
  warp 提交前校验 + fail-closed + 快路径)+ 9 单测(合法态/等长换人放行、稀释/删机构/名额/非Active/改码/
  FRG 欠员拒块 + 键推导)。
- 文档:ADR-027 §6.4(承 §6.3 follow-up)、PRIMITIVES/ADMINS/NODE_TECHNICAL 同步。

设计要点:守卫在 runtime 之外 + primitives 只读常量 → **纯节点二进制加锚,无需 migration/重新创世**
(public-admins 那条 runtime 校验若单独部署才按链开发期规则重新创世)。**只冻席位数不冻成员**:等长
换人保持 7 席即放行,已单测覆盖。

验收(全过):primitives(governance_skeleton 4)/ admin-primitives 2 / public-admins 6 回归 /
node(governance_skeleton 9)/ primitives no_std(WASM)/ node `cargo check` 零告警 / fmt(5 crate)清。

天花板(honest):档 A 冻「7 这个数」不冻「这 7 个人」;成员劫持(保持 7 席整体换攻击者密钥)须档 B
(创世根验签链,缓做)。

## 补充测试与 clippy(2026-07-09,补齐上一轮 review 缺口 2/3/4)

- **item2 runtime I6 测试**:`ensure_court_composition` 改 `pub(crate)`;public-admins 新增
  `njd_court_requires_exactly_seven_guards`(7→过 / 6 稀释、8 灌水→`InvalidCourtComposition` / 非 NJD 不约束)。✅
- **item3 守卫装配/创世双锚测试**:node 新增 `fast_path_only_triggers_on_public_admins_or_code`(快路径分支)
  + `real_runtime_genesis_satisfies_skeleton_invariants`——用 `citizenchain::RuntimeGenesisConfig::default().build_storage()`
  生成**真创世**(含 `genesis::institution::build` 播种)后跑 `check_skeleton_invariants`,是 mirror 字段序 /
  存储键 / 规格计数与真链的最强交叉钉死,等价于"守卫 `new()` 启动双锚离线确认现行代码创世能过"。✅
  node guard 单测 9→**11** 全过。
  - 仍未覆盖(需 test-client,codebase 无先例,同 `ConstitutionGuard`):`BlockImport` 经真 client 的
    execute_block→delta→KnownBad / warp 提交前 / fail-closed 整链路端到端。
  - **部署风险仍在**:创世测试验的是**现行代码**创世(=新一次重新创世),非**当前部署的冻结 chainspec**;
    守卫上现网须与用当前字段模型重新创世同步(与解码器漂移独立任务同批确认)。
- **item4 clippy**:touched crates `--tests` 跑 clippy,**我的新增/改动码零告警**。工作区 `expect_used=warn`
  (阶段1,面向生产码)会命中新增测试里的 `expect()`,按惯例给两个新 `mod tests` 加 `#![allow(clippy::expect_used)]`
  opt-out(与全仓测试用 expect/unwrap 一致)。生产码零 expect/unwrap。仓库既存基线告警(china 常量位分组、
  trait `too_many_arguments` 等)非本卡引入、未动。

## 遗留(已 spawn 独立任务)

- `node/src/admins/admin_management/codec.rs::decode_admin_account` 与
  `node/src/transaction/offchain_transaction/institution_read/chain.rs::OnChainAdminAccount` 字段序疑似落后
  于当前 `admin-primitives`(缺 `cid_number` + `role_code/role_name/admin_source_ref`),**非本卡引入**,已
  spawn 独立任务核对 deployed↔source 后对齐(正确镜像可参考本卡 `governance_skeleton.rs::MAdminAccount`)。
- 观察:`cargo test -p genesis-pallet` 的**测试 mock**(`Test` runtime)预存 6 处 `Config` 未 wire 编译错误
  (`public_admins`/`public_manage` 等),stash 掉本卡改动仍在 → **与本卡无关的既存条件**;lib 本体编过。
