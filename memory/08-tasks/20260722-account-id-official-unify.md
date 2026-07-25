# 任务卡：全仓账户标识按 Substrate 官方模型统一

状态：实现完成（2026-07-24 已完成第 1—10 步；真机、fresh 链、隔离数据库、
Cloudflare staging 和全量测试证据见第 10 步执行结果）。

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
Admin { account_id, cid_number, family_name, given_name }
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

状态：已完成。

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

执行记录：

- OnChina 后端、前端、PostgreSQL 最终 schema、HTTP/JSON、登录、Passkey、会话、
  管理员、机构岗位、公民档案、交易索引、审计和机构账户中的账户语义已统一为
  `account_id` 或准确的 `<role>_account_id`；签名公钥使用
  `public_key` / `signer_public_key` / `actor_public_key`，SS58 只作为
  `ss58_address` 派生展示值。
- 账户文本边界固定为 `^0x[0-9a-f]{64}$`。已删除旧账户字段、Serde alias、旧
  `x-wallet-*` 请求头、旧钱包交易路由、宽松大小写/前缀/SS58 fallback，以及旧
  PostgreSQL migration、兼容列和兼容索引。
- `citizens.account_id` 可空，用于公民身份尚未完成链上绑定的本地档案；公民身份与
  账户的一一对应继续由链上 citizen-identity 负责，OnChina 不建立第二真源。
  `admin_login_sign_requests.account_id` 必须非空：浏览器先扫描完整 `k=3 user_contact`
  用户码确定目标账户，后端完成链上管理员前置校验后才生成登录请求。
- 已删除并重建本机 `127.0.0.1:5433/onchina` 业务数据库，最终数据目录为
  `/Users/rhett/Library/Application Support/gmb.dev/onchina-pgdata`。两个开发期旧
  业务数据目录已移入 macOS 废纸篓，密钥、Secret、节点 keystore 和其它产品数据
  未触碰。
- PostgreSQL 真实审计结果：旧账户列为 `0`、旧账户索引为 `0`、业务数据行为 `0`；
  账户/公钥 CHECK 约束存在；登录挑战的 `account_id` 为 `NOT NULL`，非法
  `0xABC` 和大写账户值被拒绝。
- `cargo check -p onchina`、`cargo test -p onchina`（140 项）、
  `cargo check --workspace --all-targets`、`cargo test --workspace --all-targets`
  和 OnChina 前端 production build 均已通过；最终 Rust 格式化与 diff 检查无误。
- 经用户单独二次确认，已从当前 Runtime 源码生成
  `citizenchain.compact.compressed.wasm`，SHA-256 为
  `6fdd4cf2f7b5b884a63c680ecde5fd4dada73ea7df3e816b9b115129b68afcbb`；
  本次只生成并消费 WASM，没有修改 Runtime 源码、版本、chainspec 或业务逻辑。
- 当前 WASM 已注入本地 Node 并启动隔离 `citizenchain-fresh` 新链。真实创世块哈希为
  `0x49f1da82260414adbfb72ce085d8520dbf56d1413b60f583af7722955e877458`，
  state root 为
  `0x7a4505bd90b628815a5fd974282168fb578eacde77ab3b45ea2f9d191cd98298`；
  `spec_version`、`transaction_version`、`authoring_version`、`impl_version`、
  `system_version` 和 `state_version` 均为 `0`，节点无对等连接且未同步外部链。
- 已以真实 PostgreSQL、fresh 节点和 HTTPS OnChina 启动完整运行态：
  链上机构 `49,593`、机构账户 `99,231` 与本地投影一致，34 个创世机构样本审计通过，
  索引器锚定上述创世块并追平 block `0`；健康接口、生产前端 HTML 和 JS 静态资源均
  返回 `200`。
- 真实 HTTP 已证明：规范小写 `0x` 加 64 位 hex 的账户请求可进入当前流程；大写、
  无前缀、SS58、旧 `admin_account` JSON 和旧钱包交易路由被拒绝；新
  `account_id` 交易路由、机构账户和省级机构查询返回最终字段。无 token、伪 token、
  签名账户不匹配和伪签名分别按安全边界拒绝。
- 使用开发密钥 `//Alice` 对真实挑战原文生成有效 sr25519 签名后，验签成功并继续到
  链上管理员门禁，最终因该账户不是链上管理员而以
  `ONCHINA_LOGIN_ADMIN_NOT_ONCHAIN` 拒绝，证明“验签成功不等于管理员授权”。
  仓库和本次验收环境不持有创世管理员私钥，因此没有伪造或读取 Keychain 来制造成功
  管理员会话；链上岗位授权成功路径继续由 Runtime/OnChina 自动化测试覆盖。
- 验收后已正常停止 OnChina 与隔离节点，再次删除重建本机 `onchina` 业务数据库。
  最终数据库业务行和运行态行为 `0`、旧账户列 `0`、旧账户索引 `0`，账户约束存在；
  PostgreSQL 已停止，端口 `5433`、`9944`、`8964` 均关闭。隔离临时目录已移入
  macOS 废纸篓，密钥、Secret、节点 keystore 和其他产品数据未触碰。

### 第 5 步：QR 协议与生成物统一

状态：已完成。

完整技术方案：

1. 以 `citizenchain/crates/qr-protocol/registry/` 为扫码动作、字段中文名和拒绝原因的
   唯一真源，先对每个 action 的 `required_fields` 与当前 Runtime call/共享结构逐项
   对照。只改账户、公钥字段的语义名称，不修改 action key、action code、pallet、
   call、decoder、签名分类或 hash-only 白名单。
2. 单一账户字段统一为 `account_id`；多账户载荷按真实业务角色改为
   `<role>_account_id`。确定映射包括：
   `wallet_account -> account_id`、管理员结构内
   `admin_account -> account_id`、`owner_account -> owner_account_id`、
   `operator_account -> operator_account_id`、
   `target_account -> target_account_id`、
   `institution_account -> institution_account_id`、
   `personal_account -> personal_account_id`、
   `execution_account -> execution_account_id`、
   `funding_account -> funding_account_id`、
   `beneficiary -> beneficiary_account_id`。裸 `account`、`from`、`to`、`target`、
   `who`、`bank_main`、`new_bank` 和费用付款账户先按对应 Runtime call 的真实角色
   分别落为 `account_id`、`sender_account_id`、`recipient_account_id`、
   `target_account_id`、`bank_main_account_id`、`new_bank_account_id` 或准确的
   `<role>_account_id`，禁止机械替换或丢失角色。
3. 公钥字段统一为 `public_key`、`signer_public_key` 和
   `credential_signer_public_key`。`signer_pubkey` 固定改为
   `signer_public_key`，`credential_signer_pubkey` 固定改为
   `credential_signer_public_key`；`actor_pubkey`、`admin_pubkey` 必须先判断其
   字节在该动作中是签名公钥还是账户标识，再分别落为
   `actor_public_key` / `signer_public_key` 或 `actor_account_id` /
   `account_id`，不得继续用 `pubkey` 混称账户。
4. QR_V1 压缩线协议保持逐字节不变：顶层 `p/k/i/e/b`、body 的 `a/g/u/d/s`、
   `k=1/2`、base64url 编码、签名原文
   `QR_V1|<k>|<id>|<system>|<expires_at>|<principal>`、SCALE 字段顺序和待签 payload
   字节全部不变。`u` 仍是压缩传输键，但 Rust/Dart/TypeScript 内部语义属性统一为
   `signer_public_key`；若任何改名会改变 QR 文本、SCALE、哈希或签名原文，立即停止
   沟通，不继续执行。
5. 更新 `qr-protocol` 的 registry 解析、Dart 导出器、唯一性测试和 repo guard，
   新增针对旧账户/公钥字段的禁止清单，保证 required field 全部有中文标签、两个
   移动端生成物完全一致、端侧不能恢复第二动作表、旧字段或第三签名状态。
6. 从唯一 registry 重新生成 CitizenApp 与 CitizenWallet 的
   `qr_action_registry.g.dart`，禁止手改生成物。生成前后校验 action code 集合、
   hash-only 集合和拒绝原因集合完全相同，只有目标字段键及其准确中文标签变化。
7. 同步 CitizenWallet 的 SCALE 解码器、确认页字段、管理员数组 JSON、公民身份、
   机构/个人多签、资产、转账、清算、投票和治理动作。解码出来的账户机器值统一为
   小写 `0x` 加 64 位 hex；SS58 只能以 `ss58_address` 用于明确展示，不能再把
   `_bytesToSs58(...)` 的结果放进 `account_id` 字段或参与授权。
8. 同步 CitizenApp 的 QR request/response body、签名服务、广场账户动作和扫码路由；
   同步 OnChina、Node 的 QR 请求生产者与响应消费者。各端对外语义统一使用
   `signer_public_key` / `account_id`，压缩键和签名字节不变；第 4 步暂留的
   `pubkey` 语义属性在本步一次清除，不保留 alias、双读或旧 JSON。
9. 清理代码、注释、文档、fixture 和生成物中的旧账户/公钥表述，同时保留确实表示
   CitizenWallet/CitizenApp 产品、钱包选择器或私钥容器的 `wallet` 名称。更新 QR
   协议规范、动作注册表说明、统一协议入口、ADR-040 和本任务卡。
10. 静态验收执行 `cargo test -p qr-protocol`、registry 重新导出一致性检查、
    受影响 Rust crate 的格式化/check/test、CitizenApp/CitizenWallet 的 Dart
    format/analyze/targeted tests，以及 OnChina 前端 build/后端测试。全仓扫描必须证明
    QR 范围内旧字段、旧中文标签、手写动作表和兼容分支为零。
