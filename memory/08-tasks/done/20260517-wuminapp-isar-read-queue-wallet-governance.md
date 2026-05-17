# wuminapp Isar 读写队列与钱包治理页修复

## 任务需求

- 修复低端 Android 上持续出现 `IsarError: MdbxError (11): Try again` 的问题。
- 修复创建/导入钱包显示助记词后，“我的钱包”列表仍显示为空的问题。
- 修复治理机构详情页因为本地钱包数据库 busy 而整页“加载失败”的问题。
- 修复后台 pending 交易对账与前台钱包/治理页面抢 Isar 的问题。

## 修改边界

- `wuminapp/lib/isar/`：收口 Isar 读写队列。
- `wuminapp/lib/wallet/`：钱包读写、创建/导入后校验、钱包页加载与余额刷新节流。
- `wuminapp/lib/transaction/shared/`：本地交易读路径与后台对账调度。
- `wuminapp/lib/governance/`：治理机构页本地钱包读取失败分流。
- `wuminapp/lib/main.dart`：后台对账启动时机和触发方式。
- `memory/01-architecture/wuminapp/`、`memory/05-modules/wuminapp/`：同步技术文档。

## 验收标准

- `WalletIsar.instance.db()` 不再被业务模块直接用于 Isar 读写。
- 业务读写统一通过 `WalletIsar.read()` / `WalletIsar.writeTxn()` 串行执行。
- 创建/导入钱包后返回“我的钱包”，新钱包必须稳定显示。
- 治理机构详情页不能因本地钱包读取 busy 整页失败。
- 真机 logcat 不再出现 `MdbxError`、`钱包数据库繁忙`、`对账触发失败`。

## 实施记录

- `WalletIsar` 新增全局业务读写队列、busy 重试和后台调度 busy 判定。
- 钱包核心、钱包页、本地交易、pending 投票、个人/机构多签发现、治理上下文等业务读写统一改为 `read()` / `writeTxn()`。
- 钱包创建/导入增加落库后复读校验；失败时回滚对应 `walletIndex` 的 Isar 记录、seed 和助记词。
- 交易 Tab 默认首屏对账延后执行；AppLockGate 周期对账和交易页对账在本地库 busy 时跳过本轮。
- 治理机构详情页和提案上下文把本地钱包 busy 分流为空管理员钱包，不再让链上机构内容整页失败。
- 已同步 `memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md`、钱包、治理、个人多签模块文档。

## 验证记录

- `flutter analyze lib test`：通过。
- `flutter build apk --debug --target-platform android-arm,android-arm64`：通过。
- `git diff --check`：通过。
- 残留扫描 `rg "WalletIsar\\.instance\\.db\\(" wuminapp/lib -g '*.dart'`：无业务残留。
- 已安装到真机 `moto g play - 2023`，包名 `org.chinanation.citizen`，`lastUpdateTime=2026-05-17 11:13:33`。
- 冷启动 35 秒 logcat 复查：未再命中 `IsarError`、`MdbxError`、`Try again`、`对账触发失败`、`Unhandled Exception`、`钱包数据库繁忙`。
