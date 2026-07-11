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
