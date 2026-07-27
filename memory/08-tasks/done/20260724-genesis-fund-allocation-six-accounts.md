# 创世发行分账给六个指定账户

> 状态：已完成并随 2026-07-26 正式创世生效；CitizenApp 的 FCSF 协议账户派生与余额
> 展示错误已修复并通过全量验收，任务重新归档。

## 任务目标

- 从创世发行 `GENESIS_ISSUANCE`(144,349,737,800.00 元)中切出 **700 亿元**,创世一次性预置给 6 个指定账户;创世发行总量不变(方案 A,从国家储委会主账户份额扣减)。
- 6 笔分账(单位:元 / 分,1 元=100 分):
  - 联邦公民安全基金 100 亿元(1_000_000_000_000 分)
  - 公民程伟钱包 200 亿元(2_000_000_000_000 分)
  - 技术发展基金会主账户 100 亿元(1_000_000_000_000 分)
  - 联邦注册局主账户 100 亿元(1_000_000_000_000 分)
  - 国家司法院主账户 100 亿元(1_000_000_000_000 分)
  - 国储会安全基金 100 亿元(1_000_000_000_000 分)
- 国家储委会主账户余额 = `GENESIS_ISSUANCE - 19×1000万元 - 700亿元` = 7_415_973_780_000 分(741.6 亿元),仍为正。
- 白皮书 3.1 创世发行章节简明扼要补充分账说明。

## 落点来源(链上真源,SS58 前缀 2027)

| 账户 | 金额(分) | 来源 |
|---|---|---|
| 国储会安全基金 | 1_000_000_000_000 | `primitives::cid::china::china_cb::SAFETY_FUND_ACCOUNT` |
| 联邦注册局主账户 | 1_000_000_000_000 | `CHINA_ZF` 中 `FRG` 机构 `main_account` |
| 国家司法院主账户 | 1_000_000_000_000 | `CHINA_SF[0]`(国家司法院)`main_account` |
| 技术发展基金会主账户 | 1_000_000_000_000 | `citizenchain::CITIZENCHAIN_FOUNDATION.main_account` |
| 公民程伟钱包 | 2_000_000_000_000 | `citizenchain::CITIZENCHAIN_GENESIS_ADMINS[0].account_id`(`9c3e…1068`) |
| 联邦公民安全基金 | 1_000_000_000_000 | `account_derive`:`OP_FCSF` + 联邦安全局(FSC)`cid_number`,与 seeder 登记地址一致,禁硬抄 |

## 修改范围

- `citizenchain/runtime/src/genesis.rs`:`build_genesis()` 新增 6 笔 balances、NRC 主账户扣减 700 亿;更新受影响创世单测断言并新增分账断言。
- `citizenweb/src/whitepaper.md`:3.1 创世发行章节 + 发行汇总表创世发行行,简明扼要补充分账口径(中英双语)。
- `memory/`:本任务卡 + MEMORY.md 记忆索引。

## 边界与约束

- 仅改创世初始余额,不动 spec_version / pallet index / call index / 签名载荷,不触发 citizenapp/citizenwallet 双端联动。
- 联邦公民安全基金地址必须走 `account_derive` 派生唯一真源,与创世 seeder(FSC 特判)登记地址完全一致。
- 已按死规则核对金额与单位换算。

## 验收要求

- `cargo test`(genesis 相关 runtime 包)通过,含新增分账断言。
- 白皮书正文渲染无误。

## 决策记录(用户确认)

- 资金来源:从 NRC 主账户切出,创世发行总量不变。
- 程伟钱包地址:`9c3e18f575c59236832054469ef0e69f16a1fe6c50b2b580fc7c71853ab71068`。
- 管理员公钥复核(同轮第二件事):用户确认仅结构校验(无需外部基准)。

## 进度

