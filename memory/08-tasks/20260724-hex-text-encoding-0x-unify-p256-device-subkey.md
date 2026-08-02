# 任务卡：全仓密钥/签名文本编码统一带 0x（收口 P-256 设备子钥）

状态：已完成（2026-07-24 代码 + 测试 + 文档落地并全绿；ADR-041 生效）。

## 任务需求

把全仓「账户标识 / 公钥 / 签名」的**跨端文本编码**统一为小写 `0x` 加十六进制、单一形态、拒裸，定为永久铁律，防止再次出现一端带 `0x`、另一端裸 hex 的前后端不一致。

起因：CitizenApp 广场加载失败、会员「设备子钥签名校验失败」、创作者「请求过于频繁」、通讯录「离线」四处同源，根因是提交 `2574c19d 统一钱包字段` 把后端 P-256 `verifyP256Signature` 的 `0x` 归一化删成严格裸 hex，而客户端设备子钥签名仍发 `0x`，ES256 验签恒失败、广场会话建不起来，级联到四处（诊断结论见本卡「背景诊断」）。

## 边界（已与用户确认的范围）

**纳入本规则（统一为 `0x`+小写hex、拒裸）：**
- `account_id`、sr25519 32 字节公钥 —— 已由 ADR-040 落地为 `^0x[0-9a-f]{64}$`，本卡不重复改，只确认无回退。
- **P-256 设备子钥公钥（65B 未压缩点）与 ECDSA 签名（64B r‖s）的跨端文本** —— 本卡新增收口对象，ADR-040 只覆盖「32 字节公钥」，未覆盖 P-256。

**不纳入（保持现状，属另一类，绝不因本规则加 0x）：**
- sha256 内容哈希 / manifest hash / content_hash（摘要，内容寻址）。
- R2 对象键路径段 `account_id_hex`（协议 P-API-CITIZENAPP-002 明确「去掉 0x 后 64 位」）。
- SCALE 编码内部字节、签名消息 preimage、`device_key_hash` 的 sha256 preimage（内部二进制，非跨端文本）。
- 助记词 / seed / 私钥 / Keychain·Keystore·Secret（安全材料）。

## 不变边界（协议字节零改动）

- 不改设备绑定签名消息的 SCALE preimage：`signing_message(OP_SIGN_SQUARE_DEVICE_BIND, account_id ‖ p256_public_key ‖ issued_at)` 内 `p256_public_key` 仍以**裸**串 `scaleString`，与现有 golden 逐字节一致。故 `device_binding_golden_test.dart` 与 worker `DEVICE_BIND_GOLDEN_HEX` **不变**。
- 不改 `device_key_hash = sha256(stored p256_public_key)` 的 preimage 形态（存裸），会话签发与逐请求比对口径不变，**无需重注册设备子钥**。
- 不改签名算法、op_tag、字段顺序、账户字节、CID 派生。
- 全系零用户、正式创世前，数据直接按目标结构重建，不写 migration、不留兼容分支。

> 实现口径：`0x` 只出现在**跨端文本**（HTTP JSON 字段、`x-device-signature` header）。进入系统边界一次 `require 0x + strip → 内部裸`；内部一切保持裸不变。`verifyP256Signature` 保持内部裸函数（其单测「拒 0x」断言正确，不改）。

## 变更集（精确枚举，已核验 file:line）

### Worker（`citizenapp/cloudflare/src`）
1. `auth/device_subkey.ts`
   - `assertP256PublicKeyHex`：正则 `^04[0-9a-f]{128}$` → `^0x04[0-9a-f]{128}$`，strip 后返回裸（存储/签名消息继续用裸）。
   - 新增 `assertP256SignatureHex(value)`：require `^0x[0-9a-f]{128}$`，返回裸。
   - `verifyP256Signature` **不改**（内部裸）。
2. `auth/service.ts:135` createSession：`verifyP256Signature(loginMessage, assertP256SignatureHex(body.signature), subkey.p256_public_key)`。`registerDeviceSubkey:185` 已走 `assertP256PublicKeyHex`（改后要求 0x）。
3. `security/request_guard.ts:206`：逐请求 `signature` header 经 `assertP256SignatureHex` strip 后再 verify。
4. `chat/binding.ts` / `chat/service.ts:110`：`binding_signature` 同样 ingest strip。

### 客户端（`citizenapp/lib`）
5. `8964/services/device_subkey_registrar.dart` / `8964/services/square_api_client.dart:529`：注册 wire 字段 `p256_public_key` 送 `0x` 前缀；`publicKeyHex` 仍返回裸供签名消息使用（一变量两形态：签名消息用裸，wire 用 `0x`）。
6. 签名 wire（session/逐请求/chat 绑定）**已是 `0x`**（`square_session_provider.dart:58`、`square_compose_signers.dart:43`、`chat_runtime.dart:1171/1191`）→ 不改。

### 测试
7. Worker：新增 `assertP256SignatureHex` / `assertP256PublicKeyHex`（要求 0x）单测；`auth.test.ts`、`contacts.test.ts`、`chat.test.ts` 中 session/设备证明/绑定签名夹具改送 `0x`。`device_subkey.test.ts` 的 `verifyP256Signature` 断言与 golden **不变**。
8. 客户端：`device_binding_golden_test.dart` 不变；补/改注册 wire 送 `0x` 公钥的断言。

