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
| Dart / TS 文件 | snake_case 或既有框架风格 | `account_manage_service.dart` |
| 前端功能目录 | snake_case | `admins` |
| Rust 类型 | PascalCase | `InstitutionAccountInfo` |
| Dart / TS 类型 | PascalCase | `InstitutionAccountEntry` |
| 函数 / 方法 | snake_case(Rust) / lowerCamelCase(Dart/TS) | `build_call_data` / `buildCallData` |
| 常量 | SCREAMING_SNAKE_CASE(Rust) / lowerCamelCase 或 static const(Dart) | `MODULE_TAG` / `actionCreate` |
| JSON / API 字段 | snake_case | `signer_pubkey` |
| storage 字段 | PascalCase | `InstitutionAccounts` |
| QR display field key | snake_case | `cid_full_name` |
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
| `memory/01-architecture/qr/` | QR 扫码协议 | qr-protocol | CITIZEN_QR_V1 协议、签名识别、action registry 和 golden fixture 当前详细真源 |
| `memory/01-architecture/citizencode/` | CID 架构 | cid-architecture | CID 产品架构、技术框架和并发框架文档 |
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
| `memory/scripts/` | memory 脚本 | memory-scripts | memory 自检和启动协议验收脚本 |
| `citizenchain/` | 公民链 | citizenchain | runtime、节点、桌面端和打包 |
| `citizenchain/runtime/` | 链上运行时 | runtime | pallet、runtime 配置和链上规则 |
| `citizenchain/node/` | 节点桌面端 | node | 原生节点、Tauri 后端和桌面前端 |
| `citizencode/` | 在线身份系统 | cid | CID 后端、前端和部署配置 |
| `citizencode/backend/number/` | 身份 ID 编码协议 | number | CID 后端身份号码格式、SubjectProperty、机构码、分类、生成和校验唯一源码目录 |
| `citizenpassport/` | 离线实名系统 | cpms | CPMS 后端、前端、数据库和部署配置 |
| `citizenwallet/` | 公民钱包 | citizenwallet | 离线签名、扫码识别和钱包 UI |
| `citizenapp/` | 公民 | citizenapp | Flutter 客户端、钱包、治理和轻节点能力 |
| `citizenapp/im/proto/` | citizenapp 信息协议 | citizenapp-im-proto | 公民 IM 外层 Protobuf schema 真源，不放仓库根目录 proto |
| `citizenapp/lib/isar/` | citizenapp 本地数据库 | citizenapp-isar | citizenapp Isar 本地持久化实体、schema 和数据库入口 |
| `citizenapp/lib/im/` | citizenapp 信息 | citizenapp-im | 公民信息 Tab、聊天详情、统一消息层、端到端加密、消息存储、发送队列和传输抽象 |
| `citizenapp/lib/im/crypto/` | citizenapp 信息加密 | citizenapp-im-crypto | IM 设备密钥、OpenMLS、KeyPackage、安全码和钱包账户绑定 |
| `citizenapp/lib/im/storage/` | citizenapp 信息本地存储 | citizenapp-im-storage | IM 会话、路由缓存、消息、发送队列和附件缓存的本地存储边界 |
| `citizenapp/lib/im/transport/` | citizenapp 信息传输 | citizenapp-im-transport | 通信节点传输、近场传输、自动路由和去重 |
| `citizenapp/android/im/` | Android 信息近场 | android-im | citizenapp Android 近场通信原生模块，优先承载 Nearby Connections 或 Wi-Fi Direct 接入 |
| `citizenapp/ios/im/` | iOS 信息近场 | ios-im | citizenapp iOS 近场通信原生模块，承载 Multipeer Connectivity 接入 |
| `citizenchain/node/src/im/` | 通信节点 IM | node-im | 通信节点密文收件箱、设备绑定、通信端点和 libp2p IM 协议处理模块 |
| `website/` | 官网 | website | GMB 官网前端工程 |
| `docs/` | 文库 | docs | 白皮书唯一真源、展示图片和项目资料；系统规则仍以 `memory/` 为准 |
| `citizenchain/runtime/primitives/src/CitizenConstitution.html` | 公民宪法真源 | citizen-constitution-source | 公民宪法 HTML 唯一真源，编入 runtime WASM，修改后必须通过 runtime 升级生效 |
| `scripts/` | 脚本 | scripts | 仓库级脚本工具、生成器和自动化脚本 |