11. 真实运行态验收至少覆盖三类往返：非链登录签名、普通链交易签名、公民身份或机构
    岗位业务签名。由真实生产者生成 QR_V1 请求，扫码端真实解析并展示目标字段，使用
    实际 sr25519 密钥签名，再由请求端回扫、验签并验证 request id 防重放；同时证明
    旧字段 JSON 被拒绝、未知字段/未知动作红色拒签、hash-only 仍只限 Runtime 升级。
    如缺少某产品真实密钥或运行环境，只能使用仓库已有开发密钥和隔离数据，不得读取
    用户 Keychain、助记词或生产 Secret。

预计修改目录：

- `citizenchain/crates/qr-protocol/registry/`：协议数据与残留清理；统一 action
  required fields、字段中文名和旧字段禁用范围，不改变动作码或压缩线协议。
- `citizenchain/crates/qr-protocol/src/`、`citizenchain/crates/qr-protocol/tests/`：
  Rust 代码与测试；同步 registry 导出、语义类型、一致性校验和全仓 guard。
- `citizenapp/lib/qr/`、`citizenapp/lib/signer/`：Dart 代码、生成物与残留清理；
  同步签名请求/响应、广场动作和扫码消费端，不改变密钥派生或签名算法。
- `citizenapp/test/`：测试；只更新或补充 QR/签名字段、拒绝旧字段和逐字节不变验证，
  不重建 CitizenApp Isar，Isar 归第 6 步。
- `citizenwallet/lib/qr/`、`citizenwallet/lib/signer/`：Dart 代码、生成物与残留清理；
  同步离线扫码、SCALE 解码、确认页字段和签名响应，不改变安全存储。
- `citizenwallet/test/`：测试；同步 QR registry、payload 解码、字段展示、拒签和
  回扫验签用例。
- `citizenchain/onchina/src/core/qr/`、直接调用这些 QR 类型的
  `citizenchain/onchina/src/auth/` 与所属业务模块：Rust 代码与测试；只统一 QR
  request/response 语义字段，不修改 OnChina 业务权限或 PostgreSQL schema。
- `citizenchain/onchina/frontend/core/citizenQr.ts` 及直接消费它的所属功能模块：
  TypeScript 代码与测试；统一解析结果语义，保持 `p/k/i/e/b` 和 `a/g/u/d/s` 不变，
  不新建独立 API 或 chain 目录。
- `citizenchain/node/src/admins/`、`citizenchain/node/src/governance/`、
  `citizenchain/node/src/transaction/`：Rust 代码与测试；只同步现有 QR
  生产者/消费者和注释，不改变管理员、投票或交易业务规则。
- `memory/01-architecture/qr/`：协议文档与 fixture；记录最终账户、公钥和 SS58
  字段，同时固定压缩键和逐字节兼容边界。
- `memory/07-ai/unified-protocols.md`：文档；同步全仓 QR 字段契约和禁止旧字段规则。
- `memory/04-decisions/ADR-040-account-id-official-standard.md`：文档；回写第 5 步
  实施状态。
- `memory/08-tasks/20260722-account-id-official-unify.md`：文档；回写执行、真实验收和
  第 6 步完整技术方案。

完成记录（2026-07-23）：

- 账户、公钥、SS58 的 QR 语义字段已按 registry 单源统一；两端 Dart 生成物已重新
  导出，动作码、压缩键和 hash-only 范围未变。登录 action 已删除废弃
  `system_signature`，payload 固定为 UTF-8 `onchina`。
- OnChina 登录改为先扫描严格 `QR_V1/k=3 user_contact`，只从
  `b.ss58_address` 派生目标 `account_id` 并先查链上管理员；随后生成
  `QR_V1/k=1,a=1` 定向请求，`b.u` 永不为空。完成接口禁止浏览器提交或改写目标账户，
  严格比较挑战目标、响应签名公钥和实际验签账户。
- PostgreSQL `admin_login_sign_requests.account_id` 已删除可空路径并固定为
  `NOT NULL + ^0x[0-9a-f]{64}$`。旧 identify/challenge/verify 路由和 DTO 已删除。
- CitizenWallet 在解析请求后和调用私钥前两次校验当前钱包公钥等于 `b.u`；通用管理员
  载荷解码器已统一读取 `account_id + cid_number + family_name + given_name`，公权、
  私权和个人多签不再采用不同 SCALE 字段布局。
- 对照真实链时发现本地原生类型已四字段而旧内嵌 WASM 仍是三字段。经用户对列出的
  Runtime 路径再次明确授权后，已统一 Node/OnChina 私权解码器、基金会创世管理员
  公民 CID，并从当前源码重新生成 WASM；未提高任何 Runtime 版本，也未增加 migration
  或旧布局兼容。最终源码 WASM 为 6,025,415 字节，SHA-256 为
  `e14f8e483eb63e142cd660df3be8d40240097832e22c0ba7ba43a9387e5e7b10`。
- 最终 fresh 链创世哈希为
  `0x04b7b196356919eaaa0103ec76037ed59cd30214fdb373618049c85d225c7ea8`。
  `PrivateAdmins::AdminAccounts` 原始 SCALE 已真实包含程伟的
  `account_id`、`GZ000-CTZN6-198805200-2026`、`family_name=程`、`given_name=伟`。
- OnChina 真实投影再次完成 49,593 个机构、99,231 个账户和 34 个创世样本对账。
  真实登录请求返回非空 `b.u`，解码后精确等于
  `0xd6d73cfd7d6b7c5692749b7c46fd3fe398f16f84283910dbf15f74472e1e3938`；
  错误签名者返回 403，伪签名返回 422，失败均不消费挑战。仓库不持有该创世管理员
  私钥，因此未读取 Keychain、助记词或 Secret 伪造成功会话。
- `cargo fmt --check`、Node/OnChina/Runtime 定向测试、`qr-protocol` 全部测试、
  CitizenWallet analyze 与 104 项 payload decoder 测试、登录定向测试以及 OnChina
  前端 production build 均通过。验收后 OnChina、PostgreSQL 和隔离节点已正常停止。

### 第 6 步：CitizenApp 与 Isar 重建

状态：已完成。

完整技术方案：

1. 先审计 CitizenApp 的 Isar schema、collection、索引、服务、状态和 UI 边界，按
   “账户身份 / 签名公钥 / SS58 展示地址 / 业务角色账户”四类逐字段建立最终映射。
   不把产品名中的 wallet、公民 CID、机构 CID 或普通展示地址误改成账户主键。
2. Dart 模型中的唯一账户统一为 `accountId`，持久化 JSON/Isar 字段统一为
   `account_id`；多账户按业务角色使用 `<role>AccountId` /
   `<role>_account_id`。签名公钥统一为 `publicKey` / `signerPublicKey`，SS58 统一为
   `ss58Address`，只在展示和扫码边界派生。
3. 删除旧 `walletAccount`、`adminAccount`、`ownerAccount`、裸 `account/address`
   主键和相关 alias、fallback、双读、复制 getter、旧索引及旧序列化键。管理员继续
   使用统一四字段 `account_id + cid_number + family_name + given_name`，机构授权仍由
   `cid_number + role_code + account_id` 三者共同决定。
4. 同步账户恢复、钱包列表、当前账户、机构管理员/岗位、提案、投票、转账、公民身份、
   广场、Cloudflare 会话和 QR 签名页面；只改账户表达，不改变助记词派生、签名算法、
   Runtime call、SCALE 顺序、CID 生成或业务权限。
5. 删除并重建 CitizenApp 的 Isar 业务数据库与生成 schema，不写 migration、不保留
   旧 collection 或旧字段。secure storage / Keychain 中的助记词、seed、私钥和
   生物识别保护材料必须原样保留；重启后用同一安全材料重新派生相同 AccountId。
6. 重新运行 Isar/build_runner 生成流程，生成物只能来自目标模型；禁止手改生成代码。
   更新既有 fixture、golden 和测试，不新增兼容 fixture。
7. 静态验收执行 Dart format、`flutter analyze`、受影响定向测试和 CitizenApp 全量
   `flutter test`；全仓扫描 CitizenApp 范围旧字段、旧注释、旧文案、旧 schema 和
   alias 为零。
8. 真实运行态验收使用本机 CitizenApp：备份并删除业务 Isar、保留安全存储，真实启动
   后恢复同一账户；验证账户列表、SS58 展示、机构岗位授权、公民身份、扫码签名、
   投票和转账页面均使用目标字段且无旧数据回填。不得读取或输出用户助记词/私钥。

预计修改目录：

- `citizenapp/lib/db/`、实际 Isar collection 所在目录及对应生成物：Dart 模型、schema
  与残留清理；统一持久化字段并删除旧业务库结构，不触碰 secure storage。
- `citizenapp/lib/wallet/`、`citizenapp/lib/signer/`：Dart 代码与残留清理；统一账户
  恢复、选择、签名公钥和 SS58 展示，不改变派生与私钥保护。
- `citizenapp/lib/citizen/`、`citizenapp/lib/transaction/`、
  `citizenapp/lib/votingengine/`：Dart 代码与残留清理；同步管理员四字段、机构岗位、
  公民身份、投票和交易的账户语义，不改变 Runtime 业务逻辑。
- `citizenapp/lib/qr/`、`citizenapp/lib/8964/`：Dart 代码、生成物与残留清理；消费
  第 5 步固定的 QR 字段和 OnChina/Cloudflare 会话字段，不修改 QR 动作码或压缩键。
