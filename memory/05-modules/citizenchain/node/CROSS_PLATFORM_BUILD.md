# citizenchain 跨平台打包手册

## 1. 设计原则

最终用户只装一次安装包就能跑，不联网下载、不让用户自己装 VC++ / WebView2 / WebKitGTK。

| 平台 | 输出 | 体积 | 用户额外要做 |
|---|---|---|---|
| Windows | NSIS `.exe` | ~165 MB | 双击运行（管理员一次） |
| macOS | DMG | ~100 MB | 拖到 Applications |
| Linux 仓库用户 | DEB | ~80 MB | `apt install ./xxx.deb` |
| Linux 单文件 | AppImage | ~170 MB | `chmod +x && ./xxx.AppImage` |

## 2. 打包配置位置

- `citizenchain/.cargo/config.toml` —— Windows MSVC `+crt-static`，静态链接 C++ 运行时
- `citizenchain/node/tauri.conf.json` `bundle` 段 —— targets / macOS minSysVer / NSIS 中文 / WebView2 离线打包
- `citizenchain/node/tauri.conf.json` `bundle.windows.nsis.installerIcon` —— Windows 安装包
  `setup.exe` 文件自身图标；安装后的应用/桌面图标仍由 `bundle.icon` 图标列表控制。
- `citizenchain/node/tauri.conf.json` `plugins.updater` 段 —— 桌面端打开软件后检查
  GitHub Release 中的 `citizenchain-latest.json`；检查更新不等于安装更新，安装必须由设置页“更新”按钮触发。
- `.github/workflows/citizenchain.yml` —— push 与手动发布分流：
  - `push main`：只构建 5 个用户安装包并上传本次 run artifact，不读取 Tauri updater 签名密钥，不发布 Release，不部署服务器。
  - `Run workflow`：构建同样 5 个用户安装包和 updater 签名产物，发布 GitHub Release，并部署 Linux amd 服务器。

## 3. 桌面端更新协议

桌面端更新使用 Tauri 官方 updater 插件，不允许自写下载器绕过签名校验。

### 3.1 发布端要求

- 手动 `Run workflow` 必须配置：
  - `GMB_TOP_PUBKEY`：Tauri updater 公钥，写入本次发布 App 的 Tauri 配置。
  - `GMB_TOP_KEY`：Tauri updater 私钥，用于生成 `.sig`；该私钥不再拆分系统专属 secret。
- 手动发布时 workflow 会临时把 `bundle.createUpdaterArtifacts` 改为 `true`；push 构建不会打开该开关。
- GitHub Release 资产包含：
  - 普通安装包：`公民链-macOS-Intel.dmg` / `公民链-macOS-apple.dmg` / `公民链-Windows.exe` / `公民链-Linux-amd.deb` / `公民链-Linux-arm.deb`
  - updater 资产与签名：`citizenchain-updater-linux-amd.AppImage`、`citizenchain-updater-linux-arm.AppImage`、`citizenchain-updater-macos-intel.app.tar.gz`、`citizenchain-updater-macos-apple.app.tar.gz`、`公民链-Windows.exe.sig`
  - updater 元数据：`citizenchain-latest.json`

### 3.2 客户端行为

- App 打开后只调用 updater `check()` 检查 GitHub Release 元数据，不下载、不安装、不重启。
- 如果存在新版本，设置 tab 的“节点程序版本”版本号前显示“更新”按钮。
- 用户点击“更新”后，前端先调用 `prepare_desktop_update` 停止进程内节点，再执行 updater `downloadAndInstall()`，最后 `relaunch()` 重启软件。
- 没有新版本时不显示更新按钮；检查失败不影响本地节点启动和使用。

## 4. 构建机器约束（Tauri 不能跨平台 build）

### 4.1 Windows 构建机

- Windows 10 / 11 x86_64
- Visual Studio 2022 Build Tools（含 MSVC v143 + Windows 10/11 SDK）
- Rust toolchain：`rustup target add x86_64-pc-windows-msvc`
- Node.js 24+
- `cargo install tauri-cli --version "^2"` 或仓库内 `cargo tauri`（取决于工作流）
- Tauri 会自动下载 NSIS 与 WebView2 离线 bootstrapper（首次构建联网，之后走本地缓存）

### 4.2 macOS 构建机

- macOS 13+ (Ventura) Universal 推荐
- Xcode Command Line Tools
- Rust toolchain：`rustup target add aarch64-apple-darwin x86_64-apple-darwin`
- Node.js 24+
- 如需公证（distribution）：Apple Developer ID 证书 + `notarytool` 凭据

### 4.3 Linux 构建机

- Ubuntu 22.04 LTS x86_64（22.04 的 webkit2gtk 兼容性最好）
- 系统包：
  ```bash
  sudo apt install -y \
      libwebkit2gtk-4.1-dev \
      libssl-dev \
      libgtk-3-dev \
      libayatana-appindicator3-dev \
      librsvg2-dev \
      patchelf \
      file
  ```
- Rust toolchain（默认 `x86_64-unknown-linux-gnu`）
- Node.js 24+

## 5. 构建命令

各平台命令一致（因为 target 由 host 决定）：

```bash
cd citizenchain/node
npm install               # 仅首次或 frontend 依赖变化
cargo tauri build --release
```

> ⚠️ 工作区根用了 `[target.x86_64-pc-windows-msvc]` 段加 `+crt-static`，
> 所以 Windows 构建必须从 `citizenchain/` 目录下进入，让 cargo 能读到根 `.cargo/config.toml`。

### 产物路径

