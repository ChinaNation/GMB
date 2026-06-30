# 20260630 地址链上同步模块

## 状态

- 已完成

## 任务需求

把镇下地址库的链上同步能力落为可执行技术实现：链上只记录地址变更事实、当前版本和当前哈希；OnChina 本地保留随安装包发布的 `china.sqlite`，并提供地址查询与链上地址变更 call data 构造能力。

## 目标范围

- 新增 `address-registry` runtime pallet，承载地址目录版本、地址名称和完整地址的 set/remove 事件。
- FRG 管本省、CREG 管本市的权限判断复用现有 runtime 管理员与注册局口径。
- 新增 OnChina `address` 业务模块，读取本地 SQLite 地址库并构造链上调用。
- 新增 OnChina 前端地址管理入口，展示地址名称与完整地址，并能生成链写 call data。
- 更新 runtime / OnChina 技术文档，清理旧地址上链方案残留。

## 不做范围

- 不在本任务中实现链事件监听自动写回本地 SQLite。
- 不保留旧地址历史、墓碑表、双轨兼容或旧字段。
- 不新增独立 `backend/src`、`frontend/api`、`frontend/chain` 或全局链目录。

## 验收要求

- `cargo check --manifest-path citizenchain/Cargo.toml -p address-registry`
- `cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain`
- `cargo check --manifest-path citizenchain/Cargo.toml -p onchina`
- `npm --prefix citizenchain/onchina/frontend run build`
- `python3 citizenchain/onchina/src/cid/china/check_code_immutable.py`
- `sqlite3 citizenchain/onchina/src/cid/china/china.sqlite "PRAGMA integrity_check"`
- 残留搜索确认没有旧地址表、墓碑、变更日志链上方案重新出现。

## 完成记录

- 新增 `address-registry` pallet 并挂入 runtime pallet index `35`。
- 新增 OnChina 地址查询与 `AddressRegistry` call data 构造 API。
- 新增 OnChina 前端地址库页面。
- 更新 runtime / OnChina / QR / 统一协议 / ADR / AI 规则文档。
- 验证通过：`cargo check -p address-registry`、`cargo check -p citizenchain`、`cargo check -p onchina`、`npm --prefix citizenchain/onchina/frontend run build`、`check_code_immutable.py`、`PRAGMA integrity_check`。
