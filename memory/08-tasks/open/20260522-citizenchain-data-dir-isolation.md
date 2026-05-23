# 区块链软件正式版与开发版数据目录隔离

## 任务需求

将区块链软件正式版和开发版的本地数据库彻底隔离，避免两个进程同时打开同一份 RocksDB 导致 `LOCK: Resource temporarily unavailable`。目标路径：

- 正式版：`~/Library/Application Support/gmb/chains/citizenchain/db/full`
- 开发版：`~/Library/Application Support/gmb.dev/chains/citizenchain/db/full`

改完后删除旧数据库，区块链软件重启后重新从区块链网络同步数据。

## 影响范围

- `citizenchain/node/src/shared/`：改 App 数据目录解析和节点 base path。
- `citizenchain/scripts/`：开发启动脚本显式使用开发数据目录，clean-run 清理开发数据库。
- `memory/05-modules/citizenchain/node/`：同步本地数据目录技术说明。

## 执行规则

- 正式版与开发版必须完全隔离本地 App 数据。
- 开发版默认使用 `gmb.dev`，正式版默认使用 `gmb`。
- 不再使用旧的 `org.chinanation.citizenchain.desktop/node-data` 作为节点数据库路径。
- 删除旧数据库前必须确认并停止仍在占用旧库的 citizenchain 进程。

## 执行记录

- 2026-05-22：创建任务卡，准备修改数据目录规则和脚本。
- 2026-05-22：已修改 `shared/security.rs`，通过 `CITIZENCHAIN_DATA_PROFILE` 与 debug/release 默认值选择 `gmb.dev` / `gmb`。
- 2026-05-22：已修改 `shared/keystore.rs`，节点 `base-path` 不再追加 `node-data`，区块库落到 `<app_data>/chains/citizenchain/db/full`。
- 2026-05-22：已修改开发脚本，`run.sh` 与 `clean-run.sh` 固定使用开发版数据目录 `gmb.dev`，`clean-run.sh` 只清开发版 `chains/citizenchain/db`。
- 2026-05-22：已同步首页、keystore、GRANDPA 技术文档中的数据目录说明。
- 2026-05-22：已通过 `cargo check --manifest-path citizenchain/node/Cargo.toml --bin citizenchain` 与 `git diff --check`。
- 2026-05-22：已确认无 citizenchain 进程占用旧库，并删除旧区块数据库 `~/Library/Application Support/org.chinanation.citizenchain.desktop/node-data/chains/citizenchain/db`。
- 2026-05-22：按用户要求彻底删除旧数据根目录 `~/Library/Application Support/org.chinanation.citizenchain.desktop`，包括旧钱包缓存、旧审计日志、旧 node-data 和临时残留文件。
