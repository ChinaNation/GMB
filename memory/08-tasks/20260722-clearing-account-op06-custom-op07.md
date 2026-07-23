# 任务卡：新增机构协议账户「清算账户」（op_tag=0x06），自定义账户改到 0x07

任务需求：
在账户派生单源新增一类机构协议账户「清算账户」，op_tag = 0x06；原「自定义命名账户」由 0x06 改到 0x07。
私法人股份公司（机构码 SFGF）注册时，除主账户、费用账户外，自动多派生并登记一个清算账户。
五端（链端权威源 / onchina-CID / CitizenApp-Dart 两文件）逐字节对齐，定死进创世 WASM。

所属模块：
- citizenchain/runtime/primitives（账户派生权威源 + 约束）
- citizenchain/runtime/entity/private-manage（注册期建账）
- citizenchain/runtime/genesis（创世 seeder）
- citizenchain/onchina（注册局派生镜像 + 建账）
- citizenapp/lib/citizen/shared（Dart 派生镜像）

op_tag 定稿：
- 0x00 主账户 / 0x01 费用账户 / 0x02 永久质押 / 0x03 安全基金 / 0x04 两和基金 / 0x05 个人多签（不变）
- 0x06 = 清算账户（OP_CLEARING，payload = cid_number）——本次由 0x06 原语义改为清算账户
- 0x07 = 自定义账户（OP_NAME，payload = cid_number ‖ account_name）——由 0x06 挪到 0x07
- 清算账户地址 = blake2_256(b"GMB" ‖ 0x06 ‖ ss58_le(2027) ‖ cid_number)
- 保留名 RESERVED_NAME_CLEARING = "清算账户"，禁止作自定义账户注册

资格：
- required_protocol_account_kinds(SFGF) = [主账户, 费用账户, 清算账户]
- 其余机构不变 [主账户, 费用账户]

必须遵守：
- 不可突破模块边界
- 不可绕过既有契约（派生单源、五端逐字节对齐）
- 本步只让账户「存在且被正确生成」，不改任何资金流
- 范围外（创世后 runtime 升级再做）：充值/提现改落清算账户、can_spend 只准行间流通、结算改路径、手续费收链上费

输出物：
- 代码（五端）
- 中文注释
- 金标向量测试（清算账户 0x06 + 自定义账户 0x07 新位）
- 文档更新
- 残留清理

验收标准：
- 同一 SFGF cid_number 在链端/onchina/Dart 派生清算账户地址完全一致（golden 锁定）
- 自定义账户在三端都落 0x07
- 注册 SFGF 后链上 InstitutionAccounts 出现主/费/清算三账户；其余机构仍主/费
- 「清算账户」不能被注册为自定义账户名
- cargo build / cargo test 通过；dart analyze / Dart 测试通过
- 资金流 / can_spend / 结算逻辑零改动

落地记录（2026-07-22 完成）：
- 链端权威源 account_derive.rs：OP_CLEARING=0x06、OP_NAME=0x07、Clearing 种类 + AccountKind::InstitutionClearing + 全 match 补全 + 保留名禁注册。
- code.rs：新增独立常量 `pub const SFGF`。
- institution_constraints.rs：CORPORATION_PROTOCOL_ACCOUNT_KINDS=[Main,Fee,Clearing]，SFGF 分支。
- 注册期/创世：build_required_protocol_accounts + seeder 泛型遍历约束表，自动建清算账户，零改动。
- onchina：accounts/derive.rs（保留名 6、文档表）、subjects/service.rs、subjects/model.rs、node chain.rs、citizenapp/public_institution.rs 注释 —— 各加 Clearing 分支/更新 op_tag 说明。
- Dart：account_derivation.dart（kOpClearing=0x06、kOpName=0x07、deriveInstitutionClearingAccountId、路由 switch）、reserved_account_names.dart（保留名 6、禁注册）。
- 金标：account_derive_golden.rs 加 InstitutionClearing 分支；fixture 经 scripts/sync-derive-vectors.sh --write 重生并同步 Dart 副本。
- 派生真值：清算账户(SFGF, 0x06)=4b4a69b7…；自定义账户(0x07)由 4a5b09fa… 刷新为 2577fee2…；china 来源账户地址不变（行为中性）。
- 验证：cargo check --workspace 通过；primitives 73+ 测试通过；Dart 派生金标/交叉/ss58 全通过。
- 状态：完成（账户模型层）。资金流/can_spend/结算留创世后 runtime 升级（见下一步方案）。
