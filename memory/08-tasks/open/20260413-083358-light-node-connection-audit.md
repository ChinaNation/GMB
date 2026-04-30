# 任务卡：轻节点连接稳定性逐行审计：检查轻节点为何总是连不上区块链，逐文件逐行审查连接链路代码并输出完整检查报告与改进方案

- 任务编号：20260413-083358
- 状态：open
- 所属模块：citizenchain
- 当前负责人：Codex
- 创建时间：2026-04-13 08:33:58

## 任务需求

轻节点连接稳定性逐行审计：检查轻节点为何总是连不上区块链，逐文件逐行审查连接链路代码并输出完整检查报告与改进方案

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- <补充该模块对应技术文档路径>

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已完成轻节点连接链路逐文件逐行审查，覆盖 `citizenchain/node`、`citizenchain/node/vendor`、`wuminapp/lib/rpc`、`wuminapp/assets`、`wuminapp/scripts`、`wuminapp/smoldot-pow` 的关键连接路径

## 审查范围

本次按“轻节点发现节点 -> 建链 -> 同步 -> 读链 -> 恢复”全链路逐行审查了以下文件：

- `citizenchain/node/src/chain_spec.rs`
- `citizenchain/node/src/service.rs`
- `citizenchain/node/src/rpc.rs`
- `citizenchain/node/src/node_runner.rs`
- `citizenchain/node/src/home/process/mod.rs`
- `citizenchain/node/src/settings/bootnodes-address/mod.rs`
- `citizenchain/node/frontend/settings/node-key/NodeKeySection.tsx`
- `citizenchain/node/vendor/src/import.rs`
- `citizenchain/node/vendor/src/authorities.rs`
- `citizenchain/node/vendor/src/finality_proof.rs`
- `citizenchain/node/vendor/src/warp_proof.rs`
- `wuminapp/lib/rpc/smoldot_client.dart`
- `wuminapp/lib/rpc/chain_rpc.dart`
- `wuminapp/lib/main.dart`
- `wuminapp/lib/rpc/chain_event_subscription.dart`
- `wuminapp/assets/chainspec.json`
- `wuminapp/assets/chainspec.json.sha256`
- `wuminapp/scripts/wuminapp-run.sh`
- `wuminapp/pubspec.yaml`
- `wuminapp/smoldot-pow/light-base/src/lib.rs`
- `wuminapp/smoldot-pow/light-base/src/platform/address_parse.rs`
- `wuminapp/smoldot-pow/lib/src/chain_spec.rs`
- `wuminapp/smoldot-pow/lib/src/chain_spec/light_sync_state.rs`
- `wuminapp/smoldot-pow/wasm-node/CHANGELOG.md`
- `memory/07-ai/chainspec-frozen.md`

## 线上核验结果

为避免把“代码问题”和“部署状态问题”混淆，已对在线节点做只读核验：

- 在线节点 `64.181.239.233` 与 `147.224.14.117` 的 `chain_getBlockHash(0)` 返回一致：
  `0x9341a792cde9e1b298b740c15e08409501701a7162faf2accb804156278942af`
- 在线节点 `sync_state_genSyncSpec` 返回的结果包含 `lightSyncState`
- 本地对 `wuminapp/assets/chainspec.json` 中 44 个 `/dns4/.../tcp/30333/wss` 引导域名做了解析校验，结果 `44/44` 可解析
- 抽查 `nrcgch.crcfrcn.com:30333`、`prczss.crcfrcn.com:30333`，TCP 端口可达

结论：当前问题不是“47 个引导节点里全都挂了”，也不是“DNS 全部失效”。真正问题在于轻节点起点配置、同步门槛和恢复路径设计错误，导致即使有可用引导节点，移动端仍然会频繁表现为“总是连不上链”。

## 结论概览

当前轻节点“总是连不上区块链”的主根因不是 bootnode 数量，而是下面这条失败链：

