# WUMINAPP 技术总文档（当前实现态）

## 1. 项目定位

`wuminapp` 当前为单仓 Flutter 客户端项目（iOS/Android），不再内置独立后端目录。

边界说明：

- 区块链 Runtime/共识逻辑不在本仓库实现（由 `citizenchain` 提供）
- SFID 与链交互由外部服务系统承载
- `wuminapp` 负责端上钱包、登录签名、链上交易入口、绑定指令发起、状态展示

## 2. 当前技术栈

- App：Flutter + Dart
- 手机机密存储：`flutter_secure_storage`（Keychain/Keystore）
- 手机业务存储：Isar
- 链上通信：直连引导节点 Substrate JSON-RPC（`lib/rpc/`）
- 外部接口：HTTP API（由 SFID/网关系统提供，仅用于 SFID 绑定、管理员目录等非链上查询场景）

## 3. 当前目录结构

```text
wuminapp/
├── android/
├── ios/
├── assets/
├── lib/
│   ├── main.dart
│   ├── Isar/
│   ├── rpc/                ← 链上 RPC 公共模块
│   ├── governance/
│   ├── qr/                 ← 二维码统一模块（登录/收款/用户码）
│   ├── signer/
│   ├── user/
│   ├── wallet/
│   └── trade/
├── test/
└── WUMINAPP_TECHNICAL.md
```

说明：

- 原 `mobile/` 内容已上移到项目根目录
- 原 `backend/` 已移除

## 4. App 当前实现

### 4.1 主导航

底部 5 Tab：

- `广场`
- `治理`
- `消息`
- `金融`
- `我的`

### 4.2 治理页

- 机构分类卡片已内置：
  - 国储会 1
  - 省储会 43
  - 省储行 43
- 提案/投票链上交互仍在治理模块开发阶段（规范已落文档）

### 4.3 金融页

- 入口页标题为"金融"
- 已接入"链上交易"页面
- 链下交易仍为开发中占位

### 4.4 钱包与签名

钱包能力收口在 `lib/wallet/`：

- `core/`：钱包生命周期、Isar、机密 key 规范、生物识别守卫
- `capabilities/`：登录签名编排、API（SFID 绑定/管理员目录）、证明态
- `ui/`：钱包页面

余额查询由 `lib/rpc/` 模块直连链上节点完成，不经过外部网关。

签名能力收口在 `lib/signer/`：

- `qr_signer.dart`：冷钱包扫码签名协议（私钥在外部设备）
- `signature_verifier.dart`（目标新增）：统一公钥验签能力
- `signing_coordinator.dart`（目标新增）：冷热钱包统一签名编排入口

签名算法：`sr25519`。

调用点：

- 登录扫码签名前
- 链上交易签名前

签名前守卫：热钱包签名统一经过 `WalletManager.signWithWallet()` / `signUtf8WithWallet()`，内部自动触发生物识别/设备密码验证。

当前约束与目标方案：

- **硬约束**：所有使用本机私钥的签名都必须只走 `WalletManager`
- **热钱包**：唯一允许接触 seed 的模块是 `WalletManager`
- **冷钱包**：唯一允许执行外部扫码签名协议的模块是 `QrSigner`
- **业务模块**：`login`、`trade`、`governance` 只允许提交待签名原文/字节，不允许自己实现签名
- **目标改造**：增加 `SigningCoordinator`，让业务层统一调用一个签名入口；其内部再按 `signMode` 分流到热钱包或冷钱包

### 4.5 二维码模块

二维码模块在 `lib/qr/`，统一管理所有扫码能力：

- 协议定义与路由分发（`QrRouter`）
- 登录码：挑战解析、系统签名验证、防重放、回执生成
- 收款码：生成与解析，预填转账表单
- 用户码：通讯录交换，兼容旧版格式

关键口径：

