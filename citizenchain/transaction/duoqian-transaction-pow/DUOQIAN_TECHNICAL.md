# DUOQIAN_TECHNICAL

模块：`duoqian-transaction-pow`  
范围：SFID 先登记、后创建/注销的多签账户流程（当前实现）

## 1. 目标

1. SFID 系统先在链上登记 `sfid_id`。
2. 链上从 `sfid_id` 派生唯一 `duoqian_address`。
3. `create_duoqian` 只接受 `sfid_id`，不接受前端手填地址。
4. 创建和注销使用管理员链下签名，并带 nonce 防重放。

## 2. 地址派生与链域

地址派生公式（当前）：

`duoqian_address = BLAKE3("DUOQIAN_SFID_V1" || chain_domain_hash || sfid_id_bytes)`

说明：
1. `chain_domain_hash` 持久化存储在 `ChainDomainHash`，用于链域隔离。
2. 派生函数前置条件：`ChainDomainHash` 已初始化；否则返回 `ChainDomainHashUnavailable`。
3. 该派生逻辑是兼容性关键规则，已部署链上不可随意变更。

## 3. 链上存储

1. `SfidRegisteredAddress<sfid_id, duoqian_address>`
2. `AddressRegisteredSfid<duoqian_address, RegisteredInstitution { sfid_id, nonce }>`
3. `DuoqianAccounts<duoqian_address, DuoqianAccount>`
4. `ChainDomainHash<Option<Hash>>`
5. `StorageVersion = 1`

补充：
1. `nonce` 跟随 `duoqian_address` 存储，用于 create/close 统一防重放。
2. `SfidRegisteredAddress`/`AddressRegisteredSfid` 按当前制度设计不在本链解绑，解绑由 SFID 系统侧完成。

## 4. Extrinsic 规则

### 4.1 register_sfid_institution(sfid_id)

校验：
1. `sfid_id` 非空。
2. 调用者是 SFID 授权操作员。
3. `sfid_id` 未登记。
4. 派生地址未登记、非保留地址、地址格式合法。

执行：
1. 写入双向映射。
2. 初始化该地址 nonce 为 `0`。
3. 发出 `SfidInstitutionRegistered`。

### 4.2 create_duoqian(sfid_id, admin_count, duoqian_admins, threshold, amount, expires_at, approvals)

关键校验：
1. `MaxAdmins >= 2`（运行时配置防御）。
2. 调用者 `who` 非受保护源。
3. `now <= expires_at`。
4. `admin_count >= 2` 且 `duoqian_admins.len() == admin_count`。
5. `threshold` 满足 `ceil(admin_count/2) <= threshold <= admin_count`（且最小至少 2）。
6. `amount >= MinCreateAmount`。
7. `sfid_id` 已登记，且反向映射一致。
8. `duoqian_address` 未被创建为 `DuoqianAccounts`。
9. 调用者必须是管理员之一。
10. `nonce < u64::MAX`（提前失败，避免签名验证浪费）。
11. 签名通过且有效管理员签名数 `>= threshold`。

Create 签名 payload（当前版本）：

`"DUOQIAN_CREATE_V3", domain_hash, nonce, expires_at, sfid_id, duoqian_address, who, admin_count, admins, threshold, amount`

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

`"DUOQIAN_CLOSE_V3", domain_hash, nonce, expires_at, duoqian_address, beneficiary, who, admins, admin_count, threshold, min_balance`

执行后：
1. `duoqian_address` 全额转出至 `beneficiary`（`AllowDeath`）。
2. 删除 `DuoqianAccounts`。
3. `nonce += 1`（`checked_add`）。
4. 返回实际 `PostDispatch` weight（按真实 `admin_count` 退款）。
5. 发出 `DuoqianClosed`。

## 5. 错误码（重点）

1. 注册相关：`UnauthorizedSfidRegistrar`、`SfidAlreadyRegistered`、`EmptySfidId`、`DerivedAddressDecodeFailed`
2. 地址相关：`InvalidAddress`、`AddressReserved`、`AddressAlreadyExists`
3. 参数相关：`InvalidAdminCount`、`AdminCountMismatch`、`InvalidThreshold`、`DuplicatePublicKey`
4. 权限签名：`PermissionDenied`、`InvalidAdminPublicKey`、`InvalidAdminSignature`、`InsufficientSignatures`、`SignatureExpired`
5. 余额相关（细分）：`CreateAmountBelowMinimum`、`CloseBalanceBelowMinimum`、`CloseBalanceBelowRequested`、`ReservedBalanceRemaining`
6. 域与防重放：`ChainDomainHashUnavailable`、`NonceOverflow`
7. 运行时防御：`InvalidRuntimeConfig`

## 6. 安全属性

1. 地址不可前端伪造：必须先登记再创建。
2. 链域隔离：签名域与地址派生都绑定 `chain_domain_hash`。
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
5. 提交前应再次读取链上 nonce 与域哈希，避免离线签名过期/失配。