1. `wuminapp` 打包进去的 `assets/chainspec.json` 没有 `lightSyncState`
2. `smoldot` 只能从 genesis 冷启动，而不是从线上全节点给出的 finalized checkpoint 起步
3. App 侧又把几乎所有链读操作都强制前置 `waitUntilSynced(timeout: 3m)`
4. 一旦 3 分钟内没有完成冷同步，App 就把状态标成 `degraded`，业务层统一报“链不可用/同步失败”
5. 只有“完整同步成功”之后才会导出 finalized database；第一次冷启动一旦失败，后续每次重启仍然回到纯 bootnode 冷启动，无法逐步摆脱对 bootnode 的强依赖

这就是为什么“哪怕明明有在线引导节点，轻节点还是总表现得像连不上链”。

## 发现的问题

### P0-1：移动端内置 chainspec 缺少 `lightSyncState`，轻节点每次都从 genesis 冷启动

证据：

- `wuminapp/lib/rpc/smoldot_client.dart:209-223`
  App 启动时固定从 `assets/chainspec.json` 读取链规格
- `wuminapp/assets/chainspec.json`
  当前文件顶层只有 `bootNodes / chainType / codeSubstitutes / genesis / id / name / properties / protocolId / telemetryEndpoints`，不存在 `lightSyncState`
- `wuminapp/smoldot-pow/light-base/src/lib.rs:1189-1227`
  smoldot 明确优先使用 chain spec 里的 `light_sync_state()` 作为 checkpoint；没有 checkpoint 就退回到 genesis
- `citizenchain/node/src/rpc.rs:299-304`
  在线全节点已经能通过 `sync_state_genSyncSpec` 产出 `lightSyncState`

影响：

- 轻节点失去最快、最稳定的启动锚点
- 首次冷启动完全依赖 bootnode 发现 + 从零同步
- 同步时间大幅拉长，极易撞上 App 层 3 分钟超时门槛

判断：

- 这是当前“轻节点看起来总连不上链”的头号根因
- 这也是为什么用户强调“不该纠结 47 个还是 2 个 bootnode”是对的：只要有一个可用 bootnode，正确的 checkpoint + cache 设计本应足以把轻节点带起来

### P0-2：`chainspec` 冻结规则把 `lightSyncState` 一起冻死了，导致正确 checkpoint 无法进入 App

证据：

- `wuminapp/scripts/wuminapp-run.sh:32-57`
  启动脚本把 `jq -cS 'del(.bootNodes)'` 后的整个 `chainspec.json` 做 sha256 完整性校验
- `memory/07-ai/chainspec-frozen.md:47-56`
  当前流程同样只排除 `bootNodes`，其余字段全部视为“创世冻结”
- `memory/07-ai/chainspec-frozen.md:75-80`
  文档把“轻节点升级 = chainspec 不动”当成绝对规则

影响：

- `genesis/raw` 的确必须冻结，这个原则本身没错
- 但 `lightSyncState` 不是 genesis，也不参与创世哈希；把它也冻结，等于禁止移动端接收新的 finalized checkpoint
- 结果就是：线上全节点已经能吐出新的 `sync_state_genSyncSpec`，但 App 资产层永远吃不到

判断：

- 这是一个流程级设计错误，不是单纯实现遗漏
- 如果不先拆分“创世冻结字段”和“可刷新网络字段”，后面无论再加多少 bootnode，都会继续被这个错误设计拖住

### P1-1：App 侧把几乎所有链读能力都绑定到“3 分钟内完整同步成功”这个前提，导致业务层统一感知为“连不上链”

证据：

- `wuminapp/lib/rpc/smoldot_client.dart:53`
  默认同步超时固定为 `Duration(minutes: 3)`
- `wuminapp/lib/rpc/smoldot_client.dart:427-465`
  `ensureSynced()` 内部直接调用 `_chain!.waitUntilSynced(timeout: timeout)`，超时就标记 `degraded`
- `wuminapp/lib/rpc/smoldot_client.dart:477-560`
  `getStatusSnapshot / getRuntimeVersion / getMetadata / getAccountNextIndex / getBlockHash / getStorageValue...` 全部先 `await ensureSynced()`