- 登录协议：`WUMINAPP_LOGIN_V1`
- 收款协议：`WUMINAPP_TRANSFER_V1`
- 用户协议：`WUMINAPP_CONTACT_V1`
- 系统身份：通过 `sys_pubkey`/`sys_sig`/`sys_cert` 密码学验证（不再使用 `aud` 白名单）
- SFID 信任来源：通过区块链 RPC 获取 SFID 当前公钥
- CPMS 信任来源：由 SFID 使用当前有效公钥对 CPMS 公钥做背书（`sys_cert`）
- 登录签名串：

```text
WUMINAPP_LOGIN_V1|system|request_id|challenge|nonce|expires_at
```

详细技术文档见：`lib/qr/QR_TECHNICAL.md`

### 4.6 冷热一体化签名架构（目标方案）

统一原则：

- 一套业务流程，两种签名执行方式
- 热钱包和冷钱包共享同一份待签名内容与协议规范
- 本机私钥签名只能由 `WalletManager` 执行
- 外部设备签名只能由 `QrSigner` 协议执行

目标分层：

- `wallet/WalletManager`
  - 热钱包唯一私钥签名入口
- `signer/QrSigner`
  - 冷钱包唯一签名协议入口
- `signer/SignatureVerifier`
  - 统一公钥验签基础能力
- `signer/SigningCoordinator`
  - 业务层唯一签名编排入口
- `qr/login/LoginTrustService`
  - 登录信任链验证：链上 SFID 公钥 + CPMS 背书校验

业务侧调用方式（目标）：

- 登录：先验系统身份，再生成用户待签名原文，最后统一走 `SigningCoordinator`
- 转账：构造链上待签名字节后统一走 `SigningCoordinator`
- 治理：构造提案/投票待签名字节后统一走 `SigningCoordinator`

这样可保证：

- 业务模块不直接接触 seed
- 热钱包不会出现第二套本机签名实现
- 冷钱包不会在每个页面里各自维护一份扫码签名流程

## 5. 手机端三层存储（当前）

### 5.1 机密层（Secure Storage）

仅存高敏感数据：

- `wallet.secret.<wallet_id>.mnemonic.v1`
- `wallet.session.<scope>.token.v1`
- `wallet.session.<scope>.key.v1`（预留）

### 5.2 业务层（Isar）

钱包域核心集合：

- `WalletProfileEntity`
- `WalletSettingsEntity`
- `TxRecordEntity`
- `AdminRoleCacheEntity`
- `ObservedAccountEntity`
- `LoginReplayEntity`
- `AppKvEntity`

当前 schema 版本：`wallet.data.schema.version = 5`。

### 5.3 偏好层（SharedPreferences）

仍有少量非机密配置使用（按模块逐步收口）：

- 登录防重放记录：`login.used_request_ids`
- SFID 绑定状态：`sfid.bind.*`
- 用户资料：
  - `user.profile.nickname`
  - `user.profile.avatar_path`

## 6. 链上通信架构

### 6.1 通信模式

App 直连区块链引导节点的 RPC 端口，不经过中间网关服务：

```text
手机 App  --JSON-RPC-->  引导节点 :9944（44 个节点，自动选择）
```

每个引导节点同时承担两个角色：

- P2P 端口（30333）：服务于全节点网络同步
- RPC 端口（9944）：服务于手机 App 查询与交易

### 6.2 RPC 节点列表

App 内置 44 个引导节点的 RPC 地址，域名与 `citizenchain/node/src/chain_spec.rs` 中的 P2P 引导节点一致：

