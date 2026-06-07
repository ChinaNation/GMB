# 任务卡：CPMS 后端去 src/ 壳平铺 + 行政区真源重对接 china.sqlite

## 任务需求

让 CPMS 前后端目录结构对齐：删除 `cpms/backend/src/` 这一层源码壳，把源码直接平铺到 `cpms/backend/` 根（与前端 `cpms/frontend/` 平铺一致，照搬 SFID 后端已落地的无 src/ 布局）。同时修复一处既有编译中断：CPMS 后端 `#[path]` 引用的 SFID 静态表 `sfid/backend/sfid/province.rs` 已被删除（SFID 行政区真源已迁至 `china/` 的 SQLite），CPMS 当前 `cargo check` 直接报错；本次把行政区引用重对接到 `china.sqlite`。两件事撞在 `main.rs` 同一行 `#[path]` 上，必须一起做。

## 背景结论（分析已确认）

- SFID 后端早已删除 `backend/src/` 壳，源码以 `sfid/backend/` 为根，配方见 `memory/05-modules/sfid/backend/BACKEND_LAYOUT.md`；CPMS 是唯一残留 src/ 壳的异类。
- `cpms/backend/src/main.rs:30` 的 `#[path = "../../../sfid/backend/sfid/province.rs"]` 指向文件已被删除 → CPMS 后端当前编译不过。
- 行政区新真源：`sfid/backend/china/`（rusqlite + `china/data/china.sqlite`，84MB），`china::store::provinces()` 静态结构体只到镇；村数据在 `china.sqlite` 的 `villages` 表（716,219 行，列 province_code/city_code/town_code/code/name），SFID Rust API 未暴露村。
- CPMS `address.rs` 需要镇+村，遍历到 `town.villages`，落 `address_towns/address_villages`。
- `address.rs` 顶部既有设计注释：发行版只内置只读数据、运行时只启用安装码对应市、不维护第二套行政区源 → 选自包含离线方案：安装包随附 china.sqlite，CPMS 用 rusqlite 读，零改 SFID。

## 建议模块

- CPMS 后端目录平铺（`cpms/backend/`）
- CPMS 后端 `address`（行政区重对接 rusqlite）
- CPMS 安装器与部署（`cpms/scripts` / `cpms/deploy`）
- CPMS 技术文档（`cpms/CPMS_TECHNICAL.md`）
- memory 文档回写（路径表中的 `cpms/backend/src/...`）

## 影响范围

- `cpms/backend/src/*` → `cpms/backend/*`：`git mv` 平移全部源码（main.rs、address.rs、rate_limit.rs、ss58.rs、authz/、dangan/、initialize/、login/、number/、qr/、store/、super_admin/），删空 `src/` 目录。涉及代码移动。
- `cpms/backend/Cargo.toml`：新增 `[[bin]] name="cpms-backend" path="main.rs"`（SFID 同款）。涉及配置。
- `cpms/backend/main.rs`：删除 `#[path = "../../../sfid/backend/sfid/province.rs"] mod sfid_tool_province;` 整段；行政区读取改走新 rusqlite 模块。涉及代码 + 残留清理。
- `cpms/backend/address.rs`：删 `use sfid_tool_province::{...}`；`find_install_city` / `replace_city_address` 改为查 china.sqlite（province/city by code、towns by city、villages by town）。涉及代码。
- `cpms/backend/Cargo.toml`：新增 `rusqlite = { version = "0.32", features = ["bundled"] }`（与 SFID 同版本）。涉及配置。
- `cpms/backend/` 新增行政区读取模块（命名待定，建议 `china.rs` 或 `address_source.rs`），封装 china.sqlite 路径解析（env `CPMS_CHINA_DB`，默认 `./china.sqlite`，对齐现有 `CPMS_FRONTEND_DIR` 约定）与镇/村查询。涉及代码。
- `cpms/scripts/build_linux_host_installer.sh`：payload 新增 `db/` 或 `data/`，`cp sfid/backend/china/data/china.sqlite` 进发行包；manifest/校验同步。涉及部署。
- `cpms/deploy/linux/systemd/cpms-backend.service` + `install_host.sh`：设置 `CPMS_CHINA_DB` 指向部署后路径。涉及部署。
- `cpms/CPMS_TECHNICAL.md`：11 处 `src/...` 路径表改为去 src/ 后路径；`sfid_tool_province` 行改为 china.sqlite 重对接说明。涉及文档。
- memory 路径回写：`memory/07-ai/unified-protocols.md`、`memory/07-ai/unified-naming.md`、`memory/05-modules/wuminapp-vs-wumin.md`、`memory/05-modules/cpms/backend/**` 中 `cpms/backend/src/...` → `cpms/backend/...`。涉及文档。