- 已改 `citizenchain/runtime/src/genesis.rs`:6 笔分账 + NRC 主账户扣减 + 3 处单测(数量断言、NRC 断言改准确名 `genesis_issuance_splits_nrc_main_and_six_allocations`、新增 `genesis_six_allocations_have_exact_amounts`)。
- 已改 `citizenweb/src/whitepaper.md`:3.1 创世发行 + 发行汇总表(中英双语)。
- 已重生成节点内置白皮书 `citizenchain/node/frontend/local-docs.generated.ts`(纯离线 `generate-local-docs.mjs`)。
- 已修 `GENESIS_TECHNICAL.md` 第 32 行陈旧基金会账户(主/费账户按代码订正)。

## 第二件事:管理员公钥畸形(已发现并修复)

- **先前"公钥全部正确"结论错误,已更正**。真实缺陷:`citizenchain/runtime/primitives/cid/china/china_zf.rs:22`(`FEDERAL_REGISTRY_ADMINS` 中枢省第 4 个)公钥只有 **63 位十六进制**:`d6d73cfd…e393`,`hex!` 编译期 panic `Odd number of hex characters`,阻塞 primitives/runtime 全部编译。
- 漏检原因:结构校验用 `[0-9a-fA-F]{64}` 正则**静默跳过**畸形值,导致 FRG 统计 214≠215 被误判为解析器 off-by-one。教训已入记忆 [[hex-key-length-assert-exact]]。
- 修复:用户提供正确 64 位公钥 `d6d73cfd7d6b7c5692749b7c46fd3fe398f16f84283910dbf15f74472e1e3938`(原值末尾补 `8`),已补齐。复核:全仓 china_*.rs 全部恰好 64 位、该值全局唯一、FRG=215=43×5。

## 验收结果(2026-07-24 通过)

- `cargo test --manifest-path citizenchain/runtime/Cargo.toml genesis::` → **8 passed; 0 failed**,含 `genesis_issuance_splits_nrc_main_and_six_allocations`、`genesis_six_allocations_have_exact_amounts`、`genesis_contains_nrc_and_all_provincialbank_balances`。
- runtime 全 crate 编译通过(primitives 补齐后)。

## 2026-07-24 追加(用户三项要求)

1. **新增断言**:[genesis.rs](../../../citizenchain/runtime/src/genesis.rs) 测试模块加 `governance_admin_keys_unique_and_counts_match_seats`——治理机构管理员公钥数量匹配席位(NRC19/PRC9×43/PRB9×43/NJD15/FRG省数×5/基金会1)+ 全局唯一。说明:64 位长度由 `hex!`+`[u8;32]` 编译期已强制,故断言聚焦编译器抓不到的"数量/重复"。9 个创世测试全绿,rustfmt 干净。
2. **机构账户派生 in 节点守卫:是**。[cid_lifecycle.rs:522](../../../citizenchain/node/src/core/node_guard/cid_lifecycle.rs) `validate_account_record` 以 `kind.derive(SS58_FORMAT) != record.address` 强制;`validate_required_accounts` 遍历每机构必备协议账户逐个校验派生+反向索引。共识级执法(创世导入+区块处理),非仅创世写入。
3. **创世机构信息核验(派生一致性,源无关)**:重算 `blake2_256(GMB‖op_tag‖ss58_le‖cid_number)`,自检 NRC 主账户吻合;**639 个协议账户(main/fee/stake/安全/两和)派生全部一致**;**297 个创世机构 cid 全唯一、格式全合规**;7 治理机构锚点确认(NRC/PRC×43/PRB×43/NJD/FRG/FSC/基金会);其他创世机构(监察院47/教育委员会1/立法院46/政府军政部门71/省级)覆盖。逐文件精确条目:cb44·ch43·sf44·zf71·jc47·jy1·lf46·基金会1 = 297。

## 2026-07-24 全量创世机构核验(用户追问「所有创世输出机构」)

上轮只报了常量 297(实为 296 公权常量 + 1 私权基金会),**漏了派生部分**。创世 seeder [build()](../../../citizenchain/runtime/genesis/src/institution/seeder.rs) 实际输出:

