# 任务卡:onchina card 09(admin 泛化)+ 10(seed 泛化)合并 — rev2

- 状态:**完成(done,2026-06-29)**。P0-P6 实现 + R4 对抗审计(29 agent)确认无回归。
  - 验证:`cargo check -p onchina/node` 绿、`cargo test -p onchina` 72 绿、`cargo clippy -p onchina` 无 dead-code、frontend `tsc` 绿;零 `=="FRG"/"CREG"` 字面(后端 chain_runtime 谓词单点 + 前端 registryTier.ts 单点)。
  - 遗留(out of scope,已开 spawn_task):onchina `AdminOperationAuth` 后端 `PASSKEY_COLD_SIGN` vs 前端冷签流 `=== 'SCAN_SIGN'` 不匹配(pre-existing,3档 vs 2档协议未对齐)。
  - citizenapp follow-up:`kGenesisInstitutions` 的 FRG 单 mainAccount 条目读空,需按 province_code 读省组。
  - 落地要点:P0 FRG 省组链读修登录;P1 capability 加 `can_view_own_admins` + 机构类分发;P2 谓词 `is_tier1_registry/is_subordinate_registry` + AdminActionType→Tier 中性名(CreateSubordinateRegistry/UpdateSubordinateRegistry/DeleteSubordinateRegistry/UpdateGoverningRegistry/ReplaceGoverningRegistry,wire 同步)+ `requires_governing_capability`;P3 repo 去 federal_registry_scope JOIN/参数化/省取节点;P4 删 seed.rs + run_seed_federal_admins + FEDERAL_ADMIN_PROVINCES + SeedFederalAdmins CLI + gov/service::federal_registry_admins + db.rs 去 federal_registry_scope/provinces 表;P5 scope/gate/guards/signature 谓词化;P6 前端 registryTier.ts 单点谓词 + capabilityMap 加 canViewOwnAdmins + 4 视图去字面 + wire 同步;FRG 列表+换届 current-set 全走链读 `fetch_federal_registry_province_admins`。
- 承接:[20260628-onchina-onchain-write-and-followups](20260628-onchina-onchain-write-and-followups.md) item 2/3。**12 立法 web 提案另起窗口线程;R4 在 09/10 完成后做整任务收尾并结束。**
- 重检来源:本会话 workflow `wc41m2g3b`(链端 genesis_build / onchina / 链端 AdminAccountQuery / citizenapp 四面重映射)。

## 锁定决策
1. **只做去硬编码**:P1-P6 全做,能力保守——加中性 `can_view_own_admins`(只读)+ 机构类分发 + Tier 谓词;FRG/CREG 行为零变(除 P0 修复);非注册局机构仅开只读位。
2. **AdminActionType 改 Tier 中性名**(零残留,onchina 内部协议破坏式改 + 前端 action code 同步)。
3. **退役本地 seed/表,FRG 管理员 + 省映射全走链读**(2026-06-29 新决策):删 onchina `run_seed_federal_admins` 的 215 平铺播种 + `FEDERAL_ADMIN_PROVINCES[43]` 中文省名数组;`federal_registry_scope` 不再作授权/省映射真源;FRG 省映射从链上 `FederalRegistryProvinceGroupAccounts` 派生。