### 文档
9. 新增 `ADR-041`：全仓密钥/签名文本编码统一带 `0x`（含 P-256），引用并扩展 ADR-040。
10. `memory/07-ai/unified-protocols.md`：新增 §0.3 文本编码铁律；更新 P-API-CITIZENAPP-002 的 `signature` / `p256_public_key` 为 `0x` 口径。
11. `memory/07-ai/agent-rules.md`、`memory/AGENTS.md`：登记死规则。

## 验收

- `cd citizenapp/cloudflare && npx vitest run`（device_subkey / auth / contacts / chat 全绿）。
- `cd citizenapp && dart format --set-exit-if-changed`、`flutter analyze`、相关定向 `flutter test`（含 golden 不变）。
- 全仓 grep 确认无「跨端签名/公钥文本仍裸」的残留。
- 真机四页复验：广场各 tab、我的-会员/订阅、我的-创作者、我的-通讯录 恢复。

## 背景诊断（只读结论，来源本会话四路 agent + 亲验）

- culprit：`2574c19d` 删 `verifyP256Signature` 的 `.replace(/^0x/,'')`（`device_subkey.ts:52`），客户端签名仍 `0x`（`square_session_provider.dart:58`）→ 401 `invalid_signature`「设备子钥签名校验失败」（`service.ts:141`）。
- 会员/广场/通讯录同 `ensureSession()` 会话失败；创作者因会话永不缓存→握手风暴打满 `auth:{ip}` 10次/60秒桶（`request_guard.ts:264`）→ 429。
- 次生潜在项（非本卡范围，另记）：`ensureSession` 无 in-flight 去重 + auth 限流桶粒度，冷启动握手风暴。

## 完成记录（2026-07-24）

- Worker：`auth/device_subkey.ts` `assertP256PublicKeyHex` 改 `^0x04[0-9a-f]{128}$` + strip 返回裸；新增 `normalizeP256SignatureHex`（require `^0x[0-9a-f]{128}$`，返回裸，非法 null）；`verifyP256Signature` 内部裸函数不变。三处签名 ingest（`auth/service.ts` session、`security/request_guard.ts` 逐请求、`chat/service.ts` 绑定）经 normalize 规范化后验签，裸/大写/错长与验签失败维持既有 401。
- 客户端：`device_subkey_registrar.dart` 注册 wire `p256_public_key` 送 `0x` 前缀；`publicKeyHex` 仍返回裸供签名消息 SCALE preimage 使用；4 处签名（session/逐请求/chat 绑定）本就 `'0x$raw'`，未改。
- 未改：设备绑定签名消息 SCALE preimage（裸）、D1 存储、`device_key_hash` preimage、`verifyP256Signature` 断言、golden 向量 → 逐字节不变，无需重注册设备。
- 测试：新增 `assertP256PublicKeyHex`（require 0x + 返回裸）、`normalizeP256SignatureHex` 单测；`auth.test.ts` session、`contacts.test.ts` 设备证明夹具改送 `0x`。`device_subkey.test.ts` `verifyP256Signature` 断言与 golden 不变。
- 验收：`cd citizenapp/cloudflare && npx vitest run` → 29 文件 175 测试全绿；CitizenApp `flutter analyze`（改动文件 No issues）+ `device_binding_golden_test`（golden 匹配）、`device_subkey_test`、`square_account_action_test`、`square_account_deletion_service_test` 全绿。
- 文档：新增 ADR-041；`unified-protocols.md` 新增 §0.3 铁律 + 更新 P-API-CITIZENAPP-002 的 P-256 `0x` 口径；`agent-rules.md` 登记死规则。
- 真机四页复验：待用户在真机确认广场各 tab / 会员订阅 / 创作者 / 通讯录恢复（后端契约已对齐，客户端已发 0x）。

## 跟进三项完成记录（2026-07-24 第二批）

1. **Chat MLS 设备公钥 `device_public_key_hex` → 决定「不纳入」**。核查发现它是 OpenMLS 外部协议密钥（变长 2–512 hex，原生产出），在 KeyPackage、群成员、对端密钥比对（`chat_runtime.dart:1148`）与本地 MLS 状态库**文件路径**（`chat_runtime.dart:1275/1281`）中直接使用，且 MLS 边界 `mls_boundary.dart:41` 已有 `_stripHexPrefix` + `.toLowerCase()` 既有容忍归一化。强套严格 `0x` 会与 MLS 层冲突、改状态库文件路径、牵动对端互操作，风险高收益低——按 MLS 层自身编码治理，已在 ADR-041 与 §0.3 明确排除。
2. **`ensureSession` in-flight 去重（已落地）**：`square_api_client.dart` 新增 `_inflightSessions`，同账户并发共享一次握手 Future，`finally` 清理；重试语义原样保留（抽出 `_establishSessionWithRetry`）。杜绝广场/聊天多入口冷启动各跑一套握手→打满 `auth` 桶→429 的根因。
3. **auth 限流桶粒度拆分（已落地）**：`request_guard.ts` 把 `/square/auth/device/register`（每钱包一次的稀有操作）拆到独立 `authreg:{ip}` 桶，不再与频繁的 `challenge`/`session` 握手共用 `auth:{ip}`；数值保持 10/60（未放宽安全阈值，仅修正粒度）。
4. **registrar wire 0x 单测（已补）**：`test/8964/device_subkey_registrar_test.dart` 断言注册 wire `p256_public_key == '0x'+裸公钥`、`account_id`、`binding_signature` 透传。

验收（第二批）：worker `vitest run` 29 文件 175 测试全绿（限流拆分后）；客户端 registrar 新测 + `square_account_action` / `square_account_deletion_service`（ensureSession 去重后）全绿；`flutter analyze` 改动文件 No issues、`dart format` 0 改动。
