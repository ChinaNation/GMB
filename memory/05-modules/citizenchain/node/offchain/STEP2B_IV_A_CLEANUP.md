# 清算行节点 substrate 集成 · Step 2b-iv-a 节点侧省储行代码清理

- **日期**:2026-04-20
- **范围**:删除 ADR-006 宣布"退出清算"的省储行链下清算在**节点端**的全部代码
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2B_III_B_RESERVE_MONITOR.md`(清算行节点闭环 + 保底监控完整落地)
- **后续**:`STEP2B_IV_B_RUNTIME_CLEANUP.md`(runtime 老 Calls + Storage 清理,需 spec_version bump + setCode)

---

## 1. 本步目标

Step 2b-iii-b 完成后,新清算行节点(wuminapp RPC → ledger → packer → pool → runtime → listener → reserve_monitor)已完整闭环并在共存模式下可运行。**本步**把旧省储行清算在节点端的 3 个源文件、`service.rs` 的全局配置 + gossip worker、`rpc.rs` 的 3 个 RPC 端点和 2 个 helper、`ui/home/process/mod.rs` 的老 ledger 切入,全部删除。

**明确不做**(Step 2b-iv-b / Step 2c):
- Runtime 侧老 Calls(call_index 0 `submit_offchain_batch` / 1+2 `propose_institution_rate` / 9 `bind_clearing_institution`)、`RecipientClearingInstitution` / `InstitutionRateBp` / `QueuedBatches` Storage(Step 2b-iv-b)
- wuminapp 侧 `offchain.dart` RPC 客户端 + `offchain_pay_page.dart` 页面(Step 2c 重写时一并删除)

---

## 2. 改动清单

### 2.1 文件删除(3 个)

| 文件 | 行数 | 职责(已退出) |
|---|---:|---|
| `node/src/offchain_ledger.rs` | 351 | 老省储行本地账本 + virtual_balance |
| `node/src/offchain_packer.rs` | 221 | 老批次打包 + 多签 + 上链 |
| `node/src/offchain_gossip.rs` | 164 | 老 libp2p 协议 `/gmb/offchain/1` |
| **合计** | **736** | |

### 2.2 `offchain_keystore.rs`(保留 + 头注释净化)

- 头注释改为"节点签名管理员 sr25519 私钥加密存储模块",并添加 Step 2b-iv-a 历史说明
- 明确 `SigningKey.shenfen_id` 字段当前语义是"清算行管理员身份标识",字段 rename 留 Step 3
- 代码无修改

### 2.3 `main.rs`

- 删除 `mod offchain_gossip;` / `mod offchain_ledger;` / `mod offchain_packer;` 3 个声明
- 顶部注释改为"清理删除旧 offchain_{ledger,packer,gossip}.rs,keystore 保留作统一密钥容器"

### 2.4 `service.rs`

| 删除 | 行号(清理前) |
|---|---|
| `static OFFCHAIN_CONFIG` | L43 |
| `pub struct OffchainConfig` + `ledger` + `shenfen_id` 字段 | L46-49 |
| `pub fn set_offchain_config` | L52-54 |
| `fn get_offchain_config` | L57-59 |
| 老 `offchain_clearing_notification_service` 协议注册 | L483-491 |
| 老 `offchain_gossip_tx/rx` channel 创建 | L555-561 |
| `rpc_extensions_builder` 闭包 `offchain_ledger` / `offchain_shenfen_id` / `offchain_gossip_tx` 三字段赋值 | L693-696 |
| 老 `offchain-clearing-gossip` spawn 块 | L712-725 |

### 2.5 `rpc.rs`

从 **800 行 → 422 行**(删 378 行):

| 删除 | 说明 |
|---|---|
| `use crate::offchain_ledger::*;` | 顶部 import |
| `FullDeps::{offchain_ledger, offchain_shenfen_id, offchain_gossip_tx}` | 3 个字段 |
| `create_full` destructure 三字段 | 跟随字段删除 |
| `offchain_submitSignedTx` RPC 注册块 | ~210 行(签名验证 + 虚拟余额 + ledger 入账 + gossip 推送) |
| `offchain_queryTxStatus` RPC 注册块 | ~65 行(三级状态查询) |
| `offchain_queryInstitutionRate` RPC 注册块 | ~15 行(链上费率查询) |
| `fn query_institution_rate_bp` | ~44 行 helper |
| `fn calc_offchain_fee` | ~10 行 helper |
| `use std::time::{SystemTime, UNIX_EPOCH}` / `sp_core::{sr25519, Pair, H256}` / `sp_keystore::Keystore` | 跟随删除的未用 import |

保留:
- `parse_ss58_account` — 仍被 `reward_bindWallet` / `reward_rebindWallet` 调用
- `sync_state_genSyncSpec` / `mining_cpuHashrate` / `mining_gpuHashrate` / `reward_*` / `fee_blockFees` RPC
- 新清算行 `offchain_clearing_rpc` 字段与 `OffchainClearingRpcServer::into_rpc` 合并

### 2.6 `ui/home/process/mod.rs`

- 删除 `RuntimeState::offchain_ledger` 字段及 `Default` 初始化
- 删除 `start_node_sync` 里检测签名管理员 → 加载老 ledger → `set_offchain_config` 整块(~42 行)
- 删除 `stop_node_sync` 里 `pending_count > 0 拒绝停止` 保护(~16 行)

**保护失效的处置**:UI 模式下 `node_runner.rs` 向 `new_full` 传 `None, None, None`,不启动清算行组件,所以此保护对 UI 运行时一直是死代码。CLI 清算行模式下的 graceful shutdown + pending 检查需要 `task_manager.spawn_essential_handle` 生命周期绑定,留 Step 3 独立任务实现。

### 2.7 `offchain/mod.rs` + `offchain/ledger.rs` 头注释净化

- `offchain/mod.rs`:头注释从"Step 1 骨架,Step 2 接入"改为描述当前实际运行状态(对应 `--clearing-bank` CLI 触发的三个 spawn task)。删除 `#![allow(dead_code)]`(本步不删,仍有些 `#[allow(dead_code)]` 细粒度项,全局 allow 待确认可删再处理)
- `offchain/ledger.rs`:删除指向旧 `offchain_ledger.rs` 的"保持一致"引用

