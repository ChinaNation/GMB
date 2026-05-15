# GMB 统一命名文件

## 1. 定位

本文件是 GMB AI 编程系统的统一命名入口。

以后任何设计、编码、建文档、建任务卡、建目录、建文件、建字段之前，只要涉及新命名，都必须先查本文件。

本文件统一管理：

- 目录名
- 文件名
- 模块名
- 类名 / 结构体名 / 枚举名
- 函数名 / 方法名
- 变量名 / 常量名
- API 字段名
- storage 字段名
- QR display 字段名
- 任务卡文件名
- 文档文件名

协议名、载荷格式名、接口契约名归 `memory/07-ai/unified-protocols.md` 管；本文件只管命名规则和命名登记。

## 2. 强制规则

1. 所有命名尽量精简，不把需求描述塞进名称里。
2. 不确定的命名必须先报告用户确认，不得擅自新建。
3. 未获得用户明确允许时，不允许新建任何目录或文件；需要新建目录或文件时，必须先列出完整路径、用途和原因，等用户明确同意后才能创建。
4. 新命名必须说明中文名、英文名、使用位置和简介。
5. 同一概念只能有一个当前命名；旧名必须标为废弃或历史。
6. 文件名只表达主题，不表达完整需求；完整中文标题写入文件内容。
7. 目录名只表达边界，不表达流程步骤。
8. 字段名必须表达数据含义，不表达 UI 文案。
9. 跨端同一字段必须同名，除非有明确语言风格差异并在本文件登记。
10. 不允许为规避冲突随意加 `new`、`old`、`v2`、`temp`、`fix`、`final`。
11. 需要中英文名称的地方，中文名用于说明和 UI，英文名用于目录、代码、字段和接口。

## 3. 命名风格

| 对象 | 风格 | 示例 |
|---|---|---|
| 顶层目录 | lowercase | `memory` |
| Rust crate 目录 | kebab-case | `organization-manage` |
| Rust 模块 / 文件 | snake_case | `chain_duoqian_info.rs` |
| Dart / TS 文件 | snake_case 或既有框架风格 | `duoqian_manage_service.dart` |
| 前端功能目录 | snake_case | `sheng_admins` |
| Rust 类型 | PascalCase | `InstitutionAccountInfo` |
| Dart / TS 类型 | PascalCase | `InstitutionAccountEntry` |
| 函数 / 方法 | snake_case(Rust) / lowerCamelCase(Dart/TS) | `build_call_data` / `buildCallData` |
| 常量 | SCREAMING_SNAKE_CASE(Rust) / lowerCamelCase 或 static const(Dart) | `MODULE_TAG` / `actionCreate` |
| JSON / API 字段 | snake_case | `signer_admin_pubkey` |
| storage 字段 | PascalCase | `InstitutionAccounts` |
| QR display field key | snake_case | `institution_name` |
| 任务卡文件名 | 短日期 + 短 slug | `20260507-ai-unified-naming.md` |
| 技术文档文件名 | SCREAMING_SNAKE_CASE | `BACKEND_LAYOUT.md` |