- `wuminapp/lib/rpc/chain_rpc.dart:31-190`
  业务层余额、metadata、创世哈希、最新块、nonce 等路径全部建立在上述调用之上

影响：

- 只要 checkpoint 缺失，3 分钟内没完整同步完，业务层就不是“慢一点”，而是直接整体报错
- 这会把“同步慢”错误地放大成“区块链不可用”
- 连诊断类状态读取也被完整同步前置，进一步加重“看起来像完全断链”的体验

判断：

- 这不是底层 p2p 真正完全断开，而是 App 侧把“尚未完成完整同步”过度解释成“不可用”
- 这会放大任何冷启动抖动

### P1-2：finalized database 只在“完整同步成功后”才保存，失败启动永远无法积累已知节点，bootnode 依赖被锁死

证据：

- `wuminapp/lib/rpc/smoldot_client.dart:224-241`
  只有启动时存在旧缓存才会走 `databaseContent` 恢复
- `wuminapp/lib/rpc/smoldot_client.dart:349-365`
  finalized database 的导出逻辑存在，但只是一个保存函数
- `wuminapp/lib/rpc/smoldot_client.dart:452-459`
  该保存逻辑只在 `waitUntilSynced()` 成功之后才异步调用
- `wuminapp/smoldot-pow/wasm-node/CHANGELOG.md:934`
  smoldot 明确指出 database 中包含“已知网络节点”，恢复后会立刻发现这些节点，从而降低对 bootnodes 的依赖

影响：

- 第一次启动如果因为没有 checkpoint 而在 3 分钟内超时，就拿不到任何缓存
- 下次启动仍然是“纯 bootnode 冷启动”
- 轻节点永远无法进入“已经记住网络拓扑，因此一个在线引导节点就够了”的良性状态

判断：

- 这是“为什么一个在线 bootnode 没有发挥应有稳定性”的第二个关键根因
- 当前设计把 bootnode 依赖固化了，而不是逐步消除它

### P2-1：桌面端“引导节点”设置其实是本机节点身份绑定，不是远端 bootnode 管理，容易把排障方向带偏

证据：

- `citizenchain/node/src/settings/bootnodes-address/mod.rs:25-40`
  `BootnodeKey` / `GenesisBootnodeOption` 都围绕 PeerId 与机构映射
- `citizenchain/node/src/settings/bootnodes-address/mod.rs:142-159`
  实际写入的是本机 `node-key/secret_ed25519`
- `citizenchain/node/src/settings/bootnodes-address/mod.rs:223-243`
  校验的是该私钥推导出的本机 PeerId 是否属于创世 bootnode
- `citizenchain/node/frontend/settings/node-key/NodeKeySection.tsx:52-80`
  前端文案写的是“区块链引导节点”“上传私钥”，极易被理解为“配置远端引导节点地址”

影响：

- 运维或排障人员容易误以为这里能管理“轻节点应该连接哪些远端 bootnodes”
- 实际上它只是在把本机节点伪装/绑定为某个创世节点身份
- 对“移动端轻节点为什么连不上链”没有直接帮助

判断：

- 这不是当前连链失败的主根因
- 但它会严重干扰定位，建议尽快改名和改说明

### P2-2：`sync_state_genSyncSpec` 的 checkpoint 由自定义逻辑手工拼装，当前能用，但实现偏脆弱

证据：

- `citizenchain/node/src/rpc.rs:244-304`
  `grandpaAuthoritySet` 不是调用现成导出器，而是手工拼接 `Grandpa::Authorities + set_id + 4 个空字段`
- `wuminapp/smoldot-pow/lib/src/chain_spec/light_sync_state.rs:318-360`
  smoldot 解析器要求 authority set 的 SCALE 结构严格匹配

影响：

- 当前线上节点吐出的 `lightSyncState` 已能被读取，说明现阶段格式至少可工作
- 但这段实现缺少“用 smoldot-pow 真实解码验证”的自动测试，未来一旦 checkpoint 结构、编码细节或 GRANDPA 状态来源变化，很容易再出兼容性事故

判断：

- 这是中风险脆弱点，不是当前主根因
- 应补回自动兼容性测试，而不是继续靠人工拼 hex

