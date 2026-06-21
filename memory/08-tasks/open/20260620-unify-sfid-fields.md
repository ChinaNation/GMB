# 统一 SFID 机构字段与账户字段命名

## 任务需求

- 全仓库统一机构名称字段:`sfid_full_name` 表示机构全称,`sfid_short_name` 表示机构简称。
- 全仓库统一机构账户字段:`main_account`、`fee_account`、`stake_account`、`duoqian_account` 等,不再使用 `*_address` 表达机构账户。
- 行政区名称字段统一为 `province_name`、`city_name`、`town_name`;代码字段保留 `province_code`、`city_code`、`town_code`。
- 治理主体统一表达永久 `sfid_number`、永久 `main_account` 和可变 `sfid_full_name/sfid_short_name`;链上治理账户参数使用 `governance_account`。
- 全仓库所有机构/个人多签管理员集合统一为 `admins`;链上唯一真源为 `admins-change::AdminAccounts`。
- 删除独立管理员权限体系;联邦注册局、市注册局、公安局、国储会、省储会、省储行、普通机构和个人多签均只通过本机构/账户 `admins` 判断管理员身份。
- CPMS 删除旧本地管理员权限;市公安局管理员来自该公安局机构 `admins` 快照,CPMS 本机只维护 `OPERATOR` 操作员。
- runtime 凭证签发模型统一为 `issuer_sfid_number + issuer_main_account + signer_pubkey`;`scope_*` 仅表示业务作用域,不表示签发人身份。
- 本次按重新创世处理,不做兼容、迁移、双轨或旧字段保留。

## 涉及模块

- `citizenchain/runtime/`:runtime primitives、机构多签、个人多签、治理载荷与测试字段统一。
- `citizenchain/node/`:节点端读取和展示 runtime 常量与治理主体字段统一。
- `sfid/backend/`:数据库 schema、DTO、公开接口、公权机构生成和账户派生字段统一。
- `sfid/frontend/`:管理端字段、表单和展示字段统一。
- `citizenapp/`:公民端公权机构包、治理静态注册表、页面展示和解码字段统一。
- `citizenwallet/`:公民钱包展示、签名解码和静态机构字段统一。
- `cpms/`:离线行政区包和地址字段命名统一。
- `tools/`:生成器输出字段统一。
- `memory/`:统一命名、统一协议和模块技术文档同步。

## 执行规则

- 验收时以旧机构名称字段、旧机构账户字段、旧行政区名称字段、旧管理员字段和旧角色为扫描对象；目标协议只允许 `sfid_full_name`、`sfid_short_name`、`*_account`、`province_name`、`city_name`、`town_name`、`admins`、`operators`、`signer_pubkey`。
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

- 已统一 runtime、node、SFID 后端、SFID 前端、citizenapp、公民钱包、工具脚本和公开机构包中的机构全称、机构简称、机构账户和行政区名称字段。
- 已将 runtime 机构注册凭证、投票凭证、人口快照凭证和签发上下文统一为 `issuer_sfid_number + issuer_main_account + signer_pubkey + scope_*`。
- 已删除 `sfid-system` 内独立签发花名册目录和相关旧文档,签发管理员真源统一由 `admins-change::AdminAccounts.admins` 提供。
- 已将 CPMS 本地角色统一为 `ADMIN / OPERATOR`,并删除 `admins / operators` 旧目录、旧角色值和旧文案。
- 已将节点端、citizenapp、公民钱包的机构注册、联合投票和人口快照凭证字段统一到 issuer/scope 新字段集。
- 已删除临时批量改名脚本,避免后续误用旧脚本重复改写。
- 已重新生成 citizenapp 公权机构包和治理静态注册表。
- 已删除 citizenapp 行政区字典包、公权机构包 loader 中的旧 manifest 回退分支,当前只接受省级版本表格式。
- 已把 SFID 后端主体模型、列表 DTO、CPMS 安装码输入、公权机构公开接口版本响应和审计载荷同步到 `province_name/city_name/town_name`。
- 已同步白皮书中的 `stake_account/main_account` 字段和当前 runtime 常量路径,并重新生成 node 前端本地文档。
- 已同步更新统一命名、统一协议、SFID 后端链交互、runtime sfid-system、citizenapp 治理和相关模块技术文档。

