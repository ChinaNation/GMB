# 宪法守卫技术文档

## 1. 定位

`ConstitutionGuard` 是公民链最高优先级、独立最外层的原生 `BlockImport` 包装器。网络导入与本地挖矿固定为：

```text
ConstitutionGuard<NodeGuard<PowBlockImport>>
```

宪法守卫不得并入 `NodeGuard`。展示代码位于 `render.rs`，不得调用、绕过或改变守卫判定。

## 2. 代码边界

| 路径 | 职责 |
|---|---|
| `core/constitution/mod.rs` | RAW storage key、SCALE 镜像、创世基准和纯不变式 |
| `core/constitution/guard.rs` | 启动、正常区块、预计算 delta、warp 和内层导入编排 |
| `core/constitution/render.rs` | 宪法 HTML 渲染，不参与共识 |
| `core/constitution/constitution_shell.html` | 桌面展示外壳 |

## 3. block#0 基准

节点启动时从 block#0 RAW 状态派生不可修改条款、核心章非禁改条款、修宪机构和 manifest 基准，并执行完整不变式校验。以下任一异常都拒绝启动：

- `Law(0)` 或创世版本缺失、解码失败；
- `law_id/tier/scope/status/版本指针` 非法；
- manifest 与二进制清单或创世条文摘要不一致；
- `LawVersion` 身份、内容哈希、条号唯一性不合法；
- 版本 key 集不严格等于 `1..=latest_version`；
- 任一历史版本破坏不可修改条款或缺少既有凭据记录。

启动和完整态检查枚举真实 storage key，不根据不可信 `latest_version` 做大范围循环。

## 4. 永久不变式

- 宪法固定为 `Laws[0]`，值内部 `law_id=0`、`tier=Constitution`、`scope_code=0`；
- 状态只允许 `Pending/Effective`，并严格校验 effective/latest/pending 指针组合；
- `houses` 与 block#0 一致，`LawsByScope[Constitution][0]` 必须恰为 `[0]`；
- manifest 在运行期与 block#0 编码逐字一致；
- 每个版本值内部 `law_id/version` 必须与 RAW key 一致；
- `content_hash=blake2_256(chapters.encode())`；
- 全文条号全局唯一，不允许用重复条号隐藏第二份条文；
- 第 1/2/3/17/19/24/34/42 条在所有受检版本中必须恰好存在且与 block#0 逐字一致；
- 核心章修改必须记录为特别案并存在过口径公投记录；所有修宪版本必须存在过 4/7 口径的护宪终审记录。

## 5. 导入形态

- 有 body：在父状态独立只读执行，提取后置 delta；
- `ApplyChanges(Changes)` 且无 body：直接检查预计算 delta，不能走快路径；
- `ApplyChanges(Import)`：从完整下载态抽取立法院 RAW storage，提交前全历史检查；
- `Execute/ExecuteIfPossible` 且无 body：无法独立证明后置状态，fail-closed；
- `Skip`：只允许不执行、不导入状态的语义；
- 触及立法院前缀或 `:code` 时全检，任何读取、执行或解码错误都返回 `KnownBad`。

普通块依据前态已经通过守卫的归纳条件，只复核当前有效状态和本块 delta 触及的历史版本；同时拒绝 `latest_version` 回退及超范围隐藏版本。warp 和启动没有前态归纳条件，因此枚举并复核全部版本。

## 6. 票数凭据的信任上限

`ConstitutionAmendmentProof` 当前只保存 `(eligible, yes, no)`，`ConstitutionGuardVoteProof` 只保存赞成票数。节点可以冻结凭据存在性、编码、历史不可删除和阈值口径，但这些记录不携带节点可独立验证的公民签名、人口快照证明或护宪成员签名集合。

因此：不可修改八条是 block#0 + 节点二进制保证的真正死规则；公投和护宪票数检查属于纵深一致性背书，不能宣称能独立对抗完全恶意 runtime 伪造计数。若要提升为密码学证明，需要另行设计签名承诺、成员更替证明和节点原生验签协议。

## 7. 第 2 步验收

