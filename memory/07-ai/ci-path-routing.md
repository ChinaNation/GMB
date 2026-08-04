# GMB CI 路径分流规则

## 1. 目标

GMB 的 GitHub Actions 采用“只由公民控制台按钮显式发起”的策略：每个软件的 CI 彼此彻底分开，
向 `main` 推代码是纯粹的推代码，不启动任何流水线。

核心原则：

- **四个 workflow 一律不设 `push` 触发**。本地 `git push` 永远不产生任何运行记录；
  CI 只能由公民控制台对应按钮经 `workflow_dispatch` 发起，点哪个软件就只跑哪个软件
- `citizenchain-ci.yml` 与 `citizenapp-ci.yml` 保留 `pull_request` 触发：文档/残留门禁
  与金标向量是 PR 唯一的把关点，不得删除
- 文档、残留和 pallet 注册表门禁属于跨模块能力，作为 `citizenchain-ci.yml` 内部 job 对 PR 全局生效
- **CI 与 Release 是同一份代码的两种产物，四个软件形态完全一致，不得给任何一个搞特殊**：
  - `mode=ci` → 涨版本 → 构建**不签名**产物 → 上传 artifact
  - `mode=release` → 校验代码已通过 CI → 涨版本 → **在同一份代码上重新构建并正式签名** → 发布 GitHub Release
  - 代码相同、流程相同、版本规则相同、失败重跑规则相同，唯一区别就是产物签不签名
- `pull_request` 与 `mode=ci` 的 dispatch 只允许执行校验、编译、测试和检查构建,不得访问服务器、不得发布 GitHub Release、不得部署、不得读取部署 SSH 密钥或正式签名密钥
- 只有 `mode=release` 的 dispatch(控制台「正式 Release」按钮)才允许进入正式发布链路,包括 `GMB_APP_KEY`、`GMB_TOP_KEY / GMB_TOP_PUBKEY`、GitHub Release 发布、正式安装包上传和旧发布产物清理
- **Release 必须建立在已通过 CI 的提交上**：`require_ci_verified_head <workflow>` 取该
  workflow 最近一次成功运行的 `headSha`，与 `origin/main` 当前 tip 比对，不等就拒绝并要求
  先跑 CI。否则会把从未验证过的代码签名公开发布。正常流程天然通过：点 CI → 成功 →
  不改代码 → 点 Release，两者必然相等
- **已废除「固定提交名路由」**：它是 push 触发时代的分流手段，代价是每次普通 push 都留下
  一条 skipped 记录污染 Actions 列表，失败重试还只能靠造空提交换 SHA。dispatch 可直接对
  同一 SHA 重跑，该机制连同 `wait_public_workflow`、`allow_empty`、`skip_ci` 全部删除
- 全仓外部 JavaScript action 必须使用 Node 24；涉及产物上传/下载和本次 Node 20
  升级的 action 必须固定到经官方 release tag 反向核验的完整提交 SHA。既有 Node 24
  大版本 tag 是否需要固定，由正式产物供应链审查单独决定；Composite action 不受 Node
  运行时约束，但仍须在使用前核对来源与版本。

## 2. citizenchain 当前规则

### 2.1 runtime WASM

- workflow：`.github/workflows/citizenchain-wasm.yml`
- 2026-08-01 正式创世 WASM run `30721127038` 仅该次显示为“创世”；正式冻结时已删除
  临时 `run-name`，后续运行统一恢复标准名称 `CitizenChain WASM`。
- 正式升级构建入口仅为公民控制台「CitizenChain → 运行 WASM CI」：按钮位于「运行节点 CI」
  下方并明确要求
  一次 Touch ID，原子读取 `CHAIN_URL` / `CHAIN_ID` / `CHAIN_SECRET`（国储会受保护 RPC）、
  `CHAIN_GENESIS_HASH` 与 GitHub `SSH_KEY` + `GH_TOKEN`。控制台
  先要求 RPC genesis hash 等于本机明确保存的正式链指纹，再比较链上、
  本机和 `origin/main` 的 `spec_version`：仓库与链同版时同步提高源码和现有测试断言一版；
  仓库已经比链高一版时复用该候选以重试失败或不可用的 WASM CI；其他差值全部停止。
