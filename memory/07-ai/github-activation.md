# GMB GitHub Actions 激活指南

## 1. 目标

全仓只启用四条产品 workflow，不依赖独立 AI/Claude workflow：

- `.github/workflows/citizenchain-ci.yml`
- `.github/workflows/citizenchain-wasm.yml`
- `.github/workflows/citizenapp-ci.yml`
- `.github/workflows/citizenwallet-ci.yml`

CitizenChain workflow 内的 `guardrails` job 对每个非草稿 PR 执行全局 AI 编程门禁。CitizenApp
与 Cloudflare 共用一条 workflow，OnChina 归属 CitizenChain，官网不使用 GitHub Actions。

## 2. 需要的 GitHub 配置

- 常规 push / PR CI 只需仓库 `contents: read`；门禁还需 `pull-requests: read`。
- `GMB_APP_KEY` 仅供手动 CitizenApp/CitizenWallet 正式签名发布。
- `GMB_TOP_KEY` / `GMB_TOP_PUBKEY` 仅供手动 CitizenChain 桌面端正式发布。
- 常规 CI 不读取上述 Secret，WASM CI 不读取服务器或部署 Secret。

## 3. AI 门禁检查项

`citizenchain-ci.yml` 的 `guardrails` job 调用 `.github/scripts/check-ai-guardrails.sh`，执行：

- 直接验证已跟踪的启动协议文件、根入口软链接和关键规则；本机手工验收脚本保持 Git 忽略，
  GitHub runner 的独立门禁和全工程 `--startup-only` 检查均不依赖本机私有 `scripts/` 文件
- 禁止删除/迁出 AI 编程系统核心基础设施
- 改代码必须同步更新文档与任务卡，并回写对应模块文档
- 开发残留扫描（TODO/FIXME/console.log/dbg! 等）
- 较大代码改动需配套中文注释
- **PQC 前向兼容守则（ADR-022）**：当前只用 sr25519、暂不接入 PQC。保护以下 sr25519 锚点不被“净删除/改值”，以保障将来无感接入 PQC（不换钱包/账户/地址/金额）：
  - `citizenchain/runtime/src/lib.rs` 的 `Signature = MultiSignature`、`AuthorizeCall`
  - `citizenchain/runtime/primitives/src/core_const.rs` 的 `SS58_FORMAT` 与值 `2027`
  - `citizenapp` / `citizenwallet` 钱包的 `miniSecretFromEntropy` 派生
  - `citizenapp` / `citizenwallet` QR bodies 的 `sig_alg` 字段
  - 确属有意变更时，须在同一 PR 同步更新 `memory/04-decisions/ADR-022-unified-pqc-crypto.md` 守则章节作为确认，门禁才放行。

## 4. 激活后的验证方式

新开一个 PR，修改任意代码文件但不更新文档。

预期结果：CitizenChain workflow 的 `guardrails` job 失败，并报告“改代码后未更新文档”或检测到的残留。

再分别修改 CitizenApp/Cloudflare、CitizenWallet 和 CitizenChain 路径，确认只触发对应的产品 workflow；手动 WASM 在未明确执行时不得自动触发。

## 5. 启用完成判定

- `.github/workflows/` 中精确只有四个 workflow 文件。
- 非草稿 PR 能触发 CitizenChain `guardrails` job。
- CitizenChain 代码变更通过全 workspace 验证后才进入桌面端 matrix。
- CitizenChain 的 Linux 全 workspace/all-targets 验证必须安装与 Linux 桌面打包一致的
  GTK、WebKit、AppIndicator、RSVG 等系统依赖，不得因 runner 缺系统库而缩小 Rust 覆盖。
- CitizenChain 节点前端必须在 Rust 全 workspace 验证前构建；Tauri `generate_context!`
  编译期读取 `frontendDist`，CI 不提交、伪造或用空目录替代 `frontend/dist` 产物。
- CitizenApp 同一 workflow 同时覆盖 Flutter 与 Cloudflare/D1。
- 官网不存在独立 CI，WASM 不存在自动触发。
