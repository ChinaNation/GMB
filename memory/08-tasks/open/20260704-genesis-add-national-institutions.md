# 20260704 创世补齐国家级公权机构

## 任务需求

1. 将第 1 步已补齐机构码的 12 个国家级公权机构加入创世公权机构常量。
2. 保持机构码、CID、主账户、费用账户、中文全称、中文简称、英文全称和英文简称单源一致。
3. 更新创世总量、技术文档、ADR 和相关注释。
4. 清理上一轮旧创世数量口径和 `CHINA_ZF` 旧计数残留。

## 明确确认

- 用户已确认创建本任务卡。
- 用户已确认本轮涉及 `citizenchain/runtime/` 的 runtime 创世常量修改。

## 范围

- `citizenchain/runtime/primitives/cid/china/china_zf.rs`
- `citizenchain/runtime/primitives/cid/china/china_zb.rs`
- `citizenchain/runtime/primitives/cid/china/mod.rs`
- `citizenchain/runtime/genesis/src/institution.rs`
- `citizenchain/runtime/src/tests/cases.rs`
- `memory/04-decisions/`
- `memory/05-modules/citizenchain/runtime/`
- `memory/05-modules/citizenchain/node/`
- `memory/08-tasks/`

## 边界

- 本任务只把既有 12 个机构码加入创世常量。
- 不新增机构码。
- 不恢复旧格式、旧命名、旧创世流程或本地派生真源。
- 不推送 GitHub，不触发远端 CI/CD。

## 验收

- 12 个国家级公权机构进入 `CHINA_ZF` 并由创世直铸。
- `CHINA_ZF` 数量由 59 更新为 71。
- 常量直铸数量由 282 更新为 294。
- 创世公权机构总量由 49,581 更新为 49,593。
- 新机构 CID、机构码、主账户、费用账户可被测试验证。
- 源码、文档和注释无旧口径残留;上一轮发布生成物若仍保留 49,581 旧锚点,必须明确标注为历史口径且不得作为当前发布物。

## 执行记录

- 已将 `FDA/NGB/ARM/NAV/AIR/SPF/JOS/ARC/NVC/AFC/SFC/NGC` 12 个国家级公权机构加入 `CHINA_ZF`。
- 已同步 `china_zb.rs` 制度保留地址表:609 → 633,覆盖新增 12 个机构的主账户和费用账户。
- 已将 runtime 创世常量直铸计数更新为 294,创世公权机构源码口径更新为 49,593。
- 已补 primitives 级 `china_zb` 与内置账户全量一致性测试,防止后续新增常量机构时漏同步保留表。
- 已补 runtime 创世测试:新增 12 个国家级机构逐个入链,并验证主/费用账户均进入制度保留地址守卫。
- 已更新 ADR、runtime primitives、node、CitizenApp 和相关 open 任务卡文档;上一轮 49,581 bake 锚点已标注为历史口径,不得作为当前发布锚点。

## 验证记录

- `cargo fmt --manifest-path citizenchain/runtime/primitives/Cargo.toml --check`:通过。
- `cargo fmt --manifest-path citizenchain/runtime/Cargo.toml --check`:通过。
- `cargo test --manifest-path citizenchain/runtime/primitives/Cargo.toml -- --nocapture`:通过,46 个 primitives 测试全绿。
- `cargo test --manifest-path citizenchain/runtime/Cargo.toml genesis_public_institutions_full_mint_counts -- --nocapture`:通过,创世直铸总数 49,593 且新增 12 机构入链。
- `cargo check --manifest-path citizenchain/runtime/Cargo.toml`:通过。
- `cargo check --manifest-path citizenchain/onchina/Cargo.toml`:通过。
- `citizenchain/scripts/bake-chainspec.sh`:通过,本地源码 WASM 预览创世物化耗时 31s,`genesis_hash=0x9d71d312aa7d1dde786a5bcfd31a6d81b613b9b2c32b12502e550f68fd2d4a5b`,`state_root=0x591b7efdc80d15342f28a70cb15ded469c28de6bd5c6c918c780c547e13f8355`;预览模式未覆盖正式冻结 SSOT。

## 二次复查记录

- 已复核机构码表、创世常量和制度保留地址表:`FDA/NGB/ARM/NAV/AIR/SPF/JOS/ARC/NVC/AFC/SFC/NGC` 12 个机构码均存在,`CHINA_ZF` 为 71 条,新增机构的主账户和费用账户均命中制度保留地址表。
- 已修复 `citizenapp/tools/generate_public_institution_bundle.mjs` 中仍写 49,581 的生成器注释残留,同步更新为 49,593。
- 已复查第 44 条“国聘公职人员”残留:现行真源未再命中,仅任务卡历史记录保留“已修复”描述。
- 已清理立法体系文档残留:ADR 和 open 任务卡中“参议会/众议会是两个独立机构”“省参议会/省众议会”等旧表述,已改为国家立法院/省联邦立法院下设参议会和众议会。
- 已删除未跟踪临时审计脚本 `citizenchain/scripts/runtime-audit-wf.js`,避免被误提交为正式脚本。
- 复查确认 `citizenapp/assets/public_institutions/` 与 `CitizenApp/assets/public_institutions/` 仍是上一轮 49,581 发布快照;`citizenchain/node/chainspecs/`、`citizenchain/target/chainspec/` 和 App 内置 chainspec 仍对应上一轮冻结锚点。这些属于正式发布生成物残留,不得手工改 JSON 冒充新快照,必须在同一提交 CI WASM 确定后重新 `--finalize` bake、同步 OnChina 投影并重跑快照生成器。
