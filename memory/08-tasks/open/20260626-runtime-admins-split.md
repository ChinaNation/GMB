# runtime admins 拆分为五类管理员模块

## 任务目标

- 将 `citizenchain/runtime/admins/` 从旧单体管理员模块拆分为：
  - `admin-primitives/`：管理员共用类型、trait、策略、查询抽象，不放业务 storage。
  - `genesis-admins/`：创世管理员，限定国储会、省储会、省储行、联邦注册局。
  - `public-admins/`：非创世公权机构管理员。
  - `private-admins/`：私权机构管理员。
  - `personal-admins/`：个人多签管理员，包含创建、关闭、清理和管理员集合变更。
- 删除 `admin-change/` 合并概念；各类管理员在各自模块内完成更换、激活、关闭、替换等生命周期。
- 常量库中仅保留创世管理员来源；非创世机构管理员来自 CID 录入后上链。
- 更新文档、完善中文注释、清理旧单体管理员、管理员变更、泛机构管理员残留。

## 关键语义

- `admin_root_account_id`：管理员集合所属根账户，机构为主账户。
- `managed_account_id`：被管理的具体账户，包含主账户、费用账户、自定义账户。
- `admin_account_id`：管理员自己的链上账户。
- `institution_cid_number`：机构 CID。
- `admin_cid_number`：管理员公民 CID。

## 验收要求

- runtime 编译通过。
- admins 旧目录和旧命名残留清理完成。
- 文档与代码目录一致。
- 不保留旧单体管理员实现作为影子流程。

## 执行记录

- 已新增 runtime admins 五类目录，并删除旧单体管理员目录与旧个人多签 runtime 目录。
- 已将 `organization-manage` 按机构码路由到 `public-admins` / `private-admins`。
- 已将个人多签链上管理员集合收归 `personal-admins`。
- 已补齐 `PersonalAdmins(7).propose_admin_set_change(3)`，个人多签管理员更换不再依赖旧单体管理员模块。
- 已同步 node、CitizenApp、CitizenWallet 的管理员更换 call data / QR action / 离线解码路由：PMUL=7.3，创世=12.0，公权=29.0，私权=30.0；非法人按所属法人归属路由到公权或私权管理员模块。
- 已将 runtime 统一查询门面改为 `RuntimeAdminAccountQuery`。
- 已修复节点默认 fork-aware 交易池在 fresh / 普通启动场景下触发 `txpool-background` 自退的问题：默认固定为 `SingleState`，并由 `TaskManager` 持有交易池 clone。
- 已更新当前技术文档：`ADMINS_TECHNICAL.md`、跨模块矩阵、MODULE_TAG 注册表、`organization-manage`、`multisig-transfer`、`votingengine` 相关说明。
- 已清理当前 runtime 源码和当前技术文档中的旧管理员模块残留命名。
- 2026-06-27 runtime 二次确认修复：删除 CPOL 市公安局专用 runtime seed / 分类 helper，CPOL 只保留为普通市级公权机构码；从 `china_zf/china_lf/china_sf/china_jc/china_jy` 移除非创世 `admins` 数组，仅保留 `CHINA_CB`、`CHINA_CH` 和 `FEDERAL_REGISTRY_ADMINS` 作为创世管理员来源；将非法人管理员从私权硬绑定中拆出，查询层同时支持公权归属和私权归属。

## 当前验证

```bash
cargo check --manifest-path citizenchain/Cargo.toml -p node
WASM_BUILD_FROM_SOURCE=1 cargo build --manifest-path citizenchain/Cargo.toml -p citizenchain
WASM_FILE=/Users/rhett/GMB/citizenchain/target/debug/wbuild/citizenchain/citizenchain.wasm cargo build --manifest-path citizenchain/Cargo.toml -p node
cargo test --manifest-path citizenchain/Cargo.toml -p genesis-admins -p public-admins -p private-admins -p personal-admins -p organization-manage -p multisig-transfer --lib
cd citizenapp && flutter test test/governance/admins-change/admins_change_codec_test.dart test/governance/admins-change/institution_admin_service_test.dart test/governance/organization-manage/multisig_storage_codec_test.dart test/governance/organization-manage/institution_manage_storage_test.dart test/governance/shared/admin_accounts_scan_service_test.dart test/governance/personal-manage/personal_manage_service_test.dart test/governance/personal-manage/personal_manage_storage_codec_test.dart
cd citizenwallet && flutter test test/signer/pallet_registry_test.dart test/signer/payload_decoder_test.dart test/signer/offline_sign_service_test.dart
CITIZENCHAIN_HEADLESS=1 citizenchain/target/debug/citizenchain --chain citizenchain-fresh --base-path <tmp> --reserved-only --out-peers 0 --in-peers 0 --in-peers-light 0 --no-mdns --no-telemetry --mining-threads 0 --no-gpu
```

运行态验收：使用当前源码重新生成 runtime WASM 后，默认启动路径创建 `txpool_type=SingleState`，fresh 节点初始化 genesis、启动 RPC，并保持运行到脚本主动结束。

## 状态

- 已完成。
