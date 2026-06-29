# 任务卡:onchina card 09(admin 泛化)+ 10(seed 泛化)合并

- 状态:方案锁定待执行(2026-06-29)
- 承接:[20260628-onchina-onchain-write-and-followups](20260628-onchina-onchain-write-and-followups.md) 的 item 2/3(= console-refactor 09/10)。
- 完整分析:本会话 workflow wf_6bbe87e5(6 面)。**12 立法 web 提案另起窗口线程**;**R4 在 09/10 完成后做整任务收尾清理并结束任务**。

## 锁定决策(2026-06-29)
1. **只做去硬编码**:P1-P6 全做但能力保守——加中性 `can_view_own_admins`(只读)+ 机构类分发 + Tier 谓词驱动;**FRG/CREG 行为零变**;非注册局机构仅开只读位,CRUD/具体动作留各机构功能落地再开(机制就绪、不越权)。
2. **AdminActionType 改 Tier 中性名**(零残留):`CreateCityRegistry→CreateSubordinateRegistry` 等 + 前端 action code 同步(onchina 内部协议,破坏式改)。

## 现状根因(已核实)
- 身份多值化已完成(`AdminAuthContext{institution_code, admin_level∈Nat/Prov/City/Town, scope_*}`),**无 Tier**;FRG/CREG 双角色靠字面 `institution_code=="FRG"/"CREG"` 散落(后端 ~15 文件 + 前端 4 视图,共 ~132 处)。
- 根因:`primitives::cid::code::admin_level` 把 FRG 归 `National`(链端铁律不可改),但 FRG 实为省分区 → 全靠 `=="FRG"` 矫正。
- 硬前置:`capability.rs::capabilities_for` 仅 FRG/CREG,其余 EMPTY → 非注册局无 admin tab;前端 `capabilityMap.ts` 3 个 admin 能力位**声明却零消费**。
- FRG 扁平账户无省维度:215 人省归属是链下元数据,唯一来源 `federal_registry_scope` 表(seed 从 china_zf 灌,`FEDERAL_ADMIN_PROVINCES[43]×5` + `215==43×5` 断言)。

## 设计原则
- 不新增 Tier 字段:用「机构类 + admin_level」表达 Tier1(FRG=Nat 注册局)/Tier2(CREG=City);Tier 收敛单点谓词 `registry_tier_for(code)`/`is_tier1`(内部 `FRG_CODE`)。
- 能力位驱动鉴权(单源 capability.rs);**后端唯一鉴权**,补能力位必同批改 guard(否则"能看不能做")。
- FRG 省维度铁律:Tier1 省作用域只取 `tier1_admin_scope`(原 federal_registry_scope)映射,**绝不用节点 env 兜底**。
- **不碰链端**:china_zf.rs / cid code.rs / genesis 冻结常量不动;215 人分组保持 onchina seed.rs 元数据(并进 china_zf 会破坏链端 genesis_build,禁)。

## 分步(09/10 共用 Tier 谓词,合并;每步 cargo check -p onchina + 前端 tsc)
- **P1 能力前置**:`capability.rs` 加中性 `can_view_own_admins`/`can_crud_own_admins`;`capabilities_for` 改按 `primitives::cid::code` 机构类分发(fixed_governance 拒/FRG→tier1/education/public(CREG 为 City 子集 tier2 特例)/private/unincorporated);前端 `capabilityMap.ts` 同步形状 + bump 缓存版本。**本决策:非注册局只开 view_own_admins 只读位**。
- **P2 Tier 谓词 + guard**:`chain_runtime.rs` 加 `registry_tier_for/is_tier1`;`operation_auth.rs:155` `!="FRG"`→能力位;`requires_federal_admin`→`requires_governing_capability`;`AdminActionType` 5 个 FRG/CREG 动作名→Tier 中性名(+as_str/parse/auth_type/is_governance 同步)。
- **P3 repo/表泛化**:repo.rs `'FRG'`/`'CREG'` SQL(:39/49/97/107/121/131/159/175/299/820)参数化 `=$N`;`list_federal_registry_*`→`list_tier1_*`、`list_city_registry_*`→`list_subordinate_*`;catalog.rs/city_registry_admins.rs/actions.rs 停传死值改 ctx/目标码;`federal_registry_scope`→`tier1_admin_scope`(db.rs 建表 + 4 处 JOIN)。
- **P4 seed(card 10)**:`run_seed_federal_admins`→`run_seed_tier1_admins`(CLI/枚举/main.rs);入口遍历 `console_admin_pallets==[GenesisAdmins]` 且 china_zf 带创世管理员常量(当前仅 FRG,行为不变去字面);`FEDERAL_ADMIN_PROVINCES[43]×5`+`215` 断言**留 onchina seed.rs**;`is_frg`→`is_tier1`;非 Tier1 只链读不播种。
- **P5 scope/gate**:`scope/rules.rs:152` FRG 特判、`onchain_gate.rs:92` `is_federal_registry`、`repo.rs:264` `is_frg`、`guards.rs` CREG idle/national_no_province → `is_tier1`/能力位/`province_scope_source` 谓词;None 档(私权/非法人)收紧为本 `institution_id` 范围。
- **P6 前端**:4 视图 `==='FRG'/'CREG'`→接 `capabilities.canCrud*` + `useScope().lockedCityName`;目录筛选 `'CREG'`→`targetInstitutionCode` prop;`useScope.ts:195` 删 FRG 特判信任后端有效层级;新增中性「机构管理员」tab 绑 `can_view_own_admins`;App.tsx 默认 tab 能力位派生。

## 坑(实现守住)
- FRG 省维度:`tier1_admin_scope` + china_zf bootstrap 唯一来源;Tier1 走映射、非 Tier1 走 node env,不混。
- 后端/前端能力位同源(P1 必同批 P2)。三个同名"Tier"陷阱(admin_level/AdminOperationAuth/本卡 Tier)勿混。
- china_zf 不动;`215==43×5` 契约留 onchina 侧。

## 验收
- `cargo test -p onchina` + `cargo check -p node` + 前端 `tsc` 绿;FRG/CREG 行为零回归;**零 `=="FRG"`/`"CREG"` 字面**(谓词单点除外);非注册局机构登录有只读 admin tab(can_view_own_admins);seed 仅 Tier1 播种。