- `citizenapp/test/`：测试与 fixture；更新模型、Isar 重建、账户恢复、授权和页面测试，
  删除旧字段/旧 schema 断言。
- `memory/01-architecture/citizenapp/`、`memory/05-modules/citizenapp/`：文档与残留
  清理；记录最终模型、Isar 重建边界和真实验收。
- `memory/08-tasks/20260722-account-id-official-unify.md`：文档；回写第 6 步执行结果
  和第 7 步完整技术方案。

本步预计不修改 `citizenchain/runtime/`、Runtime 版本、PostgreSQL、CitizenWallet
安全存储、Cloudflare D1/R2 或任何 Secret。若审计发现 CitizenApp 当前 SCALE 与
Runtime 真实布局仍不一致，必须先停止并列出完整 Runtime 路径取得二次确认。

完成记录（2026-07-23）：

- CitizenApp 的 Dart、JSON、protobuf、QR、Isar 和页面模型已统一：通用账户使用
  `accountId` / `account_id`，多账户使用明确角色账户，签名公钥使用
  `signerPublicKey` / `signer_public_key`，SS58 只作为展示和扫码地址。旧
  `walletAccount`、`adminAccount`、`ownerAccount`、裸 `pubkey` 等同义字段及其
  getter、alias、fallback、注释和测试断言已清除。
- 钱包、个人多签、机构任职、公民身份、投票、交易、广场、用户资料、通讯录、聊天和
  Cloudflare 会话的直接消费者已同步。通讯录密文明确区分当前
  `account_id` 与联系人 `contact_account_id`，没有把两个账户压成同一 JSON key。
- 冷钱包导入边界只接受 SS58 输入，再规范解析为账户 ID；不再把裸 AccountId 或公钥
  当作展示地址兼容输入。助记词派生、sr25519 签名、SCALE、Runtime call、CID
  材料、QR 动作码和压缩键均未改变。
- Isar 已改为只打开最终 schema；删除开发期 `WalletIsarMigration`、
  `wallet.data.schema.version`、`wallet_sort_order_initialized` 及旧 collection
  清理逻辑。业务库为空时只创建唯一 `WalletSettingsEntity(id=0)`，不读取旧字段。
- QR registry 中广场动作的账户字段已从旧所有者命名统一为 `account_id`，CitizenApp
  与 CitizenWallet 生成物均由 registry 重新导出；registry 一致性 6 项测试通过，
  动作码、压缩键和签名协议未改变。
- 已在 Android 模拟器精确删除并重建
  `/data/user/0/org.citizenapp/files/citizenapp.isar` 及锁文件。删除前业务库没有钱包
  档案，因此不存在可执行的既有账户恢复样本；新版应用真实启动并重新建立 Isar，
  稳定进入“创建钱包”页，无 Flutter、Isar 或 MDBX 异常。
- 三份 Android secure storage 文件在业务库重建前后的 SHA-256 完全一致；没有读取、
  删除或改写助记词、seed、私钥、生物识别材料和 Secret。模拟器未配置系统锁屏，
  应用按安全规则禁止创建测试钱包，未为验收降低安全门槛。
- build_runner 已按最终模型重生 Isar schema；`flutter analyze --no-pub` 无问题。
  CitizenApp 全量测试最终 941 项通过、零失败；账户恢复、签名、授权、二维码、页面和
  Isar 行为由测试覆盖。CitizenApp 代码与当前文档范围的旧字段、旧 migration、
  旧 schema 注释扫描为零。
- 本步没有修改 `citizenchain/runtime/`、Runtime 版本、PostgreSQL、Cloudflare
  D1/R2、用户安全存储或任何 Secret，也没有新增兼容层。

### 第 7 步：CitizenWallet 统一

状态：已完成。

完整技术方案：

1. 先审计 CitizenWallet 当前唯一 Isar 钱包档案、secure storage key、助记词派生、
   登录签名、离线交易签名、QR 请求/响应、payload 解码、账户展示和测试夹具，逐项
   区分 `AccountId`、签名公钥、SS58 展示地址与确有业务语义的普通地址字段。
2. `WalletProfile`、`WalletProfileEntity` 及其内部派生结果统一保存
   `accountId`、`ss58Address`、`ss58Prefix`；账户文本固定为严格小写
   `0x` + 64 位 hex。删除当前 `address`、`pubkeyHex`、`wallet.*` 主键语义、
   `toLowerCase()` 宽松比较和旧 getter，不保留双读或 migration。
3. CitizenWallet 中“公钥”只用于真实验签或签名响应：字段统一为
   `publicKey` / `signerPublicKey`，文本固定为 `0x` + 64 位小写 hex。钱包身份、
   重复账户检查、目标账户比较和 Isar 索引统一使用 `accountId`，不得继续用公钥字段
   名冒充账户。
4. 登录扫码继续严格执行第 5 步协议：扫描定向 `QR_V1/k=1,a=1` 后，在调用私钥前后
   都验证请求 `b.u` 对应当前钱包 `accountId`；签名响应携带规范
   `signer_public_key`。用户二维码只输出规范 `ss58_address`，不得恢复空目标登录、
   浏览器指定账户或旧字段兼容。
5. 离线交易签名先把请求中的目标账户、公钥和 SS58 输入转换到明确模型，再核验当前
   `accountId`；payload decoder 的 Runtime 字段保持第 2、5 步已确定的
   `account_id` / `<role>_account_id` / `signer_public_key`，不得改变 SCALE 字节、
   字段顺序、签名 domain、动作码、压缩键或业务权限。
6. 删除并重建 CitizenWallet 的 Isar 业务数据库和生成 schema，不写 migration。
   `flutter_secure_storage`、Android Keystore、iOS Keychain 中的助记词密文、PIN
   派生材料和私钥保护材料必须原样保留；以同一安全材料按原算法重新派生相同
   `AccountId` 和 SS58。执行前先只读核对真实数据库路径与安全存储边界，删除目标
   不明确时立即停止沟通。
7. 由 build_runner 重新生成 Isar 文件，由 QR registry 重新导出 CitizenWallet
   动作表；更新现有钱包、登录、离线签名、payload decoder、安全存储和 UI 测试，
   删除旧字段 fixture，不新增兼容 fixture 或历史 migration。
8. 更新 CitizenWallet 现有技术文档和本任务卡，补充账户/公钥/SS58、Isar 重建、
   安全存储保护和真实验收边界；扫描清理代码、生成物、注释、文案、测试和文档中的
   旧同义字段及宽松解析残留。
9. 静态验收执行 Dart format、build_runner、`flutter analyze`、定向测试与
   CitizenWallet 全量 `flutter test`。真实运行态在本机 CitizenWallet 上保留安全
   存储、重建业务 Isar，验证应用解锁、同一账户恢复、SS58 展示、用户码、定向登录
   扫码、离线签名请求审阅和签名响应；不得读取、记录或输出助记词和私钥。

预计修改目录：

- `citizenwallet/lib/isar/`：代码、生成物与残留清理；把钱包档案改为最终
  `accountId + ss58Address` schema，删除旧字段和 migration，不触碰安全存储。
- `citizenwallet/lib/wallet/`：代码与残留清理；统一派生结果、账户去重、账户恢复和
  secure storage 映射，不改变助记词、seed、私钥或 sr25519 派生算法。
- `citizenwallet/lib/login/`：代码与测试消费边界；统一登录目标账户与签名公钥命名，
  保持 `b.u` 定向核验和既定签名消息不变。
- `citizenwallet/lib/qr/`：代码、registry 生成物与残留清理；统一账户、公钥、SS58
  字段，不改变 QR 动作码、压缩键、hash-only 规则和线上协议字节。
- `citizenwallet/lib/signer/`：代码与残留清理；统一离线签名账户校验、payload 展示
  字段和签名公钥语义，不改变 Runtime SCALE 或业务权限。
- `citizenwallet/lib/ui/`：代码、注释与文案；统一钱包列表、详情、创建/导入、登录和
  离线签名页面的 AccountId 与 SS58 展示边界。
- `citizenwallet/lib/security/`：只在账户字段直接参与解锁上下文时修改代码和测试；
  不改变 PIN、PBKDF2、Keychain/Keystore 或敏感页面保护规则。
- `citizenwallet/test/`：测试与 fixture；覆盖最终 Isar、账户恢复、定向登录、离线
  签名、QR 和严格格式拒绝，删除旧字段断言。
- `memory/05-modules/citizenwallet/`：文档与残留清理；记录最终 CitizenWallet
  账户模型、安全边界、数据重建和真实验收。
- `memory/08-tasks/20260722-account-id-official-unify.md`：文档；回写第 7 步执行
  结果和第 8 步完整技术方案。

本步预计不修改 `citizenchain/runtime/`、Runtime 版本、CitizenApp、OnChina、
PostgreSQL、Cloudflare D1/R2、助记词派生算法、签名算法或任何 Secret，也不新增文件
或目录。若审计发现必须改变 QR 线上协议字节、Runtime SCALE 或安全存储密钥布局，
立即停止并另行沟通；任何 Runtime 修改仍须完整路径二次确认。

完成记录（2026-07-23）：

- `WalletProfile` 与 `WalletProfileEntity` 已从重复的 `address + pubkeyHex` 改为
  `accountId + ss58Address + ss58Prefix`。AccountId 是钱包身份、Isar 唯一索引、
  重复导入检查和签名目标核验真源；SS58 只用于页面和二维码展示。
- 助记词创建/导入继续按原 mini-secret 与 sr25519 规则派生。派生出的 32 字节账户
  固定编码为小写 `0x` 加 64 位 hex；签名前重新派生并与存储 `accountId` 完全相等，
  已删除 `toLowerCase()`、补前缀和裸 hex 兼容比较。
