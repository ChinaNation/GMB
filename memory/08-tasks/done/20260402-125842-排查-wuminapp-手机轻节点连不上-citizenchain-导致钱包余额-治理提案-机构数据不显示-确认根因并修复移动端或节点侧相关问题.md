# 任务卡：排查 wuminapp 手机轻节点连不上 citizenchain，导致钱包余额、治理提案、机构数据不显示，确认根因并修复移动端或节点侧相关问题

- 任务编号：20260402-125842
- 状态：open
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-04-02 12:58:42

## 任务需求

排查 wuminapp 手机轻节点连不上 citizenchain，导致钱包余额、治理提案、机构数据不显示，确认根因并修复移动端或节点侧相关问题

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- wuminapp/WUMINAPP_TECHNICAL.md

## 模块模板

- 模板来源：memory/08-tasks/templates/wuminapp.md

### 默认改动范围

- `wuminapp`

### 先沟通条件

- 修改 Isar 数据结构
- 修改认证流程
- 修改关键交互路径


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/wuminapp.md

# WuMinApp 模块执行清单

- App 只是交互入口，不承担信任根职责
- Isar 结构、认证流程、关键交互变化前必须先沟通
- 关键 Flutter 交互与本地存储逻辑必须补中文注释
- 文档与残留必须一起收口


## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/wuminapp.md

# WuMinApp 完成标准

- App 仍然只是交互入口
- 关键 Flutter 交互和 Isar 逻辑已补中文注释
- 文档已同步更新
- 关键交互或数据结构变化已先沟通
- 残留已清理


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
- 已确认现网节点并未整体宕机：`147.224.14.117` 服务 active，`64.181.239.233` 返回 `peers=2`、`isSyncing=false`
- 2026-04-02 13:26 至 13:29 通过 USB 真机排查后，已推翻“问题不在 chainspec”这一早期判断
- 真机日志确认手机 smoldot 启动的链规格创世哈希是 `0xabe5…92cd`，而两台现网主节点 `147.224.14.117`、`64.181.239.233` 返回的 `chain_getBlockHash(0)` 都是 `0xea8fdcf52a9580381c5e8c38f91741b4fdc2ae787b5a646ffa19975cd056e4a9`
- 已确认手机端不是“没网”，而是“连到了节点传输层，但链定义不一致”：
  - 真机日志存在稳定 `pong`，`prczss.crcfrcn.com` 与 `nrcgch.crcfrcn.com` 的 `ping_time` 约 `41ms` 至 `89ms`
  - 同一批日志反复出现 `gossip-open-error ... ProtocolNotAvailable`
  - 同一批日志反复出现 `runtime-download-error ... StorageQueryError { errors: [] }`
  - 同一批日志反复出现 `discovery-skipped-no-peer; chain=citizenchain`
- 已从手机已安装 APK 解包 `assets/chainspec.json`，确认它不是仓库当前工作区版本，也不是 `HEAD` 版本：
  - 手机 APK 与仓库 `HEAD` 的 `bootNodes/protocolId/properties/lightSyncState` 一致
  - 但 `genesis.raw.top` 不一致，手机 APK 比 `HEAD` 多 1 个创世存储项，并且 `:code` 运行时代码不同
  - 手机 APK 的整份 `chainspec` SHA256 为 `27b933e1abbc579033eebf5c0907ac6a2ca925e7a730f20d44b90889e6f1231c`
  - 仓库 `HEAD` 中 `wuminapp/assets/chainspec.json` 的 SHA256 为 `4306f2e87e38367bab8243c5dad4576fb01adb92d09201e6d880f9e3f0f2746d`
- 已确认 `wuminapp/assets/chainspec.json` 确实通过 `pubspec.yaml` 直接打进 APK，问题源头在打包资源链定义，而不是运行时从外部拉取
- 额外发现：bootnode 列表里大量 `prc*.crcfrcn.com` 子域名在真机上会报 `failed to lookup address information: No address associated with hostname`，说明 bootnode 名单里还夹杂了大量当前不可解析节点，进一步压缩了可用 peer 范围
- 已确认根因集中在 App 侧：
  - `SmoldotClientManager.initialize()` 每次启动都清空 `smoldot_db_cache`，导致永远冷启动全量同步
  - typed capability 在 `!isReady` 时返回 `null / [] / {}`，上层会把真实链路故障误判为空余额、空提案、空机构数据
  - 部分治理页面把底层异常直接降级成空态或技术错误字符串，缺少统一的链不可用提示
- 已实施修复：
  - 恢复 smoldot finalized database 缓存，启动时优先使用 `databaseContent`
  - 缓存失效时自动清理并回退无缓存重连
  - 启动后立即后台预热同步，默认同步超时调为 3 分钟
  - 轻节点未初始化时，typed capability 改为直接抛错，不再返回空值伪装空数据
  - 钱包/治理/机构相关主页面改为显示统一的“轻节点不可用”提示
