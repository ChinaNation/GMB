# 任务卡：全仓账户标识按 Substrate 官方模型统一

状态：执行中（2026-07-22 已完成第 1、2、3 步；第 4 步待确认）。

## 任务需求

把由助记词派生、用于签名和授权的唯一账户，在 Rust、Dart、TypeScript、SQL、JSON、SCALE、QR、Cloudflare、文档和生成物中统一为 Substrate/Polkadot SDK 的账户模型：

- 类型统一为 `AccountId`，当前 runtime 的具体实现继续是 32 字节账户标识。
- 单一账户字段统一为 `account_id`；同一结构包含多个业务角色账户时使用 `<role>_account_id`。
- 公钥统一为 `public_key`；当前签名公钥使用 `signer_public_key`；凭证签名公钥使用 `credential_signer_public_key`。
- SS58 只作为展示和输入输出地址，字段统一为 `ss58_address`，不得作为授权或持久化主键。
- `account_id` 和 32 字节公钥的文本编码统一为小写 `0x` 加 64 位十六进制，校验式为 `^0x[0-9a-f]{64}$`。
- 删除 `wallet_account`、`admin_account`、`owner_account`、`wallet_pubkey`、`admin_pubkey` 等同义命名，不保留旧格式、旧字段、双读或兼容分支。
- 当前尚未正式创世，PostgreSQL、Isar、Cloudflare D1/R2 的旧业务数据全部删除重建；不编写 migration。助记词、seed、私钥和 Keychain/Keystore 安全材料不属于业务数据，禁止删除，账户按原派生规则重新得到同一 `AccountId`。

## 不变边界

- 不改变助记词到 sr25519 公钥、`AccountId` 和 SS58 地址的派生算法，不改变账户的 32 字节值。
- 不改变 ADR-022 的抗量子路线：`AccountId` 仍是身份锚点，签名算法只是授权方式。
- 不改变机构岗位授权模型：机构权限仍必须同时满足 `cid_number + role_code + account_id`。
- 不把钱包软件、人员、管理员、机构 CID、公民 CID 与链账户混为同一概念。
- 不把具有独立业务语义的 `source`、`dest`、`beneficiary_account_id`、`sender_account_id` 等角色字段粗暴压成无角色的 `account_id`。
- CID 生成中现有账户字节如参与哈希，只改字段/变量名称，不改变参与哈希的字节、顺序、域或 CID 结果；若发现必须改变协议字节，立即停止并另行确认。

## 最终命名契约

| 语义 | 唯一目标命名 | 规则 |
|---|---|---|
| runtime 账户类型 | `AccountId` | 具体 32 字节类型由 runtime 配置决定 |
| 唯一/通用账户字段 | `account_id` | 不得使用 wallet/admin/owner 等载体或身份词替代 |
| 多账户业务角色 | `<role>_account_id` | 例如 `actor_account_id`、`voter_account_id`、`sender_account_id` |
| 公钥 | `public_key` | 只有数据确实是签名公钥时使用 |
| 当前签名公钥 | `signer_public_key` | 验签后转换为签名账户，再与目标 `account_id` 比较 |
| 凭证签名公钥 | `credential_signer_public_key` | 仅用于凭证签名角色 |
| 展示地址 | `ss58_address` | 派生展示值，不是主键或授权真源 |
| 文本账户/公钥 | `0x` + 64 位小写 hex | 禁止无前缀、大写、混合大小写和 SS58 混存 |

目标 runtime 结构包括但不限于：

```text
Admin { account_id, family_name, given_name }
PublicAdmin { account_id, cid_number, family_name, given_name }
CitizenSubject { cid_number, account_id }
VotingIdentityPayload { cid_number, account_id, ... }
InstitutionAdminAssignment { cid_number, account_id, role_code, ... }
InstitutionVoteTicket { role_subject, voter_account_id }
```

目标 citizen-identity 双向索引：

```text
AccountIdByCid[cid_number] -> AccountId
CidByAccountId[account_id] -> cid_number
```