- **296 常量公权机构**:CB44 + CH43 + ZF71 + JC47 + SF44 + LF46 + JY1(逐节点直铸+双账户;CB/CH/NJD/FRG 带创世管理员,ZF/JC/SF/LF/JY 创世不带管理员)。
- **49,297 派生公权机构**:`build_template_institutions` → `official_derive::for_each_public_institution`,= 11 省部门模板×43 省 + 17 市模板×2872 市。号由 `seed+generator` 确定性生成、名由模板组装。
- **1 私权机构**:公民链基金会(→ private-manage)。
- **公权合计 49,593 + 私权 1 = 49,594**。

核验结果(真实 Rust 测试 + 构造性证明):

- `cargo test -p primitives` **74 passed**;含 `derived_count_matches_area_and_templates`(49,297)、`every_derived_number_is_valid_public_and_unique`(全派生 cid 合法公权码+唯一+名非空)、`area_counts_match_china_sqlite_snapshot`(省43/市2872 与 china.sqlite 一致)。
- **派生 vs 常量跨集合去重**:模板机构码(28 活跃+14 未用镇模板)与 45 种常量机构码**交集为空集**;码嵌在 cid seg2,码族不相交 → 不可能跨集合碰撞。
- 节点守卫 `validate_account_record` 对**每个**机构账户(常量+派生)强制 `kind.derive(SS58_FORMAT)==record.address`,共识级执法。

**潜在缺口 → 已补断言**:新增 `official_derive::tests::genesis_institution_numbers_are_globally_unique`,枚举全部 49,594 创世机构号(296 常量 + 1 基金会 + 49,297 派生)断言全局唯一。跑通(primitives official_derive 3 测试全绿),rustfmt 干净。跨集合去重从"隐式码族不相交推理"升级为显式回归测试守卫。

## 2026-07-24 FCSF 支出方案(已确认决策,待 runtime 二次确认后实现)

目标:让创世给 FSC 的 100 亿联邦公民安全基金(FCSF)可用——程伟创世任职 FSC 两岗 → FSC 内部投票(严格过半)→ 复用现有 multisig 转账从 FCSF 转出。不新建 pallet/extrinsic/call index/扫码动作。

已确认决策:①FSC 创世治理常量放 `china_zf.rs`、复用程伟账户 `9c3e…1068`;②接受 2 席严格过半(=全票=程伟单控,创世引导态);③不纳入节点守卫冻结;④复用主账户转账 action_code(`ACTION_MULTISIG_TRANSFER`)。

确切改动(均在 citizenchain/runtime,逐文件待二次确认):
1. `primitives/cid/china/china_zf.rs`:新增 FSC 创世治理常量(程伟账户/CID/姓名、LR+局长两条任职、阈值 2)。
2. `entity/entity-primitives/src/business_action.rs`:`fixed_role_permission_specs` 早段加 FSC 分支——FSC 的 LR+局长 授 `(MODULE_MULTISIG, ACTION_MULTISIG_TRANSFER)` 的 Propose+Vote(仅限 FSC,不外溢);创世现有 `store_vacant_genesis_role→store_role_permissions_from_fixed_directory` 自动写入。
3. `genesis/src/institution/seeder.rs`:新增 `insert_fsc_genesis_governance`(设 legal_representative=程伟、填 LR+局长任职、写程伟为公权管理员、写 FSC 阈值=2),build() FSC 分支改为调用它。
4. `src/configs.rs`:`can_spend` #8 由 FCSF 全拒改为仅放行 `MultisigTransferExecute`。
5. 测试:创世 FSC 治理落地、权限不外溢(非 FSC 的 LR 无多签转账权)、守卫仅放行转账、端到端 FCSF 转账。
6. 文档:whitepaper FCSF 支出机制 + 节点内置白皮书重生成 + ADR;清 SAFETY_FUND_GOVERNANCE.md 陈旧地址。

兼容性:复用 `propose_multisig_transfer`,call index/签名/扫码不变,不触发双端联动锁;属创世态变更,链开发期直接重生,无 migration。

## 2026-07-24 FCSF 支出实现(已完成并验收)

