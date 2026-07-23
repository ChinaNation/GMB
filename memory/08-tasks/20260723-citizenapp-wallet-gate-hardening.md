# CitizenApp 钱包门控加固 + accountId 单源校验器

任务需求：杜绝「无有效热钱包却能进入 App」的一切情况；顺带把散落全仓的 accountId 格式正则收敛为单源校验器。
所属模块：citizenapp / wallet（Mobile），不涉及链端。

## 缘起（真实事故）

工作树中进行中的「钱包字段统一」把 Isar 实体属性改了名（`address→accountId`、`pubkeyHex→ss58Address`），而 Isar 按属性名存取，旧库的值仍挂旧名下 → 新构建读出空串。结果：

- 钱包行仍在、`signMode=='local'` 仍可读 → `getDefaultWallet()` 返回**非 null**
- 但 `accountId` 为空串 → 下游（聊天 / 广场 / 收付款）全部静默降级成「没钱包」
- 而 `WalletGate._check()` 只判 `wallet == null` → **判定为 ready，直接放行进 App**

即：**一个 accountId 为空的半残钱包可以畅通无阻过门禁**。这是 fail-open 缺口，与字段改名无关——任何导致 accountId 读空的原因都会踩到。

## 定稿（用户确认）

1. **有效钱包判定取最严档（C）**，但用**静默**实现：
   - `signMode == 'local'`（热钱包）
   - `accountId` 匹配 `^0x[0-9a-f]{64}$`
   - `ss58Address` 非空且 `== ss58FromAccountIdText(accountId)`
   - **种子密文 blob 存在**（`hasSeed`，只读 blob 不调 `decrypt`，故**不弹生物识别**；严档 seed 是纯生物档，真解密会导致每次冷启动都弹指纹）
2. **门控依据只认有效热钱包**：冷钱包、半残钱包一律不算。
3. **运行期踢出**：用户在「我的 → 钱包」删光钱包后，一旦不再存在有效热钱包，立即踢回初始化页。
4. **半残钱包直接进初始化页**，用户在那里可「重新创建钱包」或「输入助记词恢复」——`CreateWalletOnboardingPage` 已现成具备这两条 fail-closed 路径，**零新 UI**。
5. **accountId 正则收敛为单源**，走 A2 两小步（见下）。

## accountId 正则收敛：必须按语义切，不能盲扫

全仓约 40 处 `[0-9a-f]{64}`，**至少三种语义**，盲扫会把「账户」和「区块哈希/交易哈希/文件摘要」绑进同一个校验器，恰恰违反全仓字段同名硬规则的本意（同语义同名，**异语义不得共用**）：

| 语义 | 位置 | 收敛 |
|---|---|---|
| 真 accountId（约 35 处 / 28 文件） | `app_isar`、`contact_service`、`institution_*`、`personal-manage/*`、`proposal/*`、`votingengine/*`、`wallet_*` 等 | ✅ |
| 区块哈希 | `rpc/smoldot_client.dart:1602`（上下文 `finalized_block_hash`） | ❌ |
| 交易哈希 | `rpc/signed_extrinsic_relay_api.dart:161` `_txHash` | ❌ |
| 通用 32B hex 字段（stateRoot 等） | `rpc/chain_bootstrap_api.dart:352` `_hex32` | ❌ |
| 文件 sha256（**无 `0x` 前缀**） | `update/app_update_manifest.dart:44` | ❌ 绝不能扫 |
| 通用 hex→bytes | `citizen/shared/proposal/proposal_local_store.dart:428` | ⚠ 逐调用方判定 |

## 分步（A2）

- **Step 2a**：抽单源校验器 `isAccountIdText`（落在 `citizen/shared/account_derivation.dart`，该文件自称「账户统一派生原语，全 app 唯一入口」，语义吻合且自身就有一处重复）+ 新增 `SecureSeedStore.hasSeed` + WalletManager 有效热钱包谓词 + 门控加固 + 钱包链路切到校验器。**门控这条线可独立验证**。
- **Step 2b**：批量把其余 accountId 语义的文件切到单源校验器（纯机械替换，独立验证）。最终重复数归零。

## Step 2a 落点