## 分步骤实施

### 第 1 步：冻结命名、边界与实施规则

状态：已完成。

- 新增 ADR-040，固定账户、公钥、SS58 和文本编码边界。
- 更新 AI 硬规则、统一协议入口和产品架构文档。
- 只建立目标契约，不修改 runtime、数据库、客户端代码或生成物。

完成记录（2026-07-22）：

- 已更新 `memory/AGENTS.md`、`memory/07-ai/agent-rules.md` 和 `memory/07-ai/unified-protocols.md`，禁止新代码继续产生旧账户同义字段。
- 已更新 GMB、CitizenChain、CitizenApp、OnChina 和 QR registry 架构入口，明确账户、公钥、SS58 与数据重建边界。
- 已在 ADR-023、ADR-039 标注：原权限模型继续有效，旧账户字段目标命名由 ADR-040 取代。
- 已检查 Markdown diff、文件名 UTF-8 字节长度和未授权代码范围；本步没有产生 runtime、数据库、客户端或生成物 diff。

### 第 2 步：runtime 账户类型、结构、存储与权限入口统一

状态：已完成。

- 统一共享类型、admins、citizen-identity、entity 任职、投票票据及直接消费这些类型的业务模块。
- SCALE 结构、storage 名称、事件、错误、benchmark、mock、测试和中文注释一次同步。
- runtime 版本和 StorageVersion 保持 `0`；重新创世，不写 migration。

完成记录（2026-07-22）：

- Runtime 单一账户字段已统一为 `account_id`，多账户结构已按角色统一为
  `<role>_account_id`；管理员、公民主体、机构任职、投票票据、发行、交易、治理、
  benchmark、mock 和测试同步更新，不改变账户字节、签名载荷顺序或 CID 派生材料。
- citizen-identity 双向索引已改为
  `AccountIdByCid[cid_number] -> AccountId` 与
  `CidByAccountId[account_id] -> cid_number`；fullnode 奖励索引已改为
  `RewardAccountIdByMiner`，绑定调用改为 `bind_reward_account` /
  `rebind_reward_account`。
- NodeGuard、Node/OnChina 直接 SCALE 消费者已同步新 Runtime 结构与 storage key；
  Node 与 OnChina 自身对外 DTO、数据库、RPC、桌面命令等命名继续分别归第 3、4 步，
  没有在本步建立兼容字段。
- 已清除 Runtime 代码、注释和架构文档中把账户称作钱包的残留；`CitizenWallet`、
  `CitizenApp`、冷钱包/热钱包等确实表示软件产品或签名载体的表述保留。
- `spec_version`、`transaction_version` 和所有显式 pallet `StorageVersion` 保持 `0`；
  没有 migration、兼容分支或冻结 chainspec 改写。
- `cargo check --workspace --all-targets` 与
  `cargo test --workspace --all-targets` 全部通过；Node 289 项、OnChina 137 项及所有
  Runtime/共享 crate 测试零失败。仅保留任务前已存在的
  `node/src/transaction/offchain/mod.rs::acc` 未使用警告。
- 使用当前源码编译的 WASM 启动隔离
  `--chain citizenchain-fresh --tmp` 节点，节点守卫启动自检通过、无外部同步；
  Runtime 六项版本均为 `0`。真实 metadata 包含 `AccountIdByCid`、
  `CidByAccountId`、`RewardAccountIdByMiner`，不包含对应三个旧 storage 名称；
  公权管理员新四字段 SCALE、私权基金会管理员与机构记录均已在 fresh 创世状态中核验。
- `--dev` 仍按既定设计读取旧冻结 chainspec；冻结 chainspec 与全部业务数据的删除重建
  留在第 10 步统一执行，本步没有提前改变现有冻结链或用户安全材料。

### 第 3 步：Node 与 runtime 外 Rust 代码统一

状态：已完成。

- 统一节点 RPC、索引、桌面端桥接、签名验证、CLI 和测试夹具。
- 不改变账户字节和签名协议，只统一语义命名和规范化边界。

