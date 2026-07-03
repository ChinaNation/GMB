# 存量公权机构:全量创世直铸(占号体系 卡3)

> 设计真源:`memory/04-decisions/ADR-031-cid-occupy-registry.md`(D5/D9/D4/D0)。2026-07-03 用户修订:**市/镇级同样创世直铸,与 NRC/PRC/PRB/FRG/NJD 同一模式,全程零交易**——此前"直铸不可行"只对 raw chainspec 形态成立,数据改常量+派生、部署改 plain spec 后直铸更简;批量上链驱动器与 `register_public_institutions_batch` 从存量路径整体删除(降级为未来增量场景备选,本轮不实现)。

## 背景与已完成前置

- 92 码表定稿(镇无立法/教委、省无教委/公安厅,2026-07-03 用户拍板);卡1 链端校验已完成归档。
- 嵌入式库旧机构清理已执行(2026-07-02):删旧公权 245,629+账户+gov 目录;87 储备机构对齐常量库;旧码零残留。
- 行政区真源 china.sqlite:43 省 / 2,872 市 / 39,087 镇。

## 处理决策

1. **国/省级 282:** 扩展 `runtime/genesis/src/institution.rs` 遍历全部 7 个 `china_*` 数组(现只铸 CB/CH/NJD/FRG 共 89)写入 Institutions+双账户+ProtectedGenesisAccounts;NJD/FRG 管理员特例保留。
2. **模板派生机构 596,517(2026-07-03 命名规则核验后定):数据不手写、不进 chainspec。** 命名与机构集真源 = onchina `gov/service.rs` 确定性模板(gov-deterministic-v8,已由用户统一命名),**搬进 primitives 作为链上/链下单源**(onchina 改引用):
   - 组装规则(全 282 常量+模板逆向验证零例外):`简称 = 行政区显示名 + suffix`、`全称 = 行政区显示名 + full_suffix`;国家级 = label/手写;
   - 派生范围:国家参众议会 NSN/NRP 2 + 省级部门 11 类×43 省=473 + 市级 17 类×2,872=48,824 + 镇级 14 类×39,087=547,218;
   - 号=生成器现场派生、主/费账户=派生原语现场派生,全确定性;与 282 共用 `insert_public_institution`;创世机构不带管理员(后续走联邦特权直设通道)。
3. **构建期断言**:逐号 `parse_cid_number_parts`+`is_public_legal_code`,坏号创世构建即 panic。
4. **部署形态改造(必做)**:起始 state ≈0.6-0.8GB,raw chainspec(~1.5GB)不可行——节点从「raw include_bytes! 全量 state」改「plain spec + 首启 GenesisBuilder 本地构建创世」(冻结语义=冻结 runtime+常量,创世哈希唯一;首启一次性 ~420 万存储项,分钟级);CitizenApp/smoldot 侧 chainspec 用 `stateRootHash` 轻形态。
5. **onchina 只读对账(D9)**:不再生成机构、不再上链;各市节点首启从链上对账拉全国机构册写本地 subjects(`cid_number` 新 schema);旧 `sfid` 库整体删除。

## 目标

- primitives 内嵌行政区常量表(从 china.sqlite 导出生成,含一致性校验脚本,幂等可重生)。
- genesis 全量直铸 282 + 596,517,构建期断言全过;genesis 测试断言数量与「常量数组 + 行政区×码表」推导值逐一一致。
- 节点部署链路改 plain spec + 首启本地构建;smoldot 侧 stateRootHash;重新创世(6 节点 mesh)。
- 创世后重跑 citizenapp 机构注册表生成器(死规则:否则机构全断)。
- onchina 首启链上对账同步器(只读),验收「链上 ↔ 本地库」两方一致。
- 旧 `sfid` 库删除,全仓零 `sfid_number` 残留。

## 修改范围

- `citizenchain/runtime/primitives/`(行政区常量表 + 导出工具)
- `citizenchain/runtime/genesis/src/institution.rs` 与 `genesis/src/tests/`
- `citizenchain/node/`(chainspec 形态、首启构建、打包脚本)
- `citizenapp/`(smoldot chainspec 资产、机构注册表生成器重跑)
- `citizenchain/onchina/src/`(链上机构册对账同步器,只读)
- 本机/各节点旧 `sfid` 库删除

## 验收

- 创世后链上 Institutions 总数 = 596,799(常量 282 + 模板派生 596,517),与推导值一致(genesis 测试断言);内置号 100% 通过 parse 校验。
- 全网各节点创世哈希一致;首启构建耗时记录在案。
- onchina 对账:链上 ↔ 本地库两方一致;CitizenApp 机构册可读。
- 全程零交易零手续费;`cargo test -p citizenchain` 与 genesis 相关测试通过。

## 进展