## 根因排序

按对“轻节点总是连不上链”贡献度排序如下：

1. `wuminapp/assets/chainspec.json` 没有 `lightSyncState`
2. `chainspec` 冻结规则错误地把 `lightSyncState` 一起冻结，导致这个问题无法自然修复
3. App 把所有链读能力强绑定到 `waitUntilSynced(timeout: 3m)`
4. finalized database 只在完整同步成功后才保存，导致每次失败启动都回到纯 bootnode 冷启动
5. 文案和配置模型把“本机节点身份”与“远端 bootnode 配置”混在一起，干扰排障

## 为什么“一个在线 bootnode 也应该够”在这里没有实现

理论上，用户的要求是对的：

- 一个在线 bootnode 负责把轻节点引入网络
- 之后 checkpoint 帮它从正确的 finalized 高度起步
- restored database / known nodes 帮它快速发现更多 peers
- 之后即使最初那个 bootnode 下线，轻节点也不该轻易掉回“完全失联”

但当前实现恰好把这四步中的后三步都削弱了：

- 没有打包 `lightSyncState`
- 没有允许 `lightSyncState` 刷新
- 没有在失败冷启动后尽快沉淀 finalized database / known nodes

所以系统退化成了“每次都像第一次见网”，自然就会表现为强依赖 bootnode 数量和偶然性。

## 改进方案

### 方案 A：拆分“创世冻结”和“网络可刷新”两层链规格

目标：

- 保持 genesis/raw 永久冻结
- 允许 `bootNodes` 与 `lightSyncState` 单独刷新

建议做法：

1. 保留一个冻结文件，只存真正影响 genesis hash 的字段：
   `name / id / chainType / genesis / properties / codeSubstitutes / forkId`
2. 新增一个可刷新 overlay，只存：
   `bootNodes / lightSyncState / telemetryEndpoints`
3. App 启动时在内存中合并两份 JSON，再交给 smoldot
4. `sha256` 校验拆成两套：
   - genesis 冻结校验：只覆盖 genesis 相关字段
   - overlay 新鲜度校验：只覆盖 overlay 本身

收益：

- 不破坏主网创世冻结原则
- 允许在不改 genesis 的前提下持续刷新 checkpoint
- 后续 bootnode 增删和 checkpoint 轮换都不需要和“硬分叉”绑死

### 方案 B：在发布/CI 阶段自动从在线全节点刷新 `lightSyncState`

目标：

- 每次打包 App 时，都内置一个新鲜 checkpoint

建议做法：

1. CI 或发布脚本调用可信全节点的 `sync_state_genSyncSpec`
2. 校验远端 `chain_getBlockHash(0)` 必须等于冻结创世哈希
3. 只抽取并更新 overlay 中的 `lightSyncState`
4. 失败即阻断发版，而不是默默沿用旧 checkpoint

收益：

- 新装 App 不再从 genesis 爬起
- 冷启动成功率和速度会立刻提升

### 方案 C：去掉“所有链读都必须完整同步成功”的硬门槛

目标：

- 让 App 区分“正在同步”“暂无 peer”“runtime/proof 暂不可读”“彻底初始化失败”

建议做法：

1. `getPeerCount()`、本地创世哈希、当前状态快照、finalized 头等不再强制 `ensureSynced()`
2. 只对确实依赖 runtime proof 的读操作做能力门控
3. UI 明确展示四种状态：
   - 已初始化但未发现 peer
   - 已发现 peer，正在同步
   - 已有 finalized/runtime，可读
   - 彻底失败
4. 不再把“3 分钟未完整同步”直接翻译成“区块链暂不可用”

收益：

- 用户不会再把“同步慢”误判为“完全连不上”
- 诊断信息更真实

### 方案 D：更早、更稳定地沉淀 finalized database，快速摆脱 bootnode 依赖

目标：

- 让轻节点尽快学会网络，不再每次重启都回到纯 bootnode 发现

建议做法：