完整技术方案：

1. 先盘点 `citizenchain/node/` 与共享 Rust crates 中仍表示同一链账户的
   `wallet_*`、`admin_*`、`owner_*`、裸 `address`、裸 `pubkey` 字段，逐项区分
   `account_id`、角色账户、`public_key`、`signer_public_key` 与
   `ss58_address`；钱包产品、管理员人员角色和协议中的真实业务名称不做误替换。
2. Node 内部二进制账户统一使用 `AccountId` / `[u8; 32]` 并命名为
   `account_id` 或 `<role>_account_id`。跨进程、JSON、缓存和 RPC 中表示账户 ID 的
   文本统一为严格小写 `0x` + 64 位 hex；表示展示地址的字段只允许
   `ss58_address`，进入授权逻辑前必须解析成账户字节。
3. 管理员管理、治理、转账、清算、挖矿奖励、节点设置、节点守卫和签名流程中的 DTO、
   事件、错误、函数、参数、局部变量与中文注释同步改名。管理员人员集合仍叫
   `admins`；机构权限仍按 `cid_number + role_code + account_id` 校验。
4. `reward_bindWallet` / `reward_rebindWallet`、桌面命令、Tauri 参数与结果、本地
   `reward-wallet.json` 等旧 Node 私有协议直接切换为 account-id 目标命名，不保留旧
   RPC 方法、旧命令、旧 JSON 或 fallback。`node/frontend/` 的直接消费者必须在同一步
   同步，避免 Rust 桥接与桌面页面形成断裂。
5. 签名流程统一区分 `signer_public_key` 与 `signer_account_id`：先严格验证公钥和签名，
   再得到并比较业务账户。现有 payload 字节、domain、字段顺序、签名算法和 CID/账户
   派生规则保持不变；任何会改变协议字节的发现立即停止沟通。
6. 删除 Node 非密钥缓存和测试夹具中的旧账户字段并按最终结构重建；不得删除助记词、
   seed、私钥、Keychain、keystore、节点身份密钥或 GRANDPA 密钥。冻结 chainspec、
   PostgreSQL、Isar、D1/R2 不在本步处理。
7. 同步 Node 架构文档、测试说明和任务卡，清理旧代码、注释、错误文案、RPC 名、
   Tauri 命令、JSON key、缓存文件名和测试 fixture 残留；不新增兼容层。
8. 验收依次执行精确格式化、Node/共享 crate 单测、workspace check/test、Node build，
   再启动隔离 fresh 节点，真实调用改名后的 RPC/桌面后端路径并核验
   `0x` 小写账户输出、SS58 展示字段、签名验权和节点守卫。编译和单测不代替真实运行验收。

预计修改目录：

- `citizenchain/node/src/`：代码；统一 Node RPC、管理员/治理、交易、奖励账户、设置、
  签名、节点守卫、CLI 与桌面桥接的账户、公钥、SS58 命名和校验。
- `citizenchain/node/frontend/`：代码与残留清理；同步 Node 桌面直接消费者的命令、
  参数、JSON 字段和界面文案，不扩展到 OnChina 前端。
- `citizenchain/crates/chain-signing/`：代码与测试；统一共享签名材料中的
  `signer_public_key` / `signer_account_id` 语义，不改变签名字节。
- `citizenchain/crates/blockchain-test-harness/`：测试夹具；仅在坏块、状态或 RPC
  fixture 直接引用旧 Node 账户字段时同步。
- `memory/01-architecture/citizenchain/`：文档；记录 Node 最终账户边界、RPC 和桌面桥接。
- `memory/04-decisions/`：文档与残留清理；仅同步 ADR-040 在 Node 层的实施状态。
- `memory/08-tasks/`：文档；回写第 3 步执行、验收和下一步方案。

本步预计不修改 `citizenchain/runtime/`、`citizenchain/onchina/`、CitizenApp、
CitizenWallet、Cloudflare、QR registry、冻结 chainspec 或业务数据库；若编译或协议
核对发现必须修改 Runtime，必须先停止并重新列出完整 Runtime 路径取得二次确认。