## 4. 目录结构命名总表

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `.github/` | GitHub 自动化 | github-automation | GitHub Actions、PR 模板和仓库自动化脚本 |
| `.githooks/` | Git Hook | git-hooks | 仓库级 Git hook 脚本 |
| `.vscode/` | 编辑器设置 | vscode-settings | 共享 VS Code 工作区设置 |
| `memory/` | AI 系统永久记忆 | memory | 仓库文档、规则、任务卡和 AI 系统主目录 |
| `memory/00-vision/` | 愿景 | vision | 项目目标、信任边界和长期方向 |
| `memory/01-architecture/` | 架构 | architecture | 仓库级和产品级架构文档 |
| `memory/01-architecture/qr/` | QR 扫码协议 | qr-protocol | WUMIN_QR_V1 协议、签名识别、action registry 和 golden fixture 当前详细真源 |
| `memory/01-architecture/sfid/` | SFID 架构 | sfid-architecture | SFID 产品架构、技术框架和并发框架文档 |
| `memory/03-security/` | 安全 | security | 安全规则、边界和风险要求 |
| `memory/04-decisions/` | 架构决策 | decisions | ADR 和重要设计决策 |
| `memory/05-modules/` | 模块文档 | modules | 各产品、各模块技术文档 |
| `memory/05-modules/wuminapp/rpc/` | wuminapp RPC | wuminapp-rpc | wuminapp 轻节点、RPC 和 smoldot 模块技术文档 |
| `memory/06-quality/` | 质量 | quality | 测试、缺陷、变更记录模板 |
| `memory/06-quality/fixtures/` | 测试数据 | fixtures | 跨端共享测试 fixture，作为测试数据唯一真源 |
| `memory/07-ai/` | AI 系统规则 | ai-system | AI 编程系统规则、流程、统一入口 |
| `memory/08-tasks/` | 任务卡 | tasks | open / done / templates 任务记录 |
| `memory/08-tasks/open/` | 未完成任务 | open-tasks | 仍需执行、复核或等待确认的任务卡 |
| `memory/08-tasks/done/` | 已完成任务 | done-tasks | 已完成、已替代或历史保留的任务卡 |
| `memory/08-tasks/templates/` | 任务模板 | task-templates | 任务卡模板 |
| `memory/scripts/` | memory 脚本 | memory-scripts | memory 自检和启动协议验收脚本 |
| `citizenchain/` | 公民链 | citizenchain | runtime、节点、桌面端和打包 |
| `citizenchain/runtime/` | 链上运行时 | runtime | pallet、runtime 配置和链上规则 |
| `citizenchain/node/` | 节点桌面端 | node | 原生节点、Tauri 后端和桌面前端 |
| `sfid/` | 在线身份系统 | sfid | SFID 后端、前端和部署配置 |
| `sfid/backend/sfid/` | SFID 核心规则 | sfid-core | SFID 后端核心身份号码、省市码和校验规则唯一源码目录 |
| `cpms/` | 离线实名系统 | cpms | CPMS 后端、部署配置和预留前端 |
| `wumin/` | 冷钱包 | wumin | 离线签名、扫码识别和冷钱包 UI |
| `wuminapp/` | 手机热钱包 | wuminapp | Flutter 移动端和轻节点能力 |
| `wuminapp/lib/isar/` | wuminapp 本地数据库 | wuminapp-isar | wuminapp Isar 本地持久化实体、schema 和数据库入口 |
| `website/` | 官网 | website | GMB 官网前端工程 |
| `docs/` | 文库 | docs | 白皮书唯一真源、展示图片和项目资料；系统规则仍以 `memory/` 为准 |
| `citizenchain/runtime/primitives/src/CitizenConstitution.html` | 公民宪法真源 | citizen-constitution-source | 公民宪法 HTML 唯一真源，编入 runtime WASM，修改后必须通过 runtime 升级生效 |
| `tools/` | 工具 | tools | 仓库级脚本工具 |
| `scripts/` | 脚本 | scripts | 仓库级自动化脚本 |

## 5. 当前核心命名登记

