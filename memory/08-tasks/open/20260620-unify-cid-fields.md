# 统一 CID 机构字段与账户字段命名

## 任务需求

- 全仓库统一机构全称/简称字段:`cid_full_name` 表示机构全称,`cid_short_name` 表示机构简称。
- 全仓库统一机构账户字段:`main_account`、`fee_account`、`stake_account`、`account` 等,不再使用 `*_address` 表达机构账户。
- 行政区名称字段统一为 `province_name`、`city_name`、`town_name`;代码字段保留 `province_code`、`city_code`、`town_code`。
- 治理主体统一表达永久 `cid_number`、永久 `main_account` 和可变 `cid_full_name/cid_short_name`;链上治理账户参数使用 `governance_account`。
- 全仓库所有机构/个人多签管理员集合统一为 `admins`;链上唯一真源为 `admins-change::AdminAccounts`。
- 删除独立管理员权限体系;联邦注册局、市注册局、公安局、国储会、省储会、省储行、普通机构和个人多签均只通过本机构/账户 `admins` 判断管理员身份。
- CPMS 删除旧本地管理员权限;市公安局管理员来自该公安局机构 `admins` 快照,CPMS 本机只维护 `OPERATOR` 操作员。
- runtime 凭证签发模型统一为 `issuer_cid_number + issuer_main_account + signer_pubkey`;`scope_*` 仅表示业务作用域,不表示签发人身份。
- 本次按重新创世处理,不做兼容、迁移、双轨或旧字段保留。

## 涉及模块

- `citizenchain/runtime/`:runtime primitives、机构多签、个人多签、治理载荷与测试字段统一。
- `citizenchain/node/`:节点端读取和展示 runtime 常量与治理主体字段统一。
- `citizencode/backend/`:数据库 schema、DTO、公开接口、公权机构生成和账户派生字段统一。
- `citizencode/frontend/`:管理端字段、表单和展示字段统一。
- `citizenapp/`:公民端公权机构包、治理静态注册表、页面展示和解码字段统一。
- `citizenwallet/`:公民钱包展示、签名解码和静态机构字段统一。
- `citizenpassport/`:离线行政区包和地址字段命名统一。
- `scripts/`:生成器输出字段统一。
- `memory/`:统一命名、统一协议和模块技术文档同步。

## 执行规则

- 验收时以旧机构全称/简称字段、旧机构账户字段、旧行政区名称字段、旧管理员字段和旧角色为扫描对象；目标协议只允许 `cid_full_name`、`cid_short_name`、`*_account`、`province_name`、`city_name`、`town_name`、`admins`、`operators`、`signer_pubkey`。
- 不新增兼容分支、旧字段别名、过渡格式或迁移适配。
- 涉及 `citizenchain/runtime/**` 的管理员/凭证统一修改已经单独列出路径和原因,并已获得用户二次确认。
- 改代码后必须同步文档、完善必要中文注释并清理残留。

## 验收计划

- 全仓库搜索旧字段残留。
- 全仓库搜索并删除旧注册局角色、旧 CPMS 角色、旧签发花名册和旧凭证字段残留。
- 执行受影响生成器,重新生成静态数据包和代码生成物。
- 运行各模块格式化、类型检查、测试或构建。
- 涉及数据库和公开包的部分用真实 SQLite/JSON 数据检查字段结构。

## 执行记录

