# OnChina 公民/私权机构 链→注册局本地库投影

任务需求:
补齐公民 + 私权机构的链→本地投影(现只有公权机构有),让创世实体在对应注册局本地库可见:
- 创世公民程伟 GZ000-CTZN6(基金会法定代表人,无 citizen-identity,待补档态)。
- 创世私权机构 公民链技术发展基金会 GZ018-SFGYR。
联邦省组管理员进入本省某市时,投影该市链上全部公民+私权机构进联邦本地库。

完整技术方案:memory/01-architecture/citizenchain/onchina-citizen-private-projection-design.md

所属模块:citizenchain/onchina(链读 + 投影 + 本地库 + 前端)

必须遵守:
- 本地库=链上副本+链下正本;投影只对链上字段权威,链下正本(passport_no/档案/证件/补录)绝不被覆盖(字段级 UPSERT)。
- 幂等:路径A PK 存在即跳过(不每次启动回填);路径B 按(省,市)anchor 门控。
- fail-closed:链不可达不回退本地生成。
- 正常 local-first 实体不投影。
- 联邦访问控制:省组管理员只能进本省的市,越省 403。

两条路径:
- A 市/省注册局启动创世回填(定向已知 CID,零扫描,程伟从基金会 legal_representative 取,待补档态)。
- B 联邦按市 scoped 投影(drill-in 触发,扫 CidRegistry+PrivateManage::Institutions 过滤该市,含 legal-rep-only 公民)。

分步(一次交付):
1. 私权机构投影(基金会可见)
2. 公民创世回填 + 待补档态(程伟可见)
3. 联邦按市投影端点 + anchor + 省级访问控制
4. 前端:选市触发投影 + 待补档徽标 + 补档完成放行推链

输出物:代码 + 中文注释 + 单测/集成测试 + 文档 + 残留清理
验收:程伟+基金会在对应注册局本地列表可见;补档→推链→状态转 NORMAL;保正本 UPSERT 覆盖测试通过;越省 403;cargo check/test + 前端 tsc 通过。

状态:已完成(方案定稿 + 全部实现 + 测试 2026-07-24)

创世实体落地简化(已定死,见设计文档 §2.5):
- 基金会 GZ018-SFGYR(私权)→ 绥阳市注册局节点启动回填 subjects(PRIVATE) DO NOTHING。
- 程伟 GZ000-CTZN6(公民)→ 直接落联邦注册局本地库,province=GZ、city=绥阳(018),现有省+市列表即可见;不加省级视图,不用 PENDING/待补档,不改 canPushOnchain。
- 两条纯新增 DO NOTHING,不动现有写入器与推链门。
先落这两条(立即让程伟+基金会可见),M1(indexer 增量)/M3(联邦 drill-in 通用投影)随后。

进度:
- [x] 程伟联邦播种:domains/genesis_projection.rs::seed_genesis_citizen_blocking(联邦节点启动播种程伟,
      province=GZ/city=绥阳018,幂等跳过,纯新增无链依赖,不动现有写入器/推链门),main.rs 启动接线。
      cargo check 通过、零警告。程伟在联邦可见/可按 GZ000-CTZN6 搜。
- [x] 绥阳基金会回填(私权):genesis_projection.rs::backfill_genesis_private_blocking
      (属主市节点=基金会省市时读链 institution_lookup(GZ018-SFGYR)→ 建 Institution(私权法人/
      legal_rep 程伟)→ upsert_institution_row;存在跳过;非属主/联邦跳过;链不可达告警不阻断)。
      main.rs 启动接线。cargo check 通过、零警告。基金会在绥阳市注册局可见,法定代表人=程伟。
- [x] 投影纯逻辑 domains/projection.rs::merge_citizen_record / merge_institution_record
      (作用域过滤 + 链上列权威 + 链下正本保留 + 竞选姓名覆盖/投票保留);**9 单测全绿**含两类保正本
      (公民 passport_no/姓名/档案不覆盖;机构证件照/分类/创建人不覆盖)。巧办法:merge 带回正本行
      → 复用现有 upsert 也不丢正本,无需另写保正本 SQL。
- [x] M1 indexer 增量:chain_runtime::read_chain_citizen_detail(4 读)+ event_parser::collect_entity_projection_cids
      (事件→CID)+ worker 每块投影 + 断点续读复用现有 + 作用域解析(仅城市节点,联邦跳过)。
- [x] M3 联邦 drill-in:chain_runtime for_each_chain_citizen_cid_in_scope / for_each_chain_private_institution_cid
      扫描 + projection::drill_in_project_scope + HTTP handler drill_in_project_city
      (POST /api/v1/admin/registry/drill-in-project,联邦+省级访问控制,越省 403)。
- [x] 收尾:移除 allow(dead_code)、零警告零死代码;onchina cargo test 138 passed/0 failed。

全部完成(2026-07-24)。验证:onchina cargo check 通过零警告;cargo test 138 全绿(9 投影含保正本)。
链读/事件/HTTP 接线为编译级验证(无活链运行时);保正本/作用域/映射核心逻辑离线单测覆盖。