- `WalletSignResult` 已明确区分 `accountId`、`signerPublicKey` 和签名。登录请求在解析
  后及调用私钥前两次校验当前钱包 `accountId` 等于 `b.u` 解出的规范签名公钥；
  离线交易签名执行相同严格校验，不改变登录签名原文、SCALE、签名 domain、QR 动作码
  或压缩键。
- QR 文本构造入口拒绝无 `0x`、大写、混合大小写、错误长度和非十六进制签名公钥；
  签名响应同时严格校验 32 字节公钥与 64 字节签名。payload decoder 不再接受
  `0X` 或无前缀载荷。
- CitizenWallet 页面已改为明确展示“账户 ID”和“SS58 地址”；用户二维码继续只输出
  `ss58_address`。Isar 生成物已由 build_runner 按最终模型重新生成，不包含
  `address` / `pubkeyHex` 字段、索引或查询方法。
- CitizenWallet 只打开最终 Isar schema，入口 `_openFinalSchema` 明确不执行
  migration。现有默认分组与唯一 settings 行属于空库初始化，不是旧 schema 兼容。
- Android 模拟器执行前未安装 `org.citizenwallet`，因此不存在旧钱包档案或用户安全
  材料。当前源码 Debug APK 已真实构建、安装并启动，应用稳定进入空钱包首页。
- 已精确删除并重建
  `/data/user/0/org.citizenwallet/files/citizenwallet.isar` 与
  `citizenwallet.isar.lock`。安全存储文件
  `FlutterSecureKeyStorage.xml` 重建前后 SHA-256 均为
  `7e07edfd06c65bc1896f7ff1a6acd585e972a3a0fb650f48d349a3789ac817d8`；
  未创建测试钱包，也未读取、删除或输出助记词、seed、私钥和 Secret。
- build_runner 成功，`flutter analyze --no-pub` 无问题，CitizenWallet 全量
  `flutter test` 203 项通过、零失败；账户模型、定向登录、离线签名、QR 严格格式和
  payload decoder 均有测试覆盖。真实运行日志无 Flutter、Isar 或 MDBX 异常。
- CitizenWallet 代码、生成物和测试范围内旧 `pubkeyHex`、旧账户 `address`、
  `_openAndMigrate`、账户同义字段及宽松解析扫描为零。现有文档已同步 AccountId、
  signer public key、SS58、Isar 重建和安全存储边界。
- 本步没有修改 `citizenchain/runtime/`、Runtime 版本、CitizenApp、OnChina、
  PostgreSQL、Cloudflare、助记词派生算法、签名算法或任何 Secret，也没有新增文件、
  目录或兼容层。

### 第 8 步：Cloudflare、D1/R2 与边缘协议重建

状态：已完成。

完整技术方案：

1. 使用 Cloudflare 平台规则先审计 `citizenapp/cloudflare/` 的 Worker HTTP/JSON、
   D1 基线、KV、Durable Object、Queue、R2 对象键、Stream/Images 引用和
   CitizenApp 直接消费者。逐字段区分通用 `account_id`、角色账户、
   `signer_public_key`、P-256 设备公钥及 `ss58_address`，不把媒体 provider 地址、
   EVM 充值地址或现实地址误改成链账户。
2. TypeScript 内通用账户统一为 `accountId`，多账户按角色使用
   `<role>AccountId`；HTTP/JSON、D1、KV/DO/Queue payload 统一为
   `account_id` / `<role>_account_id`。删除 `owner_account`、`creator_account`、
   `subscriber_account`、`followed_account` 等缺少 `_id` 的旧字段，不保留 alias、
   双读、双写或 fallback。
3. 所有 CitizenChain AccountId 文本入口执行 `^0x[0-9a-f]{64}$`。身份验证、
   session、challenge、nonce、会员、创作者、帖子、关注、通知、通讯录、聊天、上传、
   用量和充值归属都以 AccountId 为主键；SS58 只允许在明确展示或扫码边界转换，
   不得进入 D1 主键、KV/DO key 或 R2 路径。
4. 公钥语义统一：sr25519 使用 `signer_public_key`，P-256 设备子钥使用
   `p256_public_key`。删除 `p256_pubkey`、`ownerPubkeyHex`、宽松 trim/lowercase/
   replace 解析和把 SS58 当账户的 `decodeAddress` 路径；验签前先严格校验文本，再
   得到实际 signer AccountId 与 session 目标 AccountId 比较。
5. 重写 `migrations/0001_square_core.sql` 为唯一最终 D1 基线：账户列全部改名并增加
   规范 AccountId `CHECK`；主键、复合键、索引、外键和查询投影同步。把仍有效的
   contacts、topup 结构合并进 0001 后删除历史 `0003`、`0005` 文件，不编写字段迁移。
6. R2/Images/Stream 的对象所有者与 manifest 字段统一为角色 AccountId；对象键使用
   明确、稳定的 64 位小写账户 hex 分段，不再调用 `sanitizeOwnerAccount` 接受任意
   字符串。同步冷归档、上传、媒体清单、聊天大文件中转和对象删除路径，保持媒体内容
   hash、生命周期、签名 URL 和数据保留规则不变。
7. KV session 索引、功能开关、nonce、实时聊天 Durable Object state、通知 Queue
   job/cursor 与 Web Push 设备记录同步目标字段；清除旧 key namespace 和旧序列化
   payload，避免 D1 已统一而边缘缓存仍以旧账户命名运行。
8. 同步 CitizenApp 的 Cloudflare HTTP/JSON 直接消费者及现有 fixture；不修改
   CitizenApp Isar、助记词、安全存储或签名算法。QR registry 只有在发现边缘动作仍
   直接消费旧字段时重新生成，不改变动作码和压缩键。
9. 数据不做 migration：先只读盘点本地 `.wrangler/state` 与当前绑定的 staging、
   production D1/R2/KV/DO/Queue 实际资源、对象/行数量和 Secret 边界。删除本地业务
   状态并按最终 0001 重建；远程 staging/production 的不可恢复删除必须在列出精确
   资源、数量和影响后再次取得明确确认，未确认不得执行。
10. Secret、Cloudflare API token、R2 访问密钥、Stream webhook secret、APNs 密钥、
    CHAIN secret、Turnstile secret 和部署凭据严禁读取、输出、删除或写入仓库；
    `wrangler.toml` 只同步非密钥 binding/schema 配置。
11. 更新 Cloudflare、广场、聊天、会员、媒体、充值、QR 和账户架构文档以及任务卡，
    清理 TypeScript、SQL、JSON、测试、注释、错误文案、对象键和文档中的旧字段、
    SS58 主键、历史 migration 与兼容逻辑。
12. 验收依次执行 format、TypeScript typecheck、Vitest 全量测试、Wrangler dry-run
    与本地 D1 最终 schema 审计；随后启动本地 Worker + D1 + KV + DO + R2 模拟环境，
    通过真实 HTTP 验证挑战验签、session、资料、帖子、关注、会员、通讯录、聊天、
    上传和媒体对象键。旧字段请求必须被拒绝，旧 D1 列/索引和旧对象键必须为零。

预计修改目录：

- `citizenapp/cloudflare/src/auth/`、`src/security/`、`src/shared/`：代码与测试消费
  边界；统一挑战、验签、session、nonce、设备子钥和严格 AccountId 工具。
- `citizenapp/cloudflare/src/account/`、`src/profiles/`、`src/posts/`、
  `src/feeds/`、`src/social/`：代码与残留清理；统一广场账户、资料、帖子、关注、
  通知和注销流程的角色 AccountId。
- `citizenapp/cloudflare/src/membership/`、`src/topup/`、`src/chain/`：代码与残留
  清理；统一链镜像、会员、创作者订阅、充值归属和交易确认账户，不改变链上真源。
- `citizenapp/cloudflare/src/uploads/`、`src/media/`、`src/storage/`、
  `src/limits/`：代码与残留清理；统一上传、配额、R2/Images/Stream 对象所有者、
  manifest 和对象键。
- `citizenapp/cloudflare/src/chat/`、`src/contacts/`：代码与残留清理；统一聊天
  Durable Object、R2 中转、设备路由和端到端密文联系人隔离账户，不解密业务内容。
- `citizenapp/cloudflare/migrations/`：SQL 基线与历史文件删除；生成唯一最终 0001，
  不保留旧字段 migration。
- `citizenapp/cloudflare/test/`：测试与 fixture；覆盖严格 AccountId、最终 D1 schema、
  KV/DO/Queue payload、R2 key、验签及旧字段拒绝。
- `citizenapp/cloudflare/wrangler.toml`、`package.json`：配置与命令；仅同步非密钥
  binding、最终基线和本地验收入口，不部署、不写 Secret。
- `citizenapp/lib/8964/`、`citizenapp/lib/chat/`、`citizenapp/lib/my/`：仅同步
  Cloudflare HTTP/JSON 的直接消费者和错误模型，不改 Isar 或钱包安全材料。
- `memory/01-architecture/qr/`、`memory/01-architecture/citizenapp/`、
  `memory/05-modules/citizenapp/`：文档与残留清理；记录边缘账户、D1、R2、KV/DO、
  Queue 和数据重建边界。
- `memory/08-tasks/20260722-account-id-official-unify.md`：文档；回写第 8 步执行
  结果和第 9 步完整技术方案。