- 已统一 runtime、node、CID 后端、CID 前端、citizenapp、公民钱包、工具脚本和公开机构包中的机构全称、机构简称、机构账户和行政区名称字段。
- 已将 runtime 机构注册凭证、投票凭证、人口快照凭证和签发上下文统一为 `issuer_cid_number + issuer_main_account + signer_pubkey + scope_*`。
- 已删除 `cid-system` 内独立签发花名册目录和相关旧文档,签发管理员真源统一由 `admins-change::AdminAccounts.admins` 提供。
- 已将 CPMS 本地角色统一为 `ADMIN / OPERATOR`,并删除 `admins / operators` 旧目录、旧角色值和旧文案。
- 已将节点端、citizenapp、公民钱包的机构注册、联合投票和人口快照凭证字段统一到 issuer/scope 新字段集。
- 已删除临时批量改名脚本,避免后续误用旧脚本重复改写。
- 已重新生成 citizenapp 公权机构包和治理静态注册表。
- 已删除 citizenapp 行政区字典包、公权机构包 loader 中的旧 manifest 回退分支,当前只接受省级版本表格式。
- 已把 CID 后端主体模型、列表 DTO、CPMS 安装码输入、公权机构公开接口版本响应和审计载荷同步到 `province_name/city_name/town_name`。
- 已同步白皮书中的 `stake_account/main_account` 字段和当前 runtime 常量路径,并重新生成 node 前端本地文档。
- 已同步更新统一命名、统一协议、CID 后端链交互、runtime cid-system、citizenapp 治理和相关模块技术文档。
- 已在用户二次确认后完成 runtime 侧统一:`otherpallet/cid-system` 替换旧目录,`CidSystem` 替换旧 pallet 命名,发行、治理、交易和测试全部改为 CID 命名。
- 已将活跃代码中的旧多签账户字段统一为 `account`;链端共享派生函数改为 `derive_account`,个人多签派生函数改为 `derive_personal_account`;CID 后端 `accounts` 表只保留 `account` 列。
- 已删除 CID 后端旧 schema 兼容块,不再保留旧行政区缩写列和旧账户列迁移逻辑。
- 已将公民钱包签名解码中的旧注册局角色动作统一为 `CREATE_ADMIN / UPDATE_ADMIN / DELETE_ADMIN`。

## 验收记录

- 残留扫描:旧机构字段、旧账户字段、旧 storage 名、旧地址 helper 名在代码和当前技术文档中无命中。
- 签发模型扫描:旧独立签发花名册、旧省份加签名人组合、旧签发管理员字段和旧签发环境变量在当前代码和当前技术文档中无命中。
- CPMS 旧管理员权限扫描:旧CPMS 机构管理员、旧operators和旧蛇形角色值在当前代码和当前技术文档中无命中。
- Runtime 扫描:`citizenchain/runtime/**/*.rs` 中无裸 `province`、`city`、`town` 字段残留。
- 字段级残留扫描:旧账户字段、旧注册局角色动作、旧行政区缩写字段、名称字段旧写法均无命中;保留项仅限行政区层级枚举、钱包/交易/网络地址字段。
- 严格 manifest 扫描:citizenapp 行政区/公权机构 loader 与测试中无旧格式 manifest 回退残留。
- 活跃代码扫描:旧身份系统命名、旧多签账户字段、旧管理员角色值均无命中。
- 真实运行态验收:重建本地 `citizencode` 空库后启动 `./citizencode/citizencode-run.sh`,完成 245716 条公权机构和 491475 个账户初始化;`/api/v1/health` 返回 UP;前端 `http://localhost:5179/` 返回 200;数据库 `accounts` 表只存在 `account` 列。
- 格式化:`cargo fmt --manifest-path citizenchain/Cargo.toml --all`、CPMS 后端 `cargo fmt`、citizenwallet `dart format` 已执行。
- 构建/检查:`cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain`、citizenchain node `cargo check`、CID 后端 `cargo check`、CPMS 后端 `cargo check`、CID 前端 `npm run build`、node 前端 `npm run build`、CPMS 前端 `npm run build`、citizenapp `flutter analyze` 已通过。
- 测试:`cargo test --manifest-path citizenchain/Cargo.toml -p cid-system`、`cargo test --manifest-path citizenchain/Cargo.toml -p citizen-issuance`、citizenwallet `flutter test test/signer/payload_decoder_test.dart`、citizenapp `flutter test test/governance/organization-manage/institution_manage_service_test.dart` 已通过。
- 补充检查:`git diff --check` 已通过。
- 本轮补充测试:`cargo test --manifest-path citizenchain/runtime/Cargo.toml -p organization-manage` 26/26、`-p personal-manage` 23/23、`-p duoqian-transfer` 23/23 通过。

