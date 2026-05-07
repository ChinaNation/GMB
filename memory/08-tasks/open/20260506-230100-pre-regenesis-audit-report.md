# 清链重启前彻底审计 · 修正版总清单（v2）

- 任务关联：[20260506-230100-pre-regenesis-comprehensive-audit.md](20260506-230100-pre-regenesis-comprehensive-audit.md)
- v1 报告基于 6 个 Explore agent 输出未经原文核验,大量误判已删除
- v2 每条都标 **证据来源**(原文行号 + ADR/项目记忆),只列**已核验的真实问题**

---

# 一、v1 报告误判撤回（避免再次踩坑）

| v1 标记 | 实际状态 | 证据 |
|---|---|---|
| ❌ "P0-1：删 ClearingBank 整套" | **保留**,清算行体系完全活跃 | [ADR-006:35-38](../04-decisions/ADR-006-扫码支付-step1-同行MVP.md) L0-L3 四层架构里 L2=清算行 ; [ADR-007](../04-decisions/ADR-007-clearing-bank-three-phase.md) Step 1+2 done, Step 3 待 ; [project_institution_create_rules:澄清块](../../../.claude/projects/-Users-rhett-GMB/memory/project_institution_create_rules.md)「废除指 is_clearing_bank 字段模型,并非业务概念废除」 |
| ❌ "P0-2：删 SafetyFund / Sweep 提案 UI" | **保留**,extrinsic 真实存在 | [duoqian-transfer/lib.rs:443](../../citizenchain/runtime/transaction/duoqian-transfer/src/lib.rs:443) `propose_safety_fund_transfer` ; [:527](../../citizenchain/runtime/transaction/duoqian-transfer/src/lib.rs:527) `propose_sweep_to_main` ; [configs/mod.rs:289](../../citizenchain/runtime/src/configs/mod.rs:289) `safety_fund_account()` ; 前端 NrcSection/PrcSection 路由都正常 |
| ❌ "P0-2：删 RuntimeUpgradeProposalPage" | **保留**,joint vote 升级路径仍是合法路径 | [runtime-upgrade/lib.rs:163](../../citizenchain/runtime/governance/runtime-upgrade/src/lib.rs:163) `propose_runtime_upgrade` 与 [:217](../../citizenchain/runtime/governance/runtime-upgrade/src/lib.rs:217) `developer_direct_upgrade` 同时活跃 |
| ❌ "wuminapp ClearingBank 是死代码" | **是 ADR-007 Step 3 in-progress 工作** | [ADR-007 进度行](../04-decisions/ADR-007-clearing-bank-three-phase.md) "Step 3 待启动" |
| ❌ "wumin clearing_bank 5 条 op 死代码" | **是 ADR-007 Step 3 已就位的 wumin 部分** | wumin 的 ClearingBank decoder 与 action_labels 已为 Step 3 准备 |
| ❌ "ADR-007 整 ADR 失效需归档" | **保留**,Step 3 待实施时仍是权威依据 | 同上 |

---

# 二、已核验的真实问题（按优先级）

每条标 **证据**(文件:行 + 实际查到的代码片段)。

## 🔴 P0-1：Era 不对称(确认 mortal/immortal 共存)

[feedback_sfid_pow_chain_recipe.md](../../../.claude/projects/-Users-rhett-GMB/memory/feedback_sfid_pow_chain_recipe.md) 立铁律「PoW 链冷钱包/后端推链一律 immortal era」,但当前代码三处仍 mortal:

| 文件 | 行 | 现状 |
|---|---|---|
| [citizenchain/node/src/core/benchmarking.rs:130](../../citizenchain/node/src/core/benchmarking.rs:130) | 130 | `Era::mortal(period, best_block.saturated_into())` |
| [citizenchain/node/src/core/rpc.rs:146](../../citizenchain/node/src/core/rpc.rs:146) | 146 | `Era::mortal(period, best_number.saturated_into())` |
| [citizenchain/node/src/offchain/settlement/submitter.rs:240](../../citizenchain/node/src/offchain/settlement/submitter.rs:240) | 240 | `Era::mortal(period, best_block.saturated_into())` |

[citizenchain/node/src/governance/signing.rs](../../citizenchain/node/src/governance/signing.rs) 已 immortal。这三处需对齐。

修法：`Era::Immortal`(Substrate API 直接有该枚举)替换。

## 🔴 P0-2：wumin 冷钱包 supportedSpecVersions 锁死 spec=0

[wumin/lib/signer/pallet_registry.dart:23](../../wumin/lib/signer/pallet_registry.dart:23):
```dart
static const Set<int> supportedSpecVersions = {0};
```

清链重启后 spec 回到 0,但下次 setCode 立刻 spec=1。提前加 `1` 进集合(或后续每次升级都要再发版冷钱包)。