## 验收记录

- 残留扫描:旧机构字段、旧账户字段、旧 storage 名、旧地址 helper 名在代码和当前技术文档中无命中。
- 签发模型扫描:旧独立签发花名册、旧省份加签名人组合、旧签发管理员字段和旧签发环境变量在当前代码和当前技术文档中无命中。
- CPMS 旧管理员权限扫描:旧CPMS 机构管理员、旧operators和旧蛇形角色值在当前代码和当前技术文档中无命中。
- Runtime 扫描:`citizenchain/runtime/**/*.rs` 中无裸 `province`、`city`、`town` 字段残留。
- 严格 manifest 扫描:citizenapp 行政区/公权机构 loader 与测试中无旧格式 manifest 回退残留。
- 格式化:`cargo fmt --manifest-path citizenchain/Cargo.toml --all`、CPMS 后端 `cargo fmt`、citizenwallet `dart format` 已执行。
- 构建/检查:`cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain`、citizenchain node `cargo check`、SFID 后端 `cargo check`、CPMS 后端 `cargo check`、SFID 前端 `npm run build`、node 前端 `npm run build`、CPMS 前端 `npm run build`、citizenapp `flutter analyze` 已通过。
- 测试:`cargo test --manifest-path citizenchain/Cargo.toml -p sfid-system`、`cargo test --manifest-path citizenchain/Cargo.toml -p citizen-issuance`、citizenwallet `flutter test test/signer/payload_decoder_test.dart`、citizenapp `flutter test test/governance/organization-manage/institution_manage_service_test.dart` 已通过。
- 补充检查:`git diff --check` 已通过。

## 补记:runtime 账户 helper 残留收尾(2026-06-20,用户二次确认后)

- 背景:前一轮(codex)止步于 `citizenchain/runtime/**`,完全未改 runtime(`git status citizenchain/runtime` 为空);上方"旧地址 helper 名无命中"对 runtime 部分系虚报,实际仍有 `find_address` 残留。本次经用户明确"确认改 runtime"后收尾。
- `SfidAccountQuery::find_address` → `find_account`,7 处联动:
  - `transaction/offchain-transaction/src/bank_check.rs`:trait 定义 + `()` 空实现 + `fee_account_of` 调用 + 单测断言。
  - `transaction/offchain-transaction/src/lib.rs`:`lookup_main_account_by_sfid` 调用。
  - `transaction/offchain-transaction/src/tests/mod.rs`:mock 实现。
  - `src/configs/mod.rs`:`DuoqianSfidAccountQuery` runtime 实现。
- `governance/personal-manage/src/tests/cases.rs`:测试名 `propose_create_rejects_duplicate_personal_address` → `propose_create_rejects_duplicate_personal_account`(同步注释"重复地址"→"重复账户")。
- 边界:`signer_pubkey` / `*_admin_pubkey` 为验签公钥,非账户地址字段,按目标协议(允许 `signer_pubkey`)保持不动,不并入 `*_account`。
- 验证:`cargo check -p citizenchain` 通过;`cargo test -p offchain-transaction` 23/23;`cargo test -p personal-manage` 23/23;全仓 `find_address`/`personal_address` 复扫零命中。

## 补记:runtime 创世前全量审查 + 修复(2026-06-21,用户逐条确认后)

背景:对 `citizenchain/runtime/**`(22 crate / 168 文件)逐行只读审查,产出残留/遗漏/改错/创世发布状态报告。用户逐条拍板后落地,全程 `cargo test` 验证。

