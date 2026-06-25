# 20260624 宪法迁移到立法院模块 + 法律统一章节条款

依据:`memory/04-decisions/ADR-027-legislation-yuan.md`;前置=链端立法卡(已完成)。
立法第2步(双客户端卡 / 本卡=宪法迁移)的第2步。

## 背景 / 目标

- 公民宪法从「include_str! HTML + CitizenConstitutionApi」改为 legislation-yuan 里 `tier=宪法` 的链上法律(创世注入),宪法**唯一真源 = 立法院模块**。
- **所有法律统一 章>节>条>款 结构**(章/节/条必有、款可空、条正文 body 必填);章节条做目录,条款做正文。
- 宪法双语,其他法律单语。
- 节点桌面端「公民宪法」tab 保持现样式,改读结构化宪法。
- 迁移后删 `CitizenConstitution.html`(无用)。

## 已拍板(2026-06-24)

- 数据结构:全嵌套 章>节>条>款,章/节/条必有、body 必填、款可空、删「项」、双语(宪法填 _en,其他 None)。
- 解析器放各系统 `scripts/`,**禁止 tools/ 目录**(现存 tools/ 迁 scripts/ 后删,作残留清理)。
- 上链:宪法创世数据随现有待重新创世队列一次性 bake。
- houses=[国家立法院](引导期单机构;众/参拆分 + 选举体系另卡)。

## 数据结构

```text
LawVersion: law_id, version, title, title_en, chapters[], content_hash, vote_type, proposal_id, published_at, effective_at
Chapter(章): number, title, title_en, sections[]            // 必有
Section(节): number, title, title_en, articles[]            // 必有
Article(条): number, title, title_en, body, body_en, clauses[]  // 必有;body 必填
Clause(款): number, text, text_en                           // 可空
```
- Article.number = 全法连续条号(不可修改条款第1/2/3/17/19/23/33/41条按此校验)。

## Phase 1 — legislation-yuan 模型改造(链端,runtime 二次确认)

- types:新增 Chapter/Section;Article 加 title_en/body/body_en;Clause 加 text_en;删 Item;LawVersion.articles → chapters。
- Config:加 MaxChaptersPerLaw/MaxSectionsPerChapter/MaxArticlesPerSection;删 MaxItemsPerClause;body 用 MaxTextLen。
- 执行器 write_law_version 写嵌套 chapters;load object 解码 chapters;hash 对 chapters。
- 不可修改条款校验遍历 chapters>sections>articles 按 number 比对。
- propose_enact/amend_law 参数 articles → chapters;查询透传。
- 现有 14 测试改嵌套构造 + 补章节测试。
- configs 新常量装配。

## Phase 2 — 宪法解析器(citizenchain/scripts/)

- 读 CitizenConstitution.html(class: heading-cn 章/节、article-no 条、body-cn/body-en 条款中英)→ 结构化 LawVersion → SCALE → `constitution.scale`(include_bytes! 进 legislation-yuan)。
- 校验:解码回结构与 HTML 比对(章节条款 + 中英 + 条号连续 + 8 不可修改条款齐全)。

## Phase 3 — 创世注入(legislation-yuan genesis)

- #[pallet::genesis_config] + genesis_build:law_id=0,tier=宪法,houses=[国家立法院],status=Effective,从 include_bytes 解码写 Laws/LawVersions/LawsByScope。
- configs 配 GenesisConfig;node chain_spec 纳入 → 重新创世。

## Phase 4 — runtime API + 节点公民宪法 tab(re-point,现样式)

- LegislationApi.law_version 透传嵌套结构。
- node/src/core/rpc.rs(constitution_getDocument)+ other_tabs + frontend RuntimeConstitutionViewer 改读 LegislationApi tier=宪法 法律,章>节>条 下拉 + 条款中英,样式同现状。

## Phase 5 — 清理(禁止兼容 + tools 铁律)