## 5. 当前核心命名登记

| 中文名称 | English name | 使用位置 | 简介 |
|---|---|---|---|
| 统一命名文件 | `unified-naming.md` | `memory/07-ai/` | 管理目录、文件、字段等新命名 |
| 统一协议文件 | `unified-protocols.md` | `memory/07-ai/` | 管理协议、载荷格式和接口契约 |
| 统一必读文件 | `unified-required-reading.md` | `memory/07-ai/` | 管理每次设计和编程前必须读取的文档 |
| GMB IM 协议 | `GMB_IM_V1` | `memory/07-ai/unified-protocols.md` / `citizenapp/im/proto/im_envelope.proto` / `citizenapp/lib/im/` / `citizenchain/node/src/im/` | 公民 P2P IM 的 Protobuf 外层协议与通信节点接口契约 |
| IM Envelope | `ImEnvelope` | `GMB_IM_V1` / `citizenapp/lib/im/` | IM 外层消息信封，承载 OpenMLS wire bytes、MLS 消息类型、ratchet tree、附件引用和 ack 策略 |
| IM 路由记录 | `ImRouteRecord` | `GMB_IM_V1` / `citizenapp/lib/im/storage/im_isar_store.dart` / `citizenapp/lib/isar/wallet_isar.dart` | IM 内部路由缓存，保存对方钱包聊天账户、设备公钥、安全码和通信节点端点，不替代“我的通讯录” |
| IM KeyPackage | `ImKeyPackage` / `ImMlsKeyPackage` | `GMB_IM_V1` / `citizenchain/node/src/im/keypackage.rs` / `citizenapp/lib/im/crypto/` | OpenMLS 设备预密钥包，发布到自己通信节点的对应钱包账号池并一次性消费 |
| IM OpenMLS native 实现 | `NativeImMlsCrypto` / `ImMlsNativeBindings` | `citizenapp/lib/im/crypto/im_mls_native.dart` | Dart 侧调用现有 `libsmoldot` native 库中的 OpenMLS C ABI，生成真实 KeyPackage、执行 OpenMLS smoke、创建/恢复持久化 MLS 会话 |
| IM OpenMLS 会话模型 | `ImMlsWireMessage` / `ImMlsOutboundMessage` / `ImMlsInboundMessage` / `ImMlsMessageKind` | `citizenapp/lib/im/crypto/im_mls_session.dart` | Dart 侧描述 Welcome/application wire message、首次会话输出顺序和入站处理结果，不实现密码学 |
| IM OpenMLS 状态目录 | `ImMlsStateStore` | `citizenapp/lib/im/crypto/im_mls_state_store.dart` | App 私有 MLS 状态目录和 pending inbound 队列边界，OpenMLS provider storage 仍由 Rust native 写入 |
| IM OpenMLS Rust FFI | `gmb_im_mls_create_key_package_json` / `gmb_im_mls_two_party_smoke_json` / `gmb_im_mls_encrypt_json` / `gmb_im_mls_decrypt_json` | `citizenapp/rust/src/im_mls.rs` | 现有 `libsmoldot` native 库内的 OpenMLS C ABI 边界，不新增第二套 native 库 |
| IM 消息流状态机 | `ImMessageFlow` | `citizenapp/lib/im/im_message_flow.dart` | 远程通信节点链路的发送、接收、pending 重放和 ack 编排 |
| IM 运行态编排 | `ImRuntime` / `ImPairedNodeConfig` | `citizenapp/lib/im/im_runtime.dart` | IM 默认运行态入口，读取用户资料通信账户，连接 OpenMLS、本地 Isar、自己的通信节点端点配置和后续专用 P2P 收发同步 |
| IM 通信节点配对二维码 | `ImNodePairingBody` / `GMB_IM_NODE_PAIRING_V1` / `im_node_pairing` | `citizenapp/lib/qr/bodies/im_node_pairing_body.dart` / `citizenchain/node/src/settings/communication-node/mod.rs` | 公民在“我的 -> 设置 -> 设置通信节点”扫描桌面设置页二维码，保存或更换自己的电脑通信节点 |
| 桌面通信节点功能设置 | `CommunicationNodeState` / `get_communication_node` / `set_communication_node_enabled` | `citizenchain/node/src/settings/communication-node/mod.rs` / `citizenchain/node/frontend/settings/communication-node/` | 区块链软件设置页独立 IM 能力开关，不属于归档/普通全节点模式选择 |
| IM Isar 消息库 | `ImIsarStore` / `ImConversationEntity` / `ImRouteCacheEntity` / `ImMessageEntity` / `ImOutboundQueueEntity` / `ImPendingInboundEntity` | `citizenapp/lib/im/storage/im_isar_store.dart` / `citizenapp/lib/isar/wallet_isar.dart` | 公民端本地会话、路由缓存、消息、出站队列和待处理入站 envelope 持久化 |
| IM 路由缓存记录 | `ImRouteRecord` | `citizenapp/lib/im/storage/im_isar_store.dart` | 公民端 IM 路由缓存模型，保存对方钱包聊天账户、IM 设备公钥、安全码和通信节点端点 |
| IM 聊天页面 | `ImChatPage` | `citizenapp/lib/im/im_chat_page.dart` | 通讯录详情“消息”按钮和信息 Tab 会话列表共用的聊天详情页，使用 `flutter_chat_ui` 展示本地消息，默认由 `ImRuntime` 注入真实 P2P/MLS 发送和同步回调 |
| IM 聊天 UI 适配器 | `imStoredMessageToChatMessage` / `imStoredMessagesToChatMessages` | `citizenapp/lib/im/im_chat_ui_adapter.dart` | 将本地 IM 消息记录转换为 `flutter_chat_core.Message`，避免 UI 层直接读取 Isar entity |
| IM 节点端点 | `ImNodeEndpoint` / `ImPrivateNodeEndpoint` | `citizenchain/node/src/im/endpoint.rs` / `citizenapp/lib/im/transport/` | 通信节点的 IPv4、IPv6、dns4、dnsaddr multiaddr 入口模型 |
| IM 设备绑定请求 | `RegisterImDeviceRequest` / `ImBindingPayload` | `GMB_IM_V1` / `citizenchain/node/src/im/binding.rs` / `citizenapp/lib/im/crypto/` | 钱包聊天账户、IM 设备密钥和通信节点的绑定载荷 |
| IM 直连投递请求 | `ImDirectDeliveryRequest` | `citizenchain/node/src/im/direct.rs` | 显式 PeerId + multiaddr 到对方通信节点的密文投递请求 |
| IM 直连 KeyPackage 请求 | `ImDirectKeyPackageFetchRequest` / `ImDirectKeyPackageConsumeRequest` | `citizenchain/node/src/im/keypackage.rs` / `citizenapp/lib/im/transport/` | 显式 PeerId + multiaddr 到对方通信节点的 KeyPackage 拉取和消费请求 |
| IM 网络请求 | `ImNetworkRequest` / `ImNetworkResponse` | `citizenchain/node/src/im/network.rs` | `/gmb/im/1` request-response 的 Spike 阶段 JSON wire 请求和响应 |
| Step2D 凭证载荷 fixture | `step2d_credential_payload.json` | `memory/06-quality/fixtures/` | citizenwallet / citizenapp 共享的 ADR-008 Step2D SCALE 字节一致性测试数据 |
| 机构管理 | `organization-manage` | runtime crate / pallet | 机构多签管理 pallet |
| 个人多签管理 | `personal-manage` | runtime crate / pallet | 个人多签管理 pallet |
| 管理员变更 | `admins-change` | runtime crate / pallet | 管理员主体、阈值和管理员变更 |
| 内部投票 | `internal-vote` | runtime crate / pallet | 机构内部治理投票 |
| 联合投票 | `joint-vote` | runtime crate / pallet | 联合治理投票 |
| 机构账户 | `InstitutionAccounts` | storage | 机构账户 storage |
| 个人多签 | `PersonalAccounts` | storage | 个人多签 storage |
| 治理主体 | `Subjects` | storage | 管理员主体 storage |
| 账户级内部投票管理员模型 | `account-admin-internal-vote` | ADR / 文档 | ADR-015 记录的账户级管理员、动态阈值和内部投票治理模型 |
| 机构账户主体 | `InstitutionAccount` | AdminAccountKind / 类型 | 注册机构账户级内部投票主体，已使用 `AdminAccountKind = 0x05`，payload 为账户 `AccountId` 前 32 字节并右填零 |
| 主体身份号码 | `cid_number` | API / call data / storage key | CID 对外身份 ID 字段,所有主体统一使用该字段名 |
| 机构全称 | `cid_full_name` | API / call data | 机构全称,可随机构法定名称变更 |
| 机构简称 | `cid_short_name` | API / call data | 机构简称,用于列表和紧凑展示 |
| 账户名称 | `account_name` | API / call data | 机构账户名 |
| 签发机构 CID 号 | `issuer_cid_number` | credential / call data | 签发凭证的机构 CID 号 |
| 签发机构主账户 | `issuer_main_account` | credential / call data | 签发凭证的机构主账户,用于查询 `admins-change` 管理员真源 |
| 签发管理员公钥 | `signer_pubkey` | credential / call data | 签发机构 `admins` 中实际签名管理员的公钥 |
| 业务作用域省名称 | `scope_province_name` | credential / call data | 凭证适用的省级业务作用域 |
| 业务作用域市名称 | `scope_city_name` | credential / call data | 凭证适用的市级业务作用域 |
| 已签名交易构造器 | `SignedExtrinsicBuilder` / `signed_extrinsic_builder.dart` | `citizenapp/lib/rpc/` | 统一构造 citizenapp 在线 signed extrinsic，固定执行 immortal era 协议 |
| 电子护照档案号 | `archive_no` | CPMS ARCHIVE / CID citizens / citizenapp myid | CPMS 签发的公民档案号，三端统一使用完整字段名 |
| 电子护照护照号 | `passport_no` | CPMS archives / CPMS frontend | CPMS 后端自动生成并印刷在公民护照上的 11 位护照号 |
| 电子护照公民状态 | `citizen_status` | CPMS ARCHIVE / CID citizens / citizenapp myid | CPMS 公民状态，三端统一使用完整字段名 |
| 电子护照选举资格 | `voting_eligible` | CPMS ARCHIVE / CID citizens / citizenapp myid | CPMS 选举资格，三端统一使用完整字段名 |
| 电子护照投票状态 | `vote_status` | CID citizens / citizenapp myid | CID 按 `citizen_status + voting_eligible` 计算出的投票状态，不得和绑定状态混用 |
| 电子护照身份状态 | `identity_status` | CID citizens / citizenapp myid | CID 按公民状态与有效期计算出的身份ID状态，不得和绑定状态混用 |
| 电子护照生效日期 | `valid_from` | CPMS ARCHIVE / CID citizens / citizenapp myid | 电子护照有效期开始日期，格式 `YYYY-MM-DD` |
| 电子护照截止日期 | `valid_until` | CPMS ARCHIVE / CID citizens / citizenapp myid | 电子护照有效期截止日期，格式 `YYYY-MM-DD` |
| 公民状态更新时间 | `status_updated_at` | CPMS ARCHIVE / CID citizens | CID 内部用于拒绝旧档案码覆盖新状态的秒级时间戳 |
| 电子护照钱包地址 | `wallet_address` | CPMS ARCHIVE / CID citizens / citizenapp myid | 用户选择用于电子护照绑定的钱包 SS58 地址 |
| 电子护照钱包公钥 | `wallet_pubkey` | CPMS ARCHIVE / CID citizens / citizenapp myid | `wallet_address` 对应的 32 字节 `0x` hex 公钥 |
| 电子护照钱包签名算法 | `wallet_sig_alg` | CPMS ARCHIVE / CID citizens / citizenapp myid | 固定 `sr25519` |
| 电子护照身份ID | `cid_number` | CID citizens / citizenapp myid | CID 生成并返回给用户展示的身份ID号码 |
| 电子护照绑定状态 | `bind_status` | CID citizens / citizenapp myid | 电子护照绑定流程状态，不得使用 `status` 表达绑定状态 |
| CPMS 编号工具 | `number` | `citizenpassport/backend/number/` | CPMS 后端档案号与护照号生成模块 |
| CPMS 档案生命周期 | `lifecycle` | `citizenpassport/backend/dangan/lifecycle.rs` | CPMS 档案软删除满 100 年后的硬删除与档案号/护照号回收逻辑 |
| CPMS 状态导出 | `export` | `citizenpassport/backend/dangan/export.rs` | CPMS 离线年度状态导出模块，生成 `CPMS_STATUS_EXPORT` 文件 |
| CPMS 状态导出文件 | `CPMS_STATUS_EXPORT` | CPMS/CID 离线 JSON 文件 | CPMS 给 CID 导入的年度状态与档案号绑定释放凭证 |
| CPMS 前端鉴权 | `authz` | `citizenpassport/frontend/authz/` | CPMS 前端登录态上下文和路由守卫 |
| CPMS 前端初始化 | `initialize` | `citizenpassport/frontend/initialize/` | CPMS 前端安装初始化页面、API 和类型 |
| CPMS 前端登录 | `login` | `citizenpassport/frontend/login/` | CPMS 前端 QR-only 登录页面和 API |
| CPMS 前端管理员 | `admins` | `citizenpassport/frontend/admins/` | CPMS 前端管理员页面、操作员管理和年度报告导出 |
| CPMS 前端档案业务 | `dangan` | `citizenpassport/frontend/dangan/` | CPMS 前端档案创建、查询、编辑、软删除和档案 QR 操作 |
| CPMS 前端地址 | `address` | `citizenpassport/frontend/address/` | CPMS 前端镇和地址段查询 API 与类型 |
| 镇下地址段 | `address_unit` | CID china / CPMS archives / CPMS frontend | 镇下面的既有地名地址段，不是行政区，不强制为村或路 |
| 镇下地址段 ID | `address_unit_id` | CPMS archives / address_units | CPMS 档案选择的地址段稳定 ID |
| 详细地址输入段 | `address_detail` | CPMS archives / CPMS frontend | 管理员录入的可变详细地址文本，与地址段组合为完整详细地址 |
| 完整详细地址快照 | `address_full_snapshot` | CPMS archives | 保存时由地址段名称和详细地址输入段组成的只读快照 |
| CPMS 前端二维码 | `qr` | `citizenpassport/frontend/qr/` | CPMS 前端 CITIZEN_QR_V1 解析和浏览器扫码工具 |
| CPMS 前端通用层 | `common` | `citizenpassport/frontend/common/` | CPMS 前端 HTTP 封装、共享类型和通用组件 |

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