本步预计不修改 `citizenchain/runtime/`、Runtime 版本、CitizenWallet、OnChina、
PostgreSQL、助记词派生、签名算法或任何 Secret，也不新增文件或目录。Cloudflare
远程数据删除和部署不由普通代码确认自动授权；必须先完成只读资源盘点并再次确认精确
远程目标。若发现需要改变 QR 动作码、签名字节或链上 SCALE，立即停止沟通。

本地执行记录（2026-07-23）：

- Worker、D1、KV、Durable Object、Queue、R2 和 CitizenApp 直接消费者已统一使用
  `account_id` / `<role>_account_id`；TypeScript 使用 `accountId` /
  `<role>AccountId`。`owner_account`、缺少 `_id` 的角色账户字段、旧 session key、
  `x-chat-owner` 和任意字符串对象键清洗均已删除，不保留 alias、双读或 fallback。
- 收尾复核发现第 6 步遗留的创作者订阅 RPC 仍把实际 AccountId 参数称作
  `subscriberAddress` / `creatorAddress`，且自订阅判断错误地比较 SS58 与 AccountId。
  已在 `citizenapp/lib/rpc/subscription_rpc.dart` 及其直接消费者统一为
  `subscriberAccountId` / `creatorAccountId`，storage key 入口改为严格解析规范
  AccountId，并把自订阅判断改为 `wallet.accountId` 精确比较；SCALE 字节未改变。
- AccountId 文本入口固定执行 `^0x[0-9a-f]{64}$`，拒绝 SS58、无 `0x`、大写、
  混合大小写、错误长度和非十六进制。链 storage key 直接使用严格 AccountId 32
  字节，不再通过 `decodeAddress` 接受 SS58。P-256 设备公钥固定为无前缀小写
  `04 + 128 hex`，签名固定为无前缀小写 128 hex，不执行 trim/lowercase/replace。
- `migrations/0001_square_core.sql` 已成为唯一最终基线；账户列均有规范格式
  `CHECK`，contacts 与 topup 已合并，开发期 `0003`、`0005` 已删除。旧 Stripe 表、
  旧列和字段迁移不保留。
- R2 账户路径固定使用规范 AccountId 去掉 `0x` 后的 64 位小写 hex：
  `profile/{account_id_hex}/...`、`square/{account_id_hex}/posts/...`、
  `archive/{account_id_hex}/...`。KV session 索引固定为
  `square_sessions_by_account_id:{account_id}`；聊天内部头固定为
  `x-chat-account-id`。
- `wrangler` 已更新到 `4.114.0`，
  `@cloudflare/workers-types` 已更新到 `5.20260723.1`，
  `compatibility_date` 已同步为 `2026-07-23`；staging/production 明确列出全部
  非密钥 topup vars，dry-run 不再依赖环境继承。Secret 未读取、未输出、未删除。
- 本地 `.wrangler/state/v3` 旧业务状态已移至系统临时目录作可恢复隔离，再按最终
  0001 重建为空库。最终本地 D1 有 25 个业务表，业务样本行 0；`square_posts`
  中旧 `owner_account` 列为 0、最终 `account_id` 列为 1，旧 Stripe 表为 0。
  实际 SQLite 写入证明 SS58 被 `CHECK` 拒绝，规范 AccountId 可写入；验收样本随后
  已清空。
- 已启动真实本地 Worker + D1 + KV + Durable Object/R2 模拟环境：
  `/health` 返回 200；规范 AccountId 挑战返回 200；旧字段、SS58 和大写 AccountId
  均返回 400。使用真实 P-256 密钥、真实设备证明、D1 子钥记录和 KV session 后，
  profile、membership、contacts 三条受保护 HTTP 路径均返回 200；验收状态随后删除
  并重新生成空基线。
- `npm run typecheck` 通过；Cloudflare Vitest 29 个文件、173 项全部通过；
  根/staging/production 三套 `wrangler deploy --dry-run` 均成功。CitizenApp
  `flutter analyze --no-pub` 无问题；AccountId 严格解析、设备绑定黄金向量、账户
  动作、订阅、topup、广场和 chat 受影响测试全部通过。
- 当前步骤范围内代码、SQL、JSON、注释和现行架构文档的旧边缘字段扫描为零；历史
  任务卡仅作为历史执行记录保留，不构成协议或兼容实现。本步没有修改 Runtime、
  OnChina、PostgreSQL、CitizenWallet、助记词派生、签名算法或任何 Secret，也没有
  新增文件、目录、提交、推送或远端写操作。

远端执行前只读盘点（2026-07-23）：

- staging D1 `citizenapp-square-db-staging`
  （`4ba85b05-657a-46ac-ab19-8bbd84fe850a`）仍是旧 schema，共 58 行业务数据：
  32 条登录挑战、1 条 Stripe payment、25 条 Stripe webhook，其余已存在业务表为
  0 行。
- production D1 `citizenapp-square-db-production`
  （`0c5a0924-83ef-4347-bacc-b3f6f36da460`）仍是旧 schema，共 614 行业务数据：
  4 条设备绑定 nonce、4 台聊天设备、2 条通讯录密文、6 条设备子钥、257 条登录挑战、
  341 条请求 nonce，其余已存在业务表为 0 行。
- staging KV `91133becebc24f27bf10a00cb001f27e` 有 1 个 session key；
  production KV `b72bbbcb36d240acb317fdaf79ce46f4` 有 4 个 session key 和
  1 个旧 `square_sessions_by_owner` 索引。会话 token 值未写入文档。
- staging 的 `citizenapp-square-media-staging`、`citizenapp-chat-relay-staging`
  均为 0 对象；production 的 `citizenapp-square-media` 有 1 个对象、506B，
  `citizenapp-chat-relay` 为 0 对象。
- 两个 `CHAT_RELAY` 桶当前都只有“7 天后终止未完成 multipart upload”的默认规则，
  没有代码契约要求的全部对象 1 天过期规则；远端执行时必须分别增加 1 天 expiration
  lifecycle，避免未领取密文长期保留。
- staging/production 的 `ChatRealtimeObject` 绑定存在；当前实现使用
  Hibernatable WebSocket 且不写 Durable Object Storage，Wrangler 不提供对象实例
  枚举。部署会断开当时仍活跃的实时连接。
- `square-notify-fanout-staging` 与 `square-notify-fanout-production` 均不存在；
  staging/production Worker 历史部署存在。以上均为执行前状态，不代表本步完成后的
  当前远端状态。
- 仅核对了远端 Secret 名称，未读取任何值：两套环境的 Worker 既有核心 Secret
  名称仍在；production 有 `TOPUP_SETTLE_TOKEN`，staging 没有，因此 staging topup
  结算入口会继续按既定规则 fail-closed。历史 Stripe Secret 名称仍存在，但当前代码
  不消费；按本步已确认边界不读取、不输出值、不删除 Secret。

远端执行记录（2026-07-23）：

- 用户以精确确认语句授权删除并重建上述 staging/production Cloudflare 业务数据、
  创建两个通知队列、为两个 Chat Relay 桶增加全部对象 1 天过期规则，并部署两个
  Worker。执行严格遵循 staging 验收通过后才进入 production；没有读取、输出或删除
  任何 Secret 值。
- staging D1 已删除执行前 58 行旧业务数据和全部旧表，并以唯一
  `migrations/0001_square_core.sql` 重建为 25 张最终业务表；旧 Stripe 表和旧账户列
  为 0，重建完成时业务行为 0。部署后的定时任务按目标实现写入 1 条当前
  `chain_clock` 运行镜像，其余业务表仍为 0 行。staging KV 的 1 个旧 session key
  已删除；两个 staging R2 桶原本均为空，无需删除对象。
- 已创建 `square-notify-fanout-staging`，并为
  `citizenapp-chat-relay-staging` 增加全前缀
  `delete-relay-ciphertext-after-1-day` 规则。staging Worker
  `citizenapp-square-api-staging` 已部署为版本
  `757abe73-a54f-4353-a002-b14aca9a88ab`，队列生产者和消费者均绑定为该 Worker。
  `https://www.crcfrcn.com/api-staging/*` 继续由既有 Cloudflare Access 保护，匿名
  `/health` 返回预期 302 登录重定向，没有绕过访问策略。
- production D1 已删除执行前 614 行旧业务数据和全部旧表，并以同一唯一 0001 基线
  重建为 25 张最终业务表；旧 Stripe 表和旧账户列为 0。production KV 的 4 个
  session key 和 1 个旧 `square_sessions_by_owner` 索引已删除。
- production `citizenapp-square-media` 中唯一 506B 旧对象已精确定位为旧 SS58
  账户目录下的 `profile.json` 并删除；桶对象页真实显示“存储桶已准备就绪”，没有
  保留旧目录或对象。`citizenapp-chat-relay` 原本为空；已增加与 staging 相同的
  全前缀 1 天 expiration lifecycle。
- 已创建 `square-notify-fanout-production`。production Worker
  `citizenapp-square-api` 已部署为版本
  `141b52bf-9387-4d1c-8d5a-5afac3b3b861`，`www.crcfrcn.com/api/*`、两个既有 cron、
  D1、KV、两个 R2 桶、Durable Object、Stream 及队列生产者/消费者绑定均成功。
- production 真实 `GET /api/health` 返回 200。真实登录挑战请求证明旧
  `wallet_account` 返回 400 `invalid_account_id`，规范小写
  `account_id=0x + 64 hex` 返回 200。验收产生的挑战和限流行随后已精确删除；最终
  production D1 只保留定时任务按目标实现写入的 1 条当前 `chain_clock` 运行镜像，
  其余 24 张业务表为 0 行；KV 为 0 key，通知队列生产者/消费者各 1。staging 当前
  同样只有 1 条 `chain_clock` 运行镜像；这两行是部署后的新运行状态，不是旧数据或
  验收残留。