- 删 primitives/genesis.rs HTML include + CitizenConstitutionApi decl、apis.rs impl、**CitizenConstitution.html 文件**、节点旧宪法 HTML 路径。
- 残留扫描 CitizenConstitution/citizen_constitution_html/constitution_getDocument 零残留(文案保留)。
- tools 收口:现存 tools/ 迁 scripts/ 后删(先核引用,安全迁移)。

## 预计修改目录

- `citizenchain/runtime/governance/legislation-yuan/`(模型 + 创世 + constitution.scale;代码+测试)
- `citizenchain/scripts/`(解析器)
- `citizenchain/runtime/primitives/src/genesis.rs` + `CitizenConstitution.html`(删 HTML/API + 删文件)
- `citizenchain/runtime/src/{apis,configs/mod,core/chain_spec}.rs`
- `citizenchain/node/src/core/rpc.rs` + `src/other/other_tabs/` + `frontend/other/other-tabs/`
- `memory/`

## 验收

- cargo test -p legislation-yuan + runtime cargo check(std+no_std)+ node cargo build 全绿。
- constitution.scale 解码 = HTML 内容。
- 真实运行态:节点端宪法 tab 现样式显示(章>节>条 + 中英);非核心修宪走重要案;核心条款修改硬拒;残留零。

## 进度

- [x] **Phase 1 模型改造(2026-06-24 完成)**:legislation-yuan 全文模型由「扁平 Article」改为 `章(Chapter)>节(Section)>条(Article)>款(Clause)`;Article 加 title_en/body(必填)/body_en,Clause 加 text_en,删 Item;LawVersion.articles→chapters;Config 删 MaxItemsPerClause/MaxArticlesPerLaw,加 MaxChaptersPerLaw/MaxSectionsPerChapter/MaxArticlesPerSection;不可修改条款校验改遍历 chapters>sections>articles 按 number(find_article helper);EmptyArticles→EmptyChapters;configs 常量同步(MaxTextLen 4096→8192);ChaptersOf 别名。验收:cargo test -p legislation-yuan 14 + runtime cargo check(std+no_std)+ legislation-vote 12 无回归;fmt 干净。
- [x] **Phase 2 解析器(2026-06-24 完成)**:`citizenchain/scripts/parse_constitution.py` 读 HTML(块状 chapter/section/article-block + article-paragraph,EN heading 给阿拉伯条号)→ 章>节>条>款 + 中英双语 → 直出 SCALE(与链端字段序一致,自带 compact 编码)→ `legislation-yuan/src/constitution.scale`(217KB,原 HTML 933KB)。产物:7章/28节/140条/129款,条号连续 1..140,无空 body。验证:Rust 测试 `constitution_scale_decodes_and_is_well_formed` 解码进 ChaptersOf,7章140条+条号连续+body双语+8不可修改条款齐全。
- [x] **Phase 3 创世注入(2026-06-24 完成)**:legislation-yuan `#[pallet::genesis_config]`(constitution_houses,默认 [国家立法院]=china_lf CHINA_LF[0] + NLG 码)+ `genesis_build`(从 CONSTITUTION_SCALE 解码,写 law_id=0 tier=宪法 status=Effective version=1 title=公民宪法/Citizen Constitution);runtime 自动纳入 RuntimeGenesisConfig(default 即注入)。验证:`genesis_seeds_constitution_as_law_zero` 创世后宪法=law_id=0 tier=宪法 7章140条 houses=[国家立法院];legislation-yuan 16测 + runtime cargo check(std+no_std)全绿。
- [x] **Phase 4 节点 tab re-point(2026-06-24 完成,现样式)**:节点据链上结构化宪法重建 HTML,复用原 CSS 外壳,前端 iframe **零改动**,样式与迁移前一致。
  - 抽原 HTML 表现外壳为节点资源:`node/src/other/other_tabs/constitution_shell.html`(1-521 行:head/style/封面/目录标题,止于 `<div class="toc-list">`,624KB 主要为封面国徽 base64)+ `constitution_shell_suffix.html`(`</main>`+置顶脚本+`</body></html>`)。
  - 新建 `node/src/other/other_tabs/constitution_render.rs`:`MLawHead`/`MLawVersionHead`/`MChapter/Section/Article/Clause` 镜像(字段序与链端逐字段核对一致,SCALE 顺序解码到 chapters/current_version 即停,尾部字段不镜像);`current_version_of_law` 解当前版本号;`render_constitution_html` 据 章>节>条>款 重建 TOC(`toc-item toc-level-{1,2,3}` + 锚点 `#chapter-N/#chapter-N-section-M/#article-K`)+ 正文(`chapter-block/section-block/article-block` + `cn/en heading/body` 类,与原标记逐字一致)+ HTML 转义。
  - `node/src/core/rpc.rs`:API bound `CitizenConstitutionApi`→`LegislationApi`;`constitution_getDocument` 改为 `law(0)`→解 `current_version`→`law_version(0, v)`→重建 HTML→按内容算 blake2_256,`source` 改 `"legislation"`(读最新生效版,修宪后自动跟随)。
  - 前端 `RuntimeConstitutionViewer.tsx`/`api.ts`/`types.ts` 与 `other_tabs/mod.rs` **不改**(响应形状 `{html,blake2_256,source}` 不变,`source: string` 非字面量)。
  - 验收:`cargo check -p node` 绿;`constitution_render` 2 单测过(锚点/双语/款/哑尾忽略);fmt 干净。