- CID 后端源码壳目录。
- CID 后端独立链业务目录。
- CID 前端独立链业务目录。
- CID 前端独立业务 API 目录。
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
| `memory/05-modules/citizenpassport/` | CPMS 模块文档 | cpms-module-docs | CPMS 后端、安装和档案模块文档 |
| `memory/05-modules/citizencode/` | CID 模块文档 | cid-module-docs | CID 后端、前端和业务模块文档 |
| `memory/05-modules/website/` | 官网模块文档 | website-module-docs | 官网模块文档 |
| `memory/05-modules/citizenapp/` | citizenapp 模块文档 | citizenapp-module-docs | citizenapp 移动端模块文档 |
| `memory/05-modules/citizenapp/governance/` | citizenapp 治理 | citizenapp-governance | 移动端治理流程文档 |
| `memory/05-modules/citizenapp/offchain/` | citizenapp 链下 | citizenapp-offchain | 移动端链下交互文档 |
| `memory/05-modules/citizenapp/onchain/` | citizenapp 链上 | citizenapp-onchain | 移动端链上交互文档 |
| `memory/05-modules/citizenapp/qr/` | citizenapp QR | citizenapp-qr | 移动端扫码和签名二维码文档 |
| `memory/05-modules/citizenapp/rpc/` | citizenapp RPC | citizenapp-rpc | 移动端 RPC 和轻节点文档 |
| `memory/05-modules/citizenapp/signer/` | citizenapp 签名 | citizenapp-signer | 移动端签名流程文档 |
| `memory/05-modules/citizenapp/user/` | citizenapp 用户 | citizenapp-user | 移动端用户模块文档 |
| `memory/05-modules/citizenapp/wallet/` | citizenapp 钱包 | citizenapp-wallet | 移动端钱包模块文档 |