## 创世机构新模型(已核实,链上真相)
- **5 创世码** `is_genesis_admin_code == is_fixed_governance_code` = NRC/PRC/PRB/FRG/**NJD**(全 `kind=GenesisInstitution`,`source=Genesis`)。阈值 NRC13/PRC6/PRB6/FRG3/NJD8。
- **NRC/PRC/PRB/NJD** → `GenesisAdmins::AdminAccounts`(键=机构主账户)。NJD 新增(china_sf `NATIONAL_JUDICIAL_YUAN_ADMINS` 13 人,带 `admin_role` 护宪/首席/次席/大法官)。
- **FRG** → **不进 AdminAccounts**。215=43省×5,写 `FederalRegistryProvinceGroups[province_code]` + 反向 `FederalRegistryProvinceGroupAccounts[group_account]`(group_account=`blake2_256("GMB:FRG-PROVINCE:"+province_code)`)。换人走 `propose_federal_registry_province_admin_set_change`(call_index=2,5人/阈值3);普通 `propose_admin_set_change` 对 FRG 硬拒。
- onchina 控制台**只管 FRG**(+ 其供给的 CREG/公私权);NRC/PRC/PRB/NJD 自治不进 onchina(`console_admin_pallets` 保持拒)。

## 🔴 P0(新发现,先修):FRG 登录在新链上已断
- `fetch_active_admins_onchain` 对所有 pallet 读 `AdminAccounts[main_account]`;FRG 链上已无此键 → 恒 None → `onchain_gate` `NotOnchainAdmin` → FRG 管理员全部登不进。onchina 全仓零 `FederalRegistryProvinceGroups` 读码。
- **修复**:① 新增镜像读 `GenesisAdmins::FederalRegistryProvinceGroups[province_code]`(value 同 `OnChainAdminAccount`);② `node_institution_identity` 为 FRG 解析 province_code(由 `CID_RUNTIME_SCOPE_PROVINCE_NAME` → `primitives::cid::code::PROVINCE_CODE_INFOS` 映射);③ `fetch_active_admins_onchain` 对 FRG 分流到省组读,非 FRG 仍读 AdminAccounts。
- **关联**:citizenapp `kGenesisInstitutions` 的 FRG 单 mainAccount 条目同样读空 → 列 citizenapp 侧 follow-up(不在本卡)。

## 现状根因(已核实)
- 身份多值化已完成(`AdminAuthContext{institution_code, admin_level∈Nat/Prov/City/Town, scope_*}`),无 Tier;FRG/CREG 双角色靠字面 `=="FRG"/"CREG"` 散落(后端 ~15 文件 + 前端 4 视图)。
- `primitives::cid::code::admin_level` 把 FRG 归 `National`(链端铁律不可改),FRG 实为省分区 → 全靠 `=="FRG"` 矫正。
- `capability.rs::capabilities_for` 仅 FRG/CREG,其余 EMPTY → 非注册局无 admin tab;前端 `capabilityMap.ts` 3 个 admin 能力位声明却零消费。

## 设计原则
- 不新增 Tier 字段:用「机构类 + admin_level」表达 Tier1(FRG=创世注册局,省分区)/Tier2(CREG=City,FRG 供给的公权)。Tier 收敛单点谓词 `registry_tier_for(code)`/`is_tier1`(内部镜像 `FRG_CODE`)。
- 能力位驱动鉴权(单源 capability.rs);补能力位必同批改 guard。
- **链上是管理员唯一真源**;本地表仅链派生缓存,fail-closed(`onchain_gate` 已是)。
- FRG 省维度从链上 `FederalRegistryProvinceGroupAccounts` 派生,**绝不用 env 兜底授权**。
- **不碰链端**:china_zf/china_sf/genesis-admins/cid code.rs 冻结不动;onchina + citizenapp(本卡仅 onchina,citizenapp follow-up)。

## 分步(每步 `cargo check -p onchina` + 前端 `tsc`)
- **P0 FRG 省组链读(修登录,基座)**:`chain_runtime.rs` 加 `FederalRegistryProvinceGroups` 镜像读 + province_code 解析(PROVINCE_CODE_INFOS);`fetch_active_admins_onchain` 对 FRG 分流;`node_institution_identity`/`NodeInstitutionIdentity` 带 province_code;`onchain_gate.rs` 登录闸消费。
- **P1 能力前置**:`capability.rs` 加中性 `can_view_own_admins`(只读)+ `capabilities_for` 按 `primitives::cid::code` 机构类分发(fixed_governance:仅 FRG→tier1 放行,余拒/education/public(CREG=City 子集 tier2)/private/unincorporated);前端 `capabilityMap.ts` 同步形状 + bump 缓存版本。非注册局只开 view 位。
- **P2 Tier 谓词 + guard**:`chain_runtime.rs` 加 `registry_tier_for/is_tier1`;`operation_auth.rs` `!="FRG"`→能力位;`requires_federal_admin`→`requires_governing_capability`;`AdminActionType` FRG/CREG 动作名→Tier 中性名(+as_str/parse/auth_type/is_governance 同步)。
- **P3 repo/表泛化**:repo.rs `'FRG'`/`'CREG'` SQL 参数化;`list_federal_registry_*`→`list_tier1_*`、`list_city_registry_*`→`list_subordinate_*`;catalog/city_registry_admins/actions 停传死值;**FRG 省映射改链上派生**(删 `federal_registry_scope` 授权用途;若保留为缓存则由链回填)。
- **P4 seed 退役(card 10)**:删 `run_seed_federal_admins` 215 平铺播种 + `FEDERAL_ADMIN_PROVINCES[43]` 中文省名数组 + `215==43×5` 断言(CLI/枚举/main.rs `SeedFederalAdmins` 入口一并删);FRG 管理员/省映射全走 P0 链读;`gov/service.rs` 关联清理。
- **P5 scope/gate**:`scope/rules.rs` FRG 特判、`onchain_gate.rs` `is_federal_registry`、`repo.rs` `is_frg`、`guards.rs` CREG idle/national_no_province → `is_tier1`/能力位/链上省组;None 档收紧本机构范围。
- **P6 前端**:4 视图 `==='FRG'/'CREG'`→`capabilities.canCrud*`;目录筛选 `'CREG'`→`targetInstitutionCode` prop;`useScope.ts` 删 FRG 特判;新增中性「机构管理员」tab 绑 `can_view_own_admins`;App.tsx 默认 tab 能力位派生。

## 坑
- FRG 省维度真源=链上 `FederalRegistryProvinceGroups`(by province_code);node env 仅定位本节点省份,不作授权。
- 后端/前端能力位同源(P1 同批 P2)。三个同名"Tier"陷阱(admin_level / AdminOperationAuth / 本卡 Tier)勿混。
- 链端不动;NJD 已加入创世码全集,任何硬编码 `[FRG,NRC,PRC,PRB]` 兜底需含 NJD(若 onchina 出现)。

## 验收
- `cargo test -p onchina` + `cargo check -p node` + 前端 `tsc` 绿;**FRG 节点能登录**(P0,按省组读 Active 集合);非注册局机构有只读 admin tab;零 `=="FRG"`/`"CREG"` 字面(谓词单点除外);onchina 不再本地播种 FRG / 不再以本地表作 FRG 省映射真源。
