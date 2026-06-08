# 钱包账户 Root Seed、PQC 迁移与 Passkey 解锁方案

## 决策落定（2026-06-07，ADR-016）

- 状态：设计阶段完成（ADR + 设计文档已出），实现未启动，卡片保持 `open`。
- 决策真源：`memory/04-decisions/ADR-016-account-key-pqc-migration.md`
- 实现蓝图：`memory/05-modules/citizenchain/runtime/otherpallet/ACCOUNT_KEYS_PQC_TECHNICAL.md`
- 已定：
  - **启动算法 = ML-DSA-65**（FIPS 204 Category 3），版本标签 `0x02`，可滚动升级到 ML-DSA-87，升级不换账户。
  - **账户抽象**：canonical AccountId 永远是 sr25519 派生值；ML-DSA-65 经账户状态机（Sr25519Only → Bound → PqcOnly）绑定为同一账户的签名凭证；**不换助记词/账户/地址/余额**。
  - **同根派生**：`AccountRootSeedV1`（= 现有 mini-secret）经 HKDF 派生 sr25519 与 ML-DSA-65；过渡无需新秘密。
  - **hybrid 双签绑定** + 链上**只存公钥 hash**；冷热钱包共用 `gmb-pqc` crate。
  - **Passkey 本轮整体推迟**，作为独立后续立项（非抗量子手段，不抢助记词根地位）。
- 协议登记：`unified-protocols.md` 新增 P-TX-008 / P-TX-009 / P-STORAGE-005（草案），P-QR-002 / P-SIGN-001 补抗量子草案说明。

任务需求：

- 将已确认的钱包账户安全方案写入任务卡，作为后续冷热钱包、链上账户、PQC 迁移和 Passkey 解锁改造的执行依据。
- 当前 `wuminapp` 热钱包与 `wumin` 冷钱包都必须纳入同一账户模型，不允许只改单侧。
- 目标是用户长期只感知一套助记词、一个钱包账户、一个地址和一份余额；当前使用 sr25519，未来平滑迁移到 PQC / ML-DSA。

所属模块：

- `wuminapp`：手机热钱包、账户创建/导入、本地 seed 存储、Face ID / 设备密码解锁、在线交易签名、冷钱包扫码请求。
- `wumin`：冷钱包、离线账户创建/导入、本地 seed 存储、离线扫码签名。
- `citizenchain/runtime`：账户签名模型、`AccountId`、未来 PQC 公钥绑定和交易验签。
- `sfid`：账户绑定、管理员 Passkey / WebAuthn 安全动作边界；不得托管钱包 Root Seed。
- `memory`：ADR、模块技术文档、协议文档和任务执行记录。

当前代码事实：

- 当前代码没有名为 `Root Seed` 的实现。
- 当前 `wuminapp/lib/wallet/core/wallet_manager.dart` 和 `wumin/lib/wallet/wallet_manager.dart` 实际流程为：助记词 -> entropy -> `CryptoScheme.miniSecretFromEntropy` -> 32 字节 mini-secret -> `Keyring.sr25519.fromSeed` -> sr25519 公钥与 SS58 地址。
- 当前 `wuminapp` / `wumin` 都只支持 sr25519，未发现 ML-DSA / Dilithium / PQC 依赖或实现。
- 当前冷签二维码协议 `sign_request` / `sign_response` 写死 `sig_alg == sr25519`。
- 当前 `wuminapp/lib/rpc/signed_extrinsic_builder.dart` 编码 extrinsic 时固定使用 `SignatureType.sr25519`。
- 当前 `citizenchain/runtime/src/lib.rs` 使用 `Signature = MultiSignature`，`AccountId` 从签名者公钥推导。
- 当前 runtime 文档注释中已有“项目内 `AccountId = AccountId32`，其 32 字节原始内容即对应 sr25519 公钥”的事实约束。

目标方案：

