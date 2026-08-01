# CID 数据根改为用户钱包自持，删除服务端统一主密钥

状态：done（2026-07-31 实施完成，三端测试全绿）

## 缺陷

`CID_DATA_ROOT_MASTER_KEY` 是 Worker 侧唯一的 AES-GCM 主密钥，密封**全部用户**的
`cid_data_roots.sealed_data_root`。数据根由 Worker `crypto.getRandomValues` 生成，
App 领取时由 Worker 解封后**明文下发**。

数据根派生出五把用途子钥（`LocalKeyPurpose`）：

| 子钥 | 保护的数据 |
| --- | --- |
| `contactsLocal` | 通讯录（本地 + 云端 `square_contacts` 密文） |
| `chat` / `chatIndex` | 聊天正文与索引 |
| `attachment` | 附件 |
| `mls` | MLS 群组状态 |

后果：**持有该主密钥 + 读 D1，可解出任意 CID 的数据根，进而解开该用户全部本地与
云端密文**。云端那些表虽存密文，但密钥祖先在服务端手里，加密等于没加。
且该密钥单点、全局、不可轮换（密封过即无法更换），丢失等于所有用户跨设备恢复能力一起丢失。

违背产品原则：用户数据必须由用户自己的钱包控制，服务端零知识。

## 定稿方案

### 数据根

```text
数据根 = HKDF(母种子, salt="citizenapp.cid/root",
              info="citizenapp.cid-data-root/<cid_number>")
```

- 与钱包账户**完全无关**，永久稳定
- 母种子只在录入助记词的瞬间临时派生，用完清零，本端从不落盘（保持 ROOTLESS）
- 服务端不生成、不托管、不持有、不下发

### 为什么不能挂在账户 child 上

换绑的典型起因就是**旧私钥泄漏或丢失**；且投票/竞选身份链端强制走注册局换绑
（`citizen-identity/src/lib.rs` `CivicRebindRequiresRegistrar`、
`admin_rebind_cid_account_id`），此时用户不在自己设备上、更不持有旧 child。
数据根一旦依赖 child，换绑即等于数据不可恢复，且需要全量重加密（用户数据可达数十上百 GB）。

### 三种情形均无新增交互

| 情形 | 数据根来源 |
| --- | --- |
| 日常使用 | 本机 `CidDataRootVault` 缓存（当前 child 包装，纯性能优化） |
| 同设备换绑 A→B（两账户都在列表） | 旧账户 child 仍在本机（换绑只改链上绑定，不删钱包账户）→ 解包后用新 child 重包 32 字节，**用户无感** |
| 绑定账户不在本机 / 新设备 / 注册局换绑后 | 用户本就必须先把该账户导入钱包——走现有「＋ → 添加指定账户 / 导入钱包」录助记词，**在该已有流程内顺手派生并缓存** |

关键：凡是需要母种子的时刻，都是用户**本来就要输助记词**的时刻。派生数据根对用户完全不可见。

### 换绑成本

零。数据根不变 → 业务密钥不变 → 本地与云端全部密文原样可读，只重新包装 32 字节缓存。

### 云端边界（硬约束）

云端**只存已加密的业务数据**。不存数据根、不存包装后的数据根、不存任何形态的密钥材料。
服务端在密码学上无法解开任何一条。

## 实建

### App

- `security/local_data_key.dart`
  - 新增 `CidDataRoot.deriveFromMasterSeed({masterSeed, cidNumber})`，空母种子 / 空 CID 一律拒绝
  - `CidDataRootVault` 语义由「服务端授权安装」改为「本地派生结果的缓存」；
    `expectedDataRootHash` 改为本地自校验（错误文案同步）
- `wallet/core/wallet_manager.dart`
  - 新增 `ensureCidDataRootReady({cidNumber, bindingRevision, accountId})`：缓存命中 →
    旧包装＋旧 child 在本机则解包重包 → 都不满足抛 `CidDataRootMnemonicRequiredException`
  - 新增 `installCidDataRootFromMnemonic(...)`：母种子临时派生，`finally` 清零
  - 新增私有 `_installCidDataRoot` / `_readCidDataRootWithPreviousBinding`
  - 新增类型 `CidDataRootRequest`、异常 `CidDataRootMnemonicRequiredException`
  - **删除** `installCidDataRootForCurrentBinding`、`hasInstalledCidDataRootBinding`
  - `bindDeviceSubkeyToCurrentBinding` 增加按三元组的本机幂等标记（见下「连带修复」）
