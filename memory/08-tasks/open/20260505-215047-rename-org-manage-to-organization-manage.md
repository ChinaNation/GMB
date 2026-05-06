# 任务卡：重命名 org-manage 为 organization-manage

- 任务编号：20260505-215047
- 状态：open
- 负责人：当前主聊天入口（Architect Agent + Blockchain Agent + Mobile Agent + SFID Agent 联合执行）
- 关联前置：无
- 关联后续：20260505-215048（拆分 personal-manage）

## 1. 任务目标

把 GMB 仓库内 `org-manage` 模块的 crate 名、目录名、Rust 模块路径、pallet 对外名 `DuoqianManage`、MODULE_TAG `b"dq-mgmt"` 以及 node 后端业务目录 `duoqian_manage/` 全部统一重命名为 `organization-manage` / `organization_manage` / `OrganizationManage` / `b"org-mgmt"`，并同步更新 wumin 冷钱包、wuminapp 热钱包、sfid 后台、节点 Tauri 前端、文档与任务记录。

不改动业务逻辑、不改 Event/Error 字面名（如 `DuoqianCreated`/`DuoqianClosed` 保持原字面）、不改 `DuoqianTransfer` pallet（独立模块）、不改 wuminapp `lib/duoqian/` 业务目录命名（按多签业务而非 pallet 分层）。

## 2. 影响范围

### 2.1 citizenchain/runtime
- `citizenchain/runtime/governance/org-manage/` → `organization-manage/`
- `citizenchain/runtime/governance/org-manage/Cargo.toml` 的 `name = "org-manage"` → `name = "organization-manage"`
- `citizenchain/runtime/governance/org-manage/src/lib.rs:4` MODULE_TAG `b"dq-mgmt"` → `b"org-mgmt"`
- `citizenchain/runtime/Cargo.toml` 第 69/107/158/189 行四处 dep 与 feature
- `citizenchain/Cargo.toml:27` workspace member 路径
- `citizenchain/runtime/src/lib.rs:344` `pub type DuoqianManage = org_manage;` → `pub type OrganizationManage = organization_manage;`
- `citizenchain/runtime/src/lib.rs:418` 测试常量数组键 `"org_manage"` → `"organization_manage"`
- `citizenchain/runtime/src/configs/mod.rs` 约 50 处 `org_manage::` → `organization_manage::`、`RuntimeCall::DuoqianManage` → `RuntimeCall::OrganizationManage`
- `citizenchain/runtime/src/benchmarks.rs` use 路径
- `citizenchain/runtime/transaction/duoqian-transfer/Cargo.toml` 与 `src/lib.rs` 引用替换
- `citizenchain/runtime/transaction/offchain-transaction/src/bank_check.rs` use 路径
- `citizenchain/runtime/votingengine/src/traits.rs:327` 注释字面修正
- `citizenchain/runtime/governance/admins-change/src/lib.rs` 5 处 `b"dq-mgmt"` → `b"org-mgmt"`

### 2.2 citizenchain/node
- `citizenchain/node/src/offchain/duoqian_manage/` → `organization_manage/`
- `citizenchain/node/src/offchain/duoqian_manage/chain.rs` 内 3 处字符串字面 `"DuoqianManage"` → `"OrganizationManage"`
- `citizenchain/node/src/offchain/{mod,settlement,common/mod,common/types}.rs` use 路径与注释字面替换
- `citizenchain/node/src/governance/proposal.rs:18` 常量名 `TAG_DUOQIAN_MANAGE` → `TAG_ORGANIZATION_MANAGE`，值 `b"dq-mgmt"` → `b"org-mgmt"`；`is_duoqian_manage_proposal` → `is_organization_manage_proposal`
- `citizenchain/node/src/governance/signing.rs:122` 注释字面
- `citizenchain/node/src/desktop/mod.rs` 6 处 `crate::offchain::duoqian_manage::commands::*` → `organization_manage::commands::*`
- `citizenchain/node/frontend/offchain/duoqian-manage/` → `organization-manage/`
- `citizenchain/node/frontend/offchain/{api,section,types}.ts(x)` 路径与字符串字面替换

