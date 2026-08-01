# 公民宪法联邦政府改名 + reward-bind 去出块门槛 + 章节标题纳入修宪判定（重新创世）

状态：open（2026-08-01 开工，用户逐条确认改动内容后执行）

## 背景

用户要求把宪法中「第二章第一节 总统府」改为「第一节 联邦政府」，并把
「中华民族联邦共和国国家联邦政府」中冗余的「国家」二字去掉。理由：全称
「中华民族联邦共和国」本身即指国家，联邦政府是该国的联邦政府，再加「国家」重复。

**本次改动后即正式创世，此后不再创世，只能通过 runtime 升级或链上修宪。** 因此把两个
原本计划留到后续 runtime 升级的缺陷一并修掉。

链上机构实体不受影响：宪法所指「联邦政府」是由总统府、外事交流部等部门组成的政府整体
（概念层），而链上「总统府」是机构实体，两者不是一回事，机构命名、CID 全称/简称生成器
与公权机构根均不改。

## 一、宪法改动（8 处，中英各 8）

真源：`citizenchain/runtime/public/legislation-yuan/src/constitution.scale`
（226,398 字节 SCALE 二进制，改前 SHA256 `3408748b0a17c8fc638f246887ffaf03…`）。
无源文本、无生成器，`include_bytes!` 直接编入 runtime。

| # | 位置 | 中文 | 英文 |
| --- | --- | --- | --- |
| 1 | 第二章 节标题 | 第一节 总统府 → **第一节 联邦政府** | Section 1 Presidential Office → **Section 1 Federal Government** |
| 2 | 第八条第七款 | 删「国家」 | 删 `national` |
| 3 | 第九条开头 | 删「国家」 | 删 `national` |
| 4 | 第九条中段 | 「国家联邦政府的军事权…」→ **「其军事权…」** | 改为 `Its military power… shall be vested`，删 `of the national federal government` |
| 6 | 第五十三条 | 删「国家」 | 删 `national` |
| 7 | 第五十四条 | 删「国家」 | 删 `national` |
| 8 | 第五十五条 | 删「国家」 | 删 `national` |
| 9 | 第五十五条第四款 | 删「国家」 | 删 `national` |

### 不改（3 处，用户逐条确认）

| # | 位置 | 原因 |
| --- | --- | --- |
| 5 | 第九条后段 | 「国家联邦政府授予省联邦政府」——授予方与被授予方并举，「国家」承担层级区分，删掉会变成联邦政府授予自身子类 |
| 10 | 第五十七条 | 「省联邦政府是国家联邦政府派驻…」——同上，派驻方与被派驻方并举 |
| 11 | 第九十一条 | 用户确认不改 |

改后全文「国家联邦政府」仅剩 3 处，全部位于需与省级对举的语境。

### 实施方式

`.scale` 是 SCALE 编码，字符串带 compact 长度前缀，删字会改变前缀值且可能跨越 compact
编码长度边界，**禁止直接字节替换**。必须用 runtime 原类型
（`ChaptersOf<T>` = `BoundedVec<Chapter>`，章>节>条>款）解码 → 改字符串 → 重新 `Encode`
写回。字符串上限为标题 256 / 正文 8192 字节，本次只删字不加字，不触碰上限。

验证：重新解码后逐层比对章/节/条/款数量与改前一致；除 8 处目标字符串外字节级一致。

## 二、reward-bind 去掉出块门槛

见 [20260731-reward-bind-drop-block-gate.md](20260731-reward-bind-drop-block-gate.md)。
删除 `fullnode-issuance/src/lib.rs` 中 `bind_reward_account` 的
`MinerNeverAuthoredBlock` 检查，语义改为「预绑定」。保留
`RewardAccountAlreadyBound` 与 `RewardAccountCannotBeMiner` 两道检查。
原定等后续 runtime 升级，因本次重新创世顺带完成。

## 三、章/节标题纳入修宪档位判定

