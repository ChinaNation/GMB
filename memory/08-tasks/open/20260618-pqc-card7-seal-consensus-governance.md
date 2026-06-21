# PQC card7:seal 共识签名 ML-DSA(节点二进制协调升级,非 setCode)+ Phase C/D 收紧治理

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§6/§11.6/§14/§15)
状态:open(**与钱包 PQC 主线解耦,独立协调上线;不阻塞 card0-6**)

任务需求:
1. **seal 共识签名 ML-DSA-65(BLOCKER B13:非 setCode)**:
   - seal 产生+验证全在 **node 二进制**(`citizenchain/node/src/core/service.rs:128/276-288` `seal=(nonce, sr25519::Signature)`、`SimplePow::verify`),不在 runtime WASM → **改它是全网节点二进制协调升级,不是 setCode**。
   - 🔴 新旧节点验块会立即互相拒绝(pow_fork_storm 同类、协议级不可自愈)→ 必须给:**激活高度** + **新旧节点验块共存窗口**(import 阶段双算法接受窗口)+ 全网矿工同窗口换二进制的协调流程。
   - 矿工 `powr` 密钥从同源派生 ML-DSA-65(gmb-pqc);`blake2_256` PoW 工作量哈希不动。
   - 在该协调方案定稿前,seal **不与钱包 PQC 主线绑定上线**。
2. **Phase C/D 收紧治理(H13/决策1:无恢复通道)**:
   - 产出 `PqcPolicy.bootstrap_deadline` 的**治理设定流程 + 多轮充分公告周期**。
   - 🔴 **关窗后未绑定老用户 = 资产终态锁定,无代绑/无线下恢复通道**(决策1/2);必须在**宪法/白皮书层提前充分告知**"逾期不升级即永久锁定"。
   - Phase B→C→D 的 `PqcPolicy.phase` 推进治理流程;**定稿前不得推进到 C/D**。

所属模块:Blockchain(node 共识)+ 治理/文档

输入文档:
- memory/04-decisions/ADR-022-unified-pqc-crypto.md(§6/§11.6/§15)
- memory/07-ai/chainspec-frozen / feedback_desktop_is_miner / project_pow_fork_storm_txpool

必须遵守:
- seal 改算法=二进制硬升级,绝不当 setCode;给共存窗口避免分叉。
- 无恢复通道是已拍板决策,公告/告知是硬前置。
- 不与 card2 setCode 主线混为一次上线。

输出物:
- seal ML-DSA 节点实现 + 激活高度/共存窗口方案 + 矿工协调 runbook + Phase C/D 治理与公告方案 + 文档

验收标准:
- 全网节点同窗口换二进制后 seal=ML-DSA 出块/验块通过,共存窗口内不分叉。
- bootstrap_deadline 治理流程 + 公告周期定稿;无恢复通道已在宪法/白皮书层告知。
- 真实运行态(多节点出块/验块/共存窗口)验收。