### 2.3 wumin（冷钱包）
- `wumin/lib/signer/payload_decoder.dart` MODULE_TAG 字节数组替换
- `wumin/lib/signer/pallet_registry.dart` pallet 名字符串
- `wumin/test/signer/pallet_registry_test.dart` 同步测试用例

### 2.4 wuminapp（热钱包）
- `wuminapp/lib/duoqian/shared/duoqian_manage_service.dart` 5 处 `'DuoqianManage'` 字符串字面 + `"dq-mgmt"` 解码常量
- `wuminapp/lib/citizen/institution/institution_admin_service.dart:145` 1 处字符串字面
- `wuminapp/test/duoqian/duoqian_manage_service_test.dart` 同步测试
- 文件名 `duoqian_manage_service.dart` 不改（业务命名层），仅替换内部字符串字面

### 2.5 sfid（后台）
- `sfid/backend/indexer/event_parser.rs:245-262` 注释 + 2 处字符串字面 `("DuoqianManage", ...)` → `("OrganizationManage", ...)`
- `sfid/backend/institutions/{derive,service}.rs` 注释字面修正
- 不改业务行为；事件名 `DuoqianCreated`/`DuoqianClosed` 保持

### 2.6 文档
- `memory/05-modules/citizenchain/` 下旧命名引用同步替换
- `memory/01-architecture/citizenchain/` 同上
- `memory/08-tasks/open/` 已开任务卡内引用同步（不动已归档卡）
- `memory/MEMORY.md` 索引词条同步

## 3. 关键约束

- 不改业务行为，只做命名统一
- 不改 Event/Error 字面名（DuoqianCreated/DuoqianClosed/PersonalDuoqianProposed 等保留）
- 不改 `DuoqianTransfer` pallet（独立模块，用户未要求）
- 不改 wuminapp `lib/duoqian/` 目录命名（业务分层目录，与 pallet 解耦）
- 不动 chainspec.json（feedback_chainspec_frozen.md；本任务不需要）
- 链尚在开发期，重启 fresh genesis 即可（feedback_chain_in_dev.md）
- 跨模块联动：runtime + node + wumin + wuminapp + sfid 必须同步推进，不允许只改单边（chat-protocol §5）
- 不留兼容代码、不留旧名 alias、不留过渡分支（feedback_no_compatibility.md）
- 不引入新行为（feedback_no_scope_expansion.md）

## 4. 执行计划

1. **链端 crate 重命名**：
   - mv 目录、改 Cargo.toml `name`、workspace 路径、runtime dep 4 处
   - lib.rs MODULE_TAG 字节字面修改
   - admins-change 5 处 `b"dq-mgmt"` 同步
2. **runtime 装配层**：
   - construct_runtime 改 `pub type OrganizationManage = organization_manage`
   - configs/mod.rs / benchmarks.rs / lib.rs:418 全部 `org_manage::` 路径替换、`RuntimeCall::DuoqianManage` 替换
3. **链端依赖 crate**：duoqian-transfer / offchain-transaction / votingengine 注释 + use 修正
4. **node 后端**：
   - mv `node/src/offchain/duoqian_manage/` → `organization_manage/`
   - chain.rs 内 3 处 `"DuoqianManage"` 字符串字面修正
   - desktop/mod.rs 6 处命令路径
   - governance/proposal.rs 常量名 + 字节字面 + 函数名
5. **node 前端**：
   - mv `node/frontend/offchain/duoqian-manage/` → `organization-manage/`
   - api.ts / section.tsx / types.ts 路径与字符串
