# 任务卡：修复 Windows NSIS 安装包文件自身图标未显式指定的问题

## 任务需求

Windows 端 `citizenchain_<version>_x64-setup.exe` 安装包文件本身在资源管理器中显示的图标需要与 macOS 端应用图标保持一致。当前桌面图标已经正确，本任务只修复 NSIS 安装包 exe 自身图标。

## 影响范围

- `citizenchain/node/tauri.conf.json`：在 `bundle.windows.nsis` 中显式指定 `installerIcon`。
- `memory/05-modules/citizenchain/node/CROSS_PLATFORM_BUILD.md`：补充 Windows 安装包文件图标验收项和故障排查说明。
- 不修改 runtime、前端业务代码、桌面快捷方式图标逻辑。

## 修复目标

- NSIS setup.exe 文件自身使用 `citizenchain/node/icons/icon.ico`。
- 安装后桌面图标仍沿用现有 `bundle.icon` 配置，不改变原有正确行为。
- 文档明确区分“安装包文件图标”和“安装后应用/桌面图标”。

## 验收方式

- `tauri.conf.json` JSON 语法有效。
- 残留扫描确认 NSIS 文档不再缺少安装包文件图标验收项。
- Windows 端重新打包后，在资源管理器查看 setup.exe 图标应与 macOS 应用图标同源。

## 状态

- [x] 创建任务卡
- [x] 修改 NSIS 安装包图标配置
- [x] 更新打包文档
- [x] 残留检查
