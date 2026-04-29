# QR Action Registry(唯一事实源)

- 版本:2026-04-22
- 状态:唯一事实源(Single Source of Truth)
- 范围:所有 `kind = sign_request` 的 `body.display.action` + `body.display.fields`
- 依赖:`memory/05-architecture/qr-signing-recognition.md` § 识别方案

任何一端(`wumin/lib/signer/payload_decoder.dart` / `citizenchain/node/src/ui/` / `wuminapp/lib/`)新增或修改 action / field key,**必须先改本文件,再改代码**。CI 门禁扫本文件 vs 三端实现。

## 一、Action 清单(按 pallet 分组)

### 1.1 Balances(pallet_index = 2)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `transfer` | 3 | `transfer_keep_alive` | `to`, `amount_yuan` | node_ui, wuminapp |

### 1.2 VotingEngine(pallet_index = 9)

| action | call_index | call | fields | 签发方 |
|---|---|---|---|---|
| `internal_vote` | 0 | `internal_vote` | `proposal_id`, `approve` | node_ui, wuminapp |
| `joint_vote` | 1 | `joint_vote` | `proposal_id`, `approve` | node_ui, wuminapp |
| `citizen_vote` | 2 | `citizen_vote` | `proposal_id`, `binding_id`, `nonce`, `approve` | wuminapp |
| `finalize_proposal` | 3 | `finalize_proposal` | `proposal_id` | node_ui, wuminapp |

### 1.3 AdminsChange(pallet_index = 12)

| action | call_index | call | fields | 签发方 |
|---|---|---|---|---|
| `propose_admin_replacement` | 0 | `propose_admin_replacement` | `org`, `old_admin`, `new_admin` | node_ui, wuminapp |
| `execute_admin_replacement` | 1 | `execute_admin_replacement` | `proposal_id` | node_ui, wuminapp |

### 1.4 RuntimeUpgrade(pallet_index = 13)

| action | call_index | call | fields | 签发方 |
|---|---|---|---|---|
| `propose_runtime_upgrade` | 0 | `propose_runtime_upgrade` | `reason`, `wasm_size`, `wasm_hash`, `eligible_total` | node_ui, wuminapp |
| `developer_direct_upgrade` | 2 | `developer_direct_upgrade` | `wasm_size`, `wasm_hash` | node_ui, wuminapp |

### 1.5 ResolutionDestro(pallet_index = 14)

| action | call_index | call | fields | 签发方 |
|---|---|---|---|---|
| `propose_destroy` | 0 | `propose_destroy` | `org`, `amount_yuan` | wuminapp |
| `execute_destroy` | 1 | `execute_destroy` | `proposal_id` | wuminapp |

### 1.6 GrandpaKeyChange(pallet_index = 16)

| action | call_index | call | fields | 签发方 |
|---|---|---|---|---|
| `propose_replace_grandpa_key` | 0 | `propose_replace_grandpa_key` | `institution`, `new_key` | node_ui |
| `execute_replace_grandpa_key` | 1 | `execute_replace_grandpa_key` | `proposal_id` | node_ui |
| `cancel_failed_replace_grandpa_key` | 2 | `cancel_failed_replace_grandpa_key` | `proposal_id` | node_ui |

### 1.7 DuoqianManagePow(pallet_index = 17)

| action | call_index | call | fields | 签发方 |
|---|---|---|---|---|
| `propose_create` | 0 | `propose_create` | `sfid_id`, `account_name`, `admin_count`, `threshold`, `amount_yuan` | wuminapp |
| `propose_close` | 1 | `propose_close` | `duoqian_address`, `beneficiary` | wuminapp |
| `propose_create_personal` | 3 | `propose_create_personal` | `account_name`, `admin_count`, `threshold`, `amount_yuan` | wuminapp |
| `cleanup_rejected_proposal` | 4 | `cleanup_rejected_proposal` | `proposal_id` | wuminapp |

`register_sfid_institution`(call_index=2)由 sfid 后端 ShengSigningPubkey 直签,不走冷钱包 ⇒ **不在本表范围**。

### 1.8 DuoqianTransferPow(pallet_index = 19)

| action | call_index | call | fields | 签发方 |
|---|---|---|---|---|
| `propose_transfer` | 0 | `propose_transfer` | `org`, `beneficiary`, `amount_yuan`, `remark` | node_ui, wuminapp |
| `propose_safety_fund_transfer` | 1 | `propose_safety_fund_transfer` | `beneficiary`, `amount_yuan`, `remark` | node_ui, wuminapp |
| `propose_sweep_to_main` | 2 | `propose_sweep_to_main` | `institution`, `amount_yuan` | node_ui, wuminapp |
| `execute_transfer` | 3 | `execute_transfer` | `proposal_id` | wuminapp |
| `execute_safety_fund_transfer` | 4 | `execute_safety_fund_transfer` | `proposal_id` | wuminapp |
| `execute_sweep_to_main` | 5 | `execute_sweep_to_main` | `proposal_id` | wuminapp |

