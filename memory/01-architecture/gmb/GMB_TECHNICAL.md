# GMB 仓库技术文档（当前实现基线）

## 1. 文档目的

- 固化 `GMB` 仓库当前的四产品体系、技术边界、跨产品协作关系与维护规则。
- 作为仓库级技术总入口，帮助开发、联调、测试、发布时快速判断问题属于哪个产品、哪个模块、哪一类发布动作。
- 统一仓库文档、产品文档和模块文档职责，避免口径冲突。

## 2. 文档体系

仓库技术文档：

- `memory/01-architecture/gmb/GMB_TECHNICAL.md`

产品技术文档：

- `memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- `citizenwallet` 产品级架构文档尚未单独建立，新增前必须按仓库新增文件规则单独确认。
- `citizenweb` 产品级架构文档尚未单独建立，新增前必须按仓库新增文件规则单独确认。

公民链内部能力文档：

- `memory/01-architecture/onchina/ONCHINA_TECHNICAL.md`

模块技术文档：

- 位于 `memory/05-modules/` 中，按产品和模块目录归档，命名统一为 `*_TECHNICAL.md`。

## 3. 仓库总体定位

`GMB` 是一个多产品单仓库，围绕“公民链 + 公民 + 公民钱包 + 官方网站”构建完整数字主权系统。

仓库当前只保留四个产品：

- `citizenchain`：公民链，负责链上状态、共识、治理、发行、交易、节点运行、桌面节点软件和 OnChina 多机构工作台能力。
- `citizenapp`：公民，负责在线钱包、治理入口、交易入口、轻节点状态和用户端身份展示。
- `citizenwallet`：公民钱包，负责离线签名、扫码识别、冷钱包确认和签名结果生成。
- `citizenweb`：官方网站，负责 GMB 对外官网静态站点。

OnChina 不是独立产品；它是 `citizenchain/onchina/` 下的公民链内部能力。

### 3.1 跨产品账户标识目标

ADR-040 已冻结全仓统一账户模型，实施进度见 `memory/08-tasks/20260722-account-id-official-unify.md`：

- 链账户类型统一为 `AccountId`，单一账户字段统一为 `account_id`；多账户结构使用准确的 `<role>_account_id`。
- 公钥统一为 `public_key` / `signer_public_key` / `credential_signer_public_key`；SS58 展示字段统一为 `ss58_address`。
- `account_id` 和 32 字节公钥的文本形式统一为小写 `0x` 加 64 位十六进制。
- `account_id` 是跨产品授权、索引和持久化身份；`ss58_address` 是派生展示值；钱包是保存密钥并签名的软件，不是链账户类型。
- 当前代码仍存在的 wallet/admin/owner/pubkey/address 同义字段必须按任务卡分步删除，不得新增或保留兼容分支。

## 4. 产品矩阵与职责

### 4.1 CitizenChain

- 代码目录：`/Users/rhett/GMB/citizenchain`
- 产品文档：`/Users/rhett/GMB/memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
- 内部 OnChina 文档：`/Users/rhett/GMB/memory/01-architecture/onchina/ONCHINA_TECHNICAL.md`
- 技术栈：
  - 链节点与 Runtime：Rust + Substrate / Polkadot SDK
  - 桌面节点 UI：Rust + Tauri + React + TypeScript + Vite
  - OnChina：Rust + Axum + PostgreSQL + React + TypeScript + Vite
- 核心职责：
  - 链上状态机与共识
  - 治理、发行、交易、资格接入
  - 原生节点程序与桌面节点软件
  - OnChina 多机构工作台、注册局业务、行政区、机构登记、管理后台、公开查询和链侧凭证

### 4.2 CitizenApp