- CitizenConsole 不再提供独立 GitHub 卡片；同一对 `SSH_KEY` + `GH_TOKEN` 归入 CitizenApp、
  CitizenChain、CitizenWallet 三张产品卡片，由各产品 CI、Release 和 CitizenChain WASM CI
  在一次 Touch ID 中一并读取。两把凭据各司其职、缺一不可：`SSH_KEY` 认证 git 协议（推代码），
  `GH_TOKEN` 认证 GitHub REST API（`workflow_dispatch` 触发与运行状态查询）——SSH 私钥无法
  认证 API，这是 GitHub 平台限制。`GH_TOKEN` 需 `repo` + `workflow` 权限，只存在本机 Keychain
  白名单内（`OperationCatalog.isAllowed` 的 `github` 环境恰好三项），注入时只进单次动作的
  子进程环境，不落盘、不写 `~/.config/gh`，因此永远不触发钥匙串登录弹窗。
  密钥只保存一份，不复制、不回显。版本校验后用 SSH 把全部 Git 可见代码提交到 `main`
  （WASM 必须从已提交源码构建），提交名固定为 `CitizenChain WASM` 但**只作可读标记，
  不再承担路由职责**；推送本身不触发任何流水线，随后由 `run_workflow` 显式 dispatch
  `citizenchain-wasm.yml`。失败重试直接对同一 SHA 重新 dispatch，不再造空提交。
- 控制台用 `gh run watch` 跟踪本次 dispatch 出来的运行；`gh` 的凭据是控制台注入的
  `GH_TOKEN`，不依赖 `~/.config/gh` 登录态。
- **版本契约：每点一次 CI 都涨一版**，唯一例外是上一次运行失败——那是重跑同一份代码，
  沿用当前版本并调用 `gh run delete` 删掉那条失败记录，否则一次失败白吃一个版本号。
  判断由 `previous_failed_run <workflow>` 完成，且**必须严格早于 bump**：版本已经涨了
  再判断毫无意义。只认「已完成且结论非 `success`」，仍在运行的任务不会被误删。
  删除不可逆，该 run 在 GitHub 上的详细日志与 artifact 一并消失。
- **软件版本推进唯一真源是 `actions/common.sh` 的 `next_semver`**，公民链桌面端、
  CitizenApp、CitizenWallet 共用同一份算术，起步 `1.0.0`：版本形如 `a.b.c`，
  `c ≤ 99`、`b ≤ 99`、`a` 无上限；`ci` → `c+1`（满 99 进位到 `b`，`b` 再满 99 进位到 `a`），
  `release` → `b+1` 且 `c` 归零（`b` 满 99 进位到 `a`）。
  例：`1.0.99 --ci→ 1.1.0`、`1.99.99 --ci→ 2.0.0`、`1.99.0 --release→ 2.0.0`。
  `bump_chain_version` 同步写 `tauri.conf.json` 与 `citizenchain/Cargo.toml`
  `[workspace.package]` 同一个值；`bump_pubspec_version` 另带 `+CODE`，
  该 `versionCode` 每次 `+1` 且**永不归零**——Android 应用内更新靠它单调递增判定，
  一旦随 `a.b.c` 进位重置，旧版会被认成新版。
  **runtime 不走这套**：它用链上 `spec_version` 整数 `+1`，与语义化版本是两套东西。
- workflow 按已提交源码原样编译并上传 artifact，不查询链上版本、不读取 RPC/SSH Secret、
  不连接服务器，也不得在 CI 工作区改写版本。正式链创世哈希和升级前版本由控制台在推送前
  校验；CI 记录源码 `spec_version` 与精确提交 SHA。