- 2026-07-03:**代码核心完成并测试通过**(primitives 45 测试全绿、genesis lib + runtime + onchina 编译过、runtime 30 lib 测试过、onchina 134 测试过):
  - **命名/机构集模板单源迁 primitives**:新建 `primitives/cid/official_template.rs`(OfficialOrgTemplate + short_name/full_name 组装 + 省部门 11/市 17/镇 14/国家两院 2 全表);onchina `gov/service.rs` 删本地副本改引用,消除 D0 漂移风险。
  - **行政区表嵌入 primitives**:`gen_area_data.py` 从 china.sqlite 生成紧凑二进制 `area_data.bin`(43 省/2872 市/39087 镇,578KB);新建 `primitives/cid/china/area.rs` no_std 零拷贝解析器(`for_each_area` 单回调 AreaItem 枚举 + `area_counts`)。
  - **全量派生单源**:新建 `primitives/cid/official_derive.rs`——`for_each_public_institution(f)` 用 seed+generator 确定性派生「行政区 × 模板」全部机构(与 onchina official_institution_cid 同源,年份固定 2026,国家两院落中枢省主市);`public_institution_derived_count()`。
  - **genesis 直铸**:`genesis/institution.rs` 新增 `insert_derived_public_institution`(号确定性派生账户、构建期 parse+公权家族断言、不进 ProtectedGenesisAccounts 避免 59 万双倍保护)+ `build_template_institutions` 调 primitives 枚举;`build()` 末尾接入。常量 282 全量直铸维持。
  - **测试**:primitives 断言派生数 =596,517、每号 parse 合法+公权家族+全局唯一(596k 迭代 6s)、名称组装、国家两院国名前缀、区划计数 43/2872/39087。终态链上 = 282 + 596,517 = **596,799**,零交易。
- **剩余 = 部署操作(用户 重新创世 步)**:①chainspec 形态改造(plain spec + 首启 GenesisBuilder 本地构建,起始 state ≈0.8GB,raw 不可行;smoldot 侧 stateRootHash)②重新创世部署(6 节点 mesh)③onchina 链上机构册只读对账同步器 ④citizenapp 机构注册表生成器重跑 ⑤旧 sfid 库删除。genesis-pallet 自身 test mock 为上一轮 runtime 重构预留断链(缺 public_manage/public_admins 等 impl),故派生断言落 primitives(编译/测试均绿);全量 state 物化在真实 chainspec 构建期发生。

## 进展与审计(2026-07-03)

- **链端派生已完成**:primitives 新增 `official_template.rs`(命名模板单源:国 2/省部门 11/市 17/镇 14)、`official_derive.rs`(全量派生枚举,`public_institution_derived_count()==596,517` 断言单测绿)、`china/area.rs + area_data.bin`(行政区常量:43 省/2,872 市/39,087 镇,与 china.sqlite 一致);genesis `insert_derived_public_institution`(构建期 parse+公权家族断言;派生机构 Active、主/费双账户,**不打 ProtectedGenesisAccounts 封存标记**,留给后续治理)。
- **体积实测(按真源逐条、按存储项真实编码)**:596,799 机构创世 state 原始 KV ≈ **542MB**(每机构 ≈0.95KB:Institutions 1 条+主/费账户各 3 条索引);RocksDB 落盘含 trie/压缩 ≈ **0.8-1.4GB**;首启一次性写 ~420 万存储项(分钟级)。
- **覆盖核实**:cities 表无 "000" 保留市(count=0,无多铸);村级无机构码(制度定稿)不铸;全部省/市/镇 × 对应码族全配。
- **账户操作语义核实**:机构账户=派生地址无私钥,唯一操作路径=管理员集合经 internal_vote(multisig-transfer 要求 `InstitutionQuery::is_active` + 发起人 `is_internal_admin`);无管理员=完全不可操作(创世余额亦为 0);创世带管理员的仅 NJD/FRG/CB/CH,其余待联邦特权直设。

## 剩余工作清单

1. **node 部署形态改造(D5 必做)**:chain_spec.rs 仍为 raw include_bytes(59.7 万机构下 raw JSON 会到 GB 级)→ 改 plain spec + 首启 GenesisBuilder 本地构建;CitizenApp/smoldot 侧 stateRootHash 轻形态。
2. **onchina 机构册只读对账同步器(D9)**:未实现(首启从链上拉全国机构册写本地 subjects)。
3. **测试补课**:genesis-pallet 自带测试 mock 既有断链(Config 现要求 public_manage/public_admins 全栈);runtime 级 596,799 总数断言(本卡验收项)未加。
4. 部署操作:重新创世(6 节点)→ 重跑 citizenapp 机构注册表生成器 → 各节点旧 `sfid` 库删除。

