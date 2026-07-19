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
- 扫码端解码展示字段名
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
12. 同一个业务语义字段在全仓库必须使用同一个命名。Rust、Dart、TypeScript、SQL、JSON、文档和生成物不得为同一含义另造局部别名；语言风格差异只允许 snake_case ↔ lowerCamelCase，并必须登记在本文件。
13. 禁止用 `name`、`label`、`display_name`、`type`、`status`、`code` 等泛化字段承载已经有明确业务语义的数据。确需局部 UI 变量时只能作为临时展示变量，不得进入 API、DTO、数据库、storage、协议、常量表或持久化模型。
14. 不确定两个字段是否同义时，必须先全仓搜索既有命名和文档登记，再向用户确认；不得自行创造新字段名。

## 3. 命名风格

| 对象 | 风格 | 示例 |
|---|---|---|
| 顶层目录 | lowercase | `memory` |
| Rust crate 目录 | kebab-case | `public-manage` |
| Rust 模块 / 文件 | snake_case | `chain_multisig_info.rs` |
| Dart / TS 文件 | snake_case 或既有框架风格 | `account_manage_service.dart` |
| 前端功能目录 | kebab-case | `personal-manage` |
| Rust 类型 | PascalCase | `InstitutionAccountInfo` |
| Dart / TS 类型 | PascalCase | `InstitutionAccountEntry` |
| 函数 / 方法 | snake_case(Rust) / lowerCamelCase(Dart/TS) | `build_call_data` / `buildCallData` |
| 常量 | SCREAMING_SNAKE_CASE(Rust) / lowerCamelCase 或 static const(Dart) | `MODULE_TAG` / `actionCreate` |
| JSON / API 字段 | snake_case | `signer_pubkey` |
| storage 字段 | PascalCase | `InstitutionAccounts` |
| 扫码端解码展示字段 | snake_case | `cid_full_name` |
| 任务卡文件名 | 短日期 + 短 slug | `20260507-ai-unified-naming.md` |
| 技术文档文件名 | SCREAMING_SNAKE_CASE | `BACKEND_LAYOUT.md` |

Rust 模块目录必须与 Rust 模块名一致，统一使用 `snake_case`；不得用 `#[path = "..."]`
把 `kebab-case` 目录映射成 Rust 模块。Cargo crate/package 目录例外，继续使用
`kebab-case`。

Runtime pallet / crate 的目录名最多两段，例如 `multisig-transfer`、`provincialbank-interest`。
下级字段、函数、API key 和 UI 派生命名最多三段；凡涉及机构账户、机构类型或机构角色，
必须先对照本文件和 `memory/07-ai/institution-naming.md`，按机构标准命名派生，不得用拼音、
临时缩写或历史业务口径另造名称。

## 4. 目录结构命名总表

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `.github/` | GitHub 自动化 | github-automation | GitHub Actions、PR 模板和仓库自动化脚本 |
| `.githooks/` | Git Hook | git-hooks | 仓库级 Git hook 脚本 |
| `.vscode/` | 编辑器设置 | vscode-settings | 共享 VS Code 工作区设置 |
| `memory/` | AI 系统永久记忆 | memory | 仓库文档、规则、任务卡和 AI 系统主目录 |
| `memory/00-vision/` | 愿景 | vision | 项目目标、信任边界和长期方向 |
| `memory/01-architecture/` | 架构 | architecture | 仓库级和产品级架构文档 |
| `memory/01-architecture/qr/` | QR 扫码协议 | qr-protocol | QR_V1 协议、签名识别、action registry 和 golden fixture 当前详细真源 |
| `memory/01-architecture/onchina/` | OnChina 架构 | onchina-architecture | 公民链内置 OnChina 架构、技术框架和并发框架文档 |
| `memory/03-security/` | 安全 | security | 安全规则、边界和风险要求 |
| `memory/04-decisions/` | 架构决策 | decisions | ADR 和重要设计决策 |
| `memory/05-modules/` | 模块文档 | modules | 各产品、各模块技术文档 |
| `memory/05-modules/citizenapp/rpc/` | citizenapp RPC | citizenapp-rpc | citizenapp 轻节点、RPC 和 smoldot 模块技术文档 |
| `memory/06-quality/` | 质量 | quality | 测试、缺陷、变更记录模板 |
| `memory/06-quality/fixtures/` | 测试数据 | fixtures | 跨端共享测试 fixture，作为测试数据唯一真源 |
| `memory/07-ai/` | AI 系统规则 | ai-system | AI 编程系统规则、流程、统一入口 |
| `memory/08-tasks/` | 任务卡 | tasks | open / done / templates 任务记录 |
| `memory/08-tasks/open/` | 未完成任务 | open-tasks | 仍需执行、复核或等待确认的任务卡 |
| `memory/08-tasks/done/` | 已完成任务 | done-tasks | 已完成、已替代或历史保留的任务卡 |
| `memory/08-tasks/templates/` | 任务模板 | task-templates | 任务卡模板 |
| `scripts/` | 仓库脚本 | repo-scripts | 仓库级/AI 工作流/CI 工具脚本(含 memory 自检和启动协议验收) |
| `citizenchain/` | 公民链 | citizenchain | runtime、节点、桌面端和打包 |
| `citizenchain/runtime/` | 链上运行时 | runtime | pallet、runtime 配置和链上规则 |
| `citizenchain/node/` | 节点桌面端 | node | 原生节点、Tauri 后端和桌面前端 |
| `citizenchain/node/src/core/constitution/` | 宪法节点能力 | constitution | 宪法 RAW 数据、独立共识守卫和桌面渲染；`guard.rs` 与 `render.rs` 严格分离 |
| `citizenchain/node/src/core/node_guard/` | 节点守卫 | node-guard | 宪法守卫之外的节点永久规则统一 `BlockImport` 边界；内部策略共享区块预执行与状态导入校验 |
| `citizenchain/onchina/` | OnChina | onchina | 公民链内置多机构工作台、注册局业务、行政区、机构登记、管理后台和链侧凭证能力 |
| `citizenchain/onchina/src/cid/` | 身份 ID 编码协议 | number | OnChina 身份号码格式、SubjectProperty、机构码、分类、生成和校验唯一源码目录 |
| `citizenwallet/` | 公民钱包 | citizenwallet | 离线签名、扫码识别和钱包 UI |
| `citizenapp/` | 公民 | citizenapp | Flutter 客户端、钱包、治理和轻节点能力 |
| `citizenapp/chat/proto/` | CitizenApp 聊天协议 | citizenapp-chat-proto | 公民 Chat 外层 Protobuf schema 真源，不放仓库根目录 proto |
| `citizenapp/lib/isar/` | citizenapp 本地数据库 | citizenapp-isar | citizenapp Isar 本地持久化实体、schema 和数据库入口 |
| `citizenapp/lib/chat/` | CitizenApp 聊天 | citizenapp-chat | 公民聊天 Tab、聊天详情、统一消息层、端到端加密、消息存储、发送队列和传输抽象 |
| `citizenapp/lib/chat/crypto/` | CitizenApp 聊天加密 | citizenapp-chat-crypto | P-256 会话认证、OpenMLS、KeyPackage、安全码和设备绑定 |
| `citizenapp/lib/chat/storage/` | CitizenApp 聊天本地存储 | citizenapp-chat-storage | Chat 会话、路由缓存、消息、发送队列和附件缓存的本地存储边界 |
| `citizenapp/lib/chat/transport/` | CitizenApp 聊天传输 | citizenapp-chat-transport | Cloudflare 瞬时 WebSocket 信封转发、WebRTC 设备间附件和本机重试；云端不保存消息或附件 |
| `citizenweb/` | 官网 | citizenweb | GMB 官网前端工程 |
| `docs/` | 文库 | docs | 白皮书唯一真源、展示图片和项目资料；系统规则仍以 `memory/` 为准 |
| `citizenchain/runtime/public/legislation-yuan/` | 立法院模块 | legislation-yuan | 公民宪法唯一真源（`law_id=0`、`tier=宪法`，创世注入 `constitution.scale` + 立法投票修订）；所有法律统一章>节>条>款，展示端从链上结构化法律重建 HTML（ADR-027） |
| `scripts/` | 脚本 | scripts | 仓库级脚本工具、生成器和自动化脚本 |

## 5. 当前核心命名登记

### 5.0 机构、岗位、权限与管理员目标命名（ADR-039，2026-07-19）

