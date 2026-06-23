# ADR-026 全仓签名协议统一为单一原语 GMB+op_tag

- 状态：**Phase 1 + Phase 2 完成并验证（2026-06-22，行为按设计变更，运行时签名非创世状态，落同次重新创世前，无迁移）。** 全仓 `GMB_*_V1` 生产残留=0;签名 op_tag/signing_message 单源 primitives::sign;治理 5 个(0x10-0x14)字节零变化(后端 golden 137304f0 不变);8 哈希域 Rust↔Dart 金标逐字节对齐;ACTIVATE/DECRYPT 四方逐字节一致。未提交(待用户授权)。
  - Phase 1:primitives::sign 原语 + op_tag 注册表 + 金标;治理 5(0x10-0x14)字节不变;3 哈希域 L3_PAY(0x15)/OFFCHAIN_BATCH(0x16)/L2_ACK(0x17) 折好。
  - Phase 2:ACTIVATE_ADMIN(0x18)/DECRYPT(0x19) 内嵌前缀 → `GMB||op_tag` 4B 二进制前缀(原始字节可解析保留,payload 97B/108B,node 构造/验签/冷钱包/citizenapp 四方 + 金标 fixture 逐字节一致);IM_WALLET_BINDING/IM_NODE_PAIRING 版本串集中 primitives::sign 单源(原构造不变,不占签名 op_tag)。
  - 验证:链端 cargo check --workspace + primitives/node/offchain-transaction test;后端 77(golden 不变);citizenapp signer/trade + citizenwallet payload_decoder flutter test 全绿。
- **4 域裁决（2026-06-22 用户拍板"抖中方案"）**:这 4 个不是 `blake2_256(域||SCALE)` 哈希域,**不强折成 hash**(会丢原始字节可解析性):
  - `ACTIVATE_ADMIN(0x18)` / `DECRYPT(0x19)`:签**原始可解析字节**,域是内嵌前缀,统一为 **`GMB || op_tag`(4B)二进制前缀**,保留原始字节签名 + 按偏移解析(冷钱包/node/citizenapp 锁步;DECRYPT 的 CHALLENGE_TOTAL_LEN + sha256 完整性字段同步)。
  - `IM_WALLET_BINDING(0x1B)`:管道分隔 UTF-8 字符串 canonical,**保留原构造**,仅把版本串常量集中进 primitives 单源。
  - `IM_NODE_PAIRING(0x1A)`:QR body 协议版本字符串,**不签名**,仅常量集中单源(不是 signing_message op_tag)。
  - 注册表收口:0x10-0x19 为签名域(hash 或二进制前缀);IM 两个改为集中的字符串常量,不占签名 op_tag(撤 0x1A/0x1B 悬空或标注为字符串域 id)。
- 关联：[[ADR-024]] 账户派生单源（同 `GMB` 域 + op_tag 思想，本 ADR 把签名侧也收敛）；末尾随 T3/T4 / ADR-024 同一次重新创世生效
- 取代：7 个散落的 `b"GMB_<NAME>_V1"` 字符串签名域

## 背景（问题）

仓库签名协议**两套范式并存 + 字符串域散落重复**：
- 范式 A（已统一）：`GMB`(3B) + 1 字节 op_tag 子命名空间。账户派生 0x00-0x06 + 身份/治理签名 `OP_SIGN_*` 0x10-0x14，集中在 primitives。
- 范式 B（散落）：7 个独立 `b"GMB_<NAME>_V1"` 字符串域，各自在 feature 模块本地定义 + 跨 runtime/node/backend/dart 重复 2-5 份，primitives 零集中定义：

| 字符串域 | 用途 | 重复处 |
|---|---|---|
| GMB_L3_PAY_V1 | L3 支付 | runtime batch_item + node ledger + dart×2 + test（5） |
| GMB_OFFCHAIN_BATCH_V1 | 批次结算 | runtime batch_item + node packer + node signer（3） |
| GMB_L2_ACK_V1 | L2 确认 | node rpc（1） |
| GMB \|\| 0x18 | 管理员激活 | node activation + dart + 冷钱包 + test（4） |
| GMB \|\| 0x19 | 解密授权 | node admin_unlock + 冷钱包 + test（3） |
| GMB_IM_NODE_PAIRING_V1 | IM 节点配对 | node + dart（2） |
| GMB_IM_WALLET_BINDING_V1 | IM 钱包绑定 | node + dart×2（3） |