6. **wumin**：payload_decoder + pallet_registry + 测试
7. **wuminapp**：duoqian_manage_service.dart 5 处 + institution_admin_service.dart 1 处 + dq-mgmt 解码常量 + 测试
8. **sfid**：event_parser.rs 2 处字符串字面
9. **文档与残留**：memory/ 全文 grep 替换、MEMORY.md 索引同步
10. **验证**：编译 + 跑测试 + grep 残留扫描

## 5. 验证要求

### 5.1 编译
- `cargo check -p citizenchain` 通过
- `cargo check -p organization-manage` 通过
- `cargo check -p citizenchain-node`（含 features）通过
- `cargo check -p sfid-backend` 通过

### 5.2 测试
- `cargo test -p organization-manage` 通过（原 org-manage 全部用例迁移后过）
- `cargo test -p citizenchain --lib` 通过
- `cargo test -p admins-change` 通过
- wumin：`cd wumin && flutter test` 通过
- wuminapp：`cd wuminapp && flutter test test/duoqian/` 通过

### 5.3 残留扫描（必须零结果）
```bash
grep -rln "org-manage\|org_manage\|OrgManage" citizenchain/ wumin/ wuminapp/ sfid/ memory/ --include="*.rs" --include="*.toml" --include="*.dart" --include="*.ts" --include="*.tsx" --include="*.md" 2>/dev/null | grep -v "target/"
grep -rln "DuoqianManage" citizenchain/ wumin/ wuminapp/ sfid/ --include="*.rs" --include="*.dart" --include="*.ts" --include="*.tsx" 2>/dev/null | grep -v "target/"
grep -rln "duoqian_manage\|duoqian-manage\|duoqianManage" citizenchain/node/ wumin/lib/signer/ sfid/backend/ memory/ 2>/dev/null
grep -rln "dq-mgmt" citizenchain/ wumin/ wuminapp/ sfid/ --include="*.rs" --include="*.dart" --include="*.ts" --include="*.tsx" 2>/dev/null | grep -v "target/"
```
（wuminapp/lib/duoqian/ 业务目录与 wumin/lib/duoqian/ 相关文件名为业务分层，不在残留扫描范围）

### 5.4 行为不变量
- 链上 storage layout 在 fresh genesis 下与重命名前等价（仅 pallet prefix 哈希值变化，结构与字段名一致）
- 提案 MODULE_TAG 改为 `b"org-mgmt"` 后，wumin 冷钱包 + wuminapp 热钱包 + admins-change pallet 三方解码一致
- node 桌面端 6 个 Tauri command 名称不变（仅模块路径变）

## 6. 风险与回滚

- **风险 R1**：MODULE_TAG 改字节后，旧 fresh genesis 已派发但未消费的提案在升级瞬间会找不到 owner pallet。**缓解**：本任务在链未上线前完成，配合重启 fresh genesis 直接生效。
- **风险 R2**：wuminapp `lib/duoqian/` 目录不改但内部字符串改，可能造成"目录名 vs pallet 名"读者认知偏差。**缓解**：在 `wuminapp/lib/duoqian/shared/duoqian_manage_service.dart` 顶部加一行中文注释说明业务目录与 pallet 名解耦。
- **风险 R3**：node Tauri 前端目录 `duoqian-manage/` 改名后，浏览器缓存或打包路径若有写死会报 404。**缓解**：grep `"duoqian-manage"` 字符串字面、检查 import 路径。
- **回滚**：单一 commit / PR，回滚 = revert。

## 7. 输出物

- 代码：上述全部目录的命名替换
- 中文注释：每个新增 / 修改文件维持原中文注释风格；新增的"业务目录与 pallet 名解耦"说明
- 测试：所有原测试用例迁移到新 crate 名后全部通过
- 文档更新：memory/ 索引、模块文档、MEMORY.md 词条
- 残留清理：第 5.3 节四条 grep 全部零结果

## 8. 执行结果

（待填写）

## 9. 验证结果

（待填写）