1. 除了“完整同步成功后保存”，再增加定时或阶段性保存 finalized database
2. App 退后台/退出前也尝试导出一次
3. 对 `maxSizeBytes` 做自适应放大，不要固定死在 256 KB
4. 缓存按 genesis hash 分桶，避免错链污染

收益：

- 只要成功连上过一次，后续越来越不依赖 bootnode 数量
- 更接近“一个在线引导节点就足够”的目标

### 方案 E：补齐自动回归测试，防止 checkpoint 再次失配

目标：

- 让 `sync_state_genSyncSpec` 输出和 smoldot 输入之间的兼容性可以自动验证

建议做法：

1. 新增测试：用 `citizenchain/node/src/rpc.rs` 产出的 `lightSyncState` 喂给 `wuminapp/smoldot-pow` 的 `ChainSpec::from_json_bytes`
2. 新增测试：冻结 genesis 文件与线上 `chain_getBlockHash(0)` 必须一致
3. 新增测试：若 overlay 缺失 `lightSyncState`，发版流程直接失败

收益：

- 避免以后再次出现“线上节点能产出 checkpoint，但 App 资产没带进去”的问题
- 避免手工拼装 checkpoint 的隐性兼容性回退

### 方案 F：修正文案与模型，避免继续误导 bootnode 排障

目标：

- 把“本机节点身份绑定”和“远端 bootnode 列表”彻底区分开

建议做法：

1. 桌面端把“区块链引导节点”改名为“创世节点身份绑定”或“本机 PeerId 身份绑定”
2. 单独提供远端 bootnode 观测页或配置页
3. 在日志和 UI 中明确区分：
   - 本机节点身份
   - 远端 bootnode 列表
   - 当前已连接 peers

收益：

- 运维排障不再被错误入口带偏

## 优先级建议

建议按下面顺序执行：

1. 先改流程：拆分冻结字段与 overlay，允许 `lightSyncState` 刷新
2. 再改移动端：去掉全量 `ensureSynced(3m)` 的硬前置，至少让状态读和 peer 观测先活起来
3. 再改恢复：更早保存 finalized database，让轻节点逐步摆脱 bootnode 依赖
4. 最后补测试和文案修正

## 最终判断

本次审查结论非常明确：

- 当前问题不是“引导节点数量不够”
- 当前问题也不是“域名全坏了”或“线上链本身不通”
- 真正根因是：**移动端没有携带可刷新的 `lightSyncState`，同时又把业务链路强绑定到完整同步成功，再加上缓存沉淀时机过晚，导致轻节点每次都像第一次上网，无法把一个在线引导节点的价值放大成稳定连接能力**

如果只继续加 bootnode，不改上面这三处设计，问题还会反复出现。

## 当前产出

- 已完成代码审查与线上核验
- 已形成完整根因链与改进方案
- 本轮未改源码，仅输出检查报告

## 第二轮评审补充（2026-04-13）

本轮针对 `CP-1 ~ CP-5` 的实际代码改动做了二次评审，结论如下。

### 可行部分

- `CP-2` 方向正确：冻结校验从“只排除 `bootNodes`”调整为“排除 `bootNodes` + `lightSyncState`”，符合“冻结 genesis、放开网络层与 checkpoint 层”的原则
- `CP-1` 方向基本可行：把 `lightSyncState` 独立成 `assets/light_sync_state.json`，运行时注入内存版 chainspec，规避了线上 RPC 端口不可公网访问的问题
- `CP-3/CP-4` 有正向价值：同步超时后继续保存 database，并让 smoldot 在后台继续追赶，比原来“一次超时就直接当成不可用”更合理
- `CP-5` 文案修正正确：把“引导节点私钥”改回“节点身份密钥”，减少排障误导

### 当前仍阻塞上线的问题

1. `scripts/update-light-sync-state.sh` 还没有做“错误链 checkpoint”防护  
   现状：脚本只调用 `sync_state_genSyncSpec` 并把返回里的 `lightSyncState` 直接写入 `wuminapp/assets/light_sync_state.json`，没有校验来源节点的 genesis hash 是否与 `wuminapp/assets/chainspec.json` 一致。  
   风险：一旦连错节点、连到测试链、或误指向另一套环境，就会把“结构合法但属于别的链”的 checkpoint 打进发布包。  
   代码位置：`scripts/update-light-sync-state.sh:21-35`

