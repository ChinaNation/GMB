# 节点端 AdminAccount/AdminProfile 解码器字段序对齐 + 金标向量防漂移

任务需求：
把节点端两个 `AdminAccounts` SCALE 解码器对齐到当前 `admin-primitives::{AdminAccount, AdminProfile}`
字段序,并补金标向量测试防再次漂移。二者是 2026-07-05 字段模型 commit(`9262935d3`)落地时漏改的
旧布局残桩,重新创世后会导致清算行机构详情/管理员展示解码失败或错位。

所属模块：citizenchain / node(节点桌面端只读解码)

## 背景与核对结论

- 冻结 chainspec 最后重生 `0bdbe8cf1`(2026-07-04 12:48),**早于**字段模型 `9262935d3`(2026-07-05 19:20)。
  `git merge-base --is-ancestor 0bdbe8cf1 9262935d3` = 真 → 当前部署链仍是旧布局,尚未重新创世。
- `9262935d3^`(旧布局)的 `AdminProfile = account,admin_cid_number,name,admin_role,term_start,term_end,source`
  (3 字符串,无 role_code/role_name/admin_source_ref);`AdminAccount` 无头部 `cid_number`。
  与两个解码器解析的字段序逐字段吻合 → 证实是旧布局残桩,非有意为之。
- 源码 HEAD 整棵树已切新布局(admin-primitives + runtime/genesis 种子 + 新写 `governance_skeleton.rs`
  的 `MAdminAccount/MAdminProfile`),**只剩这两个解码器没跟**。`GovernanceSkeletonGuard` 已接进
  `service.rs` 两条 import 路径,其创世双锚按新布局解码 → 当前 HEAD 节点必然配套新创世。
- 结论:按链开发死规则(彻底改/零残留/重新创世交付),现在就修,不等。

## 当前 vs 旧布局差异

- `AdminAccount` 头部新增 `cid_number: BoundedVec<u8>`(Compact 长度 + 字节),两解码器都从
  `institution_code` 起 → 整体前移一个变长字段。
- `AdminProfile`:`admin_role` 拆成 `role_code` + `role_name`(3→4 字符串);`source`→`admin_source`;
  尾部新增 `admin_source_ref: BoundedVec<u8>`。
- `chain.rs::OnChainAdminSource` 只有 5 枚举,当前 `AdminSource` 有 6(缺 `NominationAppointment`)。

必须遵守：
- 不改链端(runtime),不创世、不 setCode、不 migration(链开发中,重新创世交付)。
- DTO 字段名保持不变(`admin_role` 展示字段来源改取链上 `role_name`),避免前端契约/缓存变更。
- `role_code`/`admin_source_ref` 展示层不用,仅解析对齐偏移。
- 个人多签(kind==2)仍是裸 `AccountId` 列表,只补头部 `cid_number` 对齐。

## 改动清单

- `node/Cargo.toml`:新增 `[dev-dependencies] admin-primitives`(金标向量用真类型 encode)。
- `node/src/admins/admin_management/codec.rs`:重写 `decode_admin_account`(cid_number 头 + 4 字符串
  + admin_source_ref 尾);替换手搓测试为金标向量测试。
- `node/src/admins/admin_management/types.rs`:`source_label` 补 `5 => "提名任免"`。
- `node/src/transaction/offchain_transaction/institution_read/chain.rs`:`OnChainAdminAccount` 补
  头部 `cid_number`;`OnChainAdminProfile` 补 `role_code`/`role_name`(原 `title`→`role_name`)+ 尾
  `admin_source_ref`;`OnChainAdminSource` 补第 6 枚举;`admin_source_meta` 补一分支;`fetch_admin_set`
  取值 `p.title`→`p.role_name`;补金标向量测试。

输出物：
- 代码 + 中文注释
- 金标向量测试(两解码器各 1,用 `admin_primitives::AdminAccount::encode()` 产字节)
- 本任务卡 + memory 回写

验收标准：
- `cargo test -p node` 相关解码测试通过
- 两解码器逐字段对齐 `admin-primitives`,与 `governance_skeleton::MAdminAccount/MAdminProfile` 同布局
- 链端零改动;无旧布局残留
- Review 问题已处理

关联：`project_governance_skeleton_guard_adr027`(其"节点解码器字段序漂移已 spawn"即本项)、
`project_institution_admin_field_model_2026_06_28`(字段模型定稿真源)。

## 完成情况(2026-07-09,已验收)

- `node/Cargo.toml`:加 `[dev-dependencies] admin-primitives`(金标向量用真类型 encode)。
- `codec.rs::decode_admin_account`:头部解 `cid_number` + profile 四字符串(admin_cid_number/admin_name/
  role_code/role_name)+ 尾 `admin_source_ref`;展示 `admin_role`←`role_name`;两手搓测试替换为金标向量测试。
- `types.rs::source_label`:补 `5 => "提名任免"`。
- `institution_read/chain.rs`:`OnChainAdminAccount` 补头部 `cid_number`;`OnChainAdminProfile` 补
  `role_code`/`role_name`(原 `title`→`role_name`)+ 尾 `admin_source_ref`;`OnChainAdminSource` 补第 6
  枚举 + `admin_source_meta` 补分支;`fetch_admin_set` 取值 `p.title`→`p.role_name`;补金标向量测试。
- 链端零改动;无旧布局残留。
- 验证:`cargo check -p node --tests` 通过;`cargo test -p node` = **200 passed / 0 failed**
  (含 3 个新金标向量测试 + `governance_skeleton::real_runtime_genesis_satisfies_skeleton_invariants`)。