### 2.8 `wuminapp/lib/trade/onchain/onchain_trade_page.dart`

- 删除 `import 'package:wuminapp_mobile/trade/offchain/offchain_pay_page.dart';`
- `_openOffchainPay()` 改为扫描后仅 SnackBar 提示"扫码支付正在切换新清算行体系,Step 2c 发布后恢复",不再跳转到老页面
- 运行时:老路径的 3 个 RPC 调用(`offchain_submitSignedTx` / `offchain_queryTxStatus` / `offchain_queryInstitutionRate`)再无触发
- `flutter analyze`:`No issues found`

---

## 3. 验证

```
$ WASM_FILE=/tmp/dummy_wasm.wasm cargo check -p node --tests
(零 error,仅 Tauri `frontend/dist` 门禁与 pre-existing pool_submitter.rs
 测试 `sp_core::sr25519` unused import 警告,均与本步无关)

$ cd wuminapp && flutter analyze lib/trade/onchain/onchain_trade_page.dart
No issues found!
```

---

## 4. 已知风险与缓解

| 风险 | 等级 | 缓解 |
|---|---|---|
| wuminapp 里 `lib/rpc/offchain.dart` + `lib/trade/offchain/offchain_pay_page.dart` 仍引用已删 RPC | **P2** | 入口在 `onchain_trade_page.dart` 已下架(SnackBar 提示);两个文件本身留 Step 2c 重写时删除 |
| CLI 清算行模式下 `stop_node` 不再有 pending 保护 | **P2** | UI 路径本来就没启动清算行;CLI `Ctrl+C` 会 drop task_manager → 所有 spawn 的 task 被取消,packer/listener/monitor 最后一次 tick 可能漏做。Step 3 改为 spawn_essential_handle + graceful shutdown 通道 |
| `offchain_keystore.SigningKey.shenfen_id` 字段名语义漂移 | **P3** | 本步仅改注释,不改字段名(避免 blast radius);rename 留 Step 3 |
| 跨行 ghost account bug(Step 2b-iii-a 发现) | **P2** | 已登记为独立任务,Step 3 跨行前修复;本步 Step 1 同行不触发 |
| Runtime 侧老 Calls 仍存在占 `RuntimeCall` 枚举槽位 | **P3** | `PowTxAmountExtractor` 分类还在;不影响功能,Step 2b-iv-b 删除 |

---

## 5. 后续

**Step 2b-iv-b**(runtime 清理,独立 **待评估**):
- 删除 `offchain-transaction-pos::pallet` 的 `submit_offchain_batch`(call_index 0)/ `propose_institution_rate`(1+2)/ `bind_clearing_institution`(9)等老 Calls
- 删除对应 `RecipientClearingInstitution` / `InstitutionRateBp` / `QueuedBatches` Storage
- `configs/mod.rs::PowTxAmountExtractor` 删对应分类分支
- **关键前置**:确认链上是否有此类历史 tx,评估 spec_version bump 下历史块重放兼容性,必要时保留 Call stub 返回 `Error::Removed`

**Step 2c**(wuminapp 重写):
- 重写 `offchain_pay_page.dart` 调用新 `offchain_submitPayment`
- 删除 `lib/rpc/offchain.dart`(老 RPC 客户端)
- `onchain_trade_page.dart` 的 `_openOffchainPay` 恢复跳转到新页面

---

## 6. 变更记录

- 2026-04-20:Step 2b-iv-a 完整落地。删除 3 个节点旧文件(736 行)+ `service.rs` 9 块(约 50 行)+ `rpc.rs` 378 行(3 RPC + 2 helper + 字段 + import)+ `ui/home/process/mod.rs` 58 行 + 头注释净化若干处。wuminapp 老入口在 `onchain_trade_page.dart` 下架。`cargo check -p node --tests` 零新 error,`flutter analyze` 零 issue。