1. 将当前 32 字节 mini-secret 升级定义为 `AccountRootSeedV1`。
2. 用户继续只拥有一套助记词；助记词恢复 `AccountRootSeedV1`。
3. 当前 sr25519 地址继续由 `AccountRootSeedV1` 生成，保证现有用户体验中的地址不变。
4. 未来 PQC 密钥从同一个 `AccountRootSeedV1` 通过 HKDF 派生，例如：

```text
AccountRootSeedV1
  ├─ sr25519：当前签名密钥，保持地址不变
  └─ ML-DSA：HKDF(AccountRootSeedV1, "GMB/ML-DSA-44/v1")
```

5. 链上不要把 sr25519 公钥地址和 PQC 公钥地址当成两份余额；应实现同一个账户主体绑定多把签名钥匙。
6. 用户界面只展示一个钱包地址、一个账户、一个余额；sr25519 和 PQC 是同一账户下面的不同签名凭证。
7. Passkey 必须结合使用，但只作为解锁、恢复辅助和高危操作确认，不得作为助记词或 `AccountRootSeedV1` 的来源。
8. Face ID / 指纹 / 设备密码只作为本机解锁条件，不是密码学根。
9. 冷钱包不得依赖在线 Passkey 才能签名，必须保持离线签名能力。

安全原则：

- 助记词是最终恢复根。
- `AccountRootSeedV1` 不上传服务器。
- SFID 不保存钱包助记词、Root Seed、私钥或 PQC 私钥。
- Passkey 可以使用 Apple / Google 同步能力提升恢复体验，但不能替代助记词。
- `wuminapp` 当前长期保存助记词的实现必须重新评估；目标状态应避免长期明文保存助记词，优先长期保存加密后的 `AccountRootSeedV1`。
- 重要签名、高危绑定、PQC 公钥绑定和账户迁移操作必须重新触发本机认证或 Passkey 安全动作。

推荐技术路线：

1. 文档和 ADR 阶段：
   - 新增账户密钥 ADR，正式定义 `AccountRootSeedV1`、sr25519 当前分支、ML-DSA 未来分支、Passkey 边界和冷热钱包一致性。
   - 更新 `wuminapp`、`wumin`、`citizenchain/runtime`、`sfid` 模块技术文档。
   - 更新统一协议文档中和 QR 签名算法、签名载荷、账户绑定相关的登记项。

2. 钱包阶段：
   - `wuminapp` 和 `wumin` 使用同一套 `AccountRootSeedV1` 派生规则。
   - 热钱包使用设备安全密钥加密保存 `AccountRootSeedV1`，通过 Face ID / 指纹 / 设备密码解锁。
   - 冷钱包使用同一派生规则和本机解锁机制，但不依赖在线 Passkey。
   - 钱包 UI 不暴露多公钥账户概念，只展示一个用户账户。

3. Passkey 阶段：
   - Passkey 用于本地解锁辅助、云同步恢复辅助或高危操作二次确认。
   - 支持 WebAuthn PRF 时，可评估用 PRF 输出参与加密包装密钥；不支持时，Passkey 只作为授权门禁，不得直接决定 Root Seed。
   - Passkey 失败或平台账号不可用时，助记词必须仍可恢复账户。

4. PQC 阶段：
   - 首选 NIST ML-DSA；**已定启动 `ML-DSA-65`**（见顶部决策落定 / ADR-016），可升级到 `ML-DSA-87`。
   - 先实现 PQC 公钥绑定到账户主体，而不是创建新的 PQC 钱包地址。
   - 过渡期可要求 sr25519 + ML-DSA 双签完成关键绑定或迁移。
   - 最终 runtime 支持 PQC 签名代表同一个 `AccountId` 发起交易。

预计修改目录：

