# OnChina 手动启动与 HTTPS 统一入口

> 2026-07-05 追加：通信节点设置入口已删除；当前设置页中链上中国平台入口位于“全节点模式”之后，不再描述为位于“通信节点功能”之前或之间。

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

- 其他电脑首次访问机构私有 CA 签发的 HTTPS 时，需要在浏览器中下载并安装本节点 CA 公钥证书。
- 不能恢复 HTTP 作为正式入口。
- 不能让 OnChina 随节点默认启动，避免只挖矿节点承担不必要服务。

## 执行计划

1. 移除桌面端启动时自动拉起 OnChina。
2. 新增 OnChina 平台状态查询与手动启动命令。
3. 设置页新增链上中国平台启动行、状态标签、启动 / 关闭按钮和二次确认。
4. OnChina 首启生成机构私有 CA，并用该 CA 签发 `onchina.local` 服务证书。
5. 拆分登录错误码映射，前端按错误码显示中文提示。
6. 更新文档、完善中文注释、清理旧 HTTP 文案残留。

## 验收标准

- 节点程序启动后 OnChina 默认不启动。
- 设置页显示 `链上中国平台`、`未开启 / 启动中 / 已开启` 状态标签、`https://onchina.local:8964` 和 `启动 / 关闭` 按钮。
- 点击启动或关闭弹出二次确认。
- 确认后 OnChina 子进程启动或停止；只有 `/api/v1/health` 返回 `UP` 后状态标签才显示 `已开启`，进程存在但健康检查未通过时显示 `启动中`。
- 点击启动时先清理上一轮异常退出后遗留的旧 OnChina 孤儿进程和 8964 监听；如果端口被非 OnChina 进程占用，必须明确显示 `启动失败`，不得误杀其它进程。
- `启动中` 不显示红色失败详情；只有启动动作最终失败、子进程退出或健康检查超时后，才显示 `启动失败` 和具体原因。
- 退出节点程序后 OnChina 子进程被清理。
- 登录签名响应失败时显示明确中文原因，不再统一掉到“登录签名响应处理失败”。
- 文档同步说明 HTTPS 统一入口和手动启动行为。

## 执行记录

- 已移除 `citizenchain/node/src/desktop/mod.rs` 中节点启动阶段自动调用 `onchina_proc::start_onchina` 的逻辑。
- 已新增 `citizenchain/node/src/settings/onchina_platform.rs`，提供 `get_onchina_platform` / `start_onchina_platform` / `stop_onchina_platform` Tauri 命令。
- 已新增 `citizenchain/node/frontend/settings/OnChinaPlatformSection.tsx`，设置页在“全节点模式”和“通信节点功能”之间显示状态标签、固定 HTTPS 入口和启动 / 关闭按钮。
- 已把设置页状态改为真实健康状态：`已开启` 只代表 OnChina 进程存活且健康接口返回 `UP`，不再把“刚点启动/仅有进程句柄”误判为已开启。
- 已补齐节点解绑 / 换机构安全闭环：`NODE_BINDING_UNBIND` 复用现有管理员安全动作 prepare/commit，要求本机会话管理员 + 冷钱包 active admin 签名确认；commit 后停用 active binding 并清退管理员会话。
- 已修正前端管理员安全动作鉴权档映射：从旧两档字符串改为后端真实三档 `SESSION / PASSKEY / PASSKEY_COLD_SIGN`，避免冷签动作被前端旧字符串误拒。
- 已将 OnChina TLS 目标收敛为机构私有 CA 模式：本节点生成 `onchina-org-root-ca.crt/.key`，服务证书固定覆盖 `onchina.local`，并用 `onchina-cert-host.txt` 标记触发旧证书再生成。
- 已新增未登录可访问的机构 CA 下载接口 `/api/v1/platform/ca-certificate` 与信息接口 `/api/v1/platform/ca-certificate/info`，员工可在 OnChina 登录页直接下载 CA 公钥证书，CA 私钥不通过 HTTP 暴露。
- 已在 OnChina 登录页增加机构 CA 证书安装提示；未受信任 HTTPS 环境下，摄像头扫码和 passkey 均提示先安装机构 CA，不再误报为单纯摄像头权限问题。
- 已将机构 CA 证书安装提示同步扩展到登录后的后台顶部，避免自动恢复登录态或已登录用户看不到下载证书入口。
- 已修复 macOS 证书兼容性：旧 rcgen 超长期默认有效期证书会因缺少新策略标记自动重建；CA 有效期固定到 2036-01-01，服务证书每次启动重签且有效期 397 天以内。
- 已将登录签名响应、挑战、链上管理员鉴权等错误映射为 `ONCHINA_LOGIN_*` 错误码，并在前端 notice 中补齐中文提示。
- 已完成 OnChina 平台环境变量与错误码命名清理：平台本地配置统一用 `ONCHINA_*`，链连接与链上凭证配置统一用 `ONCHAIN_*` / `ONCHAIN_CREDENTIAL_*`，登录、鉴权、绑定、管理员和通用 API 错误码统一用 `ONCHINA_*`。
- 已修正 OnChina 控制台能力映射：FRG 按 runtime 目标状态作为 CREG 能力超集，CREG 保留本市业务能力并新增“联邦注册局”tab 的本省只读查看能力。
- 已修正前端注册局 passkey 入口：未设置 passkey 的 FRG/CREG 管理员只显示自己机构管理员列表入口；设置完成后按后端能力表显示完整业务 tab。
- 已修正联邦注册局管理员列表：FRG 进入时显示本省编辑/更换/passkey 操作，CREG 进入时只显示本省联邦注册局管理员只读表格，不显示操作列。
- 已更新 `run.sh` / `clean-run.sh`，本地开发脚本准备 HTTPS 环境但仍要求在设置页手动启动平台。
- 2026-07-18:已修正节点设置页启动链上中国平台的旧进程处理和状态展示：`start_onchina_platform` 调用前清理旧 OnChina 孤儿进程 / 8964 端口监听；健康检查暂未通过时只返回 `启动中` 且不携带失败详情；启动失败、子进程退出或健康检查超时才返回 `启动失败`。
- 已同步更新架构文档、节点技术文档、ADR-030 和部署形态文档。

