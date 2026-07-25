# OnChina 公民/私权/公权机构 链↔注册局本地库一致性 技术方案 (v2)

状态:设计定稿 v2 + **已实现并测试**(2026-07-24)。任务卡 08-tasks/20260724-onchina-citizen-private-chain-projection.md。
关联:[[onchina-chain-projection-asymmetry-citizen-gap]]。

实现落点:
- 创世回填(§2.5):`domains/genesis_projection.rs`(程伟联邦播种 + 基金会绥阳回填,DO NOTHING 幂等)。
- 投影纯逻辑 + 保正本:`domains/projection.rs::merge_citizen_record / merge_institution_record`(9 单测,含两类保正本)。
- M1 indexer 增量:`core/chain_runtime.rs::read_chain_citizen_detail` + `indexer/event_parser.rs::collect_entity_projection_cids` + `indexer/worker.rs`(每块投影 + 作用域解析,仅城市节点)。
- M3 联邦 drill-in:`chain_runtime` 两个 scoped 扫描 + `projection::drill_in_project_scope` + `projection::drill_in_project_city`(POST `/api/v1/admin/registry/drill-in-project`,联邦+省级访问控制)。
- 验证:onchina `cargo test` 138 全绿(含 9 投影);链读/事件/HTTP 为编译级验证(无活链运行时),保正本/作用域/映射核心逻辑离线单测覆盖。
- 未做(前瞻/前端):待补档 PENDING 推链门(创世方案已绕开,不需要);M3 前端触发按钮(Step5 前端)。

## 0. 角色

- **市/县注册局(CREG)**:每市一节点一本地库,`is_tier1_registry`=false,作用域=本(省,市)。
- **联邦注册局(Tier1/FRG)**:**全国仅一节点一本地库**;43 个省组管理员,每人 `scope_province_name` 锁定一省,只能进本省下辖的市,直接在某市管理公民/私权/公权。
- **正本 vs 副本**:某注册局采集/创建的链下资料=该局**正本**;链上数据=各局共享**副本**。

## 1. 两类实体的根本差异(CID 结构决定,已核实)

CID 的 r5 段 = 省码(2)+ 市码(3)(number.rs:88-92)。

| | CID 市码段 | 真实市来自 | 属主判定 |
|---|---|---|---|
| **公权/私权机构** | 真实市码(如 `GZ018`、`ZS001`) | **CID 本身** | CID 自带市 → 市是唯一属主 |
| **公民** | `000`(省级占位,如 `GZ000-CTZN6`) | **链上 `residence_city_code`**(占号 `occupy_cid` 时写入 `CidRegistry`) | 无 CID 强制属主;正本随**采集/注册的注册局** |

## 2. 归属与对齐规则(定死)

### 机构(公权/私权)
- 联邦或市创建 → **市注册局补链下档案(待补档)**;联邦只建链上,市补链下;两边**对齐链上数据**。
- 链下正本永远在**市**(CID 属主)。

### 公民
- **档案真源在"注册该公民的注册局"**:本市办的 → 正本在本市;联邦办的 → 正本留联邦。
- 各市注册局**按作用域(链上 residence)各自维护自己的公民**,拿到的是**链副本**(姓名/居住地/护照窗口/状态,足够列表与资格判断);不强制向市补档。
- 联邦办的公民:联邦持链下正本 + 占号上链(residence=该市)→ 该市按 residence 回填链副本。

## 2.5 创世实体特例落地(本次立刻要修的两个,已定死)

创世时直接上链、未走注册局流程的仅两个,按各自 CID 特性分开落地(去掉早前"程伟塞进某市 + 待补档 + PENDING"的复杂做法):

- **基金会 `GZ018-SFGYR`(私权,CID 带市码 018=绥阳)** → 落 **绥阳市注册局(城市节点)**。绥阳节点(scope=GZ/绥阳)启动时 `institution_lookup(基金会CID)` → `INSERT subjects(PRIVATE) DO NOTHING`。绥阳看基金会时"法定代表人"字段自带程伟姓名(链上 `legal_representative`)。联邦贵州组管理员进绥阳市时经 M3 也能看到基金会。
- **程伟 `GZ000-CTZN6`(公民,CID 无市码,无 citizen-identity,无居住地)** → **直接落联邦注册局本地库**,置于 **province=GZ(贵州)、city=绥阳市码(018,与基金会同市)**。联邦贵州组管理员**进绥阳市→显示程伟**,进贵州其他市→不显示。**不加省级/创世特殊视图**——他就是联邦库里一条普通"省+市"公民记录,现有列表/搜索(`GZ000-CTZN6`)即可见。绥阳城市节点本身不持有程伟(他是联邦的)。姓名/CID/账户取自 `primitives` 创世常量 + 链上 `legal_representative` 权威值;`INSERT citizens + subjects(CITIZEN) DO NOTHING`。
- 两条均**纯新增、DO NOTHING、不动现有写入器与推链门**;程伟不用 PENDING/待补档,故**不改** `canPushOnchain`。
- M1/M3 仍是普通实体的通用机制,不受这两个特例影响。