## 收尾与部署技术方案(2026-07-03 定稿,下一步执行序)

技术支点(已实码核验):runtime 已实现 `GenesisBuilder` API(runtime/src/apis.rs:319);`fresh_genesis_config()` 已是 plain 构建器形态(ChainSpec::builder(wasm)+with_genesis_config_patch+复用冻结 bootnodes);`institution::build` 挂在 genesis-pallet `BuildGenesisConfig`(创世构建期在 WASM 内执行);citizenapp chainspec 资产在 citizenapp/assets/chainspec.json(smoldot-pow 分支)。

### A. node 部署形态改造(plain spec + 首启本地构建)
- A1 生成冻结 plain spec:`fresh_genesis_config()` → `as_json(raw=false)` 落 `node/chainspecs/citizenchain.plain.json`(内嵌 WASM+genesis patch+bootnodes+properties,MB 级);新增导出入口(CLI 子命令或 clean-run.sh 步骤)。
- A2 `chain_config()` 切换:include_bytes 冻结 plain JSON(替换 raw);raw 文件删除零残留;clean-run.sh / fuwuqi.sh 打包链路适配。
- A3 首启行为:sc-service 经 WASM `GenesisBuilder_build_state` 物化 596,799 机构(596k 号派生 + ~420 万存储项);预计 1-3 分钟、RAM 峰值 2-4GB、落盘 0.8-1.4GB;耗时记录进验收。退化方案:官方预构建创世 DB 快照分发(仍零交易)。
- A4 冻结语义:冻结的是 plain JSON(runtime WASM + patch + bootnodes),创世哈希由其唯一决定,全网一致(派生全确定性)。

### B. citizenapp/smoldot 轻形态
- B1 新脚本从首个已启动节点读 genesis header(块 0 哈希 + stateRoot),产出 `assets/chainspec.json` 轻形态:name/id/bootNodes/properties + `genesis.stateRootHash`(不含 596k state,不含 runtimeGenesis)。
- B2 smoldot-pow 分支对 `genesis.stateRootHash` 解析的兼容性验证(真机,验收项)。
- B3 机构注册表生成器重跑(死规则)。

### C. onchina 机构册对账(D9 修订:同源派生 + 抽样对账)
- C1 首启物化:新模块用 primitives(official_derive + china_* 数组,与创世同一套代码)本地生成 596,799 条,批量写 subjects/gov/accounts(COPY,分钟级);meta 表记录物化版本(模板/区划数据 hash),幂等跳过。
- C2 启动对账:随机抽样 N=32 个号 fetch 链上 `Institutions` 核对名称/状态,不一致 fail-closed 拒绝启动(防 runtime 与 onchina 版本漂移)。
- C3 验收工具:一次性 CLI 全量遍历链上 Institutions ↔ 本地比对(部署验收用)。
- C4 运行期增量:占号先行创建的新机构走卡2 已建路径落库。
- 修订理由:创世数据与 onchina 同源(primitives),按需全量 WSS 拉取(~200MB/节点)是浪费;抽样对账+全量验收工具达到同等保证。

### D. 测试补课
- D1 runtime 级总数断言:runtime tests 调 `genesis_pallet::institution::build::<Runtime>()` 后 `Institutions::iter().count()==596,799` + 抽查(市/镇名称组装、账户派生、NJD/FRG 管理员在位)。
- D2 genesis-pallet 自带 mock 修复:镜像 public-manage tests 全栈 mock(system/balances/votingengine/admins/entity),恢复其相位切换测试。

### E. 部署 runbook(代码完成后,用户执行)
1. 生成并冻结 plain spec → 提交;
2. fuwuqi.sh 出 deb → 6 节点部署;首启构建耗时记录;`chain_getBlockHash(0)` 全网一致核对;
3. citizenapp 轻形态 chainspec 替换 + 注册表生成器重跑 + 真机 smoldot 验证;
4. onchina 各节点首启物化+抽样对账通过;本机与各节点旧 `sfid` 库删除(零残留);
5. 终态验收:链上 Institutions=596,799、创世哈希一致、onchina 全量比对 CLI 通过、CitizenApp 机构册可读、公民建档占号端到端(真链)walkthrough。

### F. 风险
- WASM 创世构建耗时/内存(退化=DB 快照);smoldot stateRootHash 真机未验;bootnodes 复用已由 fresh_genesis_config 处理;创世哈希决定性=同 WASM+同 patch(派生代码确定性已测)。

## 状态

- 2026-07-02:建卡(原批量交易上链方案)。
- 2026-07-03:改为**全量创世直铸**终稿;**代码核心(模板单源+区划嵌入+genesis 全量派生+断言测试)完成并全绿**;剩部署操作(chainspec 形态/重新创世/onchina 对账/注册表重跑)属用户 重新创世 步。