## 10b. 错误码文档命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `memory/05-modules/citizencode/ERROR_CODES.md` | CID 错误码规范 | cid-error-codes | CID HTTP 状态码、稳定业务错误码和前端错误处理规则 |
| `memory/05-modules/citizenpassport/ERROR_CODES.md` | CPMS 错误码规范 | cpms-error-codes | CPMS 离线系统 HTTP 状态码、稳定业务错误码和前端错误处理规则 |

## 11. CID 功能目录命名登记

### CID 后端目录

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizencode/backend/core/` | 应用核心 | core | 后端启动、路由、HTTP 响应、HTTP 安全、跨模块核心能力和通用链工具 |
| `citizencode/backend/citizens/` | 公民 | citizens | 公民身份与资料管理 |
| `citizencode/backend/citizenpassport/` | CPMS 对接 | cpms | CID 与 CPMS 对接能力 |
| `citizencode/backend/crypto/` | 密码工具 | crypto | 签名、哈希、密钥和密码学工具 |
| `citizencode/backend/indexer/` | 索引器 | indexer | 链上或业务索引能力 |
| `citizencode/backend/gov/` | 公权机构 | gov | 公安局、公权自动目录和公权机构管理接口 |
| `citizencode/backend/private/` | 私权机构 | private | 六类私权机构路由边界;根层不得恢复总 handler |
| `citizencode/backend/private/common/` | 私权共用规则 | private-common | 私权类型到主体属性、机构码、盈利属性和法人资格的规则单一来源 |
| `citizencode/backend/private/sole/` | 个体经营 | sole | 个体经营模型、校验、创建和列表边界 |
| `citizencode/backend/private/partnership/` | 合伙企业 | partnership | 有限合伙和无限合伙模型、校验、创建和列表边界 |
| `citizencode/backend/private/company/` | 股权公司 | company | 股权有限公司/有限责任公司模型、校验、创建和列表边界 |
| `citizencode/backend/private/corporation/` | 股份公司 | corporation | 股份有限公司模型、校验、创建和列表边界 |
| `citizencode/backend/private/welfare/` | 公益组织 | welfare | 非营利法人模型、校验、创建和列表边界 |
| `citizencode/backend/private/association/` | 注册协会 | association | 具有法人资格的协会类组织边界 |
| `citizencode/backend/private/participants/` | 参与人关系 | participants | 负责人、合伙人、股东、成员等通用关系边界 |
| `citizencode/backend/accounts/` | 机构账户 | accounts | 机构多签账户管理接口 |
| `citizencode/backend/docs/` | 机构资料库 | docs | 机构资料上传、下载、列表和删除接口 |
| `citizencode/backend/subjects/` | 身份主体 | subjects | 公权/私权/公民共用主体索引、详情、链端公开查询和非法人能力 |
| `citizencode/backend/admins/login/` | 管理员登录 | admins-login | 管理端登录、扫码登录、鉴权守卫和签名校验 |
| `citizencode/backend/admins/model.rs` | 管理员模型 | admins-model | 联邦注册局机构管理员、市注册局机构管理员和管理员列表 DTO |
| `citizencode/backend/admins/security_model.rs` | 管理员安全模型 | admins-security-model | Passkey、挑战、grant 等管理员安全状态模型 |
| `citizencode/backend/core/qr/` | QR | core-qr | 后端 CITIZEN_QR_V1 协议辅助和统一 sign_request 构造 |
| `citizencode/backend/scope/` | 权限范围 | scope | 权限范围和访问边界 |
| `citizencode/backend/number/` | 身份 ID 编码协议 | number | 身份号码格式、SubjectProperty、机构码、分类、生成和校验规则 |
| `citizencode/backend/china/` | 中国行政区划 | china | SQLite 行政区划真源读取层 |
| `citizencode/backend/admins/` | 管理员 | admins | 联邦注册局机构管理员、市注册局机构管理员、Passkey 和签名挑战写操作 |
| `citizencode/backend/admins/operation_auth.rs` | 管理端操作权限 | operation-auth | CID 管理端 `LOGIN_STATE / PASSKEY / PASSKEY_CHALLENGE` 权限分级真源 |
| `citizencode/backend/store/` | Store | store | Store 聚合体、省级进程内分片缓存和存储边界模型 |
| `citizencode/backend/tests/` | 测试 | tests | 后端测试 |

### CID 前端目录

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizencode/frontend/assets/` | 静态资产 | assets | 图片、字体等前端静态资产 |
| `citizencode/frontend/auth/` | 认证 | auth | 前端登录和认证流程 |
| `citizencode/frontend/citizens/` | 公民 | citizens | 公民管理界面 |
| `citizencode/frontend/core/` | 前端核心 | core | 前端通用组件、共享 UI、扫码账户弹窗、公民钱包签名面板和 QR 工具 |
| `citizencode/frontend/citizenpassport/` | CPMS 对接 | cpms | CPMS 对接界面 |
| `citizencode/frontend/hooks/` | Hooks | hooks | 前端共享 hooks |
| `citizencode/frontend/gov/` | 公权机构 | gov | 公安局和公权机构界面 |
| `citizencode/frontend/private/` | 私权机构 Shell | private | 省市选择、当前私权类型页面和详情跳转 |
| `citizencode/frontend/private/common/` | 私权机构前端共用 | private-common | 共用 API、列表、创建弹窗和单类型页面壳 |
| `citizencode/frontend/private/sole/` | 个体经营前端 | sole | 个体经营页面、API 和类型边界 |
| `citizencode/frontend/private/partnership/` | 合伙企业前端 | partnership | 合伙企业页面、API 和类型边界 |
| `citizencode/frontend/private/company/` | 股权公司前端 | company | 股权公司页面、API 和类型边界 |
| `citizencode/frontend/private/corporation/` | 股份公司前端 | corporation | 股份公司页面、API 和类型边界 |
| `citizencode/frontend/private/welfare/` | 公益组织前端 | welfare | 公益组织页面、API 和类型边界 |
| `citizencode/frontend/private/association/` | 注册协会前端 | association | 注册协会页面、API 和类型边界 |
| `citizencode/frontend/accounts/` | 机构账户 | accounts | 机构账户界面 |
| `citizencode/frontend/docs/` | 机构资料库 | docs | 机构资料库界面 |
| `citizencode/frontend/subjects/` | 身份主体 | subjects | 主体共享类型、字段标签和链端公开查询封装 |
| `citizencode/frontend/core/qr/` | QR | core-qr | 前端二维码解析和 CITIZEN_QR_V1 工具 |
| `citizencode/frontend/china/` | 中国行政区划 | china | 前端行政区划元数据 API 和缓存 |
| `citizencode/frontend/admins/` | 管理员 | admins | 联邦注册局机构管理员、市注册局机构管理员、Passkey 和签名挑战前端流程 |
| `citizencode/frontend/theme/` | 主题 | theme | 主题变量和样式边界 |
| `citizencode/frontend/utils/` | 工具 | utils | 前端通用工具；业务 API 不放在这里 |