- 本步没有修改 Runtime、OnChina、PostgreSQL、CitizenWallet、助记词派生、签名算法
  或任何 Secret，没有创建仓库文件、提交或推送 GitHub。浏览器只用于读取 R2 对象
  键和确认空桶，实际对象删除使用 Wrangler 精确路径完成。

### 第 9 步：CitizenConsole、脚本、CI、文档和全仓残留清理

状态：已完成（2026-07-24）。

仓库更新复核结论：

1. 当前 `main` 工作树干净，HEAD 为 `8d3d9b75`，相对 `origin/main` 多出
   `0ba5b7f5`、`8d3d9b75` 两个本地提交。本次重大更新涉及创世发行拆分、OnChina
   链投影、公民编辑、CitizenApp/Node/CitizenWallet 消费者及全仓命名，但没有推翻
   本任务第 1—8 步的数据模型和执行结果。
2. 已直接核对当前 Runtime 真源：管理员仍为
   `Admin { account_id, cid_number, family_name, given_name }`，公民主体仍为
   `CitizenSubject { cid_number, account_id }`，机构岗位任职仍使用
   `InstitutionAdminAssignment.account_id`。因此第 9 步不再重复修改这些 Runtime
   结构，也不重做 OnChina 新增的链投影或公民编辑业务。
3. 新增的审计整改第 3 轮已经完成 CitizenConsole Touch ID、Keychain stdin、
   子进程输出脱敏、状态批量缓存和会话比较；第 4 轮已经完成官网、pallet registry
   CI 门禁和 smoldotpow 上游追踪。本任务只复用其结果，不重复实现、不覆盖其安全
   边界。
4. 用户已于 2026-07-24 明确确认 Git 边界：`citizenconsole/` 必须继续整目录由
   Git 忽略，不得纳入 Git 或推送 GitHub。当前 `.gitignore` 的 `/citizenconsole/`
   是目标状态；`memory/AGENTS.md`、agent rules、repo map 和 GMB 架构中要求其
   非密钥源码由 Git 追踪的表述与用户最终决定冲突，必须改为“本机私有、整目录忽略”。
5. 当前真正未收口的第二项是 CitizenConsole 协议升级命名：
   `NRC_ADMIN_PUBKEY`、`pubkeyHex`、`adminPubkey`、会话 `pubkey` 和宽松的
   去前缀/转小写解析仍存在；它们不符合 `signer_public_key` 及严格
   `0x + 64 位小写 hex` 契约。
6. 当前 WASM 路径总体符合已确认规则：只有
   `citizenconsole/actions/citizenchainwasm.sh` 调用版本自增函数，
   `.github/workflows/citizenchain-wasm.yml` 只校验提交源码并原样编译。第 9 步只
   补测试、注释和门禁，不重新设计版本流程。`citizenchain-ci.yml` 当前存在一条重复
   残留注释，需要清理。
7. 当前代码真源已基本使用目标字段，但现行模块文档和统一协议仍保存
   `CitizenSubject.wallet_account`、管理员三字段、`credential_signer_pubkey`、
   `InstitutionAdminAssignment.admin_account` 及已废弃 Chat 版本标签；ADR-040
   还并列保留了一条无 `cid_number` 的旧 `Admin` 结构。这些是本步主要文档残留。

更新后的完整技术方案：

1. 先冻结本步基线并保存精确扫描结果。账户字段只按真实语义分类：
   CitizenChain `AccountId`、签名公钥、SS58 展示地址分别处理；钱包产品、助记词/
   私钥容器、macOS Keychain 命令自身的 `account` 参数、EVM 地址、现实地址和
   `AdminAccount` 个人多签业务类型不做机械替换。
2. 保持 `.gitignore` 对 `/citizenconsole/` 的整目录忽略，不删除、不反选、不强制
   添加其中任何文件。当前 34 个源码/配置/测试文件、`.runtime`、`node_modules`、
   日志、二进制、WASM、本机状态和私密材料全部继续留在本机，不进入 Git 索引、
   commit、GitHub 或任何远端。第 9 步对 CitizenConsole 的修改只在当前本机生效。
3. 收口 CitizenConsole 协议升级链路：JavaScript 内部统一使用
   `signerPublicKey`，HTTP/JSON 和配置语义统一为 `signer_public_key`，
   Keychain target 统一为 `NRC_SIGNER_PUBLIC_KEY`；仅由该公钥派生、供
   Polkadot API 查询 nonce 和 `addSignature` 使用的展示地址命名为
   `ss58Address`。会话、函数参数、返回值、前端状态、中文文案和注释同步修改。
4. 所有公钥入口必须直接满足 `^0x[0-9a-f]{64}$`；删除
   `replace(/^0x/i, '')`、`toLowerCase()` 和回执公钥大小写兼容。内部转成 32 字节
   后再编码 QR 的 `b.u`；扫码回执必须逐字节等于预期签名公钥。禁止接受无 `0x`、
   大写、混合大小写或 SS58 作为替代输入。
5. Runtime 升级 QR 的 `p/k/i/e/b`、`b.a/g/u/d/s`、action code、SCALE call、
   nonce、WASM hash、签名 payload、sr25519 算法和交易组装字节保持不变；本步只改
   内部命名和边界校验。`QR_V1` 继续是唯一扫码协议版本标识，不新增第二协议或兼容
   envelope。
6. 旧 Keychain target 不做双读、迁移或 fallback，也不读取/输出其值。执行时只允许
   检查 `rtupg:NRC_ADMIN_PUBKEY` 是否存在；若存在，先报告精确 target 并停下取得
   删除确认。目标公钥由用户通过控制台按
   `rtupg:NRC_SIGNER_PUBLIC_KEY` 重新配置，用户安全材料和其它 Keychain 项保持
   原样。
7. 保持 WASM 版本职责不变并补足可验证门禁：只有控制台“运行 WASM CI”读取已配置
   目标链 genesis/spec_version，在源码版本与链上版本严格相等时把
   `citizenchain/runtime/src/lib.rs` 和现有版本测试断言精确提高 1；普通 workflow、
   手工编译和协议升级 QR 不提高版本。隔离测试只对临时副本运行自增函数，证明
   `+1`、版本漂移拒绝、上限拒绝和测试断言漂移拒绝，不在本步产生 Runtime diff。
8. 清理 `.github/workflows/citizenchain-ci.yml` 的重复注释，并复核
   `.github/workflows/citizenchain-wasm.yml` 只校验源码版本/目标链指纹后原样编译。
   不执行 `git push`、`gh workflow run`、Release、部署或任何会触发 GitHub
   Actions 的操作。
9. 清理当前架构、模块、ADR、统一命名和统一协议文档：全部按当前代码真源更新为
   `Admin { account_id, cid_number, family_name, given_name }`、
   `CitizenSubject { cid_number, account_id }`、
   `InstitutionAdminAssignment.account_id` 和
   `credential_signer_public_key`。删除已废弃的 `GMB_CHAT_V1` 现行协议标签，
   Chat 只引用真实 `ChatEnvelope`/Protobuf schema，不另造新的版本化字符串。
   旧任务卡和 ADR 只有在明确说明“已拒绝/已删除旧称”时才可引用历史字符串，不得再
   被当前契约当作有效字段。
10. 清理安全可再生残留和错误白名单。QR registry 中用于防止旧字段回归的精确负面
    测试保留，因为它是门禁而不是兼容字段；`activate_admin_account` 等真实业务动作
    名和 `AdminAccount` 个人多签业务类型保留，因为它们不表示链账户字段。不得用宽
    目录排除掩盖当前代码、配置、注释、生成物或文档残留。
11. 静态验收执行 CitizenConsole `npm run check`、`npm test`、全部 Shell
    `shellcheck`、JavaScript 语法检查、`actionlint`、WASM 版本函数隔离测试、
    QR registry 一致性测试和精确全仓残留扫描。验证整个 `citizenconsole/` 继续由
    Git 忽略，目录内不存在被跟踪、暂存或准备提交的文件。
12. 真实运行态验收启动 CitizenConsole 本地服务和真实页面，验证模块清单、密钥
    状态、严格公钥失败路径、WASM CI 目标链预检、协议升级表单和 QR 生成/回扫前置
    门禁。可使用隔离生成的测试公钥验证编码和本地验签，但不得提交 Runtime 升级
    交易、不得实际提高 Runtime 版本、不得读取用户私钥或生产 Secret、不得再次写
    Cloudflare 远端。需要真实 NRC 公钥或运行节点才能继续的动作必须停下沟通。
13. 完成后更新本任务卡、ADR-040、当前架构/模块/协议文档，清理临时服务和测试状态，
    再输出第 10 步“全链真实运行态总验收”的更新后完整技术方案；未经确认不执行
    第 10 步。

预计修改目录：

- `.gitignore`：配置只读核对；保留 `/citizenconsole/` 整目录忽略规则，不增加
  反选或跟踪例外，不改变其它模块忽略规则。
- `citizenconsole/`：本机私有代码、配置和测试；统一协议升级签名公钥/SS58 命名和
  严格校验，复核 Touch ID、Keychain、部署动作与 WASM CI。全部修改只在本机生效，
  整个目录继续不被 Git 跟踪、不提交、不推送 GitHub。
- `.github/workflows/`：CI 配置与注释；清理桌面 CI 重复注释，确认 WASM workflow
  只校验并原样编译源码，不提高版本、不触发发布或部署。本步不运行 workflow。