[wumin/test/signer/payload_decoder_test.dart:603](../../wumin/test/signer/payload_decoder_test.dart:603) 等多处用 `supportedSpecVersions.first` 作 spec 默认值,加 1 后测试需同步。

## 🔴 P0-3：sfid 后端/前端 spec_version 硬编码 0

| 文件 | 行 | 代码 |
|---|---|---|
| [sfid/backend/citizens/binding.rs:51](../../sfid/backend/citizens/binding.rs:51) | 51 | `"spec_version": 0,` 硬编码 |
| [sfid/frontend/sheng_admins/SuperAdminSubTab.tsx:426](../../sfid/frontend/sheng_admins/SuperAdminSubTab.tsx:426) | 426 | `spec_version: 0,` 硬编码 |

清链后 0→1 立坏,且违反 [feedback_no_chain_changes.md](../../../.claude/projects/-Users-rhett-GMB/memory/feedback_no_chain_changes.md) 精神(应从链动态读)。

修法:从 `state_getRuntimeVersion` 读取(后端缓存,前端拉一次)。

## 🟠 P1-1：MORTAL_ERA_PERIOD 死常量 + encode_mortal_era 死函数

[citizenchain/node/src/governance/signing.rs:17](../../citizenchain/node/src/governance/signing.rs:17) `MORTAL_ERA_PERIOD: u64 = 64` 与 [signing.rs:1286](../../citizenchain/node/src/governance/signing.rs:1286) `encode_mortal_era` 函数 —— 我前几小时改 immortal 后 lib 里无引用。但单元测试 `encode_mortal_era_period64` 还在测它(line 1460+)。

修法:删常量 + 函数 + 对应单元测试,一并清理。

## 🟠 P1-2：wumin payload_decoder.dart 注释头与实际不符

[wumin/lib/signer/payload_decoder.dart:858-859](../../wumin/lib/signer/payload_decoder.dart:858):
```dart
// OrganizationManage(17) / propose_create_personal(4)
// 格式：[17][4][BoundedVec account_name]...
```

实际路由是 `PersonalManage(7) call_index=0`(B 阶段拆分,2026-05-06 已落地)。注释头还停在拆分前的旧形态。函数体逻辑正确,**只是文档头错**。

修法:改注释为 `PersonalManage(7) / propose_create(0)`,格式行 `[7][0][...]`。

## 🟠 P1-3：memory 文档里 AdminsChange::Institutions 旧 storage 名残留

C 阶段 rename `Institutions → Subjects` 已落地,但下面 9 处文档仍写旧名:

| 文件 | 位置 |
|---|---|
| [citizenchain/runtime/governance/grandpakey-change/src/weights.rs](../../citizenchain/runtime/governance/grandpakey-change/src/weights.rs) | 51, 84(weights proof 注释) |
| [citizenchain/runtime/governance/resolution-destro/src/weights.rs](../../citizenchain/runtime/governance/resolution-destro/src/weights.rs) | 58, 85 |
| [citizenchain/runtime/governance/runtime-upgrade/src/weights.rs](../../citizenchain/runtime/governance/runtime-upgrade/src/weights.rs) | 56, 87 |
| [memory/05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md](../05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md) | 28, 112-145 |
| [memory/05-modules/citizenchain/node/governance/GOVERNANCE_TECHNICAL.md](../05-modules/citizenchain/node/governance/GOVERNANCE_TECHNICAL.md) | 126 |
| [memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md](../05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md) | 247 |
| [memory/05-modules/citizenchain/runtime/transaction/duoqian-transfer/DUOQIAN_TRANSFER_TECHNICAL.md](../05-modules/citizenchain/runtime/transaction/duoqian-transfer/DUOQIAN_TRANSFER_TECHNICAL.md) | 84 |
| [memory/05-modules/citizenchain/node/offchain/NODE_CLEARING_BANK_TECHNICAL.md](../05-modules/citizenchain/node/offchain/NODE_CLEARING_BANK_TECHNICAL.md) | 22 |
| [memory/05-modules/citizenchain/node/offchain/STEP2B_II_B_2_A_SUBMITTER.md](../05-modules/citizenchain/node/offchain/STEP2B_II_B_2_A_SUBMITTER.md) | 52 |
| [citizenchain/node/frontend/offchain/api.ts](../../citizenchain/node/frontend/offchain/api.ts) | 152, 155, 158, 168(注释) |
| [citizenchain/node/frontend/offchain/organization-manage/institution-detail.tsx](../../citizenchain/node/frontend/offchain/organization-manage/institution-detail.tsx) | 1(注释) |

修法:全局替换 `AdminsChange::Institutions` → `AdminsChange::Subjects`(注释/文档/weights proof)。

