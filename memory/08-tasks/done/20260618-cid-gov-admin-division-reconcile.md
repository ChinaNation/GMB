# 任务卡：CID 公权机构随行政区变更自动对账

## 任务目标

行政区开发库 `citizencode/backend/china/china.sqlite` 变更后，CID 运行库中的确定性公权机构必须同步变化：

- `gov_manifest` 必须记录并校验当前 `china.sqlite` hash。
- `ensure-gov` 不能只按数量判断已初始化，必须发现行政区 hash/目录 hash 变化。
- `serve` 启动时必须守卫公权机构目录版本，防止旧运行库继续对外服务。
- 部署脚本安装新版 `china.sqlite` 后必须先执行公权机构对账和严格校验，再启动服务。
- 自动派生公权机构与手动公权机构要有边界，自动对账不得误删手动机构。

## 预计修改目录

- `citizencode/backend/gov/`：公权机构确定性目标目录、hash 校验、对账删除边界。
- `citizencode/backend/main.rs`：`ensure-gov`、`serve` 启动守卫、维护命令日志。
- `citizencode/backend/core/db.rs`：`gov.source` schema 收敛，区分 `GENERATED` 与 `MANUAL`。
- `citizencode/deploy/prod/`：部署脚本强制执行 `reconcile-gov --changed-only` 和 `check-gov --strict`。
- `memory/`：更新 ADR、CID 技术文档、部署文档和任务记录。

## 验收要求

- 行政区 hash 变化时，`ensure-gov` 不得跳过。
- `serve` 在公权目录过期时拒绝启动；本地显式开启自动对账时可先同步再启动。
- 自动派生公权机构写入 `gov.source='GENERATED'`，手动创建写入 `MANUAL`。
- obsolete 删除只作用于 `GENERATED` 公权机构。
- 当前本机 CID 运行库对账后不得残留 `HU/107`、`管理市` 或旧 `gov_manifest.china_hash`。

## 执行记录

- 2026-06-18：按用户确认开始执行。
- 2026-06-18：`gov` 表新增并收敛 `source` 字段:
  - `GENERATED`:行政区和模板确定性派生的公权机构/公安局,允许对账更新或删除。
  - `MANUAL`:管理员手动创建的公权机构,行政区对账不得当作 obsolete 删除。
- 2026-06-18：`ensure-gov` 改为先执行目录校验和 manifest 校验;目录完整但 manifest 过期时只修复 manifest,目录缺失/错配/缺账户/obsolete 时执行对账。
- 2026-06-18：`serve` 启动前校验全局 `gov_manifest`;生产目录过期直接拒绝启动,本地显式设置 `ONCHINA_GOV_AUTO_RECONCILE=1` 才允许启动前自动对账。
- 2026-06-18：`reconcile-gov --changed-only` 增加全局 `all:all` manifest 刷新。省级对账后若全局仍不一致,自动执行一次全局对账。
- 2026-06-18：生产安装脚本在 systemd 重启前强制执行:
  - `citizencode-backend reconcile-gov --changed-only`
  - `citizencode-backend check-gov --strict`
- 2026-06-18：本机真实 CID PostgreSQL 已完成运行库对账:
  - 首轮省级对账:`inserted=2024 updated=239467 account_inserted=483020 removed=268662`
  - 全局收敛:`inserted=0 updated=249413 account_inserted=498870 removed=6983`
  - `check-gov --strict`: `ok=true manifest_current=true target_count=249413 active_count=249413 missing=0 mismatched=0 missing_accounts=0 obsolete=0`
  - `catalog_hash=68bc1fe7522ce5e2b3e063a3c7af1e79bbd1e75451009366654ce49844a8e2ad`
  - `china.sqlite SHA-256=20b99f029d8d72fabc368ae24f39970e424a25255f103c82199a13c404cef800`
- 2026-06-18：关键 SQL 抽查:
  - `HU/107` subjects/gov/accounts 均为 `0`
  - `HU/106 洪江市` 活跃派生公权机构 `147`
  - `管理市` subjects 残留 `0`
  - `gov.source='GENERATED'` 总数 `249413`,`MANUAL` 总数 `0`
  - `gov_manifest all:all` hash 与当前 `china.sqlite` SHA-256 一致,`status=OK`
- 2026-06-18：真实启动验收:`ONCHINA_BIND_ADDR=127.0.0.1:0 citizencode-backend serve`
  启动通过,日志确认 `cid gov directory manifest matches current china sqlite` 并进入监听。
- 2026-06-18：文档和注释已同步更新:
  - ADR-021 发布流程改为“对账 + strict 校验 + 导出公权机构包”
  - CID 技术架构补充 `serve` 守卫、`gov.source` 边界和部署命令
  - subjects/china/生产部署文档补充自动目录联动和验收口径
  - 清理旧的“只缺失初始化”“临时服务 ensure-gov 导出”等残留口径

## 验收结果

- `cd citizencode/backend && cargo fmt && cargo check` 通过。
- `cd citizencode/backend && cargo test` 通过:65 passed。
- `python3 citizencode/backend/china/check_code_immutable.py` 通过。
- `ONCHINA_BIND_ADDR=127.0.0.1:0 ./target/debug/citizencode-backend serve` 通过 manifest 守卫并进入监听,随后手动停止验收进程。
- `git diff --check` 通过。
- `git diff --name-only -- citizenchain/runtime citizenchain/primitives/china` 无输出,本任务未触碰链端 runtime/primitives。