## 12. citizenapp 功能目录命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizenapp/lib/citizen/` | 公民 | citizen | citizenapp 底部“公民”Tab 入口、公民投票页和公共页 |
| `citizenapp/lib/governance/organization-manage/` | 机构多签管理 | organization-manage | citizenapp 机构多签创建、关闭、机构账户入口、机构 storage codec 和 OrganizationManage 链上交互；不得承载 personal-manage 个人主业务 |
| `citizenapp/lib/governance/personal-manage/` | 个人多签管理 | personal-manage | citizenapp 个人多签创建、关闭、查询、提案历史、待激活和 PersonalManage 链上编解码 |
| `citizenapp/lib/governance/shared/` | 治理共享 | shared | 治理提案通用模型、上下文、查询、缓存、机构信息和 Subject 解码 |
| `citizenapp/lib/transaction/duoqian-transfer/` | 多签转账 | duoqian-transfer | citizenapp 多签转账提案、详情、投票、余额提示和转账入口 |
| `citizenapp/lib/governance/` | 治理 | governance | 治理提案和投票业务 |
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
| `citizenapp/lib/votingengine/citizen-vote/` | 公民投票 | citizen-vote | citizenapp 公民投票客户端预留目录 |
| `citizenapp/lib/wallet/` | 钱包 | wallet | 钱包账户和资产 |