完成记录（2026-07-22）：

- Node 管理员、治理、交易、清算、挖矿、设置、节点守卫和桌面桥接中的账户字段已统一
  为 `account_id` / `<role>_account_id`，公钥统一为 `public_key` /
  `signer_public_key`，SS58 展示值统一为 `ss58_address`。账户与公钥文本入口严格
  拒绝无 `0x`、大写、混合大小写、错误长度和非十六进制输入。
- `chain-signing` 已统一公钥解析和签名账户派生；签名流程先校验
  `signer_public_key`，再得到并比较 `signer_account_id`，没有改变 payload、domain、
  SCALE 字段顺序、签名算法、CID 材料或账户字节。
- 奖励账户私有 RPC 已改为 `reward_bindAccount` / `reward_rebindAccount`，桌面命令、
  DTO、本地存储和模块路径已改为奖励账户语义。本地非密钥配置固定为
  `reward-account.json`；旧 RPC、旧文件、旧 JSON、旧源码文件名和兼容读取均已删除。
- Node 激活缓存、挖矿缓存和桌面账户列表只接受当前 schema；已删除开发配置下
  `cold-wallets.json`、`mining-dashboard-cache.json`、`reward-wallet.json` 旧业务缓存。
  macOS Keychain、节点 keystore、助记词、seed、私钥、节点身份密钥和 GRANDPA 密钥
  未删除、未迁移。
- 经明确确认，仅修正
  `citizenchain/onchina/src/core/chain_submit.rs` 对 `chain-signing` 的直接调用，
  同步 `public_key` / `signer_public_key` 局部语义；没有修改 OnChina 数据库、HTTP
  接口、业务流程或授权规则，其完整统一仍归第 4 步。
- Node 架构、奖励账户、GRANDPA、keystore、管理员、治理、交易、NodeGuard 和主页
  文档已同步当前实现；Node 本步范围内旧字段、旧 RPC、旧文件路径、旧注释和旧文案
  扫描为零。QR 线上 `pubkey` 字段和 action 名属于第 5 步，不在本步提前改协议。
- `cargo fmt -p node -p chain-signing`、OnChina 单文件 rustfmt、
  `cargo check --workspace --all-targets` 和 `cargo test --workspace --all-targets`
  全部通过。Node 289 项、OnChina 137 项、`chain-signing` 6 项以及全部 Runtime/
  共享 crate 测试零失败。
- Node 前端 `tsc --noEmit` 与 Vite 生产构建通过，共转换 116 个模块；仅保留 Vite
  超过 500 kB 的既有 chunk 体积警告。
- 使用当前源码重新构建 WASM 与 Release Node，启动隔离
  `citizenchain-fresh --tmp` 新链。`system_health.isSyncing=false`，
  `ss58Format=2027`，block#0 为
  `0x64c67e4aaf38be35562be3a494a6a12d06ea7e3881d3c2270e82c6225bf905f3`；
  方法表包含两个新奖励账户 RPC 且无旧钱包 RPC。非法文本和大写账户 ID 均被真实
  RPC 拒绝为“小写 `0x` + 64 位十六进制”。验收节点已正常停止，临时链数据未保留。

### 第 4 步：OnChina 后端、前端与 PostgreSQL 重建

状态：完整技术方案已输出，待确认。

完整技术方案：

1. 先以 `citizenchain/onchina/src/core/db.rs` 的当前建表入口为中心，逐项盘点
   PostgreSQL 列、Rust model/repo/service、HTTP 路由与 JSON、React 类型与请求参数中的
   账户、公钥和 SS58 语义。单一账户统一为 `account_id`，多账户交易按业务角色使用
   `<role>_account_id`，公钥使用 `public_key` / `signer_public_key` /
   `credential_signer_public_key`，展示地址只允许使用 `ss58_address`。