| 中文名称 | English name | 使用位置 | 简介 |
|---|---|---|---|
| 法定代表人姓名 | `legal_representative_name` | runtime / OnChina / CitizenApp / CitizenWallet | 任免生效后的链上公开法定代表人姓名；废弃 `legal_rep_name` |
| 法定代表人 CID 号 | `legal_representative_cid_number` | runtime / OnChina / CitizenApp / CitizenWallet | 法定代表人唯一公民 CID；废弃 `legal_rep_cid_number` |
| 法定代表人账户 | `legal_representative_account` | runtime / OnChina / CitizenApp / CitizenWallet | 法定代表人唯一钱包账户 |
| 机构岗位 | `InstitutionRole` | runtime entity / 跨端模型 | 某机构内的岗位身份和制度事实 |
| 机构管理员任职 | `InstitutionAdminAssignment` | runtime entity / 跨端模型 | 绑定机构、管理员钱包账户、岗位、任期和来源 |
| 岗位授权主体 | `RoleSubject` | runtime entity / votingengine / 业务模块 / 跨端模型 | 完整 `(cid_number, role_code)`；机构业务授权和岗位投票资格的唯一主体，禁止缩成管理员账户或单独岗位码 |
| 业务动作标识 | `BusinessActionId` | runtime 业务模块 / votingengine / 跨端模型 | 固定字段 `module_tag + action_code: u32`，稳定标识业务模块内具体动作；不使用泛化 `action` 字符串作为链上真源 |
| 岗位权限操作 | `RolePermissionOperation` | runtime entity / 业务模块 / 跨端模型 | SCALE discriminant 固定为 `Propose = 0`、`Vote = 1` |
| 岗位业务权限 | `RoleBusinessPermission` | runtime entity / 业务模块 / 跨端模型 | 固定字段 `role_subject + business_action_id + operation`，绑定完整岗位主体与业务动作 |
| 授权主体 | `AuthorizationSubject` | runtime / votingengine / 跨端模型 | `Institution(RoleSubject) = 0`、`PersonalMultisig(AccountId) = 1`；禁止把个人多签混入机构岗位 |
| 投票引擎类型 | `VotingEngineKind` | runtime votingengine / 跨端模型 | `Internal = 0`、`Joint = 1`、`Election = 2`、`Legislation = 3`；只能由业务模块静态选择 |
| 投票计划 | `VotePlan` | runtime 业务模块 / votingengine / 跨端模型 | 固定字段 `business_action_id + proposal_owner + proposer_subject + voter_subjects + voting_engine + business_object_hash` |
| 机构岗位随机数 | `institution_role_nonce` / `InstitutionRoleNonce` | runtime entity 字段 / storage | 每个机构单调递增的动态岗位码生成随机数，不得回退或复用 |
| 已用岗位码 | `UsedRoleCodes` | runtime entity storage | 永久记录机构内已经生成过的岗位码；岗位删除后仍保留占用记录 |
| 任职来源 | `assignment_source` | runtime entity / API / 跨端 | 创世、注册局、普选、互选、提名任免或本机构内部治理 |
| 任职来源记录 | `assignment_source_ref` | runtime entity / API / 跨端 | 对应的选举、投票、登记或任免记录 ID |
| 管理员集合 | `admins` | runtime admins / 跨端 | 当前 `Admin { admin_account, family_name, given_name }` 可任职人员集合；账户本身没有机构业务权限，姓、名只展示 |
| 机构治理动作 | `InstitutionGovernanceAction` | runtime entity / OnChina / CitizenWallet | 本机构内部治理载荷，原子表达管理员集合、岗位任职和法定代表人目标变更 |
| 解除法定代表人 | `clear_legal_representative` | OnChina API / frontend | 机构治理请求中清空链上法定代表人三字段的布尔开关，不得和 `legal_representative_cid_number` 同时提交 |

上述字段不得用 `legal_rep_*`、`admin_role`、`admin_title`、`admin_term_*`、`admin_source_*` 或其他局部别名进入新目标结构。

目标模型不使用泛化 `role_permissions` 字符串集合。岗位权限统一用强类型 `RoleBusinessPermission` 表达，完整授权主体必须是 `RoleSubject(cid_number, role_code)`；业务模块仍是业务动作、指定投票引擎和通过后执行的真源。上述共享类型和跨端 SCALE 已在任务卡第 2 步同步；storage 与业务流程尚未迁移，当前不得把旧 admins 授权实现视为兼容路径。

动态岗位码固定显示为 `R_<32 位大写十六进制>`；域分隔符唯一命名为 `GMB_ROLE_V1`，不得另造别名。