2. `CP-4` 现在会重复堆叠后台重试任务，缺少单实例防抖  
   现状：`_waitForSync()` 每次超时都会 `unawaited(_scheduleRetrySync())`，但没有 `_retrySyncScheduled` / `Timer` / `CancelableOperation` 之类的守卫。与此同时，超时分支又把 `_syncFuture = null`，后续新的读请求还会重新触发 `ensureSynced()`。  
   风险：在弱网或长时间追块场景下，可能同时堆出多组“60 秒 × 5 次”的后台轮询，重复打 `waitUntilSynced()` 和 `finalizedDatabase` 导出，造成无意义的额外压力。  
   代码位置：`wuminapp/lib/rpc/smoldot_client.dart:492-503`、`wuminapp/lib/rpc/smoldot_client.dart:511-535`

3. 原始架构问题还没有真正解除：多数链读依然被 `ensureSynced()` 硬前置  
   现状：虽然超时状态从 `degraded` 改成了 `syncing`，但 `request()` 默认仍要求 `requireSynced = true`，而 `getStatusSnapshot()`、`getRuntimeVersionJson()`、`getMetadataHex()`、`getBlockHash()`、`getBlockExtrinsics()` 等公开读接口仍全部先 `await ensureSynced()`。  
   风险：这意味着“同步慢”仍然会被放大成“读链失败”，只是错误文案更温和了；并没有真正实现“哪怕还在追块，也先把可读状态和诊断能力放出来”。  
   代码位置：`wuminapp/lib/rpc/smoldot_client.dart:410-418`、`wuminapp/lib/rpc/smoldot_client.dart:459-478`、`wuminapp/lib/rpc/smoldot_client.dart:543-629`

### 次级问题

- `scripts/check-chainspec-frozen.sh` 的成功提示文案还写着“bootNodes 变更不受限”，但代码实际已经同时放开 `lightSyncState`；文案应同步修正，避免误导  
  代码位置：`scripts/check-chainspec-frozen.sh:24-27`、`scripts/check-chainspec-frozen.sh:48`
- `scripts/update-light-sync-state.sh` 与 App 注入逻辑只做了最小字段检查，建议至少同时校验 `finalizedBlockHeader` 与 `grandpaAuthoritySet` 都存在，再允许写入/注入  
  代码位置：`scripts/update-light-sync-state.sh:27-35`、`wuminapp/lib/rpc/smoldot_client.dart:340-352`

### 评审结论

- `CP-2` 可以保留，方向没有问题
- `CP-1` 在补上“genesis hash 一致性校验”前，不建议直接作为发布流程定版
- `CP-3/CP-4` 有帮助，但当前只是缓解，不是根治；要真正解决“轻节点总像没连上”，还得继续拆掉“所有链读必须先完整同步成功”这层硬门槛
- 因此，这组修复目前属于“方向对，但还不能判定为可安全上线的最终版本”

## 第三轮复核（2026-04-13）

本轮对后续补丁再次复核，结论按问题编号如下：

- `P0`：已完成  
  `scripts/update-light-sync-state.sh` 已在写入 checkpoint 前先调用 `chain_getBlockHash(0)`，并与固定 genesis hash 比对；不匹配会直接退出。  
  代码位置：`scripts/update-light-sync-state.sh:27-42`

- `P1-1`：已完成  
  `wuminapp/lib/rpc/smoldot_client.dart` 已新增 `_retrySyncRunning` 守卫，`_scheduleRetrySync()` 入口会去重，并在 `finally` 中复位，解决了弱网下后台重试堆叠的问题。  
  代码位置：`wuminapp/lib/rpc/smoldot_client.dart:42`、`wuminapp/lib/rpc/smoldot_client.dart:525-554`

