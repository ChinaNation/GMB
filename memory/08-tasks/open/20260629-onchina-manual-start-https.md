# OnChina 手动启动与 HTTPS 统一入口

## 任务需求

- 统一链上中国平台访问入口为 `https://onchina.local:8964`。
- 节点程序启动后默认不启动链上中国平台。
- 在节点设置页“全节点模式”和“通信节点功能”之间新增“链上中国平台”启动行。
- 用户点击“启动 / 关闭”后必须二次确认，确认后才启动或停止链上中国平台。
- 启动成功后不自动打开浏览器，由局域网内管理员自行访问固定入口。
- 登录签名响应失败必须返回可区分的登录错误码，并由前端显示对应中文提示。

## 影响范围

- `citizenchain/node` 桌面端 OnChina 子进程生命周期。
- `citizenchain/node` 设置页 UI 与 Tauri 命令。
- `citizenchain/onchina` HTTPS 证书域名与统一入口文案。
- `citizenchain/onchina` 登录错误码映射与前端中文提示。
- `memory/` 相关部署与架构文档。

## 风险点

- 其他电脑首次访问自签 HTTPS 仍可能需要信任证书；本任务只统一服务入口和证书域名。
- 不能恢复 HTTP 作为正式入口。
- 不能让 OnChina 随节点默认启动，避免只挖矿节点承担不必要服务。

## 执行计划

1. 移除桌面端启动时自动拉起 OnChina。
2. 新增 OnChina 平台状态查询与手动启动命令。
3. 设置页新增链上中国平台启动行、状态标签、启动 / 关闭按钮和二次确认。
4. OnChina TLS 证书覆盖 `onchina.local`。
5. 拆分登录错误码映射，前端按错误码显示中文提示。
6. 更新文档、完善中文注释、清理旧 HTTP 文案残留。

## 验收标准

- 节点程序启动后 OnChina 默认不启动。
- 设置页显示 `链上中国平台`、`未开启 / 已开启` 状态标签、`https://onchina.local:8964` 和 `启动 / 关闭` 按钮。
- 点击启动或关闭弹出二次确认。
- 确认后 OnChina 子进程启动或停止，状态标签同步更新。
- 退出节点程序后 OnChina 子进程被清理。
- 登录签名响应失败时显示明确中文原因，不再统一掉到“登录签名响应处理失败”。
- 文档同步说明 HTTPS 统一入口和手动启动行为。

## 执行记录

- 已移除 `citizenchain/node/src/desktop/mod.rs` 中节点启动阶段自动调用 `onchina_proc::start_onchina` 的逻辑。
- 已新增 `citizenchain/node/src/settings/onchina_platform/mod.rs`，提供 `get_onchina_platform` / `start_onchina_platform` / `stop_onchina_platform` Tauri 命令。
- 已新增 `citizenchain/node/frontend/settings/OnChinaPlatformSection.tsx`，设置页在“全节点模式”和“通信节点功能”之间显示状态标签、固定 HTTPS 入口和启动 / 关闭按钮。
- 已将 OnChina 自签 TLS 目标主机收敛为 `onchina.local`，并用 `onchina-cert-host.txt` 标记触发旧证书再生成。
- 已将登录签名响应、挑战、链上管理员鉴权等错误映射为 `CID_LOGIN_*` 错误码，并在前端 notice 中补齐中文提示。
- 已更新 `run.sh` / `clean-run.sh`，本地开发脚本准备 HTTPS 环境但仍要求在设置页手动启动平台。
- 已同步更新架构文档、节点技术文档、ADR-030 和部署形态文档。

## 验收记录

- `npm --prefix citizenchain/node/frontend run build`：通过。
- `npm --prefix citizenchain/onchina/frontend run build`：在只包含本窗口 `notice.ts` 改动的临时干净 worktree 中通过，并刷新 OnChina 前端打包产物。
- `cargo check --manifest-path citizenchain/Cargo.toml -p node`：通过。
- `cargo check --manifest-path citizenchain/Cargo.toml -p onchina`：通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p onchina login -- --nocapture`：通过，当前筛选下 0 个测试执行、72 个测试被过滤，编译与测试入口通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p onchina passkey -- --nocapture`：通过，3 个 passkey 相关测试通过。
- `git diff --check`：通过。
- 旧入口扫描：`rg "http://onchina\\.local:8964|http://127\\.0\\.0\\.1:8964"` 已确认代码和记忆文档中不再保留旧正式入口文案。
- 未执行完整 Tauri 桌面真实运行态验收：该命令会在当前机器启动真实区块链节点与挖矿进程，本次先以构建、Rust check 和静态入口扫描收口；后续如需真机验收，应在可接受启动本机节点的窗口中执行。