| 中文名称 | English name | 使用位置 | 简介 |
|---|---|---|---|
| 统一命名文件 | `unified-naming.md` | `memory/07-ai/` | 管理目录、文件、字段等新命名 |
| 机构命名规范 | `institution-naming.md` | `memory/07-ai/` | 管理机构具体中英文全称/简称；字段命名仍以本文件为总入口 |
| 统一协议文件 | `unified-protocols.md` | `memory/07-ai/` | 管理协议、载荷格式和接口契约 |
| 统一必读文件 | `unified-required-reading.md` | `memory/07-ai/` | 管理每次设计和编程前必须读取的文档 |
| GMB Chat 协议 | `GMB_CHAT_V1` | `memory/07-ai/unified-protocols.md` / `citizenapp/chat/proto/chat_envelope.proto` / `citizenapp/lib/chat/` / `citizenapp/cloudflare/src/chat/` | 公民私密聊天的 Protobuf 外层协议，统一承载 Cloudflare 互联网聊天和未来近场聊天的 OpenMLS 密文 |
| Chat Envelope | `ChatEnvelope` | `GMB_CHAT_V1` / `citizenapp/lib/chat/` | Chat 外层瞬时信封，只承载 OpenMLS wire bytes、MLS 消息类型和 ratchet tree；不含云端附件引用或确认状态 |
| Chat 路由记录 | `ChatRoute` | `GMB_CHAT_V1` / `citizenapp/lib/chat/storage/chat_store.dart` / `citizenapp/lib/isar/app_isar.dart` | Chat 设备本地路由缓存，保存对方钱包聊天账户、设备公钥、安全码和近场提示，不替代“我的通讯录” |
| Chat KeyPackage | `ChatKeyPackage` / `MlsKeyPackage` | `GMB_CHAT_V1` / `citizenapp/lib/chat/crypto/` / `citizenapp/cloudflare/src/chat/` | OpenMLS 设备预密钥包，发布到对应钱包账户的一次性池并在消费时硬删除 |
| Chat OpenMLS native 实现 | `NativeMlsCrypto` / `MlsNativeBindings` | `citizenapp/lib/chat/crypto/mls_native.dart` | Dart 侧调用现有 `libsmoldot` native 库中的 OpenMLS C ABI，生成真实 KeyPackage、执行 OpenMLS smoke、创建/恢复持久化 MLS 会话 |
| Chat OpenMLS 会话模型 | `MlsWireMessage` / `MlsOutboundMessage` / `MlsInboundMessage` / `MlsMessageKind` | `citizenapp/lib/chat/crypto/mls_session.dart` | Dart 侧描述 Welcome/application wire message、首次会话输出顺序和入站处理结果，不实现密码学 |
| Chat OpenMLS 状态目录 | `MlsStateStore` | `citizenapp/lib/chat/crypto/mls_state_store.dart` | App 私有 MLS 状态目录和 pending inbound 队列边界，OpenMLS provider storage 仍由 Rust native 写入 |
| Chat OpenMLS Rust FFI | `gmb_chat_mls_create_key_package_json` / `gmb_chat_mls_two_party_smoke_json` / `gmb_chat_mls_encrypt_json` / `gmb_chat_mls_decrypt_json` | `citizenapp/rust/src/chat_mls.rs` | 现有 `libsmoldot` native 库内的 OpenMLS C ABI 边界，不新增第二套 native 库 |
| Chat 消息流状态机 | `ChatFlow` | `citizenapp/lib/chat/chat_flow.dart` | 瞬时互联网聊天和近场聊天的发送、接收、设备本地排队与重试编排 |
| Chat 运行态编排 | `ChatRuntime` | `citizenapp/lib/chat/chat_runtime.dart` | Chat 默认运行态入口，连接 OpenMLS、本地 Isar、瞬时 WebSocket、通用推送唤醒和 WebRTC 附件传输 |
| Chat Isar 消息库 | `ChatStore` / `ChatConversationEntity` / `ChatRouteCacheEntity` / `ChatMessageEntity` / `ChatOutboundQueueEntity` / `ChatPendingInboundEntity` | `citizenapp/lib/chat/storage/chat_store.dart` / `citizenapp/lib/isar/app_isar.dart` | 公民端本地会话、路由缓存、消息、出站队列和待处理入站 envelope 持久化 |
| Chat 路由缓存记录 | `ChatRoute` | `citizenapp/lib/chat/storage/chat_store.dart` | 公民端设备本地 Chat 路由缓存模型，保存对方钱包聊天账户、Chat 设备公钥、安全码和近场提示 |
| Chat 聊天页面 | `ChatPage` | `citizenapp/lib/chat/chat_page.dart` | 通讯录详情“消息”按钮和聊天 Tab 会话列表共用的聊天详情页，使用 `flutter_chat_ui` 展示本地消息，默认由 `ChatRuntime` 注入真实 P2P/MLS 发送和同步回调 |
| Chat 聊天 UI 适配器 | `storedMessageToChatMessage` / `storedMessagesToChatMessages` | `citizenapp/lib/chat/chat_ui_adapter.dart` | 将本地 Chat 消息记录转换为 `flutter_chat_core.Message`，避免 UI 层直接读取 Isar entity |
| Chat Cloudflare 传输 | `ChatCloudTransport` | `citizenapp/lib/chat/transport/` / `citizenapp/cloudflare/src/chat/` | 互联网聊天的瞬时 `ChatEnvelope` 与 SDP/ICE 转发；接收方不可达时仅触发通用推送，密文留在发送设备本地队列 |
| Chat 近场传输 | `ChatNearbyTransport` | 规划中的 Android / iOS 原生 transport | 无互联网近场聊天通过蓝牙和 Wi-Fi 手机直连传输同一种 `ChatEnvelope`；未落地前不保留占位目录或空壳 |
| Chat 设备绑定 | `ChatDeviceBinding` / `OP_SIGN_CHAT_DEVICE_BIND` | `citizenapp/lib/chat/crypto/chat_device_binding.dart` / `citizenapp/cloudflare/src/chat/binding.ts` | session owner、MLS 设备标识和设备公钥的 P-256 绑定载荷；不得使用钱包主私钥或客户端账户字段作为授权真源 |
| Step2D 凭证载荷 fixture | `step2d_credential_payload.json` | `memory/06-quality/fixtures/` | CitizenWallet / CitizenApp 共享的 ADR-008 Step2D SCALE 字节一致性测试数据 |
| 公权机构管理 | `public-manage` | runtime crate / pallet | 公权机构生命周期 pallet(idx30) |
| 私权机构管理 | `private-manage` | runtime crate / pallet | 私权机构生命周期 pallet(idx31) |
| 个人多签管理 | `personal-manage` | runtime crate / pallet | 个人多签管理 pallet |
| 节点守卫 | `NodeGuard` / `node_guard` | `citizenchain/node/src/core/node_guard/` | 宪法外节点永久规则的唯一原生导入包装器；不得把 `ConstitutionGuard` 并入其中，也不得为内部策略另建平行包装器 |
| 宪法守卫 | `ConstitutionGuard` / `constitution::guard` | `citizenchain/node/src/core/constitution/guard.rs` | 整条链最高优先级、独立最外层 `BlockImport` 包装器；不得并入 `NodeGuard` 或展示模块 |
| 管理员变更 | `admins-change` | runtime crate / pallet | 管理员主体、阈值和管理员变更 |
| 内部投票 | `internal-vote` | runtime crate / pallet | 机构内部治理投票 |
| 联合投票 | `joint-vote` | runtime crate / pallet | 联合治理投票 |
| 机构账户 | `InstitutionAccounts` | storage | 机构账户 storage |
| 个人多签 | `PersonalAccounts` | storage | 个人多签 storage |
| 治理主体 | `Subjects` | storage | 管理员主体 storage |
| 账户级内部投票管理员模型 | `account-admin-internal-vote` | ADR / 文档 | ADR-015 记录的账户级管理员、动态阈值和内部投票治理模型 |
| 机构账户主体 | `InstitutionAccount` | AdminAccountKind / 类型 | 注册机构账户级内部投票主体，已使用 `AdminAccountKind = 0x05`，payload 为账户 `AccountId` 前 32 字节并右填零 |
| 机构工作台 | `workspace` / `InstitutionWorkspace` | `citizenchain/onchina/src/workspace/` / `citizenchain/onchina/frontend/workspace/` | OnChina 当前登录机构的工作台框架，注册局、司法院、立法机构和通用机构都通过该框架挂载 UI |
| 机构工作台类型 | `WorkspaceKind` / `workspace_kind` | OnChina auth API / workspace DTO / 前端 workspace 类型 | 当前登录机构对应的工作台类型，取值 `registry`、`judicial`、`legislation`、`generic` |
| 机构工作台分区 | `WorkspaceSection` / `workspace_section` | OnChina auth API / workspace DTO / 前端 workspace 类型 | 工作台顶层分区，固定为 `operations`、`display`、`records` |
| 机构工作台入口 | `WorkspaceAction` / `workspace_action` | OnChina auth API / workspace DTO / 前端 workspace 类型 | 当前机构工作台下的动作或页面入口，例如本机构信息、本机构管理员、护宪终审 |
| 主体身份号码 | `cid_number` | API / call data / storage key | CID 对外身份 ID 字段,所有主体统一使用该字段名 |
| 机构全称 | `cid_full_name` | API / call data | 机构全称,可随机构法定名称变更 |
| 机构简称 | `cid_short_name` | API / call data | 机构简称,用于列表和紧凑展示 |
| 机构英文全称 | `cid_full_name_en` | API / call data / runtime primitives | 机构英文全称,具体名称规范见 `memory/07-ai/institution-naming.md` |
| 机构英文简称 | `cid_short_name_en` | API / call data / runtime primitives | 机构英文简称,具体名称规范见 `memory/07-ai/institution-naming.md` |
| 快照区块高度 | `snapshot_block_number` | JSON manifest / API | 本地快照导出时对应的链上区块高度,创世快照固定为 0 |
| 快照区块哈希 | `snapshot_block_hash` | JSON manifest / API | 本地快照导出时对应的链上区块哈希,创世快照等于 `genesis_hash` |
| 创世哈希 | `genesis_hash` | JSON manifest / API / node RPC | 链身份锚点,来自 `chain_getBlockHash(0)` |
| 状态根 | `state_root` | JSON manifest / chainspec | 快照区块头中的 state root,用于校验轻节点和快照来源 |
| 公权机构根哈希 | `public_institution_root` | CitizenApp 公权机构快照 manifest / 创世链状态包 manifest | 按省级分片 hash 计算出的公权机构快照根哈希,只证明快照内容,不作为真源 |
| 分片哈希表 | `shard_hashes` | CitizenApp 公权机构快照 manifest | 省级公权机构分片文件名到 sha256 的映射 |
| 链投影区块哈希 | `chain_block_hash` | OnChina `chain_projection_state` / API | OnChina 本地投影对应的链上 finalized 区块哈希;创世投影固定等于 `genesis_hash`,不得为空 |
| 链投影区块高度 | `chain_block_number` | OnChina `chain_projection_state` / API | OnChina 本地投影对应的链上 finalized 区块高度 |
| 账户名称 | `account_name` | API / call data | 机构账户名 |
| 操作机构 CID 号 | `actor_cid_number` | credential / call data | 当前操作机构唯一主键；目标授权必须继续携带/解析 `role_code` 并形成完整 `RoleSubject`，不得仅凭该 CID 的 admins 授权 |
| 凭证签发机构 CID 号 | `credential_issuer_cid_number` | credential / call data | 注销等凭证中显式记录的签发机构 CID，不得以机构账户替代 |
| 凭证签发管理员公钥 | `credential_signer_pubkey` | credential / call data | `actor_cid_number` 或 `credential_issuer_cid_number` 对应 `admins` 中实际签名管理员公钥 |
| 业务作用域省名称 | `scope_province_name` | credential / call data | 凭证适用的省级业务作用域 |
| 业务作用域市名称 | `scope_city_name` | credential / call data | 凭证适用的市级业务作用域 |
| 已签名交易构造器 | `SignedExtrinsicBuilder` / `signed_extrinsic_builder.dart` | `citizenapp/lib/rpc/` | 统一构造 citizenapp 在线 signed extrinsic，固定执行 immortal era 协议 |
| 电子护照公民状态 | `citizen_status` | CID citizens / citizenapp myid | 注册局维护的公民状态，三端统一使用完整字段名 |
| 电子护照选举资格 | `voting_eligible` | CID citizens / citizenapp myid | 注册局维护的选举资格，三端统一使用完整字段名 |
| 电子护照投票状态 | `vote_status` | CID citizens / citizenapp myid | CID 按 `citizen_status + voting_eligible` 计算出的投票状态，不得和绑定状态混用 |
| 电子护照身份状态 | `identity_status` | CID citizens / citizenapp myid | CID 按公民状态与护照有效期计算出的身份 CID 状态 |
| 电子护照生效日期 | `passport_valid_from` | CID citizens / citizenapp myid | 电子护照有效期开始日期，格式 `YYYY-MM-DD` |
| 电子护照截止日期 | `passport_valid_until` | CID citizens / citizenapp myid | 电子护照有效期截止日期，格式 `YYYY-MM-DD` |
| 公民状态更新时间 | `status_updated_at` | CID citizens | CID 内部用于拒绝旧状态覆盖新状态的秒级时间戳 |
| 电子护照钱包地址 | `wallet_address` | CID citizens / citizenapp myid | 用户选择用于电子护照的钱包 SS58 地址 |
| 电子护照钱包公钥 | `wallet_pubkey` | CID citizens / 后端内部 | `wallet_address` 对应的 32 字节 `0x` hex 公钥,不得在普通前端展示 |
| 电子护照钱包签名算法 | `wallet_sig_alg` | CID citizens / citizenapp myid | 固定 `sr25519` |
| 电子护照身份CID | `cid_number` | CID citizens / citizenapp myid | CID 自动生成并返回给用户展示的身份 CID 号码 |
| 镇下地址名称编号 | `address_name_code` | OnChina china / AddressRegistry | 同一镇下的地址名称 3 位编号，范围 `001..999` |
| 镇下地址名称 | `address_name` | OnChina china / AddressRegistry | 镇下村、路、社区、小区等地址名称 |
| 镇下地址号 | `address_local_no` | OnChina china / AddressRegistry | 同一地址名称下的 4 位地址号，范围 `0001..9999`，可为空 |
| 详细地址输入段 | `address_detail` | OnChina china / AddressRegistry / citizens | 房间、楼层、附属说明等详细地址文本，可为空 |
| 完整详细地址快照 | `address_full_snapshot` | CID citizens | 保存时由省、市、镇、地址名称、地址号和详细地址组成的只读快照 |

