# 任务卡：投票账户绑定/解绑 完整流程

- 创建日期：2026-04-11
- 状态：open / step3-done
- 最后更新：2026-04-11
- 负责入口：GMB 主聊天

## 一、背景

wuminapp 用户资料页有"投票账户"设置项，用于将钱包账户绑定到 SFID 公民身份系统并写入区块链。当前代码存在严重断裂：

- wuminapp 调用的 `/api/v1/app/bind/request` 接口已被 SFID 后端删除
- 选择钱包后不签名直接发 pubkey，无法证明私钥所有权
- SFID 后端 `citizen_bind()` 验签存库后不推链
- wuminapp 没有监听链上确认状态
- SFID 号与档案号未实现永久绑定

## 二、完整流程

### 绑定流程（6 步）

```
Step 1: wuminapp 选择钱包 → 签名证明私钥所有权 → 推送 pubkey 到 SFID
        热钱包：本机签名
        冷钱包：生成 sign_request → wumin 签名 → wuminapp 扫回执获取签名
        状态：未设置 → 待绑定

Step 2: SFID 收到 pubkey → 创建 CitizenRecord（只有 pubkey）→ 公民列表显示"待绑定"

Step 3: 用户到 SFID 现场 → 管理员扫 CPMS 档案码（QR4）
        首次绑定：扫 QR4 → 获取档案号 + 省份 → 生成 SFID 号 → 档案号与 SFID 号永久绑定
        重新绑定：管理员输入档案号或 SFID 号查到已有记录 → 跳过 QR4

Step 4: SFID 生成 WUMIN_QR_V1 挑战码 → 用户钱包签名
        热钱包：wuminapp 投票账户"待绑定"旁的【签名】按钮 → 扫码签名 → 展示回执码
        冷钱包：wumin 扫码签名 → 展示回执码
        SFID 扫回执码验签

Step 5: 市管理员点"绑定" → SFID 后端用本省省级签名密钥构造 bind_sfid extrinsic → 提交区块链
        参数：binding_id = blake2_256(archive_no), bind_nonce = UUID, signature = 省级密钥签名

Step 6: 链上铸块确认 → SFID InBestBlock 回调更新记录为 Bound
        wuminapp 轮询 SFID 状态接口 → 检测到 Bound → 更新显示"已绑定"
```

### 解绑流程（4 步）

```
Step A: SFID 管理员点"解绑" → 生成 WUMIN_QR_V1 挑战码

Step B: 用户钱包签名（同 Step 4 方式，热/冷钱包各走各的路径）

Step C: 管理员点"确认解绑" → SFID 用省级密钥构造 unbind_sfid(target) extrinsic → 提交区块链
        链上 unbind_sfid 已改造为管理员专用，仅 SFID 主账户/省级账户可调用

Step D: 链上确认 → SFID 清空 CitizenRecord 的 account_pubkey（保留 sfid_code + archive_no）
        wuminapp 轮询检测到解绑 → 更新显示
```

### 重新绑定流程

```
管理员在公民列表搜索档案号或 SFID 号 → 找到已解绑记录（有 sfid_code + archive_no，无 pubkey）
→ 点"重新绑定" → 用户推送新钱包 pubkey → 管理员绑定新钱包 → 签名 → 推链
不需要 QR4，因为档案号和 SFID 号已永久存储
```

## 三、数据模型

### CitizenRecord（改动后）

```rust
pub(crate) struct CitizenRecord {
    pub(crate) id: u64,
    pub(crate) account_pubkey: Option<String>,    // 绑定的钱包公钥，解绑时清空
    pub(crate) account_address: Option<String>,   // 新增：SS58 地址，方便显示
    pub(crate) archive_no: Option<String>,        // QR4 扫码获取，永久保留
    pub(crate) sfid_code: Option<String>,         // 首次绑定时生成，永久保留
    pub(crate) sfid_signature: Option<String>,
    pub(crate) province_code: Option<String>,     // QR4 获取，永久保留
    pub(crate) chain_confirmed: bool,             // 新增：链上是否已确认
    pub(crate) bound_at: Option<DateTime<Utc>>,
    pub(crate) bound_by: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
}
```