## 3. 发现机制(市侧如何知道"我市有这个实体")

- **机构**:按 **CID 市码**。联邦一上链,市按 CID 市码即可发现。
- **公民**:按 **链上 `residence_city_code`**(占号写入 `CidRegistry`)。**公民只有上链(至少占号)后,市才可发现**;纯本地未上链的联邦公民不属于任何市、也不该被发现。
  - 规则:联邦新建、应归某市的公民,**必须占号上链(带 residence)**才落到该市。

## 4. 三种维护机制

### M1 · indexer 事件增量(各市注册局稳态主力)
- 每个 CREG 节点 indexer 订阅 finalized 区块;对本作用域事件 upsert:
  - 公民:`CidOccupied`/`VotingIdentityRegistered`/`VotingIdentityUpdated`/`CandidateIdentityUpgraded`/`CandidateIdentityUpdated`/`CitizenIdentityRevoked`/`CidRevoked` → 读该 CID 的 `CidRegistry`+`VotingIdentityByCid` 取 residence,`residence_city`==本市则 upsert。
  - 私权机构:私权创建/更新事件 → 读 `PrivateManage::Institutions[cid]`,CID 市码==本市则 upsert。
- 断点续读(记 last_processed_block):重启只补处理停机期间的新区块,**非全表扫**。
- 联邦省组管理员在某市新建的公民 X:X 上链的区块 finalized → 该市 indexer 近实时拿到 X 链副本。

### M2 · 启动创世一次性读(创世实体不在事件流,在 block0 状态)
- 节点启动,对 `primitives::cid::china` 已知创世 CID(基金会 GZ018-SFGYR;程伟 GZ000-CTZN6),做**单点状态读**:
  - CID(机构)/residence(公民)∈本作用域 且本地无 → 回填。
  - 只有属主市(绥阳)命中并插入;别的市对 2 个已知 CID 做一次廉价 scope 判断即跳过。**不是全国全扫。**
- 幂等:PK 存在跳过。

### M3 · 联邦 drill-in 按市投影(联邦无 indexer,按需)
- 联邦单节点不跑全国 indexer;省组管理员进入本省某市 → 对该市做**按作用域链读**投影进联邦本地库:
  - 公民:扫 `CidRegistry` 过滤 `residence`==该市 → 逐 CID 读明细 upsert。
  - 私权:扫 `PrivateManage::Institutions` 过滤 CID 市码==该市 → upsert + 账户;含 legal-rep-only 公民(如程伟)。
  - 公权:已有 `sync_gov_chain_projection`(全国),按市过滤展示即可,无需新读。
- 幂等:(省,市)anchor 记 finalized head,未变跳过;可手动刷新。
- 访问控制:目标省==管理员 `scope_province_name`,市属该省,越省 403。
- **触发点(as-built 2026-07-24)**:M3 私权投影已接进读取路径
  `institution/subjects/registration.rs::list_institutions_inner`——联邦(Tier1)管理员按市查私权列表时,
  先 `projection::drill_in_project_private_scope((省,市))` 再查库。链读失败只 warn 不阻断(fail-open)。
  → 解决"联邦贵州组管理员进绥阳市看不到基金会"报障:基金会正本在绥阳市注册局,联邦按需 drill-in 才可见,非播种。

> 稳态:各市靠 M1;创世靠 M2;联邦看外市靠 M3。全状态扫描仅极端兜底。

## 4.5 正本补档:公民编辑(不可变字段锁定,as-built 2026-07-24)

创世/待补公民(如程伟)投影落库后正本残缺(护照有效期空 → `computed_identity_status` 判注销),
需注册局补齐正本才能推链。为此加 `POST /api/v1/admin/citizens/:cid/edit`
(`admin_entry::admin_update_citizen`):

字段可变性按"现实是否可变"定死:
- **可变**:姓、名、居住市、居住镇、voting_eligible(人会改名、会搬家)。姓名不进锁定集,
  handler 直接取 input(必填非空);居住市/镇限本省内改(按 existing.province_code 校验归属)。
- **不可变**(现实不可变,初始化后永久锁定):性别、出生日期、出生地(省市镇)、护照号。
  `lock_immutable`——现存非空即锁定拒改,空则允许初始化,存成功后永久锁定。
- **护照号/有效期**:不接受前端直填,服务端确定性签发(`allocate_passport_no` + 出生日期派生年限,
  与建档同源,命名空间用最终居住省市);现存已签发则锁定原样保留。保护护照号唯一性与有效期口径。
- **固定不动**:居住省 province_code(= CID 省 = 分区键)、公民 CID(主键)。
  **跨省居住迁移属"跨地区"(分区迁移 + 注册局交接),本入口不做,后续单独处理**。
- 链投影字段、账户、状态、onchain_*、创建人、created_at 一律保留(仍守保正本铁律,§10)。
- 前端 `citizens/EditCitizenModal.tsx`:姓名/居住市镇可编辑;性别/出生日期/出生地按现存值锁定
  (非空只读、空放开级联录入);居住省只读;居住市为固定省下级联 Select。

