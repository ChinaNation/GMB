# PR-C 旧 storage 真源与模块名叙述清理

## 任务目标

执行重新创世前总审计 PR-C 第二段：清理当前活跃代码注释和当前技术文档中仍把旧 `duoqian-manage`、旧 `DuoqianAccounts` 或旧 `AdminsChange.Institutions` 当作当前真源的叙述。

本任务不改业务逻辑、不改 storage、不改 ABI，只修正当前说明文字。

## 当前真源

- 机构多签：`organization-manage`
- 个人多签：`personal-manage`
- 管理员主体和阈值：`admins-change::Subjects`
- 机构账户：`OrganizationManage::InstitutionAccounts`
- 个人多签账户：`PersonalManage::PersonalDuoqians`
- 旧 `DuoqianAccounts`：只允许出现在明确标注为 legacy/history 的说明中。

## 预计修改目录

- `citizenchain/runtime/`：修正活跃 runtime 注释中的旧模块名和旧 storage 真源；不改业务逻辑。
- `wuminapp/lib/`：修正一处活跃 Dart 注释里的旧 runtime 来源；不改类名和逻辑。
- `memory/05-modules/`：修正当前技术文档中的旧 storage、旧模块名和旧管理员真源叙述。
- `tools/`：修正生成器输出的制度保留地址注释，避免重新生成后旧名回流。
- `memory/08-tasks/open/`：记录执行范围、结果和验收；只涉及文档。

## 执行清单

- [x] 修正 runtime 活跃注释中的 `duoqian-manage` 旧模块名。
- [x] 修正 runtime 活跃注释中的 `DuoqianAccounts` 旧 storage 真源。
- [x] 修正 wuminapp 活跃注释中的旧 runtime 来源。
- [x] 修正当前技术文档中的旧 storage / 旧模块名 / 旧管理员真源。
- [x] 修正 `tools/duoqian.py` 生成的旧模块名注释。
- [x] 回写审计文档并运行验收。

## 不处理范围

- 历史任务卡中的旧名记录本轮不批量改写，留到 PR-D 任务卡归档/冻结阶段处理。
- `DuoqianManageService` / `DuoqianManageDetailPage` 属于 wuminapp “多钱管理”业务命名，本轮不重命名。
- `citizenchain/runtime/primitives/` 当前有外部未暂存改动；本轮只补 `china_zb.rs` 第一段注释里的旧模块名，不触碰常量数据。

## 验收标准

- 活跃 runtime 和当前技术文档不再把 `duoqian-manage` / `DuoqianAccounts` / `AdminsChange.Institutions` 表述为当前真源。
- `cargo check -p offchain-transaction` 通过。
- `cargo check -p duoqian-transfer` 通过。
- `cargo check -p organization-manage` 通过。
- `git diff --cached --check` 通过。

## 执行结果

2026-05-07 已执行：

- 活跃 runtime 注释中的当前模块名已统一为 `organization-manage` / `personal-manage` / `admins-change::Subjects`。
- 当前技术文档中的 `DuoqianAccounts` 当前真源叙述已改为 `OrganizationManage::InstitutionAccounts` 或 `PersonalManage::PersonalDuoqians`。
- wuminapp 活跃注释中注册多签来源已改为 `organization-manage` 的机构主账户。
- `tools/duoqian.py` 与 `china_zb.rs` 的制度保留地址注释已从旧 `duoqian-manage` 改为 `organization-manage`。

剩余扫描命中均为明确 legacy/history 语境：

- 旧 mirror 已删除的代码注释。
- 迁移代码中用于清理旧 `AdminsChange::Institutions` 的字符串。
- 当前技术文档中明确写“替代旧 `DuoqianAccounts`”或“真源不再是旧字段”的历史说明。

验收记录：

- `cargo check -p offchain-transaction`：通过。
- `cargo check -p duoqian-transfer`：通过。
- `cargo check -p organization-manage`：通过。
- `rg -n 'duoqian-manage|DuoqianAccounts|AdminsChange\\.Institutions|AdminsChange::Institutions|AdminChange::Institutions|DuoqianManage\\.DuoqianAccounts' citizenchain/runtime wuminapp/lib memory/05-modules tools/duoqian.py --glob '!**/target/**'`：仅剩 legacy/history 命中。
- `git diff --check`：通过。
- `git diff --cached --check`：通过。
