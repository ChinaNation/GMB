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
- `website` 产品级架构文档尚未单独建立，新增前必须按仓库新增文件规则单独确认。

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
- `website`：官方网站，负责 GMB 对外官网静态站点。

OnChina 不是独立产品；它是 `citizenchain/onchina/` 下的公民链内部能力。

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
  - 轻节点链上读取
  - 公民身份状态展示

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

- 代码目录：`/Users/rhett/GMB/website`
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
├── website/           # 官方网站代码
├── memory/            # AI 编程系统与正式文档真源
├── scripts/           # 本机私密脚本区，必须被 Git 忽略
├── docs/              # 展示资料与静态发布资料
└── .github/           # CI/CD、审查和安装包流水线
```

## 8. 发布边界

- Runtime 升级：修改 `citizenchain/runtime/**` 或被 runtime 直接依赖且影响链上行为的 primitives。
- Native Node / 桌面安装包：修改 `citizenchain/node/**`、桌面前端、Tauri、打包或发布脚本。
- OnChina 服务发布：修改 `citizenchain/onchina/src/**`、`citizenchain/onchina/frontend/**`、数据库、权限、扫码或公开接口。
- Mobile App 发布：修改 `citizenapp/**` 或 `citizenwallet/**`。
- Website 发布：修改 `website/**`。
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
