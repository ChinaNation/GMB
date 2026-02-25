# DUOQIAN_TECHNICAL

模块：`duoqian-transaction-pow`  
范围：SFID 先登记后创建的多签账户流程

## 1. 目标

在创建多签账户前，必须先由 SFID 系统在链上登记机构 ID（`sfid_id`），并由链上按固定Blake3算法派生 `duoqian_address`。

只有已登记地址，才允许 `create_duoqian` 创建。

## 2. 核心规则

1. `sfid_id` 是机构唯一标识，由 SFID 系统提供。
2. `duoqian_address` 不由用户随意指定来源，必须来自链上登记映射。
3. 地址派生算法固定为：
   - `duoqian_address = BLAKE3("DUOQIAN_SFID_V1" || sfid_id_bytes)`
4. 同一 `sfid_id` 只能登记一次。
5. 只有 SFID 系统授权账户（主账户或备用账户）可登记。
6. `create_duoqian` 只接收 `sfid_id`，并据此解析已登记的 `duoqian_address`，未登记则拒绝。

## 3. 链上数据结构

- `SfidRegisteredAddress<sfid_id, duoqian_address>`
- `AddressRegisteredSfid<duoqian_address, sfid_id>`
- `DuoqianAccounts<duoqian_address, account_config>`

说明：
- 前两者是“登记证”；
- `DuoqianAccounts` 是真正创建后的多签账户配置。

## 4. 链上交易流程

### 4.1 SFID 登记

调用：`register_sfid_institution(sfid_id)`

校验：
1. `sfid_id` 非空；
2. 调用者为 SFID 授权账户；
3. `sfid_id` 未登记；
4. 派生地址未被占用、非保留地址、地址合法。

执行：
1. 计算派生地址（BLAKE3）；
2. 写入双向映射；
3. 发出 `SfidInstitutionRegistered` 事件。

### 4.2 创建多签

调用：`create_duoqian(sfid_id, admin_count, duoqian_admins, threshold, amount, approvals)`

新增强约束：
- `sfid_id` 必须存在于 `SfidRegisteredAddress`。

其余既有约束保持不变：
- `N >= 2`
- `M >= ceil(N/2)` 且 `M <= N`
- 管理员公钥数量与 `N` 一致且不可重复
- 发起人必须是管理员之一
- 有效管理员签名数 `>= M`
- 创建金额 `>= 1.11`

## 5. 错误码

新增错误：
- `InstitutionNotRegistered`：地址未登记，不允许创建
- `UnauthorizedSfidRegistrar`：非 SFID 授权账户登记
- `SfidAlreadyRegistered`：同一 `sfid_id` 重复登记
- `EmptySfidId`：`sfid_id` 为空
- `DerivedAddressDecodeFailed`：派生地址转账户失败

## 6. 安全收益

1. 防抢注：未登记地址无法创建。
2. 可审计：每个多签地址可追溯到唯一 `sfid_id`。
3. 一致性：地址由链上固定算法派生，前后端统一。
4. 最小信任：仅 SFID 系统授权账户可登记。

## 7. 前端接入（手机 App / Web）

### 7.1 目标

用户输入 `sfid_id` 后：
1. 自动查询链上是否已登记。
2. 自动展示链上解析出的 `duoqian_address`（只用于展示）。
3. 提交时只传 `sfid_id`，不再上传 `duoqian_address` 参数。

### 7.2 查询流程

1. 用户输入 `sfid_id`（例如 `GFR-LN001-CB0C-617776487-20260222`）。
2. 前端通过节点 RPC/WS 调用链上存储：
   - `api.query.duoqianTransactionPow.sfidRegisteredAddress(sfidIdBytes)`
3. 若返回 `None`：提示“机构未登记，不能创建多签”。
4. 若返回 `Some(address)`：
   - 自动展示 `duoqian_address = address`
   - 控件设为只读（`readonly`）。
5. 提交 `create_duoqian` 时仅提交 `sfid_id`，并在提交前再次查询确认仍已登记。

### 7.3 前端示例（polkadot.js）

```ts
import { ApiPromise } from '@polkadot/api';

export async function resolveDuoqianAddressBySfid(api: ApiPromise, sfidId: string) {
  const sfidBytes = new TextEncoder().encode(sfidId);
  const opt = await api.query.duoqianTransactionPow.sfidRegisteredAddress(sfidBytes);
  if (opt.isNone) return null;
  return opt.unwrap().toString();
}

export async function onSfidInput(api: ApiPromise, sfidId: string) {
  const address = await resolveDuoqianAddressBySfid(api, sfidId);
  if (!address) {
    return {
      canCreate: false,
      message: '该 sfid_id 未登记到链上，不能创建多签账户',
      duoqianAddress: '',
      readonly: true,
    };
  }
  return {
    canCreate: true,
    message: '',
    duoqianAddress: address,
    readonly: true,
  };
}
```

### 7.4 UI 约束

1. `duoqian_address` 字段只读。
2. 隐藏“手动修改地址”按钮。
3. `create_duoqian` 前端提交前再次链上校验。
4. 若链上登记缺失或变更，直接阻止提交并提示重试。
