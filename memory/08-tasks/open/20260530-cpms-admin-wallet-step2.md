# CPMS 管理员与钱包绑定第2步修复

## 任务需求

- 同一个钱包账户只能绑定一个未硬删除档案；软删除仍占用钱包，100 年硬删除物理删除档案后才释放。
- 管理员列表显示超级管理员和操作管理员，初始超级管理员固定第一位。
- 初始化绑定的超级管理员不能删除；后续新增超级管理员和操作管理员可以删除。
- 超级管理员总数最多 5 个。
- 两种管理员都只能编辑姓名。
- 新增管理员必须选择超级管理员或操作管理员，姓名和账户地址必填。
- 完成后更新文档、补中文注释、清理旧 operator 命名残留。

## 预计修改目录

- `cpms/backend/db`：收口当前基准 schema，增加钱包公钥唯一索引。
- `cpms/backend/src/operator_admin`：增加钱包唯一绑定校验和错误返回。
- `cpms/backend/src/super_admin`：升级管理员列表、创建、编辑姓名和删除规则。
- `cpms/backend/src/main.rs`：补管理员错误码映射。
- `cpms/frontend/super_admin`：管理员列表 UI、角色选择、编辑姓名、删除按钮规则和 API/type 命名。
- `cpms/frontend/operator_admin`：钱包重复绑定错误提示。
- `cpms/.gitignore`：忽略本机公民资料库运行数据目录，避免运行数据进入代码变更。
- `cpms/CPMS_TECHNICAL.md` 与 `memory/05-modules/cpms`：同步规则和错误码。
- `memory/08-tasks`：记录执行与验证结果。

## 执行清单

- [x] 创建任务卡。
- [x] 实现钱包唯一绑定后端约束。
- [x] 实现管理员列表/新增/编辑姓名/删除后端规则。
- [x] 实现前端管理员列表和新增/编辑交互。
- [x] 更新文档和错误码。
- [x] 清理残留命名。
- [x] 运行完整验证。

## 验证结果

- `cargo test --manifest-path cpms/backend/Cargo.toml`：通过，31 个后端测试全部通过。
- `cargo clippy --manifest-path cpms/backend/Cargo.toml --all-targets -- -D warnings`：通过。
- `npm run build`（`cpms/frontend`）：通过。
- `git diff --check`：通过。
- 残留扫描：未发现旧 `/api/v1/admin/operators`、`/admin/operators`、`OperatorList`、旧操作员接口或旧“操作员”文案残留。
