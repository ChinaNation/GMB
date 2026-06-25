# 任务卡:宪法不可修改条款节点共识守卫(L2)

- 卡号:20260624-constitution-immutable-guard
- 关联:ADR-027 §6.1、卡 `20260624-constitution-migration`(宪法已上链是前提)
- 模块:Blockchain Agent(citizenchain/node,极少量节点端)
- 状态:代码已落地并通过单测 + 编译,**待用户多节点真机 QA**

## 目标

宪法第 1/2/3/17/19/23/33/41 条为「真不可修改条款」:改代码、改 runtime 升级都改不动,改了**只能重新创世**。
纯 runtime 校验(L1)可被一次 setCode 解除,故把关搬出可升级 runtime,放节点共识层并锚定创世。

## 不变式

对 `law_id=0` 中条号 ∈ `[1,2,3,17,19,23,33,41]` 的条文,其全字段内容在**任意高度的当前状态**下必须与
**创世(block#0)状态逐字节相等**;违反即拒块。唯一修改路径 = 改创世(创世哈希变=新链)或改节点二进制(硬分叉)。

## 三层纵深

- **L1 运行时提案守卫**(已有):`legislation-yuan::ensure_immutable_preserved`,`propose_amend_law` 碰这 8 条即
  `ImmutableArticleViolation`。第一线、报错干净,但可被 runtime 升级绕过 → 非最终保证。
- **L2 节点共识守卫**(本卡核心):`node/src/core/constitution.rs::ConstitutionGuard` 包住 PoW `BlockImport`。
- **L3 创世锚 + 二进制锚 + 链上 manifest**:内容基准从 block#0 状态派生;清单 = `primitives::count_const::IMMUTABLE_CONSTITUTION_ARTICLES`
  常量编译进节点二进制(链上 WASM 改不到节点副本);`legislation-yuan` 新增只读 `ConstitutionImmutableManifest`(清单+逐条摘要,仅创世写),
  `genesis_build` 逐条强断言不可修改条款存在(缺即 panic),节点启动期交叉校验三者一致否则拒启。

## L2 实现要点(`node/src/core/constitution.rs`,统一节点端宪法能力单文件)

- 节点端宪法渲染(原 `other_tabs/constitution_render.rs`)**迁入本文件**,与守卫共用镜像类型;外壳
  外壳合并为**单文件** `core/constitution_shell.html`(整页 head/style/封面/目录/正文容器,两个占位标记
  `<!--CONSTITUTION_TOC-->`/`<!--CONSTITUTION_CONTENT-->` 渲染时 `replace`);`rpc.rs` 改引 `crate::core::constitution`;旧渲染文件删,零残留。
- `ConstitutionGuard::new` 装配时从 block#0 状态 RAW 派生不可修改条款基准(条号→规范 SCALE 字节)。
- `detect_violation`:对携带 body 的非状态同步块,用 runtime API 在**父状态**只读执行(`execute_block(block.into())`,
  LazyBlock)取 `into_storage_changes`;仅当变更触及立法院模块前缀(`twox128("LegislationYuan")`)才走慢路径;
  据「变更 ∪ 父状态」RAW 重建 `Laws[0]`→`current_version`→`LawVersions[0][v]`→比对基准;命中违规返回 `true`。
- key 推导硬编码 `Blake2_128Concat`(`Laws: u64`、`LawVersions: u64+u32`),**不读链上 metadata**。
- `import_block`:`Ok(true)`→`ImportResult::KnownBad`(内层不调用,块不入库不成最佳);`Ok(false)`→委派内层正常导入
  (只读执行不改提交路径);`Err`(守卫自身执行/取数失败)→放行内层(决定论下内层一致处理,避免守卫 bug 误停全链)。
- `service.rs` 两处装配:`new_partial` 网络导入队列 + `new_full` 本地挖矿 worker —— 诚实节点既不接受也不产出违规块。

## 威胁覆盖

| 攻击 | 结果 |
|---|---|
| 普通 amend 碰 8 条 | L1 `ImmutableArticleViolation` |
| setCode 删 L1 再 amend / migration 直写 `LawVersions` | L2 拒块 |
| setCode 把清单常量改空 | L2 用二进制副本,无视链上 → 拒块 |
| setCode 改 `current_version` 指向篡改版本 | L2 读当前版本 → 拒块 |
| 改 pallet/storage 名让 key 落空 | fail-safe 拒块 |
| 真要改 8 条 | 改创世 + 改二进制 = 重新创世 ✓ |

## 验收

- 已过:`cargo check -p node` 绿(无新增警告);`core/constitution.rs` **10 单测**(key 推导/基准派生/可变条改放行/
  不可修改条改拒/删条拒/存储缺失拒/渲染锚点 + manifest 一致放行/清单不符拒/摘要篡改拒)全过;`legislation-yuan` **16 单测**
  (含 genesis manifest 清单+逐条摘要断言)全过 + no_std 编译绿;fmt 干净;残留扫描 `constitution_render` 0。
- **待用户做**:多节点真机 QA —— ① 起多节点链;② 构造一个"恶意改第一条"的块(如临时恶意 runtime + amend 或直写存储);
  ③ 验证诚实节点拒该块、链停在前一块、坏块全网 orphan;④ 正常 amend 可变条仍正常上链。
- 关键风险点(QA 重点):`BlockImport` 只读执行 + `into_storage_changes` 在本 polkadot-sdk fork 下的实际行为;
  双执行(守卫只读执行 + 内层正常执行)对 PoW 出块/同步性能的影响。

## 加固五项(2026-06-24,review 发现绕过面后追加,ADR-027 §6.2)

用户 review 出 5 个绕过面,合并一轮加固(决策:H2 校验导入态保 warp、#5 仅展示端、H1 连 houses 钉):

- **H1**(node `core/constitution.rs`):`check_immutable_articles` 补 `Laws[0]` 元数据(`tier==Constitution`/`scope==0`/`status!=Repealed`/`houses==创世`)+ `LawsByScope[宪法][0]==[0]` 唯一性;`MLawHead` 补解 `status`;判别常量 `TIER_CONSTITUTION=0`/`LAW_STATUS_{PENDING=0,REPEALED=2}`,由 legislation-yuan `enum_discriminants_match_node_guard` 钉死。**status 不钉 Effective**(放行修宪 Pending 窗口)。
- **H2**:`import_block` 对 `with_state()`(warp)块改为"内层导入后 `verify_committed_state` RAW 校验",违规 `KnownBad`。
- **H3**:`rpc.rs::constitution_getDocument` 改 `StorageProvider` RAW 读(不走 runtime API,删 `LegislationApi` bound)+ `effective_version_of_law`(Pending 回退前一版),`source="legislation-raw"`。删死函数 `current_version_of_law`。
- **H4**(runtime `legislation-yuan`):`propose_enact_law` 拒 `tier==Constitution`(新 `Error::CannotEnactConstitution`)。
- **fail-open**:保留(决定论兜底),文档化;宪法读/解码/比对本就 fail-closed。
- 验收:node 17 + legislation-yuan 18 单测全过 + no_std + fmt。待 QA:H2 warp 多节点(实现风险最高)。

## 不做(边界)

- 链上 manifest:**已加固**(2026-06-24,见 L3)—— `legislation-yuan` 加只读 `ConstitutionImmutableManifest` storage + `ImmutableManifest` 结构 + `genesis_build` 写入与强断言(动 runtime,经用户二次确认);节点 `verify_manifest` 启动交叉校验。runtime test 16(含 manifest 断言)+ node test 10(含 3 manifest 校验)全过。
- 立法机构选举体系(citizen-vote 选举→admins 通道)仍是独立待开卡,与本卡无关。