## 12b. citizenchain node 治理功能目录命名登记

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `citizenchain/node/src/governance/organization-manage/` | 机构多签管理后端 | organization-manage | node Tauri 后端机构多签管理命令、CID 凭证、链上机构详情与创建签名请求 |
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
| `citizenchain/runtime/otherpallet/cid-system/` | CID 系统 | cid-system | 链上 CID 系统 pallet |
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
| `citizenchain/node/src/duoqian_transfer/` | 多签转账后端 | duoqian-transfer-node-backend | node 后端多签转账 Tauri 命令、AccountId 编码和签名提交 |
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
| CID 号码 | `cid_number` | API / call data / storage key | 机构或公民 CID 编号 |
| 机构全称 | `cid_full_name` | API / call data / QR display | 机构全称,可随机构法定名称变更 |
| 机构简称 | `cid_short_name` | API / call data / QR display | 机构简称,用于列表和紧凑展示 |
| 账户名称列表 | `account_names` | CID registration-info API | 机构账户名数组 |
| 账户名称 | `account_name` | API / call data / QR display | 单个机构或个人账户名 |
| 私权机构类型 | `private_type` | CID API / subjects / private | 私权机构目标类型,取值 `SOLE/PARTNERSHIP/COMPANY/CORPORATION/WELFARE/ASSOCIATION` |
| 合伙类型 | `partnership_kind` | CID API / subjects / private | 合伙企业内部类型,取值 `GENERAL/LIMITED` |
| 法人资格 | `has_legal_personality` | CID API / subjects / private | 私权机构是否具有法人资格 |
| 注册随机数 | `register_nonce` | credential / call data | CID 机构注册凭证随机数 |
| 省名称 | `province_name` | API / call data / storage | 行政区省级名称 |
| 签发机构 CID 号 | `issuer_cid_number` | credential / call data | 签发凭证的机构 CID 号 |
| 签发机构主账户 | `issuer_main_account` | credential / call data | 签发凭证的机构主账户,用于查询 `admins-change` 管理员真源 |
| 签发管理员公钥 | `signer_pubkey` | credential / call data | 签发机构 `admins` 中实际签名管理员的公钥 |
| 业务作用域省名称 | `scope_province_name` | credential / call data | 凭证适用的省级业务作用域 |
| 业务作用域市名称 | `scope_city_name` | credential / call data | 凭证适用的市级业务作用域 |
| 签名 | `signature` | credential / call data | 凭证签名 |
| 主体 ID | `account_id` | call data / storage key | 管理员主体统一 ID |
| 公钥 | `pubkey` | QR body | 发起签名请求的公钥 |
| 签名算法 | `sig_alg` | QR body | 签名算法标识 |
| 载荷十六进制 | `payload_hex` | QR body | 待签名或待解码 payload |
| display 字段 key | `display.fields[*].key` | QR body | 展示字段 key，具体值见第 16 节 |