## 补记:runtime 账户 helper 残留收尾(2026-06-20,用户二次确认后)

- 背景:前一轮(codex)止步于 `citizenchain/runtime/**`,完全未改 runtime。本次经用户明确"确认改 runtime"后完成 runtime 账户 helper 收尾。
- `CidAccountQuery` 统一为 `find_account`,7 处联动:
  - `transaction/offchain-transaction/src/bank_check.rs`:trait 定义 + `()` 空实现 + `fee_account_of` 调用 + 单测断言。
  - `transaction/offchain-transaction/src/lib.rs`:`lookup_main_account_by_cid` 调用。
  - `transaction/offchain-transaction/src/tests/mod.rs`:mock 实现。
  - `src/configs/mod.rs`:`DuoqianCidAccountQuery` runtime 实现。
- `governance/personal-manage/src/tests/cases.rs`:重复个人账户测试名和注释统一为 account 口径。
- 边界:`signer_pubkey` / `*_admin_pubkey` 为验签公钥,非账户地址字段,按目标协议(允许 `signer_pubkey`)保持不动,不并入 `*_account`。
- 验证:`cargo check -p citizenchain` 通过;`cargo test -p offchain-transaction` 23/23;`cargo test -p personal-manage` 23/23;全仓账户旧字段复扫零命中。

## 补记:runtime 创世前全量审查 + 修复(2026-06-21,用户逐条确认后)

背景:对 `citizenchain/runtime/**`(22 crate / 168 文件)逐行只读审查,产出残留/遗漏/改错/创世发布状态报告。用户逐条拍板后落地,全程 `cargo test` 验证。

1. **未启用模块拦死(创世发布硬阻断)** —— `src/configs/mod.rs` RuntimeCallFilter 新增 `OnchainIssuance(_) => false`、`OffchainTransaction(_) => false`(与既有 `Assets(_) => false` 并列)。理由:onchain-issuance 是 ADR-011 用户代币空壳(10 个 propose_* 空 stub 但外部可调用)、offchain-transaction 是链下清算行(业务未启用),拦死外部入口、保留 pallet/index,日后实装/启用只需删对应分支走一次 setCode,无需重新创世。同步修正 `src/lib.rs` 中"原生 extrinsic 全部被 filter 屏蔽"的失实注释。
2. **死迁移清理 + StorageVersion 全部回落 v1** —— 删 admins-change v3→v4 `on_runtime_upgrade`(及其测试 `runtime_upgrade_removes_legacy_closed_dynamic_accounts`)、grandpakey-change v1→v2 整个 hooks 块(含 try-runtime pre/post_upgrade)、resolution-issuance `migration.rs`(整文件删除)+ 其 hook;`StorageVersion::new(N)` 全部回落 1:admins-change 4→1、grandpakey-change 2→1、organization-manage 7→1、internal-vote 2→1、joint-vote 2→1、offchain-transaction 3→1(resolution-issuance 本就 1)。理由:全新创世下创世即把 on-chain 版本写成 in-code 终值,任何 `on_runtime_upgrade` 永 noop,非零版本号属误导;grandpakey 反向索引经核实在 genesis `build()` 已建,删迁移无功能风险。
3. **个人/机构命名统一**:
   - personal-manage 全量使用 `Personal*` 目标态命名，事件、错误、类型、storage 和字段均以 `PersonalAccount` / `PersonalCreate` / `PersonalClose` / `account` 为唯一口径。
   - organization-manage:事件/错误字段统一为 `account`(机构有主/费/质押/自定义多账户,字段只装"被操作的那一个",故用 `account` 而非 `institution_account`);错误 `DuoqianNotFound/DuoqianNotActive/NotInstitutionDuoqian` → `AccountNotFound/AccountNotActive/NotInstitutionAccount`。
   - 字段和 helper 已统一为 `account`、`derive_account`、`derive_personal_account`。
   - 多签业务模块名保留为 `duoqian-transfer` / `DuoqianTransfer`,只作为业务域名,不再作为账户字段名。
