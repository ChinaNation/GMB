# 官网新增「公民宪法」tab:经 Cloudflare Worker 读链渲染(白皮书风)

任务需求：
官网(citizenweb)在「白皮书」右侧、「关于我们」左侧新增「公民宪法」tab,显示链上唯一真源的公民宪法。经 Cloudflare Worker(已有到国储会节点的 CF Access 反代)读链 → Worker 内 TS 解码 SCALE → 官网用白皮书样式自渲染(左目录树+右正文+不可修改徽章+版本标签+中英双语)。修宪后自动更新。四端(公民App/节点/链上中国/官网)同源同一份宪法。

所属模块：citizenweb(官网前端)+ citizenapp/cloudflare(Worker,读链新端点)；节点/链零改。

输入文档：
- citizenchain/node/src/core/rpc.rs(constitution_getDocument:RAW 读语义参考)
- citizenchain/node/src/core/constitution/mod.rs(storage_key:Pallet=LegislationYuan,Laws/LawVersions/LawVersionLabels/ConstitutionImmutableManifest)
- citizenchain/scripts/check-constitution-genesis.py(parse_law/parse_version/parse_manifest 可移植底本;当前 `houses` 为 `Vec<CidNumber>`,每项按 SCALE `Vec<u8>` 读取)
- citizenchain/runtime/public/legislation-yuan/src/lib.rs(Law/LawVersion/Chapter/Section/Article/Clause/LawVersionLabel 字段序)
- citizenchain/node/src/core/constitution/render.rs(渲染对齐:双语/徽章/目录)
- citizenapp/cloudflare/src/chain/{rpc,storage_key,identity}.ts(已有 state_getStorage + xxhash/blake2 + SCALE readCompact 原语)
- citizenweb/src/pages/Whitepaper.tsx + src/index.css(whitepaper-* 样式,60 条,含双语类)

必须遵守：
- 读链只用已放行的 state_getStorage(不改 chain/rpc.ts 方法白名单);经 Worker,浏览器不直连节点
- RAW 读存储(与节点 constitution_getDocument 同安全口径,不走 runtime API)
- 只展示 effective_version(不露修宪待生效版,ADR-027 §6.1);effective_version=None 时 404/空
- 官网结构化渲染用 JSX,不用 dangerouslySetInnerHTML
- 复用 whitepaper-* 样式;tab 位置严格在白皮书右/关于我们左
- 宪法极少变 → Worker 加缓存(Cache API 短 TTL)
- SCALE 解码字段序逐字节对齐 runtime 结构

输出物：
- Worker:src/chain/constitution.ts(存储键 + SCALE 解码 Law→effective_version / LawVersion→章节树 / manifest→不可修改条号 / label→版本标签)+ 路由 GET /v1/constitution(公开、缓存)+ routes.ts 注册/assertKnownRoute
- 官网:Header tab + App 路由 + src/pages/Constitution.tsx + index.css 补 constitution-* 徽章/版本
- 中文注释 + Worker vitest(解码器对 constitution.scale 章节夹具断言;Law/manifest/label 小夹具)
- 文档更新(memory/05-modules 相关)+ 残留清理

验收标准：
- Worker tsc + vitest 全绿;官网 tsc/eslint 通过、build 通过
- /v1/constitution 返回结构化 章>节>条>款 + 中英 + 不可修改条号 + 版本标签
- 官网 tab 显示与节点同一份宪法(含刚改的第三章教委会/第四章储委会)
- 修宪(链上换 effective_version)后官网自动反映
- 文档更新、残留清理、Review 处理

## 进度

- [x] Worker:storage key(storageValueKey/storageDoubleMapKey/encodeU64Le/encodeU32Le)+ SCALE 解码模块 `chain/constitution.ts`
- [x] Worker:GET /v1/constitution 路由 + KV 短缓存 + routes 注册 + catalog 白名单 + guard 公开放行(复用现有 CORS)
- [x] Worker:vitest(真 constitution.scale 章节树夹具断言教委会/储委会;Law/manifest/label 夹具;端到端 mock 链读)——8 项;全套 143 全绿;tsc 干净
- [x] 官网:Header tab(白皮书右/关于我们左)+ App lazy 路由 + Constitution.tsx + index.css(constitution-*)
- [x] 官网:tsc -b + vite build 通过(Constitution 独立懒 chunk 1.73kB gz)、eslint 通过
- [x] 浏览器验证:桌面+移动截图,tab 位置正确、白皮书风目录/双语/徽章/版本、第三章教委会第四章储委会、无 console 错误;临时 mock 已删
- [x] 文档更新(CITIZENWEB_TECHNICAL 3.3 + 模块定位)+ 残留清理(无遗留)
- [x] 2026-07-18 回归修复:runtime `Houses` 已是 `Vec<CidNumber>`，Worker 旧解码仍按 House 定长 36B 跳过，导致线上 `Laws[0].effective_version=Some(1)` 被读偏成 `None`，官网显示“链上宪法尚无生效版本”。已改为逐项读取 `CidNumber(Vec<u8>)`，并用 26 字节线上同型 CID fixture 覆盖回归测试。
- [ ] 【部署】wrangler 部署 Worker + 官网构建部署后线上核对(读链需现网 CHAIN_URL;可选设 CONSTITUTION_TTL_SECONDS)
