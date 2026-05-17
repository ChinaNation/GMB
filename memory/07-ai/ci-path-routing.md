# GMB CI 路径分流规则

## 1. 目标

GMB 的 GitHub Actions 采用“按改动目录精确触发”的策略，避免无关模块互相拖慢。

核心原则：

- 改哪个模块，就优先只跑哪个模块
- 共享依赖变更时，允许多模块联动触发
- 安全门禁与 Claude 审查属于跨模块能力，继续对 PR 全局生效

## 2. citizenchain 当前规则

### 2.1 runtime WASM

- workflow：`.github/workflows/citizenchain-wasm.yml`
- 主要命中目录：
  - `citizenchain/runtime/**`
  - `.github/workflows/citizenchain-wasm.yml`

### 2.2 node 桌面安装包

- workflow：`.github/workflows/citizenchain.yml`
- 主要命中目录：
  - `push main` 会触发桌面端三平台构建检查，但只上传本次 run artifact，不发布 GitHub Release，不部署服务器
  - GitHub 页面手动 `Run workflow` 才进入正式发布路径：生成 updater 签名产物、发布 GitHub Release、部署 Linux 服务器
  - 单个 workflow 通过 matrix 同时构建 Linux / Windows / macOS，三端使用同一个桌面端版本号
  - 三端安装包不下载、不内置最新 `citizenchain-wasm` artifact；现有链运行 runtime 以链上 `System.set_code` 为准
  - 本地开发启动和重新创世脚本使用当前源码构建 runtime，不从 GitHub CI 下载 WASM
  - 手动发布成功后上传三端安装包、updater 签名产物与 `citizenchain-latest.json` 到 GitHub Release，供桌面端点击更新链路使用
  - Linux 产物继续包含 `公民链.deb`；仅手动发布时，成功上传本次 `公民链-linux` artifact 后先预检查 6 台服务器 SSH 登录，全部通过后再顺序滚动部署同一个 deb
  - 手动发布与 Linux 服务器部署都成功后，删除上一条 `citizenchain.yml` 已完成 CI run；push 构建不执行清理
- 代码目录：
  - `citizenchain/node/**`
  - `citizenchain/node/frontend/**`
  - `citizenchain/node/src/<功能名>/**`

## 3. 其他模块的分流方向

当前仓库规则已经明确为：

- `sfid`
  - CI：`.github/workflows/sfid-ci.yml`
  - 部署：`.github/workflows/sfid-deploy.yml`
- `cpms`
  - CI：`.github/workflows/cpms-ci.yml`
- `wuminapp`
  - CI：`.github/workflows/wuminapp-ci.yml`
- `docs`
  - Pages：`.github/workflows/pages.yml`

## 4. 当前结论

路径分流的目的不是减少安全检查，而是减少无关重复构建。

因此：

- 全局门禁继续保留
- Claude 审查继续保留
- 模块级构建和测试按目录精确触发
- 共享 Rust 根目录变更允许触发多个 citizenchain workflow