- WASM workflow 的唯一触发入口就是 `workflow_dispatch`，而控制台按钮在 dispatch 前
  已完成正式链指纹与 `spec_version` 校验，因此不存在「脱离校验的第二入口」。
  推送不再触发任何流水线，四条 workflow 之间不会互相误触发。
- 正式创世前没有可读取的正式目标链时，控制台升级入口必须停止并保持项目版本 `0`。
- 正式创世普通源码构建必须钉死不可变 head SHA。优先使用直接指向该提交的轻量候选 tag；
  用户明确指定使用 `main` 运行时，必须同时钉死 run ID、40 位 head SHA、唯一 artifact ID
  与 GitHub artifact digest，不得按“最新成功”推断产物。
- WASM 上传前必须校验三个产物存在且非空，并把大小和 Blake2-256 写入 CI Summary；上传失败必须使运行失败。历史 artifact 由 retention 自动过期，不得在新上传前删除审计证据。
- **两层留存：artifact 供升级，Release 供审计。** artifact 只留 30 天，超期后历史升级的
  原始 WASM 字节永远消失，既无法复核当初上链的到底是什么，也无法回滚——而 runtime 是全链
  安全等级最高的产物，一次 `set_code` 直接改写全网 44 个节点的执行逻辑。因此 WASM CI 成功后
  自动发布**不可变永久档案 Release**：tag 为 `runtime-v<spec_version>`，与 `spec_version`
  一一对应，携带三个 wasm 产物与各自 Blake2-256。**tag 已存在则整次运行失败，绝不覆盖**
  ——同一个 `spec_version` 出现两份不同字节，正是最该当场拦下的情况。
  升级流程本身仍走 artifact（30 天内够用），Release 不进热路径。
- 候选 tag 运行通过 `download-wasm.sh --run-id ... --head-sha ... --ref ...` 下载；用户明确
  指定的 `main` 运行必须由 GitHub CLI 精确读取同一 run 的 workflow、event、status、
  conclusion、head SHA、head branch、唯一未过期 artifact ID/digest，再下载指定 artifact。
- 2026-07-30 已清理全仓 action 的 Node 20 残留：`upload-artifact v7.0.1`、
  `download-artifact v8.0.1`、`setup-node v7.0.0`、`setup-java v5.6.0` 和
  `setup-android v4.0.1` 均固定到官方 release commit；其余 JavaScript action 已逐项
  核验为 Node 24，Rust toolchain 与 Flutter action 为 composite。
- 正式 WASM workflow 必须固定 `ubuntu-24.04` runner label、checkout 完整提交和
  Rust 1.97.1 toolchain action 完整提交；不得使用 `ubuntu-latest`、可移动 checkout
  大版本 tag 或上游 `refs/heads/1.97.1` 分支。Cargo 必须同时执行
  `metadata --locked` 和 `build --locked`，并在 Step Summary 记录 runner image、
  Rust/Cargo/Protobuf/Clang 版本、Cargo.lock SHA-256 与 action SHA。
- 触发契约：只有 `workflow_dispatch`，由控制台「运行 WASM CI」按钮发起。只有一种构建
  模式，故不设 `inputs`。任何 `main` push 都不会产生 WASM 运行记录。

### 2.2 CitizenChain 全工程与桌面安装包