- `memory/04-decisions/`：新增账户密钥 ADR，记录 `AccountRootSeedV1`、Passkey、PQC 和地址不变决策；涉及文档。
- `memory/05-modules/wuminapp/`：更新热钱包账户、签名、QR 和本地存储技术文档；涉及文档。
- `memory/05-modules/wumin/`：更新冷钱包账户、离线签名和同源派生技术文档；涉及文档。
- `memory/05-modules/citizenchain/`：更新 runtime 账户签名模型和 PQC 迁移说明；涉及文档。
- `memory/05-modules/sfid/`：更新 SFID 账户绑定与 Passkey 边界说明；涉及文档。
- `wuminapp/`：后续实现热钱包 `AccountRootSeedV1`、本地加密、Passkey/Face ID 解锁、PQC 派生预留；涉及代码、测试、注释和残留清理。
- `wumin/`：后续实现冷钱包同源派生、离线签名协议扩展和本地解锁；涉及代码、测试、注释和残留清理。
- `citizenchain/runtime/`：后续实现账户主体绑定多签名凭证、PQC 公钥绑定、PQC 验签和迁移；涉及代码、测试、注释和残留清理。
- `sfid/`：后续按需调整账户绑定字段和 Passkey 安全动作边界；涉及代码、测试、注释和残留清理。

输入文档：

- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/repo-map.md`
- `memory/03-security/security-rules.md`
- `memory/07-ai/agent-rules.md`
- `memory/07-ai/workflow.md`
- `memory/07-ai/context-loading-order.md`
- `memory/07-ai/unified-required-reading.md`
- `memory/07-ai/unified-naming.md`
- `memory/07-ai/unified-protocols.md`
- `memory/07-ai/definition-of-done.md`
- `memory/07-ai/pre-submit-checklist.md`
- 对应模块技术文档与模块级完成标准

必须遵守：

- 不允许只改热钱包或只改冷钱包；冷热钱包必须使用同一账户派生规则。
- 不允许把 Passkey 设计为钱包资产恢复根。
- 不允许 SFID 保存原始助记词、Root Seed 或私钥。
- 不允许把 PQC 公钥直接设计成新余额账户，除非用户明确放弃地址不变目标。
- 不允许默认设计历史兼容、双轨兼容或影子旧流程；应按目标账户模型彻底收敛。
- 涉及 QR 签名协议、签名载荷、交易编码、runtime 签名模型时，必须先更新统一协议和模块文档。
- 改代码后必须更新文档、补中文注释、清理残留并跑对应测试。

输出物：

- 账户密钥 ADR
- 更新后的冷热钱包技术文档
- 更新后的 runtime / SFID 边界文档
- 热钱包和冷钱包同源派生代码
- Passkey / Face ID 解锁实现
- PQC 派生和账户绑定实现
- QR 和 signed extrinsic 协议更新
- 对应单元测试、跨端 fixture 和必要集成测试
- 残留清理记录

验收标准：

- 一套助记词可以在 `wuminapp` 和 `wumin` 中恢复同一个账户主体。
- 当前 sr25519 地址在目标方案下保持不变。
- 用户界面只展示一个钱包账户和一份余额。
- `AccountRootSeedV1` 不离开本机或用户主动持有的助记词恢复路径。
- Passkey 可提升解锁或恢复体验，但 Passkey 丢失不导致资产永久丢失。
- PQC 公钥绑定后不生成独立余额账户。
- 冷钱包仍可离线完成签名。
- 代码已补中文注释。
- 文档已更新。
- 测试通过。
- 残留已清理。

待确认问题（更新于 2026-06-07）：

- ~~PQC 默认参数采用 `ML-DSA-44` 还是 `ML-DSA-65`~~ → **已定 ML-DSA-65**（ADR-016）。
- ~~Passkey 是否必须支持 WebAuthn PRF~~ → **Passkey 本轮整体推迟**，留独立后续立项再议 PRF。
- 是否允许热钱包长期保存加密助记词，还是只保存加密后的 `AccountRootSeedV1`（任务卡安全原则倾向后者）→ **实现期定**。
- 现有开发期账户迁移说明 → 走账户状态机自然过渡（Sr25519Only → Bound → PqcOnly），不设计长期双轨兼容。
- 新增实现期待定：general-tx 手续费向 canonical 账户计费的落点；`fips204` 是否暴露 seed-based 确定性 keygen；切 PqcOnly 的全网治理截止策略。