已按确认方案实现,5 处改动(实现中发现能力策略需同步,故实为 5 点):
1. `primitives/cid/china/china_zf.rs`:新增 FSC 创世治理常量(程伟账户/CID/姓名、LR+局长两条任职 `FSC_GENESIS_ASSIGNMENTS`、`FSC_GOVERNANCE_THRESHOLD=2`)。
2. `entity/entity-primitives/src/business_action.rs`:①`fixed_role_permission_specs` 早段加 FSC 分支——FSC 的 LR+局长授 `(multisig, ACTION_MULTISIG_TRANSFER)` Propose+Vote(仅 FSC);②`fixed_institution_capability_allows` 加 FSC 角色集 [LR,局长](顶层能力放行,否则创世写权限被 `InstitutionCapabilityDenied` 挡)。
3. `genesis/src/institution/seeder.rs`:新增 `insert_fsc_genesis_governance`——FSC 岗位由 `insert_public_institution` 的 `store_vacant_genesis_role` 建为空缺(并随建岗写多签权限),本函数直接填两岗任职为程伟(仿任免路径写 `InstitutionRoleAssignments`)、设 legal_representative=程伟、写程伟为公权管理员、写阈值 2;build() 在 FRG 后调用。
4. `src/configs.rs`:`can_spend` #8 FCSF 由全拒改为仅放行 `MultisigTransferExecute`。
5. 测试:`business_action` 新增 `fsc_lr_and_director_get_multisig_transfer_others_do_not`(权限+不外溢);`runtime/src/tests/cases.rs` 新增 `fsc_genesis_governance_enables_fcsf_multisig_transfer`(任职/阈值/法定代表人/守卫/授权 5 断言,真跑全量创世 seeding)。

验收:runtime 全量 **51 passed / 0 failed**(含上述集成测试);FSC 权限单测通过;编译 0 error/0 warning。

文档:whitepaper 补 FCSF 支出机制(中英)+ 节点内置白皮书重生成;订正 SAFETY_FUND_GOVERNANCE.md 陈旧安全基金地址(045bdb35→4ac77985)。

关键实现要点(免踩坑):FSC 非固定治理机构,故 ①岗位权限只能经 `store_vacant_genesis_role`(建岗即写)写入,不能用 `store_genesis_fixed_role_permissions`(它对非固定机构报 `FixedRoleDefinitionImmutable`);②顶层能力策略 `fixed_institution_capability_allows` 必须同步认 FSC,否则写权限被拒;③任职用直接写 `InstitutionRoleAssignments` 填空缺,不能用 `store_genesis_roles_and_assignments`(它拒已存在岗位)。

## 2026-07-24 追加:白皮书 3.1 创世发行口径统一(用户指出 244 行过期+按岗位写)

上一轮把六账户分账**单独加成 263 行**,没并进 244 行正文,导致 3.1 正文(244)仍是旧的「19管理员各1000万+余额入主账户」缺六账户、且按岗位叙述;汇总表中文(142)/英文(157)也不一致(英文更漏了六账户)。本轮按用户「按账户列、不提岗位」定调统一:
- **244 行重写**为一处完整、按账户、无岗位职责叙述的分配:六个指定账户共700.00亿元(逐一列名) + 国家储委会19个创世管理员账户各1000.00万元 + 余额74,159,737,800.00元写入国家储委会主账户;总量不变。账目闭合 700亿+1.9亿+741.597378亿=1443.497378亿=144,349,737,800元。
- **删 263 行**(六账户明细已折叠进 244,避免重复)。
- **262 行**「主账户余额和19名管理员预置余额」改「上述各账户的创世预置余额」。
- **汇总表 142/157** 同步(英文补齐六账户)。
- 未动:261(每人100元建国寓意)——属发行寓意,非分配口径。
- 追加(用户第二次要求):**删除 FCSF「两岗内部投票支出治理」整段**(原264/现263)——用户要求发行章节只留账户分配、去岗位支出叙述;该基金 100 亿分配已在 244 行写明,故整段冗余删除。发行章节(3.x)已无同类岗位支出叙述;其余「岗位」命中均在治理/投票/机构模组等章节(4–6),本就讲机制,不动。
- 重生 `local-docs.generated.ts` + 重新构建官网 dist;链上 `genesis.rs` 未动(仅文字口径)。

## 状态:分账+公钥+全量机构核验+去重断言+FCSF 支出 全部完成并验收