| 中文名称 | English name | 使用位置 | 简介 |
|---|---|---|---|
| 统一命名文件 | `unified-naming.md` | `memory/07-ai/` | 管理目录、文件、字段等新命名 |
| 统一协议文件 | `unified-protocols.md` | `memory/07-ai/` | 管理协议、载荷格式和接口契约 |
| 统一必读文件 | `unified-required-reading.md` | `memory/07-ai/` | 管理每次设计和编程前必须读取的文档 |
| Step2D 凭证载荷 fixture | `step2d_credential_payload.json` | `memory/06-quality/fixtures/` | wumin / wuminapp 共享的 ADR-008 Step2D SCALE 字节一致性测试数据 |
| 机构管理 | `organization-manage` | runtime crate / pallet | 机构多签管理 pallet |
| 个人多签管理 | `personal-manage` | runtime crate / pallet | 个人多签管理 pallet |
| 管理员变更 | `admins-change` | runtime crate / pallet | 管理员主体、阈值和管理员变更 |
| 内部投票 | `internal-vote` | runtime crate / pallet | 机构内部治理投票 |
| 联合投票 | `joint-vote` | runtime crate / pallet | 联合治理投票 |
| 机构账户 | `InstitutionAccounts` | storage | 机构账户 storage |
| 个人多签 | `PersonalDuoqians` | storage | 个人多签 storage |
| 治理主体 | `Subjects` | storage | 管理员主体 storage |
| 账户级内部投票管理员模型 | `account-admin-internal-vote` | ADR / 文档 | ADR-015 记录的账户级管理员、动态阈值和内部投票治理模型 |
| 机构账户主体 | `InstitutionAccount` | SubjectKind / 类型 | 注册机构账户级内部投票主体，已使用 `SubjectKind = 0x05`，payload 为账户 `AccountId` 前 32 字节并右填零 |
| 机构身份号码 | `sfid_number` | API / call data / storage key | SFID 机构 ID |
| 机构名称 | `institution_name` | API / call data | 机构显示名称 |
| 账户名称 | `account_name` | API / call data | 机构账户名 |
| 签发省份 | `province` | credential / call data | SFID 省级签名来源 |
| 签发管理员公钥 | `signer_admin_pubkey` | credential / call data | 省级签发 admin 公钥 |
| 已签名交易构造器 | `SignedExtrinsicBuilder` / `signed_extrinsic_builder.dart` | `wuminapp/lib/rpc/` | 统一构造 wuminapp 在线 signed extrinsic，固定执行 immortal era 协议 |

## 6. 新命名登记模板

新增命名时，按这个模板登记：

```text
### 中文名称

- English name：
- 类型：目录 / 文件 / 字段 / 类 / 函数 / 常量 / storage / 任务卡 / 文档
- 使用位置：
- 简介：
- 命名理由：
- 是否确认：已确认 / 待确认
```

## 7. 不确定命名处理

以下情况必须先报告确认：

- 同一概念已有 2 个以上候选名
- 中文业务词难以直译
- 命名会影响跨端字段、storage、接口或协议
- 命名会导致目录移动或文件重命名
- 命名会影响用户可见 UI 文案
- 命名需要保留旧词但旧词已被标记为废弃

报告格式：

```text
命名待确认：

对象：
候选 1：
候选 2：
推荐：
原因：
影响范围：
```

## 8. 禁止命名

禁止新增以下命名形态：

- `old_*`
- `new_*`
- `tmp_*`
- `temp_*`
- `final_*`
- `v2_*`
- `fix_*`
- `xxx2`
- `copy`
- `backup`
- 无意义缩写
- 只有拼音且不能稳定表达业务含义的名称

禁止新增或恢复以下目录：

- SFID 后端源码壳目录。
- SFID 后端独立链业务目录。
- SFID 前端独立链业务目录。
- SFID 前端独立业务 API 目录。
- wuminapp 旧大写 Isar 目录。

历史文件或外部工具生成物中已有的，不因此自动修改；新建命名禁止使用。

## 9. 顶层配置与工程文件命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `AGENTS.md` | AI 启动协议 | ai-startup-protocol | Codex / Claude 进入仓库后的最高优先级启动协议 |
| `CODEX.md` | Codex 入口文件 | codex-entry | Codex 入口兼容文件，规则以 `AGENTS.md` 和 `memory/07-ai/` 为准 |
| `CLAUDE.md` | Claude 入口文件 | claude-entry | Claude 入口兼容文件，规则以 `AGENTS.md` 和 `memory/07-ai/` 为准 |
| `README.md` | 仓库说明 | repo-readme | 仓库根说明文件 |
| `Cargo.toml` | Rust 工作区配置 | cargo-workspace-config | Rust workspace 和 crate 依赖配置 |
| `Cargo.lock` | Rust 依赖锁定 | cargo-lockfile | Rust 依赖版本锁定文件 |
| `Dockerfile` | 容器构建文件 | dockerfile | 仓库级容器构建配置 |
| `.dockerignore` | 容器忽略规则 | docker-ignore | Docker 构建上下文忽略规则 |
| `.gitignore` | Git 忽略规则 | git-ignore | Git 工作区忽略规则 |