- `P1-2`：未完成，仅部分缓解  
  当前改动只是把“后台重试期间再次调用 `ensureSynced()`”从原先可能重复触发 3 分钟阻塞，改成了最多短等 30 秒后报“同步中”。  
  但所有公开链读接口依然普遍先 `await ensureSynced()`，因此“完整同步成功前禁止多数链读”这一架构硬门槛并没有真正拆掉。  
  代码位置：`wuminapp/lib/rpc/smoldot_client.dart:463-475`、`wuminapp/lib/rpc/smoldot_client.dart:563-620`

- `P2`：已完成  
  `scripts/check-chainspec-frozen.sh` 的成功提示已更新为同时放开 `bootNodes / lightSyncState`。  
  代码位置：`scripts/check-chainspec-frozen.sh:24-26`、`scripts/check-chainspec-frozen.sh:48`

### 当前结论

- 可以确认已完成：`P0`、`P1-1`、`P2`
- 不能确认已完成：`P1-2`
- 因此，若按“这 4 个后续问题是否全部修完”来判断，答案是：**没有全部完成，还差 `P1-2` 的架构层收尾**

## 第四轮复核（2026-04-13）

本轮针对 `P1-2` 再次复核，结论更新如下：

- 原 finding 已基本修复，不再按原结论成立  
  当前 `smoldot_client.dart` 已明确拆分出“基础读取（不要求完整同步）”与“最新状态读取（必须完整同步）”两层接口：
  - 不再强制 `ensureSynced()`：`getRuntimeVersionJson()`、`getMetadataHex()`、`getBlockHash()`
  - 仍要求 `ensureSynced()`：`getStatusSnapshot()`、`getAccountNextIndex()`、`getBlockExtrinsics()`、`getSystemAccountSnapshot()`、`getStorageValueHex()`、`getStorageValuesHex()`、`submitExtrinsicHex()`
  代码位置：`wuminapp/lib/rpc/smoldot_client.dart:563-660`

- 业务调用链已开始使用这条“同步中可读”通道  
  `ChainRpc.fetchRuntimeVersion()`、`fetchGenesisHash()`、`fetchMetadata()` 分别走 `getRuntimeVersionJson()`、`getBlockHash(0)`、`getMetadataHex()`，不再被完整同步硬阻塞。  
  代码位置：`wuminapp/lib/rpc/chain_rpc.dart:57-77`、`wuminapp/lib/rpc/chain_rpc.dart:127-137`

- 残余说明  
  `getStatusSnapshot()` 仍要求完整同步，因此如果目标是“同步中也要实时展示 best/finalized/peer 状态”，还可以继续拆出一个 raw status 接口；但这已经不属于原 finding 的范围。原 finding 关注的是“完全没有同步中可读通道”，而这点现在已经被修正。

### 更新后的判断

- `P1-2`：按原 finding 定义，现可判定为**已修复**
- 仍可继续优化，但不再应以“原 finding 仍成立”来阻塞合并

## 第五轮实现记录（2026-04-13）

为补齐“同步过程中也能实时查看 peer / best / finalized / syncing 状态”的能力，本轮已直接落地以下代码：

- 在 `wuminapp/lib/rpc/smoldot_client.dart` 新增 `getStatusSnapshotRaw()`  
  特点：
  - 不走 `ensureSynced()`
  - 不等待 peer，因为 `peerCount=0` 本身就是诊断信息
  - 仍复用 `_withRetry()` 统一处理瞬断

- `printDiagnostics()` 已切到 `getStatusSnapshotRaw()`  
  避免诊断日志本身再被“完整同步”硬门槛卡住

- 在 `wuminapp/lib/rpc/chain_rpc.dart` 新增 `fetchChainProgress()`  
  用作业务层/UI 层读取“同步中原始进度”的标准入口

### 当前边界

- 已完成：底层 raw 状态接口 + 业务层封装入口
- 未强制修改：现有页面展示逻辑  
  也就是说，页面如果要展示同步进度，还需要后续明确接到 `fetchChainProgress()`；但接口本身已经具备，不再需要继续改轻节点层

## 第六轮实现记录（2026-04-13）

已把 raw 状态接口接入钱包页：