## 验收记录

- `npm --prefix citizenchain/node/frontend run build`：通过。
- `npm --prefix citizenchain/onchina/frontend run build`：在只包含本窗口 `notice.ts` 改动的临时干净 worktree 中通过，并刷新 OnChina 前端打包产物。
- `npm --prefix citizenchain/onchina/frontend run build`：命名清理后再次通过，并刷新 `dist/assets/index-*.js`，旧错误码构建产物残留清零。
- `npm --prefix citizenchain/onchina/frontend run build`：机构 CA 下载提示和 passkey 安全上下文修复后通过，并刷新 `dist/assets/index-*.js`。
- `npm --prefix citizenchain/onchina/frontend run build`：登录后后台顶部证书提示修复后通过，并刷新 `dist/assets/index-*.js`。
- `npm --prefix citizenchain/onchina/frontend run build`：FRG/CREG 能力映射、passkey 入口和 CREG 联邦注册局只读 tab 修复后通过，并刷新 `dist/assets/index-*.js`。
- `npm --prefix citizenchain/node/frontend run build`：命名清理后再次通过。
- `cargo check --manifest-path citizenchain/Cargo.toml -p onchina -p node`：命名清理后通过。
- `cargo check --manifest-path citizenchain/Cargo.toml -p onchina`：机构 CA TLS 和公开证书接口修复后通过。
- `cargo build --manifest-path citizenchain/Cargo.toml -p onchina`：机构 CA TLS 和公开证书接口修复后通过。
- `cargo check --manifest-path citizenchain/Cargo.toml -p onchina`：macOS 证书有效期修复后通过。
- `cargo check --manifest-path citizenchain/Cargo.toml -p onchina`：FRG/CREG 控制台能力映射修复后通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p onchina platform::capability -- --nocapture`：通过，2 个能力映射单测通过，锁定 FRG 为 CREG 超集、CREG 可只读联邦注册局 tab 且无注册局维护写权。
- `cargo build --manifest-path citizenchain/Cargo.toml -p onchina`：macOS 证书有效期修复后通过。
- 临时端口运行态验收：以 `ONCHINA_BIND_ADDR=127.0.0.1:8979`、独立 `/tmp/onchina-cert-policy-*` TLS/PG 目录启动新构建的 `target/debug/onchina serve`；生成的 CA 为 `2026-01-01` 到 `2036-01-01`，服务证书为启动日前一天到 397 天后，策略标记为 `onchina-ca-v2-ca2036-server397d`。
- macOS 证书兼容验收：`/api/v1/platform/ca-certificate/info` 返回的 SHA-256 与下载 CA 证书 DER 指纹一致；`openssl verify -CAfile onchina-org-root-ca.crt onchina-server.crt` 返回 `OK`；验收后已停止临时服务并清理 `/tmp/onchina-cert-policy-*`。
- 临时端口运行态验收：以 `ONCHINA_BIND_ADDR=127.0.0.1:8976`、`ONCHINA_ENABLE_TLS=1`、独立 `/tmp/onchina-ca-verify-*` TLS/PG 目录启动新构建的 `target/debug/onchina serve`，`curl -k https://127.0.0.1:8976/api/v1/health` 返回 `status=UP`。
- 机构 CA 接口运行态验收：`/api/v1/platform/ca-certificate/info` 返回 `filename=onchina-org-root-ca.crt` 和证书 DER SHA-256；`/api/v1/platform/ca-certificate` 返回 `content-type=application/x-x509-ca-cert`、`content-disposition=attachment; filename="onchina-org-root-ca.crt"`；下载证书 DER SHA-256 与服务端信息一致。
- TLS 证书链验收：`openssl verify -CAfile onchina-org-root-ca.crt onchina-server.crt` 返回 `OK`，服务证书 SAN 为 `DNS:onchina.local`；验收后已停止临时服务并删除 `/tmp/onchina-ca-verify-*`、下载证书和 header 临时文件。
- 残留扫描：旧 TLS 文件名、旧单证书表述、旧摄像头/取消混合提示均为 0 命中；`citizenchain/runtime/` 无本次 diff。
- `cargo build --manifest-path citizenchain/Cargo.toml -p onchina`：命名清理后通过，已生成新 debug 二进制用于运行态验收。
- `node --check citizenapp/tools/generate_admin_division_bundle.mjs && node --check citizenapp/tools/generate_public_institution_bundle.mjs`：通过。
- 临时端口运行态验收：以新 `ONCHINA_*` / `ONCHAIN_*` 环境变量启动 `citizenchain/target/debug/onchina serve`，内嵌 PG 初始化成功，`curl -k https://127.0.0.1:8974/api/v1/health` 返回 `status=UP`；验收后已停止服务并删除 `/tmp/onchina-codex-env-clean-*`。
- 目标残留扫描：旧链 WS 环境变量、旧平台环境变量、旧登录/鉴权/绑定/API 错误码、旧节点身份误配置提示均为 0 命中；`citizenchain/runtime/` 无本次 diff。
- 权限残留扫描：旧 FRG/CREG 控制台降权描述在当前代码和记忆文档中为 0 命中。
- `cargo check --manifest-path citizenchain/Cargo.toml -p node`：通过。
- `cargo check --manifest-path citizenchain/Cargo.toml -p onchina`：通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p onchina login -- --nocapture`：通过，当前筛选下 0 个测试执行、72 个测试被过滤，编译与测试入口通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p onchina passkey -- --nocapture`：通过，3 个 passkey 相关测试通过。
- `git diff --check`：通过。
- `cargo build --manifest-path citizenchain/Cargo.toml -p onchina -p node`：通过，已生成本次代码对应的真实 debug 二进制。
- `cargo build --manifest-path citizenchain/Cargo.toml -p onchina`：新增 `NODE_BINDING_UNBIND` 后重新构建通过。
- 临时端口运行态验收：以 `ONCHINA_BIND_ADDR=127.0.0.1:8974`、`ONCHINA_ENABLE_TLS=1` 启动本次新构建的 `target/debug/onchina serve`，`curl -k https://127.0.0.1:8974/api/v1/health` 返回 `status=UP`；验收后已停止临时进程并清理 `/tmp/onchina-codex-*` 临时 TLS/日志。
- 旧入口扫描：`rg "http://onchina\\.local:8964|http://127\\.0\\.0\\.1:8964"` 已确认代码和记忆文档中不再保留旧正式入口文案。
- 未执行完整 Tauri 桌面真实运行态验收：该命令会在当前机器启动真实区块链节点与挖矿进程，本次先以构建、Rust check 和静态入口扫描收口；后续如需真机验收，应在可接受启动本机节点的窗口中执行。