- `my/myid/myid_service.dart`
  - **删除** `_runBindingTakeover` / `_doRunBindingTakeover` /
    `_reconcileFinalizedBindingTakeover` / `reconcileFinalizedBindingTakeover` /
    `_isBindingTakeoverComplete` / `_writeSyncedMarker` / `_takeoverInflight` / `_signatureHex`
  - 新增 `_ensureBindingReady`：数据根就位 → 登记设备子钥（顺序不可颠倒）
  - 身份解析流程不再急切对账，改由门禁按需触发——身份页不该因缺数据根而失败
- `my/myid/widgets/identity_registration_gate.dart`：新增 `_GateStatus.mnemonicRequired`
  与 `_importMnemonic()`，跳既有导入页并回来重判
- `wallet/pages/import_wallet_page.dart`：新增可选 `dataRootRequest`，导入成功后
  **用同一份助记词**顺手派生（母种子只在这一刻存在）
- **删除** `my/myid/identity_synced_account_store.dart`、
  `SquareApiClient.takeoverCidDataRoot`、`CidDataRootGrant`

### Worker

- **删除** `src/account/cid_data_root.ts` 全文
- **删除** 路由 `/v1/square/identity/takeover/challenge`、`/v1/square/identity/takeover`
  及 `src/limits/catalog.ts` 中对应限流条目、`request_guard.ts` 限流分支
- `schema/citizenapp.sql` 删 `cid_data_roots` 表，`SCHEMA VERSION` 升 `v1.1.0` 并追加日志（表数 27→26）
- `src/types.ts` 的 `Env` 删 `CID_DATA_ROOT_MASTER_KEY`；`src/account/purge.ts` 删对应清理语句
- 测试删 takeover 用例与 `TakeoverStmt` / `TakeoverDb` / `takeoverEnv` / `jsonPost` 死假件
- `package.json` 补声明 `aws4fetch`（原缺失导致清 R2 `ERR_MODULE_NOT_FOUND`）

### 控制台

- `test/production-security.test.mjs` 基线断言由「必须含 `CREATE TABLE cid_data_roots`」
  改为「不得再建该表」（只钉建表语句，变更日志里的历史记录要留）

## 连带修复：重复弹生物识别

`MyIdService` **不是单例**，五处门禁各 `new` 一个，`_subkeyBindInflight` 进程内去重
从来没生效过。子钥登记要签名、签名要弹生物识别，于是每次进入需 CID 页面都可能重复弹。

修法：幂等标记下沉到 `WalletManager`，按 `(cid_number, binding_revision, account_id)`
落本机。登记成功才写标记；失败不写，下次重试。换绑推进 revision → 标记名改变 → 自动重新登记。

## 一处返工记录

`_installCidDataRoot` 起初漏掉了原 `installCidDataRootForCurrentBinding` 里的
「清理旧账户级命名残留」（`wallet_contacts_key_v1_*` 等三个旧名）。
测试 `通讯录密钥只读写新域并主动删除旧命名残留` 直接抓到，已补回。
教训：删除并重写一个方法时，要逐条核对它原本承担的**全部**职责，不能只看主干。

## 验证

| 套件 | 结果 |
| --- | --- |
| CitizenApp `flutter test` | **1062 passed / 0 failed**（改造前 1051，净增 11） |
| Worker `npm test` | **32 文件 / 218 passed** |
| Worker `npm run typecheck` | 通过 |
| CitizenConsole `npm test` | **38 passed** |
| `dart analyze lib test` | 无问题 |

新增测试：

- `test/security/local_data_key_test.dart`：数据根派生确定性、CID 隔离、母种子隔离、
  CID 空白容忍、空输入拒绝（5 条）
- `test/wallet/wallet_manager_test.dart`：子钥登记幂等、换绑后重新登记、失败不写标记（2 条）
- `test/my/myid/myid_service_test.dart`：缺数据根时上抛待补录信号且**不登记子钥**
  （避免留下「已绑定但读不了数据」的半截状态）

## 连带解决

CitizenApp「设备绑定未完成」的最后一个 503（`cid_data_root_key_unavailable`）随之消失——
该主密钥根本不该存在，不需要配置。这也是设备绑定故障链的终点：
CID 解析 bug → 控制台不可用 → Worker 未部署 → tunnel 未装 → D1 缺表 → 主密钥缺失。

## 必须知悉的后果

**助记词丢失 = 数据永久无法恢复**，服务端救不了。这是拆掉中心化后门的必然代价，
也是「用户自己的钱包控制自己的数据」的真正含义。

## 待用户执行

代码已就绪，线上尚未同步：

1. 控制台 →「清空并重建全部数据」（D1 按新基线重建，`cid_data_roots` 不再创建）
2. 控制台 → 部署 Worker（takeover 路由随之下线）
3. 设备上重进聊天/广场，验证设备绑定闭环

## 时机

`cid_data_roots` 表刚重建、零数据，云端业务表亦为空。此时改造零迁移成本。
