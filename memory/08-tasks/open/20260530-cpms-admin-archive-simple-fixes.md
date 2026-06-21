# CPMS 管理员与档案表单第1类修复

## 任务需求

- 档案创建/编辑中，公民状态为注销时选举资格默认并固定为无选举资格；只有公民状态为正常时才能选择选举资格。
- 具体地址改为详细地址，并在详细地址、公民状态、选举资格后显示必填星号。
- 出生日期不能选择当天和未来日期，后端也必须拒绝 `birth_date >= 当前 UTC 日期`。
- operators登录时，公民档案列表左侧显示当前 CPMS 安装省市。
- 右上角身份显示改为 `CPMS 机构管理员 · 姓名` 或 `operators · 姓名`，当前无姓名时显示预留名。
- 公民档案列表左侧省市之间使用上下左右居中的 `·` 分隔，列表标题显示为“公民档案列表”。
- 设置页删除CPMS 机构管理员已绑定数量/状态栏。
- 系统管理员列表改为管理员列表；用户 ID 完整显示；账户列显示完整 SS58 地址；角色文案显示operators。

## 预计修改目录

- `cpms/backend/dangan`：补出生日期后端校验和注销状态选举资格兜底。
- `cpms/backend/src/login`：登录态返回管理员姓名。
- `cpms/backend/admins`：管理员列表返回 SS58 地址。
- `cpms/backend/src/main.rs`：管理员结构读取姓名字段。
- `cpms/frontend/common`：登录态用户类型增加姓名。
- `cpms/frontend/login`：登录结果类型同步管理员姓名。
- `cpms/frontend/dangan`：档案创建/编辑表单、公民列表省市展示。
- `cpms/frontend/admins`：顶部身份显示、设置页清理、管理员列表文案与完整字段展示。
- `cpms/CPMS_TECHNICAL.md` 与 `memory/05-modules/cpms`：同步第 1 类规则。
- `memory/08-tasks`：记录本次修复与验证结果。

## 执行清单

- [x] 创建任务卡。
- [x] 修复出生日期和注销状态选举资格规则。
- [x] 修复登录态姓名和顶部身份显示。
- [x] 修复公民档案列表省市展示和标题文案。
- [x] 修复设置页和管理员列表简单项。
- [x] 更新文档。
- [x] 运行验证。

## 验证结果

- `cargo test --manifest-path cpms/backend/Cargo.toml`：通过，28 个测试通过。
- `cargo clippy --manifest-path cpms/backend/Cargo.toml --all-targets -- -D warnings`：通过。
- `npm run build`（`cpms/frontend`）：通过。
- `git diff --check`：通过。
- 残留扫描：未发现 `系统管理员列表`、旧 `系统管理员` 文案、旧 `具体地址` 文案、出生日期允许当天的前端校验残留。