## 10. `memory/05-modules/` 模块目录命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `memory/05-modules/citizenchain/` | 公民链模块文档 | citizenchain-module-docs | citizenchain runtime、node、桌面端模块文档 |
| `memory/05-modules/cpms/` | CPMS 模块文档 | cpms-module-docs | CPMS 后端、安装和档案模块文档 |
| `memory/05-modules/sfid/` | SFID 模块文档 | sfid-module-docs | SFID 后端、前端和业务模块文档 |
| `memory/05-modules/website/` | 官网模块文档 | website-module-docs | 官网模块文档 |
| `memory/05-modules/wuminapp/` | wuminapp 模块文档 | wuminapp-module-docs | wuminapp 移动端模块文档 |
| `memory/05-modules/wuminapp/governance/` | wuminapp 治理 | wuminapp-governance | 移动端治理流程文档 |
| `memory/05-modules/wuminapp/offchain/` | wuminapp 链下 | wuminapp-offchain | 移动端链下交互文档 |
| `memory/05-modules/wuminapp/onchain/` | wuminapp 链上 | wuminapp-onchain | 移动端链上交互文档 |
| `memory/05-modules/wuminapp/qr/` | wuminapp QR | wuminapp-qr | 移动端扫码和签名二维码文档 |
| `memory/05-modules/wuminapp/rpc/` | wuminapp RPC | wuminapp-rpc | 移动端 RPC 和轻节点文档 |
| `memory/05-modules/wuminapp/signer/` | wuminapp 签名 | wuminapp-signer | 移动端签名流程文档 |
| `memory/05-modules/wuminapp/user/` | wuminapp 用户 | wuminapp-user | 移动端用户模块文档 |
| `memory/05-modules/wuminapp/wallet/` | wuminapp 钱包 | wuminapp-wallet | 移动端钱包模块文档 |

## 11. SFID 功能目录命名登记

### SFID 后端目录

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `sfid/backend/app_core/` | 应用核心 | app-core | 后端启动、路由和跨模块核心能力 |
| `sfid/backend/citizens/` | 公民 | citizens | 公民身份与资料管理 |
| `sfid/backend/cpms/` | CPMS 对接 | cpms | SFID 与 CPMS 对接能力 |
| `sfid/backend/crypto/` | 密码工具 | crypto | 签名、哈希、密钥和密码学工具 |
| `sfid/backend/db/` | 数据库 | db | 数据库连接和迁移边界 |
| `sfid/backend/indexer/` | 索引器 | indexer | 链上或业务索引能力 |
| `sfid/backend/institutions/` | 机构 | institutions | 机构注册、凭证和链端信息接口 |
| `sfid/backend/login/` | 登录 | login | 管理端登录和认证 |
| `sfid/backend/models/` | 数据模型 | models | 后端共享数据模型 |
| `sfid/backend/qr/` | QR | qr | 后端二维码生成和解析 |
| `sfid/backend/scope/` | 权限范围 | scope | 权限范围和访问边界 |
| `sfid/backend/scripts/` | 脚本 | scripts | 后端维护脚本 |
| `sfid/backend/sfid/` | SFID 核心 | sfid-core | 身份号码、省市码和校验规则 |
| `sfid/backend/sheng_admins/` | 省级管理员 | sheng-admins | 省级管理员管理 |
| `sfid/backend/store_shards/` | 存储分片 | store-shards | 存储分片相关能力 |
| `sfid/backend/tests/` | 测试 | tests | 后端测试 |

