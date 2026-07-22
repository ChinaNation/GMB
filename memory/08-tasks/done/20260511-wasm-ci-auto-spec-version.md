# 20260511 WASM CI 自动对齐链上 spec_version

> 历史记录：本实现已于 2026-07-21 被正式创世前项目版本归零规则取代。当前 GitHub WASM workflow 按源码版本原样编译，不查询链上版本、不读取 SSH Secret、不临时修改 `spec_version`；正式创世后的加一只允许由公民控制台在本机读取明确目标链后写入并提交源码。以下内容仅记录当时事实。

## 任务目标

- 修复开发升级使用最新 WASM 时因 `System::SpecVersionNeedsToIncrease` 被链上拒绝的问题。
- WASM CI 每次编译前先查询链上 `state_getRuntimeVersion.specVersion`。
- 当源码 `citizenchain/runtime/src/lib.rs` 中的 `spec_version` 小于或等于链上版本时，只在 CI 工作区临时提升到 `链上版本 + 1` 后再编译。
- 不让 CI 自动提交版本号回 `main`，避免二次触发和版本提交噪音。

## 预计修改目录

- `.github/workflows`
  - 修改 CitizenChain WASM CI，在编译前增加链上版本检查与临时 bump；涉及 CI 配置和旧注释清理。
- `citizenchain/runtime/src`
  - 调整 runtime 版本单测，不再把 `spec_version` 写死为单一值；涉及测试代码。
- `memory/05-modules/citizenchain`
  - 更新 WASM CI 与开发升级文档，说明 `spec_version` 的链上校验和 CI 产物规则；涉及文档。

## 执行记录

- 已在 `.github/workflows/citizenchain-wasm.yml` 增加链上版本检查步骤。
- 历史实现曾使用系统专属部署密钥登录服务器读取链上 `state_getRuntimeVersion`；该口径已废弃，当前统一使用 `GMB_SSH_KEY`。
- 未配置 SSH key 时，CI 才使用 `CITIZENCHAIN_RPC_URL` 直连 HTTP RPC。
- 当源码 `spec_version` 小于或等于链上版本时，CI 工作区临时改成 `链上版本 + 1` 后再编译 WASM。
- 已保留“不自动 commit 回 main”的边界，避免二次触发 WASM CI。
- 已删除旧的“版本只能纯手动管理 / 删除 spec_version 自增”的过时注释。
- 已调整 runtime 版本测试，不再写死单一 `spec_version`。
- 已同步更新 node 首页技术文档与 runtime-upgrade 技术文档。

## 验证记录

- `git diff --check`：通过。
- `cargo fmt --manifest-path citizenchain/Cargo.toml -p citizenchain --check`：通过。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo test --manifest-path citizenchain/Cargo.toml -p citizenchain runtime_version_and_block_types_are_sane`：通过，1 passed。
- 使用模拟链上 `specVersion=0` 本地执行 CI 版本检查脚本：通过，源码版本为 1 时不会被错误改写。
- 追加修复 GitHub runner 访问公网 9944 超时问题：WASM CI 已改为优先 SSH 到服务器后查询本机 RPC。
- 历史修复曾处理 GitHub Secret 私钥不匹配/带密码导致的 `Permission denied (publickey)`；当前密钥命名已统一为 `GMB_SSH_KEY`。