- `.github/scripts/`、`scripts/`：现有门禁和生成器的只读复核，只有发现当前账户/
  协议负面门禁与真实代码不一致时才修改；不恢复部署脚本，不写 Secret。
- `citizenchain/crates/qr-protocol/`：现有测试与文档消费边界；保留并校验旧字段禁止
  回归的负面测试，不改变 action code、字段 registry、QR 字节或生成物。
- `memory/01-architecture/`、`memory/05-modules/`、`memory/07-ai/`：现行文档、协议
  和命名表残留清理；按当前代码更新管理员、公民主体、任职、公钥、SS58、
  CitizenConsole 本机私有边界、WASM CI 和 Chat schema 表述。
- `memory/AGENTS.md`：仓库规则文档；把 CitizenConsole “非密钥源码必须由 Git
  追踪”的旧规则改为用户最终确认的“本机私有、整目录由 Git 忽略、不得推送
  GitHub”，继续保留 Keychain、Touch ID 和禁止明文 Secret 的安全要求。
- `memory/04-decisions/ADR-020-citizenapp-p2p-chat.md`：文档与残留清理；删除已废弃的
  Chat 版本标签，改为引用真实 `ChatEnvelope`/Protobuf schema，不改变 Chat 代码或
  线上消息。
- `memory/04-decisions/ADR-040-account-id-official-standard.md`：文档；记录全仓统一
  当前状态，删除旧管理员三字段结构，补充 CitizenConsole 本机私有边界和最终验收
  口径。
- `memory/08-tasks/20260722-account-id-official-unify.md`：文档；回写第 9 步执行与
  真实验收结果，并输出第 10 步完整技术方案。

本步预计不修改 `citizenchain/runtime/`、Runtime 版本、D1/R2/KV 远端数据、
PostgreSQL、Isar、OnChina/CitizenApp/CitizenWallet 业务逻辑、助记词派生、签名算法
或任何 Secret，也不新增仓库文件或目录。现有 34 个 CitizenConsole 非密钥文件会
继续与整个 `citizenconsole/` 一起被 Git 忽略，不会暂存、提交或推送。执行前仍需
用户确认更新后的第 9 步；若发现必须产生任何 Runtime diff，或需要删除旧 Keychain
target，必须分别再次列出精确目标并取得对应确认。

执行结果：

- 用户确认后精确删除 Keychain target
  `GMB Deploy / rtupg:NRC_ADMIN_PUBKEY`；没有读取或输出其值。最终复核旧 target
  与新 `GMB Deploy / rtupg:NRC_SIGNER_PUBLIC_KEY` 均不存在，新公钥必须由用户
  今后通过控制台重新配置。
- CitizenConsole 本机代码已把协议升级链路统一为
  `signerPublicKey / signer_public_key / NRC_SIGNER_PUBLIC_KEY`，派生展示地址统一
  为 `ss58Address / ss58_address`。所有入口严格执行
  `^0x[0-9a-f]{64}$`，不接受无前缀、大写、混合大小写、首尾空白或 SS58，也没有
  旧 target 双读、自动迁移、大小写兼容或 fallback。
- Runtime 升级 QR 的 action code、SCALE call、签名 payload、WASM hash、nonce 和
  sr25519 字节没有改变；`QR_V1` 仍是唯一版本化协议标识。隔离临时副本证明控制台
  WASM 按钮只在目标链与源码 `spec_version` 相等时执行 `0 → 1`，源码漂移、测试
  断言漂移和 `u32` 上限均失败关闭；真实 `citizenchain/runtime/` 没有 diff。
- `.github/workflows/citizenchain-ci.yml` 的重复注释已删除；
  `.github/workflows/citizenapp-ci.yml` 两个 `gh api` 路径已加引号，消除
  Actionlint/ShellCheck 的 SC2086。没有运行 workflow、推送、Release 或部署。
- 当前架构、模块、统一命名、统一协议和相关 ADR 已按代码真源更新为四字段
  `Admin`、`CitizenSubject.account_id`、`InstitutionAdminAssignment.account_id`、
  `credential_signer_public_key` 与 `ChatEnvelope` schema。ADR-019 已明确标记为
 被 ADR-039/040 取代，不再把旧聚合管理员账户索引写成待实施方案。
- `citizenconsole/` 继续由 `.gitignore:16` 的 `/citizenconsole/` 整目录规则忽略，
  `git ls-files citizenconsole` 为空；本机修改没有进入 Git 索引、暂存区或 GitHub。
  仓库规则、repo map 和架构文档均已同步这一最终边界。
- 静态验收通过：CitizenConsole `npm run check`、26 项 Node 测试、全部控制台 Shell
  的 ShellCheck、全仓 Workflow Actionlint、严格签名公钥六类拒绝测试、QR registry
  7 项一致性测试及 `git diff --check` 全部通过。
- 真实本地服务和页面验收通过：WASM 模块只显示
  `NRC_SIGNER_PUBLIC_KEY` 且为“未配置”；输入控件固定 66 字符和严格 pattern；
  大写测试公钥被后端明确拒绝；缺少新公钥时协议升级立即失败关闭，没有触发
  Touch ID、链交易或远端 CI。验收期间没有修改 Runtime、Cloudflare、PostgreSQL、
  Isar、Secret 或钱包安全材料；现有独立 CitizenApp 本地运行进程仍保持运行。

### 第 10 步：真实运行态总验收

状态：已确认并执行完成。

更新后的完整技术方案：

1. 冻结验收基线。记录当前 HEAD、工作树和运行服务，只把本任务已确认范围作为验收
   对象；`citizenapp/android/app/src/main/jniLibs/arm64-v8a/libsmoldot.so` 等现有
   用户/其它线程改动只保护、不覆盖。先执行账户字段、协议标识、生成物和数据库
   schema 门禁，发现代码漂移时停止，不在总验收中顺手改业务。
2. 使用当前源码构建 release Node 和原样 Runtime WASM，核对 Runtime 六项项目版本
   仍为 `0`、StorageVersion 仍为 `0`、metadata 与 QR registry 可解析。不得点击
   CitizenConsole“运行 WASM CI”，不得提高 `spec_version`，不得修改
   `citizenchain/runtime/`；如构建或真实交易暴露 Runtime 缺陷，单独列出完整路径并
   重新取得二次确认后再修复。
3. 在仓库外临时 base path 启动隔离 fresh CitizenChain，不替换当前开发节点数据。
   记录 block #0、state root、genesis hash、metadata 大小和节点健康；验收完成后停止
   隔离节点。创世必须直接使用最终四字段 Admin、最终账户文本边界和当前固定机构/
   岗位规则，不运行 migration。
4. 为验收创建独立本地 PostgreSQL 数据库并启动真实 HTTPS OnChina，绑定隔离 fresh
   链；不删除当前正在使用的业务库。真实检查 49,593 个公权机构、99,232 个
   账户、创世治理机构和公民链技术发展基金会投影，以及
   `account_id / signer_public_key / ss58_address` JSON 边界。完成后删除验收专用
   数据库，不保留测试公民、机构、签名会话或审计残留。
5. 使用隔离测试助记词在 CitizenApp 与 CitizenWallet 真实运行态分别恢复同一账户，
   验证二者得到完全相同的小写 `0x + 64 hex account_id`，SS58 只用于展示，Isar 最终
   schema 无旧列、旧 collection 或兼容读。不得读取、导出或覆盖用户现有助记词和
   Keychain/Keystore；没有可用隔离设备时停下，由用户提供验收设备。
6. 真实验证 OnChina 登录链路：管理员先出示严格用户二维码，OnChina 只读取其中
   `account_id`/可派生 SS58 身份；随后生成绑定该账户的登录挑战，CitizenWallet
   签名，OnChina 验签并建立 `institution_code + workspace` 登录态。继续验证
   `Admin { account_id, cid_number, family_name, given_name }` 链读、过期/错账户/
   SS58 主键/旧字段全部失败关闭。
7. 真实验证机构权限：使用创世测试账户证明同一人可在同一或不同机构担任多个岗位；
   每次业务操作必须同时满足机构 CID、岗位码和签名账户。验证只有 admins、只有岗位、
   错机构、错岗位、旧绑定账户均不能发起或投票，管理员姓名和本地登录态不能形成第二
   授权真源。
8. 真实验证公民身份：由注册局有效岗位管理员与目标公民账户完成现有双签上链，
   `citizen-identity` 保存永久公民 CID 与当前 `account_id` 双向绑定、分开姓名、
   居住地、状态和护照有效期；投票/竞选身份只使用各自最终字段。过期、状态异常、
   CID↔账户错配或身份字段不完整均不能进入人口分母、投票或竞选资格。
9. 真实验证投票引擎边界：业务模块创建提案并静态选择内部、联合、选举或立法引擎，
   引擎只消费 entity 岗位任职和 citizen-identity 人口真源。至少覆盖同人多岗位独立
   票据、机构阈值不随钱包数变化、公民按永久 CID 去重、护照过期不计人口、业务模块
   通过后才执行具体动作。协议升级与决议发行要验证 NRC + 43 PRC 委员可发起/投票、
   43 PRB 董事只投票的联合计划；若完整 87 机构签名夹具当前无法安全获得，只执行
   既有真实创世测试账户可覆盖的阶段并明确列出缺口，不伪造结果。
10. 真实验证扫码与转账：OnChina/Node/CitizenApp 生成请求，CitizenWallet 严格解码、
    核对 `b.u` 与当前签名公钥、签一次并回扫；错账户、旧字段、非法大小写、截断和
    尾随载荷全部拒绝。机构转账必须从机构费用账户扣费并由有效岗位发起；个人多签走
    独立主体。协议升级只验证请求、回执和 dry-run，不提交 `set_code`，不改变版本。