## 🟠 P1-4：runtime 注释里 OBSOLETE concept 残留

| 文件 | 行 | 残留概念 |
|---|---|---|
| [organization-manage/src/lib.rs](../../citizenchain/runtime/governance/organization-manage/src/lib.rs) | 100, 172, 347, 1027 | `finalize_create`(2026-05-02 Phase 3 已删) |
| [organization-manage/src/lib.rs](../../citizenchain/runtime/governance/organization-manage/src/lib.rs) | 477, 488, 840 | `ACTION_CREATE_PERSONAL` / 单账户机构 call=0 / call=3 hole |
| [organization-manage/src/institution/create.rs:4](../../citizenchain/runtime/governance/organization-manage/src/institution/create.rs:4) | 4 | 同上 |
| [organization-manage/src/benchmarks.rs:187](../../citizenchain/runtime/governance/organization-manage/src/benchmarks.rs:187) | 187 | 同上 |
| [runtime/src/configs/mod.rs](../../citizenchain/runtime/src/configs/mod.rs) | 351, 352, 1730 | 同上 |
| [node/src/governance/signing.rs:122](../../citizenchain/node/src/governance/signing.rs:122) | 122 | `vote_X` wrapper extrinsics(Phase 4 已删) |

修法:注释更新为「Phase 3/4 unified voting 已删」或直接清掉过时叙述。

## 🟡 P2-1：sfid chain_client 仅文档化 SfidSystem(10),其它 pallet 缺

[sfid/backend/app_core/chain_client.rs:48-57](../../sfid/backend/app_core/chain_client.rs:48):
```rust
pub(crate) const SFID_SYSTEM_PALLET_INDEX: u8 = 10;
pub(crate) const CALL_INDEX_ADD_SHENG_ADMIN_BACKUP: u8 = 2;
...
```

只列 SfidSystem(10) 的 pallet+call。**实际 sfid backend 推链只发 sfid_system 的 extrinsic**(根据这些常量名),所以**严格说不算 bug**。但如果未来要让 sfid 后端推 organization-manage 或 personal-manage 的 extrinsic(比如帮机构创建多签),需要补 pallet_index=17 / 7。

修法:**清链重启前不动**。等 sfid 后端确实需要发其它 pallet 的 extrinsic 时再补。

## 🟡 P2-2：sfid backend `#[allow(dead_code)]` 占位多处

| 文件 | 行 | 内容 |
|---|---|---|
| [sfid/backend/main.rs](../../sfid/backend/main.rs) | 32-36, 55-56, 62-64 | qr / cpms_register_inflight / sharded_store transition 占位 |
| [citizenchain/runtime/transaction/offchain-transaction/src/solvency.rs:62](../../citizenchain/runtime/transaction/offchain-transaction/src/solvency.rs:62) | 62 | `emit_warning_if_low` 死函数(Step 3 占位) |
| [citizenchain/runtime/governance/organization-manage/src/common.rs:43](../../citizenchain/runtime/governance/organization-manage/src/common.rs:43) | 43 | `_unused_dispatch_result_anchor` 占位 |

修法:清链前不动(占位的功能本就在等 Step 3 等后续阶段),只在 PR-5 文档清扫顺手验证一遍是否还需要。

## 🟡 P2-3：wumin test fixture step2d_credential_payload.json 含已删用例

[wumin/test/fixtures/step2d_credential_payload.json](../../wumin/test/fixtures/step2d_credential_payload.json) 含 `propose_runtime_upgrade` 用例,但 [payload_decoder_test.dart:603](../../wumin/test/signer/payload_decoder_test.dart:603) 注释明说该 case 已删。

修法:从 fixture 删该 entry。

## 🟡 P2-4：wuminapp test 用 wuminapp 自己的 ClearingBank* 测试可保留

[wuminapp/test/trade/clearing_bank_settings_page_test.dart](../../wuminapp/test/trade/clearing_bank_settings_page_test.dart)、[clearing_bank_prefs_test.dart](../../wuminapp/test/trade/clearing_bank_prefs_test.dart) —— ADR-007 Step 3 in-progress,**保留**(不算死代码)。但要确认这些测试是否真覆盖了当前 wuminapp 实现(Step 3 还没完整落地,可能测试断言错的对象)。

修法:清链前不动,Step 3 落地时一并 review。

## 🔵 P3-1：InstitutionPalletId 已改名 SubjectId,memory 文档未同步

| 文件 | 位置 |
|---|---|
| [memory/05-modules/citizenchain/runtime/governance/organization-manage/ORGANIZATION_MANAGE_TECHNICAL.md](../05-modules/citizenchain/runtime/governance/organization-manage/ORGANIZATION_MANAGE_TECHNICAL.md) | 39, 49, 50 |
| [memory/05-modules/citizenchain/runtime/STEP2_D_LAYER_B_PALLET_INTEGRATION.md](../05-modules/citizenchain/runtime/STEP2_D_LAYER_B_PALLET_INTEGRATION.md) | multiple |

