# citizenchain 三平台打包手册

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

## 3. 构建机器约束（Tauri 不能跨平台 build）

### 3.1 Windows 构建机

- Windows 10 / 11 x86_64
- Visual Studio 2022 Build Tools（含 MSVC v143 + Windows 10/11 SDK）
- Rust toolchain：`rustup target add x86_64-pc-windows-msvc`
- Node.js 20+
- `cargo install tauri-cli --version "^2"` 或仓库内 `cargo tauri`（取决于工作流）
- Tauri 会自动下载 NSIS 与 WebView2 离线 bootstrapper（首次构建联网，之后走本地缓存）

### 3.2 macOS 构建机

- macOS 13+ (Ventura) Universal 推荐
- Xcode Command Line Tools
- Rust toolchain：`rustup target add aarch64-apple-darwin x86_64-apple-darwin`
- Node.js 20+
- 如需公证（distribution）：Apple Developer ID 证书 + `notarytool` 凭据

### 3.3 Linux 构建机

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
- Node.js 20+

## 4. 构建命令

三平台命令一致（因为 target 由 host 决定）：

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

## 5. 验收清单（QA 必做）

### 5.1 Windows（Win10 + Win11 各一次）

- [ ] 干净虚拟机（**未装** VC++ Redistributable、**未装** WebView2）双击 setup.exe
- [ ] 安装界面**全程简体中文**
- [ ] 不弹语言选择框
- [ ] 装完后双击 citizenchain 启动，**不再报 MSVCP140.dll 错误**
- [ ] 主窗口标题显示「公民链」，WebView 渲染正常
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

### 5.2 macOS（10.15 / 13 / 14 各一次）

- [ ] 双击 DMG，拖到 Applications
- [ ] 首次启动如果系统拦截，「右键 → 打开」放行
- [ ] 不弹「需要 XX 运行库」
- [ ] 节点 RPC 监听 `127.0.0.1:9944`

### 5.3 Linux（最小化 Ubuntu 22.04）

- [ ] **不**预装 webkit2gtk，只跑 `chmod +x xxx.AppImage && ./xxx.AppImage`，能直接启动
- [ ] 标准 Ubuntu 22.04 上 `sudo apt install ./xxx.deb`，能装上
- [ ] 两种方式启动后节点 RPC 监听 `127.0.0.1:9944`

## 6. 故障排查

| 症状 | 原因 | 处理 |
|---|---|---|
| Windows 启动报 `MSVCP140.dll 缺失` | `+crt-static` 没生效 / 构建未从 `citizenchain/` 目录跑 | 检查产物 `dumpbin /dependents`，确认 `.cargo/config.toml` 路径 |
| Windows 安装界面是英文 | `nsis.languages` 没生效 | 确认 `tauri.conf.json` 的 `bundle.windows.nsis.languages = ["SimpChinese"]` |
| Windows 装机时联网下载 WebView2 | `webviewInstallMode` 默认是 `downloadBootstrapper` | 确认改成 `offlineInstaller` |
| Linux AppImage 启动报缺 webkit | AppImage 打包时未把 webkit 一同打入 | 确认在 Ubuntu 22.04 上构建（旧版本 webkit ABI 不兼容） |
| macOS 启动闪退 | 系统版本低于 10.15 | 升级或换机器 |

## 7. 相关任务卡

- `memory/08-tasks/open/20260505-220000-windows-macos-linux-installer-zero-dep.md`