- [x] **Phase 5 清理(2026-06-24 完成)**:删 `genesis.rs` `CITIZEN_CONSTITUTION_HTML` 常量 + `CitizenConstitutionApi` decl、`apis.rs` 对应 impl、**`CitizenConstitution.html` 文件**(git rm,933KB);`decl_runtime_apis!` 仅留 `LegislationApi`。残留扫描:`CitizenConstitutionApi`/`citizen_constitution_html`/`citizen_constitution_blake2_256`/`CITIZEN_CONSTITUTION_HTML` **代码零残留**(仅描述性注释保留);`constitution_getDocument` RPC 名保留(现由结构化宪法支撑)。验收:primitives no_std + runtime/node std 编译绿;legislation-yuan 16 + node 2 + primitives 24 测试全过。
  - 解析器溯源:`parse_constitution.py` 改为「一次性迁移工具」状态说明(产物已入库,输入 HTML 按单一真源已删,需重算先 git 恢复 HTML);章/节标题与款正文改存**完整原文**(目录显示「第一章 总则」、款含「第N款」前缀),number 仅供锚点;删死函数 `strip_en_prefix`。constitution.scale 重生 = 219KB(7章/28节/140条/129款)。
  - tools 收口:本卡解析器自始置于 `citizenchain/scripts/`(从未建 tools/),无需迁移;其余系统遗留 tools/(whitepaper、citizenapp bundle)不在本卡边界,不动。
  - 文档同步:`README.md`、`memory/07-ai/unified-naming.md`、`OTHER_TABS_TECHNICAL.md` 公民宪法真源指向改为链上立法院模块。
  - **待用户跑**:重新创世后真机 QA —— 节点桌面端宪法 tab 现样式显示(章>节>条 + 中英)、blake2 随内容、CitizenApp 浏览(双客户端卡另线程)。
- **后续(2026-06-24,新卡 `20260624-constitution-immutable-guard`)**:Phase 4 的节点渲染(原 `other_tabs/constitution_render.rs` + 外壳)已**统一迁入 `node/src/core/constitution.rs`**,与新增的「不可修改条款 L2 共识守卫」共用镜像/外壳;`rpc.rs` 改引 `crate::core::constitution`,旧渲染文件删、零残留。外壳 prefix/suffix 两文件**并为单文件** `core/constitution_shell.html`(两占位标记 `<!--CONSTITUTION_TOC-->`/`<!--CONSTITUTION_CONTENT-->` 渲染时替换,整页结构一处维护)。不可修改条款真不可变性见该卡与 ADR-027 §6.1。