## 16. QR display field key 命名登记

本节登记 `CITIZEN_QR_V1 / sign_request` 中 `body.display.fields[*].key` 的当前命名；字段语义和 action 对照以 `memory/01-architecture/qr/qr-action-registry.md` 为准。

| 中文名称 | English name | 使用位置 | 简介 |
|---|---|---|---|
| 操作 | `action` | QR display | 签名请求动作名 |
| CID 号码 | `cid_number` | QR display | 机构或主体 CID 编号 |
| 机构全称 | `cid_full_name` | QR display | 机构全称 |
| 机构简称 | `cid_short_name` | QR display | 机构简称 |
| 账户名称 | `account_name` | QR display | 单个账户名称 |
| 管理员数量 | `admins_len` | QR display | 管理员总数 |
| 阈值 | `threshold` | QR display | 多签通过阈值 |
| 金额 | `amount_yuan` | QR display | 人民币元口径金额 |
| 总金额 | `total_amount_yuan` | QR display | 总发行或总转账金额 |
| 账户金额 | `amount_<account_name>` | QR display | 按账户名展开的金额字段 |
| 签发省份名称 | `province_name` | QR display | 签发凭证省份名称 |
| 签发管理员公钥 | `signer_pubkey` | QR display | 签发管理员公钥 |
| 提案 ID | `proposal_id` | QR display | 链上提案 ID |
| 是否同意 | `approve` | QR display | 投票是否同意 |
| 收款人 | `beneficiary` | QR display | 转账或关闭后的收款地址 |
| 备注 | `remark` | QR display | 交易备注 |
| 多签账户 | `account` | QR display | 个人或机构多签账户 |