- `lib/citizen/shared/account_derivation.dart`：新增 `isAccountIdText(String)` 单源校验器；本文件原有那处正则改调它。
- `lib/wallet/core/secure_seed_store.dart` + `hardware_bound_seed_vault.dart` + `fake_hardware_bound_seed_vault.dart`：新增静默 `hasSeed(walletIndex)`（真实现只 `_blobStore.read(_seedBlobKey(idx)) != null`，**不调 `decrypt`**；fake 用 `_seeds.containsKey`）。
- `lib/wallet/core/wallet_manager.dart`：新增有效热钱包谓词并对外暴露；`_normalizeAccountId` 改调单源校验器。
- `lib/wallet/capabilities/wallet_type_service.dart`：`_requireAccountId` 改调单源校验器。
- `lib/wallet/wallet_gate.dart`：判定换谓词（保留 error 态，读库失败不得误判成「无钱包」）；监听 `WalletManager.walletsRevision` 做运行期踢出；踢回前 `popUntil` 回根。
- `test/wallet/wallet_gate_test.dart`：随谓词调整；新增半残钱包被拦、只剩冷钱包被拦、运行期删光钱包即时踢回。

## 边界与风险

- **踢回时的页面栈**：`WalletGate` 在 `AppShell` 之上，切状态会重建到初始化页，但 AppShell 内 push 的页面栈仍在 Navigator 里——不 `popUntil` 回根会出现「初始化页被旧页面盖住」。本步最易出错处。
- **fail-closed 误锁**：判定过严且无自救路径 = 用户永久进不去。必须保留 error 态与重试；`hasSeed` 抛异常走 error 态而非判死。
- 分层：`wallet/` 已依赖 `citizen/shared/`（`wallet_type_service.dart`），`account_derivation.dart` 无反向依赖，不成环。
- 冷启动多一次 blob 读 IO，量级极小。

## 验收

- `flutter analyze` 0 问题；全量 `flutter test` 通过。
- 真机：有效热钱包正常进入；删光钱包立即踢回初始化页且无旧页面残留；初始化页两条路径可达。

## 执行结果

### Step 2a（2026-07-23，完成）

- **单源校验器**：`citizen/shared/account_derivation.dart` 新增 `isAccountIdText(String)`（内部 `_accountIdPattern`），注释明确**只用于账户**，区块哈希 / 交易哈希 / stateRoot / 文件 sha256 虽同为 32 字节 hex 但语义不同，不得复用（异语义共用等于制造假单源）。该文件自身原有的正则改调它。
- **静默 `hasSeed`**：`SecureSeedStore` 接口新增；`HardwareBoundSeedVault` 只 `_blobStore.read(_seedBlobKey(idx)) != null`，**不调 `decrypt`**，因此不弹生物识别（严档 seed 是纯生物档，真解密会导致每次冷启动弹指纹）；后端异常抛 `SecureStoreUnavailable` 交上层走错误态。`FakeHardwareBoundSeedVault` 与 `test/support/fake_secure_seed_store.dart` 同步实现，后者特意**不计入 `readSeedCount`**（它不是一次 seed 读取，否则会打乱既有「每次签名读一次 seed」断言）。
- **有效热钱包谓词**：`WalletManager.isUsableHotWallet(WalletProfile)` + `getValidDefaultWallet()`。`getDefaultWallet()` 保持原语义不动（调用方众多），门控改用新入口。
- **门控加固**：`wallet_gate.dart` 判定换谓词；新增 `dispose` 中摘除监听；监听 `WalletManager.walletsRevision` 做运行期踢出，**踢回前 `Navigator.popUntil(isFirst)` 清空 AppShell 页面栈**（删钱包动作本身发生在深层页面，不清栈初始化页会被旧页面盖住）；error 态与重试保留。
- **钱包链路切校验器**：`wallet_manager._normalizeAccountId`、`wallet_type_service._requireAccountId`。
- **测试**：`wallet_gate_test.dart` 新增 6 例 —— 运行期删光钱包即时踢回；谓词五例（四条全过有效 / 冷钱包不算 / accountId 空的半残不算 / ss58 与 accountId 对不上不算 / 有壳无钥不算）。
- **验收**：`flutter analyze`(lib+test) 0 问题；**全量 `flutter test` 785 通过 / 5 跳过 / 0 失败**（较改前 +6）。真机：门控加固后携修复好的热钱包3 正常放行，无误锁。
- **未在真机验证**：运行期踢出需删光该机全部钱包（破坏性，会再次丢失钱包3），故不在用户设备上执行，由 widget 测试覆盖。
