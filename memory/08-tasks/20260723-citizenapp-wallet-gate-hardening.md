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

### Step 2b（2026-07-23，完成）

- **批量收敛**：28 文件 / 37 处 `RegExp(r'^0x[0-9a-f]{64}$').hasMatch(x)` → `isAccountIdText(x)`，逐处核对变量与上下文确属账户语义后再切，**未做正则批量 sed**。`identity_badge_snapshot_store.dart` 的具名字段 `_accountIdPattern` 一并删除并改调单源。
- **补切一处**：`citizen/shared/proposal/proposal_local_store.dart` 的 `_hexToBytes` 原按「通用 hex」保留待判，核对后其**唯一调用方是 `executionAccountId`**（:217），确属账户语义，已切并补注释说明。
- **最终残留核查**：全仓 `[0-9a-f]{64}` 只剩 5 处 —— `account_derivation.dart:83` 的**单源定义本身**，加 4 处**明确异语义**：`app_update_manifest.dart`（文件 sha256，无 `0x`）、`chain_bootstrap_api.dart`（通用 hex32/stateRoot）、`signed_extrinsic_relay_api.dart`（交易哈希）、`smoldot_client.dart`（区块哈希）。**accountId 语义的重复归零。**
- **验收**：`flutter analyze`(lib+test) 0 问题；全量 `flutter test` **785 通过 / 5 跳过 / 0 失败**（与 2a 持平，纯机械替换无行为变化）；真机安装运行正常、logcat 无 `FlutterError-Diag` / `ErrorWidget-Diag` / `FormatException`。

### Step 2c：单行坏数据毒化整表（同根因的第三个暴露面，2026-07-23，完成）

**现象**：真机上「我的」页身份显示钱包3，但「我的钱包」列表却空白并提示「还没有钱包，请选择一种方式开始」+ SnackBar `本地钱包读取失败：FormatException`。

**根因**（两条不同代码路径造成的表里不一）：

- `wallet_page.dart _loadWallets`：`getWallets()` 两个钱包都读到了，但紧接的 `for` 循环对每个钱包调 `ChainTxMonitor.watchWallet(ss58, accountId)`，其首行 `LocalTxStore.requireAccountId(accountId)` 对**冷钱包2 的空 accountId** 抛 `FormatException` → 异常掀翻整个 `_loadWallets` → `setState(_wallets = list)` 永不执行 → 列表整体空白。
- 而「我的」页身份走 `getDefaultWallet()`，它只遍历取第一个热钱包、**不碰 ss58/accountId 派生**，所以钱包3 照常显示。

**性质**：与门控 fail-open 同源的另一种失败姿势 —— **一行坏数据让整份列表谎称「还没有钱包」**，空态文案还会诱导用户新建，把数据搞得更乱。非 Step 2a/2b 引入（2b 只把内联正则换成单源，判定与抛错逐字等价）；自 Isar 属性改名起即存在。

**修法（用户选 A）**：

- `_loadWallets`：`watchWallet` 调用**按钱包隔离** —— 先用 `isAccountIdText` 拦坏行并 `debugPrint` 记录（非静默），再对正常行 try/catch 兜底；坏行跳过链上监听但**仍进列表**。外层 catch 回归本职，只处理整份读库失败。
- `WalletListTile` 新增 `isBroken`：坏行不显余额（读不到身份就对不上链，展示余额是误导），改显橙色警示「身份数据异常，请删除后重新导入」。
- 列表 `onTap` 拦截：坏行不进详情（详情页同样会在空 accountId 上抛错，不拦等于把炸点从列表挪到详情），改弹提示。
- `isIdentityWallet` 判定加 `!isBroken` 前置：坏行 accountId 为空，与同为空的身份账户比对会误挂「身份钱包」徽标。
- 删除路径已现成且对坏行安全：`deleteWallet` 的 `_contactCacheKeys('')` 只拼字符串不校验，冷钱包跳过 `signMode=='local'` 的种子清理分支。

**验收**：`flutter analyze` 0 问题；全量 `flutter test` **788 通过 / 5 跳过 / 0 失败**（+3）；真机列表同时显示钱包3（正常 + 默认用户徽标）与钱包2（橙色警示 + 可删），FormatException 提示消失，logcat 输出 `[Wallet] 身份异常，跳过链上监听: index=2`。

### 附：丢失钱包「旅行者」恢复（2026-07-23，完成）

**现象**：用户指出真机上原有的自定义命名钱包「旅行者」不见了。清点时只有 index=2（冷）与 index=3（热），index=1 缺失。

**证据链**：

- secure storage 中 `wallet_seed_env_v1_1` 与 `wallet_recovery_env_v1_1` **都还在**（index=2 两者皆无，符合冷钱包无种子；index=3 两者皆在）。
- `deleteWallet` 对热钱包会连 seed 与助记词一起删 → 两者俱在即可断定「旅行者」**不是走 App 删除路径消失的**，其 Isar 行是异常丢失。
- 另有残留 `wallet_contacts_key_v1_w5GPoqEs…`（旧代码用 SS58 作通讯录密钥名），非钱包3 的地址，推测为旅行者 —— **恢复后地址实测正是 `w5GPoqEsUkBtqzPMakf2njBAmgbsG8YPYzyJfeWFYLcZnoBoc`，推测得证**。

**结论订正**：先前依据「只看到 2 行仍并存」判断「唯一索引在空值上折叠没有发生」，该结论**证伪** —— 当时并不知道原本有 3 行。`accountId` / `ss58Address` 两个 `@Index(unique: true, replace: true)` 在改名迁移后全变空值，**行在迁移中丢失是首要嫌疑**（未验证 Isar 确切行为，不下定论）。这说明未收尾的「钱包字段统一」重构危险性高于先前评估：它不只让钱包读不出，**还可能吃掉整行**。

**恢复方式（用户选 A）**：一次性动作读 index=1 的宽档助记词 → 直接调既有 `importWallet(mnemonic)` 重建（沿用已测试的 fail-closed 路径，失败自动回滚）。**助记词全程不打印、不出设备。** 结果：`index=1 accountId=0xd4cad887… ss58=w5GPoqEs…` 重建成功，列表恢复为 3 个钱包。自定义名「旅行者」随 Isar 行一起丢失，重建后为默认名「钱包1」，需用户改回。一次性代码已整段删除，`analyze` 0 问题、全量 788 通过、全仓零残留。

## 总体收尾

Step 2a + 2b 全部完成。门控自此**只认有效热钱包**（四条判定），半残钱包与冷钱包一律拦到初始化页；运行期删光钱包即时踢回且先清页面栈。accountId 格式判定收敛为 `isAccountIdText` 单源，异语义（区块/交易哈希、文件摘要）刻意不并入。
