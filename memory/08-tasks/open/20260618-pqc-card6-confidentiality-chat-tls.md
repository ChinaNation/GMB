# PQC card6:机密性域抗量子(Chat 一次到位 ML-KEM-768 混合 + Chat 钱包绑定签名 + TLS 混合)

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§10/§14/§15 决策3/6)
状态:open(机密性域,与链端 card 并行,app-level)

任务需求:
1. 🔴 **(决策6)Chat/MLS 一次到位升级(card0 不动 Chat)**:`citizenapp/rust/src/chat_mls.rs:27-28` 当前 `MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519`(经典 X25519 + **AES-128**)→ **一次性**换 X-Wing(X25519+ML-KEM-768 混合 KEM)+ **AES-256/ChaCha20-256**(不分两步,避免 MLS 套件二次变更破坏已有会话)。
   - **阻断点**:`openmls_rust_crypto 0.5.1` 对 X-Wing `unimplemented!()`(`provider.rs:61-63`),`supports()` 只接经典 3 套 → **必须换 libcrux/X-Wing-capable provider**,升 `citizenapp/rust/Cargo.toml:29-33` openmls 全家桶。Dart 侧 `chat_mls_native.dart` 零改(cipherSuite 透传)。
   - 🔴 **(决策3)Chat 的 ML-KEM 是 MLS 设备/会话密钥,与 `AccountSeedV1` 无关**(账户不派生 ML-KEM);走前向保密 rekey,绝不复用账户密钥。
2. 🔴 **(H15)Chat 设备绑定签名 `chat_device_bind`**(P-CHAT-001:钱包 sr25519 对 Chat 设备/PeerId/端点签名)纳入升级——量子破后可伪造劫持 Chat 设备绑定。按 sig_alg 升 ML-DSA;归属验证同 §9(ML-DSA 公钥经 CID 查链 AccountPqcKey 证明属于该地址)。
3. **P2P/RPC TLS 混合**:`citizenchain/node/libp2p-websocket/src/tls.rs:82/129/150` `rustls::crypto::ring::default_provider` → `aws-lc-rs` 配 `kx_groups=[X25519MLKEM768]`(ring 0.17 无 ML-KEM);`Cargo.toml:40-42` futures-rustls 版本对齐(全工作区 rustls/ring/aws-lc-rs 统一,否则编译冲突)。
4. 🔴 **(L6)既有 MLS 群 rekey 归属**:明确 rekey 由谁触发(升级后自动/群主/用户)、新旧套件群迁移期共存策略、未 rekey 旧群降级处理;把"既有群已 rekey"拆成可执行子步骤,非仅验收断言。
5. **上游受限项记录**:节点 Noise 握手(substrate sc-network,经典 X25519)= 仓外上游依赖,等 libp2p/substrate 提供 PQC Noise,显式列 blocker。

所属模块:Blockchain(node 传输)+ Mobile(citizenapp Chat)

输入文档:ADR-022(§10/§14)/ unified-protocols(P-CHAT-001 / chat_device_bind / 新增 Chat/TLS 机密性登记)/ ADR-020 / reference_citizenapp_ci_native_smoldot

必须遵守:混合(经典⊕抗量子)破一仍守;X-Wing draft-6 跨版本群组可能不互通须锁版本;KEM 不当身份认证;账户不复用 KEM;不影响链上账户(四不变);本卡纯 app-level/传输,非 setCode/非硬分叉。

输出物:Chat provider 替换 + X-Wing 套件(含 AES-256)+ chat_device_bind 升级 + 群 rekey 流程 + TLS provider 切换 + 中文注释 + 测试(两端 Chat 收发/绑定/TLS 握手)+ 文档(协议登记)。

验收标准:
- Chat 新建群用 ML-KEM-768 混合套件(AES-256)加密、两端收发通过;既有群按定义流程已 rekey;Chat 设备绑定签名升 ML-DSA 且归属经查链验证。
- TLS 协商 X25519MLKEM768、节点互联正常、全工作区 rustls 版本无冲突;Noise 上游 blocker 已记录;真实运行态(Chat 会话/绑定/节点组网)验收。