- `wuminapp/lib/wallet/ui/wallet_page.dart`
  - 新增 `_chainProgress` / `_chainProgressError` 本地状态
  - 在 `_refreshBalancesFromChain()` 开始阶段调用 `_refreshChainProgress()`
  - 新增 `_buildChainProgressBanner()`，在钱包列表顶部展示：
    - `peer`
    - `best`
    - `finalized`
    - 当前处于“连接网络 / 同步区块头 / 已就绪 / 状态读取失败”哪一类
  - `printDiagnostics()` 仍保留用于日志，但页面已不再只靠日志排障

### 结果

- 用户打开钱包页时，可以直接看到轻节点当前是在：
  - 无 peer
  - 正在同步
  - 已就绪
  - 状态读取失败
- 这一步不改变交易、余额、nonce、storage 的严格同步语义，只补 UI 观测性

## 第七轮实现记录（2026-04-13）

为避免钱包页、交易页、治理页各自维护一套状态条，本轮把链路状态卡抽成复用组件：

- 新增：`wuminapp/lib/ui/widgets/chain_progress_banner.dart`
  - 内部调用 `ChainRpc.fetchChainProgress()`
  - 默认 6 秒轮询一次，直到：
    - 已有 peer 且不再 syncing
    - 且没有错误
  - 展示统一的 `peer / best / finalized / syncing` 信息

- 已接入页面：
  - `wuminapp/lib/wallet/ui/wallet_page.dart`
  - `wuminapp/lib/trade/onchain/onchain_trade_page.dart`
  - `wuminapp/lib/governance/proposal_types_page.dart`

### 用户侧效果

- 钱包页：除余额刷新外，还能直接看链路进度
- 交易页：进入页面即可知道当前是“没 peer / 正在追块 / 已就绪”，避免用户在无法发交易时只看到失败结果
- 发起提案页：进入前就能看到链状态，避免提案动作触发后才暴露同步问题

### 校验结果

- `dart analyze` 无 error / warning
- 仅剩若干历史遗留 `info`，本轮新接入页面未新增阻塞问题

## 第八轮实现记录（2026-04-13）

本轮把“入口页可见状态”继续下沉到真正的提案提交页，避免用户已经进入页面后，链路断开或仍在同步时还能继续走签名/提交流程：

- 已接入页面：
  - `wuminapp/lib/governance/transfer_proposal_page.dart`
  - `wuminapp/lib/governance/runtime_upgrade_page.dart`
  - `wuminapp/lib/duoqian/institution/institution_duoqian_create_page.dart`
  - `wuminapp/lib/duoqian/institution/institution_duoqian_close_page.dart`
  - `wuminapp/lib/duoqian/personal/personal_duoqian_close_page.dart`

- 统一改动：
  - 页面顶部接入 `ChainProgressBanner`
  - 新增 `_chainProgress / _chainProgressError` 本地状态
  - 新增 `_submitBlockedReason`
  - 提交入口在链未连上、仍在同步、状态不可用时直接禁用
  - 用户点击提交时若链未就绪，会立即收到明确原因，而不是进入签名后再失败

### 用户侧效果

- 同步中仍可进入页面并填写表单
- 但真正的链上提交会被前置拦截，避免无意义签名和失败回滚
- 页面会明确告诉用户当前卡在：
  - 没有 peer
  - 正在同步区块头
  - 区块链状态尚未就绪

### 校验结果

- 针对以下文件执行 `dart analyze`：
  - `chain_progress_banner.dart`
  - `transfer_proposal_page.dart`
  - `runtime_upgrade_page.dart`
  - `institution_duoqian_create_page.dart`
  - `institution_duoqian_close_page.dart`
  - `personal_duoqian_close_page.dart`
  - `proposal_types_page.dart`
  - `onchain_trade_page.dart`
  - `wallet_page.dart`
  - `chain_rpc.dart`
  - `smoldot_client.dart`
- 结果：无 `error` / `warning`
- 仍有若干仓库既有 `info` 级 lint，包括部分 `use_build_context_synchronously`、`prefer_const_constructors`、`deprecated_member_use`