- 代码目录：`/Users/rhett/GMB/citizenapp`
- 产品文档：`/Users/rhett/GMB/memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- 技术栈：Flutter + Dart + Isar + smoldot
- 核心职责：
  - 在线钱包与端上签名
  - 治理和交易入口
  - 内置轻节点链上读取与交易提交
  - 公民身份状态展示
  - Cloudflare 边缘服务接入：Chat 瞬时密文/信令转发、广场媒体/feed、轻节点启动清单和受控签名交易转发

### 4.3 CitizenWallet

- 代码目录：`/Users/rhett/GMB/citizenwallet`
- 产品文档：尚未单独建立，新增前必须单独确认。
- 技术栈：Flutter + Dart
- 核心职责：
  - 离线钱包和冷签名
  - QR_V1 扫码识别
  - 中文确认页展示
  - 签名响应二维码生成

### 4.4 Website

- 代码目录：`/Users/rhett/GMB/citizenweb`
- 产品文档：尚未单独建立，新增前必须单独确认。
- 技术栈：React + TypeScript + Vite
- 核心职责：
  - 官方网站静态页面
  - 对外展示资料和发布说明入口

## 5. 跨产品主流程

### 5.1 公民绑定主流程

1. OnChina 注册局管理员录入公民电子护照身份。
2. CitizenApp 查询身份状态并提交账户绑定签名。
3. OnChina 验签后写入结构化绑定结果。
4. CitizenChain 通过链侧凭证或公开查询承接资格结果。
5. CitizenApp 读取绑定状态并展示用户身份能力。

### 5.2 链上投票主流程

1. CitizenChain 创建内部投票、联合投票、选举投票或立法投票提案。
2. OnChina 提供人口快照、投票资格校验和投票凭证签名。
3. CitizenApp 作为公民端入口提交投票签名或交易。
4. CitizenChain runtime 完成投票记账、状态流转与结果处理。

OnChina 不实现投票流程；投票流程统一归属投票引擎。

### 5.3 管理员扫码签名主流程

1. OnChina 生成 `QR_V1 / k=1 sign_request`。
2. CitizenWallet 扫码、解码、展示中文确认字段并签名。
3. CitizenWallet 生成 `QR_V1 / k=2 sign_response`。
4. OnChina 回收签名响应并完成验签或提交前校验。

CitizenApp 不承担管理员登录或冷钱包确认职责。

### 5.4 CitizenApp 链连接与边缘服务主流程

1. CitizenApp 启动后先读取本地 chain spec、lightSyncState 和缓存数据库，启动内置 smoldot 轻节点。
2. CitizenApp 通过 Cloudflare 边缘入口获取启动清单、推荐 bootnodes、服务健康状态和聊天/广场入口；这些信息只用于加速和服务发现，不是链上状态真源。
3. CitizenApp 轻节点连接 CitizenChain P2P 网络，并以 finalized 链状态作为余额、身份、提案、投票和交易成功的最终判断。
4. P2P 暂时不可用时，聊天和广场继续走 Cloudflare；链上关键状态进入降级展示，只显示最近 finalized 快照或等待同步状态。
5. 已签名交易可由受控 API 转发到服务节点 RPC，但 API 不接触私钥、不改交易、不把广播成功等同链上成功。

国储会核心节点不作为公民端公共 RPC 入口；生产网络必须拆分核心/权威节点、公开 bootnode、RPC service node、Archive/Indexer 等角色。

### 5.5 公民币订阅主流程

平台订阅与创作者订阅统一使用公民币，并入 `SquarePost` pallet index `34`；详细契约见 `memory/01-architecture/gmb/subscription-part1-tech.md` 和 `memory/07-ai/unified-protocols.md` 的 P-TX-014、P-STORAGE-006。

1. CitizenApp 从 finalized 链状态读取平台价格或创作者付款套餐。
2. CitizenChain 完成首扣，以当前区块唯一共识时间戳按 UTC 真实公历计算到期时间，并登记自动续费调度。
3. 到期后 runtime 无需用户再次签名，自动按最新链上价格从订阅钱包扣款并推进一个真实公历周期；不使用区块高度或固定天数表达期限。
4. 停链期间到期的周期在恢复出块后按到期顺序补扣；余额不足或套餐失效立即终止且不重试。
5. 平台款进入公民链基金会费用账户，创作者款全额进入创作者钱包。
6. CitizenApp 对订阅、取消、换套餐以及创作者覆盖设置自己套餐分别只签名一次并显示链上真实日期；Cloudflare 在交易 finalized 后只用 Bearer 会话与链读复核付款字段、保存镜像和创作者展示资料，镜像及失败重试不得再次生成账户签名或设备请求签名，也不计算日期、不提交扣款、不持有第二份价格真源。
7. OnChina 与 CitizenWallet 只承接公民链基金会平台调价的治理冷签流程，普通订阅保持 CitizenApp 热签。

成为创作者的唯一资格是当前拥有有效平台订阅。创作者改价对存量订阅的下一次真实扣款生效；当前已付周期不补差价。runtime 使用有界到期索引在区块结束阶段自动续费。现有链必须通过 StorageVersion 原地升级保留全部无关状态，禁止重新创世、替换 chainspec 或恢复旧订阅流程。

订阅、取消、换套餐和创作者设置套餐都是用户发起的非系统链上交易，统一收取链上交易费；业务转账金额为零不代表免交易费。runtime 内部自动续费不是外部交易，不另收用户交易费。

## 6. 共享协议与统一口径

### 6.1 QR_V1

- 扫码协议只有一个：`QR_V1`。
- 相关产品和能力：CitizenApp、CitizenWallet、CitizenChain node、OnChina。
- 二维码外层字段、动作码、签名原文、签名响应和中文展示字段必须以 `memory/07-ai/unified-protocols.md` 为唯一登记入口。

### 6.2 链地址与链参数

- 地址编码：`SS58 = 2027`
- 相关产品：CitizenChain、CitizenApp、CitizenWallet、OnChina。
- 地址显示、genesis hash、链 ID、Token 展示和交易 payload 必须跨端一致。

### 6.3 CID 链侧能力

CID 是身份号码和凭证能力，不是独立产品名。链侧能力由 CitizenChain 承接，OnChina 提供数据和凭证来源：

- 机构 CID 登记前置
- 公民身份上链确认
- 投票凭证
- 联合投票人口快照
- 注册局验签账户和管理员集合管理

## 7. 仓库目录结构

```text
GMB/
├── citizenchain/      # 公民链产品代码，含 runtime、node、OnChina
├── citizenapp/        # 公民代码
├── citizenwallet/     # 公民钱包代码
├── citizenweb/           # 官方网站代码
├── memory/            # AI 编程系统与正式文档真源
├── scripts/           # 非部署仓库工具，本机目录
├── citizenconsole/            # 可追踪的本机部署控制台源码，.runtime 与私密材料忽略
├── docs/              # 展示资料与静态发布资料
└── .github/           # CI/CD、审查和安装包流水线
```

## 8. 发布边界

本机统一发布入口固定为 `citizenconsole/` 可视化控制台。不含密钥的控制台源码由 Git 追踪，`.runtime/`、日志、编译产物和私密材料精确忽略。控制台用七个模块图标分别承载 CitizenConsole、CitizenApp Cloudflare、CitizenWeb、CitizenChain WASM（上排四）与 CitizenApp、CitizenWallet、CitizenChain（下排三）；其中 CitizenConsole 卡点击进入专属整页（非弹窗）管理稳定币充值发币订单与发币热钱包，其余六个为部署模块，点开弹窗显示可执行操作、Keychain/GitHub Secrets 状态和每项密钥的简短中文用途，但绝不读取到浏览器或显示密钥明文。

测试部署和 CI 无需密码；production、Release 和服务器部署每次执行前必须通过 macOS Touch ID，失败时不得启动目标命令。部署 Secret 只保存在 macOS Keychain 或 GitHub Secrets，`.ssh`、仓库及根目录不得保留部署私钥明文。GitHub `workflow_dispatch` 使用显式 `mode=ci/release` 隔离构建与发布；服务器部署由本地控制台独立执行，目标服务器直接下载 GitHub 最新成功 CI 产物，CI 模式不得创建 Release 或部署服务器。

CitizenWeb 只保留“测试部署”和“生产部署”两个按钮卡片：“测试部署”在启动前自动停止旧本地测试进程，再在本机构建并启动 `http://127.0.0.1:41732`，不创建测试 Pages 项目；生产部署只更新已经存在的 `citizenweb` Pages 项目，并继续使用 `https://www.crcfrcn.com` 做真实健康检查。官网部署固定使用 `citizenweb/package-lock.json` 锁定的 Wrangler 版本；生产项目存在性门禁只解析 `wrangler pages project list --json`，不得再 grep 表格输出。

