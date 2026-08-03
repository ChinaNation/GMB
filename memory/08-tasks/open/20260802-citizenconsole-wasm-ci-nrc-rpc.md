# CitizenConsole WASM CI 改读国储会私有 RPC

## 任务目标

CitizenConsole 的「运行 WASM CI」不得依赖本机 CitizenChain 节点。单次 Touch ID
原子读取现有国储会 Access 配置、正式链创世哈希和 GitHub SSH 私钥，通过
`chain.crcfrcn.com`、Cloudflare Access、`nrcgch-rpc` Tunnel 与固定方法网关读取国储会
节点 finalized Runtime 版本，完成既有版本判断后才允许进入提交、推送和 CI 等待流程。

## 硬边界

- `30333` 是 libp2p P2P 端口，不承担 JSON-RPC。
- 国储会节点 `9944` 继续只监听回环地址，不开放公网入站。
- 复用 `production:CHAIN_URL / CHAIN_ID / CHAIN_SECRET`，不新增第二套链服务配置。
- WASM CI 单次 Touch ID 同时读取链服务配置、`CHAIN_GENESIS_HASH` 与 `SSH_KEY`。
- 只允许固定只读 RPC 方法，不恢复通用 JSON-RPC 代理。
- 不修改 runtime、P2P、订阅业务、充值发币和开发升级流程。
- 不部署国储会节点，不推送 GitHub，不触发远端 CI，除非用户另行明确允许。

## 实施步骤

- [x] 将 Runtime 版本读取改为 Access 保护的 HTTPS JSON-RPC。
- [x] WASM CI 移除 `topup:NODE_WS` 依赖，接入三项现有链服务配置。
- [x] 以 finalized 区块读取 `spec_version` 并继续校验 block #0 创世哈希。
- [x] 补齐认证、重定向、超时、响应上限、错误响应和版本漂移测试。
- [x] 更新技术文档、清理本机节点旧口径。
- [x] 通过 CitizenConsole 自身的 Touch ID「编译」动作重建签名包并完成完整性验收。
- [ ] 使用同一次 Touch ID 完成真实国储会只读 RPC 验收。

## 预计修改目录

- `citizenconsole/server.mjs`：调整 WASM CI 单次鉴权配置读取和运行编排。
- `citizenconsole/rtupg/`：实现国储会 Access HTTPS RPC 的 finalized Runtime 版本读取。
- `citizenconsole/test/`：补充远端链读取与安全失败测试。
- `memory/01-architecture/citizenchain/`：记录 CitizenConsole 的国储会私有 RPC 使用边界。
- `memory/08-tasks/open/`：维护本任务执行和验收记录。

## 执行记录

- 2026-08-02：用户确认实施。当前本机未监听 `9944`；现有失败由 WASM CI 读取
  `topup:NODE_WS=ws://127.0.0.1:9944` 导致。仓库已存在国储会 Access + Tunnel 私有 RPC
  架构和 `production:CHAIN_URL / CHAIN_ID / CHAIN_SECRET` 单一配置。
- 2026-08-02：WASM CI 已改为同一次 Touch ID 原子读取国储会 Access 三项配置、正式链
  创世哈希与 GitHub SSH 私钥；经三个固定 HTTPS JSON-RPC 方法读取 finalized 头、block #0
  与 finalized RuntimeVersion。客户端拒绝非 `chain.crcfrcn.com`、非 HTTPS、重定向、超限
  响应、无效 JSON-RPC 和无效版本；本机 `NODE_WS` 不再进入该流程。
- 2026-08-02：`npm test` 58 项全部通过，`npm run check`、ShellCheck 与
  `git diff --check` 通过；已通过控制台「编译」的独立 Touch ID 重建正式签名包，
  `start.sh verify` 以及签名包内 `server.mjs / rtupg/tx_common.mjs` 与源码逐字节比对通过。
- 2026-08-02：尚未点击「运行 WASM CI」执行真实 Access RPC 预检，因为该生产动作在预检
  成功后会继续整仓提交、推送 `origin/main` 并触发 GitHub Actions；当前任务没有单独取得
  GitHub 推送授权。该边界不是本地实现失败，任务继续保持 open，不能把模拟测试冒充真实
  国储会 RPC 验收。
