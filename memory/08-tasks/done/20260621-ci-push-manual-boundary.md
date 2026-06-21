# 统一 GitHub CI 的 push 与手动发布部署边界

## 任务需求

- 仓库所有系统的 GitHub Actions 必须统一为:push / pull_request 自动触发时只执行 CI 校验。
- 只有 GitHub 页面手动 `Run workflow` 触发某个系统时,才允许进入发布、签名、部署、清理旧发布产物等正式发布链路。
- push 自动 CI 不得读取部署 SSH 密钥、正式签名 keystore、Tauri updater 私钥等 GitHub Secrets。

## 所属模块

- 仓库级 GitHub Actions CI/CD。
- CitizenChain WASM 与桌面端。
- CitizenCode(CID)。
- CitizenPassport(CPMS)。
- CitizenApp。
- CitizenWallet。

## 输入文档

- `memory/07-ai/ci-path-routing.md`
- `memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/CROSS_PLATFORM_BUILD.md`
- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- `memory/01-architecture/citizencode/CID_TECHNICAL.md`
- `memory/01-architecture/citizenpassport/CITIZENPASSPORT_TECHNICAL.md`

## 必须遵守

- 不修改 `citizenchain/runtime/`。
- 不引入跨系统部署密钥 fallback。
- push / pull_request 不访问服务器、不发布 Release、不部署服务器、不读取正式签名密钥。
- 手动 `workflow_dispatch` 才允许发布、签名、部署、清理旧 artifact 或旧 workflow run。

## 输出物

- `.github/workflows/` 中各系统 workflow 边界修正。
- CI 边界与系统技术文档更新。
- 残留密钥、部署、发布触发条件复查。

## 验收标准

- `citizenchain-wasm.yml` push 不再查询链上 runtime 版本,不再读取 SSH/RPC Secret。
- `citizenwallet-ci.yml` push 不再恢复 release keystore,只做 Debug/检查构建。
- `citizencode-ci.yml` push 不再构建正式 `.deb` 发布包。
- 所有 SSH 部署和 GitHub Release 发布只存在于 `workflow_dispatch` 路径。
- workflow YAML 语法检查通过。

- 状态：done

## 完成信息

- 完成时间：2026-06-21 12:50:22
- 完成摘要：统一 GitHub Actions push/manual 边界:push 只 CI,手动才读取签名/部署密钥、发布 Release 或部署服务器;已更新 workflow 与 CI 文档并完成残留检查。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