- Runtime 升级：修改 `citizenchain/runtime/**` 或被 runtime 直接依赖且影响链上行为的 primitives。
- Native Node / 桌面安装包：修改 `citizenchain/node/**`、桌面前端、Tauri、打包或发布脚本。
- OnChina 服务发布：修改 `citizenchain/onchina/src/**`、`citizenchain/onchina/frontend/**`、数据库、权限、扫码或公开接口。
- Mobile App 发布：修改 `citizenapp/**` 或 `citizenwallet/**`。
- citizenweb 发布：修改 `citizenweb/**`。
- Chain Spec / Genesis 变更：修改 `citizenchain/node` chainspec 或 genesis preset。

## 9. 联调与变更控制

必须同步联调：

- QR_V1、签名和验签：CitizenWallet + 生成方（CitizenApp、CitizenChain node 或 OnChina）。
- 链地址、genesis hash、SS58、交易 payload：CitizenChain + CitizenApp + CitizenWallet + OnChina。
- 绑定、投票凭证、人口快照：CitizenChain + OnChina + CitizenApp。

必须同步更新文档：

- 产品边界变化：更新仓库文档和对应产品文档。
- 模块职责变化：更新产品文档和对应模块文档。
- 共享协议变化：更新 `memory/07-ai/unified-protocols.md`、相关产品文档和模块文档。

## 10. 维护要求

- 不得恢复 旧独立身份系统和旧离线实名系统作为产品目录、产品文档、CI 或部署入口。
- 不得把 OnChina 写成第五个产品。
- 任一产品新增或下线时，必须先更新本文件中的产品矩阵、目录结构与文档索引。