### 状态枚举（改动后）

```rust
pub(crate) enum CitizenBindStatus {
    Pending,       // 有 pubkey，无 archive_no（用户推送了钱包，未到现场）
    Bindable,      // 有 pubkey + archive_no + 签名验证通过，待管理员推链
    Bound,         // chain_confirmed = true
    Unlinked,      // 解绑后：有 archive_no + sfid_code，无 pubkey
}
```

### 反向索引（新增）

```rust
pub(crate) citizen_id_by_sfid_code: HashMap<String, u64>,  // 新增：sfid_code → id
```

### wuminapp 本地状态

```dart
enum VoteAccountStatus {
  unset,     // 未设置：无任何记录
  pending,   // 待绑定：已推送 pubkey 到 SFID，等待现场绑定
  bound,     // 已绑定：链上已确认
}

class VoteAccountState {
  final VoteAccountStatus status;
  final String? walletAddress;
  final String? walletPubkeyHex;
  final bool isColdWallet;      // 标记冷热钱包，决定是否显示【签名】按钮
}
```

## 四、接口设计

### wuminapp → SFID（新增 2 个接口）

#### 4.1 推送投票账户

```
POST /api/v1/app/vote-account/register
Body: {
  "address": "5Gx...",             // SS58 地址
  "pubkey": "0x...",               // 公钥 hex
  "signature": "0x...",            // sr25519 签名 hex
  "sign_message": "CITIZEN_VOTE_REGISTER|5Gx...|1712836800"
}
Response: { "code": 0 }
```

SFID 后端验签后创建 CitizenRecord（只有 pubkey，状态 Pending）。

签名消息格式统一：`CITIZEN_VOTE_REGISTER|{SS58地址}|{unix_timestamp}`，用 sr25519 签名。

#### 4.2 查询绑定状态

```
GET /api/v1/app/vote-account/status?pubkey=0x...
Response: {
  "code": 0,
  "status": "pending" | "bound" | "unset",
  "address": "5Gx..." | null,
  "sfid_code": "GMR-GD000-..." | null
}
```

wuminapp 在投票账户页 initState / onResume 时调用，检测状态变化。

### SFID 管理端（改动 + 新增）

#### 4.3 推链绑定（新增）

```
POST /api/v1/admin/citizen/bind/push-chain
Body: {
  "citizen_id": 123
}
Response: {
  "code": 0,
  "tx_hash": "0x..."
}
```

逻辑：
1. 读取 CitizenRecord，确认 pubkey + archive_no + 签名验证都通过
2. `build_bind_credential(state, account_pubkey, archive_no, uuid_nonce)`
   - `binding_id = blake2_256(archive_no.as_bytes())`
   - 省级密钥签名 payload
3. 构造 `SfidCodeAuth::bind_sfid(credential)` extrinsic
4. 参考 `submit_register_sfid_institution_extrinsic` 模式：
   - `resolve_business_signer(state, ctx)` 获取省级 Pair
   - 显式 nonce + immortal + submit_and_watch + 等 InBestBlock
5. 成功后 `chain_confirmed = true`

#### 4.4 推链解绑（新增）

```
POST /api/v1/admin/citizen/unbind/push-chain
Body: {
  "citizen_id": 123,
  "challenge_id": "...",
  "signature": "0x..."
}
Response: {
  "code": 0,
  "tx_hash": "0x..."
}
```

SFID 后端用省级密钥调用链上 `unbind_sfid(target)` 完成解绑。
链上 unbind_sfid 已改造为管理员专用接口。

#### 4.5 公民列表（改动）

```
GET /api/v1/admin/citizens?keyword=...
```

- 搜索改为只按 `account_address`（SS58）匹配，去掉 archive_no 和 sfid_code 搜索
- 返回新增 `account_address` 字段
- 状态显示：Pending="待绑定"、Bindable="待推链"、Bound="已绑定"、Unlinked="已解绑"
- 按省过滤：走 `scope::filter_by_scope`，省管理员只看本省公民

#### 4.6 重新绑定（BindModal 新模式）