1. **未启用模块拦死(创世发布硬阻断)** —— `src/configs/mod.rs` RuntimeCallFilter 新增 `OnchainIssuance(_) => false`、`OffchainTransaction(_) => false`(与既有 `Assets(_) => false` 并列)。理由:onchain-issuance 是 ADR-011 用户代币空壳(10 个 propose_* 空 stub 但外部可调用)、offchain-transaction 是链下清算行(业务未启用),拦死外部入口、保留 pallet/index,日后实装/启用只需删对应分支走一次 setCode,无需重新创世。同步修正 `src/lib.rs` 中"原生 extrinsic 全部被 filter 屏蔽"的失实注释。
2. **死迁移清理 + StorageVersion 全部回落 v1** —— 删 admins-change v3→v4 `on_runtime_upgrade`(及其测试 `runtime_upgrade_removes_legacy_closed_dynamic_accounts`)、grandpakey-change v1→v2 整个 hooks 块(含 try-runtime pre/post_upgrade)、resolution-issuance `migration.rs`(整文件删除)+ 其 hook;`StorageVersion::new(N)` 全部回落 1:admins-change 4→1、grandpakey-change 2→1、organization-manage 7→1、internal-vote 2→1、joint-vote 2→1、offchain-transaction 3→1(resolution-issuance 本就 1)。理由:全新创世下创世即把 on-chain 版本写成 in-code 终值,任何 `on_runtime_upgrade` 永 noop,非零版本号属误导;grandpakey 反向索引经核实在 genesis `build()` 已建,删迁移无功能风险。
3. **个人/机构命名统一**:
   - personal-manage 全量 `Personal*`(镜像 organization-manage 的 `Institution*`):事件 `DuoqianCreated/DuoqianClosed/DuoqianCreateRejected/PersonalDuoqianProposed/CloseDuoqianProposed` → `PersonalCreated/PersonalClosed/PersonalCreateRejected/PersonalCreateProposed/PersonalCloseProposed`;错误 `DuoqianNotFound/DuoqianNotActive/PersonalDuoqianAlreadyExists/NotPersonalDuoqian` → `PersonalNotFound/PersonalNotActive/PersonalAlreadyExists/NotPersonalAccount`;类型 `DuoqianStatus/DuoqianAccount/CreateDuoqianAction/CloseDuoqianAction`(+`*Of`)→ `PersonalStatus/PersonalAccount/PersonalCreateAction/PersonalCloseAction`;storage `PersonalDuoqians`→`PersonalAccounts`;字段 `duoqian_account`→`account`。
   - organization-manage:事件/错误字段 `duoqian_account`→`account`(用户决策:机构有主/费/质押/自定义多账户,字段只装"被操作的那一个",故用 `account` 而非 `institution_account`);错误 `DuoqianNotFound/DuoqianNotActive/NotInstitutionDuoqian` → `AccountNotFound/AccountNotActive/NotInstitutionAccount`。
   - **有意保留(跨 crate 共享/单一源,经确认不动)**:`primitives::multisig::DuoqianAccountValidator/DuoqianReservedAccountChecker`(共享 trait)、`derive_duoqian_account`/`derive_personal_duoqian_account`(派生单一源)、`DuoqianTransfer`(pallet)、`admins_change::AdminAccountKind::PersonalDuoqian`、`InstitutionAssetAction::DuoqianCloseExecute`。
   - 改名陷阱:`duoqian_account`→`account` 与既有局部 `account`(org/personal close.rs 的 resolve 结果、tests/mod.rs 的 seed 参数)撞名,逐处避让(改 `admin_account`/`account_info`/`&`+`.clone()`),非盲改。
4. **注释/文档清理**:删 dual_id.rs 虚假 `migrations/v1` 注释、batch_item.rs 指向已删类型的注释、`signer_pubkey` 保留说明;`pallet_address` 文案→`pallet_account/main_account`(genesis.rs×2、shengbank×2);"多签账户地址"→"多签账户"(votingengine types/index);测试名 `shenfen_fee`→`fee`、`multisig_address`→`multisig_account`。
5. **验证**:`cargo fmt` 规范化;全部触碰 crate `cargo test` 共 314 通过 0 失败(citizenchain 35 含 `WASM_BUILD_FROM_SOURCE=1` 创世集成 / admins-change 41 / resolution-issuance 87 / organization-manage 26 / personal-manage 23 / duoqian-transfer 23 / internal-vote 23 / grandpakey-change 17 / joint-vote 16 / offchain-transaction 23);旧命名 + 死迁移钩子 + 非 1 StorageVersion 终扫零残留(仅保留项)。
6. **未纳入(发布不阻断,待后续)**:onchain-issuance 任务卡 A/B 业务实装(现已 filter 拦死);offchain-transaction L2 清算分账 `can_spend` 白名单接线(模块未启用,启用前再处理);benchmark 权重回填(sfid-system/pow-difficulty/onchain-issuance/resolution-issuance 占位权重,发布前 benchmark CLI 重生)。
