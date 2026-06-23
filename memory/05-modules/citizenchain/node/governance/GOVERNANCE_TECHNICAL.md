# 节点桌面端治理子模块 — 技术文档

## 模块结构

```
governance/
├── mod.rs              # Tauri 命令入口：提案创建、投票、签名请求/提交
├── admins_change/      # 管理员管理：激活、主体读取、管理员集合变更、签名提交
├── organization-manage/# 机构多签管理：CID 凭证、机构详情、创建机构多签签名请求
├── runtime_upgrade/    # 协议升级：开发期直升、运行期协议升级业务签名与提交
├── signing.rs          # QR 签名协议实现：payload 构建、签名验证、交易提交
├── proposal.rs         # 提案查询与解码：从链上 storage 读取并解析提案详情
├── institution.rs      # 机构信息查询：管理员列表、机构全称/简称
├── storage_keys.rs     # 链上存储 key 构造：twox_128 / blake2b_128 / double_map_key
└── types.rs            # 共享类型定义
```

## 核心职责

- 为前端治理页面提供所有 Tauri 命令（提案列表、提案详情、发起提案、投票、执行）
- 实现管理员激活机制（冷钱包扫码签名 → 本地验证 → 解锁提案操作）
- 实现 CITIZEN_QR_V1 QR 签名协议（离线签名设备 ↔ 节点桌面端）
- 从链上 RPC 解码提案数据（联合投票/内部投票/机构管理员/销毁/发行/运行时升级）
- 治理聚合层不得实现投票流程、人口快照获取、计票或投票状态推进；这些职责统一归投票引擎
- 治理详情中的余额、发行/销毁/多签转账等金额字段统一按 finalized block hash 读取；提案/交易进度仍可展示 pending、inBlock、finalized 状态

前端对应结构：
- `node/frontend/governance/api.ts`：治理专用 Tauri API
- `node/frontend/governance/admins-change/`：管理员列表与管理员更换页面
- `node/frontend/governance/runtime-upgrade/`：协议升级与开发升级页面，只提交业务提案，不实现投票流程
- `node/frontend/governance/organization_manage/`：机构多签管理页面、API 和 DTO
- `node/frontend/governance/types.ts`：治理页面 DTO 类型
- `node/frontend/shared/qr/`：QR 扫码组件与 CITIZEN_QR_V1 解析协议，治理前端通过共享层引用，不再把扫码能力放在治理目录内
- `node/frontend/shared/ss58.ts` / `node/frontend/shared/format.ts`：SS58 地址展示与金额格式化

## admins_change/activation.rs — 管理员激活

### 设计原则

所有用户安装区块链软件后看到统一的机构详情页：管理员列表 + 灰色不可操作的提案按钮。
真正的管理员通过冷钱包扫码签名激活后，提案按钮变为可操作。

### 激活流程

1. 用户点击管理员行的"激活"按钮
2. 后端验证公钥在链上管理员列表中
3. 构建 `GMB_ACTIVATE_SUBJECT_V1` subject 级签名 payload（非链上交易）
4. 生成 CITIZEN_QR_V1 格式的 QR 签名请求
5. 用户用 citizenwallet 公民钱包扫码签名
6. 后端验证 sr25519 签名，并重新确认链上主体仍 Active
7. 签名验证成功 → 写入本地加密存储
8. 前端刷新 → 管理员变绿 + 提案按钮可操作

### 激活 payload 格式

```
GMB_ACTIVATE_SUBJECT_V1 (23 字节 ASCII)
+ account_id (48 字节)
+ org (1 字节)
+ kind (1 字节)
+ pubkey (32 字节)
+ timestamp (8 字节, u64 LE)
+ random_nonce (16 字节)
= 总计 129 字节
```

非链上交易，不需要 nonce/era/genesis_hash 等扩展。

### Tauri 命令