2. OnChina 后端所有账户文本入口统一执行 `^0x[0-9a-f]{64}$`，直接拒绝无 `0x`、
   大写、混合大小写、长度错误、裸 SS58 和把公钥字段冒充账户字段的输入；SQL、
   Rust、JSON 和 TypeScript 不做大小写兼容、alias、fallback、双读或双写。
3. 登录、会话、Passkey、管理员名册和机构岗位授权统一以 `account_id` 为身份锚点。
   签名请求明确保存 `signer_public_key`，验签后得到 `signer_account_id`，再与
   `account_id` 比较；登录成功只证明账户身份，业务操作仍必须按链上
   `cid_number + role_code + account_id` 校验，不建立 OnChina 独立授权真源。
4. 公民本地档案和链上身份推送不再保存 `wallet_pubkey` / `wallet_address`。
   表示链账户的字段改为 `account_id`，只在确实需要展示时派生
   `ss58_address`；注册局管理员与公民双签的业务规则、CID 永久性、公民资料字段和
   citizen-identity 真源边界不变。
5. 交易索引、机构账户、创建人/上传人和审计记录按真实角色改名，例如
   `sender_account_id`、`recipient_account_id`、`creator_account_id`、
   `uploader_account_id`。现有 `/app/wallet/:address/...` 等把账户错误描述为钱包或
   裸地址的 API 同步改成账户语义，后端和前端一次切换，不保留旧路由。
6. 删除 `x-wallet-pubkey`、`admin_account`、`bound_admin_pubkey`、宽松公钥解析、
   Serde alias、旧错误码文案和旧测试 fixture；替换为最终账户/签名公钥字段。
   QR action registry 的压缩线协议字段和其他产品生成物归第 5 步，本步只同步
   OnChina 自身 HTTP/JSON 载荷，不擅自改变 QR 协议字节。
7. PostgreSQL 不写 migration：停止 OnChina 后，先只读确认实际连接目标和本地
   embedded PostgreSQL 实例，再删除当前 OnChina 业务数据库并由最终
   `init_current_schema` 一次重建。删除范围只包括 OnChina PostgreSQL 业务结构和
   数据，不删除数据库程序、日志目录、Keychain、节点 keystore、私钥、Secret 或
   其他产品数据；远程数据库若与预期不一致立即停止沟通。
8. 把 `core/db.rs` 中用于迁移旧列的 `ALTER ... DROP/ADD`、旧 schema 清理和兼容
   修复改为唯一的创世前最终建表定义；所有索引、唯一约束、外键、查询、投影同步
   使用最终列名。数据库重建后执行 schema 审计，必须不存在旧列和旧索引。
9. 同步 OnChina 前端所有类型、状态、API 参数、表格字段、登录流程和文案；账户值按
   `account_id` 传输，SS58 仅在明确展示组件中使用 `ss58_address`。不新建
   `frontend/api/` 或独立 chain 目录，业务 API 和链交互继续留在所属模块。
10. 更新 OnChina 架构文档、ADR-040 和任务卡，清理代码、SQL、JSON、错误文案、
    注释、测试、前端构建产物与文档中的本步旧字段；不新增兼容层。
11. 验收依次执行 Rust 精确格式化、OnChina 单测、workspace check/test、前端
    typecheck/build；随后启动真实本地 PostgreSQL、OnChina 后端和 fresh 节点，
    通过真实 HTTP 验证登录挑战与验签、会话读取、机构岗位权限、公民链上身份准备、
    账户查询和页面字段。最后查询 `information_schema` 证明旧列为零，并验证旧
    HTTP/JSON 字段与路由被拒绝。

预计修改目录：

- `citizenchain/onchina/src/core/`：代码、SQL 与残留清理；重建唯一 PostgreSQL
  schema、严格账户校验、HTTP 安全头、链账户解析和共享数据库边界。
- `citizenchain/onchina/src/auth/`：代码与测试；统一登录、会话、Passkey、挑战、
  验签和授权上下文中的 `account_id` / `signer_public_key`。
- `citizenchain/onchina/src/institution/`：代码与测试；统一机构管理员、岗位任职、
  主体和机构账户模型，继续以链上 `cid_number + role_code + account_id` 为权限真源。
