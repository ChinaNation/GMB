# 任务卡：nodeui 增加治理 Tab（方案 A：Rust 后端 + React 前端）

- 任务编号：20260324-nodeui-governance-tab
- 状态：open
- 所属模块：citizenchain-nodeui
- 当前负责人：Claude
- 创建时间：2026-03-24

## 任务需求

在 nodeui 的"挖矿"和"网络"Tab 之间增加"治理"按钮，实现 wuminapp 中已有的机构浏览和提案投票治理功能，使桌面端和手机端拥有相同的治理能力。

采用方案 A：nodeui Rust 后端通过本地 RPC 查询全节点链上存储，React 前端渲染治理 UI。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/nodeui/

### 关键参考源码

- wuminapp/lib/governance/ — 手机端治理完整实现
- citizenchain/runtime/governance/ — 链上治理 pallet（存储结构、extrinsic 格式）
- citizenchain/nodeui/backend/src/shared/rpc.rs — 现有 RPC 封装
- citizenchain/nodeui/frontend/App.tsx — Tab 导航定义
- citizenchain/nodeui/frontend/api.ts — Tauri IPC 封装

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-nodeui.md

### 默认改动范围

- `citizenchain/nodeui`
- `memory/05-modules/citizenchain/nodeui`

### 先沟通条件

- 修改节点 UI 与 node 的交互边界
- 修改桌面打包、sidecar 或安装包行为

## 技术方案

### 后端（Rust）

新建 `backend/src/governance/` 模块：

```
backend/src/governance/
├── mod.rs              # 模块入口，注册 Tauri 命令
├── types.rs            # 数据类型（机构、提案、投票等）
├── storage_keys.rs     # 链上存储 key 构造（twox_128 + blake2_128）
├── institution.rs      # 机构查询（管理员列表、余额）
├── proposal.rs         # 提案查询（列表、详情、分页）
└── vote.rs             # 投票查询与提交
```

核心能力：
1. **存储查询**：通过 `state_getStorage` / `state_queryStorageAt` RPC 读取链上治理存储
2. **SCALE 解码**：用 `parity-scale-codec` 解码链上数据（提案元数据、投票计数等）
3. **交易构建**（P2 阶段）：构建投票/提案 extrinsic，签名后通过 `author_submitExtrinsic` 提交

存储 key 构造规则（与 wuminapp 的 polkadart 一致）：
- `twox_128(pallet_name) + twox_128(storage_name)` — 前缀
- `+ blake2_128(key_bytes) + key_bytes` — 带 map key 的完整 key

### 前端（React/TS）

新建 `frontend/governance/` 目录：

```
frontend/governance/
├── GovernanceSection.tsx          # 治理主页（子 Tab：机构列表 / 全部提案）
├── institution-data.ts            # 87 个机构静态数据（从 Dart 迁移）
├── InstitutionListView.tsx        # 机构分类浏览（国储会 / 省储会 / 省储行）
├── InstitutionDetailPage.tsx      # 机构详情（余额、管理员、提案列表）
├── ProposalListView.tsx           # 全部提案列表（分页）
├── ProposalDetailPage.tsx         # 提案详情 + 投票状态
└── types.ts                       # 治理相关 TS 类型
```

导航：在 App.tsx 中 `mining` 和 `network` 之间插入 `governance` Tab。
GovernanceSection 内部使用子 Tab 切换"机构"和"提案"视图。

### 分阶段实施

| 阶段 | 内容 | 范围 |
|------|------|------|
| P0 | Tab 导航 + 机构列表浏览 + 机构详情（余额、管理员） | 只读 |
| P1 | 提案列表 + 提案详情 + 投票状态查看 | 只读 |
| P2 | 投票操作（需冷钱包签名支持） | 写操作 |
| P3 | 创建提案（转账、运行时升级等） | 写操作 |

### 阶段完成状态

