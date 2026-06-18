# 任务卡：CPMS china.sqlite 定位三层兜底 + CI 触发加固

## 任务需求

CPMS 后端 `china_db_path()` 当前只有「env `CPMS_CHINA_DB` → 默认 `./china.sqlite`」两级，本地 dev `cargo run` 因 env 未设、CWD 无文件而报 `open china sqlite ./china.sqlite failed`。改为三层兜底，让 dev 零配置即通，生产仍以 env 为准且更稳。

## 背景结论（分析已确认）

- 单仓 monorepo，cpms 与 sfid 同仓；`china.sqlite` 是 88MB 普通 blob 直接进 git（非 LFS）。
- `actions/checkout@v5` 默认全量检出 → CI 一定带 `sfid/backend/china/china.sqlite`。
- CI 两条路径已通：`cargo test`（测试 helper 指向 manifest 相对 SFID 源，CI 有文件 → 真跑且过）、`workflow_dispatch` 出 `.run`（build 脚本从 sfid 路径 `cp`）。唯一缺口是本地 `cargo run`。
- 房子约定：SFID `china/store.rs` 用 `concat!(env!("CARGO_MANIFEST_DIR"), "/china/china.sqlite")` 编译期定位。

## 建议模块

- CPMS 后端 `china`
- CPMS CI workflow
- CPMS 技术文档

## 影响范围

- `cpms/backend/china.rs`：`china_db_path()` 改三层兜底：
  1. env `CPMS_CHINA_DB` 非空 → 用它（生产 install_host 写 `/opt/cpms/data/china.sqlite`；dev 也可手动覆盖）。
  2. 二进制旁 `<exe_dir>/../data/china.sqlite`（部署自定位：`/opt/cpms/bin` → `/opt/cpms/data`，env 丢失也能找到）。
  3. `concat!(env!("CARGO_MANIFEST_DIR"), "/../../sfid/backend/china/china.sqlite")`（dev 源码，`cargo run` 零配置）。
  涉及代码。
- `.github/workflows/cpms-ci.yml`：`paths:` 触发补 `sfid/backend/china/china.sqlite`，SFID 行政区数据变更时 CPMS 也重验。涉及 CI 配置。
- `cpms/CPMS_TECHNICAL.md` 第 11 节：补三层兜底说明。涉及文档。

## 主要风险点

- dev 源码路径（第 3 层）被编译进生产二进制：仅兜底用，env/exe 旁文件优先，且生产 env 必设；信息层面暴露构建机路径，影响可忽略。
- 第 1 层信任 env 原值（不校验存在性），设错即 fail-loud，符合预期。
- 不引入新依赖；`current_exe()` 为 std。

## 是否需要先沟通

- 机制已与用户确认：代码三层兜底（生产仍走 `CPMS_CHINA_DB`，不受影响）。

## 执行清单

- [x] `china.rs` 改 `china_db_path()` 三层兜底（env → exe 旁 `../data/china.sqlite` → manifest 相对 SFID 源）。
- [x] `cpms-ci.yml` push/pull_request paths 触发补 `sfid/backend/china/china.sqlite`。
- [x] `CPMS_TECHNICAL.md` 第 11 节补三层兜底说明。
- [x] `cargo fmt --check` OK + `cargo clippy --all-targets` 零警告 + `cargo test` 32 passed。
- [x] 验证第 3 层 dev 兜底从 `cpms/backend` manifest 解析命中真源 `/Users/rhett/GMB/sfid/backend/china/china.sqlite`（cargo run 启动期先连 postgres，china 定位逻辑已按路径解析验证）。

## 完成记录

- 2026-06-06：创建任务卡，开始执行。
- 2026-06-06：执行完成。
  - `china_db_path()` 三层兜底落地；本地 `cargo run` 不再需要手动设 `CPMS_CHINA_DB`，编译期 manifest 相对路径直指 SFID 唯一源。
  - 生产仍以 `CPMS_CHINA_DB` 为准，新增的 exe 旁 `../data/china.sqlite` 自定位让 env 丢失时也能找到 `/opt/cpms/data/china.sqlite`。
  - CPMS CI paths 触发纳入 china 数据文件；确认 monorepo 全量 checkout 自带该 88MB blob，CI 的 `cargo test` 与 `workflow_dispatch` 出包两条路径本就可用。
  - 文档同步。fmt/clippy/test 全绿。