### 缺陷

`constitution_amendment_scope` 计算变更条号时，`all_article_numbers` 与 `find_article`
只遍历 `sections → articles`，**完全忽略 `Chapter.title` 与 `Section.title`**。后果：

1. 只改章节标题的修宪提案会被 `classify` 判为 `NoChange` → 提案被拒，标题永远改不了；
2. 修宪提案顺带改标题时，该部分不计入档位判定，等于绕过表决门槛。

### 修法（用户确认语义）

**禁改保护的对象是「条」本身，不是它所在的标题。** 标题改动按所属章走档位，不触发
`ImmutableViolation`。

并行算两个 scope 取更严的一档，`classify` 签名与实现一字不改：

- 条文 diff → 原有 `classify(&changed, &core, &IMMUTABLE)`
- 标题 diff → 新增 `title_amendment_scope`：核心章（第一章）的章标题或其下任一节标题变更
  → `CoreChapter`；第二章及以后 → `GeneralOnly`；无变更 → `NoChange`
- 严格性排序：`ImmutableViolation` > `CoreChapter` > `GeneralOnly` > `NoChange`

章/节按 `number` 对齐而非下标，防止增删导致错位。

禁改条仍由 `ensure_immutable_articles_unchanged` 逐字冻结独立保护（在 scope 判定之前
执行，比对 `Article` 完整编码字节），两条路径正交。

### 顺带订正

`primitives/src/constitution.rs` 顶部注释称「runtime 与节点守卫复用同一份 `classify`」
与实际不符：节点守卫走 `check_core_chapter_tier` 背书路径（`ImmutableReference.core_articles`
存创世核心章非禁改条字节供 diff），不调用 `classify`。`classify` 仅在 runtime 侧使用。

## 四、宪法守卫检查结论（动手前已验证）

`ConstitutionGuard::new` 从 **block#0 派生基准**（`from_raw_reader(read_genesis)`），
创世状态本身即基准，守卫据此校验后续区块，不反向校验创世内容是否合法。重新创世即
重新定义基准，**不存在被旧基准拦截的问题**。

三条依据：① 不可改条款基准从创世现取，只要求 8 条存在、不校验内容应为何，且本次改动
的第 8/9/53/54/55 条均不在禁改清单；② 节标题在守卫视野之外（`find_article` 不读 title）；
③ `verify_imported_state`/`check_delta` 作用于导入区块，创世块不走该路径。

## 五、重新创世流程

1. 改宪法 8 处 + reward-bind + 标题 diff
2. `cargo fmt` / `clippy -D warnings` / 全 workspace 测试
3. `check-constitution-genesis.py` 校验
4. 推轻量候选 tag → 手动跑唯一 `CitizenChain WASM` CI
5. 用该 CI 产物执行 `bake-chainspec.sh --finalize`，同批重生节点 plain SSOT、App 轻形态
   chainspec、light-sync checkpoint、43 个公权机构分片、Cloudflare 链身份
6. 跑 `CitizenChain` 四平台 CI 出安装包
7. 三台节点重装：**贵州省 → 中枢省 → 国储会**（逐台验证后再动下一台，国储会最后，
   它持全网唯一 GRANDPA 权威与 Cloudflare Tunnel 出口）
8. Worker 重新部署（新 `genesis_hash`/`state_root`）
9. App 重新打包发版
10. 回写 `memory/`：`chainspec-frozen.md` 新锚点及全部相关文档

## 六、已知影响

- **现有链上数据全部作废**：当前链 #9、国储会矿工账户 19,998 元挖矿收益及已绑定收款账户、
  中枢省与贵州省各 1,000 元转账，重新创世后归零。均为测试期数据。
- **node-key 与 GRANDPA 密钥不变**，三台 PeerId 不变，chainspec 的 5 条 bootNodes 与
  DNS 均不需改动。
- cloudflared 为独立 unit，不碰链数据，节点重装不影响 Tunnel。
