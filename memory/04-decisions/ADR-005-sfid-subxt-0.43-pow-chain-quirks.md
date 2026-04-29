# ADR-005 sfid 后端在 PoW 链上使用 subxt 0.43 的协议对齐

- 日期：2026-04-07
- 模块：`sfid/backend` ↔ `citizenchain`
- 状态：已采纳

## 背景

sfid 后端通过 `subxt = 0.43.1` 向 citizenchain 提交两类 extrinsic：

- `DuoqianManage.register_sfid_institution`（生成机构 SFID 后上链登记）
- `SfidSystem.rotate_sfid_keys`（密钥环旋转）

citizenchain 是 **PoW 链**：出块依赖矿工算力，GRANDPA finality 显著落后 best block
（实测 best=64 / finalized=62 是常态）。

subxt 0.43 默认 `DefaultExtrinsicParams` + `inject_account_nonce_and_block`
是为 PoS 链（finality≈head）设计的，在 PoW 链上有三处隐性踩坑：

1. **nonce 视图错位**
   `tx_client.rs:608` 把 nonce 从 `latest_finalized_block_ref()` 拉，
   而 txpool 用 best 块视图 validate。finalized 落后 best 时，best 视图里
   该账户的 nonce 已经被某笔尚未 finalize 的 extrinsic 推进，
   subxt 写入的 finalized 视图旧 nonce 会被 `CheckNonce::validate` 判定为
   `InvalidTransaction::Stale`（"Transaction is outdated"）。

2. **mortal era 的 birth-block 视图错位**
   `DefaultExtrinsicParamsBuilder::default()` 默认 `mortal=32 blocks`，
   subxt 把 finalized 块的 hash 当 birth block hash 写进 era。
   链端 `frame_system::CheckMortality::implicit()` 用 best 视图查
   `BlockHash::contains_key(birth_n)`，PoW 链 reorg 后查不到，
   返回 `InvalidTransaction::AncientBirthBlock`。

3. **wait_for_finalized 在 PoW 链上不可用**
   sfid 业务语义只需要"extrinsic dispatch 成功"，不需要 GRANDPA 最终化。
   但代码原本调 `wait_for_finalized()`，120s 超时常常先于 finality 触发，
   表面错误是 `timed out waiting for finalization`，实际块已上链。

此外发现 `chain/runtime_align.rs` 里 `INSTITUTION_DOMAIN` 历史上被错误声明为
`&[u8]`，与链端 verifier 用的 `b"GMB_SFID_INSTITUTION_V2"`（类型 `&[u8; 23]`）
SCALE 编码不同（`&[u8]` 多写 1 字节 Compact 长度前缀），导致
`build_institution_credential` 算出的 blake2_256 与链端 verifier 算出的不一致，
链端返回 `Pallet error: DuoqianManage::InvalidSfidInstitutionSignature`。
这是协议对齐 bug，与上述 PoW 链坑无关，但本次一并修掉。

> **尾注 · 2026-04-20**: 上述 `GMB_SFID_INSTITUTION_V2` 域名已在
> `20260420-unified-DUOQIAN_V1-domain` 任务中彻底退役，改用
> `(DUOQIAN_DOMAIN = b"DUOQIAN_V1", OP_SIGN_INST = 0x13, ...)`。SCALE
> 类型对齐铁律（`[u8; N]` 无长度前缀）在新方案下依旧适用。

## 决策

`sfid/backend` 推链路径上**强制**做以下四件事：

1. 用 `subxt::backend::legacy::LegacyRpcMethods::system_account_next_index(&signer)`
   从链 best+pool 视图取 nonce，**不依赖** subxt 内部 `inject_account_nonce_and_block`。
2. extrinsic 强制 immortal：
   `DefaultExtrinsicParamsBuilder::new().immortal().nonce(chain_nonce).build()`。
   重放保护由 `register_nonce`（链端 `UsedRegisterNonce` 单射存储）和 `CheckNonce`
   双重承担，era 在此场景下不提供任何额外安全性。
3. submit 后只等 `TxStatus::InBestBlock` 即返回（必要时同时接受 `InFinalizedBlock`），
   绝不调用 `wait_for_finalized`。120s 是兜底超时，正常路径几秒内返回。
4. 任何与链端 runtime verifier 共享 SCALE 编码的 domain/常量，必须声明为
   `[u8; N]` 数组，与链端 `b"..."` 字面量类型严格对齐。`&[u8]` 切片在 SCALE
   下会多写 Compact 长度前缀，导致 hash 不一致 → BadProof / InvalidSignature。

落地代码位置：

- `sfid/backend/src/sheng-admins/institutions.rs::submit_register_sfid_institution_extrinsic`
- `sfid/backend/src/key-admins/mod.rs::submit_rotate_sfid_keys_extrinsic`
- `sfid/backend/src/chain/runtime_align.rs` 顶部 `*_DOMAIN` 常量段及注释

## 影响

- ✅ register_sfid_institution / rotate_sfid_keys 在 PoW 链上稳定工作
- ✅ 任何未来新加的链上 verifier domain 都会被代码顶部注释提醒使用数组类型
- ⚠️ sfid 推链不再等 GRANDPA finality；如果未来出现需要"等到不可逆"的业务
  场景（例如跨链桥/重资产托管），需要显式 opt-in `wait_for_finalized`，
  并且业务超时要按 GRANDPA 节奏重新设计
- ⚠️ 显式 nonce 取自 best+pool 视图：如果同一账户被并发推多笔交易，
  sfid 必须串行化提交（已通过单写入路径保证）

## 备选方案

1. **升级 subxt 到 ≥ 0.50** —— 新版引入 `BlockRefT` 抽象，可以选择
   from-best 取 nonce。但需要同步升级 jsonrpsee/scale 系列依赖，且
   PoW 链下 mortal era 仍然踩坑，必须 immortal。性价比低，弃用。
2. **改用 substrate-api-client** —— API 更原始，需要更多手写胶水。
   现有 sfid 后端 24 处 subxt 调用全都要改，工作量过大，弃用。
3. **链端 runtime 改造**：让 GRANDPA finality 跟上 PoW best —— 改动
   citizenchain 共识层，超出当前任务卡范围，且违反 "no chain restart" 铁律。

## 后续动作

- [x] 修改两处推链点 + 加注释
- [x] 修复 INSTITUTION_DOMAIN 类型 bug
- [x] 写本 ADR
- [x] 回写 memory feedback（防止后续被回滚）
- [ ] 后续打通"链上验证收到 SFID"环节（独立任务卡）