## 5.1 机构名称字段硬规则

机构名称只允许以下字段承载:

| 含义 | JSON / Rust / SQL | Dart / TypeScript | 使用边界 |
|---|---|---|---|
| 机构中文全称 | `cid_full_name` | `cidFullName` | API、数据库、链端解码、移动端/桌面端模型 |
| 机构中文简称 | `cid_short_name` | `cidShortName` | 列表、标题左段、紧凑展示 |
| 机构英文全称 | `cid_full_name_en` | `cidFullNameEn` | 内置重要机构、冷钱包/签名摘要 |
| 机构英文简称 | `cid_short_name_en` | `cidShortNameEn` | 内置重要机构、紧凑英文展示 |

禁止用 `name`、`display_name`、`displayName`、`institution_name`、`institutionName`、`org_name`、`orgName`、`subject_name`、`subjectName` 承载机构全称、简称或英文名。

允许继续使用 `name` 的例外:

- 账户名称变量或链上 `name` 参数,但对外字段必须是 `account_name` / `accountName`。
- 钱包名、文件名和自然人姓名；联系人姓名必须使用 `contact_name` / `contactName`。
- UI 局部派生展示变量可以使用 `title` / `label`,但不得作为 API、DTO、数据库或持久化字段承载机构名称。

行政区字典记录不得再使用裸 `name` / `code` 承载对外或持久化字段;必须按层级使用
`country_name` / `country_code`、`province_name` / `province_code`、
`city_name` / `city_code`、`town_name` / `town_code`。泛行政区缓存或通用列表才允许使用
`division_name` / `division_code`,且必须同时携带层级字段。

## 5.2 非机构姓名与展示字段硬规则

非机构姓名和展示字段必须用具体业务语义命名,不得继续使用能承载任意对象名称的 `display_name` / `displayName` / `orgName`。

| 含义 | JSON / SQL / Rust | Dart / TypeScript | 使用边界 |
|---|---|---|---|
| 管理员姓 | `family_name` | `familyName` | runtime / Node / OnChina / CitizenApp / CitizenWallet / SQL / JSON |
| 管理员名 | `given_name` | `givenName` | runtime / Node / OnChina / CitizenApp / CitizenWallet / SQL / JSON |
| 管理员账户选择标签 | `account_label` | `accountLabel` | App 本地管理员账户候选展示,不承载机构名称真源 |
| 钱包候选标签 | `wallet_label` | `walletLabel` | node 前端钱包选择器展示,不承载机构名称真源 |
| 权威节点标签 | `authority_node_label` | `authorityNodeLabel` | node 设置页 bootnode/GRANDPA 绑定展示,不是机构全称或简称 |
| Chat 路由显示名 | `route_display_name` | `routeDisplayName` | Chat 路由缓存和 protobuf 路由记录,不是通讯录真源 |
| 行政区省名称 | `province_name` | `provinceName` | OnChina 行政区 API、App 省份列表、生成物 manifest |
| 行政区市名称 | `city_name` | `cityName` | OnChina 行政区 API、App/前端市级选择 |
| App 行政区内部名称 | `division_name` | `divisionName` | App Isar 行政区缓存内部字段 |
| App 省级展示名称 | `province_display_name` | `provinceDisplayName` | App 省级入口展示 |
| 用户联系人姓名 | `contact_name` | `contactName` | `QR_V1/k=3` body、通讯录导入服务 |
| 转账收款人姓名 | `recipient_name` | `recipientName` | `QR_V1/k=4` body |

管理员姓名不保留合并字段或显示名字段；界面按中文顺序拼接 `family_name + given_name`。缺失值分别使用“管理”“员”。

## 5.3 产品展示名硬规则

产品名同时存在“人读展示名”和“模块 id / 路径名”两层，不得混用。

| 产品 | 中文展示名 | English display name | 模块 id / 路径名 | 使用边界 |
|---|---|---|---|---|
| 公民 | 公民 | `CitizenApp` | `citizenapp` | 人读文案、注释、技术说明使用 `CitizenApp`;Dart package、目录、脚本、bundle id、环境变量和文件路径继续使用 `citizenapp` |
| 公民钱包 | 公民钱包 | `CitizenWallet` | `citizenwallet` | 人读文案、注释、技术说明使用 `CitizenWallet`;Dart package、目录、脚本、bundle id、MethodChannel 和文件路径继续使用 `citizenwallet` |

禁止在 UI 文案、技术说明正文或代码注释中把产品展示名写成 `citizenapp` / `citizenwallet`。只有当它们表示真实路径、package import、脚本名、文件名、环境变量、bundle id、MethodChannel、任务卡 slug 或模块 id 时才允许小写。

## 5.4 废弃旧名映射

以下旧名只允许出现在历史任务卡、历史 ADR、历史变更记录或明确说明“已废弃”的段落中，不得作为当前实现、当前文档路径、代码注释或新命名使用。

| 废弃旧名 | 当前命名 | 类型 | 当前边界 |
|---|---|---|---|
| `uninorg` | `unincorporated_org` | CID 非法人机构目录名 | `citizenchain/onchina/src/subjects/unincorporated_org/` |
| `backend/institutions` | `backend/subjects` | CID 主体共享目录 | `citizenchain/onchina/src/subjects/` |
| `node/src/offchain` | `node/src/transaction/offchain_transaction` | 节点链下交易后端目录 | `citizenchain/node/src/transaction/offchain_transaction/` |
| `node/frontend/offchain` | `node/frontend/transaction/offchain-transaction` | 节点链下交易前端目录 | `citizenchain/node/frontend/transaction/offchain-transaction/` |
| `network-overview` | `network_overview` | Rust 后端模块目录 | `citizenchain/node/src/mining/network_overview/` |
| `bootnodes-address` | `bootnodes_address` | Rust 后端模块目录 | `citizenchain/node/src/settings/bootnodes_address/` |
| `grandpa-address` | `grandpa_address` | Rust 后端模块目录 | `citizenchain/node/src/settings/grandpa_address/` |
| `fee-address` | `fee_account` | Rust 后端模块目录 | `citizenchain/node/src/settings/fee_account/`;节点前端现有 `settings/fee-address/` 仍按前端路径命名登记处理 |
| `duoqian-transfer` / `duoqian_transfer` / `DuoqianTransfer` | `multisig-transfer` / `multisig_transfer` / `MultisigTransfer` | 多签转账 pallet、node 后端、App 前端和文档 | 当前实现只允许 `multisig-transfer` 一套命名 |
| `shengbank-interest` / `shengbank_interest` / `ShengBankInterest` | `provincialbank-interest` / `provincialbank_interest` / `ProvincialBankInterest` | 省行利息 pallet、runtime 配置和文档 | pallet 目录最多两段，统一用 `provincialbank-interest` |
| `nrc_anquan_account` / `anquanAccount` | `safety_fund_account` / `safetyFundAccount` | 国家储委会安全基金账户字段 | 安全基金是账户用途，不使用拼音字段名 |
| `ACCOUNT_NAME_ANQUAN` / `RESERVED_NAME_ANQUAN` / `OP_AN` | `ACCOUNT_NAME_SAFETYFUND` / `RESERVED_NAME_SAFETYFUND` / `OP_SAFETY` | 制度账户名称和派生 op tag | 与 `SAFETY_FUND_ACCOUNT` 统一 |
| `guochuhui` / `shengchuhui` / `shengchuhang` | `nrc` / `prc` / `prb` | 机构角色代码、列表字段和类型派生 | 对应国家储委会、省储委会、省储行标准缩写 |

## 5.5 CitizenApp 会员与 Worker 配置短名

以下名称经用户于 2026-07-11 确认。`CF` 是 Cloudflare 的部署配置缩写；同一
Cloudflare 账户只允许使用一个 `CF_ACCOUNT_ID`，R2、Images、Stream 不得另造账户
字段。会员档与链上身份档必须分开命名，且彼此解耦（ADR-036，任意身份可订任意档）。

