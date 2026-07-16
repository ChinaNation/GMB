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
- 常量直铸数量由 282 更新为 294,后续随 NSN/NRP 常量化更新为 296。
- 创世公权机构总量由 49,581 更新为 49,593。
- 新机构 CID、机构码、主账户、费用账户可被测试验证。
- 源码、文档和注释无旧口径残留;上一轮发布生成物若仍保留 49,581 旧锚点,必须明确标注为历史口径且不得作为当前发布物。

## 执行记录

- 已将 `FDA/NGB/ARM/NAV/AIR/SPF/JOS/ARC/NVC/AFC/SFC/NGC` 12 个国家级公权机构加入 `CHINA_ZF`。
- 已同步 `china_zb.rs` 制度保留地址表:609 → 633 → 637,覆盖新增 12 个机构和 NSN/NRP 的主账户、费用账户。
- 已将 runtime 创世常量直铸计数更新为 296,创世公权机构源码口径保持 49,593。
- 已将国家立法院参议会 `NSN`、国家立法院众议会 `NRP` 从模板派生迁入 `CHINA_LF`,CID、主账户和费用账户沿用原值。
- 已将 FDA 中文全称修正为“公民生活保障部食品药品监督管理局”,简称、英文名、CID 和账户不变。
- 已同步 CitizenApp 公权机构快照中枢省分片的 FDA 全称,并重算 manifest:`public_institution_root=fae09caa31e07cf03953b1a774be72e2614735dce2859a4e2f91fee248955492`。
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
- 当时复查曾确认 `citizenapp/assets/public_institutions/` 与 `CitizenApp/assets/public_institutions/`、`citizenchain/node/chainspecs/`、`citizenchain/target/chainspec/` 和 App 内置 chainspec 还停留在上一轮旧生成物;这些发布生成物残留已在后续 CI WASM 正式 bake、冻结链启动、OnChina 投影同步和快照生成器重跑后清理到 49,593 口径。

## 本轮 NSN/NRP 常量化复查记录

- 已将 `NSN/NRP` 从 `official_derive` 模板派生迁入 `china_lf.rs` 常量,保留原 CID、主账户和费用账户;省级 PSN/PRP 仍保持模板拼接。
- 已删除国家参众议会模板派生残留,派生总数更新为 49,297;常量总数更新为 296;创世总数仍为 49,593。
- 已将 `china_zb.rs` 制度保留地址表更新为 637,覆盖 NSN/NRP 主账户和费用账户。
- 已将 FDA 中文全称和 CitizenApp 快照同步为“公民生活保障部食品药品监督管理局”,并重算中枢省分片 hash 与 `public_institution_root=fae09caa31e07cf03953b1a774be72e2614735dce2859a4e2f91fee248955492`。
- 残留搜索已通过:未再命中旧国家模板、旧派生数量、旧常量数量和 FDA 旧全称等关键口径。
- `cargo fmt --manifest-path citizenchain/runtime/primitives/Cargo.toml --check`:通过。
- 本轮 Rust 文件定向 `rustfmt --edition 2021 --check`:通过。
- `cargo test --manifest-path citizenchain/runtime/primitives/Cargo.toml -- --nocapture`:通过,46 个 primitives 测试和 2 个 golden 测试全绿。
- `cargo test --manifest-path citizenchain/runtime/Cargo.toml genesis_public_institutions_full_mint_counts -- --nocapture`:通过。
- `cargo check --manifest-path citizenchain/onchina/Cargo.toml`:通过。
- `git diff --check`:通过。
- `cargo fmt --manifest-path citizenchain/runtime/Cargo.toml --check`:未作为本轮通过项;报出既有无关 `citizenchain/runtime/src/configs/mod.rs` import 排序差异,本轮未改该文件。

## 上一轮正式 bake 冻结记录（历史）

