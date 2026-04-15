# GMB 仓库技术文档（当前实现基线）

## 1. 文档目的
- 固化 `GMB` 仓库当前的产品体系、技术边界、跨产品协作关系与维护规则。
- 作为仓库级技术总入口，帮助开发、联调、测试、发布时快速判断问题属于哪个产品、哪个模块、哪一类发布动作。
- 统一 3 类技术文档的职责，避免仓库文档、产品文档、模块文档之间口径冲突。

## 2. 技术文档体系

### 2.1 三层技术文档
- 仓库技术文档：描述整个仓库的产品体系、共享协议、跨产品流程、维护规则。
- 产品技术文档：描述某一个产品的定位、架构、实现基线、发布边界。
- 模块技术文档：描述某一个功能模块的需求与技术实现，模块文档第 0 部分固定为功能需求。

### 2.2 当前固定文档命名
- 仓库技术文档：
  - `memory/01-architecture/gmb/GMB_TECHNICAL.md`
- 产品技术文档：
  - `memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
  - `memory/01-architecture/sfid/SFID_TECHNICAL.md`
  - `memory/01-architecture/cpms/CPMS_TECHNICAL.md`
  - `memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md`
- 模块技术文档：
  - 位于 `memory/05-modules/` 中，按产品和模块目录归档，命名统一为 `*_TECHNICAL.md`

### 2.3 维护规则
- 改代码之前先读对应层级文档。
- 改代码之后必须回写对应层级文档。
- 改共享协议、共享字段、共享签名串时，必须同步更新所有相关产品文档。
- 改模块代码时必须补中文注释。
- 模块文档与产品文档冲突时，以当前代码实现为准，并在本次改动内修正文档。

## 3. 仓库总体定位

`GMB` 是一个多产品单仓库（monorepo），围绕“公民币区块链 + 身份识别 + 档案系统 + 移动客户端”构建完整数字主权系统。

仓库当前包含 4 个核心产品：
- `citizenchain`：区块链主产品，负责链上状态、共识、治理、发行、交易、节点运行与桌面节点软件。
- `sfid`：身份识别码系统，负责公民绑定、资格校验、人口快照、公民投票凭证、管理员站点与链侧接口。
- `cpms`：离线档案与二维码签发系统，负责建档、签章二维码、机构公钥登记二维码。
- `wuminapp`：移动端客户端，负责钱包、登录签名、治理入口、交易入口与端上状态展示。

## 4. 产品矩阵与职责分工

### 4.1 CitizenChain
- 代码目录：`/Users/rhett/GMB/citizenchain`
- 产品文档：`/Users/rhett/GMB/memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
- 当前技术栈：
  - 链节点与 Runtime：Rust + Substrate / Polkadot SDK
  - 桌面节点 UI：Rust + Tauri + React + TypeScript + Vite
- 核心职责：
  - 链上状态机与共识
  - 治理、发行、交易、资格接入
  - 原生节点程序与桌面节点软件

### 4.2 SFID
- 代码目录：`/Users/rhett/GMB/sfid`
- 产品文档：`/Users/rhett/GMB/memory/01-architecture/sfid/SFID_TECHNICAL.md`
- 当前技术栈：
  - 前端：React + TypeScript + Vite + Ant Design
  - 后端：Rust + Axum + PostgreSQL
- 核心职责：
  - 公民身份绑定与解绑
  - 公民投票资格与绑定有效性查询
  - 人口快照与投票凭证签名
  - 管理员与机构管理

### 4.3 CPMS
- 代码目录：`/Users/rhett/GMB/cpms`
- 产品文档：`/Users/rhett/GMB/memory/01-architecture/cpms/CPMS_TECHNICAL.md`
- 当前技术栈：
  - 后端：Rust + Axum + SQLx + PostgreSQL
  - 前端：当前仓库仅保留 `cpms/frontend/` 预留目录，未落地独立前端实现
- 核心职责：
  - 离线档案录入
  - 档案二维码签发与打印
  - 机构公钥登记二维码生成
  - 机构管理员 / 系统管理员管理

### 4.4 WuminApp
- 代码目录：`/Users/rhett/GMB/wuminapp`
- 产品文档：`/Users/rhett/GMB/memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md`
- 当前技术栈：
  - Flutter + Dart
  - Secure Storage + Isar
- 核心职责：
  - 钱包与端上签名
  - 手机扫码登录
  - 链上交易入口
  - 治理入口与用户状态展示

## 5. 跨产品主流程

### 5.1 公民绑定主流程
1. `citizenchain` 接收链上账户与绑定请求。
2. `CPMS` 在线下生成带签名档案二维码。
3. `SFID` 管理员站扫码并验签二维码，完成档案号与区块链公钥绑定。
4. `SFID` 将绑定结果回传给 `citizenchain`。
5. `wuminapp` 或其他链上客户端读取绑定状态并展示用户身份能力。

### 5.2 公民投票主流程
1. `citizenchain` 创建联合投票 / 公民投票提案。
2. `SFID` 提供人口快照、投票资格校验、投票凭证签名验证。
3. `wuminapp` 作为公民端入口提交投票签名或交易。
4. `citizenchain` 在 runtime 中完成投票记账、状态流转与最终结果处理。

### 5.3 管理员扫码登录主流程
1. `CPMS` 或 `SFID` 生成登录 challenge。
2. `wuminapp` 扫码并完成签名。
3. `CPMS` / `SFID` 回收签名回执并完成验签。
4. 对应管理后台生成会话并授权。

### 5.4 节点部署与使用主流程
1. `citizenchain/node` 提供原生节点程序。
2. `citizenchain/nodeui` 将节点程序以内嵌 sidecar 方式打包为桌面应用。
3. 用户安装桌面节点软件后可直接启动本地节点。
4. `wuminapp` 与其他产品通过 RPC、链侧接口或间接服务读取链状态。

