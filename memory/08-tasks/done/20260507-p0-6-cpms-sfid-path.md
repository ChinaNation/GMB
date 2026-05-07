# P0-6 CPMS 旧 SFID 路径与 Wumin spec 残留清理

## 任务目标

执行重新创世前总审计 P0-6：清理 CPMS 编译脚本中已失效的 SFID 旧核心规则路径，并同步清理 Wumin 本地脚本与 CI 中已删除的 `supportedSpecVersions` 写源码残留。

## 当前真源

- SFID 代码目录：`sfid/backend/sfid/`
- SFID 禁止恢复目录：旧后端源码壳
- Wumin 冷钱包规则：`supportedSpecVersions / isSupported` 已物理移除，冷钱包不再通过 spec_version 集合门控拒签。

## 预计修改目录

- `cpms/backend/`：修正 build 脚本和省市码生成注释中的 SFID 真实路径；涉及构建脚本与代码注释。
- `wumin/scripts/`：删除本地启动脚本中已失效的 `supportedSpecVersions` 写源码逻辑；涉及脚本残留清理。
- `.github/workflows/`：删除 Wumin CI 中同类 `supportedSpecVersions` 写源码逻辑；涉及流程残留清理。
- `memory/07-ai/`：登记 `sfid/backend/sfid/` 目录真源和禁止恢复目录；只涉及统一命名文档。
- `memory/08-tasks/open/`：记录 P0-6 执行范围、结果和验收；只涉及文档。

## 执行清单

- [x] 将 CPMS 默认 SFID 路径改为 `../../sfid/backend/sfid`。
- [x] 将 CPMS 环境变量提示改为 `/path/to/sfid/backend/sfid`。
- [x] 更新 CPMS 省市码生成注释中的旧路径。
- [x] 删除 `wumin/scripts/wumin-run.sh` 中 `supportedSpecVersions` sed 残留。
- [x] 删除 `.github/workflows/wumin-ci.yml` 中 `supportedSpecVersions` sed 残留。
- [x] 在统一命名文件登记 `sfid/backend/sfid/` 目录真源。
- [x] 回写审计文档并运行验收。

## 验收标准

- `cargo check --manifest-path cpms/backend/Cargo.toml` 通过，证明 build.rs 能找到 `province.rs` 和 `city_codes/`。
- `bash -n wumin/scripts/wumin-run.sh` 通过。
- 旧核心规则路径与 `supportedSpecVersions` 写源码逻辑不再命中旧可执行残留。
- `git diff --cached --check` 通过。

## 执行结果

2026-05-07 已执行：

- `cpms/backend/build.rs` 默认读取路径已改为 `../../sfid/backend/sfid`。
- `cpms/backend/build.rs` 错误提示已改为 `CPMS_SFID_DIR=/path/to/sfid/backend/sfid`。
- `cpms/backend/src/dangan/province_codes.rs` 注释已同步为新路径。
- `wumin/scripts/wumin-run.sh` 已删除链上 `spec_version` 读取与 `supportedSpecVersions` 写源码逻辑。
- `.github/workflows/wumin-ci.yml` 已删除同类 `supportedSpecVersions` 写源码逻辑。
- `memory/07-ai/unified-naming.md` 已登记 `sfid/backend/sfid/` 为 SFID 核心规则源码目录，并禁止恢复旧后端源码壳等旧边界目录。

验收记录：

- `cargo check --manifest-path cpms/backend/Cargo.toml`：通过；当前 CPMS 仍有 19 个既有 warning，未纳入本次 P0-6 范围。
- `bash -n wumin/scripts/wumin-run.sh`：通过。
- 旧核心规则路径与 `supportedSpecVersions` 写源码逻辑扫描：无输出。
- `git diff --cached --check`：通过。
