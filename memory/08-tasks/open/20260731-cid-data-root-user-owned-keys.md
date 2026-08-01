# CID 数据根改为用户钱包自持，删除服务端统一主密钥

状态：open（2026-07-31 方案定稿，待实施）

## 缺陷

`CID_DATA_ROOT_MASTER_KEY` 是 Worker 侧唯一的 AES-GCM 主密钥，密封**全部用户**的
`cid_data_roots.sealed_data_root`。数据根由 Worker `crypto.getRandomValues` 生成
（`cid_data_root.ts:304`），App 领取时由 Worker 解封后**明文下发**。

数据根派生出五把用途子钥（`LocalKeyPurpose`）：

| 子钥 | 保护的数据 |
| --- | --- |
| `contactsLocal` | 通讯录（本地 + 云端 `square_contacts` 密文） |
| `chat` / `chatIndex` | 聊天正文与索引 |
| `attachment` | 附件 |
| `mls` | MLS 群组状态 |

后果：**持有该主密钥 + 读 D1，可解出任意 CID 的数据根，进而解开该用户全部本地与
云端密文**。云端那些表虽存密文，但密钥祖先在服务端手里，加密等于没加。
且该密钥单点、全局、不可更换（密封过即无法轮换），丢失等于所有用户跨设备恢复能力一起丢失。

违背产品原则：用户数据必须由用户自己的钱包控制，服务端零知识。

## 定稿方案

### 数据根

```
数据根 = HKDF(母种子, "citizenapp.cid-data-root" ‖ cid_number)
```

- 与钱包账户**完全无关**，永久稳定
- 母种子只在录入助记词的瞬间临时派生，用完清零，本端从不落盘（保持 ROOTLESS）
- 服务端不生成、不托管、不持有、不下发

### 为什么不能挂在账户 child 上

换绑的典型起因就是**旧私钥泄漏或丢失**；且投票/竞选身份链端强制走注册局换绑
（`citizen-identity/src/lib.rs:1352` `CivicRebindRequiresRegistrar`、
`admin_rebind_cid_account_id`），此时用户不在自己设备上、更不持有旧 child。
数据根一旦依赖 child，换绑即等于数据不可恢复，且需要全量重加密（用户数据可达数十上百 GB）。

### 三种情形均无新增交互

| 情形 | 数据根来源 |
| --- | --- |
| 日常使用 | 本地 `CidDataRootVault` 缓存（当前 child 包装，纯性能优化） |
| 同设备换绑 A→B（两账户都在列表） | 旧账户 child 仍在本机（换绑只改链上绑定，不删钱包账户）→ 解包后用新 child 重包 32 字节，**用户无感** |
| 绑定账户不在本机 / 新设备 / 注册局换绑后 | 用户本就必须先把该账户导入钱包——走现有「＋ → 添加指定账户 / 导入钱包」录助记词，**在该已有流程内顺手派生并缓存** |

关键：凡是需要母种子的时刻，都是用户**本来就要输助记词**的时刻。派生数据根对用户完全不可见。
入口已存在：`wallet_page.dart:580`（＋三项）、`add_account_sheet.dart`。

### 换绑成本

零。数据根不变 → 业务密钥不变 → 本地与云端全部密文原样可读，只重新包装 32 字节缓存。

### 云端边界（硬约束）

云端**只存已加密的业务数据**。不存数据根、不存包装后的数据根、不存任何形态的密钥材料。
服务端在密码学上无法解开任何一条。

## 改造清单

**App**

- `ensureCidDataRootForCurrentBinding`：不再向服务端领取。改为「缓存命中即用 →
  未命中且旧账户 child 在本机则解包重包 → 都没有则由钱包导入流程派生」
- `CidDataRootVault`（`security/local_data_key.dart:135`）保留，语义由「服务端授权安装」
  改为「本地派生结果的缓存」；`expectedDataRootHash` 来源改为本地自校验
- `createWallet` / `importWallet` / `addAccounts` 在母种子清零前派生并缓存数据根
- 删除 `takeoverCidDataRoot`、`_runBindingTakeover`、`identity_synced_account_store` 相关逻辑
- `ensureDeviceSubkeyBound` 只剩「绑设备子钥」

**Worker**

- 删除 `src/account/cid_data_root.ts` 全文
- 删除路由 `/v1/square/identity/takeover/challenge`、`/v1/square/identity/takeover`
- 从 `schema/citizenapp.sql` 删除 `cid_data_roots` 表（升 SCHEMA VERSION + 追加变更日志）
- 从 `src/types.ts` 的 `Env` 删除 `CID_DATA_ROOT_MASTER_KEY`

## 连带解决

CitizenApp「设备绑定未完成」的最后一个 503（`cid_data_root_key_unavailable`）随之消失——
该主密钥根本不该存在，不需要配置。这也是设备绑定故障链的终点：
CID 解析 bug → 控制台不可用 → Worker 未部署 → tunnel 未装 → D1 缺表 → 主密钥缺失。

## 必须知悉的后果

**助记词丢失 = 数据永久无法恢复**，服务端救不了。这是拆掉中心化后门的必然代价，
也是「用户自己的钱包控制自己的数据」的真正含义。

## 时机

`cid_data_roots` 表刚重建、零数据，云端业务表亦为空。此时改造零迁移成本。

## 遗留

`citizenapp/cloudflare/package.json` 未声明 `aws4fetch`，导致「清空并重建全部数据」
第 3 步清 R2 失败（`ERR_MODULE_NOT_FOUND`）。属独立问题，随本卡一并修复。