4. **注释/文档清理**:删 dual_id.rs 虚假 `migrations/v1` 注释、batch_item.rs 指向已删类型的注释、`signer_pubkey` 保留说明;账户相关文案统一为 `account`/`main_account` 口径。
5. **验证**:`cargo fmt` 规范化;全部触碰 crate `cargo test` 共 314 通过 0 失败(citizenchain 35 含 `WASM_BUILD_FROM_SOURCE=1` 创世集成 / admins-change 41 / resolution-issuance 87 / organization-manage 26 / personal-manage 23 / duoqian-transfer 23 / internal-vote 23 / grandpakey-change 17 / joint-vote 16 / offchain-transaction 23);旧命名 + 死迁移钩子 + 非 1 StorageVersion 终扫零残留(仅保留项)。
6. **未纳入(发布不阻断,待后续)**:onchain-issuance 任务卡 A/B 业务实装(现已 filter 拦死);offchain-transaction L2 清算分账 `can_spend` 白名单接线(模块未启用,启用前再处理);benchmark 权重回填(cid-system/pow-difficulty/onchain-issuance/resolution-issuance 占位权重,发布前 benchmark CLI 重生)。

## 补记:citizenapp 账户命名残留收尾(2026-06-21)

- 个人账户派生、个人账户 storage key、个人账户本地 Isar entity、注册机构账户 identity helper、个人账户 identity helper 和页面展示文案统一为 `account`/账户口径。
- `PersonalManage` 客户端 storage 名称统一读取 `PersonalAccounts`;Isar 生成文件通过 `dart run build_runner build` 重新生成,避免手工改 generated schema 导致 collection id 错误。
- `WalletIsarMigration` schema version 行改为按 key 原地 upsert,重复打开/迁移测试库时不再触发唯一索引冲突。
- 文档同步清理旧账户字段、旧账户派生 helper、旧账户 identity helper 和旧地址文案。
- 验证:`flutter analyze` 通过;citizenapp 账户派生、identity、admins-change、个人账户 storage、个人提案历史、机构账户 storage、公民端公开机构详情相关测试在 `flutter test --concurrency=1 ...` 下 52/52 通过;`git diff --check` 通过;目标旧字段残留扫描 0 命中。

## 补记:runtime 账户命名二次确认收尾(2026-06-21)

- 已按用户二次确认修改 `citizenchain/runtime/**`: `AdminAccountKind::PersonalAccount` 作为个人多签账户主体名;旧个人账户主体名、旧注册个人账户测试 helper、旧保留账户 trait 名和旧 storage getter 全部清理。
- `primitives::multisig` 的保留账户检查 trait 统一为 `ReservedAccountGuard`;runtime 配置实现统一为 `RuntimeReservedAccountGuard`。
- node 侧注册机构账户输入前缀同步统一为 `institution-account:`,并清理 node、脚本和样式注释中的旧地址式账户表述。
- citizenapp 两个个人账户页面文件同步改为 `personal_account_create_page.dart` / `personal_account_close_page.dart`,并同步 import、技术文档和历史任务卡路径。
- 中文账户文案中的旧地址式口径同步收敛为“账户/多签账户/主账户”,保留钱包、收付款、网络地址等真实地址语义。
- 验证:`cargo fmt --manifest-path citizenchain/runtime/Cargo.toml` 相关包通过;`cargo test -p admins-change` 41/41、`-p personal-manage` 23/23、`-p organization-manage` 26/26、`-p duoqian-transfer` 23/23、`-p internal-vote` 87/87 通过;`cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain` 通过;node 侧用临时 `TAURI_CONFIG` 指向已有资源目录完成 `cargo check --manifest-path citizenchain/node/Cargo.toml`;`flutter analyze` 通过;citizenapp 相关账户测试串行通过;账户旧命名和中文旧地址文案残留扫描 0 命中;`git diff --check` 通过。
