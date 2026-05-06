# 任务卡：三平台安装包零依赖化 + Windows 中文引导

- 任务编号：20260505-220000
- 状态：open
- 负责人：当前主聊天入口（Blockchain Agent 主导，节点客户端打包，runtime/链端 0 改动）
- 关联前置：无
- 关联后续：无

## 1. 任务目标

让 `citizenchain` 桌面客户端在 **Windows / macOS / Linux** 三平台上做到「装一次安装包就能跑」：

1. **Windows**：消除 `MSVCP140.dll` / `VCRUNTIME140.dll` 缺失报错；WebView2 离线打包进 NSIS，不联网；安装引导锁简体中文。
2. **macOS**：明示 `minimumSystemVersion = 10.15`（WKWebView 起点），DMG 自包含不变。
3. **Linux**：保留 DEB 给 apt 用户；新增 AppImage 输出，单文件零依赖（含 WebKitGTK）。

不动 runtime / 链端 / 业务代码，仅打包配置。

## 2. 影响范围

### 2.1 citizenchain 工作区

- **新建** `citizenchain/.cargo/config.toml`：
  - `[target.x86_64-pc-windows-msvc]` 段加 `rustflags = ["-C", "target-feature=+crt-static"]`
  - 仅对 Windows MSVC target 生效，不影响 macOS / Linux 构建

### 2.2 citizenchain/node

- **修改** [citizenchain/node/tauri.conf.json](citizenchain/node/tauri.conf.json) `bundle` 段：
  - `targets`: `["dmg", "nsis", "deb"]` → `["dmg", "nsis", "deb", "appimage"]`
  - 新增 `bundle.windows`：
    - `webviewInstallMode.type = "offlineInstaller"`
    - `nsis.installMode = "perMachine"`
    - `nsis.languages = ["SimpChinese"]`
    - `nsis.displayLanguageSelector = false`
  - 新增 `bundle.macOS.minimumSystemVersion = "10.15"`（与现有 `infoPlist` 并列）

### 2.3 文档与脚本

- 新增 `memory/05-modules/citizenchain/node/CROSS_PLATFORM_BUILD.md`：列明三平台构建机器约束、命令、产物路径、QA 自测步骤、故障排查
  - 注：`citizenchain/scripts/` 整目录被 `.gitignore` 排除（用于本机临时脚本），所以构建手册落地在 memory 模块文档下而不是 `citizenchain/scripts/`
- memory 索引追加项目记录（见第 5 节）

### 2.4 不动清单（必须确认零改动）

- 任何 `runtime/` 下的 crate
- 任何 `node/src/` 下的业务代码
- 任何 wumin / wuminapp / sfid 代码
- workspace 根 `Cargo.toml`（仅 `.cargo/config.toml` 是新文件）
- `Cargo.lock`（不应被本任务改动）

## 3. 必须遵守

- 不可突破模块边界（仅打包配置，不动 runtime / 链端 / 业务）
- 不可绕过既有契约
- 不可擅自修改安全红线（`tauri.conf.json` 的 `app.security.csp` 不动）
- 遵守 [feedback_no_chain_changes.md](../../../.claude/projects/-Users-rhett-GMB/memory/feedback_no_chain_changes.md)：本任务零 citizenchain 链端代码改动
- 遵守 [feedback_no_compatibility.md](../../../.claude/projects/-Users-rhett-GMB/memory/feedback_no_compatibility.md)：直接切换，不留旧打包配置
- NSIS 语言锁简体中文，不保留英文兜底（用户明确要求中文化）

## 4. 输出物

- `citizenchain/.cargo/config.toml`（新建）
- `citizenchain/node/tauri.conf.json`（修改 `bundle` 段）
- `memory/05-modules/citizenchain/node/CROSS_PLATFORM_BUILD.md`（新建，三平台构建手册）
- 中文注释（`.cargo/config.toml` 内说明为什么静态链接 CRT）
- 测试：本任务无单元测试维度，需手工三平台 QA（见验收标准）
- 残留清理：检查 `Cargo.lock` 未被改动；老的 `bundle.targets` 中 `"deb"` 保留不删