| 序号 | 机构 | 域名 |
| --- | --- | --- |
| 1 | 国储会 | `nrcgch.wuminapp.com` |
| 2 | 中枢省 | `prczss.wuminapp.com` |
| 3 | 岭南省 | `prclns.wuminapp.com` |
| 4 | 广东省 | `prcgds.wuminapp.com` |
| 5 | 广西省 | `prcgxs.wuminapp.com` |
| 6 | 福建省 | `prcfjs.wuminapp.com` |
| 7 | 海南省 | `prchns.wuminapp.com` |
| 8 | 云南省 | `prcyns.wuminapp.com` |
| 9 | 贵州省 | `prcgzs.wuminapp.com` |
| 10 | 湖南省 | `prchus.wuminapp.com` |
| 11 | 江西省 | `prcjxs.wuminapp.com` |
| 12 | 浙江省 | `prczjs.wuminapp.com` |
| 13 | 江苏省 | `prcjss.wuminapp.com` |
| 14 | 山东省 | `prcsds.wuminapp.com` |
| 15 | 山西省 | `prcsxs.wuminapp.com` |
| 16 | 河南省 | `prches.wuminapp.com` |
| 17 | 河北省 | `prchbs.wuminapp.com` |
| 18 | 湖北省 | `prchis.wuminapp.com` |
| 19 | 陕西省 | `prcsis.wuminapp.com` |
| 20 | 重庆省 | `prccqs.wuminapp.com` |
| 21 | 四川省 | `prcscs.wuminapp.com` |
| 22 | 甘肃省 | `prcgss.wuminapp.com` |
| 23 | 北平省 | `prcbps.wuminapp.com` |
| 24 | 海滨省 | `prchas.wuminapp.com` |
| 25 | 松江省 | `prcsjs.wuminapp.com` |
| 26 | 龙江省 | `prcljs.wuminapp.com` |
| 27 | 吉林省 | `prcjls.wuminapp.com` |
| 28 | 辽宁省 | `prclis.wuminapp.com` |
| 29 | 宁夏省 | `prcnxs.wuminapp.com` |
| 30 | 青海省 | `prcqhs.wuminapp.com` |
| 31 | 安徽省 | `prcahs.wuminapp.com` |
| 32 | 台湾省 | `prctws.wuminapp.com` |
| 33 | 西藏省 | `prcxzs.wuminapp.com` |
| 34 | 新疆省 | `prcxjs.wuminapp.com` |
| 35 | 西康省 | `prcxks.wuminapp.com` |
| 36 | 阿里省 | `prcals.wuminapp.com` |
| 37 | 葱岭省 | `prccls.wuminapp.com` |
| 38 | 天山省 | `prctss.wuminapp.com` |
| 39 | 河西省 | `prchxs.wuminapp.com` |
| 40 | 昆仑省 | `prckls.wuminapp.com` |
| 41 | 河套省 | `prchts.wuminapp.com` |
| 42 | 热河省 | `prcrhs.wuminapp.com` |
| 43 | 兴安省 | `prcxas.wuminapp.com` |
| 44 | 合江省 | `prchjs.wuminapp.com` |

RPC 地址格式：`http://<域名>:9944`

### 6.3 节点选择策略

- 启动时随机打乱节点列表
- 依次尝试连接，使用第一个可达的节点
- 当前节点连接失败时自动切换到下一个
- 44 个节点全部不可达时抛出异常

### 6.4 RPC 公共模块（`lib/rpc/`）

`lib/rpc/` 是链上通信的唯一收口模块，所有业务模块共享：

```text
lib/rpc/
├── chain_rpc.dart   ← 节点列表、连接管理、底层 JSON-RPC 方法
├── onchain.dart     ← onchain 模块 RPC 功能（extrinsic 构造、转账、状态查询）
└── rpc.dart         ← barrel export
     ↑          ↑          ↑
  wallet/    trade/    governance/
 （余额查询）（转账）  （提案/投票）
```

详细技术文档见：`lib/rpc/RPC_TECHNICAL.md`

### 6.5 链上能力

| 能力 | RPC 方法 | 模块 | 状态 |
| --- | --- | --- | --- |
| 余额查询 | `state_getStorage`（`System.Account`） | `wallet` | 已实现 |
| 转账 | 直连节点构造/提交 extrinsic | `trade/onchain` | 已实现 |
| 提案 | 直连节点提交治理 extrinsic | `governance` | 规划中 |
| 投票 | 直连节点提交投票 extrinsic | `governance` | 规划中 |

## 7. 外部 API 对接（当前）