## 5. 字段映射 · 公民(citizens)

| 列 | 正常公民(有 citizen-identity) | 待补/联邦办公民 |
|---|---|---|
| cid_number | CID | CID |
| province_code | CID 省 / residence 省 | 同 |
| city_code | **链上 residence_city_code** | 同 |
| town_code | VotingIdentity.residence_town | '' |
| account_id | AccountIdByCid | AccountIdByCid / legal_rep.account_id |
| citizen_status | VotingIdentity(NORMAL/REVOKED) | PENDING(仅程伟这类无 identity 者) |
| voting_eligible | 派生(NORMAL+护照窗口) | false |
| passport_valid_* | VotingIdentity | '' |
| family/given/sex/birth_* | CandidateIdentity(有则) | legal_rep 姓名(程/伟)其余空 |
| passport_no/archive_hash/documents | **链下正本,投影不写** | '' |
| creator_account_id | 公民自身账户(来源锚点) | 同 |

- 市侧拿到的是**链副本**;链下正本(passport_no/证件)在**采集局**(本市或联邦),不跨节点同步。
- 程伟(无 citizen-identity,仅基金会法定代表人)→ PENDING 待补档,姓名从 `legal_representative` 取。

## 6. 字段映射 · 私权机构(subjects kind=PRIVATE + 明细)

- subjects:cid、kind='PRIVATE'、全称/简称、省/市/镇码(**CID 直接给省市**)、机构码、法人标记、`legal_representative_*`、created_at。
- 账户:`InstitutionAccounts` → 私权账户明细。
- 链下(证件照等)由**市补档**,投影不写。

## 7. chain_runtime 新增

- `for_each_chain_private_institution(cb)` + `read_chain_private_institution(cid)`(镜像公权,码校验 `is_private_legal_code`)。
- `for_each_chain_citizen(scope, cb)`(扫 `CidRegistry` 过滤 residence)+ `read_chain_citizen_detail(cid)`(Voting/Account/Candidate 单点)。
- `read_foundation_legal_representative(inst_cid)`(取程伟)。
- 复用 `for_each_chain_institution_account`(已支持 PrivateManage)。

## 8. indexer 扩展

- `event_parser.rs`/`worker.rs`:识别公民/私权/私权机构相关事件 → 以 CID 为线索**读状态 + scope 过滤 + upsert**;记 `last_processed_block` checkpoint,重启续读。
- 联邦节点不启用该 scope indexer(按 M3 drill-in)。

## 9. 本地库 schema

- 复用 subjects/citizens/私权明细。
- 新增 `indexer_checkpoint(node_scope, last_block, updated_at)`(M1 续读)。
- 新增 `projection_anchor(domain, province_code, city_code, genesis_hash, block_hash, block_number, synced_at, PK)`(M3 幂等)。
- `citizen_status` 允许 `PENDING`(现 DDL 无 CHECK);前端列表加"待补档"徽标,不隐藏。

## 10. 保正本 UPSERT(铁律)

```
INSERT ... ON CONFLICT (province_code, cid_number) DO UPDATE SET
  <仅链上来源列> = EXCLUDED.<...>
-- 链下正本列(passport_no/archive_hash/documents/证件/市补录) 不进 UPDATE SET → 保留
-- 姓名:EXCLUDED 为空则保留本地(NULLIF/COALESCE)
```
机构同理。任何"整行写"都会抹掉链下正本 —— 最易埋 bug 处。

## 11. 访问控制

联邦 Tier1:省组管理员 `scope_province_name` 锁省;M3 目标 city 必属该省;跨省 403(复用 `get_visible_scope`/`includes_province`)。

## 12. 分步实现(一次交付,内部)

1. 私权机构链读 + subjects(PRIVATE)/明细 upsert(基金会可见,市补档)。
2. 公民创世一次性读 M2 + 待补档态(程伟可见)。
3. indexer 扩展 M1(各市稳态自维护,含联邦新建公民近实时到市)。
4. 联邦 drill-in M3 端点 + anchor + 省级访问控制。
5. 前端:待补档徽标;补档完成放行 PENDING 推链;联邦选市触发 M3。

## 13. 测试

- 单测:字段映射;PENDING 待补档;**保正本 UPSERT(补录后再 upsert 不覆盖)**;residence/CID 市码 scope 过滤;越省 403;indexer 事件→scope→upsert;checkpoint 续读。
- 集成(离线 fake chain):创世回填→程伟+基金会可见;联邦某市新建公民→该市 indexer 拿到链副本;机构联邦创建→市待补档→补齐→对齐链上;M3 drill-in 投影一市全量。

## 14. 规模 / 回滚

- 稳态零全表扫(M1 事件增量 + checkpoint);M3 按市 on-demand + anchor 缓存;M2 仅少数已知创世 CID。
- 回滚:投影只写本地库,清表/清 checkpoint/anchor 即回滚,不触链。
