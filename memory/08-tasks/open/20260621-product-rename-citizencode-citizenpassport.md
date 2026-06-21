# 任务卡：CitizenCode / CitizenPassport 全仓统一改名

## 任务需求

- `cid` 系统统一改名为 CitizenCode。
  - 中文全称：公民身份识别码系统
  - 英文全称：Citizen Identity Code System
  - 简称：CitizenCode
  - 缩写：CID
- `cpms` 系统统一改名为 CitizenPassport。
  - 中文全称：公民护照管理系统
  - 英文全称：Citizen Passport Management System
  - 简称：CitizenPassport
  - 缩写：CPMS
- 前端包名统一为 `citizencode-web` / `citizenpassport-web`。
- 仓库目录、模块文档目录、CI 路径、部署安装路径统一为全小写：`citizencode` / `citizenpassport`。
- 删除旧 `cid` / `CID` 产品命名、旧 `cpms` / `CPMS` 产品命名残留；`CPMS` 只允许作为 CitizenPassport 的缩写存在。

## 执行边界

- 顶层目录改为 `citizencode/` 与 `citizenpassport/`。
- 产品展示名、文档标题和 UI 文案继续使用 `CitizenCode` / `CitizenPassport`；只要不是路径、包名、服务名、artifact 名或机器标识，不按目录残留处理。
- 非 runtime 代码、脚本、文档、注释、包名和协议描述同步更新。
- `citizenchain/runtime/**` 修改遵守 runtime 二次确认硬规则。

## 验收要求

- 全仓路径、包名、文档和注释不得残留旧产品名。
- 全仓不得残留旧大写仓库目录、旧大写文档目录、旧大写部署安装路径或旧大写包文件名。
- `cid_number` 等身份识别码字段统一为 `cid_number`。
- `CID_CPMS_V1` 统一为 `CID_CPMS_V1`。
- 完成后执行真实构建、分析和残留扫描。

## 执行记录

- 已将仓库顶层目录统一为 `citizencode/`、`citizenpassport/`。
- 已将 `memory/01-architecture/` 与 `memory/05-modules/` 下的对应模块目录统一为 `citizencode/`、`citizenpassport/`。
- 已将 CI、脚本、部署配置、安装路径和文档中的目录路径统一为小写。

## 当前验收记录

- `npm run build`（`citizencode/frontend`）通过。
- `npm run build`（`citizenpassport/frontend`）通过。
- `cargo fmt --manifest-path citizenpassport/backend/Cargo.toml --check && cargo check --manifest-path citizenpassport/backend/Cargo.toml` 通过。
- 脚本语法检查通过：`citizenpassport/scripts/build_linux_host_installer.sh`、`citizenpassport/deploy/linux/*.sh`、`citizencode/citizencode-run.sh`、`citizenpassport/citizenpassport.sh`、`citizencode/deploy/prod/scripts/*.sh`。
- `cargo check --manifest-path citizencode/backend/Cargo.toml` 仍被 runtime 未统一字段阻断：runtime primitives 仍导出 `SFID_NUMBER_MAX_BYTES`、`sfid_number`、`sfid_full_name`，而 CitizenCode 后端已统一调用 `CID_NUMBER_MAX_BYTES`、`cid_number`、`cid_full_name`。该阻断必须按 `citizenchain/runtime/**` 二次确认规则处理。