App 通过 `ApiClient` 访问非链上外部服务，当前已使用接口：

- `GET /api/v1/health`
- `POST /api/v1/chain/bind/request`
- `GET /api/v1/admins/catalog`

### 7.1 区块链能力矩阵（转账 / 提案 / 投票）

| 能力 | 链上入口 | 手机端模块 | 签名域 | 当前状态 |
| --- | --- | --- | --- | --- |
| 转账 | `Balances::transfer_keep_alive` extrinsic（直连 RPC 节点） | `lib/trade/onchain` | `onchain_tx` | 已实现（本机签名主链路） |
| 提案 | 业务治理 pallet `propose_*` | `lib/governance`（规范已定） | `onchain_tx`（交易签名）+ SFID 快照签名字段 | 待开发 |
| 投票 | 业务治理 `vote_*` / 投票引擎 `submit_joint_institution_vote` / `citizen_vote` | `lib/governance`（规范已定） | `onchain_tx`（交易签名）+ 联合投票管理员门限 proof + SFID 投票凭证签名字段 | 待开发 |

### 7.2 区块链字段与格式标准（总则）

- 地址：SS58 字符串（当前链 `ss58 = 2027`）。
- 机构 ID：链上 `[u8; 48]`，App 统一使用 `0x` + 96 hex 表达。
- 签名算法：统一 `sr25519`。
- `nonce/signature`：治理场景均使用字节向量（运行时上限当前为 64 字节）。
- 提案状态：`voting/passed/rejected`（内部执行失败状态由业务 pallet 事件单独体现）。
- 投票引擎外部禁用项：
  - `create_joint_proposal`（外部调用禁止）
  - `internal_vote`（外部调用禁止）
  - 必须通过业务治理 pallet 发起。

详细字段与流程见：

- `lib/trade/onchain/ONCHAIN_TECHNICAL.md`（转账）
- `lib/governance/GOVERNANCE_TECHNICAL.md`（提案/投票）

## 8. 安全基线（当前）

- 私钥/助记词不落 Isar 与远端服务
- 助记词读取强制生物识别/设备密码验证（存储层统一守卫，不可关闭）
- 设备无生物识别也无密码时自动跳过验证
- 登录系统身份通过密码学签名验证（`sys_pubkey`/`sys_sig`/`sys_cert`）
- 绑定请求与交易状态依赖外部服务返回

## 9. 已知限制

- 登录防重放当前仍在 `SharedPreferences`，尚未切到 Isar 的 `LoginReplayEntity`
- `SfidBindingService` 状态仍在 `SharedPreferences`（`sfid.bind.*`）
- 链下交易模块仍为占位
- 扫码签名当前已完成协议层实现，业务 UI 仍以本机签名为主
- 提案/投票尚未实现直连 RPC

## 10. 本地开发

```bash
cd /Users/rhett/GMB/wuminapp
flutter pub get
flutter run \
  --dart-define=WUMINAPP_RPC_URL=http://127.0.0.1:9944 \
  --dart-define=WUMINAPP_API_BASE_URL=http://<外部服务地址> \
  --dart-define=WUMINAPP_API_TOKEN=<token>
```

- `WUMINAPP_RPC_URL`：覆盖默认 RPC 节点地址，本地开发时指向本机节点。不设置时 App 自动从 44 个引导节点中选择。
- 真机调试时地址需为手机可达地址，不可用 `127.0.0.1`。

## 11. 关联模块文档

- RPC 模块：`lib/rpc/RPC_TECHNICAL.md`
- 二维码模块：`lib/qr/QR_TECHNICAL.md`
- 签名模块：`lib/signer/SIGNER_TECHNICAL.md`
- 治理模块：`lib/governance/GOVERNANCE_TECHNICAL.md`
- 用户模块：`lib/user/USER_TECHNICAL.md`
- 钱包模块：`lib/wallet/WALLET_TECHNICAL.md`
- 链上交易模块：`lib/trade/onchain/ONCHAIN_TECHNICAL.md`
