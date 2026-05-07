# 冷钱包扫码签名两色识别方案

- 版本:2026-05-07
- 状态:当前详细事实源,由 `memory/07-ai/unified-protocols.md` 统一管辖
- 范围:wumin 冷钱包扫描 QR 后的识别、展示与签名放行规则
- 依赖:
  - `memory/01-architecture/qr/qr-protocol-spec.md`
  - `memory/01-architecture/qr/qr-protocol-fixtures/`
  - `memory/01-architecture/qr/qr-action-registry.md`

## 一、四条铁律

1. **扫码协议只有一个**:`WUMIN_QR_V1`。
2. **两色终态**:绿色 = 识别通过并允许签名;红色 = 识别失败并禁止签名。禁止黄色盲签兜底。
3. **识别 = 结构识别 + payload 交叉验证**。envelope 合法、payload 可被冷钱包完整解码、display 与 decoder 输出逐字一致,才允许签名。
4. **不残留、不兼容、不过渡**。旧 action、旧 pallet/call、旧字段门控不得作为兼容分支保留。

## 二、冷钱包消费的 kind

`qr-protocol-spec.md` 当前登记 6 种 kind。冷钱包只消费 2 种:

| 扫入 kind | 冷钱包产出 | 结果 |
|---|---|---|
| `login_challenge` | `login_receipt` | 通过后允许生成登录回执 |
| `sign_request` | `sign_response` | 通过后允许签名并生成签名回执 |

其余 4 种当前 kind(`login_receipt` / `sign_response` / `user_contact` / `user_transfer`)不由冷钱包消费,扫到即红色拒绝。已下线的 `user_duoqian` 不属于当前 kind 枚举。

## 三、envelope 层校验

全部为真才进入 kind 专属校验:

- `envelope.proto == 'WUMIN_QR_V1'`
- `envelope.kind ∈ {login_challenge, sign_request}`
- `envelope.id` 长度 16-128,字符集 `[a-zA-Z0-9_-]`
- `envelope.issued_at` 与 `envelope.expires_at` 均存在
- `now <= envelope.expires_at`
- `envelope.body` 字段集匹配 `qr-protocol-spec.md`,不得出现未知字段、旧字段、别名字段

任一失败即红色拒绝。

## 四、kind = login_challenge

补充校验:

- `body.system` 非空,当前只允许 `sfid` / `cpms`
- `body.sys_pubkey` 是合法 `0x<64hex>`
- `body.sys_sig` 是合法 `0x<128hex>`
- 按 `qr-protocol-spec.md` 的签名原文规则验证 `sys_sig`

全过即绿色,允许生成 `login_receipt`。UI 必须展示 `system`,由用户确认是否为预期登录系统。

## 五、kind = sign_request

补充校验:

- `body.address` 是 SS58 地址
- `body.pubkey` 是合法 `0x<64hex>`
- `body.sig_alg == 'sr25519'`
- `body.payload_hex` 是非空 `0x<hex>`
- `body.display.action` 非空,且已登记在 `qr-action-registry.md`
- `body.display.fields[*].key` 与 registry 字段逐字对齐
- `PayloadDecoder.decode(body.payload_hex)` 返回 `decoded != null`
- `decoded.action == body.display.action`
- 对 `decoded.fields` 每一项 `(key, value)`,在 `body.display.fields` 中按同名 `key` 找到的 value 必须与 decoded 侧 value 逐字相等

全过即绿色,允许生成 `sign_response`。

`spec_version` 不再参与 envelope 层识别,也不存在 `supportedSpecVersions` 集合门控。链端验签所需的 runtime `spec_version` 仍在 Substrate signing payload 的 additional_signed 中,不作为 QR envelope 字段。

## 六、Runtime 升级哈希直签例外

`propose_runtime_upgrade` 与 `developer_direct_upgrade` 的完整 WASM call data 体积过大,不能放入 QR。当前规则:

- `display.action ∈ {propose_runtime_upgrade, developer_direct_upgrade}`
- `payload_hex` 必须是 32 字节 hash
- `display.fields` 必须包含 `wasm_hash`
- 用户必须在冷钱包屏幕核对 `wasm_hash`

仅满足以上条件时绿色放行。除此之外,任何 decode 失败都红色拒绝。

## 七、action / fields 对齐规则

唯一详细登记:`memory/01-architecture/qr/qr-action-registry.md`

任何一端新增或修改 action / field key,必须:

1. 先改 registry。
2. 再改 decoder 与签发方。
3. 补 fixture 与测试。
4. 扫描旧 action、旧 pallet/call、旧字段门控残留。

## 八、已废弃识别规则

以下规则不得恢复:

- `supportedSpecVersions` / `isSupported` envelope 门控
- `allowedHashedActions` 黄色盲签白名单
- `VotingEngine(9)` 旧投票 action
- 业务 pallet 的 `execute_*` / `cancel_failed_*` wrapper action
- `DuoqianManage` 旧 pallet 名
- `user_duoqian` 当前 kind

## 九、验证清单

### 应绿色通过

1. `login_challenge` → `login_receipt`
2. `transfer`
3. `internal_vote`
4. `joint_vote`
5. `cast_referendum`
6. `finalize_proposal` / `retry_passed_proposal` / `cancel_passed_proposal`
7. `propose_create_institution` / `propose_close_institution`
8. `propose_create_personal` / `propose_close_personal`
9. `propose_transfer` / `propose_safety_fund_transfer` / `propose_sweep_to_main`
10. `propose_runtime_upgrade` / `developer_direct_upgrade` 的 32 字节哈希直签例外

### 应红色拒绝

1. `proto != WUMIN_QR_V1`
2. 未登记 kind
3. 已过期 envelope
4. 随机 `payload_hex`
5. `display.action` 与 decoder action 不一致
6. `display.fields` 与 decoder fields 不一致
7. 旧 `spec_version` 门控、旧 wrapper action、旧 `DuoqianManage` 名称试图作为当前协议依据

## 十、拒绝的替代方案

### 10.1 envelope 内嵌 issuer_sig

桌面端和移动端都是客户端程序,反编译即可拿到打包私钥。把来源证明放到客户端不能建立稳定 provenance,反而会制造两套判断模型。拒绝。

### 10.2 黄色盲签兜底

白名单按 action 字符串猜来源,无法验证 payload 实际内容。任何 decode 失败都必须红色拒绝。拒绝。

### 10.3 老版兼容模式

本仓库处于重新创世前收口阶段,协议固定前必须清掉旧分支。拒绝。