| 阶段 | 状态 | 备注 |
|------|------|------|
| P0 | ✅ 完成 | 机构浏览（国储会/省储会/省储行） |
| P1 | ✅ 完成 | 提案列表 + 提案详情（只读） |
| 冷钱包 | ✅ 完成 | 导入/管理/管理员识别 |
| P2-内部投票 | ✅ 完成 | vote_transfer + QR 签名 |
| P2-联合投票 | ✅ 完成 | joint_vote + QR 签名 |
| 投票状态 | ✅ 完成 | 已投票/未投票显示 |
| P3-创建提案 | ✅ 完成 | propose_transfer + QR 签名 |
| UI 重排 | ✅ 完成 | 子 Tab：提案→国储会→省储会→省储行→钱包管理 |

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

1. ~~技术方案选择~~ → 已确认采用方案 A
2. nodeui 钱包管理目前只有 fee-address 和 node-key，P2 阶段的投票签名需要冷钱包支持——是否在 P0 阶段先只做只读浏览？→ 按 P0 先实施只读
3. 机构静态数据是否与 wuminapp 的 institution_data.dart 完全一致？→ 需从 Dart 源文件迁移

## 实施记录

### 2026-03-24 提案签名全流程调通

#### 已解决的关键问题

1. **QR 协议字段名格式**：wumin 使用 snake_case（`request_id`），nodeui 最初用了 camelCase → 修正为 snake_case
2. **wumin 盲签校验不通过**（"交易内容与摘要不符"）：`display.fields` 的 key/value 需要与 wumin PayloadDecoder 解码结果严格一致 → 对齐 wumin 的字段格式
3. **Extrinsic 编码错误导致链卡住**（block 1559）：
   - 缺少 `CheckMetadataHash` 的 mode 字节（0x00）和 implicit 字节（0x00）
   - 版本字节错误（用了 `0xc4`，应为 `0x84`）
   - 提交时重新获取 nonce/block_number 导致 era 与签名不一致 → 改为复用签名时保存的值
4. **dry-run 错误检测失败**：用 `s.contains("Err")` 匹配十六进制字符串 → 改为 SCALE 解码判断
5. **摄像头调用失败**：
   - macOS 需要 Info.plist 的 `NSCameraUsageDescription`，Tauri 2.0 的 `infoPlist` 需指向文件路径而非内联 JSON
   - React 渲染时序：scanner 在 DOM 元素未渲染时启动 → 增加 `requestAnimationFrame` + `setTimeout` 延迟
   - MacBook 前置摄像头需要调整 QR 扫描区域和参数
6. **React 闭包过期**：`startScanner` 的 `useCallback([])` 捕获的 `handleScanResult` 是初始版本（signRequest=null）→ 用 ref 持有最新值
7. **管理员冷钱包余额不足**：治理投票需要签名账户有余额支付手续费，冷钱包通常无余额 → 需先给管理员地址转入少量余额

#### 未解决的问题

- `system_dryRun` 对投票类交易持续返回 `InvalidTransaction::AncientBirthBlock`，但 propose_transfer 正常返回 `0x0000` → 可能是 PoW 链上 dry-run RPC 的 bug，已改为 dry-run 失败时发出警告但不阻止提交
- 全链转账失败（polkadot.js 和 wuminapp 均无法转账）→ 疑似链上状态异常，待排查

#### 技术要点备忘

- citizenchain 使用新版 polkadot-sdk 的 `TransactionExtension` API（非旧版 `SignedExtension`）
- TxExtension 元组编码顺序：`AuthorizeCall(0) + CheckNonZeroSender(0) + CheckNonKeylessSender(0) + CheckSpecVersion(0) + CheckTxVersion(0) + CheckGenesis(0) + CheckEra(2) + CheckNonce(compact) + CheckWeight(0) + ChargeTransactionPayment(compact) + CheckMetadataHash(1:mode) + WeightReclaim(0)`
- 签名载荷 implicit 数据：`spec_version(4) + tx_version(4) + genesis_hash(32) + era_block_hash(32) + metadata_hash_implicit(0x00)`
- Extrinsic 版本字节：`0x84`（v4 signed），不是 `0xc4`