## 5. 验收标准

### 5.1 构建产物

- Windows：在 Windows + MSVC 工具链上 `cargo tauri build` 出 `*.exe` NSIS 安装包，体积约 ~150-180 MB
- macOS：在 macOS 上 `cargo tauri build` 出 `*.dmg`，体积与原值相比基本不变
- Linux：在 Ubuntu 22.04 上 `cargo tauri build` 同时出 `*.deb` 和 `*.AppImage`

### 5.2 Windows QA（必须在干净 Win10 / Win11 上各测一次）

- [ ] 全新虚拟机（未装过任何 VC++ 运行库、未装 WebView2）双击 NSIS exe，能弹出安装界面
- [ ] 安装界面**全程简体中文**，无英文／语言选择框
- [ ] 安装完成后双击桌面快捷方式启动 `citizenchain.exe`，**不再报 `MSVCP140.dll`**
- [ ] 启动后 Tauri WebView 渲染正常，节点能起 RPC :9944
- [ ] 卸载从控制面板正常进行，残留目录可接受

### 5.3 macOS QA

- [ ] DMG 在 macOS 10.15 / 13 / 14 上各启动一次，能正常拉起 WebView
- [ ] 启动时不弹出"需要 XX 运行库"提示
- [ ] 节点 RPC :9944 可达

### 5.4 Linux QA

- [ ] AppImage 在最小化的 Ubuntu 22.04（**未** apt install webkit2gtk）上 `chmod +x && ./xxx.AppImage` 能直接跑
- [ ] DEB 在标准 Ubuntu 22.04 上 `apt install ./xxx.deb` 能装上
- [ ] 两种方式启动后节点 RPC :9944 可达

### 5.5 静态链接验证（Windows）

在产出的 `citizenchain.exe` 上跑 `dumpbin /dependents`（或 `Dependency Walker`）：

- ✅ 不出现 `MSVCP140.dll`、`VCRUNTIME140.dll`、`VCRUNTIME140_1.dll`
- ✅ 仅依赖 Windows 系统 DLL（`KERNEL32`、`USER32`、`ADVAPI32`、`WS2_32`、`WININET` 等）

### 5.6 文档与残留

- [ ] `build-cross-platform.md` 写明三平台构建机器要求、命令、产物校验方法
- [ ] memory 索引补一条 `project_installer_zero_dep_2026_05_05.md` 记录本次决定
- [ ] `Cargo.lock` 未被本任务意外改动

## 6. 风险与回滚

- **风险 1**：`+crt-static` 与某些 C 依赖（rocksdb 的 `librocksdb-sys`）链接时可能要求自身也用 `/MT`。已知 Substrate 节点通常能编过，但需在 Windows 真机上跑一次 `cargo build --release --target x86_64-pc-windows-msvc` 验证。
- **风险 2**：WebView2 offlineInstaller 让 NSIS 包体积 +120 MB。能接受。
- **风险 3**：Linux AppImage 与 DEB 共存时，Tauri 的 build 时间翻倍。能接受。
- **回滚**：删除 `.cargo/config.toml`，把 `tauri.conf.json` 的 `bundle` 段还原即可，无链端／DB 影响。

## 7. 开放项（开工前必须确认）

| 项 | 默认 | 用户可改 |
|---|---|---|
| Windows NSIS 安装模式 | `perMachine`（要管理员一次） | 可改 `perUser` |
| 是否同时编 GPU 挖矿（`gpu-mining` feature） | 否（默认 features 不含） | 可改 |
| 是否在 Windows 包名/快捷方式上写「公民链」中文 | 是（沿用 `tauri.conf.json` 现有 `productName: "citizenchain"` + `windows.title: "公民链"`） | 可改 productName |

## 8. 分工

- **Blockchain Agent**：
  - 改 `tauri.conf.json` 与新建 `.cargo/config.toml`
  - 写 `build-cross-platform.md`
  - 提交 PR
- **用户**：
  - 三平台真机构建 + QA（CI 没 Windows / macOS runner 时不可避）
  - 跑 5.2 / 5.3 / 5.4 验收清单