| 中文名称 | English name | 类型 | 使用位置与边界 |
|---|---|---|---|
| 自由会员档 | `freedom` | API / D1 / chain enum | 三档 `membership_level` 之一；不得再用身份值 `visitor` 表示会员 |
| 民主会员档 | `democracy` | API / D1 / chain enum | 三档 `membership_level` 之一；不得使用泛化旧值 `visitor_pro` |
| 薪火会员档 | `spark` | API / D1 / chain enum | 三档 `membership_level` 之一（ADR-036 取代旧 `voting`/`candidate` 会员档）；与身份解耦，不绑定 `identity_level` |
| Cloudflare 账户 ID | `CF_ACCOUNT_ID` | Worker secret | R2、Images、Stream 共用的唯一 Cloudflare 账户字段 |
| Cloudflare API Token | `CF_API_TOKEN` | Worker secret | Worker 调用 Images / Stream API 的令牌 |
| R2 Access ID | `R2_ACCESS_ID` | Worker secret | 仅供 Worker 内部签发冷归档回灌只读地址 |
| R2 Secret Key | `R2_SECRET_KEY` | Worker secret | 仅供 Worker 内部签发冷归档回灌只读地址 |
| R2 Bucket | `R2_BUCKET` | Worker var | R2 bucket 名称 |
| Images 地址 | `IMAGES_URL` | Worker var | Images delivery 地址前缀 |
| Stream 地址 | `STREAM_URL` | Worker var | Stream 播放地址前缀 |
| Stream 回调密钥 | `STREAM_HOOK_SECRET` | Worker secret | Stream webhook 签名密钥 |
| 平台会员镜像对账开关 | `MEMBERSHIP_RECONCILE_ENABLED` | Worker var | 只控制平台 finalized 镜像对账，不改变链上真源 |
| 创作者会员镜像对账开关 | `CREATOR_RECONCILE_ENABLED` | Worker var | 只控制创作者 finalized 镜像对账，不改变链上真源 |
| 会员镜像对账批量 | `MEMBERSHIP_RECONCILE_BATCH` | Worker var | 单轮最多核对的镜像行数 |
| Session TTL | `SESSION_TTL_SECONDS` | Worker var | Worker session 有效秒数 |
| Upload TTL | `UPLOAD_TTL_SECONDS` | Worker var | 上传授权有效秒数 |
| 资源限制键 | `resource_key` | TypeScript / D1 / API | 指向 `cloudflare/src/limits/catalog.ts` 的统一硬上限项 |
| 资源预留编号 | `reservation_id` | D1 | 与广场 `upload_id` 同值，标识一次原子额度预留 |
| 资源预留状态 | `reservation_state` | D1 | 只允许 `reserved` / `used` |
| 归档开关 | `ARCHIVE_ENABLED` | Worker var | 退订视频归档开关 |
| 归档等待天数 | `ARCHIVE_LAPSE_DAYS` | Worker var | 权益失效后等待归档的天数 |
| 启动清单 TTL | `BOOT_TTL_SECONDS` | Worker var | 轻节点启动清单缓存秒数 |
| 交易广播开关 | `RELAY_ENABLED` | Worker var | 已签名交易广播开关 |
| 交易广播字节上限 | `RELAY_MAX_BYTES` | Worker var | 单笔已签名交易最大字节数 |
| 交易广播分钟限额 | `RELAY_PER_MINUTE` | Worker var | 单 IP 每分钟广播次数上限 |
| 官网 API 地址 | `VITE_API_URL` | CitizenWeb build var | 官网调用 Worker 的根地址 |
| App API 地址 | `SQUARE_API_URL` | CitizenApp build var | App 调用广场、聊天和链启动清单 Worker 的根地址 |
| 创作者档位 ID | `tier_id` | runtime / Dart / TypeScript / SQL / JSON | 创作者账户内稳定唯一的付款档位 ID；不得使用 `tier_code` 或数组下标表达同一语义 |
| 周期价格列表 | `prices_fen` | runtime / Dart / TypeScript / JSON | 一个创作者档位支持的周期价格集合 |
| 计费周期 | `billing_period` | runtime / Dart / TypeScript / SQL / JSON | 统一取值 `monthly` / `quarterly` / `yearly`，链上标签为 0/1/2 |
| 预期价格（分） | `expected_price_fen` | call data / Dart | 只保护首次扣款或换套餐签名到入块期间的价格，不是续费真源 |
| 待生效套餐 | `pending_plan` | runtime / Dart / TypeScript / SQL | 用户已签名、下一次成功续费时生效的目标套餐 |
| 首次扣款时间 | `started_at` | runtime / Dart / TypeScript / SQL | 首次成功扣款所在 finalized 区块的共识 unix 毫秒时间戳 |
| 最近扣款价格（分） | `last_charged_price_fen` | runtime / Dart / TypeScript / SQL | 最近一次成功扣款审计值，不作为下次扣款依据 |
| 最近扣款时间 | `last_charged_at` | runtime / Dart / TypeScript / SQL | 最近一次成功扣款的共识 unix 毫秒时间 |
| 已付权益截止时间 | `paid_until` | runtime / Dart / TypeScript / SQL | runtime 根据共识时间戳和 UTC 真实公历计算的 unix 毫秒独占上界 |
| 订阅状态 | `subscription_status` | runtime / Dart / TypeScript / SQL | 只允许 `active` / `cancelled` / `terminated` 对应链上枚举 |

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

- OnChina 后端源码壳目录。
- OnChina 后端独立链业务目录。
- OnChina 前端独立链业务目录。
- OnChina 前端独立业务 API 目录。
- citizenapp 旧大写 Isar 目录。

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
| `memory/05-modules/citizenchain/onchina/` | OnChina 模块文档 | onchina-module-docs | OnChina 后端、前端和业务模块文档 |
| `memory/05-modules/citizenweb/` | 官网模块文档 | citizenweb-module-docs | 官网模块文档 |
| `memory/05-modules/citizenapp/` | citizenapp 模块文档 | citizenapp-module-docs | citizenapp 移动端模块文档 |
| `memory/05-modules/citizenapp/governance/` | citizenapp 治理 | citizenapp-governance | 移动端治理流程文档 |
| `memory/05-modules/citizenapp/transaction/offchain-transaction/` | citizenapp 链下交易 | citizenapp-offchain-transaction-docs | 移动端链下交易文档 |
| `memory/05-modules/citizenapp/transaction/onchain-transaction/` | citizenapp 链上交易 | citizenapp-onchain-transaction-docs | 移动端链上交易文档 |
| `memory/05-modules/citizenapp/qr/` | citizenapp QR | citizenapp-qr | 移动端扫码和签名二维码文档 |
| `memory/05-modules/citizenapp/rpc/` | citizenapp RPC | citizenapp-rpc | 移动端 RPC 和轻节点文档 |
| `memory/05-modules/citizenapp/signer/` | citizenapp 签名 | citizenapp-signer | 移动端签名流程文档 |
| `memory/05-modules/citizenapp/user/` | citizenapp 用户 | citizenapp-user | 移动端用户模块文档 |
| `memory/05-modules/citizenapp/wallet/` | citizenapp 钱包 | citizenapp-wallet | 移动端钱包模块文档 |

## 10b. 错误码文档命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `memory/05-modules/citizenchain/onchina/DATA_SECURITY_TECHNICAL.md` | OnChina 数据安全规范 | onchina-data-security | OnChina HTTP 状态码、稳定业务错误码、权限、行政区和前端错误处理规则 |

## 11. OnChina 功能目录命名登记