### SFID 前端目录

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `sfid/frontend/assets/` | 静态资产 | assets | 图片、字体等前端静态资产 |
| `sfid/frontend/auth/` | 认证 | auth | 前端登录和认证流程 |
| `sfid/frontend/citizens/` | 公民 | citizens | 公民管理界面 |
| `sfid/frontend/common/` | 通用组件 | common | 前端通用组件和共享 UI |
| `sfid/frontend/cpms/` | CPMS 对接 | cpms | CPMS 对接界面 |
| `sfid/frontend/hooks/` | Hooks | hooks | 前端共享 hooks |
| `sfid/frontend/institutions/` | 机构 | institutions | 机构管理界面 |
| `sfid/frontend/qr/` | QR | qr | 二维码界面和工具 |
| `sfid/frontend/sfid/` | SFID 核心 | sfid-core | SFID 核心展示和工具 |
| `sfid/frontend/sheng_admins/` | 省级管理员 | sheng-admins | 省级管理员界面 |
| `sfid/frontend/shi_admins/` | 市级管理员 | shi-admins | 市级管理员界面 |
| `sfid/frontend/theme/` | 主题 | theme | 主题变量和样式边界 |
| `sfid/frontend/utils/` | 工具 | utils | 前端通用工具；业务 API 不放在这里 |

## 12. wuminapp 功能目录命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `wuminapp/lib/citizen/` | 公民 | citizen | wuminapp 底部“公民”Tab 入口、公民投票页和公共页 |
| `wuminapp/lib/governance/organization-manage/` | 机构多签管理 | organization-manage | wuminapp 机构多签创建、关闭、机构账户入口、机构 storage codec 和 OrganizationManage 链上交互；不得承载 personal-manage 个人主业务 |
| `wuminapp/lib/governance/personal-manage/` | 个人多签管理 | personal-manage | wuminapp 个人多签创建、关闭、查询、提案历史、待激活和 PersonalManage 链上编解码 |
| `wuminapp/lib/governance/shared/` | 治理共享 | shared | 治理提案通用模型、上下文、查询、缓存、机构信息和 Subject 解码 |
| `wuminapp/lib/transaction/duoqian-transfer/` | 多签转账 | duoqian-transfer | wuminapp 多签转账提案、详情、投票、余额提示和转账入口 |
| `wuminapp/lib/governance/` | 治理 | governance | 治理提案和投票业务 |
| `wuminapp/lib/isar/` | 本地数据库 | isar | Isar 本地持久化实体、schema 和数据库入口 |
| `wuminapp/lib/transaction/offchain-transaction/` | 链下 | offchain | 链下请求和链下业务辅助 |
| `wuminapp/lib/transaction/onchain-transaction/` | 链上 | onchain | 链上交易和链上状态辅助 |
| `wuminapp/lib/transaction/shared/` | 交易共享 | shared | 本地交易记录与 pending 对账等交易共享底座 |
| `wuminapp/lib/qr/` | QR | qr | 扫码、二维码和签名请求 |
| `wuminapp/lib/rpc/` | RPC | rpc | RPC 客户端、轻节点和 extrinsic 构造 |
| `wuminapp/lib/security/` | 安全 | security | 移动端安全策略和工具 |
| `wuminapp/lib/signer/` | 签名 | signer | 移动端签名辅助 |
| `wuminapp/lib/transaction/` | 交易 | transaction | 交易 Tab、链上支付、链下支付、多签转账与交易共享能力 |
| `wuminapp/lib/ui/` | UI | ui | 移动端通用 UI |
| `wuminapp/lib/my/` | 我的 | my | 我的页、电子护照、用户身份和个人工具 |
| `wuminapp/lib/votingengine/internal-vote/` | 内部投票 | internal-vote | wuminapp 内部投票查询、提交、待确认和投票 UI |
| `wuminapp/lib/votingengine/joint-vote/` | 联合投票 | joint-vote | wuminapp 联合投票客户端预留目录 |
| `wuminapp/lib/votingengine/citizen-vote/` | 公民投票 | citizen-vote | wuminapp 公民投票客户端预留目录 |
| `wuminapp/lib/wallet/` | 钱包 | wallet | 钱包账户和资产 |

