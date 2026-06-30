# OnChina 打包与去中心化部署形态

## 单安装包(三平台零依赖)

Tauri 打包(dmg/nsis/deb)随包"五件套",装好即用、无外部依赖:

| 件 | 位置 | 运行期 |
|----|------|--------|
| node 矿工端 | 主程序 | 桌面=节点运维台,默认全核挖矿 |
| OnChina 二进制 | `resources/onchina-bin/onchina` | 节点设置页二次确认后由 `onchina_proc` 拉起 |
| PostgreSQL 官方二进制 | `resources/postgres/<os>/`(bin/lib/share) | OnChina **自管**内嵌私有实例 |
| OnChina 前端产物 | `resources/onchina-frontend/dist` | OnChina 同源托管(`ONCHINA_FRONTEND_DIST`) |
| china.sqlite | `resources/china.sqlite` | 行政区只读单源(`ONCHINA_CHINA_DB`) |

打包流程:`citizenchain/scripts/prepack.{sh,ps1}` 组装(build onchina+前端、拷 china.sqlite、把官方 PG 二进制 `CITIZENCHAIN_PG_DIST` 拷进 resources)→ 在 `node/` 跑 `npm run tauri build`。
PG 官方二进制来源:https://www.postgresql.org/download/(解压后含 bin/lib/share)。

## 进程编排(设置页手动拉起,OnChina 自管 PG/TLS)

- 节点 `desktop` setup 不启动 OnChina；用户在设置页“链上中国平台”行点击“启动”并二次确认后，`start_onchina_platform` 调用 `onchina_proc::start_onchina(app)`。
- `onchina_proc` 用 env 把资源/数据路径告诉 OnChina(`ONCHINA_PG_BIN_DIR`/`ONCHINA_PG_DATA_DIR`/`ONCHINA_PG_PORT`/`ONCHINA_TLS_DIR`/`ONCHINA_PG_WAL_ARCHIVE_DIR`/`ONCHINA_FRONTEND_DIST`/`ONCHINA_CHINA_DB`/`ONCHAIN_WS_URL`/`ONCHINA_EMBEDDED_PG=1`/`ONCHINA_ENABLE_TLS=1`)。
- OnChina 启动:`embedded_pg::ensure_started()`(首启 initdb→起 postgres@127.0.0.1:私有端口→建 onchina 库→自拼 DATABASE_URL)→ schema 幂等建 → `tls`(机构私有 CA 签发 HTTPS,主机 `onchina.local`)→ 服务。
- 退出:node 停子进程信号 → OnChina 收 SIGTERM/Ctrl-C → `embedded_pg::stop()` → 退出。node **不碰 PG**。
- 开发期(无随包 PG)继承 `run.sh` / `clean-run.sh` 注入的外部 PostgreSQL 二进制、数据目录、前端产物和 HTTPS 配置。

## 内网 TLS + 扫码鉴权

- OnChina 内网 API 固定入口为 `https://onchina.local:8964`，服务监听 `0.0.0.0:8964` 并通过 mDNS 广告 `onchina.local`。
- OnChina 内网 API 走 HTTPS(机构私有 CA 签发,证书持久化 `ONCHINA_TLS_DIR`);CA 有效期到 2036-01-01,服务证书每次启动重签且有效期 397 天以内,证书 SAN 为 `onchina.local`。
- 身份认证 = 扫码签名(3b 链上 Active 管理员集合鉴权),TLS 只负责传输加密。与 node 的 libp2p WSS 证书相互独立。

## 大市机房形态(如香港:800万公民/500万公司/百管理员)

- 机房服务器 + RAID/NAS(数十 TB:法人照片、档案材料)+ UPS。
- 数据库两选:① 内嵌私有 PG(`ONCHINA_EMBEDDED_PG=1`,OnChina 自管);② 外部托管 PG(关 `ONCHINA_EMBEDDED_PG`,直接给 `DATABASE_URL`;调优参考 `citizenchain/scripts/onchina-postgresql.conf.sample`)。
- **备份/PITR**:`ONCHINA_PG_WAL_ARCHIVE_DIR` 指向 NAS → 持续 WAL 归档;`citizenchain/scripts/onchina-backup.sh` cron 每日 `pg_basebackup` 全量落 NAS(默认保留 14 份);`citizenchain/scripts/onchina-restore.sh` 做 PITR 恢复(可指定 `RECOVERY_TARGET_TIME`)。温备:NAS + 第二台服务器持全量 + WAL,故障切换。
- 联邦节点**按省管理**:每市自治节点跑自己的 OnChina+PG;联邦注册局按省给市配管理员(链上,3a/3b)。
- **联邦注册局(FRG)每节点单省部署**(2026-06-29 起):本节点所辖省由首次 active admin 登录后绑定的 FRG 省组确定;
  管理员成员资格「全走链读」链上 `GenesisAdmins::FederalRegistryProvinceGroups[绑定省码]`(见 [[project_onchina_registry_tier_chainread_2026_06_29]])。
  `federal_registry_scope`/`provinces` 本地投影表 + `seed-federal-admins` CLI **已退役**——不再播种、不再以本地表作省映射真源。
  FRG 节点不再要求安装前配置 `ONCHAIN_CREDENTIAL_SCOPE_PROVINCE_NAME`;未绑定时由冷钱包管理员登录后确认本节点省组。

## 约束(已遵守)

- 三平台桌面端零依赖([[project_installer_zero_dep_2026_05_05]]);chainspec 创世后冻结、升级走 setCode([[feedback_chainspec_frozen]]);桌面=矿工端全核挖矿不动([[feedback_desktop_is_miner]])。
