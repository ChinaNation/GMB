# DUOQIAN_TECHNICAL

模块：`duoqian-transaction-pow`  
范围：SFID 先登记、后创建/注销的多签账户流程（当前实现）

## 0. 功能需求

### 0.1 核心职责
- 提供机构多签地址的链上登记、创建、注销三类能力。
- 强制采用“SFID 先登记，再创建多签账户”的流程，不允许前端手填多签地址。
- 对创建和注销操作执行管理员多签校验，并要求提交者本人属于管理员集合。

### 0.2 地址与域隔离需求
- `duoqian_address` 必须由链上按 `BLAKE2b("DUOQIAN_SFID_V1" || ss58_prefix_le || sfid_id)` 派生。
- `ss58_prefix_le` 为 SS58 前缀 2027 的小端 u16 字节，用于链域隔离。
- 派生地址必须通过地址合法性校验，且不能落入制度保留地址或受保护地址集合。

### 0.3 创建流程需求
- `create_duoqian` 必须只接受已登记的 `sfid_id`，并从登记映射中解析目标地址。
- 管理员数量必须 `>= 2`，且 `duoqian_admins.len() == admin_count`。
- 阈值必须满足 `ceil(admin_count / 2) <= threshold <= admin_count`，同时最小不少于 2。
- 管理员公钥必须逐个校验格式，且不能重复。
- 创建金额必须 `>= MinCreateAmount`。
- 发起人必须是管理员之一，且有效管理员签名数必须 `>= threshold`。
- 创建成功后必须完成入金、写入多签配置并递增机构 nonce。

### 0.4 注销流程需求
- `close_duoqian` 必须只作用于已存在的多签账户。
- 提交者必须是该多签账户管理员之一，且有效管理员签名数必须 `>= threshold`。
- 多签账户余额必须同时满足 `free_balance >= MinCloseBalance` 和 `free_balance >= min_balance`。
- 多签账户存在保留余额时必须拒绝注销。
- `beneficiary` 不能等于多签地址自身，且不能是保留地址、非法地址或受保护地址。
- 注销成功后必须全额转出余额、删除多签配置并递增机构 nonce。

### 0.5 防重放与签名需求
- 创建和注销必须共用同一个机构 nonce。
- 签名 payload 必须绑定：操作类型版本、SS58 前缀、nonce、过期高度、提交者 `who` 以及关键业务参数。
- `expires_at` 过期后必须拒绝交易，旧签名在 nonce 变化后必须无法重放。

### 0.6 存储与制度边界
- 链上必须维护 `sfid_id -> duoqian_address` 和 `duoqian_address -> { sfid_id, nonce }` 双向映射。
- 按当前制度，SFID 登记映射不在本模块内解绑；注销多签账户只删除多签配置，不删除 SFID 登记。
- 模块只负责机构多签交易制度与安全校验，不负责 SFID 系统外部的解绑、迁移与前端协同流程。

### 0.7 可观测性与性能需求
- 创建和注销事件必须记录关键账户、管理员规模和金额，便于审计追踪。
- 模块必须提供 benchmark 入口与权重接口，供 runtime 后续收敛到自动生成权重。

## 1. 目标

1. SFID 系统先在链上登记 `sfid_id`。
2. 链上从 `sfid_id` 派生唯一 `duoqian_address`。
3. `create_duoqian` 只接受 `sfid_id`，不接受前端手填地址。
4. 创建和注销使用管理员链下签名，并带 nonce 防重放。

## 2. 地址派生与链域

地址派生公式（当前）：

`duoqian_address = Blake2b256("DUOQIAN_SFID_V1" || ss58_prefix_le || sfid_id_bytes)`

说明：
1. `ss58_prefix_le` 为 SS58 前缀 2027 的小端 u16 字节（`[0xEB, 0x07]`），用于链域隔离。
2. 该派生逻辑是兼容性关键规则，已部署链上不可随意变更。

## 3. 链上存储

1. `SfidRegisteredAddress<sfid_id, duoqian_address>`
2. `AddressRegisteredSfid<duoqian_address, RegisteredInstitution { sfid_id, nonce }>`
3. `DuoqianAccounts<duoqian_address, DuoqianAccount>`
4. `StorageVersion = 1`

补充：
1. `nonce` 跟随 `duoqian_address` 存储，用于 create/close 统一防重放。
2. `SfidRegisteredAddress`/`AddressRegisteredSfid` 按当前制度设计不在本链解绑，解绑由 SFID 系统侧完成。
3. 链域隔离使用 SS58 前缀 2027 的小端 u16 字节（`ss58_prefix_le`），不再使用 genesis hash。

## 4. Extrinsic 规则

### 4.1 register_sfid_institution(sfid_id, register_nonce, signature)

校验：
1. `sfid_id` 非空。
2. `register_nonce` 未被消费。
3. `signature` 必须能通过 `("GMB_SFID_INSTITUTION_V1", genesis_hash, sfid_id, register_nonce)` 验签，且只认当前 SFID `MAIN`。
4. `sfid_id` 未登记。
5. 派生地址未登记、非保留地址、非受保护地址、地址格式合法。