- 当时已核对 GitHub `CitizenChain WASM` 成功 run `28700551692`,分支 `main`,headSha `21057d4f9459e32ee12cd6aeecc5757038503f64`,与当时本地 HEAD 一致；该记录已被 #99 WASM 冻结替代,仅作历史对照。
- 已下载 artifact `citizenchain-wasm`;`citizenchain.compact.compressed.wasm` 文件大小 1,057,995 字节,sha256 `467a031f7021f46fd18a38963d826a32e085e44503b6b1abe66535b95554fca1`,blake2_256 `0x4c39fdd6aee34329df34b2a66cbae71c1e6b407a3e35af1c90141c7d716921c0`。
- 已执行 `citizenchain/scripts/bake-chainspec.sh --finalize --wasm citizenchain/target/wasm-ci/citizenchain.compact.compressed.wasm`:通过,创世物化耗时 30s,`genesis_hash=0x48275a91dfb46317ebf494ac03a92af97fff78276533f7609660f0298f2a2005`,`state_root=0x93e98c251678ab2b2ac756464787e9123df5965219c2f034b874b5d0be12b3f3`,`chainspec_hash=57e8e641ba0fa371262a6cfcf5ba53a0607a6caca940d16d77729ae45b0cf3de`。
- bake 已同步冻结 SSOT:`citizenchain/node/chainspecs/citizenchain.plain.json` 与 `citizenapp/assets/chainspec.json`;大小写 `CitizenApp/assets/chainspec.json` 与 `citizenapp/assets/chainspec.json` 为同一 inode,已同步到新 `stateRootHash`。
- `target/chainspec/genesis-state/manifest.json` 已生成;后续已启动冻结链、同步 OnChina 投影并由 CitizenApp 快照生成器重生 49,593 快照根,上一轮 `public_institution_root=9e1a8d96737e0668175867ed04ea94e8694c4538b5cdbb4bf435040f360a51c2`,未沿用上一轮 49,581 根。本轮 NSN/NRP 常量化与 FDA 全称修正后,正式发布前必须重新 bake 并重跑链上投影;当前端上快照名称残留已先清理到 `public_institution_root=fae09caa31e07cf03953b1a774be72e2614735dce2859a4e2f91fee248955492`。

## 2026-07-04 #99 WASM 冻结记录（历史）

- GitHub `CitizenChain WASM` #99 / run `28716997121` 的锚点仅保留为历史记录；当前唯一冻结基线已由 2026-07-16 runtime 源提交 `7abac7982a5c5ee25580583d456523ce2132743e`、WASM CI run `29530114067` 的正式 bake 替代，不得继续引用 #99 作为当前发布锚点。
- CitizenApp 轻形态 `citizenapp/assets/chainspec.json` 已同步到 #99 `stateRootHash=0x6a380e96686b152d1eaff8aafc526c23da43058cac2b98be8e98ea1f9e5eff63`;本轮冻结只取最新成功 `CitizenChain WASM` artifact,不等待也不引用 CitizenApp CI。

## 上一轮 CitizenApp 快照生成记录（历史）

- 已用上一轮冻结 chainspec 启动临时节点:`chain_getBlockHash(0)=0x48275a91dfb46317ebf494ac03a92af97fff78276533f7609660f0298f2a2005`,`stateRoot=0x93e98c251678ab2b2ac756464787e9123df5965219c2f034b874b5d0be12b3f3`；该链已被 #99 冻结替代,不得作为当前发布锚点。
- 已用临时 OnChina PostgreSQL 同步链上投影:`chain_institutions=49593`,`chain_accounts=99186`,`local_institutions=49593`,`local_accounts=99186`。
- 已通过 `ONCHINA_BASE_URL=http://127.0.0.1:8975 GEN_DELAY_MS=0 node citizenapp/tools/generate_public_institution_bundle.mjs ...` 生成 43 个省级分片,合计 49,593 个创世公权机构。
- 已逐条抽样确认新增 12 个国家级机构进入 `中枢省.json`:`FDA/NGB/ARM/NAV/AIR/SPF/JOS/ARC/NVC/AFC/SFC/NGC`。
- 已执行 OnChina `audit-chain-catalog` 全量双向对账:本地 49,593 / 链上 49,593 / 不一致 0 / 链上多出 0 / 链上缺失 0。
