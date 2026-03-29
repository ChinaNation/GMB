# nodeui 治理子模块 — 技术文档

## 模块结构

```
governance/
├── mod.rs              # Tauri 命令入口：提案创建、投票、签名请求/提交
├── signing.rs          # QR 签名协议实现：payload 构建、签名验证、交易提交
├── proposal.rs         # 提案查询与解码：从链上 storage 读取并解析提案详情
├── institution.rs      # 机构信息查询：管理员列表、机构名称
├── storage_keys.rs     # 链上存储 key 构造：twox_128 / blake2b_128 / double_map_key
├── sfid_api.rs         # SFID 人口快照 API 客户端
└── types.rs            # 共享类型定义
```

## 核心职责

- 为前端治理页面提供所有 Tauri 命令（提案列表、提案详情、发起提案、投票、执行）
- 实现 WUMIN_SIGN_V1.0.0 QR 签名协议（离线签名设备 ↔ nodeui）
- 从链上 RPC 解码提案数据（联合投票/内部投票/机构管理员/销毁/发行/运行时升级）
- 从 SFID 服务获取人口快照（eligible_total + snapshot_nonce + signature）

## signing.rs — QR 签名协议

### 协议流程
1. nodeui 构建 `VoteSignRequest`（含 payload + metadata），生成 QR 码
2. 离线签名设备扫描 QR，使用冷钱包私钥签名
3. 签名设备生成 `QrSignResponse`，nodeui 扫描回传
4. nodeui 验证签名、构造完整 extrinsic、提交到链上

### 支持的签名类型
- 联合投票（省储会/省储行管理员投票）
- 内部投票（机构管理员替换/销毁/多签）
- 公民投票
- 决议发行提案创建
- 运行时升级提案创建
- 开发期直接升级

### 安全设计
- nonce 有 90 秒 TTL，防止重放
- payload 包含 genesis_hash 做链域隔离
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

## sfid_api.rs — 人口快照

- 默认端点：`http://147.224.14.117:8899`
- 可通过环境变量 `SFID_BASE_URL` 覆盖
- 超时：10 秒
- 返回：`PopulationSnapshot { eligible_total, snapshot_nonce, signature }`

## institution.rs — 机构查询

- 读取 `AdminsOriginGov::CurrentAdmins` 存储
- 解码管理员 AccountId 列表
- 提供机构名称查询（从 CHINA_CB / CHINA_CH 常量表）

## storage_keys.rs — 存储 key 构造

- `twox_128`：Substrate pallet/storage 前缀哈希
- `blake2b_128`：Blake2_128Concat hasher
- `shenfen_id_to_fixed48`：身份 ID 编码（与 runtime primitives 一致）
- `storage_map_key` / `double_map_key`：完整存储 key 拼接

## 依赖关系

- `shared/rpc.rs`：所有 RPC 调用通过共享 RPC 客户端
- `shared/constants.rs`：SS58 前缀、RPC 响应限制
- `settings/address_utils.rs`：SS58 地址解码
- `settings/cold-wallets/`：冷钱包公钥管理
