# 任务卡：机构管理员、岗位和最小首次登记三步改造

## 当前状态

- 状态：进行中
- 当前步骤：正式创世前统一改造第4步 CitizenApp 管理员三字段对齐已完成；下一步为第5步 CitizenWallet 与统一 QR 协议对齐
- 用户确认：2026-07-17
- 执行规则：每一步先确认方案；执行完成后立即更新文档、完善中文注释、清理残留，再输出下一步技术方案

## 任务需求

机构唯一主键继续使用 `cid_number`，但管理员和岗位必须彻底分离：

- 管理员是人，机构和个人多签统一保存在 `admins`，每项字段顺序固定为 `admin_account + family_name + given_name`。
- `admin_account` 是钱包账户和唯一签名授权字段；`family_name`、`given_name` 只用于姓名展示，不参与授权。
- 管理员钱包能从 OnChina 公民资料解析姓名时分别写入姓、名；无法解析时分别使用“管理”“员”，前端按中文顺序合并显示“管理员”。
- 普通机构始终至少有两个管理员；固定治理机构继续遵守制度精确人数。
- 岗位是机构职位，不是管理员；管理员可无岗位，岗位可空缺。
- 每个机构必须默认且唯一存在 `LR / 法定代表人` 岗位；该岗位不可删除、停用、改名或改码。
- 首次创建不自动把管理员任命为法定代表人，法定代表人三字段保持 `None`。
- 岗位任职不能再反向派生或覆盖 `admins`。
- 首次机构登记只提交最小身份资料与管理员，runtime 自动创建制度账户、默认法定代表人岗位和严格多数阈值。
- 注册协会 `SFAS` 的盈利属性按实例选择，不能固定为非盈利。
- runtime、Node、OnChina、公民、CitizenWallet 五端同步，不保留旧载荷或兼容分支。

## 所属模块

- `citizenchain/runtime/admins`
- `citizenchain/runtime/entity`
- `citizenchain/runtime/primitives`
- `citizenchain/runtime/genesis`
- `citizenchain/runtime/src`
- `citizenchain/node`
- `citizenchain/onchina`
- `citizenapp`
- `citizenwallet`
- `memory`

## 输入文档

- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/repo-map.md`
- `memory/03-security/security-rules.md`
- `memory/07-ai/unified-protocols.md`
- `memory/07-ai/unified-naming.md`
- `memory/05-modules/citizenchain/runtime/admins/ADMINS_TECHNICAL.md`
- `memory/05-modules/citizenchain/runtime/entity/entity-primitives/ENTITY_PRIMITIVES_TECHNICAL.md`
- `memory/05-modules/citizenchain/runtime/entity/public-manage/PUBLIC_MANAGE_TECHNICAL.md`
- `memory/05-modules/citizenchain/runtime/entity/private-manage/PRIVATE_MANAGE_TECHNICAL.md`
- `memory/01-architecture/onchina/ONCHINA_TECHNICAL.md`
- `memory/05-modules/citizenchain/onchina/BACKEND_TECHNICAL.md`
- `memory/05-modules/citizenchain/onchina/FRONTEND_TECHNICAL.md`

## 三步范围

### 第1步

- `admins` 改为管理员姓名与钱包账户的人员集合。
- 删除“岗位有效任职并集派生 admins”的链上逻辑。
- 所有机构自动建立唯一 `LR / 法定代表人` 岗位，允许空缺。
- 首次创建载荷收紧为最小身份字段、管理员集合和注册局授权字段。
- runtime 自动派生机构码、全部强制协议账户和严格多数阈值。
- OnChina 按钱包分别解析公民姓、名，无法解析时分别使用“管理”“员”。
- `SFAS` 盈利属性改为实例必选。
- CitizenWallet、Node、CitizenApp 同步新 storage/call 契约。

### 第2步

- 机构管理员新增、删除、换人和姓名更新。
- 普通岗位新增、变更、停用和删除。
- 管理员与岗位任职维护。
- 法定代表人任命、更换、解除及三字段原子更新。
- 普通岗位短随机码唯一生成。

### 第3步

- 五端读侧统一、OnChina 页面入口收口、普通岗位短随机码生成和全仓残留审计。
- 真实本地链、PostgreSQL、OnChina 页面和二维码签名全链路验收。
- 完成最终文档和任务归档。

## 第1步验收标准

- [x] `admins` 每项只使用 `admin_account + family_name + given_name`，授权只比较账户。
- [x] 普通机构管理员少于2人时拒绝。
- [x] 没有任何岗位任职的管理员仍然拥有机构管理员签名权限。
- [x] 岗位新增或清空任职不会改变管理员集合。
- [x] 每个运行期及创世机构都有且只有一个 `LR / 法定代表人` 岗位。
- [x] `LR` 岗位允许空缺，首次创建不伪造法定代表人。
- [x] 最小首次创建call不再携带法定代表人、账户数组、完整岗位/任职或手工阈值。
- [x] runtime 自动创建完整强制协议账户集合，初始余额为零。
- [x] 注册局管理员只签名，0.1元费用只从注册局费用账户扣除。
- [x] `SFAS` 支持盈利和非盈利两类CID，未选择时拒绝。
- [x] Node、OnChina、CitizenApp、CitizenWallet按新协议编译和测试通过。
- [x] 第1步完成组件级真实编译、单测和前端构建；真实本地链、PostgreSQL、页面与二维码全链路验收按既定三步范围统一在第3步执行，不在本步伪报。
- [x] 文档已更新、中文注释已完善、旧代码和旧口径已清理。

## 第2步验收标准

- [x] 公权/私权机构统一新增 `propose_institution_governance`，本机构管理员可发起内部治理提案。
- [x] 公权/私权机构统一新增 `register_institution_admins`，注册局管理员可按注册局权限直接完整替换目标机构 `admins`。
- [x] 机构管理员集合变更使用内部投票引擎管理员变更互斥通道，不建立第二套管理员真源。
- [x] 岗位、任职和法定代表人治理通过 `InstitutionGovernanceAction` 原子表达；岗位任职来源只能是 `InstitutionGovernance`，不能伪装成普选、互选或任命结果。
- [x] 注册局直接登记管理员只替换 `admins`，不反向写岗位任职。
- [x] 新增 call 纳入 runtime 机构操作费用路由，0.1 元只从 `actor_cid_number` 费用账户扣除，管理员钱包只签名，不允许回落。
- [x] Node、OnChina、CitizenApp、CitizenWallet 已同步新来源枚举、call index、QR 动作码和解码规则。
- [x] 第2步完成链端、协议、扫码解码和移动端组件级验收；OnChina 页面入口、短随机岗位码生成和真实链/数据库/页面/二维码全链路验收在第3步执行，不伪造运行态结论。
- [x] 文档已更新、中文注释已完善、旧协议口径已清理。

## 第3步验收标准

- [x] OnChina 后端新增本机构治理 prepare 入口，构造 `propose_institution_governance` 链签名请求。
- [x] OnChina 后端新增注册局直接登记机构管理员 prepare 入口，构造 `register_institution_admins` 链签名请求。
- [x] 统一链签 submit 已支持机构治理 purpose；交易成功后只记审计，不本地改 `admins`、岗位或任职投影。
- [x] 公权/注册局详情页新增“机构治理”tab，支持管理员集合、岗位、任职、法定代表人任命/更换/解除和注册局直接登记管理员。
- [x] 私权详情页新增“机构治理”tab，支持本机构内部治理和法定代表人任命/更换/解除。
- [x] 普通岗位码由页面自动生成短随机码，链上继续按 `(cid_number, role_code)` 最终校验唯一。
- [x] 组件级验收通过：OnChina 后端 `cargo check`、OnChina 编码器测试、OnChina 前端生产构建和 diff 空白检查均通过。
- [x] 正式创世前第3步：OnChina 登录态、链读/链写、PostgreSQL 和前端全部统一为 `admin_account + family_name + given_name`，旧合并姓名字段只保留在删除旧列的清理 SQL 中。
- [ ] 真实运行态验收：当前源码 WASM fresh 链、临时 PostgreSQL、OnChina HTTP 页面和链投影已通过；交互式 CitizenWallet 扫码签名仍需真实管理员登录会话与扫码设备，本线程未伪造私钥或会话，未标记全链路完成。
- [x] 法定代表人“解除为空”：runtime `InstitutionLegalRepresentativeChange::Clear` 已能原子清空三字段，OnChina 与 CitizenWallet 已同步。

## 强制约束

- 不建立第二套管理员授权真源。
- 不按管理员姓名鉴权。
- 不把岗位名称当业务权限标识。
- 不从 `admins[0]` 推导法定代表人。
- 不保留旧call、旧SCALE布局、旧二维码解码或旧数据库写入流程。
- 不在链确认前写入OnChina正式机构投影。
- 机构和个人多签管理员使用同一个 `Admin` 三字段结构；个人多签仍保持独立业务和 storage，不与机构岗位任职混用。
- 不推送GitHub、不部署、不重新创世，除非用户另行授权。

## 输出物

- runtime、Node、OnChina、CitizenApp、CitizenWallet代码
- 中文注释
- 单元、集成和真实运行态测试
- `memory`协议与模块文档更新
- 旧载荷、旧字段、旧注释、旧文案和旧测试残留清理

## 执行记录

- 2026-07-17：用户确认第1步、新任务卡创建及指定runtime路径二次修改权限。
- 2026-07-17：runtime 管理员人员记录与岗位完成解耦；首次登记自动建立空缺 `LR / 法定代表人` 岗位、严格多数阈值和零余额强制协议账户。
- 2026-07-17：公权/私权创建 call、OnChina 生成端、CitizenWallet 解码端统一为最小载荷；旧法定代表人、账户数组、岗位任职、阈值和注资字段已删除，不保留兼容分支。
- 2026-07-17：OnChina 删除机构创建链确认前业务草稿区；创建机构/创建公民只允许 `chain_sign_sessions` 承载短期签名会话，且会话不参与 CID/名称占用，submit 成功或失败后删除。链上确认成功后才写 `subjects/accounts/institution_admins` / `citizens` 正式投影。
- 2026-07-17：协会 `SFAS` 的规则值改为 `p1=None`，明确表示实例必须显式选择盈利属性；删除模块内固定非盈利残留。
- 2026-07-17：验收通过：runtime 43 项、public/private admins 13 项、public/private manage 26 项、Node Guard 9 项、OnChina 3 项目标测试、CitizenApp 10 项目标测试、CitizenWallet payload decoder 87 项测试；两个 Flutter analyze、OnChina/Node cargo check、前端生产构建和格式检查均通过。
- 2026-07-17：本线程未连接 app terminal，且本机未发现 9944/9933/8964/5173/5432 监听服务；按第3步范围保留真实链、数据库、页面和二维码全链路验收，不伪造运行态结论。
- 2026-07-17：用户确认第2步并二次确认允许修改 `citizenchain/runtime/`。
- 2026-07-17：runtime 新增 `InstitutionGovernanceAction/Proposal`、机构治理签名域、PublicManage/PrivateManage call 8/9、内部投票管理员变更互斥通道和机构操作费用路由；本机构治理与注册局直接登记管理员均只认 `actor_cid_number + admins + origin`。
- 2026-07-17：Node、OnChina、CitizenApp、CitizenWallet 同步 `InstitutionGovernance` 来源、call index、QR 动作码和冷钱包解码；旧“机构管理员任职无独立扫码 call”的文档口径已清理。
- 2026-07-17：第2步验收通过：`cargo check -p entity-primitives -p internal-vote -p public-manage -p private-manage -p citizenchain -p node -p onchina`、`cargo test -p citizenchain --lib`、`cargo test -p onchina core::institution_call`、CitizenWallet `pallet_registry_test.dart + payload_decoder_test.dart`、CitizenApp 机构 storage codec 测试、两个 Flutter analyze 均通过。
- 2026-07-17：用户确认第3步。
- 2026-07-17：OnChina 后端新增机构治理 prepare、注册局直接登记管理员 prepare，并扩展统一链签会话 submit 的机构治理 purpose；机构治理交易成功后只审计，不本地改正式投影。
- 2026-07-17：OnChina 公权/注册局详情页和私权详情页新增“机构治理”tab；普通岗位码页面自动生成短随机码，管理员集合/岗位/任职/法定代表人任命、更换或解除均构造链上治理交易。
- 2026-07-17：runtime 法定代表人治理补齐 `Set/Clear`，解除时 `InstitutionInfo.legal_representative_name/cid_number/account` 三字段原子清空；OnChina API 新增 `clear_legal_representative`，CitizenWallet 已同步解码。
- 2026-07-17：第3步组件级验收通过：`cargo check -p onchina`、`cargo fmt -p onchina -- --check`、`cargo test -p onchina core::institution_call`、OnChina 前端 `npm run build`、`git diff --check`。
- 2026-07-17：真实运行态补验：`WASM_BUILD_FROM_SOURCE=1 cargo build -p node --bin citizenchain` 通过；当前源码 `citizenchain-fresh --tmp` 在 RPC `127.0.0.1:19944` 启动成功，`system_health.isSyncing=false`，genesis `0x17280b79d2136bb45813890a6effbb2c9b78ea46b6f77e05226e6de1140d3b63`，metadata hex 长度 `416058`。OnChina 使用临时内嵌 PostgreSQL `127.0.0.1:15433` 和 HTTP `127.0.0.1:18964` 启动成功，链投影 `subjects=49,593`、`accounts=99,231`，首页 `/` 返回 200，旧 `legal_rep_*` 列数量为 0，新 `legal_representative_*` 三字段列齐备且当前投影非空值为 0；验收后 OnChina、内嵌 PG 和 fresh 节点均已停止。
- 2026-07-17：修复 OnChina 新增机构第 1 步“请求内容不正确”：删除 `INSTITUTION_CREATE` prepare 阶段残留 `threshold` 校验；公权、教育、私权三个前端入口统一复用 `buildInstitutionCreatePayload`，扫码授权 payload 与正式提交 body 完全一致；授权始终只认管理员账户。
- 2026-07-17：修复 OnChina 新增机构正式提交“创建机构失败”：原因是前端只提交 `x-cid-security-grant`，未提交后端 `PASSKEY_COLD_SIGN` 安全门要求的 `X-Passkey-Assertion`。统一新增 `createColdSignSubmitHeaders/securityGrantSubmitHeaders` 正式提交入口，创建机构、创建/删除账户、公民身份上链、机构资料上传/删除、机构详情更新均改为同时携带冷签 grant 与 Passkey assertion；资料和详情更新的授权 payload 已按后端 `grant_payload` 逐字段同形清理，删除业务模块手写半套安全头残留。
- 2026-07-17：修复 OnChina 新增机构“双钱包签名”模型错误：创建机构不再生成 `INSTITUTION_CREATE` 安全动作、不再使用 `a=8 institution_create_credential`、不再携带 `register_nonce/signature/credential_signer_pubkey/scope_*` 内层凭证；后端只生成最终链交易签名会话，管理员钱包签一次后直接提交统一链交易 submit。
- 2026-07-18：正式创世前最终协议取代旧迁移方案：删除 `public-admins` / `private-admins` 历史存储翻译，不保留纯账户、单姓名或双轨布局。
- 2026-07-18：正式创世前统一改造第1步完成。`admin-primitives::Admin` 字段顺序固定为 `admin_account + family_name + given_name`；公权、私权、个人多签和创世固定管理员全部保存同一结构，权限判断只比较账户，缺失姓名在签名/投票/存储前分别规范化为“管理”“员”。
- 2026-07-18：第1步验证通过：runtime 及 9 个相关 crate 生产编译、`runtime-benchmarks` 编译、127 项专项单测、格式化与 diff 检查全部通过；runtime 中旧合并姓名字段、旧管理员类型和历史迁移代码残留为 0。
- 2026-07-18：正式创世前统一改造第2步完成。Node 共享 SCALE 解码、治理 DTO、链下清算读取、管理员激活、投票状态页和 NodeGuard 全部统一为 `admin_account + family_name + given_name`；管理员允许没有岗位，姓名只展示，授权和治理组成只比较账户。
- 2026-07-18：第2步验证通过：`cargo check -p node`、Node 280 项测试、Node 前端 TypeScript 与 Vite 生产构建、旧管理员结构 / 纯账户布局 / 合并姓名字段 / DTO 泛化账户字段残留检查全部通过。当前源码强制重建 WASM 后，隔离 `citizenchain-fresh` 真实启动成功，block#0 `0xc1dc759689aed0a8f8361dc3cb0e39c1faf19cfc55c7611b02ccc79ce04524c6`、`stateRoot=0x967155d28abe492052ef4bfd59a1ddbebce8cdaa57d9baaad446028848061a5e`、`isSyncing=false`；节点已停止并清理临时数据，本步未烘焙正式 chainspec 或切换正式数据。
- 2026-07-19：正式创世前统一改造第3步完成。OnChina 链上管理员严格解码、登录候选与会话、注册局目录、机构创建/治理载荷、机构管理员投影、PostgreSQL 和现有 UI 页面全部统一为 `admin_account + family_name + given_name`；授权只比较账户，页面只在展示时合并姓、名，没有岗位的管理员仍保留人员行。
- 2026-07-19：第3步验证通过：OnChina 137 项测试、后端生产编译、前端 TypeScript/Vite 生产构建和旧字段残留检查通过。隔离 fresh 节点 block#0 为 `0xc1dc759689aed0a8f8361dc3cb0e39c1faf19cfc55c7611b02ccc79ce04524c6`、`stateRoot=0x967155d28abe492052ef4bfd59a1ddbebce8cdaa57d9baaad446028848061a5e`、`isSyncing=false`；临时 PostgreSQL 实测旧合并姓名列为 0，并验证旧单列重启后被直接删除、默认落为“管理”“员”。链投影 49,593 个机构、99,231 个账户，健康接口和首页正常。所有验收进程已停止，临时数据已移入废纸篓；未烘焙正式 chainspec、未切换正式节点数据。
- 2026-07-19：正式创世前统一改造第4步完成。CitizenApp 机构/个人多签管理员模型、严格 SCALE 解码、创建/更换 call、本地快照、钱包匹配与现有页面全部统一为 `admin_account + family_name + given_name`；授权只比较账户，无岗位管理员仍保留，姓、名只在 UI 合并显示，创建/更换仍只有一次最终交易签名。
- 2026-07-19：第4步验证通过：`flutter analyze --no-fatal-infos` 无问题，41 项专项测试和 741 项完整测试通过，5 项因纯 Dart 环境缺少原生 smoldot 库按既有条件跳过。隔离 fresh 节点运行正常，block#0/stateRoot 与第2、3步一致；真实 RPC 抽样的 `PublicAdmins::AdminAccounts` 解出 9 个完整三字段管理员且无尾部字节。Android arm64 APK 在 Pixel 8a 安装启动，并在现有 Android 模拟器完成真实页面渲染；验收节点、模拟器和临时截图已清理。本步未修改 runtime、未烘焙正式 chainspec、未切换正式数据。
- 2026-07-17：OnChina、CitizenWallet、CitizenApp 已同步删除 `institution_create_credential` 动作码；CitizenWallet 对 `0x1e05/0x1f05` 按新 call-data 顺序解码并统一中文展示，创建机构链路不再存在内层凭证 Option 分支。
- 2026-07-17：本轮验收通过：`cargo check -p citizenchain`、`cargo check -p onchina`、`cargo test -p public-manage -p private-manage`、`cargo test -p onchina core::institution_call`、OnChina 前端 `npm run build`、CitizenWallet `flutter analyze`、CitizenWallet `flutter test test/signer/payload_decoder_test.dart test/signer/field_labels_test.dart`、CitizenApp `flutter test test/qr/qr_router_test.dart`、CitizenApp `flutter analyze`、`git diff --check`。OnChina test 构建仍有既有 `GENESIS_CITIZEN_MAX` 未用常量警告，与本次创建机构签名链路无关。