- `citizenchain/onchina/src/domains/`：代码与测试；按各业务真实角色统一公民、机构、
  文档、地址、立法和私权业务的账户字段，不改变业务流程。
- `citizenchain/onchina/src/indexer/`：代码与测试；统一交易索引路由、查询参数和
  发送方/接收方账户列，删除裸 address 主键语义。
- `citizenchain/onchina/src/citizenapp/`：代码与测试；同步 OnChina 提供给
  CitizenApp 的账户 JSON 边界，不改 CitizenApp 客户端本体。
- `citizenchain/onchina/src/cid/`、`citizenchain/onchina/src/scope/`、
  `citizenchain/onchina/src/workspace/`、`citizenchain/onchina/src/main.rs`：
  代码与残留清理；仅同步直接消费账户字段的 CID 材料变量、范围上下文、工作台和
  路由装配，不改变 CID 字节、范围规则或业务权限。
- `citizenchain/onchina/frontend/`：代码、构建产物与残留清理；同步 React 类型、
  API、登录、页面展示和文案，不扩展到 CitizenApp/CitizenWallet。
- OnChina 当前实际使用的本地 PostgreSQL 业务数据库：数据与 schema 删除重建；
  仅在执行时只读确认目标后操作，不涉及仓库新增文件，不触碰密钥或其他数据库。
- `memory/01-architecture/onchina/`：文档；记录 OnChina 最终账户、验签、授权、
  API 和数据库边界。
- `memory/04-decisions/`：文档；同步 ADR-040 的 OnChina 实施状态。
- `memory/08-tasks/`：文档；回写第 4 步执行、真实验收和第 5 步完整技术方案。

本步预计不修改 `citizenchain/runtime/`、QR action registry、CitizenApp、
CitizenWallet、Cloudflare、CitizenConsole 或冻结 chainspec；如果核对发现必须改变
QR/SCALE/CID/签名协议字节或 Runtime，立即停止并按对应路径重新取得确认。

### 第 5 步：QR 协议与生成物统一

状态：待确认。

- 修改唯一 action registry、字段字典、Rust 生成器和所有消费端生成物。
- 同步签名请求/响应和 SCALE 解码字段，不保留旧字段。

### 第 6 步：CitizenApp 与 Isar 重建

状态：待确认。

- Dart 模型、服务、状态、RPC、UI、测试统一为目标命名。
- 删除并重建 Isar 业务库；保留 secure storage 中助记词/seed/私钥并重新派生同一账户。

### 第 7 步：CitizenWallet 统一

状态：待确认。

- 离线签名、二维码、账户展示和测试统一；不改助记词派生和私钥安全存储。

### 第 8 步：Cloudflare、D1/R2 与边缘协议重建

状态：待确认。

- Worker、Durable Object、D1、R2 key、JSON 和测试统一。
- 删除并重建旧 D1/R2 业务数据，不删除密钥或 Secret。

### 第 9 步：CitizenConsole、脚本、CI、文档和全仓残留清理

状态：待确认。

- 统一剩余工具、控制台、部署脚本、fixture 和所有文档。
- 全仓扫描并清除旧同义字段、旧注释、旧文案、旧 schema 和旧生成物。

### 第 10 步：真实运行态总验收

状态：待确认。

- 重新创世并重建 PostgreSQL、Isar、D1/R2 业务数据。
- 真实验证账户恢复、登录、机构岗位授权、公民身份、扫码签名、投票、转账、Cloudflare 会话和页面展示。
- 编译、静态检查、单元测试和 build 只是前置，不代替真实运行态验收。

## 每步共同完成条件

- 先输出该步完整技术方案和“预计修改目录”，取得确认后才执行。
- 涉及 `citizenchain/runtime/` 时必须单独列出完整路径并取得二次确认。
- 改代码后同步更新文档、完善中文注释、清理旧代码/注释/文档/配置/数据残留。
- 不新增兼容层，不保留旧字段，不推送 GitHub。
