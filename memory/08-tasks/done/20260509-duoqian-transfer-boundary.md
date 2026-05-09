# 2026-05-09 duoqian-transfer 多签转账独立目录整理

## 任务需求

按用户确认后的模块边界执行多签转账目录整理：

- `duoqian-transfer` 是多签转账独立模块，runtime、node 前端、node 后端、wuminapp 都必须按独立目录实现。
- `wuminapp/lib/proposal/` 不允许保留多签转账页面、service、model、投票逻辑、余额检查、进度条或入口代码。
- `wuminapp/lib/duoqian/` 不允许实现多签转账页面、service、model、投票逻辑、余额检查或进度条；账户页只允许挂载所属业务模块提供的入口组件。
- `citizenchain/node/src/governance/` 和 `citizenchain/node/frontend/governance/` 不允许保留多签转账实现。
- `citizenchain/node/src/offchain/` 和 `citizenchain/node/frontend/offchain/` 不允许保留多签转账残留，清算行只负责清算行功能。
- wuminapp 端全部多签转账实现集中到 `wuminapp/lib/duoqian-transfer/`。
- node 后端全部多签转账实现集中到 `citizenchain/node/src/duoqian_transfer/`。
- node 前端全部多签转账实现集中到 `citizenchain/node/frontend/duoqian-transfer/`。
- 同步修复管理员钱包余额不足提示、投票 pending 状态、投票进度条快照读取和错误文档。

## 影响范围

- `wuminapp/lib/duoqian-transfer/`
- `wuminapp/lib/proposal/`
- `wuminapp/lib/duoqian/`
- `wuminapp/lib/vote/`
- `wuminapp/lib/institution/`
- `citizenchain/node/src/duoqian_transfer/`
- `citizenchain/node/src/governance/`
- `citizenchain/node/src/offchain/`
- `citizenchain/node/src/desktop/`
- `citizenchain/node/frontend/duoqian-transfer/`
- `citizenchain/node/frontend/governance/`
- `citizenchain/node/frontend/offchain/`
- `memory/05-modules/`
- `memory/07-ai/`

## 风险点

- `propose_transfer(19.0)` 和 `InternalVote::cast(22.0)` 的 SCALE 字段顺序必须与 runtime 和 wumin 解码保持一致。
- node 后端 Rust 模块目录必须使用 `duoqian_transfer`，前端和 wuminapp 目录按用户确认使用 `duoqian-transfer`。
- wuminapp 的 `duoqian` 仍负责个人/机构多签账户注册、注销、管理；迁移时不能误删管理功能。
- node 端只支持治理机构和注册多签机构账户，不支持个人多签转账。
- 现有投票 pending 和进度条问题涉及链上状态、交易池状态和提案快照，必须用真实状态退出 loading。

## 执行状态

- [x] 创建任务卡
- [x] 建立 wuminapp `duoqian-transfer` 独立目录并迁移多签转账实现
- [x] 清理 wuminapp `proposal` 与 `duoqian` 中的多签转账实现残留
- [x] 建立 node 后端 `duoqian_transfer` 独立目录并迁移多签转账实现
- [x] 建立 node 前端 `duoqian-transfer` 独立目录并迁移多签转账实现
- [x] 清理 node `governance` 与 `offchain` 多签转账残留
- [x] 修复余额提示、投票 pending 和进度条快照
- [x] 更新统一命名、统一协议和模块技术文档
- [x] 运行测试和残留扫描

## 验证记录

- `flutter analyze lib/duoqian-transfer lib/proposal/shared lib/proposal/proposal_types_page.dart lib/duoqian lib/institution/institution_detail_page.dart lib/vote/vote_view.dart`（`wuminapp`）：通过。
- `flutter test test/duoqian`（`wuminapp`）：通过。
- `npm run build`（`citizenchain/node/frontend`）：通过。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo check --manifest-path citizenchain/node/Cargo.toml --bin citizenchain`：通过，保留既有 dead_code warning。
- `cargo test --manifest-path citizenchain/Cargo.toml -p duoqian-transfer --lib`：22 个测试通过。
- `flutter test test/signer/payload_decoder_test.dart test/signer/pallet_registry_test.dart`（`wumin`）：通过。