修法:全局替换 `InstitutionPalletId → SubjectId`。

## 🔵 P3-2：memory 老 task card 长期 open 未关闭

| 文件 | 状态 |
|---|---|
| [memory/08-tasks/open/20260322-公民投票完整实现.md](20260322-公民投票完整实现.md) | 2026-03 任务,已超期 |
| [memory/08-tasks/open/20260325-133531-统一-duoqian-manage-命名.md](20260325-133531-统一-duoqian-manage-命名.md) | 已被 [20260505-215047 rename](20260505-215047-rename-org-manage-to-organization-manage.md) 取代 |
| [memory/08-tasks/open/20260331-step2-node-offchain-rpc-链下待结算账本.md](20260331-step2-node-offchain-rpc-链下待结算账本.md) | 2026-03 开,无后续 |
| [memory/08-tasks/open/20260401-step2a-offchain-pallet-简化密钥机制.md](20260401-step2a-offchain-pallet-简化密钥机制.md) | 同上 |
| [memory/08-tasks/open/20260404-offchain-packer-worker-接入.md](20260404-offchain-packer-worker-接入.md) | 同上 |
| [memory/08-tasks/open/20260502-sfid-step2b-duoqian-manage-credential.md](20260502-sfid-step2b-duoqian-manage-credential.md) | 标题旧名 |

修法:逐一确认是否真的完成 → 移到 `done/` 或者用 OBSOLETE 注脚标记。

## 🔵 P3-3：ADR-010 时间逆序

[memory/04-decisions/ADR-010-subject-id-protocol.md:11](../04-decisions/ADR-010-subject-id-protocol.md:11) 写「A 阶段(2026-05-04)前的派生协议」但 ADR 自身日期标 2026-05-06。

修法:核对实际拆分时间,统一时间线。

---

# 三、清链重启路径(精简版)

按 v2 报告分 3 个 PR(原 v1 的 5 个 PR 中 PR-1/2 整体撤回):

| PR | 内容 | 估时 |
|---|---|---|
| **PR-A** | P0-1 / P0-2 / P0-3:Era 切 immortal 三处 + wumin spec 集合加 1 + sfid spec_version 动态读 | 1-2 h |
| **PR-B** | P1-1 / P1-2 / P1-4:删 MORTAL_ERA_PERIOD 死常量 + 改 wumin 注释头 + 清 OBSOLETE 注释 | 1 h |
| **PR-C** | P1-3 / P3-1 / P3-2 / P3-3:文档/memory 全局清扫 + 老 task card 整理 | 2-3 h |

3 个 PR 全部合并 + cargo/flutter check 0 警告 + 测试通过 → **6 台 `fuwuqi.sh q` 清链重启** → 重新导出 chainspec → 删客户端 Institutions 临时 patch → 重 build → UI 应该正常(因为重启后所有数据都在 NEW 路径,无需绕过)。

---

# 四、本次审计的方法论教训(立条铁律)

**新铁律建议**:

> Explore subagent 报告**不是结论**,只是**线索池**。每条 finding 都必须回原文(代码 `file:line` + 权威 ADR/项目记忆)核验后才能进入正式报告。Agent 输出可能基于 prompt 过度简化或对历史叙述误读。

应在 [memory/07-ai/agent-rules.md](../07-ai/agent-rules.md) 或新建 `memory/07-ai/audit-recipe.md` 立条:**审计类任务,subagent 输出仅作 leads,正式报告以 `file:line + ADR 锚点`形式给证据**。

---

# 五、对比 v1 报告的核心修正

| v1 类别 | v1 数量 | v2 数量 | 删减/重定 |
|---|---:|---:|---|
| P0 阻塞 | 7 大类 | **3 项** | 删 P0-1(ClearingBank)、P0-2(Safety/Sweep/Upgrade UI)、P0-6(CPMS QR1 待用户确认)、P0-7 合入 P0-3 |
| P1 命名残留 | 5 大类 | **4 项** | 删 P1-3(wumin PersonalManage 二重身改为只是 stale comment),P1-5(memory OBSOLETE 归档撤回:ADR-007 不归档) |
| P2 死代码 | 8 处 | **4 项** | 死代码确认 4 处真死,4 处 in-progress 占位保留 |
| P3 文档漂移 | 5 大类 | **3 项** | 大部分合并到 P1-3 |
| P4 推荐改进 | 4 项 | **1 项** | 只剩"audit 方法论铁律" |

**v1 → v2 删除约 60% 误判项**,v2 保留 **~25 条已核验真实问题**,均以 ADR / 代码 file:line 为证据。
