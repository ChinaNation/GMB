# registry 打包与去中心化部署形态(ADR-029 Card 05)

## 单安装包(三平台零依赖)

Tauri 打包(dmg/nsis/deb)随包"五件套",装好即用、无外部依赖:

| 件 | 位置 | 运行期 |
|----|------|--------|
| node 矿工端 | 主程序 | 桌面=节点运维台,默认全核挖矿 |
| registry 二进制 | `resources/registry-bin/registry` | node 子进程拉起(`registry_proc`) |
| PostgreSQL 官方二进制 | `resources/postgres/<os>/`(bin/lib/share) | registry **自管**内嵌私有实例 |
| registry 前端产物 | `resources/registry-frontend/dist` | registry 同源托管(`REGISTRY_FRONTEND_DIST`) |
| china.sqlite | `resources/china.sqlite` | 行政区只读单源(`CID_CHINA_DB`) |

打包流程:`citizenchain/scripts/prepack.{sh,ps1}` 组装(build registry+前端、拷 china.sqlite、把官方 PG 二进制 `CITIZENCHAIN_PG_DIST` 拷进 resources)→ 在 `node/` 跑 `npm run tauri build`。
PG 官方二进制来源:https://www.postgresql.org/download/(解压后含 bin/lib/share)。

## 进程编排(node 拉起,registry 自管 PG/TLS)

- node `desktop` setup:`registry_proc::start_registry(app)` → 用 env 把资源/数据路径告诉 registry(`CID_PG_BIN_DIR`/`CID_PG_DATA_DIR`/`CID_PG_PORT`/`CID_TLS_DIR`/`CID_PG_WAL_ARCHIVE_DIR`/`REGISTRY_FRONTEND_DIST`/`CID_CHINA_DB`/`CID_CHAIN_WS_URL`/`CID_EMBEDDED_PG=1`/`CID_ENABLE_TLS=1`)。
- registry 启动:`embedded_pg::ensure_started()`(首启 initdb→起 postgres@127.0.0.1:私有端口→建 registry 库→自拼 DATABASE_URL)→ schema 幂等建 → `tls`(rcgen 自签 HTTPS)→ 服务。
- 退出:node 停子进程信号 → registry 收 SIGTERM/Ctrl-C → `embedded_pg::stop()` → 退出。node **不碰 PG**。
- 开发期(无随包 PG)自动退化为外部 `DATABASE_URL` + HTTP。

## 内网 TLS + 扫码鉴权

- registry 内网 API 走 HTTPS(rcgen 自签,证书持久化 `CID_TLS_DIR`);内网客户端首次信任自签证书。
- 身份认证 = 扫码签名(3b 链上 Active 管理员集合鉴权),TLS 只负责传输加密。与 node 的 libp2p WSS 证书相互独立。

## 大市机房形态(如香港:800万公民/500万公司/百管理员)

- 机房服务器 + RAID/NAS(数十 TB:法人照片、档案材料)+ UPS。
- 数据库两选:① 内嵌私有 PG(`CID_EMBEDDED_PG=1`,registry 自管);② 外部托管 PG(关 `CID_EMBEDDED_PG`,直接给 `DATABASE_URL`;调优参考 `citizenchain/scripts/registry-postgresql.conf.sample`)。
- **备份/PITR**:`CID_PG_WAL_ARCHIVE_DIR` 指向 NAS → 持续 WAL 归档;`citizenchain/scripts/registry-backup.sh` cron 每日 `pg_basebackup` 全量落 NAS(默认保留 14 份);`citizenchain/scripts/registry-restore.sh` 做 PITR 恢复(可指定 `RECOVERY_TARGET_TIME`)。温备:NAS + 第二台服务器持全量 + WAL,故障切换。
- 联邦节点**按省管理**:每市自治节点跑自己的 registry+PG;联邦注册局按省给市配管理员(链上,3a/3b)。

## 约束(已遵守)

- 三平台桌面端零依赖([[project_installer_zero_dep_2026_05_05]]);chainspec 创世后冻结、升级走 setCode([[feedback_chainspec_frozen]]);桌面=矿工端全核挖矿不动([[feedback_desktop_is_miner]])。