执行：
1. 写入双向映射。
2. 记录 `register_nonce` 已消费。
3. 发出 `SfidInstitutionRegistered`。
4. 初始化该地址 nonce 为 `0`。

### 4.2 create_duoqian(sfid_id, admin_count, duoqian_admins, threshold, amount, expires_at, approvals)

关键校验：
1. `MaxAdmins >= 2`（运行时配置防御）。
2. 调用者 `who` 非受保护源。
3. `now <= expires_at`。
4. `admin_count >= 2` 且 `duoqian_admins.len() == admin_count`。
5. `threshold` 满足 `ceil(admin_count/2) <= threshold <= admin_count`（且最小至少 2）。
6. `amount >= MinCreateAmount`。
7. `sfid_id` 已登记，且反向映射一致。
8. `duoqian_address` 非保留地址、非受保护地址，且未被创建为 `DuoqianAccounts`。
9. 调用者必须是管理员之一。
10. `nonce < u64::MAX`（提前失败，避免签名验证浪费）。
11. 签名通过且有效管理员签名数 `>= threshold`。

Create 签名 payload（当前版本）：

`"DUOQIAN_CREATE_V3", ss58_prefix_le, nonce, expires_at, sfid_id, duoqian_address, who, admin_count, admins, threshold, amount`

执行后：
1. 转账 `who -> duoqian_address`。
2. 写入 `DuoqianAccounts`。
3. `nonce += 1`（`checked_add`）。
4. 发出 `DuoqianCreated`。

### 4.3 close_duoqian(duoqian_address, beneficiary, min_balance, expires_at, approvals)

关键校验：
1. `now <= expires_at`。
2. `duoqian_address` 不是受保护源。
3. `beneficiary`：不等于自身、非保留地址、地址合法、非受保护地址。
4. `duoqian_address` 必须已存在 `DuoqianAccounts`。
5. 调用者必须是管理员之一。
6. `free_balance >= MinCloseBalance`。
7. `free_balance >= min_balance`。
8. `reserved_balance == 0`。
9. `nonce < u64::MAX`。
10. 签名通过且有效管理员签名数 `>= threshold`。

Close 签名 payload（当前版本）：

`"DUOQIAN_CLOSE_V3", ss58_prefix_le, nonce, expires_at, duoqian_address, beneficiary, who, admins, admin_count, threshold, min_balance`

执行后：
1. `duoqian_address` 全额转出至 `beneficiary`（`AllowDeath`）。
2. 删除 `DuoqianAccounts`。
3. `nonce += 1`（`checked_add`）。
4. 返回实际 `PostDispatch` weight（按真实 `admin_count` 退款）。
5. 发出 `DuoqianClosed`。

## 4.4 投票引擎回调与 STATUS_EXECUTED

`create_duoqian` 和 `close_duoqian` 提交后进入投票引擎流程。投票通过后，投票引擎回调本模块执行 `execute_create` 或 `execute_close`。

执行成功后，本模块调用 `voting_engine_system::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_EXECUTED)` 将投票引擎侧的提案状态标记为已执行，防止同一提案被重复执行。

提案状态流转：`VOTING → PASSED → EXECUTED`

说明：
- `PASSED` 由投票引擎在投票通过时设置。
- `EXECUTED` 由本模块在 `execute_create`/`execute_close` 成功后通过 `set_status_and_emit` 设置。
- 一旦提案进入 `EXECUTED`，投票引擎不会再次触发执行回调，从而保证幂等性。

## 5. 错误码（重点）

1. 注册相关：`UnauthorizedSfidRegistrar`、`SfidAlreadyRegistered`、`EmptySfidId`、`DerivedAddressDecodeFailed`
2. 地址相关：`InvalidAddress`、`AddressReserved`、`AddressAlreadyExists`
3. 参数相关：`InvalidAdminCount`、`AdminCountMismatch`、`InvalidThreshold`、`DuplicatePublicKey`
4. 权限签名：`PermissionDenied`、`InvalidAdminPublicKey`、`InvalidAdminSignature`、`InsufficientSignatures`、`SignatureExpired`
5. 余额相关（细分）：`CreateAmountBelowMinimum`、`CloseBalanceBelowMinimum`、`CloseBalanceBelowRequested`、`ReservedBalanceRemaining`
6. 域与防重放：`NonceOverflow`
7. 运行时防御：`InvalidRuntimeConfig`

## 6. 安全属性

1. 地址不可前端伪造：必须先登记再创建。
2. 链域隔离：签名域与地址派生都绑定 `ss58_prefix_le`。
3. 操作防重放：create/close 共用 nonce，且每次成功后递增。
4. 提交者绑定：create/close payload 都绑定 `who`。
5. 审计可追踪：事件 + 映射 + payload 域分离。

## 7. 前端接入要求

1. 仅输入 `sfid_id`，不允许手输 `duoqian_address`。
2. 先查询 `sfidRegisteredAddress(sfid_id)`，无登记则禁止创建。
3. 创建签名和注销签名时，payload 必须使用当前版本：
   1. create：`DUOQIAN_CREATE_V3`
   2. close：`DUOQIAN_CLOSE_V3`
4. payload 中必须包含提交者 `who` 与 `expires_at`。
5. 提交前应再次读取链上 nonce，避免离线签名过期/失配。
