# 20260502 SFID institutions 粗粒度整合

## 任务目标

- 将不属于机构模块的匿名证书能力从 `sfid/backend/institutions` 拆出，直接并入 `sfid/backend/cpms` 根目录。
- 将 `institutions` 模块保持为粗粒度文件结构：`model/store/service/handler/derive/chain_duoqian_info`，避免按小 handler/service 过度拆分。
- 更新相关文档、补充中文注释并清理残留引用。

## 改动范围

- `sfid/backend/institutions`
- `sfid/backend/cpms`
- `sfid/backend/main.rs`
- `sfid/backend/citizens`
- `memory/05-modules/sfid`

## 验收标准

- 旧机构匿名证书目录被移除，匿名证书引用改为 `cpms::rsa_blind`。
- `institutions` 目录只保留机构属性文件和机构链查询文件。
- 后端格式化、编译检查通过。
- 文档说明与实际目录一致。

## 状态

- 已完成。

## 完成记录

- 旧机构匿名证书目录已移除，RSA 盲签名能力直接归入 `sfid/backend/cpms/rsa_blind.rs`。
- 旧链查询 DTO 与 handler 拆分文件已按属性整合进 `chain_duoqian_info.rs`。
- `institutions` 后端目录已收敛为粗粒度机构文件。
- 已更新 SFID 后端目录、链交互归属、机构模块、CPMS 模块文档。
- 已执行 `cargo fmt --manifest-path sfid/backend/Cargo.toml`、`cargo check --manifest-path sfid/backend/Cargo.toml` 和 `npm run build`。