另：冷钱包 `'cid_admin_governance'`（非 GMB_ 格式 QR 域，命名不统一，待并入或正名）。

病根同 isForbidden 漂移：同域多份 copy，改一处忘改另一处 = 创世后**验签静默失败**。

## 决策

**全仓一切签名消息 = 唯一原语 `primitives::sign::signing_message`，一个 `GMB` 域 + 一张 op_tag 注册表。**

### 统一消息构造（两范式可无缝归一的字节证明）
SCALE 元组 = 各元素 SCALE 拼接，故 `SCALE((GMB, op_tag, f1, f2))` 逐字节 == `GMB || op_tag || SCALE((f1,f2))`。定义：
```rust
// primitives::sign
pub fn signing_message(op_tag: u8, scale_payload: &[u8]) -> [u8; 32] {
    let mut data = Vec::with_capacity(GMB.len() + 1 + scale_payload.len());
    data.extend_from_slice(GMB);          // core_const::GMB (3B)
    data.push(op_tag);                    // 1B 子命名空间
    data.extend_from_slice(scale_payload);// 调用方 SCALE 编码的 payload 字段
    blake2_256(&data)
}
```
- 治理 5 个（0x10-0x14）改调 `signing_message(OP_SIGN_X, (fields).encode())`：**逐字节不变 → 签名不变**。
- 7 个字符串域改 op_tag：`blake2_256(b"GMB_L3_PAY_V1" || SCALE)` → `signing_message(OP_SIGN_L3_PAY, SCALE)`，**签名字节变**（域前缀 13B→4B），结构归一。

### op_tag 注册表（签名段 0x10-0x1F，单一真源 primitives::sign）
0x10 BIND / 0x11 VOTE / 0x12 POP / 0x13 INST / 0x14 DEREGISTER（不变）
0x15 L3_PAY / 0x16 OFFCHAIN_BATCH / 0x17 L2_ACK / 0x18 ACTIVATE_ADMIN / 0x19 DECRYPT / 0x1A IM_NODE_PAIRING / 0x1B IM_WALLET_BINDING（新，取代字符串域）

### 单源纪律
- `primitives::sign` 持 `signing_message` + 全部 `OP_SIGN_*`；删 7 个字符串域常量（batch_item/ledger/packer/signer/rpc/activation/admin_unlock/communication-node/im::binding）。
- runtime/node/backend 全 import 调用；删本地 concat。
- Dart（citizenapp + citizenwallet）手写镜像，靠**金标向量**（signing_message(op_tag,payload)→hash）逐字节断言对齐，CI sync 防漂移（类比 account_derive 金标）。
- 冷钱包 `cid_admin_governance` QR 域评估并入/正名。

## 范围 / 破坏性
- 破坏式：7 协议签名字节变 → runtime 验签 + node 签/验 + backend + 热钱包(L3_PAY) + 冷钱包(ACTIVATE_ADMIN/DECRYPT) 必须**锁步**。
- 但都是**运行时签名、非创世状态**（不动账户/余额），系统正重新创世 → 落创世前即可，**无需迁移**。治理 5 个字节不变（零风险)。

## 实施顺序（PR / workflow 阶段）
1. **primitives::sign 新模块** + op_tag 注册表 + signing_message;`cargo check -p primitives`。
2. **runtime 迁移**：configs OP_SIGN 5 处改调（验字节不变）+ offchain-transaction batch_item L3_PAY/BATCH 改 op_tag；删本地域常量。
3. **node 迁移**：ledger/packer/signer/rpc/activation/admin_unlock/communication-node/im::binding 全改调 + 删重复。
4. **backend 迁移**：chain_runtime OP_SIGN 路径核对（治理字节不变）。
5. **Dart 迁移**：citizenapp(payment_intent/im/admin_activation) + citizenwallet(payload_decoder activate-admin/decrypt) 改 op_tag 镜像 + 金标。
6. **金标 + 验证**：Rust 导出 signing golden（op_tag→hash 向量）+ Dart 断言；全量编译/测试/签名 golden + 残留=0。

## 验收
- 全仓 `GMB_*_V1` 字符串域残留=0;签名域常量仅存 primitives::sign;治理 5 个签名 golden 逐字节不变（回归证明非破坏治理侧）；7 协议新 op_tag 签名 Rust↔Dart 金标逐字节对齐;链端/node/backend/双钱包全绿。