| 平台 | 产物 |
|---|---|
| Windows | `citizenchain/target/release/bundle/nsis/citizenchain_<version>_x64-setup.exe` |
| macOS | `citizenchain/target/release/bundle/dmg/citizenchain_<version>_aarch64.dmg`（或 `_x64`） |
| Linux DEB | `citizenchain/target/release/bundle/deb/citizenchain_<version>_amd64.deb` |
| Linux AppImage | `citizenchain/target/release/bundle/appimage/citizenchain_<version>_amd64.AppImage` |

## 6. 验收清单（QA 必做）

### 6.1 Windows（Win10 + Win11 各一次）

- [ ] 干净虚拟机（**未装** VC++ Redistributable、**未装** WebView2）双击 setup.exe
- [ ] `setup.exe` 文件自身在资源管理器中显示 `icons/icon.ico` 对应图标，与 macOS 应用图标同源
- [ ] 安装界面**全程简体中文**
- [ ] 不弹语言选择框
- [ ] 装完后双击 citizenchain 启动，**不再报 MSVCP140.dll 错误**
- [ ] 主窗口标题显示「公民链」，WebView 渲染正常
- [ ] “公民宪法” tab 样式正常，标题与目录不粘连，目录展开/高亮/回到顶部交互可用
- [ ] 有新 GitHub Release 时，设置 tab 的“节点程序版本”版本号前显示“更新”按钮；不点击按钮时不会自动安装
- [ ] 点击“更新”后节点先停止，updater 安装完成后软件自动重启
- [ ] 节点 RPC 监听 `127.0.0.1:9944`
- [ ] 控制面板可正常卸载

#### 静态链接验证

在 Windows 上对产出的 `citizenchain.exe` 跑：

```powershell
dumpbin /dependents citizenchain.exe
```

期望输出**不含**：
- `MSVCP140.dll`
- `VCRUNTIME140.dll`
- `VCRUNTIME140_1.dll`

仅含 Windows 系统 DLL（`KERNEL32`、`USER32`、`ADVAPI32`、`WS2_32` 等）。

### 6.2 macOS（10.15 / 13 / 14 各一次）

- [ ] 双击 DMG，拖到 Applications
- [ ] 首次启动如果系统拦截，「右键 → 打开」放行
- [ ] 不弹「需要 XX 运行库」
- [ ] 设置 tab 只在发现新 Release 时显示“更新”按钮，点击后才安装并重启
- [ ] 节点 RPC 监听 `127.0.0.1:9944`

### 6.3 Linux（最小化 Ubuntu 22.04）

- [ ] **不**预装 webkit2gtk，只跑 `chmod +x xxx.AppImage && ./xxx.AppImage`，能直接启动
- [ ] 标准 Ubuntu 22.04 上 `sudo apt install ./xxx.deb`，能装上
- [ ] AppImage 版本能通过设置 tab “更新”按钮走 updater 安装流程
- [ ] 两种方式启动后节点 RPC 监听 `127.0.0.1:9944`

## 7. 故障排查

| 症状 | 原因 | 处理 |
|---|---|---|
| Windows 启动报 `MSVCP140.dll 缺失` | `+crt-static` 没生效 / 构建未从 `citizenchain/` 目录跑 | 检查产物 `dumpbin /dependents`，确认 `.cargo/config.toml` 路径 |
| Windows 安装界面是英文 | `nsis.languages` 没生效 | 确认 `tauri.conf.json` 的 `bundle.windows.nsis.languages = ["SimpChinese"]` |
| Windows 安装包文件自身图标不对 | NSIS 没显式指定 `installerIcon` / Windows 图标缓存未刷新 | 确认 `bundle.windows.nsis.installerIcon = "icons/icon.ico"`；重新生成安装包后换目录或清理资源管理器图标缓存再看 |
| Windows 公民宪法 tab 目录和中英文标题粘连 | WebView2 CSP 拦截 runtime 宪法 HTML 内联 `<style>` / `<script>` | 确认 `style-src-elem` 与 `script-src-elem` 允许元素级内联，且 iframe 保持 `sandbox=\"allow-scripts\"` |
| Windows 装机时联网下载 WebView2 | `webviewInstallMode` 默认是 `downloadBootstrapper` | 确认改成 `offlineInstaller` |
| push 触发后发布了 Release 或部署服务器 | workflow 缺少 `github.event_name == 'workflow_dispatch'` 边界 | 检查 `publish-github-release` / `deploy-linux-servers` / `cleanup-old-runs` 的 job-level `if` |
| 手动发布缺少 updater 产物或 `.sig` | 没有配置 `GMB_TOP_KEY / GMB_TOP_PUBKEY`，或没有打开 `createUpdaterArtifacts` | 检查 GitHub secrets，并确认 workflow 手动运行时把 `bundle.createUpdaterArtifacts` 临时置为 `true` |
| 设置页不显示“更新”按钮 | `citizenchain-latest.json` 不存在、版本不高于本机版本、或 updater 公钥/签名不匹配 | 检查最新 GitHub Release 资产、`citizenchain-latest.json` 的 `version/platforms/signature/url` 字段和发布密钥 |
| Linux AppImage 启动报缺 webkit | AppImage 打包时未把 webkit 一同打入 | 确认在 Ubuntu 22.04 上构建（旧版本 webkit ABI 不兼容） |
| macOS 启动闪退 | 系统版本低于 10.15 | 升级或换机器 |

## 8. 相关任务卡

- `memory/08-tasks/open/20260505-220000-windows-macos-linux-installer-zero-dep.md`
- `memory/08-tasks/open/20260517-citizenchain-ci-manual-release-desktop-update.md`