当记录状态为 Unlinked（已解绑）时：
- 管理员点"重新绑定"
- 不需要扫 QR4（已有 archive_no + sfid_code）
- 用户推送新钱包 pubkey → 绑定新钱包 → 签名 → 推链

## 五、wuminapp 投票账户 UI

### 状态显示

```
┌─────────────────────────────────────────┐
│ 投票账户                                 │
│                                         │
│ 未设置时：                               │
│   "未设置"（灰色）          [点击设置 >] │
│                                         │
│ 待绑定 + 热钱包：                        │
│   "待绑定"（橙色）  [签名]（绿色按钮）   │
│                                         │
│ 待绑定 + 冷钱包：                        │
│   "待绑定"（橙色）                       │
│                                         │
│ 已绑定：                                 │
│   "已绑定"（绿色）  5Gx...xxx           │
│                                         │
└─────────────────────────────────────────┘
```

### 热钱包【签名】按钮流程

点击 → 打开扫码页（复用摄像头扫码 UI）→ 扫 SFID 屏幕上的 sign_request 挑战码 → 解析为 WUMIN_QR_V1 envelope（kind=sign_request）→ 验证 pubkey 与当前绑定钱包一致 → 用热钱包私钥签名 payload → 构造 sign_response envelope → 展示回执二维码 → SFID 扫回执完成验签。

### 签名页实现

新建 `wuminapp/lib/user/vote_sign_page.dart`：

```dart
class VoteSignPage extends StatefulWidget {
  const VoteSignPage({super.key, required this.wallet});
  final WalletProfile wallet;
}
```

核心逻辑：
1. 打开摄像头扫码
2. `QrEnvelope.parse(raw)` 解析，确认 `kind == signRequest`
3. 验证 `body.pubkey` 与 `wallet.pubkeyHex` 一致
4. `walletManager.signWithWallet(wallet.walletIndex, payloadBytes)` 签名
5. 构造 `QrEnvelope<SignResponseBody>` 回执
6. `QrImageView` 展示回执二维码

## 六、SFID 号永久绑定设计

### 核心规则

- **SFID 号 + 档案号一对一永久绑定**，首次生成后不再改变
- **解绑只清空 account_pubkey**，sfid_code + archive_no + province_code 永久保留
- **重新绑定不需要 QR4**，通过档案号或 SFID 号查到已有记录直接绑新钱包
- **重复扫 QR4**：同一档案号在系统内已存在 → 报错拒绝，不重复生成

### SFID 号生成（不改动）

保持现有 `generate_sfid_code()` 逻辑不变：
- 输入：`account_pubkey + GMR + 省 + 市 + ZG + 日期`
- 输出：`A3-R5-T2P1C1-N9-D8`
- 只在首次扫 QR4 时调用一次，结果永久存入 sfid_registry

### 防重复

```rust
// citizen_bind() 中 bind_archive 模式
if store.citizen_id_by_archive_no.contains_key(&archive_no) {
    return api_error(400, 3001, "该档案号已在系统中，请使用重新绑定功能");
}
```

## 七、链上解绑方案

直接改造现有 `unbind_sfid`（call_index 1）：

- 用户不允许自行解绑
- 仅 SFID 主账户 / 省级签名账户可调用
- 新增 `target` 参数指定被解绑用户

改造后签名：

```rust
#[pallet::call_index(1)]
pub fn unbind_sfid(
    origin: OriginFor<T>,
    target: T::AccountId,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    // 验证 who 是 SFID 主账户或省级签名账户
    // 移除 target 的绑定映射
}
```

## 八、改动文件清单

### wuminapp（4 个文件改动 + 1 个新建）

| 文件 | 改动 |
|---|---|
| `user/user.dart` | 投票账户状态显示重构 + 【签名】按钮 + 状态轮询 |
| `wallet/ui/wallet_page.dart` | 标题"选择绑定钱包"→"设置投票账户" |
| `wallet/capabilities/sfid_binding_service.dart` | 重写：调新接口 + 冷热钱包标记 + 轮询状态 |
| `wallet/capabilities/api_client.dart` | 删 `requestChainBindByPubkey()`，新增 `registerVoteAccount()` + `queryVoteAccountStatus()` |
| **新建** `user/vote_sign_page.dart` | 热钱包扫码签名专用页面 |

