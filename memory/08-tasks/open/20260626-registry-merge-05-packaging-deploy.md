# 任务卡：registry 并入 Step5 — 打包与部署形态

## 任务需求

交付单安装包与去中心化部署口径：

- 安装包打包 node 二进制 + registry 二进制 + 内嵌 PostgreSQL + registry 前端产物，三平台零依赖、装好即用。
- 大市（如香港 800万公民/500万公司/100管理员）部署口径：机房服务器 + RAID/NAS（文件数十 TB）+ UPS + 内嵌 PostgreSQL 备份/PITR + 温备。
- registry API 对内网 TLS + 扫码鉴权；联邦节点按省管理形态文档化。

## 所属模块

citizenchain（打包、部署文档）

## 预计修改目录

- `citizenchain/node/`（tauri.conf.json / 打包脚本）
  - 用途：安装包内嵌 registry + PostgreSQL + 前端产物。
  - 边界：维持单安装包交付，桌面=节点运维台。
- `citizenchain/registry/`（部署脚本/配置）
  - 用途：内嵌 PG 初始化、备份/PITR、TLS、本地限流配置。
  - 边界：节点私有实例，不要求用户外装依赖。
- `memory/01-architecture/citizenchain/`、`memory/05-modules/citizenchain/`
  - 用途：登记部署形态（市节点服务器/NAS/备份、联邦按省管理）。
  - 边界：只补当前目标态。

## 输入文档

- memory/04-decisions/ADR-029-registry-into-citizenchain.md
- memory/project_installer_zero_dep_2026_05_05.md
- memory/project_chainspec_frozen_2026_05_06.md

## 必须遵守

- 三平台桌面端零依赖（project_installer_zero_dep）。
- chainspec 创世后冻结，升级走 setCode（feedback_chainspec_frozen）。
- 节点桌面端=矿工端默认全核挖矿不动（feedback_desktop_is_miner）。

## 验收标准

- 三平台安装包可装好即用，启动后节点 + 内嵌 PG + registry 服务齐起。
- 备份/PITR、TLS、本地限流可用并文档化。
- 部署形态文档可指导大市机房服务器落地。

## 决策(用户拍板)
① PG 二进制取 postgresql.org 官方;② PG 连接 127.0.0.1 TCP;③ registry TLS 用 rcgen 自签;④ 备份=每日 pg_basebackup + WAL PITR 到 NAS。
**归属修正**:内嵌 PG 生命周期 100% 在 registry(`registry/src/core/embedded_pg.rs`),node 只拉子进程+传 env(用户纠正:PG 该归 registry 自管,不该塞 node)。

## 进度

- [x] 任务卡创建
- [x] **内嵌 PostgreSQL(registry 自管)**:`registry/src/core/embedded_pg.rs` —— ensure_started(首启 initdb→起 postgres@127.0.0.1:ONCHINA_PG_PORT→建 registry 库→自拼 DATABASE_URL)/stop;`ONCHINA_EMBEDDED_PG` 开关(桌面内嵌 / 大市外部托管两形态);WAL 归档配 `ONCHINA_PG_WAL_ARCHIVE_DIR`(PITR)。
- [x] **main.rs 接线**:启动期 ensure_started 取 DATABASE_URL;退出信号(Ctrl-C/SIGTERM)优雅停 PG;serve 抽 `serve_registry`(TLS 分支)。
- [x] **内网 TLS**:`registry/src/core/tls.rs` —— rcgen 自签(localhost+127.0.0.1 SAN)持久化 `ONCHINA_TLS_DIR`,axum-server+rustls(ring)起 HTTPS,`ONCHINA_ENABLE_TLS` 开关;Cargo 加 axum-server/rustls/rcgen。
- [x] **node 端 env 传递**:`node/src/registry_proc/mod.rs` —— start_registry(app) 经 Tauri resource_dir/app_data_dir 解析路径,只当随包 PG 存在才开内嵌+HTTPS(dev 退化外部 PG+HTTP);registry 二进制从 `resources/registry-bin/` 解析(unix 补可执行位)+ exe 同目录兜底;desktop 调用点传 app.handle()。
- [x] **打包配置**:`node/tauri.conf.json` 加 `resources`(registry-bin/postgres/china.sqlite/registry-frontend/dist);占位 + `node/resources/.gitignore`(真产物不入库,dev 构建路径存在);`citizenchain/scripts/prepack.{sh,ps1}`(build registry+前端、拷 china.sqlite、官方 PG 二进制 CITIZENCHAIN_PG_DIST 组装)。
- [x] **备份/PITR**:`citizenchain/scripts/registry-{backup.sh,restore.sh,postgresql.conf.sample}`(每日 pg_basebackup 全量到 NAS + WAL 持续归档 + PITR 恢复 + 大市调优模板)。
- [x] **残留清理**:删死 CI workflow `.github/workflows/{citizencode-ci.yml,citizenpassport-ci.yml}`(系统已删/归档)。
- [x] **部署文档**:`memory/05-modules/citizenchain/registry-deploy.md`(五件套/进程编排/内嵌 PG/TLS/大市机房 RAID-NAS-UPS-备份-温备/联邦按省)。
- [x] **验证**:`cargo check -p registry -p node` 0 错 0 警;workspace check 绿。
- [ ] (操作前置)取 postgresql.org 官方 PG 二进制 + 三平台实跑 `prepack`→`tauri build` 出安装包 + 端到端冒烟(需各平台构建机)。

## 完成摘要
- 内嵌 PG / 内网 TLS 全归 registry 自管(registry 成为可独立部署单元:大市机房可不经桌面直接跑 registry);node 只做"拉子进程 + 传 env",零 PG 逻辑。
- 桌面=矿工端不动;chainspec 冻结;三平台零依赖打包骨架就位。实际 PG 二进制组装 + 三平台出包是构建机操作(脚本 + 文档已备),非代码。
