# 任务卡：治理投票引擎彻底改名为 voting-engine

## 状态

- done

## 背景

治理投票引擎当前命名过长，需要统一收敛为 `voting-engine`。本次按彻底改名处理，代码、运行时元数据、联动端、脚本、CI、网站展示与正式文档均需同步，完成后不得保留旧命名残留。

## 目标

- 将治理投票引擎源码目录与 Rust crate 名统一为 `voting-engine`。
- 将 runtime 中对外暴露的 pallet 名统一为 `VotingEngine`，保持 pallet index 不变。
- 将 Rust 引用、存储 key 字符串、签名端常量、脚本、CI 与网站展示全部同步。
- 更新 memory 正式文档与任务索引。
- 全仓库搜索确认旧命名残留清零。

## 涉及模块

- `citizenchain/runtime/governance/voting-engine`
- `citizenchain/runtime`
- `citizenchain/node`
- `wuminapp`
- `wumin`
- `.github`
- `website`
- `memory`

## 验收标准

- 全仓库不再出现旧投票引擎命名。
- `cargo` 包依赖解析使用 `voting-engine`。
- runtime pallet index 仍为 9。
- 存储 key 与客户端查询统一使用 `VotingEngine`。
- 相关 Rust、Dart、前端和文档引用一致。

## 完成信息

- 完成时间：2026-04-29
- 完成摘要：已将治理投票引擎彻底收敛为 `voting-engine` / `VotingEngine`，同步 runtime、客户端、冷钱包、CI、脚本、网站与 memory 文档，并清理项目文件与 Cargo 构建缓存内旧命名残留。
- 验证：
  - `cargo check -p voting-engine --offline`
  - `cargo check -p citizenchain --no-default-features --offline`
  - `cargo test -p voting-engine --offline`
  - `flutter test test/signer`
  - `flutter analyze`
  - `cargo clean`
  - 项目文件与路径旧命名残留搜索无结果