- `constitution`：38/38 通过；
- `node_guard`：11/11 通过；
- node `cargo check`、`cargo fmt --check` 通过；
- 当前源码 fresh 创世真实启动成功；
- `chain_getBlockHash(0)` 与 `constitution_getDocument` RPC 成功；
- 旧单文件宪法实现和旧 HTML 路径残留为 0。

## 8. runtime 与 node 字段契约基线（2026-07-12）

ConstitutionGuard 固定读取 `LegislationYuan` 的 `Laws`、`LawVersions`、`LawVersionLabels`、
`LawsByScope`、`ConstitutionImmutableManifest`、`ConstitutionAmendmentProof` 和
`ConstitutionGuardVoteProof`。map/double-map key 均使用 `Blake2_128Concat`，不读取 runtime metadata。

`Law` 的 SCALE 顺序固定为：`law_id`、`tier`、`scope_code`、`houses`、`effective_version`、
`latest_version`、`pending_version`、`status`。守卫要求 `law_id=0`、`tier=Constitution(0)`、
`scope_code=0`，状态只允许 `Pending(0)` 或 `Effective(1)`，并冻结 block#0 houses 与合法版本指针组合。

`LawVersion` 的 SCALE 顺序固定为：`law_id`、`version`、`title`、`title_en`、`chapters`、
`content_hash`、`vote_type`、`proposal_id`、`published_at`、`effective_at`。runtime 创世测试用完整 tuple
编码钉死顺序；node 要求历史版本从 1 连续到 `latest_version`，不存在删除、改写或隐藏版本。

不可修改条款固定为第 `1/2/3/17/19/24/34/42` 条。核心章修改必须为 `Special(4)` 且公投凭据满足
参与率至少 70%、参与者赞成率至少 70%；所有修宪必须有护宪赞成数至少 4。任一相关 delta 或
`:code` 变化触发复核；畸形 key、缺值、解码失败和尾随字节均 fail-closed。

2026-07-12 对齐验收：runtime 的 enum 判别值、创世 LawVersion 字段序定向测试和
ConstitutionGuard `39/39` 全部通过。凭据仍是计数型状态，不是签名集合或人口快照的密码学证明。

## 9. 第 6.2 步恶意状态与拒绝矩阵

ConstitutionGuard 39 个定向测试已经覆盖 Law[0] 缺失/身份错误、tier/status/houses/指针异常、scope
唯一索引、manifest 篡改、不可修改条款删除或修改、条号重复、版本身份/内容哈希、历史版本删除或
改写、隐藏版本、核心章特别案、公投凭据、护宪凭据、SCALE 尾随字节、预计算 delta 和 `:code` 全检。

本步额外加固 `LawVersions` 与两类修宪凭据的 RAW key 解析：提取版本号前必须重算并比对
`Blake2_128Concat` 的 16 字节 hash，畸形 hasher 不再仅凭 key 长度和尾部 u32 被识别为规范版本。
对应测试同时覆盖历史版本 key 和护宪凭据 key 的 hasher 篡改。

外层 ConstitutionGuard 与 NodeGuard 共用无状态 `import_if_verified` 闸门：校验错误统一返回
`KnownBad`，不调用下一层；连续拒绝后合法输入仍可继续委派。完整 warp、真实数据库和三节点行为
不在本步结论内，继续由后续真实验收覆盖。

## 10. 第 6.3 步完整宪法状态校验

完整导入态只抽取 `LegislationYuan` RAW storage，并在提交前复用正常路径的不变式：Law[0] 身份与
指针、scope 唯一索引、manifest、全部真实历史版本、不可修改条款、内容哈希和修宪凭据。版本集合
必须严格连续，不依据不可信 `latest_version` 构造大循环。

本步新增完整态畸形 key 拒绝：任何以 `LawVersions[0]` 或两类宪法凭据 storage 前缀开头、但长度、
尾部编码或 Blake2_128Concat 不合法的 key，统一返回 `StorageKeyMalformed`，不能作为无关 key 忽略。
合法状态、不可修改条款篡改、缺关键 key、隐藏版本、历史版本篡改以及畸形版本/凭据 hasher 均有
提交前测试覆盖。2026-07-12 ConstitutionGuard `40/40` 通过；当前 fresh block#0 真实启动也通过
ConstitutionGuard 创世基准构造。