### OnChina 后端目录

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizenchain/onchina/src/core/` | 应用核心 | core | 后端启动、路由、HTTP 响应、HTTP 安全、跨模块核心能力和通用链工具 |
| `citizenchain/onchina/src/citizens/` | 公民 | citizens | 公民身份与资料管理 |
| `citizenchain/onchina/src/crypto/` | 密码工具 | crypto | 签名、哈希、密钥和密码学工具 |
| `citizenchain/onchina/src/indexer/` | 索引器 | indexer | 链上或业务索引能力 |
| `citizenchain/onchina/src/gov/` | 公权机构 | gov | 公安局、公权自动目录和公权机构管理接口 |
| `citizenchain/onchina/src/private/` | 私权机构 | private | 六类私权机构路由边界;根层不得恢复总 handler |
| `citizenchain/onchina/src/private/common/` | 私权共用规则 | private-common | 私权类型到主体属性、机构码、盈利属性和法人资格的规则单一来源 |
| `citizenchain/onchina/src/private/sole/` | 个体经营 | sole | 个体经营模型、校验、创建和列表边界 |
| `citizenchain/onchina/src/private/partnership/` | 合伙企业 | partnership | 有限合伙和无限合伙模型、校验、创建和列表边界 |
| `citizenchain/onchina/src/private/company/` | 股权公司 | company | 股权有限公司/有限责任公司模型、校验、创建和列表边界 |
| `citizenchain/onchina/src/private/corporation/` | 股份公司 | corporation | 股份有限公司模型、校验、创建和列表边界 |
| `citizenchain/onchina/src/private/welfare/` | 公益组织 | welfare | 非营利法人模型、校验、创建和列表边界 |
| `citizenchain/onchina/src/private/association/` | 注册协会 | association | 具有法人资格的协会类组织边界 |
| `citizenchain/onchina/src/private/participants/` | 参与人关系 | participants | 负责人、合伙人、股东、成员等通用关系边界 |
| `citizenchain/onchina/src/accounts/` | 机构账户 | accounts | 机构多签账户管理接口 |
| `citizenchain/onchina/src/docs/` | 机构资料库 | docs | 机构资料上传、下载、列表和删除接口 |
| `citizenchain/onchina/src/subjects/` | 身份主体 | subjects | 公权/私权/公民共用主体索引、详情、链端公开查询和非法人能力 |
| `citizenchain/onchina/src/admins/login/` | 管理员登录 | admins-login | 管理端登录、扫码登录、鉴权守卫和签名校验 |
| `citizenchain/onchina/src/admins/model.rs` | 管理员模型 | admins-model | 联邦注册局机构管理员、市注册局机构管理员和管理员列表 DTO |
| `citizenchain/onchina/src/admins/security_model.rs` | 管理员安全模型 | admins-security-model | Passkey、挑战、grant 等管理员安全状态模型 |
| `citizenchain/onchina/src/core/qr/` | QR | core-qr | 后端 QR_V1 协议辅助和统一 sign_request 构造 |
| `citizenchain/onchina/src/scope/` | 权限范围 | scope | 权限范围和访问边界 |
| `citizenchain/onchina/src/cid/` | 身份 ID 编码协议 | number | 身份号码格式、SubjectProperty、机构码、分类、生成和校验规则 |
| `citizenchain/onchina/src/cid/china/` | 中国行政区划 | china | SQLite 行政区划真源读取层 |
| `citizenchain/onchina/src/admins/` | 管理员 | admins | 联邦注册局机构管理员、市注册局机构管理员、Passkey 和签名挑战写操作 |
| `citizenchain/onchina/src/admins/operation_auth.rs` | 管理端操作权限 | operation-auth | OnChina 管理端 `LOGIN_STATE / PASSKEY / PASSKEY_CHALLENGE` 权限分级真源 |
| `citizenchain/onchina/src/store/` | Store | store | Store 聚合体、省级进程内分片缓存和存储边界模型 |
| `citizenchain/onchina/src/workspace/` | 机构工作台 | workspace | 后端机构工作台类型、机构码分类、三段式分区和登录态工作台清单生成 |
| `citizenchain/onchina/src/workspace/model.rs` | 机构工作台模型 | workspace-model | `InstitutionWorkspace`、`WorkspaceKind`、`WorkspaceSection` 和 `WorkspaceAction` DTO |
| `citizenchain/onchina/src/workspace/kind.rs` | 工作台分类 | workspace-kind | 机构码到 `workspace_kind` 的分类规则 |
| `citizenchain/onchina/src/workspace/manifest.rs` | 工作台清单 | workspace-manifest | 按能力位生成当前登录机构的操作、显示和记录入口清单 |
| `citizenchain/onchina/src/tests/` | 测试 | tests | 后端测试 |

### OnChina 前端目录

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizenchain/onchina/frontend/assets/` | 静态资产 | assets | 图片、字体等前端静态资产 |
| `citizenchain/onchina/frontend/auth/` | 认证 | auth | 前端登录和认证流程 |
| `citizenchain/onchina/frontend/citizens/` | 公民 | citizens | 公民管理界面 |
| `citizenchain/onchina/frontend/core/` | 前端核心 | core | 前端通用组件、共享 UI、扫码账户弹窗、公民钱包签名面板和 QR 工具 |
| `citizenchain/onchina/frontend/hooks/` | Hooks | hooks | 前端共享 hooks |
| `citizenchain/onchina/frontend/gov/` | 公权机构 | gov | 公安局和公权机构界面 |
| `citizenchain/onchina/frontend/private/` | 私权机构 Shell | private | 省市选择、当前私权类型页面和详情跳转 |
| `citizenchain/onchina/frontend/private/common/` | 私权机构前端共用 | private-common | 共用 API、列表、创建弹窗和单类型页面壳 |
| `citizenchain/onchina/frontend/private/sole/` | 个体经营前端 | sole | 个体经营页面、API 和类型边界 |
| `citizenchain/onchina/frontend/private/partnership/` | 合伙企业前端 | partnership | 合伙企业页面、API 和类型边界 |
| `citizenchain/onchina/frontend/private/company/` | 股权公司前端 | company | 股权公司页面、API 和类型边界 |
| `citizenchain/onchina/frontend/private/corporation/` | 股份公司前端 | corporation | 股份公司页面、API 和类型边界 |
| `citizenchain/onchina/frontend/private/welfare/` | 公益组织前端 | welfare | 公益组织页面、API 和类型边界 |
| `citizenchain/onchina/frontend/private/association/` | 注册协会前端 | association | 注册协会页面、API 和类型边界 |
| `citizenchain/onchina/frontend/accounts/` | 机构账户 | accounts | 机构账户界面 |
| `citizenchain/onchina/frontend/docs/` | 机构资料库 | docs | 机构资料库界面 |
| `citizenchain/onchina/frontend/subjects/` | 身份主体 | subjects | 主体共享类型、字段标签和链端公开查询封装 |
| `citizenchain/onchina/frontend/core/qr/` | QR | core-qr | 前端二维码解析和 QR_V1 工具 |
| `citizenchain/onchina/frontend/china/` | 中国行政区划 | china | 前端行政区划元数据 API 和缓存 |
| `citizenchain/onchina/frontend/admins/` | 管理员 | admins | 联邦注册局机构管理员、市注册局机构管理员、Passkey 和签名挑战前端流程 |
| `citizenchain/onchina/frontend/theme/` | 主题 | theme | 主题变量和样式边界 |
| `citizenchain/onchina/frontend/utils/` | 工具 | utils | 前端通用工具；业务 API 不放在这里 |
| `citizenchain/onchina/frontend/workspace/` | 机构工作台 | workspace | 前端机构工作台路由、通用壳和机构专属 UI 挂载边界 |
| `citizenchain/onchina/frontend/workspace/registry/` | 注册局工作台 | registry-workspace | 注册局既有 UI 的工作台挂载层，不改注册局业务 UI |
| `citizenchain/onchina/frontend/workspace/judicial/` | 司法院工作台 | judicial-workspace | 国家司法院专属工作台，按操作、显示、记录分类 |
| `citizenchain/onchina/frontend/workspace/generic/` | 通用机构工作台 | generic-workspace | 未落专属 UI 的公权、私权和非法人机构通用工作台 |

## 12. citizenapp 功能目录命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizenapp/lib/citizen/` | 公民 | citizen | citizenapp 底部“公民”Tab 入口、投票交互和公共页 |
| `citizenapp/lib/citizen/proposal/` | 公民提案 | proposal | citizenapp 统一发起提案入口、提案能力表、管理员更换、协议升级和提案详情路由 |
| `citizenapp/lib/citizen/shared/` | 公民共享 | shared | 公民页共享机构模型、提案模型、上下文、查询、缓存、账户列表和共用详情 |
| `citizenapp/lib/citizen/institution/` | 机构管理(机构组织生命周期) | institution | citizenapp 机构身份/账户/管理员**只读**链访问核心(InstitutionChainService + multisig_storage_codec + governance_registry + institution_pallet_router 按机构码路由 PublicManage/PrivateManage)+ 统一机构模型(ADR-028);机构创建/关闭已收归 onchina,不在此 |
| `citizenapp/lib/transaction/multisig-transfer/` | 多签转账(交易业务) | multisig-transfer | citizenapp 公权/私权/个人**共用**的多签转账交易(从 citizen/proposal/transaction 迁入 transaction 交易域) |
| `citizenapp/lib/transaction/personal-manage/` | 个人多签管理 | personal-manage | citizenapp 个人多签创建、关闭、查询、提案历史、待激活和 PersonalManage 链上编解码 |
| `citizenapp/lib/transaction/multisig-transfer/` | 多签转账 | multisig-transfer | citizenapp 多签转账提案、详情、投票、余额提示和转账入口 |
| `citizenapp/lib/citizen/governance/` | 治理视图 | governance | 公民 Tab 的治理机构视图；不得承载提案业务实现 |
| `citizenapp/lib/isar/` | 本地数据库 | isar | Isar 本地持久化实体、schema 和数据库入口 |
| `citizenapp/lib/transaction/offchain-transaction/` | 链下 | offchain | 链下请求和链下业务辅助 |
| `citizenapp/lib/transaction/onchain-transaction/` | 链上 | onchain | 链上交易和链上状态辅助 |
| `citizenapp/lib/transaction/shared/` | 交易共享 | shared | 本地交易记录与 pending 对账等交易共享底座 |
| `citizenapp/lib/qr/` | QR | qr | 扫码、二维码和签名请求 |
| `citizenapp/lib/rpc/` | RPC | rpc | RPC 客户端、轻节点和 extrinsic 构造 |
| `citizenapp/lib/security/` | 安全 | security | 移动端安全策略和工具 |
| `citizenapp/lib/signer/` | 签名 | signer | 移动端签名辅助 |
| `citizenapp/lib/transaction/` | 交易 | transaction | 交易 Tab、链上支付、链下支付、多签转账与交易共享能力 |
| `citizenapp/lib/ui/` | UI | ui | 移动端通用 UI |
| `citizenapp/lib/my/` | 我的 | my | 我的页、电子护照、用户身份和个人工具 |
| `citizenapp/lib/votingengine/internal-vote/` | 内部投票 | internal-vote | citizenapp 内部投票查询、提交、待确认和投票 UI |
| `citizenapp/lib/votingengine/joint-vote/` | 联合投票 | joint-vote | citizenapp 联合投票客户端预留目录 |
| `citizenapp/lib/wallet/` | 钱包 | wallet | 钱包账户和资产 |

