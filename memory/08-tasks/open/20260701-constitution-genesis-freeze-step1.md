# 宪法创世冻结与法律生效时间改造第 1 步

## 任务目标

- 在正式创世前闭环宪法创世冻结：创世只写入一部公民宪法，`law_id=0`，`v1` 直接生效。
- 把法律版本状态从旧的版本号减一推导改为显式字段：生效版本、最新版本、待生效版本。
- 把立法/修法的 `effective_at` 从旧的区块高度口径改为“生效时间戳”，用户发起提案时只选择生效时间。
- 投票通过后才写入新版本；如果生效时间已到则立即生效，否则进入待生效状态，到时间自动生效。
- 补齐 chainspec 烘焙脚本和宪法创世检查脚本，但本步骤不生成正式 raw、不推 GitHub、不正式创世。

## 修改范围

- `citizenchain/runtime/public/legislation-yuan/`：法律主表版本字段、生效时间调度、创世宪法写入和单测。
- `citizenchain/runtime/primitives/src/`：清理宪法旧真源残留。
- `citizenchain/scripts/`：新增 chainspec 烘焙入口与宪法创世检查脚本。
- `citizenchain/node/src/` 与 `citizenchain/node/frontend/`：桌面端宪法读取默认取已生效版本，解码字段对齐新结构。
- `citizenchain/onchina/`：立法提案 DTO、SCALE 编码、链读取和前端发起弹窗改为生效时间语义。
- `citizenapp/`：法律模型、SCALE 解码和法律阅读页改为生效时间语义。
- `memory/`：更新 ADR、模块文档和本任务卡。

## 边界铁律

- 本步骤只做正式创世前的代码、脚本、文档、测试闭环；不提交正式 `citizenchain.raw.json`。
- 正式 raw 必须等待代码推送 GitHub 且 WASM CI 成功后，用 CI WASM 烘焙。
- 不保留旧的区块高度生效 UI、注释、文档或交易载荷口径。
- 不恢复第二套宪法真源；链上结构化宪法是唯一真源。
- 投票流程仍归投票引擎，立法院模块只在投票终态回调后写法律版本。

## 验收要求

- `cargo test -p legislation-yuan --manifest-path citizenchain/Cargo.toml` 通过。
- `cargo test --manifest-path citizenchain/node/Cargo.toml constitution` 通过。
- `cargo test -p onchina --manifest-path citizenchain/onchina/Cargo.toml` 或等效 OnChina 测试通过。
- CitizenApp 相关 Dart 测试/静态检查能覆盖模型解码改动；如环境限制需记录原因。
- OnChina 前端类型检查/构建通过，且旧区块高度生效文案残留搜索为 0。
- 本地临时 raw 可被 `check-constitution-genesis.py` 校验出宪法创世键；正式 raw 不在本步骤写入。

## 进度

- [x] 用户确认新增任务卡与脚本文件。
- [x] 用户确认修改 runtime 路径。
- [x] 修改 runtime 法律版本状态与生效时间调度。
- [x] 更新桌面端、OnChina、CitizenApp 解码与展示。
- [x] 补齐 chainspec 脚本和宪法创世校验脚本。
- [x] 更新文档、完善注释、清理残留。
- [x] 执行测试和运行态/脚本验收。

## 验收记录

- `cargo test -p legislation-yuan --manifest-path citizenchain/Cargo.toml`：23/23 通过。
- `cargo test -p node --manifest-path citizenchain/Cargo.toml constitution`：21/21 通过。
- `cargo test -p onchina --manifest-path citizenchain/Cargo.toml law`：27/27 通过。
- `npm run build`（`citizenchain/onchina/frontend`）：通过。
- `flutter test test/legislation/legislation_codec_test.dart`（`citizenapp`）：5/5 通过。
- `citizenchain/scripts/bake-chainspec.sh --out citizenchain/target/chainspec/constitution-preview.raw.json`：预览 raw 导出成功，`check-constitution-genesis.py` 校验通过；未覆盖正式 SSOT。
- `python3 citizenchain/scripts/check-constitution-genesis.py citizenchain/node/chainspecs/citizenchain.raw.json || true`：当前冻结 SSOT 仍缺 `Laws[0]`，符合“正式 raw 等 GitHub WASM CI 后再烘焙”的边界。