## 6. 共享协议与统一口径

### 6.1 登录扫码协议
- 协议名：`WUMIN_QR_V1`
- 相关产品：
  - `wuminapp`
  - `sfid`
  - `cpms`
- 要求：
  - 挑战字段、签名原文、`aud` 口径必须一致
  - 任一产品改动登录挑战串，必须同步更新其余两个产品文档与实现

### 6.2 CPMS 业务二维码协议
- 生产方：`cpms`
- 消费方：`sfid`
- 用途：
  - 公民档案二维码验签
  - 机构公钥登记二维码验签
- 要求：
  - 字段顺序、签名上下文、状态语义冻结后必须跨产品同步

### 6.3 区块链地址与链参数口径
- 地址编码：`SS58 = 2027`
- 相关产品：
  - `citizenchain`
  - `wuminapp`
  - `sfid`（链侧接口）
- 要求：
  - 地址显示、链 ID、Token 展示口径统一

### 6.4 SFID 链侧五项能力口径
- 机构 SFID 登记前置
- 公民身份绑定凭证
- 公民投票凭证
- 联合投票人口快照
- SFID 主备验签账户管理

这 5 项能力的详细归属在 `memory/01-architecture/sfid/SFID_TECHNICAL.md` 中定义，但其链侧承接能力由 `citizenchain` 负责落地。

## 7. 仓库目录结构与共享层

```text
GMB/
├── citizenchain/   # 区块链主产品代码
├── sfid/           # 身份识别码系统代码
├── cpms/           # 离线档案系统代码
├── wuminapp/       # 移动端客户端代码
├── memory/         # AI 编程系统与正式文档真源
├── tools/          # 环境与开发工具
├── docs/           # 图片、白皮书素材与参考资料
└── .github/        # CI/CD、脚本、安装包流水线
```

### 7.1 `memory/`
- 统一承载 AI 编程系统、仓库级文档、产品文档与模块文档。
- 是正式文档真源，不在产品目录中复制第二份。
- 任意产品实现变更后，都应优先回写 `memory/` 中的对应文档。

### 7.2 `tools/`
- 提供环境安装、辅助脚本、身份相关工具
- `tools/zhujichi.py` 用于批量生成助记词与公钥清单，当前默认输出为 `vault_without_salt.txt` 纯文本文件
- 不承载产品主业务逻辑

### 7.3 `docs/`
- 存放白皮书、图片、参考材料
- 不是产品运行时代码目录

## 8. 发布边界与升级类型

### 8.1 Runtime 升级
- 适用产品：`citizenchain`
- 触发条件：
  - 修改 runtime
  - 修改被 runtime 直接依赖的 pallet
  - 修改被 runtime 直接依赖且影响链上行为的 `primitives`
- 影响：
  - 已运行链需要走 runtime 升级流程，或在开发阶段清库重启新链

### 8.2 Native Node / 安装包发布
- 适用产品：`citizenchain`
- 触发条件：
  - 修改 `node/`
  - 修改 `nodeui/`
  - 修改安装包脚本、打包流水线
- 影响：
  - 一般不要求 runtime 升级
  - 需要重新发布节点二进制或桌面安装包

### 8.3 Web / Backend 发布
- 适用产品：
  - `sfid`
  - `cpms`
- 触发条件：
  - API、数据库、前后端管理页、权限逻辑变更
- 影响：
  - 需要重新部署服务
  - 若涉及共享协议，必须同步联调其他产品

### 8.4 Mobile App 发布
- 适用产品：`wuminapp`
- 触发条件：
  - Flutter UI、钱包、签名、扫码流程、端上存储策略变更
- 影响：
  - 需要重新打包移动端
  - 若涉及共享协议，必须与 `SFID/CPMS/citizenchain` 联调

### 8.5 Chain Spec / Genesis 变更
- 主要发生在 `citizenchain/node` 与 `citizenchain/runtime/src/genesis_config_presets.rs`
- 这类改动很多时候不是“给现有链做 runtime 升级”，而是：
  - 重发 chain spec
  - 启动新链
  - 开发阶段清库重建链

## 9. 联调与变更控制

### 9.1 必须同步联调的改动
- 登录扫码协议改动：`wuminapp + sfid + cpms`
- CPMS 二维码字段 / 签名串改动：`cpms + sfid`
- 链地址口径 / SS58 / 交易字段改动：`citizenchain + wuminapp + sfid`
- 绑定 / 投票凭证 / 人口快照改动：`citizenchain + sfid`

### 9.2 必须同步更新文档的改动
- 产品边界变化：更新仓库文档 + 对应产品文档
- 模块职责变化：更新产品文档 + 对应模块文档
- 共享协议变化：更新仓库文档 + 所有涉及产品文档 + 相关模块文档

### 9.3 推荐开发顺序
1. 先确认改动属于哪个产品。
2. 再判断是产品级改动还是模块级改动。
3. 若涉及共享协议，先冻结字段和签名串。
4. 改代码。
5. 回写模块文档。
6. 若边界或共享口径变化，再回写产品文档与仓库文档。

## 10. 产品技术文档索引
- `memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
- `memory/01-architecture/sfid/SFID_TECHNICAL.md`
- `memory/01-architecture/cpms/CPMS_TECHNICAL.md`
- `memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md`

## 11. 结论性维护要求
- `memory/01-architecture/gmb/GMB_TECHNICAL.md` 只描述仓库级总览、跨产品协议、发布边界与维护规则。
- 单个产品的实现细节不在本文件展开到底层代码级别，而应继续下钻到对应产品技术文档。
- 任一产品新增或下线时，必须先更新本文件中的产品矩阵、目录结构与文档索引。
