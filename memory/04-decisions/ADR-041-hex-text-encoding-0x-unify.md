# ADR-041：全仓密钥/签名文本编码统一带 0x

状态：Accepted（2026-07-24；P-256 设备子钥收口已落地并全绿，规则永久生效）。

## 背景

ADR-040 已把 `account_id` 与 sr25519 32 字节公钥的跨端文本编码固定为 `^0x[0-9a-f]{64}$`。但 ADR-040 只覆盖「32 字节公钥」，未覆盖 **P-256 设备子钥**（65 字节未压缩点公钥 + 64 字节 ECDSA 签名，Web Crypto ES256，用于 CitizenApp↔Cloudflare 广场会话与逐请求设备证明）。

提交 `2574c19d 统一钱包字段` 把后端 `verifyP256Signature` 从「归一化 `0x` 后验签」改成「严格裸 hex、拒 0x」，而 CitizenApp 客户端设备子钥签名仍发 `0x`，导致 ES256 验签恒失败、广场会话建不起来，级联使广场/会员/创作者/通讯录四处加载失败。根因是同一把钥匙的文本编码在前后端分叉（一端 0x、一端裸）。

## 决策

**凡「账户标识 / 公钥 / 签名」的跨端文本编码，全仓统一为小写 `0x` 加十六进制、单一形态、拒裸。** 这是 ADR-040 文本编码原则的推广，适用于所有密码学体系（sr25519 账户、P-256 设备子钥，及未来任何密钥/签名文本），不因子系统不同而分叉。

- 账户 / sr25519 公钥：`^0x[0-9a-f]{64}$`（ADR-040 已落地，不变）。
- P-256 设备子钥公钥：`^0x04[0-9a-f]{128}$`（65 字节未压缩点）。
- P-256 设备子钥签名：`^0x[0-9a-f]{128}$`（64 字节 r‖s）。
- 进入系统边界一次 `require 0x + strip → 内部裸`；内部只保留裸形态，不同时保存带 `0x`、大写、混合大小写。拒绝裸/大写/错长的跨端输入。

### 不纳入（不是「密钥/签名文本编码」，保持现状）

- sha256 内容哈希 / manifest hash / content_hash（内容寻址摘要）。
- R2 对象键路径段 `account_id_hex`（协议 P-API-CITIZENAPP-002 明确「去掉 0x 后 64 位」）。
- SCALE 编码内部字节、签名消息 SCALE preimage、`device_key_hash` 的 sha256 preimage（内部二进制，非跨端文本）。
- 助记词 / seed / 私钥 / Keychain·Keystore·Secret（安全材料）。
- **Chat MLS 设备公钥 `device_public_key_hex`（OpenMLS 设备身份密钥）**：外部协议密钥，非 GMB 账户/授权材料，变长（2–512 hex），由原生 OpenMLS 产出并在 KeyPackage、群成员、对端密钥比对（`chat_runtime.dart` peer 比较）与本地 MLS 状态库**文件路径**中直接使用；MLS 边界 `mls_boundary.dart` 已有 `_stripHexPrefix` + `.toLowerCase()` 的既有容忍归一化。强套 GMB 严格 `0x`（require 0x、拒裸、单一形态）会与 MLS 层冲突、改动状态库文件路径、牵动对端互操作，风险高收益低。该密钥按 MLS 层自身编码治理，明确不纳入本 ADR。

### 实现边界（P-256 收口，逐字节零改动协议）

- `0x` 只出现在跨端文本（HTTP JSON 字段 `p256_public_key` / `signature` / `binding_signature`、`x-device-signature` header）。
- 内部裸口径不变：设备绑定签名消息仍以裸公钥串 `scaleString`，与 golden 逐字节一致；D1 存储与 `device_key_hash` preimage 继续裸。故 golden 向量与已签摘要**不变，无需重注册设备子钥**。
- `verifyP256Signature` 保持内部裸函数（拒 0x），`0x` 由边界 `normalizeP256SignatureHex` / `assertP256PublicKeyHex` 先 strip。
- 全系零用户、正式创世前，无 migration、无兼容分支、无「两者都收」的容错层。

## 影响

- 直接落地：`citizenapp/cloudflare/src/auth/device_subkey.ts`（`assertP256PublicKeyHex` require 0x + strip；新增 `normalizeP256SignatureHex`）、`auth/service.ts`、`security/request_guard.ts`、`chat/service.ts` 边界规范化；`citizenapp/lib/8964/services/device_subkey_registrar.dart` 注册 wire 送 `0x` 公钥。
- 验收：cloudflare `vitest run` 29 文件 175 测试全绿；CitizenApp 设备绑定 golden、device_subkey、square account action 测试全绿，golden 未变。
- Chat MLS 设备公钥经核查后**明确排除**（见「决策 / 不纳入」），不并入 `0x`。
- 次生健壮性收口（同批完成）：CitizenApp `SquareApiClient.ensureSession` 增加 in-flight 去重（同账户并发共享一次握手）；Worker `guardRequest` 把 `/v1/square/auth/device/register` 拆到独立 `authreg:{ip}` 限流桶，不再与挑战/会话握手挤占同一 `auth:{ip}` 配额。

## 备选方案

- 后端恢复「归一化两端都收 0x」：拒绝。等于「两者都收」的兼容层，违反无兼容死规则，且不是单一形态。
- P-256 保持裸、账户保持 0x（双口径）：拒绝。同一类「密钥/签名文本」在不同子系统分叉正是本次 bug 的根源，违反「统一=零例外」。
- 客户端签名改裸对齐后端裸：拒绝。与 ADR-040 已确立的 `0x` 账户口径分裂，留下「公钥裸/账户 0x」的不一致。

## 后续动作

- 任务卡 `memory/08-tasks/20260724-hex-text-encoding-0x-unify-p256-device-subkey.md` 记录实施与验收。
- 统一协议入口 `memory/07-ai/unified-protocols.md` §0.3 登记本铁律，并更新 P-API-CITIZENAPP-002 的 `signature` / `p256_public_key` 为 `0x` 口径。
- AI 硬规则 `memory/07-ai/agent-rules.md`、`memory/AGENTS.md` 登记死规则，禁止新代码再产生裸密钥/签名跨端文本。
