# 2026-05-09 admins_change 管理员更换 node 与 wuminapp 整理

## 任务需求

按确认后的目录设计执行管理员更换模块整理：

- node 节点端前后端都在各自 `admins_change` 目录承载管理员更换业务。
- 管理员激活属于管理员管理，统一移动到 node 后端 `admins_change/activation.rs`。
- wuminapp 不区分传统前后端，不新建 backend，管理员更换作为一级业务模块放在 `wuminapp/lib/admins_change/`。
- wuminapp 的管理员激活服务迁入 `lib/admins_change/services/admin_activation_service.dart`。
- wuminapp 的 `InstitutionAdminService` 查询门面迁入 `lib/admins_change/services/`，旧 `lib/institution/institution_admin_service.dart` 不保留。
- 清理 `wuminapp/lib/proposal/admin_change/` 空占位。
- `offchain/organization-manage` 详情页“换管理员”按钮进入 `governance/admins_change` 管理员更换流程。
- 更新技术文档，记录目录边界和实现入口。

## 影响范围

- `citizenchain/node/src/governance/admins_change/`
- `citizenchain/node/frontend/governance/admins_change/`
- `citizenchain/node/src/governance/`
- `citizenchain/node/src/desktop/`
- `citizenchain/node/frontend/governance/`
- `citizenchain/node/frontend/offchain/`
- `citizenchain/node/frontend/offchain/organization-manage/`
- `wuminapp/lib/admins_change/`
- `wuminapp/lib/institution/institution_activation_service.dart`
- `wuminapp/lib/institution/institution_admin_service.dart`
- `wuminapp/test/admins_change/`
- `wuminapp/lib/proposal/admin_change/`
- `memory/05-modules/citizenchain/node/admins-change/`
- `memory/05-modules/wuminapp/admins-change/`

## 风险点

- `AdminsChange.propose_admin_set_change` 的 call data 必须与 runtime pallet index / call index / SCALE 编码保持一致。
- 现有管理员读取逻辑散在 institution 与 duoqian shared，需要迁移时保留兼容入口。
- wuminapp 只有 Flutter 客户端与 Rust FFI，不允许误建传统 backend 目录。

## 执行状态

- [x] 建立 node 后端 `admins_change` 目录并迁移管理员主体能力
- [x] 将 node 管理员激活迁入 `governance/admins_change/activation.rs`
- [x] 建立 node 前端 `admins_change` 目录并接入换管理员页面
- [x] 将 node 前端激活 API 从根 `governance/api.ts` 收口到 `governance/admins_change/api.ts`
- [x] 将 `offchain/organization-manage` 的“换管理员”按钮接入 `AdminSetChangePage`
- [x] 建立 wuminapp `lib/admins_change` 一级模块
- [x] 将 wuminapp 管理员激活服务迁入 `lib/admins_change/services/admin_activation_service.dart`
- [x] 将 wuminapp `InstitutionAdminService` 门面迁入 `lib/admins_change/services/`
- [x] 清理旧占位和更新文档
- [x] 运行必要验证

## 验证记录

- `npm run build`（`citizenchain/node/frontend`）：通过。
- `flutter analyze lib/admins_change lib/institution lib/proposal lib/vote lib/duoqian/shared test/admins_change`（`wuminapp`）：通过。
- `flutter test test/admins_change`（`wuminapp`）：通过。
- `rustfmt --edition 2021 --check citizenchain/node/src/governance/admins_change/activation.rs citizenchain/node/src/governance/admins_change/mod.rs citizenchain/node/src/governance/mod.rs citizenchain/node/src/desktop/mod.rs citizenchain/node/src/offchain/settlement/admin_unlock.rs`：通过。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo test --manifest-path citizenchain/Cargo.toml -p node admins_change`：继续编译，但被既有 offchain 问题阻断：
  - `node/src/offchain/duoqian_transfer/mod.rs` 缺失。
  - `node/src/offchain/settlement/packer.rs` 访问 `OffchainLedger.inner` 私有字段。

上述 Rust 阻断项不属于本次 `admins_change` 边界。