## 主要风险点

- `#[path]` 平移后相对层级会变，但本次直接删除该 mod 改 rusqlite，不再保留 `#[path]`，规避层级陷阱。
- Cargo 模块解析依赖 `main.rs` 所在目录；平移后必须确认 `mod dangan;` 等仍解析到 `cpms/backend/dangan/mod.rs`（SFID 已验证此布局可行）。
- 安装包 +84MB（china.sqlite）。属设计注释明确的"发行版内置只读数据"，且比旧"编译期嵌入二进制"更轻；china.sqlite 仅安装/同步时打开，稳态运行不常驻内存。
- 不得在 CPMS 维护第二套行政区"源"：china.sqlite 是 SFID 唯一源在构建期的随附拷贝，不在 CPMS 侧编辑/维护。
- rusqlite bundled 会引入 C 编译；需确认 CPMS Linux 发行构建链支持（SFID 已用同款，风险低）。
- 必须 `cargo check` 转绿才算修复既有中断；这是验收硬指标。
- `git mv` 保留历史，禁止删了重建丢历史。

## 是否需要先沟通

- 方向与范围已与用户确认：删后端 src/ 壳 + 行政区重对接，两件一起做。
- 84MB 随附为设计注释支持的既定方向，采用自包含离线方案，零改 SFID。

## 执行清单

- [x] `git mv` 把 `cpms/backend/src/*` 平移到 `cpms/backend/`，删空 src/。
- [x] `Cargo.toml` 加 `[[bin]] path="main.rs"` + `rusqlite`（bundled）。
- [x] main.rs 删 `sfid_tool_province` mod 与 `#[path]`，改 `mod china;`。
- [x] 新增 `china.rs` 读取模块（路径 env `CPMS_CHINA_DB` + 镇/村窄查询，村按市单查后归组）。
- [x] address.rs 重写 `find_install_city` / `replace_city_address` 走 rusqlite，含 villages。
- [x] 安装器随附 china.sqlite（payload/data），install_host 装到 `/opt/cpms/data` 并设 `CPMS_CHINA_DB`，uninstall 同步清理。
- [x] `cargo fmt --check && cargo clippy --all-targets`（零警告）+ `cargo test`（32 passed）。
- [x] 更新 `CPMS_TECHNICAL.md`（模块表 + 第 11 节行政区）与 CPMS 相关 memory 文档路径。
- [x] 验收：`test ! -d cpms/backend/src` 通过；活跃代码/文档 `sfid_tool_province`/`cpms/backend/src` 残留归零。

## 完成记录

- 2026-06-06：分析完成（目录对比 + SFID 无 src/ 先例 + 既有编译中断定位 + china.sqlite villages 确认），创建任务卡。
- 2026-06-06：执行完成。
  - 后端去 src/ 壳：`git mv` 12 个条目上提到 `cpms/backend/`，`Cargo.toml` 加显式 `[[bin]] path="main.rs"`（照搬 SFID 配方）。
  - 修复既有编译中断：删悬空 `#[path = "../../../sfid/backend/sfid/province.rs"]`；新增 `china.rs` 用 rusqlite 只读 `china.sqlite`（`CPMS_CHINA_DB`，默认 `./china.sqlite`），address.rs 改查 china 真源（省市名还原 + 该市镇/村）。零改 SFID。
  - 顺带修两处随源码移动而失效的 `include_str!("../../db/schema.sql")` → `../db/schema.sql`。
  - 安装链路：build 脚本拷 `sfid/backend/china/data/china.sqlite`（84MB）入 payload/data 并校验；install_host 装到 `/opt/cpms/data/china.sqlite`、env 写 `CPMS_CHINA_DB`（老安装升级 grep 补齐）；uninstall 清理。
  - 测试：initialize 两条 install QR 测试因行政区校验改走 sqlite，新增 `point_to_china_source()` 指向 SFID 唯一源、缺失则跳过；32 passed。
  - 文档：CPMS_TECHNICAL.md + unified-protocols/unified-naming/wuminapp-vs-wumin + 4 个 cpms/backend 模块 TECHNICAL 路径回写。
- 遗留（不在本卡范围）：约 15 张 2026-05-30 批次 open 任务卡仍写 `cpms/backend/src/...` 且引用已不存在的 `operator_admin`（现为 `super_admin`），属各自任务的陈旧规划文档，需各卡自行对账，未在本卡改动。