### SFID 后端（5 个文件改动）

| 文件 | 改动 |
|---|---|
| `main.rs` | 新增路由：register、status、push-chain |
| `operate/binding.rs` | 新增 `app_vote_account_register()`、`app_vote_account_status()`、`citizen_push_chain()`；`citizen_bind()` 增加档案号重复检查；`citizen_unbind()` 改为只清 pubkey 不删记录 |
| `models/mod.rs` | CitizenRecord 加 `account_address` + `chain_confirmed`；状态枚举改 4 值；新增 `citizen_id_by_sfid_code` 索引 |
| `operate/query.rs` | 公民列表搜索改为只按 account_address 匹配；增加 scope 过滤 |
| `chain/runtime_align.rs` | 新增 `submit_bind_sfid_extrinsic()` 参考 `submit_register_sfid_institution_extrinsic` 模式 |

### SFID 前端（3 个文件改动）

| 文件 | 改动 |
|---|---|
| `views/citizens/CitizensView.tsx` | 状态显示 4 值；搜索只按账户；已绑定和已解绑记录分别显示【解绑】和【重新绑定】按钮 |
| `views/citizens/BindModal.tsx` | 增加"重新绑定"模式（跳过 QR4）；签名完成后增加"推链"确认步骤 |
| `views/citizens/UnbindModal.tsx` | 管理员点解绑 → 生成挑战码 → 用户签名 → 管理员确认 → 调 push-chain 解绑接口 → 链上 unbind_sfid(target) → 清 pubkey |

### citizenchain 链上 pallet（1 个文件改动）

| 文件 | 改动 |
|---|---|
| `citizenchain/runtime/otherpallet/sfid-code-auth/src/lib.rs` | 改造 `unbind_sfid`（call_index 1）：新增 `target` 参数，权限改为仅 SFID 主账户/省级签名账户可调用，禁止用户自行解绑 |

### 不改动

| 模块 | 原因 |
|---|---|
| CPMS | 档案二维码 QR4 不变 |
| wumin 冷钱包 | 离线签名流程已支持 WUMIN_QR_V1 sign_request |
| SFID 号生成逻辑 | `generate_sfid_code()` 不变，只调一次永久存储 |

## 九、执行顺序

1. **链上 pallet**：改造 `unbind_sfid`（call_index 1）为管理员专用 + 更新 benchmarks → runtime upgrade
2. **SFID 后端**：模型改动 → 3 个新接口 → push-chain 绑定/解绑推链逻辑 → 档案号重复检查 → 解绑只清 pubkey
3. **SFID 前端**：状态显示 4 值 → BindModal 重新绑定模式 + 推链步骤 → UnbindModal 改造（管理员发起 → 用户签名 → 确认推链）
4. **wuminapp**：新接口对接 → VoteAccountState 重构 → vote_sign_page 热钱包签名页 → 状态轮询
5. **联调测试**

## 十、测试场景（8 个必须全绿）

1. 热钱包首次绑定全流程：选钱包 → 本机签名推送 → SFID 显示待绑定 → 扫 QR4 → 热钱包现场签名 → 推链 → 链上确认 → 两端显示已绑定
2. 冷钱包首次绑定全流程：同上但签名环节使用 wumin 冷钱包
3. 解绑流程：管理员点解绑 → 生成挑战码 → 用户钱包签名 → 管理员确认 → SFID 用省级密钥调链上 unbind_sfid(target) → 链上确认 → SFID 清 pubkey 保留 sfid_code+archive_no → 两端更新
4. 重新绑定（换钱包）：解绑后 → 管理员搜索 SFID 号 → 用户推新钱包 → 不扫 QR4 → 签名推链 → 绑定成功
5. 重复扫 QR4 拒绝：同一档案号再次扫码 → 报错"该档案号已在系统中"
6. SFID 号永久不变：解绑 + 重新绑定后 SFID 号与首次一致
7. 省份过滤：省管理员只能看到本省公民
8. 所有签名都是 WUMIN_QR_V1 协议格式
