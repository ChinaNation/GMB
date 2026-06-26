# 20260625 管理员字段扩展 + 护宪大法官生产成员解析(E1+E2 合并卡)

承接 `20260625-legislation-signing-5type-revision.md` 的 E 收尾。本卡把原 E1(管理员字段扩展)+ E2(护宪生产成员解析)合并实现;E3(重新创世 + 真机 QA)不在本卡(用户指定跳过,随整套上链统一处理);E4(残留文档清理)已在母卡随手做完。

## 背景 / 目标

护宪大法官终审(F)链端状态机已落地,唯一缺口是生产环境取不到 7 名护宪大法官:`InternalAdminProvider::constitution_guard_members()` 现为 **trait 默认返回空**(`votingengine/src/traits.rs:628`),仅测试 mock 注 7 人(`legislation-vote/src/tests/mod.rs:181`)。生产解析方案=「按职务过滤国家司法院(NJD)admins 取 7 人」,前置 = admins 记录需带「职务」字段。故 E1(字段扩展)解锁 E2(成员解析),合并一卡。

## 已确认范围

- **E1 管理员字段扩展**:admins 记录从「账户/SS58 单值」扩为结构体 `{ 姓名, 账户, CID号, 职务 }`。
- **E2 护宪生产成员解析**:用 E1 的「职务」字段实现 `constitution_guard_members()` 生产体——读 admins-change 中国家司法院(NJD)机构的 admins,过滤职务=护宪大法官,取 7 人;替换 trait 默认空。
- 同一职务字段后续可复用于权责划分(如 NJD 全部 admins 中划出 7 名护宪大法官)与 B3 法定代表人(职务=机构首脑)对齐。

## E1 设计(链端,runtime 二次确认)

1. **AdminRecord 结构**(单一真源,放 `primitives` 或 admins-change types):`name: BoundedVec<u8>`、`account: AccountId`、`cid_number: BoundedVec<u8>`、`role: AdminRole`。
2. **AdminRole 职务取值**:枚举(单一源),至少含「机构首脑 / 护宪大法官 / 普通委员(或议员)」。具体取值集合 = **待确认**(见开放问题)。
3. **admins-change 存储改造**:现 admins 列表(`Vec<AccountId>` 形态)整体切换为 `Vec<AdminRecord>`(禁止兼容,全切)。波及该 pallet 的 storage、add/remove/set extrinsic 入参、事件、查询。
4. **创世写入**:`primitives/china/china_*.rs` 各机构 admins 需补 `name/cid_number/role` 字段。`role` 为功能必需(NJD 7 名护宪大法官 + 各机构首脑)。`name/cid_number` 数据来源见开放问题。
5. **B3 法定代表人对齐**:`LegalRepresentatives`(已存在)与「职务=机构首脑」语义重叠——确认是「职务派生 legal_rep」还是「两者并存」(避免双源,参照单一真源铁律)。
6. **客户端读取**:CitizenApp/CitizenWallet 读 admins 的解码结构需同步(本卡只负责链端 DTO 定稿 + 各端解码对齐清单,UI 在 D 卡)。改 DTO 字段须 bump 客户端缓存版本(参照母卡踩坑铁律 `feedback-dto-field-rename-bump-cache-version`)。

## E2 设计(链端)

1. `constitution_guard_members()` 生产实现放 `runtime/src/configs/mod.rs` 的 `RuntimeInternalAdminProvider`(现委托 admins-change 处)。
2. 逻辑:取 NJD 机构 admins → `filter(role == 护宪大法官)` → 取 7 人(数量校验:不足/超出的处理策略 = **待确认**)。
3. NJD 机构码常量走单一源(china 常量库);护宪大法官职务取值走 E1 的 AdminRole 单一源。
4. 测试:mock 注职务字段,验证生产解析正好取出 7 名护宪大法官;非护宪职务 admins 不入选;数量异常按确认策略处理。

## 硬规则约束

runtime 二次确认 / 禁止兼容(admins 结构全切,无过渡) / 单一真源(AdminRole + NJD 码) / 字段改动 bump 客户端缓存版本 / 真实运行态验收(随 E3 上链后,不在本卡)。

## 验收

- `cargo check -p admins-change -p votingengine -p legislation-vote -p citizenchain`(std + no_std)全绿。
- admins-change 单测覆盖新结构 add/remove/set;`constitution_guard_members()` 生产体单测(取 7 / 过滤 / 数量异常)。
- legislation-vote 护宪用例由 mock 改读生产路径仍全过(或保留 mock + 新增生产体单测,二选一明确)。
- `cargo fmt` 干净;残留扫描旧 admins 形态零残留。
- 真机:留 E3(重新创世后)统一验,不在本卡。

## 开放问题(需用户拍板)

1. **AdminRole 取值集合**:仅「机构首脑 / 护宪大法官 / 委员」三值,还是按机构类型细分更多职务?
2. **姓名 / CID号 数据来源**:创世 ~数百 admins 是否都有真实姓名/CID?若无,初期是否允许 `name` 占位、`cid_number` 可空?(职务为功能必需,姓名/CID 可能阶段性留空)
3. **护宪人数策略**:NJD 中护宪大法官恰好 7 人是硬约束还是「取前 7 / 全取」?不足 7 人时 `constitution_guard_members()` 返回什么(空=护宪流程走默认否决?)。
4. **legal_representative 与 role=机构首脑**:并存还是合一(单一真源)?

## 进度

- [ ] E1 AdminRecord 结构 + AdminRole 单一源
- [ ] E1 admins-change 存储/extrinsic/事件切换
- [ ] E1 创世 china_*.rs 字段补齐
- [ ] E1 B3 法定代表人对齐
- [ ] E2 constitution_guard_members 生产体 + 测试
- [ ] 编译 + 单测 + fmt + 残留扫描