11. Cloudflare 只在 staging 做可清理的真实写验收：验证账户登录、设备子钥、Chat
    瞬时转发、联系人、广场、会员、通知队列和 R2 生命周期均以 `account_id` 为键，
    旧字段请求失败。只删除本次验收创建且可精确识别的 staging 行、KV key、R2 对象
    和队列消息；production 只做健康、schema/binding 和空残留只读核对，不重新删除
    或部署。任何 production 写、队列重建或 Worker 部署都必须另行取得明确确认。
12. 汇总证据并清理：停止隔离 Node/OnChina，清除仓库外临时 base path、验收数据库、
    测试 App 数据和 staging 验收对象；不碰用户现有服务和安全材料。再次运行全仓
    残留门禁、编译/测试和 `git diff --check`，记录真实区块、交易、HTTP、页面和
    Cloudflare 证据。若任何业务失败，先报告真实原因并输出针对该缺陷的新方案，
    不把“部分通过”写成任务完成。

完成标准：

- 同一助记词在 CitizenApp/CitizenWallet 恢复同一 `account_id`，全端文本格式唯一。
- 机构 `CID + 岗位码 + account_id` 和公民 `CID + account_id` 权限模型均有真实正反
  例；管理员、业务模块和投票引擎职责没有串位。
- PostgreSQL、Isar、链 storage、QR、HTTP/JSON 与 Cloudflare staging 不存在旧字段、
  旧格式、SS58 主键或兼容入口。
- fresh Node、真实 OnChina、真实页面、真实扫码和至少一条合法链业务交易均通过；
  纯编译、单元测试或 mock 不能代替。
- Runtime 版本、生产 Cloudflare、用户数据库、用户助记词/私钥和 GitHub 远端保持
  未修改；所有验收临时状态完成清理。

预计修改目录：

- `citizenchain/runtime/`：只读构建、metadata/版本/storage 校验和真实交易消费；
  预计无源码 diff。任何改动都必须停止并重新取得完整路径二次确认。
- `citizenchain/node/`：release 构建和仓库外隔离 fresh 节点运行；预计不改代码，
  不覆盖当前节点 base path。
- `citizenchain/onchina/`：真实 HTTPS 服务、隔离 PostgreSQL、链投影、登录、权限和
  公民双签验收；预计不改代码，验收库完成后删除。
- `citizenapp/`：真实 App/Isar、账户恢复、页面、扫码和 Cloudflare staging 消费
  验收；预计不改代码，不覆盖用户现有 App 数据。
- `citizenwallet/`：真实冷签、账户恢复、请求严格解码和回执验收；预计不改代码，
  不读取或迁移用户现有安全材料。
- `citizenapp/cloudflare/`：现有 staging Worker/D1/KV/R2/Queues 的真实读写和精确
  清理；预计不改代码、不部署，production 只读。
- `citizenweb/`：会员/账户边界页面只读联调；预计不改代码、不部署。
- `citizenconsole/`：本机私有控制台只用于查看状态和协议升级 dry-run，继续整目录
  Git 忽略；不得运行 WASM CI、提交升级或推送 GitHub。
- `memory/04-decisions/ADR-040-account-id-official-standard.md`、
  `memory/05-modules/`、`memory/08-tasks/20260722-account-id-official-unify.md`：
  仅在全部验收完成后回写证据、最终状态和残留清理结果。

本步预计不新增仓库文件或目录、不修改 Runtime、不删除当前本地业务库、不写
production Cloudflare、不提交协议升级、不触发 GitHub Actions、不提交或推送。
执行前需要用户确认第 10 步；若用户要求改为重建当前本地数据库、写 production 或
真实提交 `set_code`，必须把精确目标和影响另行列出并再次确认。

第 10 步执行结果（2026-07-24）：

- 当前源码 release Node 与原样 Runtime WASM 构建成功。release Node SHA-256 为
  `0e9f62f718dfca36d50d990e1da223f226fa2340449925692d6188427d9b81c7`，
  compact WASM SHA-256 为
  `4aaa078fbd5919364d30821d4dd2f0f82b60358837a81849bf64a989b9aac12c`，
  compressed compact WASM SHA-256 为
  `ea028f8de66474870329ebc3cf4a5429ccfe3b03b095f8f4096a6bf14cefbdbd`。
  Runtime 六项版本与全部 StorageVersion 仍为 `0`，没有运行 CitizenConsole WASM
  CI，也没有产生 `citizenchain/runtime/` 源码 diff。
- 仓库外 fresh 链真实启动成功：genesis hash
  `0xafac9d55a77a10780b5c5cb29da6118ecf4a7b9652960e52502ebacf5d403535`，
  state root
  `0x48d835d720885fc999133cb62c56f8c6b7981f3a7ecbd0d70f0f87cf7bf81a5b`，
  metadata 为 `223,128` 字节。没有运行 migration。
- 独立 PostgreSQL 与真实 OnChina 页面完成 fresh 链投影：49,593 个机构、99,232 个
  账户；99,232 = 49,593 个主账户 + 49,593 个费用账户 + 43 个省储行质押账户 +
  两和基金、安全基金、联邦公民安全基金。规范账户格式错误、重复账户、缺主体、
  `wallet_account/admin_account/owner_account/citizen_full_name/legal_rep_*` 旧列
  均为 0。验收数据库与服务在收口阶段精确删除和停止，不触碰用户原有业务数据库。
- Pixel 8a（Android 16/API 36）真实清空并安装当前 CitizenApp/CitizenWallet，
  两端使用仓库公开测试助记词恢复得到相同
  `account_id=0x625e25364c7b68e0a83065ccb40afed43f8fe933e669b24f3d69a57eddb3b715`
  和相同展示 SS58
  `w5Don5m6XHNfHH7Xkzwf9LZATjCh8vD1utuFJy7f92F8nUCK2`。CitizenApp 的真实
  Turnstile 通过；两端 Isar 最终钱包档案均为 `accountId + ss58Address`，未保存
  助记词。验收结束清除两包应用数据。
- OnChina 登录严格边界真实通过：完整 `QR_V1/k=3` 用户码进入链上管理员门禁并以
  `ONCHINA_LOGIN_ADMIN_NOT_ONCHAIN` 拒绝非管理员测试账户；裸 SS58、旧
  `wallet_account`、空 `contact_name` 均返回 400。登录挑战、结果和会话表保持 0，
  证明失败路径没有遗留状态。当前 fresh 创世管理员源码只包含公开
  `account_id=0x9c3e18f575c59236832054469ef0e69f16a1fe6c50b2b580fc7c71853ab71068`，
  仓库没有对应私钥或助记词；因此没有读取外部 Secret、没有伪造成功管理员登录，
  也没有伪造一条线上管理员业务交易。
- Rust 全 workspace 所有 target 测试通过。真实 runtime externalities 正例覆盖：
  公民永久 CID 与当前账户双向绑定、过期护照不计人口、同一账户按多个岗位取得独立
  票据、旧绑定账户失权、NRC/PRC/PRB 联合投票、机构阈值转账、决议发行与协议升级
  回调执行。它证明链内业务和权限执行路径，不冒充缺少创世管理员私钥时无法完成的
  live RPC 签名交易。
- CitizenWallet 账户/QR/payload 专项 124 项通过；全量 191 项通过。全量首轮发现
  3 个金额默认单位断言仍写旧 `GMB`，实现与注释已统一为“元”，本步同步修正现有
  测试断言后全绿。CitizenApp 全量 792 项通过、5 项按测试声明跳过；Cloudflare
  typecheck 与 174 项测试均通过。
- Cloudflare production 仅执行健康、D1 schema、binding 和生命周期只读检查。
  staging 真实验证 D1 严格小写账户 CHECK、KV 与 R2 账户键写入/读取/精确删除；
  大写账户被约束拒绝，所有测试行、KV key 与 R2 object 均清零。staging/production
  Chat Relay R2 均保持“1 天后删除”的现有规则，未部署 Worker、未写 production、
  未重建队列。
- 两个不属于账户命名逻辑的运行环境事实已记录：CitizenWallet 当前 arm64
  `libisar.so` ELF LOAD 对齐仍为 `0x1000`，Android 16 会显示兼容提示，但 APK
  `zipalign -P 16` 通过；OnChina 隔离启动在当前环境提供 HTTP 页面，临时 CA 未被
  浏览器信任。两项都没有通过放宽账户权限、协议或伪造验收来掩盖。
- 清理后再次确认：隔离 Node、OnChina、PostgreSQL、真机测试数据、staging 测试
  对象和仓库外临时目录均无残留；用户原有 `9944` 节点、OnChina 与 Vite 进程保持
  不变。未修改 Runtime、未写 production、未触发 GitHub Actions、未提交或推送。
- `git diff --check` 通过。只读 `cargo fmt --all --check` 发现工作树中其它既有/
  并发改动含未格式化 Rust，范围同时跨越 OnChina 与 Runtime；本步没有运行会制造
  Runtime diff 的格式化命令，也没有覆盖这些非本任务改动。

## 每步共同完成条件

- 先输出该步完整技术方案和“预计修改目录”，取得确认后才执行。
- 涉及 `citizenchain/runtime/` 时必须单独列出完整路径并取得二次确认。
- 改代码后同步更新文档、完善中文注释、清理旧代码/注释/文档/配置/数据残留。
- 不新增兼容层，不保留旧字段，不推送 GitHub。