## 12b. citizenchain node 治理功能目录命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizenchain/node/src/transaction/offchain_transaction/institution_read/` | 清算行机构只读 | institution-read | node 机构身份链上只读(B0:机构创建/管理已下沉 onchina,node 仅保留清算行所需机构读) |

## 13. citizenchain runtime 目录命名登记

费用协议类型统一命名：

- 唯一类型名：`FeeRoute<AccountId, Balance>`。
- 唯一路由实现名：`RuntimeFeeRouter`。
- 唯一注入接口名：`CallFeeRoute`。
- 五个变体：`Free`、`Onchain`、`Offchain`、`Vote`、`Reject`。
- 付款字段统一为 `payer`；链上计费基数统一为 `transaction_amount`；链下已算费用统一为 `fee_amount`。多付款人的链下批次统一用 `OffchainFeePayer::BatchItemPayers` 表达付款来源，不伪造单一机构付款账户。
- 禁止恢复已删除的费种分类器、付款方提取器或任何并行变体；费用类型与确切付款方必须同时来自 `FeeRoute`。

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizenchain/runtime/genesis/` | 创世配置 | genesis | 创世状态和初始配置 |
| `citizenchain/runtime/governance/` | 治理 | governance | 治理类 pallet |
| `citizenchain/runtime/admins/admin-primitives/` | 管理员共用类型 | admin-primitives | 管理员共用类型、状态、kind 和生命周期 trait |
| `citizenchain/runtime/admins/public-admins/` | 公权管理员 | public-admins | 普通公权机构与固定治理机构管理员 pallet |
| `citizenchain/runtime/admins/private-admins/` | 私权管理员 | private-admins | 普通私权机构管理员 pallet |
| `citizenchain/runtime/admins/personal-admins/` | 个人多签管理员 | personal-admins | 个人多签管理员与个人多签账户 pallet |
| `citizenchain/runtime/governance/grandpakey-change/` | GRANDPA 密钥变更 | grandpakey-change | GRANDPA authority 变更 pallet |
| `citizenchain/runtime/entity/public-manage/` | 公权机构管理 | public-manage | 公权机构生命周期 pallet(idx30) |
| `citizenchain/runtime/entity/private-manage/` | 私权机构管理 | private-manage | 私权机构生命周期 pallet(idx31) |
| `citizenchain/runtime/entity/personal-manage/` | 个人多签管理 | personal-manage | 个人多签管理 pallet |
| `citizenchain/runtime/governance/resolution-destroy/` | 决议销毁 | resolution-destroy | 决议销毁 pallet |
| `citizenchain/runtime/governance/runtime-upgrade/` | 运行时升级 | runtime-upgrade | runtime 升级治理 pallet |
| `citizenchain/runtime/issuance/` | 发行 | issuance | 发行类 pallet |
| `citizenchain/runtime/issuance/citizen-issuance/` | 公民发行 | citizen-issuance | 公民发行 pallet |
| `citizenchain/runtime/issuance/fullnode-issuance/` | 全节点发行 | fullnode-issuance | 全节点发行 pallet |
| `citizenchain/runtime/issuance/resolution-issuance/` | 决议发行 | resolution-issuance | 决议发行 pallet |
| `citizenchain/runtime/issuance/provincialbank-interest/` | 省行利息 | provincialbank-interest | 省行利息 pallet |
| `citizenchain/runtime/misc/` | 其他 pallet | misc | 非治理、非交易、非发行类 pallet |
| `citizenchain/runtime/misc/pow-difficulty/` | PoW 难度 | pow-difficulty | PoW 难度 pallet |
| `citizenchain/runtime/misc/citizen-identity/` | 链上公民身份 pallet | citizen-identity | 投票身份、参选身份、人口统计与投票引擎资格真源 |
| `citizenchain/runtime/primitives/` | 运行时基础类型 | primitives | runtime 共享基础类型 |
| `citizenchain/runtime/src/` | runtime 入口 | runtime-src | runtime 配置、类型和测试入口 |
| `citizenchain/runtime/transaction/` | 交易 | transaction | 交易类 pallet |
| `citizenchain/runtime/transaction/multisig/` | 多签转账 | multisig | 多签转账 pallet |
| `citizenchain/runtime/transaction/institution-asset/` | 机构资产 | institution-asset | 机构资产 pallet |
| `citizenchain/runtime/transaction/offchain/` | 链下交易 | offchain | 链下交易 pallet |
| `citizenchain/runtime/transaction/onchain/` | 链上交易 | onchain | 链上交易与统一手续费模块 |
| `citizenchain/runtime/votingengine/` | 投票引擎 | votingengine | 投票引擎父目录 |
| `citizenchain/runtime/votingengine/election-vote/` | 选举投票 | election-vote | 选举投票 pallet |
| `citizenchain/runtime/votingengine/internal-vote/` | 内部投票 | internal-vote | 机构岗位主体或个人多签主体的内部投票 pallet |
| `citizenchain/runtime/votingengine/joint-vote/` | 联合投票 | joint-vote | 多主体联合投票 pallet |

## 14. citizenchain node 后端目录命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizenchain/node/src/core/` | 核心 | core | 节点核心启动和共享能力 |
| `citizenchain/node/src/desktop/` | 桌面端 | desktop | Tauri 桌面端命令和集成 |
| `citizenchain/node/src/transaction/multisig_transfer/` | 多签转账后端 | multisig-transfer-node-backend | node 后端多签转账 Tauri 命令、AccountId 编码和签名提交 |
| `citizenchain/node/src/governance/` | 治理 | governance | 节点治理业务和签名构造 |
| `citizenchain/node/src/home/` | 首页 | home | 桌面端首页后端能力 |
| `citizenchain/node/src/mining/` | 挖矿 | mining | 挖矿业务能力 |
| `citizenchain/node/src/transaction/offchain_transaction/` | 链下 | offchain | 链下业务、索引和外部服务对接 |
| `citizenchain/node/src/other/` | 其他 | other | 未归入专门边界的节点能力 |
| `citizenchain/node/src/settings/` | 设置 | settings | 节点设置和配置 |
| `citizenchain/node/src/shared/` | 共享 | shared | 节点后端共享类型和工具 |

## 14.1 citizenchain node 前端目录命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizenchain/node/frontend/transaction/multisig-transfer/` | 多签转账前端 | multisig-transfer-node-frontend | node 前端多签转账创建页、API 和类型 |
| `citizenchain/node/frontend/governance/` | 治理前端 | governance-frontend | node 前端治理机构、提案列表和非多签转账治理页面 |

## 15. API 字段名登记

