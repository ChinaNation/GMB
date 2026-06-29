# 20260629 genesis-manage 与创世管理员重构

## 任务目标

- 新增 `runtime/entity/genesis-manage`，把创世机构本体从公权机构生命周期中拆出。
- 创世机构信息在创世时写入链上独立 storage，并提供查询/保护能力。
- 保留 `admins/genesis-admins` 管理创世机构管理员，并把联邦注册局管理员从 215 人平铺集合改为省级分组治理。
- OnChina 生成联邦注册局管理员更换 QR 时按目标省 5 人分组处理，不再使用全体 215 人集合。

## 边界

- 不把创世机构实体逻辑放入 `runtime/genesis`，该模块继续只承载链级创世参数。
- 不把创世机构写入 `public-manage` 或 `private-manage` 的普通机构生命周期。
- 不保留旧的 FRG 全量管理员更换流程。

## 验收要点

- 创世机构信息、主账户和费用账户在 `genesis-manage` 中可查。
- `GenesisAdmins` 创世写入仍覆盖 NRC、PRC、PRB、FRG，但 FRG 支持省级分组。
- FRG 管理员更换只影响同省 5 人分组，阈值来自代码级固定阈值 `FRG=3`。
- OnChina 只为目标省 FRG 分组生成管理员更换 call data。
- 相关技术文档同步更新。

## 完成记录

- 状态：已完成。
- `runtime/entity/genesis-manage` 已新增为创世机构本体模块，创世写入 CHINA_CB、CHINA_CH、CHINA_ZF 的创世机构信息、主账户、费用账户和封存索引。
- `admins/genesis-admins` 继续管理创世管理员；FRG 不再写为 215 人平铺 `AdminAccounts`，改为 43 个省级 5 人组，阈值统一来自代码级固定阈值 `FRG=3`。
- `primitives` 已把 FRG 纳入 NRC/PRC/PRB 同级固定治理码，`internal-vote` 只接受 FRG 省级 5 人组作为内部投票主体，不允许 FRG 主账户暴露成 215/3 多签配置。
- FRG 管理员更换新增 `GenesisAdmins.propose_federal_registry_province_admin_set_change(12.2)`，旧 `12.0` 通用入口对 FRG 明确拒绝。
- OnChina 的 FRG 管理员更换 QR 改为按管理员所在省读取 5 人组并编码 `0x0c02`。
- node / CitizenApp 的通用管理员更换编码器均拒绝 FRG，避免继续生成旧 `12.0` call data。
- runtime、QR 注册表、node 管理员更换文档、创世/机构/管理员相关技术文档已同步更新。

## 验收命令

- `cargo check -p genesis-manage --manifest-path citizenchain/Cargo.toml`
- `cargo test -p genesis-manage --manifest-path citizenchain/Cargo.toml --lib`
- `cargo check -p primitives --manifest-path citizenchain/Cargo.toml`
- `cargo test -p primitives --manifest-path citizenchain/Cargo.toml --lib`
- `cargo check -p genesis-admins --manifest-path citizenchain/Cargo.toml`
- `cargo test -p genesis-admins --manifest-path citizenchain/Cargo.toml --lib`
- `cargo test -p internal-vote --manifest-path citizenchain/Cargo.toml --lib`
- `cargo check -p citizenchain --manifest-path citizenchain/Cargo.toml`
- `cargo check -p node --manifest-path citizenchain/Cargo.toml`
- `cargo test -p node --manifest-path citizenchain/Cargo.toml admin_management -- --nocapture`
- `cargo check -p onchina --manifest-path citizenchain/Cargo.toml`
- `cargo test -p onchina --manifest-path citizenchain/Cargo.toml institution_call -- --nocapture`
- `flutter test test/governance/admins-change/admins_change_codec_test.dart`
- `git diff --check`

## 运行态补充验收

- 已尝试短时启动本地无头 dev 节点：`CITIZENCHAIN_HEADLESS=1 cargo run -p node --manifest-path citizenchain/Cargo.toml -- --chain dev --tmp --no-telemetry --mining-threads 0 --rpc-port 19944 --port 30399`。
- 启动被既有冻结 chainspec 阻塞，错误为 `护宪守卫:创世不可修改条款基准派生失败:ConstitutionLawMissing`；`--chain dev` 在当前 node 中映射到冻结 raw chainspec，不是本次 FRG 固定阈值逻辑报错。
- 已尝试 fresh chainspec 导出：`cargo run -p node --manifest-path citizenchain/Cargo.toml -- export-chain-spec --chain citizenchain-fresh --raw`。
- fresh 创世导出被本地构建条件阻塞，错误为 `fresh genesis 需要 WASM_BINARY；请通过 WASM_FILE 指向最新 CI WASM 后再构建`。
