# 统一 GitHub Actions 密钥命名

## 任务需求

- 服务器 SSH 手动部署和手动链上查询只使用 `GMB_SSH_KEY`。
- 公民与公民钱包两个 Android App 使用同一套签名材料,GitHub 只暴露一个 secret:`GMB_APP_KEY`。
- 桌面端 Tauri updater 签名统一使用 `GMB_TOP_KEY` 和 `GMB_TOP_PUBKEY`。
- 清理 workflow 和文档中的旧系统专属密钥口径。

## 所属模块

- GitHub Actions CI/CD。
- CitizenChain 桌面端和 WASM 手动 workflow。
- CitizenApp Android。
- CitizenWallet Android。
- 仓库记忆文档。

## 输入文档

- `memory/07-ai/ci-path-routing.md`
- `memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/CROSS_PLATFORM_BUILD.md`
- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- `memory/05-modules/citizenwallet/CITIZENWALLET_PQC_TECHNICAL.md`

## 必须遵守

- 不修改 `citizenchain/runtime/` 代码。
- push / pull_request 自动 CI 不读取密钥。
- 手动 `workflow_dispatch` 才读取 `GMB_*` 密钥。
- 先确认 `GMB_*` 新 secret 写入成功,再删除 GitHub 旧 secret,避免不可恢复的旧值先被清掉。

## 输出物

- workflow secret 名称统一。
- Android 签名材料统一读取。
- 文档与残留清理。

## 验收标准

- workflow 不再引用旧服务器 SSH secret 名称。
- Android 两个 App workflow 不再引用旧 App 专属签名 secret 名称。
- Tauri updater workflow 不再引用旧系统专属桌面签名 secret 名称。
- YAML 解析通过,残留扫描通过。

- 状态：done

## 完成信息

- 完成时间：2026-06-21 13:21:38
- 完成摘要：统一 GitHub Actions 密钥命名:服务器只用 GMB_SSH_KEY,移动端只用 GMB_APP_KEY,桌面 updater 使用 GMB_TOP_KEY/GMB_TOP_PUBKEY;GitHub 已新增目标 secret 并删除旧系统专属 secret;已清理旧 secret 名称残留并完成 YAML/条件检查。
- GitHub 当前保留的仓库 secret：`ANTHROPIC_API_KEY`、`GMB_APP_KEY`、`GMB_SSH_KEY`、`GMB_TOP_KEY`、`GMB_TOP_PUBKEY`。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
