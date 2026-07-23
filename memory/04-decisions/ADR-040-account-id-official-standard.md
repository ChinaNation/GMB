# ADR-040：全仓账户标识采用 Substrate 官方模型

状态：Accepted（2026-07-22；目标契约已冻结，代码按任务卡分步实施）。

## 背景

仓库把同一个由助记词派生、用于签名和授权的链账户写成了 `wallet_account`、`admin_account`、`owner_account`、`wallet_pubkey`、`admin_pubkey`、`wallet_address` 等多组名称。它们混合了链账户、钱包软件、人员角色、公钥和展示地址，导致跨 Rust、Dart、TypeScript、SQL、JSON、QR 和文档的字段不一致，也容易把 SS58 展示字符串误当授权主键。

Polkadot SDK 的运行时身份模型以 `AccountId` 表示账户，以公钥完成签名验证，以 SS58 表示人类可读地址。ADR-022 已固定 `AccountId` 是账户身份锚点、签名算法只是授权方式；本 ADR 在该密码学边界之上统一全仓命名和文本编码。

## 决策

### 1. 账户、公钥和地址严格分层

- runtime 账户类型使用 `AccountId`；当前具体值仍是 32 字节账户标识。
- 单一账户字段统一为 `account_id`。
- 一个结构同时出现多个具有独立业务角色的账户时，统一使用 `<role>_account_id`，例如 `actor_account_id`、`voter_account_id`、`creator_account_id`、`sender_account_id`、`recipient_account_id`、`subscriber_account_id`、`registrar_account_id`、`beneficiary_account_id`、`representative_account_id`。
- 签名公钥统一为 `public_key`；当前签名者公钥为 `signer_public_key`；凭证签名者公钥为 `credential_signer_public_key`。
- SS58 只允许命名为 `ss58_address`，只用于输入、输出和界面展示。授权、关系索引、数据库主键、缓存 key 和链上真源必须使用 `account_id`。
- `wallet` 是链外软件，不得再用于 runtime 账户字段；`admin`、`owner` 是人员或业务角色，不得在只有一个账户的结构中替代账户本体命名。

### 2. 文本编码唯一

- `account_id` 和 32 字节公钥的跨端文本表示固定为小写 `0x` 加 64 位十六进制。
- 唯一校验式为 `^0x[0-9a-f]{64}$`。
- 进入系统边界时必须一次规范化并校验；内部不得同时保存无 `0x`、大写、混合大小写或 SS58 形式的同一账户。
- `ss58_address` 必须从账户字节按指定网络格式派生；不得反向成为账户权限或数据库身份真源。

### 3. 授权与验签

- 签名流程固定为：校验 `signer_public_key` 和签名，按运行时账户识别规则得到 `signer_account_id`，再与业务要求的 `account_id` 或 `<role>_account_id` 比较。
- 机构岗位权限继续由 `cid_number + role_code + account_id` 三者共同成立；管理员账户本身不获得业务权限。
- 公民身份继续由 `cid_number + account_id` 共同校验；citizen-identity 是 CID 与账户一一对应的唯一真源。
- 机构 CID、公民 CID、岗位码、账户 ID、公钥和 SS58 地址分别表达不同语义，禁止互相替代。

### 4. 目标结构与存储

```text
Admin { account_id, family_name, given_name }
Admin { account_id, cid_number, family_name, given_name }
CitizenSubject { cid_number, account_id }
VotingIdentityPayload { cid_number, account_id, ... }
InstitutionAdminAssignment { cid_number, account_id, role_code, ... }
InstitutionVoteTicket { role_subject, voter_account_id }

AccountIdByCid[cid_number] -> AccountId
CidByAccountId[account_id] -> cid_number
```

结构中存在第二个账户时必须根据业务角色命名，不得为避免重名恢复 `wallet_account`、`admin_account`、`owner_account` 等旧称。

### 5. 数据与兼容策略

- 当前尚未正式创世，runtime 和 pallet StorageVersion 保持 `0`，直接采用最终结构，不写 migration。
- PostgreSQL、Isar、Cloudflare D1/R2 旧业务数据全部删除并按最终 schema 重建。
- 不保留旧字段、旧 JSON、旧 SQL 列、双写、双读、fallback 或任何过渡兼容。
- 助记词、seed、私钥、macOS Keychain、iOS Keychain、Android Keystore 和 GitHub Secrets 不属于待删除业务数据，必须保留；同一安全材料按未改变的派生规则得到同一账户。
- CID 生成、签名 payload、SCALE 字段顺序或哈希材料不得因纯命名重构而改变字节。实施中若发现必须改变协议字节，必须停止并单独取得确认。

## 影响

- 这是全仓 breaking rename，影响 runtime、Node、OnChina、CitizenApp、CitizenWallet、Cloudflare、QR registry、SQL、JSON、SCALE、测试、生成物和文档。
- 旧名称只有在任务卡实施完成前用于准确描述当前代码时才可出现；不得新增使用，完成对应步骤时必须连同代码、注释、文档和数据一起删除。
- 角色字段不会被无差别压平。`source`、`dest` 等框架语义，以及确有多个账户时的 `<role>_account_id`，继续保留其业务区分。
- ADR-022 的账户派生、AccountId 锚点和未来 PQC 授权路线不变；ADR-023、ADR-039 中的旧账户字段名由本 ADR 的目标命名取代，管理员名册、岗位权限和投票边界不变。

## 实施状态

- 第 1 步已冻结全仓命名、格式和无兼容原则。
- 第 2 步已完成 runtime 结构、存储、事件、权限入口及其直接 SCALE 消费者统一；正式创世前版本与 StorageVersion 保持 `0`。
- 第 3 步已完成 Node、桌面直接消费者和 `chain-signing` 共享 crate 统一：账户、公钥和 SS58 已分层，跨进程账户与公钥严格使用小写 `0x` 加 64 位十六进制，签名流程先验公钥再比较账户，奖励账户 RPC 与本地非密钥缓存已按最终命名重建且没有兼容入口。
- 第 4 步已完成 OnChina 后端、前端、PostgreSQL 最终 schema、HTTP/JSON、登录与授权上下文统一，并删除重建本地 PostgreSQL 业务数据库。经单独二次确认，已使用当前 Runtime WASM 启动隔离 fresh chain，真实完成 PostgreSQL、HTTPS OnChina、链投影、账户格式、登录验签和管理员链上门禁验收；验收后数据库再次清空重建且全部服务已停止。
- 第 5 步 QR 协议与生成物完整技术方案已写入任务卡，待确认后实施。CitizenApp、CitizenWallet、Cloudflare、控制台和其余全仓文档按任务卡后续步骤继续实施；对应步骤完成前的旧名称只用于定位待删除实现，不构成允许新增旧字段的例外。

## 备选方案

- 保留 `wallet_account`：拒绝。钱包是软件载体，不是 runtime 身份类型。
- 全部统一为 `public_key`：拒绝。账户标识、公钥和 SS58 地址是不同层次，未来签名算法变化后尤其不能混用。
- 全部统一为 `account_id` 而删除角色：拒绝。同一结构有多个账户时会失去发送方、接收方、投票人等业务语义。
- 保留旧字段兼容：拒绝。当前没有正式创世数据，兼容会永久制造双轨。

## 后续动作

- 按 `memory/08-tasks/20260722-account-id-official-unify.md` 分步实施。
- 每一步先输出完整技术方案；涉及 runtime 时按完整路径取得二次确认。
- 每一步完成代码、文档、中文注释和残留清理；最终重新创世并完成真实运行态验收。
