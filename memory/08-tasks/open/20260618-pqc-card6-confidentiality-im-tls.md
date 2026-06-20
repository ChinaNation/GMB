# PQC card6:机密性域抗量子(IM ML-KEM-768 + TLS 混合)

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§1)
状态:open(机密性域,旧 PQC 方案完全未覆盖的盲区;与链端 card 并行,app-level)

任务需求:
补 harvest-now-decrypt-later 最不可逆的暴露面——IM 消息内容与 P2P/RPC 传输的抗量子:
1. **IM/MLS 换抗量子混合套件**:`wuminapp/rust/src/im_mls.rs:27-28` 当前 `MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519`(经典 X25519 + AES-128)→ X-Wing(X25519+ML-KEM-768 混合 KEM)+ AES-256/ChaCha20-256。
   - **阻断点**:`openmls_rust_crypto 0.5.1` 对 X-Wing `unimplemented!()`(`provider.rs:61-63`),`supports()` 只接经典 3 套(`:88-101`)。**必须换 / 升 crypto provider**(libcrux/X-Wing-capable backend),升 `wuminapp/rust/Cargo.toml:29-33` openmls 全家桶。**这是 provider 替换工程,非改一行常量**。
   - Dart 侧 `im_mls_native.dart` 零改(cipherSuite 透传)。既有 MLS 群需 rekey/重建(X-Wing 仅护新建群)。
2. **P2P/RPC TLS 混合**:`citizenchain/node/libp2p-websocket/src/tls.rs:82/129/150` 的 `rustls::crypto::ring::default_provider` → `aws-lc-rs` provider 显式配 `kx_groups=[X25519MLKEM768]`(ring 0.17 无 ML-KEM;需 aws-lc-rs 1.x + rustls 0.23);`Cargo.toml:40-42` futures-rustls 版本对齐(全工作区 rustls/ring/aws-lc-rs 版本统一,否则编译冲突)。
3. **上游受限项记录(不在本卡范围)**:节点 Noise 握手(substrate sc-network,经典 X25519)= 仓外上游依赖,等 libp2p/substrate 提供 PQC Noise,显式列为 blocker。

所属模块:Blockchain(node 传输)+ Mobile(wuminapp IM)

输入文档:
- memory/04-decisions/ADR-022-unified-pqc-crypto.md
- memory/07-ai/unified-protocols.md(新增 IM/TLS 机密性登记项)
- ADR-020(wuminapp p2p IM)、reference_wuminapp_ci_native_smoldot

必须遵守:
- 混合(hybrid)策略:经典 ⊕ 抗量子并联,破一仍守。
- X-Wing draft-6 字节格式风险:跨版本群组可能不互通,锁定版本。
- 不影响链上账户(四不变成立);本卡纯 app-level/传输,非 setCode/非硬分叉。

输出物:
- IM provider 替换 + 套件升级 + TLS provider 切换 + 群 rekey + 中文注释 + 测试(两端 IM 收发 / TLS 握手)+ 文档(协议登记)

验收标准:
- IM 新建群用 ML-KEM-768 混合套件加密,既有群已 rekey,两端收发通过。
- TLS 协商 X25519MLKEM768,节点互联正常;全工作区 rustls 版本对齐无冲突。
- AES-256 落地;Noise 上游 blocker 已记录;真实运行态(IM 会话 / 节点组网)验收。