| 命令 | 说明 |
|------|------|
| `build_activate_admin_request` | 验证链上管理员身份 → 生成激活签名 QR JSON |
| `verify_activate_admin` | 验证 sr25519 签名 → 写入本地加密存储 |
| `get_activated_admins` | 读取已激活管理员 + 链上交叉校验自动清除失效 |
| `deactivate_admin` | 手动取消激活（需设备密码） |

### 存储

- 文件：`{app_data}/activated-admin-subjects.json`
- 格式：JSON 数组，每条记录包含 `pubkey_hex / account_id_hex / org / kind / activated_at_ms / signature_hex / payload_hash_hex`
- 安全：文件权限限制（通过 security::write_text_atomic_restricted）
- 失效：每次 `get_activated_admins` 调用时与链上管理员主体的 `org/kind/admins/status` 交叉校验

### 省储行验证者

省储行管理员激活后，额外显示"设为验证者"按钮。
点击后输入私钥种子，调用 `set_signing_admin` 将私钥写入 OffchainKeystore。

## signing.rs — QR 签名协议

### 协议流程
1. 节点桌面端构建 `VoteSignRequest`（含 payload + metadata），生成 QR 码
2. 离线签名设备扫描 QR，使用冷钱包私钥签名
3. 签名设备生成 `QrSignResponse`，节点桌面端扫描回传
4. 节点桌面端验证签名、构造完整 extrinsic、提交到链上

### 支持的签名类型
- 管理员激活（activate_admin_account，非链上交易）
- 联合投票（省储会/省储行管理员投票）
- 内部投票（机构管理员替换/销毁/多签）
- 公民投票
- 决议发行提案创建
- 运行时升级提案创建
- 开发期直接升级

### 安全设计
- nonce 有 90 秒 TTL，防止重放
- payload 包含 genesis_hash 做链域隔离（链上交易）
- 签名验证使用 sr25519（与链上一致）

## proposal.rs — 提案查询

### 解码能力
- 投票引擎 Proposal 状态（Voting/Passed/Rejected/Executed）
- 联合投票 JointTally / CitizenTally
- 内部投票 InternalTally
- 提案元数据（创建时间、通过时间）
- 业务数据（发行分配、运行时升级 code_hash 等）

### RPC 调用
- `state_getStorage`：读取链上存储
- `state_getKeysPaged`：遍历 StorageMap

## runtime_upgrade/ — 协议升级

- 后端只负责读取 WASM、构建协议升级业务 call data、生成公民钱包签名请求、验证签名回执并提交交易。
- 前端只负责协议升级表单、WASM 选择、公民钱包签名流程和提交结果展示。
- `runtime_upgrade` 不获取 CID 人口快照，不接收联合签名上下文，不推进投票状态，不实现联合投票/公民投票流程。
- 协议升级提案进入链上后，由投票引擎统一负责投票流程、状态、计票、通过/否决判定和结果回调。

## institution.rs — 机构查询

- 管理员列表读取委托到 `admins_change/storage.rs`
- 内置机构管理员 account_id 使用 `0x01` Builtin kind tag，与 `core_const::account_id_from_cid_number` 字节级一致
- 解码管理员 AccountId 列表
- 提供机构全称/简称查询（从 CHINA_CB / CHINA_CH 常量表）

## storage_keys.rs — 存储 key 构造

- `twox_128`：Substrate pallet/storage 前缀哈希
- `blake2b_128`：Blake2_128Concat hasher
- `account_id_from_cid_number`：内置机构 AccountId 编码（与 runtime primitives 一致）
- `map_key` / `double_map_key`：完整存储 key 拼接

`AdminsChange::AdminAccounts` 专用 storage key 已收口到 `governance/admins_change/storage.rs`，不得再在通用 `storage_keys.rs` 中新增管理员更换专用读取函数。

## 依赖关系

- `shared/rpc.rs`：所有 RPC 调用通过共享 RPC 客户端
- `shared/constants.rs`：SS58 前缀、RPC 响应限制
- `settings/address_utils.rs`：SS58 地址解码
- `settings/cold-wallets/`：签名管理员（验证者）设置