## 2026-07-26 CitizenApp 机构账户修复

### 只读诊断结论

- 正式链 finalized `#6` 的 FCSF 正确账户
  `0xc0e4ce3c11401ad661ae139081bbc797db51d0efe71df3ffb107f3dcb0064802`
  实际余额为 `1_000_000_000_000` 分（100 亿元），资金没有丢失。
- CitizenApp 已声明 `kOpFcsf = 0x08`，但账户名路由漏掉“联邦公民安全基金”，并把链快照
  `custom_account_names` 中的该协议账户按普通 `OP_NAME` 派生为
  `0x1f5f77852f56e6d97b7f300d0ee883909e7ebbcd5b68d2a59c38a5520fcd1204`；
  该错误账户未激活，所以 App 显示没有余额。
- runtime、OnChina、节点读取、正式创世余额均正确；本次不修改 runtime、不重新创世、
  不补发、不转账、不修改订阅逻辑。

### 预计修改目录

- `citizenapp/lib/citizen/shared/`：补齐 FCSF 保留账户名和 `OP_FCSF` 名称派生路由；
  涉及 Dart 代码、中文注释和旧“6 个保留名”残留清理。
- `citizenapp/lib/citizen/institution/`：机构附加账户统一经名称路由构造，协议账户不再按
  普通自定义账户过滤或派生；涉及 Dart 代码和中文注释。
- `citizenapp/test/governance/shared/`：增加 FCSF 链端固定 AccountId 派生回归测试；
  仅涉及测试代码。
- `citizenapp/test/citizen/institution/`：增加联邦安全局三账户、基金 AccountId 和
  100 亿元余额页面验收；仅涉及测试代码。
- `memory/05-modules/citizenapp/governance/`、`memory/01-architecture/citizenapp/`：
  更新机构账户名称路由、链快照和按需余额读取边界；仅涉及文档和旧口径清理。
- `memory/08-tasks/`：复用并恢复本任务卡，记录修复与真实验收结果；仅涉及文档。

### 验收边界

- Flutter 派生测试必须得到链上正确 FCSF AccountId，且该保留名禁止作为普通自定义账户注册。
- 联邦安全局账户列表必须固定为主账户、费用账户、联邦公民安全基金三个账户。
- 使用本地正式链 RPC 读取正确账户的 finalized 余额，并在真实 Flutter 页面显示
  `10,000,000,000.00 元`。
- 不执行真实购买、订阅、转账或任何链上写操作。

### 修复结果与真实验收

- `reserved_account_names.dart` 已补齐第七个保留名“联邦公民安全基金”，并纳入禁止普通
  自定义注册规则。
- `account_derivation.dart` 已把该名称路由到 `kOpFcsf(0x08)`；普通非空名称仍唯一回落
  `OP_NAME`，没有兼容旧错误账户。
- `institution_accounts.dart` 已将快照附加账户统一交给名称路由，联邦安全局账户列表稳定为
  主账户、费用账户、联邦公民安全基金三行；固定治理账户显示名同时统一为链上保留名。
- 定向 Flutter 测试：`18 passed / 0 failed`，包含 FCSF 固定 AccountId、禁止自定义注册、
  联邦安全局三账户和页面显示 `10,000,000,000.00 元`。
- CitizenApp 全量测试：`809 passed / 5 skipped / 0 failed`；5 项为仓库原有条件跳过。
- CitizenApp 全量 `flutter analyze`：`No issues found`。
- 正式链只读验收：finalized `#6`
  (`0xf1375204579d3e73f407388a639f71f388a7c2b07bda93f241cac82c79d4763b`) 的正确账户
  `0xc0e4ce3c11401ad661ae139081bbc797db51d0efe71df3ffb107f3dcb0064802`
  余额为 `1_000_000_000_000` 分（100 亿元）；CitizenApp 中枢省目录快照同步确认
  `account_count=3` 和唯一附加名称“联邦公民安全基金”。
- 未修改 runtime、OnChina、节点、创世数据或订阅逻辑；未执行转账、购买、订阅或其它链上写操作。