## 12b. citizenchain node 治理功能目录命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizenchain/node/src/governance/organization-manage/` | 机构多签管理后端 | organization-manage | node Tauri 后端机构多签管理命令、SFID 凭证、链上机构详情与创建签名请求 |
| `citizenchain/node/frontend/governance/organization-manage/` | 机构多签管理前端 | organization-manage | node 前端机构多签管理页面、Tauri API 和 DTO |

## 13. citizenchain runtime 目录命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizenchain/runtime/genesis/` | 创世配置 | genesis | 创世状态和初始配置 |
| `citizenchain/runtime/governance/` | 治理 | governance | 治理类 pallet |
| `citizenchain/runtime/governance/admins-change/` | 管理员变更 | admins-change | 管理员主体、阈值和管理员变更 pallet |
| `citizenchain/runtime/governance/grandpakey-change/` | GRANDPA 密钥变更 | grandpakey-change | GRANDPA authority 变更 pallet |
| `citizenchain/runtime/governance/organization-manage/` | 机构管理 | organization-manage | 机构多签管理 pallet |
| `citizenchain/runtime/governance/personal-manage/` | 个人多签管理 | personal-manage | 个人多签管理 pallet |
| `citizenchain/runtime/governance/resolution-destro/` | 决议销毁 | resolution-destro | 决议销毁 pallet |
| `citizenchain/runtime/governance/runtime-upgrade/` | 运行时升级 | runtime-upgrade | runtime 升级治理 pallet |
| `citizenchain/runtime/issuance/` | 发行 | issuance | 发行类 pallet |
| `citizenchain/runtime/issuance/citizen-issuance/` | 公民发行 | citizen-issuance | 公民发行 pallet |
| `citizenchain/runtime/issuance/fullnode-issuance/` | 全节点发行 | fullnode-issuance | 全节点发行 pallet |
| `citizenchain/runtime/issuance/resolution-issuance/` | 决议发行 | resolution-issuance | 决议发行 pallet |
| `citizenchain/runtime/issuance/shengbank-interest/` | 省行利息 | shengbank-interest | 省行利息 pallet |
| `citizenchain/runtime/otherpallet/` | 其他 pallet | otherpallet | 非治理、非交易、非发行类 pallet |
| `citizenchain/runtime/otherpallet/pow-difficulty/` | PoW 难度 | pow-difficulty | PoW 难度 pallet |
| `citizenchain/runtime/otherpallet/sfid-system/` | SFID 系统 | sfid-system | 链上 SFID 系统 pallet |
| `citizenchain/runtime/primitives/` | 运行时基础类型 | primitives | runtime 共享基础类型 |
| `citizenchain/runtime/src/` | runtime 入口 | runtime-src | runtime 配置、类型和测试入口 |
| `citizenchain/runtime/transaction/` | 交易 | transaction | 交易类 pallet |
| `citizenchain/runtime/transaction/duoqian-transfer/` | 多签转账 | duoqian-transfer | 多签转账 pallet |
| `citizenchain/runtime/transaction/institution-asset/` | 机构资产 | institution-asset | 机构资产 pallet |
| `citizenchain/runtime/transaction/offchain-transaction/` | 链下交易 | offchain-transaction | 链下交易 pallet |
| `citizenchain/runtime/transaction/onchain-transaction/` | 链上交易 | onchain-transaction | 链上交易 pallet |
| `citizenchain/runtime/votingengine/` | 投票引擎 | votingengine | 投票引擎父目录 |
| `citizenchain/runtime/votingengine/citizen-vote/` | 公民投票 | citizen-vote | 公民投票 pallet |
| `citizenchain/runtime/votingengine/internal-vote/` | 内部投票 | internal-vote | 机构或主体内部管理员投票 pallet |
| `citizenchain/runtime/votingengine/joint-vote/` | 联合投票 | joint-vote | 多主体联合投票 pallet |