- workflow：`.github/workflows/citizenchain-ci.yml`
- 主要规则：
  - 所有 PR 运行文档/残留/pallet 注册表门禁；纯文档 PR 不运行链编译与桌面打包。
  - `citizenchain/**`、`primitives/**` 或共享 Cargo 变更先执行启动协议、pallet 注册表、宪法 SCALE 自检、全 Rust workspace fmt/check/test/clippy、OnChina 前端和节点前端构建。
  - 全工程验证成功后，由同一 workflow matrix 构建并上传 4 个用户安装包 artifact。
  - `pull_request` 和 `mode=ci` 不读取 Tauri updater 签名密钥，不发布 GitHub Release，不生成客户端更新通知，不部署服务器。
  - 只有控制台「正式 Release」按钮（`mode=release`）进入正式发布路径：构建同样 4 个用户安装包，使用 `GMB_TOP_KEY / GMB_TOP_PUBKEY` 生成 updater 签名产物，发布 GitHub Release，更新 `citizenchain-latest.json`
  - 单个 workflow 通过 matrix 同时构建 macOS Apple / Windows / Linux amd / Linux arm，四个安装包使用同一个桌面端版本号（macOS 仅保留 ARM，不再构建 Intel）
  - 四个用户安装包名称固定为：
    - `公民链.dmg`
    - `公民链.exe`
    - `公民链AMD.deb`
    - `公民链ARM.deb`
  - 暂时不做 macOS / Windows / Linux 系统级签名；Tauri updater 签名不属于系统安装包签名，手动正式发布时必须继续保留
  - 自动更新、GitHub Release、Linux 服务器部署属于正式发布链路，不允许因为统一 4 个用户安装包而删除
  - 三端安装包不下载、不内置最新 `citizenchain-wasm` artifact；现有链运行 runtime 以链上 `System.set_code` 为准
  - 本地开发启动和重新创世脚本使用当前源码构建 runtime，不从 GitHub CI 下载 WASM
  - 手动发布成功后上传 4 个用户安装包、updater 内部资产、updater 签名产物与 `citizenchain-latest.json` 到 GitHub Release，供桌面端点击更新链路使用
  - 桌面端启动检查到可用 updater 后，顶部 `设置` tab 显示红点；红点只读取 Tauri updater 状态，不另建已读/未读状态
  - Linux 服务器部署只允许通过本机 `citizenconsole/` 控制台选择一个权威节点，并使用当前提交最新成功 CI 的 `公民链-Linux-amd.deb`；节点 IP、身份私钥、GRANDPA 私钥和 SSH 私钥来自该节点独立 Keychain 项，不允许恢复 GitHub workflow 固定 IP 批量部署。
- 代码边界：`citizenchain/**`、`primitives/**`、根 Cargo 真源及对应检查脚本。

## 3. 其他模块的分流方向

当前仓库规则已经明确为：

- `citizenchain/onchina`
  - 归属公民链产品 CI 边界，不得恢复独立 旧独立身份系统 CI
  - `pull_request` 与 `mode=ci`:只允许执行 OnChina 后端编译、后端测试、前端依赖安装和前端构建
  - 正式发布跟随公民链发布边界，不构建独立身份系统安装包
- `citizenapp`
  - CI：`.github/workflows/citizenapp-ci.yml`
  - `pull_request` 与 `mode=ci`:同时执行 Flutter analyze/test、Cloudflare 类型与测试、全新本地 D1 schema 和 Debug APK 检查，不读取 release keystore
  - Cloudflare 属于 CitizenApp 同一条 CI，禁止新建独立 workflow
  - `mode=release`:读取 `GMB_APP_KEY`,构建正式签名 `公民.apk`,发布 Android 更新 Release
- `citizenwallet`
  - CI：`.github/workflows/citizenwallet-ci.yml`
  - `mode=ci`:只做 Flutter analyze/test 与 Debug APK 检查构建,不读取 release keystore
  - `mode=release`:读取同一个 `GMB_APP_KEY`,构建并上传正式 `公民钱包.apk`
- `citizenweb`
  - 当前暂无专用 GitHub Actions，发布前在本地执行构建并部署静态产物

## 4. 当前结论

路径分流的目的不是减少安全检查，而是减少无关重复构建。

因此：

- 全仓严格只保留 CitizenChain、CitizenChain WASM、CitizenApp、CitizenWallet 四个 workflow
- 全局 PR 门禁继续保留，但作为 CitizenChain workflow 的内部 job，不再独立占用第五条流水线
- 每个软件的 CI 由各自的控制台按钮显式 dispatch，CitizenChain 本身保持全 workspace 覆盖
- `mode=ci` 的校验构建与 `mode=release` 的发布部署必须保持密钥边界隔离
