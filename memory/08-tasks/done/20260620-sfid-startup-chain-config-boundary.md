# SFID 启动与链端签发配置边界修复

## 任务需求

SFID 是由联邦注册局运维的中心化独立系统,部署后应先能启动、登录和管理公权机构等本地业务。区块链只是 SFID 的链交互对象,链端签发身份不得成为 SFID 后端启动前置条件。

## 问题现状

- `sfid-backend` 启动入口强制读取 `SFID_RUNTIME_ISSUER_SFID_NUMBER`、`SFID_RUNTIME_ISSUER_MAIN_ACCOUNT`、`SFID_RUNTIME_SIGNER_PUBKEY`。
- 本地 `.env.dev.local` 未配置这些链端签发身份时,`./sfid-run.sh` 在 `ensure-gov` 阶段直接 panic。
- 部署文档和安装模板没有清楚区分 SFID 基础启动必填项与链交互启用项。

## 目标状态

- SFID 后端启动只校验本地运行必需配置。
- 链端签发身份只在投票凭证、人口快照凭证、机构链注册凭证等链交互路径按需校验。
- 链端签发配置缺失时,链接口返回明确的服务不可用错误,不杀死 SFID 进程。
- 部署模板和技术文档同步说明配置边界。

## 影响范围

- `sfid/backend/main.rs`:移除启动入口中的链端签发身份强制校验。
- `sfid/backend/core/chain_runtime.rs`:保留链交互按需校验,增加链配置错误分类。
- `sfid/backend/citizens/*`:链投票相关接口缺链配置时返回 503。
- `sfid/backend/subjects/*`:机构链注册凭证接口缺链配置时返回 503。
- `sfid/deploy/prod`:更新生产 env 模板注释。
- `memory/05-modules/sfid`:更新部署说明。

## 验收标准

- 未配置 `SFID_RUNTIME_ISSUER_*` 时,`./sfid-run.sh` 不再因启动入口 panic。
- 普通 SFID 后端健康检查能通过。
- 未配置链签发身份时,链交互接口返回配置未完成错误,普通 SFID 本地功能不受影响。

## 完成记录

- 已移除 `main.rs` 启动入口对 `SFID_RUNTIME_ISSUER_*` / `SFID_RUNTIME_SIGNER_PUBKEY` 的强制校验。
- 已将链端签发配置缺失错误收口到链交互接口 503 响应。
- 已补齐管理员、CPMS、主体分区表旧字段到目标字段的启动期幂等收敛。
- 已更新生产部署模板和部署说明,明确链配置不阻断 SFID 基础站点启动。
- 已执行 `cargo check --manifest-path backend/Cargo.toml --package sfid-backend`。
- 已执行 `./sfid-run.sh` 真实启动验收:后端健康检查 `/api/v1/health` 返回 `UP`,前端 `http://localhost:5179/` 返回 200。