### 1.9 链下数据(`sign_request` 载体,非链上 extrinsic)

| action | payload 结构 | fields | 签发方 | 用途 |
|---|---|---|---|---|
| `activate_admin` | `GMB_ACTIVATE`(12B) + `shenfen_id`(48B 右补零) + `timestamp`(8B u64) + `nonce`(16B) = 84B | `shenfen_id` | node_ui(`citizenchain/node/src/ui/governance/activation.rs`) / sfid 后端 | 管理员激活,sfid 后端验签。decoder 检测 `GMB_ACTIVATE` 前缀走专用分支。 |

## 二、字段渲染规则

| key | 类型 | 渲染格式 |
|---|---|---|
| `to` / `beneficiary` / `duoqian_address` / `old_admin` / `new_admin` | `AccountId32` | SS58,prefix = 2027 |
| `amount_yuan` | `Balance` u128,decimals = 10 | `"X.XX GMB"`(整数位不截断,小数位 2 位) |
| `proposal_id` / `binding_id` / `nonce` | u64 | 十进制字符串 |
| `approve` | bool | `"true"` / `"false"` |
| `org` | u32 `ORG_CODE` | 机构名(查 `duoqian/institutions.rs` 表,找不到回退为 `"u32(<raw>)"`) |
| `institution` | 48B shenfen_id(右补零 UTF-8) | decoder 解出 shenfen_id 后查 `chain/institutions.dart` 表转为机构中文名(找不到回退原 shenfen_id 字符串)。`propose_sweep_to_main` / `propose_replace_grandpa_key` 使用。 |
| `institution_id` | **当前 decoder 跳过不回填** | 后续 UX 跟进:`joint_vote` payload 包含 48B 机构 id,冷钱包现只展示 `proposal_id` / `approve`,用户无法确认"投哪个机构身份"。见跟进任务卡 `20260422-joint-vote-institution-id-display.md`(待建)。 |
| `wasm_size` | u32 字节 | `"X.XX MB"`(> 1 MB)或 `"X KB"` |
| `wasm_hash` | **sha256** (`[u8;32]`) of wasm bytes | `"0x<64hex>"` 小写。三端共用 sha256 算法:节点 Tauri UI 的 `sha256_hash(&wasm_code)`、冷钱包 `package:crypto` 的 `sha256.convert(bytes)`。不是 blake2_256。 |
| `eligible_total` | u64 | 十进制字符串。`propose_runtime_upgrade` 独有,对应链端 `eligible_total: u64` 参数。 |
| `reason` / `remark` / `account_name` | UTF-8 bytes | 字符串(UI 超长截断加省略号,签名原文不截断) |
| `sfid_id` | `[u8;32]` 或 UTF-8 | 按具体业务实现 |
| `admin_count` / `threshold` | u8 | 十进制字符串 |
| `shenfen_id` | 48 字节 UTF-8 右补零 | 右 trim 零后的 UTF-8 字符串 |
| `new_key` | `[u8;32]` ed25519 pubkey | `"0x<64hex>"` 小写 |

## 三、字段约束

1. **key 字面常量**:三端的 key 字符串必须与本表第一列**逐字相等**,禁止大小写变体、驼峰、别名、`a ?? b` 回退。
2. **顺序**:本表 fields 列的顺序 = decoder 输出顺序 = UI 展示顺序。偏离视为 bug。
3. **格式**:渲染格式三端一致。示例:`amount_yuan` 必然是 `"X.XX GMB"`(decimals=10 转十进制截到两位小数 + 单位),**绝不是** `"1234567890 Planck"` 或 `"1234567890"` 或 `"0x..."`。
4. **签发方**:本表明确标注的签发方才允许发出对应 action 的 `sign_request`。其他签发方(含 sfid/cpms 后端)发出同 action 的 `sign_request` 视为非法,冷钱包红色拒签。

## 四、新增 action 流程

1. 修改本文件,加入新行(pallet_index + call_index + fields + 签发方),**PR 单独提交**
2. 修改 `wumin/lib/signer/payload_decoder.dart` 加对应 decode 分支,返回的 action / fields key **逐字对齐本表**
3. 修改 `wumin/lib/signer/pallet_registry.dart` 补 pallet_index / call_index 常量
4. 修改签发方代码(`citizenchain/node/src/ui/` 或 `wuminapp/lib/`),`display.action` + `SignDisplayField.key` 逐字对齐
5. 补 `wumin/test/` 单测 + `memory/05-architecture/qr-protocol-fixtures/` golden fixture
6. 端到端 Flutter / Rust 测试覆盖
7. PR 标题包含 `[qr-registry]`,CI 门禁扫本文件 vs 三端