## 14. citizenchain node 后端目录命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizenchain/node/src/core/` | 核心 | core | 节点核心启动和共享能力 |
| `citizenchain/node/src/desktop/` | 桌面端 | desktop | Tauri 桌面端命令和集成 |
| `citizenchain/node/src/duoqian_transfer/` | 多签转账后端 | duoqian-transfer-node-backend | node 后端多签转账 Tauri 命令、SubjectId 编码和签名提交 |
| `citizenchain/node/src/governance/` | 治理 | governance | 节点治理业务和签名构造 |
| `citizenchain/node/src/home/` | 首页 | home | 桌面端首页后端能力 |
| `citizenchain/node/src/mining/` | 挖矿 | mining | 挖矿业务能力 |
| `citizenchain/node/src/offchain/` | 链下 | offchain | 链下业务、索引和外部服务对接 |
| `citizenchain/node/src/other/` | 其他 | other | 未归入专门边界的节点能力 |
| `citizenchain/node/src/settings/` | 设置 | settings | 节点设置和配置 |
| `citizenchain/node/src/shared/` | 共享 | shared | 节点后端共享类型和工具 |

## 14.1 citizenchain node 前端目录命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizenchain/node/frontend/duoqian-transfer/` | 多签转账前端 | duoqian-transfer-node-frontend | node 前端多签转账创建页、API 和类型 |
| `citizenchain/node/frontend/governance/` | 治理前端 | governance-frontend | node 前端治理机构、提案列表和非多签转账治理页面 |

## 15. API 字段名登记

| 中文名称 | English name | 使用位置 | 简介 |
|---|---|---|---|
| SFID 号码 | `sfid_number` | API / call data / storage key | 机构或公民 SFID 编号 |
| 机构名称 | `institution_name` | API / call data / QR display | 机构显示名称 |
| 账户名称列表 | `account_names` | SFID registration-info API | 机构账户名数组 |
| 账户名称 | `account_name` | API / call data / QR display | 单个机构或个人账户名 |
| 注册随机数 | `register_nonce` | credential / call data | SFID 机构注册凭证随机数 |
| 省份 | `province` | credential / call data | 签发凭证的省级区域 |
| 签发管理员公钥 | `signer_admin_pubkey` | credential / call data | 签发凭证的省级管理员公钥 |
| 签名 | `signature` | credential / call data | 凭证签名 |
| 主体 ID | `subject_id` | call data / storage key | 管理员主体统一 ID |
| 公钥 | `pubkey` | QR body | 发起签名请求的公钥 |
| 签名算法 | `sig_alg` | QR body | 签名算法标识 |
| 载荷十六进制 | `payload_hex` | QR body | 待签名或待解码 payload |
| display 字段 key | `display.fields[*].key` | QR body | 展示字段 key，具体值见第 16 节 |

## 16. QR display field key 命名登记

本节登记 `WUMIN_QR_V1 / sign_request` 中 `body.display.fields[*].key` 的当前命名；字段语义和 action 对照以 `memory/01-architecture/qr/qr-action-registry.md` 为准。

| 中文名称 | English name | 使用位置 | 简介 |
|---|---|---|---|
| 操作 | `action` | QR display | 签名请求动作名 |
| SFID 号码 | `sfid_number` | QR display | 机构或主体 SFID 编号 |
| 机构名称 | `institution_name` | QR display | 机构显示名称 |
| 账户名称 | `account_name` | QR display | 单个账户名称 |
| 管理员数量 | `admin_count` | QR display | 管理员总数 |
| 阈值 | `threshold` | QR display | 多签通过阈值 |
| 金额 | `amount_yuan` | QR display | 人民币元口径金额 |
| 总金额 | `total_amount_yuan` | QR display | 总发行或总转账金额 |
| 账户金额 | `amount_<account_name>` | QR display | 按账户名展开的金额字段 |
| 省份 | `province` | QR display | 签发凭证省份 |
| 签发管理员公钥 | `signer_admin_pubkey` | QR display | 签发管理员公钥 |
| 提案 ID | `proposal_id` | QR display | 链上提案 ID |
| 是否同意 | `approve` | QR display | 投票是否同意 |
| 收款人 | `beneficiary` | QR display | 转账或关闭后的收款地址 |
| 备注 | `remark` | QR display | 交易备注 |
| 多签地址 | `duoqian_address` | QR display | 个人或机构多签地址 |