| 中文名称 | English name | 使用位置 | 简介 |
|---|---|---|---|
| CID 号码 | `cid_number` | API / call data / storage key | 机构或公民 CID 编号 |
| 公民护照号 | `passport_no` | OnChina citizens API / SQL | 公民终身唯一护照号,不同于 `cid_number` |
| 公民姓 | `citizen_family_name` | OnChina citizens API / SQL | 公民档案中的姓 |
| 公民名 | `citizen_given_name` | OnChina citizens API / SQL | 公民档案中的名 |
| 公民性别 | `citizen_sex` | OnChina citizens API / SQL | 公民档案性别字段,取值 `MALE/FEMALE` |
| 公民出生日期 | `citizen_birth_date` | OnChina citizens API / SQL | 公民档案出生日期,格式 `YYYY-MM-DD` |
| 护照有效期起 | `passport_valid_from` | OnChina citizens API / SQL | 当前电子护照有效期开始日期 |
| 护照有效期止 | `passport_valid_until` | OnChina citizens API / SQL | 当前电子护照有效期截止日期 |
| 投票账户地址 | `wallet_address` | OnChina citizens API / SQL / frontend | 面向用户展示的 SS58 地址 |
| 投票账户公钥 | `wallet_pubkey` | OnChina citizens SQL / backend internal | 系统验签和查询使用的内部公钥字段,不得在普通前端展示 |
| 机构全称 | `cid_full_name` | API / call data / 扫码端解码展示 | 机构全称,可随机构法定名称变更 |
| 机构简称 | `cid_short_name` | API / call data / 扫码端解码展示 | 机构简称,用于列表和紧凑展示 |
| 机构英文全称 | `cid_full_name_en` | API / call data / 扫码端解码展示 | 机构英文全称 |
| 机构英文简称 | `cid_short_name_en` | API / call data / 扫码端解码展示 | 机构英文简称 |
| 账户名称列表 | `account_names` | CID registration-info API | 机构账户名数组 |
| 账户名称 | `account_name` | API / call data / 扫码端解码展示 | 单个机构或个人账户名 |
| 私权机构类型 | `private_type` | CID API / subjects / private | 私权机构目标类型,取值 `SOLE/PARTNERSHIP/COMPANY/CORPORATION/WELFARE/ASSOCIATION` |
| 合伙类型 | `partnership_kind` | CID API / subjects / private | 合伙企业内部类型,取值 `GENERAL/LIMITED` |
| 法人资格 | `has_legal_personality` | CID API / subjects / private | 私权机构是否具有法人资格 |
| 注册随机数 | `register_nonce` | credential / call data | CID 机构注册凭证随机数 |
| 省名称 | `province_name` | API / call data / storage | 行政区省级名称 |
| 市名称 | `city_name` | API / call data / storage | 行政区市级名称 |
| 管理员姓 | `family_name` | CID admins / auth API / SQL | 管理员真实姓；缺失时为“管理” |
| 管理员名 | `given_name` | CID admins / auth API / SQL | 管理员真实名；缺失时为“员” |
| 工作台 | `workspace` | OnChina auth API / frontend auth state | 当前登录机构的工作台清单对象 |
| 工作台类型 | `workspace_kind` | OnChina auth API / frontend workspace | 当前登录机构工作台类型,取值 `registry` / `judicial` / `legislation` / `generic` |
| 工作台标题 | `workspace_title` | OnChina auth API / frontend workspace | 当前登录机构工作台页面标题,通常由 `cid_short_name` 派生 |
| 工作台分区列表 | `workspace_sections` | OnChina auth API / frontend workspace | 当前工作台可见分区数组 |
| 工作台分区 | `workspace_section` | OnChina auth API / frontend workspace | 单个工作台分区,固定为 `operations` / `display` / `records` |
| 工作台分区标题 | `workspace_section_title` | OnChina auth API / frontend workspace | 单个工作台分区的人读标题 |
| 工作台入口列表 | `workspace_actions` | OnChina auth API / frontend workspace | 当前分区下可见动作或页面入口数组 |
| 工作台入口 | `workspace_action` | OnChina auth API / frontend workspace | 单个动作或页面入口的稳定枚举值 |
| 工作台入口标题 | `workspace_action_title` | OnChina auth API / frontend workspace | 单个动作或页面入口的人读标题 |
| 工作台入口启用状态 | `workspace_action_enabled` | OnChina auth API / frontend workspace | 入口是否已经接入可操作能力；未接入时只能显示禁用态 |
| 管理员账户标签 | `account_label` | App local cache / account selector | 本地展示标签,不作为机构名称真源 |
| 钱包标签 | `wallet_label` | node frontend wallet selector | 钱包候选展示标签,不作为机构名称真源 |
| 权威节点标签 | `authority_node_label` | node settings bootnode / GRANDPA | 节点身份或 GRANDPA 私钥匹配到的权威节点标签,不作为机构名称真源 |
| Chat 路由显示名 | `route_display_name` | Chat protobuf / local cache | 通信路由列表展示,不作为联系人或机构名称真源 |
| 联系人姓名 | `contact_name` | QR body / Dart service | 用户联系方式二维码和通讯录导入服务中的联系人姓名 |
| 加密联系人 ID | `contact_id` | CitizenApp 联系人同步 / Cloudflare D1 | 端侧以通讯录索引密钥对联系人 `address` 做 HMAC-SHA256 得到的 64 位 hex；只用于密文 CRUD，不能替代联系人钱包账户 |
| 收款人姓名 | `recipient_name` | QR body | 用户转账二维码中的收款人姓名 |
| 操作机构 CID 号 | `actor_cid_number` | credential / call data | 当前操作机构唯一主键；目标授权必须继续携带/解析 `role_code` 并形成完整 `RoleSubject`，不得仅凭该 CID 的 admins 授权 |
| 凭证签发机构 CID 号 | `credential_issuer_cid_number` | credential / call data | 注销等凭证中显式记录的签发机构 CID，不得以机构账户替代 |
| 凭证签发管理员公钥 | `credential_signer_pubkey` | credential / call data | `actor_cid_number` 或 `credential_issuer_cid_number` 对应 `admins` 中实际签名管理员公钥 |
| 业务作用域省名称 | `scope_province_name` | credential / call data | 凭证适用的省级业务作用域 |
| 业务作用域市名称 | `scope_city_name` | credential / call data | 凭证适用的市级业务作用域 |
| 平台会员等级 | `membership_level` | subscription call / storage / API / SQL | 平台会员档位，统一为 `freedom` / `democracy` / `spark` |
| 创作者档位 ID | `tier_id` | subscription call / storage / API / SQL | 创作者账户内稳定唯一的付款档位 ID |
| 周期价格列表 | `prices_fen` | subscription storage / API / JSON | 每个档位的计费周期和分单位价格集合 |
| 计费周期 | `billing_period` | subscription call / storage / API / SQL | `monthly` / `quarterly` / `yearly` |
| 预期价格（分） | `expected_price_fen` | subscription call data | 首扣和换套餐入块价格保护，不作为续费价格真源 |
| 待生效套餐 | `pending_plan` | subscription storage / API / SQL | 下一次成功续费时原子替换当前套餐 |
| 首次扣款时间 | `started_at` | subscription storage / API / SQL | 首次成功扣款所在 finalized 区块的共识 unix 毫秒时间戳 |
| 最近扣款价格（分） | `last_charged_price_fen` | subscription storage / API / SQL | 最近成功扣款审计值 |
| 最近扣款时间 | `last_charged_at` | subscription storage / API / SQL | 最近成功扣款的共识 unix 毫秒时间 |
| 已付权益截止时间 | `paid_until` | subscription storage / API / SQL | runtime 按 UTC 真实公历计算的 unix 毫秒独占上界 |
| 订阅状态 | `subscription_status` | subscription storage / API / SQL | `active` / `cancelled` / `terminated` |
| 签名 | `signature` | credential / call data | 凭证签名 |
| 主体 ID | `account_id` | call data / storage key | 管理员主体统一 ID |
| QR 协议版本 | `p` | QR envelope | 固定 `QR_V1` |
| QR 流向码 | `k` | QR envelope | 数字流向码 |
| QR 请求 ID | `i` | QR envelope | 临时码 request/session id |
| QR 过期时间 | `e` | QR envelope | 临时码过期 unix 秒 |
| QR body | `b` | QR envelope | 由 `k` 决定的 body 对象 |
| QR 动作码 | `a` | QR sign_request body | `k=1` 的业务动作码 |
| QR 签名算法码 | `g` | QR sign_request body | 当前 `1 = sr25519` |
| QR 公钥 | `u` | QR sign_request/sign_response body | 32B 公钥 base64url |
| QR payload | `d` | QR sign_request body | 待签 payload bytes base64url |
| QR 签名 | `s` | QR sign_response body | 64B sr25519 signature base64url |

## 16. QR_V1 字段命名登记

本节登记 `QR_V1` 线上字段。扫码确认页的人类展示字段不进入 QR,只能由 `b.a + b.d` 在扫码端解码生成；字段语义和 action 对照以 `memory/01-architecture/qr/qr-action-registry.md` 为准。

| 中文名称 | English name | 使用位置 | 简介 |
|---|---|---|---|
| 协议版本 | `p` | QR envelope | 恒为 `QR_V1` |
| 流向码 | `k` | QR envelope | `1=sign_request,2=sign_response,3=user_contact,4=user_transfer`;旧 `5=chat_node_pairing` 已删除并按未知类型拒绝 |
| 请求 ID | `i` | QR envelope | 临时码 request/session id |
| 过期时间 | `e` | QR envelope | 临时码过期 unix 秒 |
| Body | `b` | QR envelope | body 对象 |
| 动作码 | `a` | `k=1` body | 业务动作码 |
| 签名算法码 | `g` | `k=1` body | 当前 `1 = sr25519` |
| 公钥 | `u` | `k=1/2` body | 32B 公钥 base64url |
| Payload | `d` | `k=1` body | 待签 payload bytes base64url |
| 签名 | `s` | `k=2` body | 64B 签名 base64url |
| 钱包地址 | `address` | `k=3/4` body | SS58 钱包地址 |
| 联系人姓名 | `contact_name` | `k=3` body | 联系人名 |
| 收款人姓名 | `recipient_name` | `k=4` body | 收款人名 |
| 收款金额 | `amount` | `k=4` body | 字符串金额 |
| 币种 | `symbol` | `k=4` body | 当前 `GMB` |
| 备注 | `memo` | `k=4` body | 收款备注 |
| 清算标识 | `bank` | `k=4` body | 清算网络/清算行标识 |
| 节点 PeerId | `node_peer_id` | 旧 `k=5` body | 已删除旧字段；旧通信节点 PeerId，不得恢复 |
| 节点 Multiaddr | `node_multiaddr` | 旧 `k=5` body | 已删除旧字段；旧通信节点 multiaddr，不得恢复 |
| 端点类型 | `endpoint_kind` | 旧 `k=5` body | 已删除旧字段；旧通信节点 `ip4` 或 `ip6`，不得恢复 |
