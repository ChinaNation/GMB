# 任务卡：机构管理员、岗位和最小首次登记三步改造

## 当前状态

- 状态：已完成，2026-07-26 正式创世与无资金回归验收收口，已归档至 `done/`
- 当前步骤：第8.3C-2无真实资金完整测试和第8.3D正式创世收口均已完成；正式钱包
  `Rhett`、固定 RPC、Cloudflare 链时钟、CitizenConsole 发币配置以及充值/平台订阅/
  创作者订阅自动化测试均已验收，真实购买按用户最终要求不执行
- 用户确认：2026-07-17
- 执行规则：每一步先确认方案；执行完成后立即更新文档、完善中文注释、清理残留，再输出下一步技术方案

## 任务需求

机构唯一主键继续使用 `cid_number`，但管理员和岗位必须彻底分离：

- 管理员是人，机构和个人多签统一使用字段名 `admins`；公权、私权机构和个人多签项统一为 `Admin { account_id, cid_number, family_name, given_name }`。
- `admin_account` 是钱包账户；机构业务授权还必须同时满足机构 CID、岗位码、有效任职和岗位业务权限，管理员账户本身没有业务权限。
- 公权管理员的公民 CID、姓、名当前允许为空；非空 CID 必须由 `citizen-identity` 唯一真源确认 CID↔钱包绑定。私权管理员姓名保持非空展示字段。
- 机构管理员集合至少一人；机构治理阈值由 entity 独立保存，不得按管理员人数、岗位数或席位数推导。
- 岗位是机构职位，不是管理员；管理员可无岗位，岗位可空缺。
- 每个机构必须默认且唯一存在 `LR / 法定代表人` 岗位；该岗位不可删除、停用、改名或改码。
- 首次创建不自动把管理员任命为法定代表人，`legal_representative` 整体保持 `None`。
- 岗位任职不能再反向派生或覆盖 `admins`。
- 旧首次机构登记入口已关闭；后续恢复登记必须另立方案，机构治理阈值须作为 entity 配置独立确定。
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
- runtime 自动派生机构码和全部强制协议账户；机构阈值必须在 entity 独立确定，不由人数推导。
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

- [x] `admins` 统一四字段 SCALE 布局；账户本身不直接拥有机构业务权限。
- [x] 机构管理员集合至少一人；机构治理阈值与管理员人数独立。
- [x] 没有任何岗位任职的管理员仍保留人员记录，但没有机构业务权限。
- [x] 岗位新增或清空任职不会改变管理员集合。
- [x] 每个运行期及创世机构都有且只有一个 `LR / 法定代表人` 岗位。
- [x] `LR` 岗位允许空缺，首次创建不伪造法定代表人。
- [x] 旧最小首次创建 call 已关闭，不作为当前机构权限重构入口。
- [x] 既有创世机构继续由创世 seeder 写入身份、岗位、任职、阈值和协议账户。
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
- 2026-07-17：runtime 管理员人员记录与岗位完成解耦；当时首次登记自动推导严格多数阈值的结论已被 2026-07-20 entity 独立机构阈值取代。
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
- 2026-07-17：runtime 法定代表人治理补齐 `Set/Clear`；当前终态进一步收口为 `InstitutionInfo.legal_representative: Option<{ family_name, given_name, cid_number, account }>`，解除时整个结构原子清空；OnChina API 与 CitizenWallet 已同步。
- 2026-07-17：第3步组件级验收通过：`cargo check -p onchina`、`cargo fmt -p onchina -- --check`、`cargo test -p onchina core::institution_call`、OnChina 前端 `npm run build`、`git diff --check`。
- 2026-07-17：真实运行态补验：`WASM_BUILD_FROM_SOURCE=1 cargo build -p node --bin citizenchain` 通过；当前源码 `citizenchain-fresh --tmp` 在 RPC `127.0.0.1:19944` 启动成功，`system_health.isSyncing=false`，genesis `0x17280b79d2136bb45813890a6effbb2c9b78ea46b6f77e05226e6de1140d3b63`，metadata hex 长度 `416058`。OnChina 使用临时内嵌 PostgreSQL `127.0.0.1:15433` 和 HTTP `127.0.0.1:18964` 启动成功，链投影 `subjects=49,593`、`accounts=99,231`，首页 `/` 返回 200，旧 `legal_rep_*` 列数量为 0，新 `legal_representative_*` 三字段列齐备且当前投影非空值为 0；验收后 OnChina、内嵌 PG 和 fresh 节点均已停止。
- 2026-07-17：修复 OnChina 新增机构第 1 步“请求内容不正确”：删除 `INSTITUTION_CREATE` prepare 阶段残留 `threshold` 校验；公权、教育、私权三个前端入口统一复用 `buildInstitutionCreatePayload`，扫码授权 payload 与正式提交 body 完全一致；授权始终只认管理员账户。
- 2026-07-17：修复 OnChina 新增机构正式提交“创建机构失败”：原因是前端只提交 `x-cid-security-grant`，未提交后端 `PASSKEY_COLD_SIGN` 安全门要求的 `X-Passkey-Assertion`。统一新增 `createColdSignSubmitHeaders/securityGrantSubmitHeaders` 正式提交入口，创建机构、创建/删除账户、公民身份上链、机构资料上传/删除、机构详情更新均改为同时携带冷签 grant 与 Passkey assertion；资料和详情更新的授权 payload 已按后端 `grant_payload` 逐字段同形清理，删除业务模块手写半套安全头残留。
- 2026-07-17：修复 OnChina 新增机构“双钱包签名”模型错误：创建机构不再生成 `INSTITUTION_CREATE` 安全动作、不再使用 `a=8 institution_create_credential`、不再携带 `register_nonce/signature/credential_signer_pubkey/scope_*` 内层凭证；后端只生成最终链交易签名会话，管理员钱包签一次后直接提交统一链交易 submit。
- 2026-07-18：正式创世前最终协议取代旧迁移方案：删除 `public-admins` / `private-admins` 历史存储翻译，不保留纯账户、单姓名或双轨布局。
- 后续最终状态：公权、私权机构和个人多签全部统一为四字段 `Admin`，旧三字段和 `PublicAdmin` 类型均已删除。
- 2026-07-18：第1步验证通过：runtime 及 9 个相关 crate 生产编译、`runtime-benchmarks` 编译、127 项专项单测、格式化与 diff 检查全部通过；runtime 中旧合并姓名字段、旧管理员类型和历史迁移代码残留为 0。
- 后续最终状态：Node 按统一四字段布局验收；管理员允许没有岗位，但只有有效岗位任职和岗位业务权限才能构成机构授权。
- 2026-07-18：第2步验证通过：`cargo check -p node`、Node 280 项测试、Node 前端 TypeScript 与 Vite 生产构建、旧管理员结构 / 纯账户布局 / 合并姓名字段 / DTO 泛化账户字段残留检查全部通过。当前源码强制重建 WASM 后，隔离 `citizenchain-fresh` 真实启动成功，block#0 `0xc1dc759689aed0a8f8361dc3cb0e39c1faf19cfc55c7611b02ccc79ce04524c6`、`stateRoot=0x967155d28abe492052ef4bfd59a1ddbebce8cdaa57d9baaad446028848061a5e`、`isSyncing=false`；节点已停止并清理临时数据，本步未烘焙正式 chainspec 或切换正式数据。
- 后续最终状态：OnChina 对公权、私权统一使用四字段 `Admin`，业务授权按完整岗位主体校验。
- 2026-07-19：第3步验证通过：OnChina 137 项测试、后端生产编译、前端 TypeScript/Vite 生产构建和旧字段残留检查通过。隔离 fresh 节点 block#0 为 `0xc1dc759689aed0a8f8361dc3cb0e39c1faf19cfc55c7611b02ccc79ce04524c6`、`stateRoot=0x967155d28abe492052ef4bfd59a1ddbebce8cdaa57d9baaad446028848061a5e`、`isSyncing=false`；临时 PostgreSQL 实测旧合并姓名列为 0，并验证旧单列重启后被直接删除、默认落为“管理”“员”。链投影 49,593 个机构、99,231 个账户，健康接口和首页正常。所有验收进程已停止，临时数据已移入废纸篓；未烘焙正式 chainspec、未切换正式节点数据。
- 后续最终状态：CitizenApp 对公权、私权和个人多签统一使用四字段解码，空身份资料不再伪造展示值。
- 2026-07-19：第4步的旧公权三字段 fresh 验收只保留为历史记录，不再代表当前存储布局；当前 fresh 链必须以四字段 `PublicAdmin` 重新验收。
- 后续最终状态：CitizenWallet 按统一四字段 `Admin` 严格解码，拒绝旧三字段。
- 2026-07-20：CitizenWallet 已按实际 runtime call 布局分流解码公权/私权机构治理和注册局登记；公权身份字段允许暂空，非空 CID 只接受 CTZN 结构，旧内层凭证字段不再属于这两个 call。
- 2026-07-19：CitizenWallet 同一已扫描请求只允许一次密钥调用：签名进行中或已生成响应二维码时拒绝重复触发；同一次业务操作不叠加姓名确认签名或其它第二签名。统一 action registry 已重新生成 CitizenApp/CitizenWallet 两端产物，个人多签创建的必显字段补齐 `admins`。
- 2026-07-19：第5步验证通过：`qr-protocol` 6 项一致性/守卫测试、CitizenWallet `flutter analyze`、179 项全量测试、Android arm64 debug 构建、CitizenApp `flutter analyze` 及 QR/签名 53 项测试全部通过。Pixel 8a 真机安装后 `org.citizenwallet/.MainActivity` 真实启动、进程存活并渲染“还没有钱包”首屏；设备无钱包，未创建测试私钥、未伪造扫码签名或链交易。Android 16 同时报告现有 Flutter/插件原生库未满足 16 KB 页面对齐，关闭系统提示后应用正常渲染；该独立发布风险留待正式创世前阻塞审计处理。
- 2026-07-19：第5步旧字段、旧载荷和旧文档口径已清理；全仓非 runtime 仅保留 OnChina 启动时 `DROP COLUMN IF EXISTS` 删除旧数据库合并姓名列的清理 SQL，不存在兼容读取。本步未修改 runtime、未烘焙 chainspec、未切换节点数据、未部署或推送。
- 2026-07-20：私权创世机构最终重新定义为 SFGY 非营利法人“公民链技术发展基金会”：CID `GZ018-SFGYR-201206100-2026`，主/费用账户按最终 CID 权威派生；法定代表人程伟引用公民 CID `GZ000-CTZN6-198805200-2026`，不伪造第二份公民记录。block#0 在 `PrivateManage/PrivateAdmins` 只写一名程伟管理员，同一钱包分别任职 `LR / GENESIS_PRODUCT_MANAGER / GENESIS_PROGRAMMER` 三岗，每岗一席，机构阈值保持 2。
- 2026-07-19：公民链基金会身份、协议账户和治理骨架纳入 NodeGuard，总保护数为 90（89 公权 + 1 私权）；固定岗位不能增删、改名或停用，法定代表人、创世产品经理和创世程序员各固定一席。任职账户必须属于 `admins`，同一账户允许跨岗位兼任；基金会当前由一名私权 `Admin` 同时担任三个岗位，法定代表人原子结构与 `LR` 任职账户一致。依法换人时，岗位任职和法定代表人结构必须原子同步；新任人员尚不在 `admins` 时才同时更新名册。
- 2026-07-19：第6步验证通过：primitives 72 项、private-admins/private-manage 17 项、runtime 45 项、Node 281 项、OnChina 137 项测试全部通过；生产/no-default-features 编译、WASM 强制重建、格式和残留检查通过。当前源码以隔离 `citizenchain-fresh --tmp` 真实启动，节点守卫启动自检通过，block#0 `0x1732f0f1005d7e8ee7f9292e35a036698ece48569f6aca8e01f56f264761083d`、`stateRoot=0x37379726b7245af3123618fe032fa5355e17ec847159703daf6a9b5322a04fd3`、`isSyncing=false`。本步未烘焙正式 chainspec、未更新 CitizenApp/Cloudflare 正式资产、未切换节点数据、未部署或推送。
- 2026-07-17：OnChina、CitizenWallet、CitizenApp 已同步删除 `institution_create_credential` 动作码；CitizenWallet 对 `0x1e05/0x1f05` 按新 call-data 顺序解码并统一中文展示，创建机构链路不再存在内层凭证 Option 分支。
- 2026-07-17：本轮验收通过：`cargo check -p citizenchain`、`cargo check -p onchina`、`cargo test -p public-manage -p private-manage`、`cargo test -p onchina core::institution_call`、OnChina 前端 `npm run build`、CitizenWallet `flutter analyze`、CitizenWallet `flutter test test/signer/payload_decoder_test.dart test/signer/field_labels_test.dart`、CitizenApp `flutter test test/qr/qr_router_test.dart`、CitizenApp `flutter analyze`、`git diff --check`。
- 2026-07-25：第7步以当前源码强制构建 WASM 并导出隔离
  `citizenchain-fresh`，block#0 为
  `0xa75336f2847a159012127590f0a90974f9d12b4b057bf34ad133c237ee680177`，
  state root 为
  `0xcbd2f72f289f235506da0bbe96d2e697f9fedfbf3c6dea993604c95d899d52d3`。
  创世宪法脚本、17 项创世专项测试以及 NodeGuard
  `real_genesis_complete_state_passes_all_policies_in_one_scan` 全部通过；普通测试构建中因
  无内嵌 WASM 而显式跳过的 chainspec 用例未冒充通过。
- 2026-07-25：真实 RPC 核对确认私权创世机构唯一目标态为
  `GZ018-SFGYR-201206100-2026 / 公民链技术发展基金会`。机构使用一名管理员程伟，
  `account_id=0x9c3e18f575c59236832054469ef0e69f16a1fe6c50b2b580fc7c71853ab71068`，
  公民 CID 为 `GZ000-CTZN6-198805200-2026`；同一账户分别担任
  `LR / GENESIS_PRODUCT_MANAGER / GENESIS_PROGRAMMER`，三个岗位均启用且各一席，
  治理阈值为 2。机构主账户
  `0xaa23304c7b663ba25a9d3a2fb1efafdd650ecf2504a2caedc228fe81b46b4333`
  与费用账户
  `0xe4837d50cd5ef677c9b10ea49baf8c7d60cf11b422d748377e9e5f750640e927`
  和源码常量一致。
- 2026-07-25：两套独立 fresh 数据库已成功启动并通过加密 P2P 地址互联，双方均读取同一
  block#0，peer 角色分别为 `AUTHORITY` 与 `FULL`。当前链禁止空交易池出块，而临时
  Alice/Bob 矿工账户创世余额为 0；本机也不存在 44 名正式 GRANDPA 权威的任一私钥。
  因此本轮没有篡改创世余额、替换正式权威或复制生产私钥，非创世区块导入和
  finalized 持续推进保持未验收，不标记为通过。
- 2026-07-25：`OnchainTransaction::transfer_with_remark` 元数据入口存在，call index
  为 `4,0`；runtime 费用路由、按交易金额收费、精确付款账户及分账专项测试均通过。
  基金会费用账户
  `0xe4837d50cd5ef677c9b10ea49baf8c7d60cf11b422d748377e9e5f750640e927`
  在 fresh 创世余额为 0，而基金会机构操作只允许从该费用账户扣费；正式链启用机构操作前
  必须由用户确认合法注资来源、金额和执行时点。
  CitizenConsole 当前发币账户
  `0x36d00d0a9701b6e860c51476ce2d7ac5f3b35b6ff067b81d958afa1b0551c303`
  在 fresh 创世余额为 0。正式冻结前必须由用户确认该账户的合法资金来源和金额；未经确认
  不得擅自增加创世分配或假设创世后人工转账。
- 2026-07-25：本步未修改 runtime、未覆盖正式 chainspec、未切换正式节点数据、未触碰
  GitHub/CI、未部署。隔离节点和临时数据在验收后清理。
- 2026-07-25：正式创世前本地总验收再次通过当前冻结资产同源脚本、`git diff --check`、
  runtime 51 项测试、NodeGuard 84 项测试、Node 全目标 clippy、OnChina 144 项测试、
  QR 协议 8 项守卫、CitizenApp 806 项测试、CitizenWallet 192 项测试、Worker 174 项测试
  和 CitizenConsole 27 项资金安全测试。Node 与 OnChina 前端生产构建均通过。
- 2026-07-25：用户最终确认仓库当前订阅逻辑正确，禁止在本轮创世验收中修改。此前把
  `Suspended`、`NeedReconsent`、`InsufficientBalance` 或 `CreatorPaused` 判为正式创世
  阻塞的结论已撤回；本轮未修改 runtime 或任一订阅实现。
- 2026-07-25：Node 前端把 DOMPurify 更新到 `3.4.12`、Vite 更新到 `6.4.3`，OnChina
  前端把 Vite 更新到 `6.4.3`，并更新兼容范围内的 Babel、PostCSS、Picomatch、Rollup 等
  间接依赖。两端 `npm audit --audit-level=low` 均为 0，生产构建通过；仍保留约
  889 KB 和 1.539 MB 主包体积性能告警，不属于依赖安全告警。
- 2026-07-25：AI 门禁已改用兼容 macOS BSD grep/GNU grep 的新增行匹配，并按精确路径
  排除固定 WalletConnect bundle 与 Wrangler 生成类型的源码残留/中文注释扫描。用户完成
  runtime 二次确认后，10 个既有大改文件均补充了关键中文注释；权重文件只增加生成说明，
  未修改权重数值、读写次数或执行逻辑。门禁比较口径同时由“基线到 HEAD”改为“基线到当前
  工作树”，保证本地验收覆盖本轮未提交修改，CI 干净工作树结果不变；最终复跑通过。
- 2026-07-25：Cloudflare Worker 已删除手写 `@cloudflare/workers-types`，改由
  `wrangler types --env-interface CloudflareBindings` 生成并跟踪
  `worker-configuration.d.ts`；类型漂移检查、typecheck、29 个测试文件 174 项测试和
  依赖审计全部通过。production `TOPUP_DISBURSE_ACCOUNT_ID` 已写入 CitizenConsole
  现有发币账户的规范 AccountId。
- 2026-07-25：OnChina 生产二进制及全部测试目标自身的严格
  `cargo clippy -p onchina --all-targets --no-deps -- -D warnings` 已通过，144 项测试通过。
  测试中的断言式 `expect/unwrap` 豁免只放在各自 `#[cfg(test)]` 模块；生产代码没有使用
  crate 级静音。
- 2026-07-25：用户逐批完成 runtime 路径二次确认后，
  `cargo clippy --workspace --all-targets -- -D warnings` 已在整个 CitizenChain 工作区
  零告警通过，`cargo test --workspace --all-targets` 全部通过；范围覆盖 runtime、Node、
  OnChina、协议 crate、测试工具和固定 Substrate GRANDPA 上游源码。FRAME extrinsic
  既定多参数 ABI 与上游 vendor 只使用带中文原因的最窄 lint 范围，没有全局跳过目录。
  43 个省级 `stake_amount` 仅分组数字格式变化，逐项数值与 HEAD 完全一致。
  本轮未修改 `square-post` 订阅实现，订阅真实日历、自动续费、取消保留已付权益、
  创作者暂停恢复及价格重确认等既有测试全部通过。
- 2026-07-25：本轮最终复验通过 `git diff --check`、全仓严格 Clippy、全量测试和
  AI 守卫；已确认文件完成 rustfmt，未在本轮授权清单内的 runtime 文件没有因全仓格式化
  被改动。链上资产发行骨架中的旧开发标记已改为明确的“当前授权入口/后续任务卡执行边界”，
  没有伪装成业务已完成，也没有补写或改变资产发行逻辑。
- 2026-07-25：用户确认正式创世时基金会费用账户与 CitizenConsole 发币账户保持零余额；
  两账户到账且正式链最终性通过前，机构付费操作和稳定币充值业务保持关闭。2026-07-26
  用户进一步明确国储会节点不由本任务部署；本任务不得读取其 SSH 配置、连接或改动服务器。
- 2026-07-25：第8.1步把提交 `29e316e2b810fca40f4a7f7a7ebed49fd9626da1`
  推送到 `origin/main`；CitizenApp CI run `30189033908` 与 CitizenChain CI run
  `30189033913` 全部成功。因推送同时包含本地此前领先远端的提交，GitHub 额外自动触发了
  不在本步授权清单内的 CitizenWallet CI run `30189033915`，发现后立即取消，最终结论为
  `cancelled`。
- 2026-07-25：首次 CitizenChain WASM run `30189706702` 在 no_std 编译
  `genesis-pallet` 时失败：Clippy 清理把两处单元素 `Vec` 构造收口为 `vec![]`，但
  `institution/seeder.rs` 只导入 `alloc::vec::Vec`，没有导入 `alloc::vec` 宏。修复仅补充
  no_std 宏导入，不改变创世数据、账户、岗位、管理员、余额或订阅逻辑。本地
  `cargo check -p genesis-pallet --no-default-features`、
  `cargo clippy -p genesis-pallet --all-targets -- -D warnings` 与
  `WASM_BUILD_FROM_SOURCE=1 cargo build --release -p citizenchain` 均通过。
- 2026-07-25：最小修复提交
  `ac6de21b2432f52f45f1767f88f4e6833a2c79d0` 已推送到 `origin/main`，未触发
  CitizenApp、CitizenChain 桌面端或 CitizenWallet 自动 CI。第二次 CitizenChain WASM
  run `30190068925` 成功，artifact id `8628330093`、名称 `citizenchain-wasm`、
  GitHub 压缩包大小 `5,023,255` 字节，绑定 HEAD SHA 与修复提交完全一致。
  三份解包产物分别为：`citizenchain.wasm` `6,359,876` 字节、
  SHA-256 `7ef98cc86672fc5d64498f67a82b803144c8023a3fc5c57ef5e8b8f5ad300443`；
  `citizenchain.compact.wasm` `6,082,526` 字节、
  SHA-256 `2469753a161e4757fc770a70198a7d79ef01601e13ed7f7eed262e4cd28629f1`；
  `citizenchain.compact.compressed.wasm` `1,180,651` 字节、
  SHA-256 `a838dd763c1c7003aca1edf177738d85b64936bbc1ba98dda7da348cc57d0d1a`。
- 2026-07-25：第8.2步严格使用 WASM run `30190068925`、HEAD
  `ac6de21b2432f52f45f1767f88f4e6833a2c79d0` 的
  `citizenchain.compact.compressed.wasm` 执行正式 `--finalize`。第一次候选在覆盖任何正式
  资产前因公权机构生成器仍按三个旧独立 `Option` 解码法定代表人而失败；已把生成器收口为
  当前 runtime 的单个原子
  `Option<{ family_name, given_name, cid_number, account_id }>`，并补齐 `None/Some`
  两类 SCALE 自测。没有增加旧布局兼容分支。
- 2026-07-25：第二次从头烘焙成功，正式
  `genesis_hash=0xe8f4067de2323dc27b2a2c409fa4b3ab882e4e88dfa6f4a81355f51f8cf8eb45`、
  `state_root=0xbdc2593a538b7010717ac475b0b59973dd57c77d35683c4e7d9b8058b9ae18f9`、
  `chainspec_hash=3e79942fabad332fee5e8692b503c393005730bc5b2d85b9d38694833fada652`、
  `light_sync_state_hash=95beb873cce95ca1744193c0aa0c7023a4b4070346b8ba68758d7a140d8a61c0`、
  `public_institution_root=c21f99f5bd40bc3c9fcee9439de9f6902c98212b2510dd7440c9630284ab939f`。
  release 状态包 manifest 只包含 `chains/citizenchain/db`，公权机构包为 43 省、49,593
  个机构；节点、App、checkpoint、公权缓存和 Cloudflare 三环境链锚点交叉校验通过。
- 2026-07-25：独立验收确认链规范内嵌 WASM 为 `1,180,651` 字节、SHA-256
  `a838dd763c1c7003aca1edf177738d85b64936bbc1ba98dda7da348cc57d0d1a`；
  完整真实创世状态 NodeGuard 全量扫描与 ConstitutionGuard 创世测试均通过。基金会费用
  账户和 CitizenConsole 发币账户均不在 70 项创世余额表中，余额保持 0。Cloudflare
  三环境只改创世哈希和状态根，订阅实现无差异；本步未切换正式数据、未部署、未转入资金、
  未提交或推送。
- 2026-07-25：第8.3A步完成正式切换前只读盘点，没有删除本机、手机、Cloudflare 或
  服务器数据。CitizenConsole 当前“清空链数据”只停止开发节点并删除
  `~/Library/Application Support/gmb.dev/chains/citizenchain/db`，“清空 OnChina
  数据”只删除 `gmb.dev/onchina-pgdata`；两个动作都不覆盖生产 `gmb` 数据根和历史
  `org.chinanation.citizenchain.desktop` 数据根。当前本机同时存在约 88 MB 开发链、
  79 MB 生产链及 7 MB 历史链数据库，但三个数据根都没有创世 manifest，禁止在只读阶段
  猜测其链身份。开发/生产 OnChina PostgreSQL 目录当前都不存在。
- 2026-07-25：Pixel 8a 的 `org.citizenapp` 在主用户和私密空间均已安装，主用户应用
  正在运行且 Isar 业务库约 36.7 MB；FlutterSecureStorage 中存在热钱包 seed、恢复材料
  和通讯录密钥。2026-07-26 用户明确授权主用户和私密空间都执行 CitizenApp 全量清除，
  包括上述钱包材料与 Android Keystore 条目；清除后必须重新创建或恢复钱包。
- 2026-07-25：Cloudflare 双环境只读盘点确认两个 R2 媒体桶和两个聊天桶均为 0 对象，
  Durable Object 没有持久化存储；staging KV 为空，production KV 有 15 项会话/索引
  残留。staging D1 只有指向旧链 block #9 的 `chain_clock`，production D1 除同一旧链
  时钟外还有设备子钥、登录 challenge、通知已读、限流窗口和请求 nonce 残留；平台会员、
  创作者档位/订阅、充值订单、帖子、媒体、通讯录和聊天业务表均为 0。production
  bootstrap 仍返回旧创世
  `0x840d5b12c541a010783e54069c9168a13d102ba63cd8f3a00263440c1803aad9`；
  本步没有部署新 Worker 或改远端绑定。
- 2026-07-26：已查明此前只为 staging 生成并保存 `TOPUP_INTENT_SECRET`，production
  从未落入 Keychain。CitizenConsole 已增加 CSPRNG“生成并保存”入口；production 专用
  48 字节随机密钥已经一次 Touch ID 写入 macOS Keychain，值未回显、未写日志或仓库，
  等冻结资产软件 CI 成功后再同步到 production Worker。两个远端 Worker 的
  `STRIPE_API_KEY` 与 `STRIPE_HOOK_SECRET` 已经一次 Touch ID 精确删除并复核不存在，
  禁止恢复 Stripe 业务。
- 2026-07-25：当前六个 bootnode 中，`nrcgch`、`prczss`、`prcsds` 的公网 30333
  可连接，`prchbs`、`prches`、`prcsxs` 不可连接。该记录只作为冻结前网络盘点；
  2026-07-26 用户明确国储会节点及服务器部署全部移出本任务，本任务不再把 SSH 配置或
  服务器切换列为阻塞，也不得连接或改动这些服务器。
- 2026-07-26：节点发布合同统一为轻量安装包：WASM CI 成功后先用该产物冻结 plain
  chainspec、release 状态审计包和 CitizenApp/Cloudflare 锚点，再运行 CitizenApp CI 与
  CitizenChain 软件 CI。四平台安装包只携带节点软件和内嵌冻结 plain chainspec，首启本地
  物化同一块 0；258 MB release RocksDB 只作为创世审计制品保留，不重复进入安装包。
  `prepack.sh`、`prepack.ps1`、GitHub CI 和节点文档已统一，不保留双轨打包口径。
- 第8.3A步形成的待执行删除范围如下，当前仅登记、尚未执行：
  - 本机链数据库：精确删除 `~/Library/Application Support/gmb.dev/chains/citizenchain/db`、
    `~/Library/Application Support/gmb/chains/citizenchain/db` 和
    `~/Library/Application Support/org.chinanation.citizenchain.desktop/node-data/chains/citizenchain/db`；
    不递归删除三个上级数据根。
  - OnChina：切换瞬间再次检查并只删除存在的
    `~/Library/Application Support/gmb.dev/onchina-pgdata` 与
    `~/Library/Application Support/gmb/pgdata`；盘点时两者均不存在。
  - CitizenApp：主用户和私密空间均对 `org.citizenapp` 执行全量 package data 清除，
    同时清除私密空间历史 `org.chinanation.citizen`；包括 Isar、SharedPreferences、
    FlutterSecureStorage、Android Keystore 条目、钱包 seed、恢复材料、轻节点与 WebView
    缓存。CitizenWallet 不在本任务删除范围。
  - Cloudflare D1：staging/production 清空 25 张业务表中的全部行，保留表结构、
    `_cf_KV` 内部对象和 staging 的 `d1_migrations`；重建后只允许写入新正式链
    `chain_clock`。
  - Cloudflare KV/Queue：删除 production 的 15 项会话和账户索引，staging 当前无键；
    精确 purge 两套通知队列。R2 四桶当前为空，Durable Object 无持久数据，不执行无目标
    删除。
  - Cloudflare Secret：两环境废弃的 `STRIPE_API_KEY`、`STRIPE_HOOK_SECRET` 已删除；
    production `TOPUP_INTENT_SECRET` 已在 CitizenConsole Keychain 重新生成，待软件 CI
    成功后同步远端。
- 2026-07-26 用户取代第8.3A保留建议，明确授权删除本机、Cloudflare 和 Pixel 8a
  CitizenApp 全部旧业务数据。本机将整目录删除 `gmb.dev`、`gmb`、历史/当前桌面产品运行
  目录，包括旧 keystore、network secret、TLS 与审计数据；手机删除 CitizenApp 钱包材料。
  唯一保留项为仓库正式冻结资产、CitizenConsole macOS Keychain、发币钱包/部署凭据以及
  CitizenWallet 数据。
- 2026-07-26：第8.3B步冻结资产提交
  `a5204a39b90bf83daab8b91d83da6dd150269d9a` 已推送 `origin/main`。CitizenApp CI
  `30211805216` 与 CitizenChain CI `30211805231` 均成功；CitizenChain 的 Linux arm64、
  Linux amd64、macOS arm64 和 Windows 四个平台任务全部通过。WASM 没有在冻结后重新
  触发，继续使用已冻结的成功产物 `30190068925`。
- 2026-07-26：Cloudflare staging/production 已删除并按唯一 `0001_square_core.sql`
  基线重建 25 张业务表，KV、四个 R2 桶和两套通知队列已清空；两个废弃 Stripe Secret
  均不存在，两个环境都已配置 `TOPUP_INTENT_SECRET`，production 值由 CitizenConsole
  一次 Touch ID 生成并保存，执行过程没有回显 Secret。两套 Worker 已发布，production
  `/health` 返回 200，staging 继续由 Access 以 302 拦截匿名请求。
- 2026-07-26：第一次 Cloudflare 清理完成后、国储会节点正式切换前，Cron 曾从固定
  `CHAIN_URL` 重新写入旧链 block #9 的唯一 `chain_clock`。国储会部署完成后只读复核确认
  staging/production 的同一固定 RPC 已返回正式创世
  `0xe8f4067de2323dc27b2a2c409fa4b3ab882e4e88dfa6f4a81355f51f8cf8eb45`，
  finalized head 为 block #0。随后再次通过 CitizenConsole Touch ID 重建双环境 D1、
  清空 KV、四个 R2 桶和两套通知队列；旧 block #9 已彻底消失，25 张业务表、KV 和 R2
  当前全部为零。正式 block #0 没有 `Timestamp.Now`，因此 `chain_clock` 保持空表并令
  Cloudflare 权益门禁 fail-closed；首个带时间戳的 finalized 区块产生后 Cron 才能写入
  新正式链时钟。本任务没有修改订阅逻辑，也没有连接或操作国储会服务器。
- 2026-07-26：本机 `gmb.dev`、旧 `gmb`、历史/当前桌面产品运行目录和受保护容器均已
  彻底删除；macOS 受保护容器由 Finder 精确移出后单项永久清除，没有全局清空废纸篓。
  CI macOS 应用已安装至 `/Applications/citizenchain.app` 并从冻结 plain chainspec
  首启物化新 `gmb` 数据。真实 RPC 返回 block #0
  `0xe8f4067de2323dc27b2a2c409fa4b3ab882e4e88dfa6f4a81355f51f8cf8eb45`、
  state root `0xbdc2593a538b7010717ac475b0b59973dd57c77d35683c4e7d9b8058b9ae18f9`，
  finalized head 为 block #0、`isSyncing=false`。
- 2026-07-26：Pixel 8a 主用户和私密空间的 `org.citizenapp` 已全量清空，私密空间历史
  `org.chinanation.citizen` 包已删除；两个空间的 `org.citizenwallet` 均保留。当前提交
  源码重新构建的 `org.citizenapp` 1.0.0 已安装并在主用户真实启动，渲染全新“权限设置”
  首屏且无崩溃。APK 内 chainspec SHA-256 与仓库资产一致，公权机构 manifest 和
  light-sync checkpoint 均携带冻结正式创世哈希与状态根。
- 2026-07-26：正式链继续产出并 finalized 到 block #4，区块哈希
  `0x48e0fc5df7690ec1af702a1b07a9512b5aa5fb0e8589c7ce21a0cfbada119461`，
  同区块 `Timestamp.Now=1785090070105`。Cloudflare staging/production 的定时任务均已
  从同一正式链写入 block #3 链时钟，旧 block #9 没有复现，证明固定 Access + Tunnel
  RPC、正式链时间戳和边缘链时钟恢复正常。
- 2026-07-26：CitizenConsole 发币账户只读验收通过：规范 `account_id` 为
  `0x36d00d0a9701b6e860c51476ce2d7ac5f3b35b6ff067b81d958afa1b0551c303`，
  正式链可用余额为 `100000000` 分、nonce 为 0、reserved/frozen 均为 0；本次没有签名、
  发币或产生交易。
- 2026-07-26：交易验收进入 CitizenApp 时发现本机仍显示旧钱包和旧聊天资料，会员页因
  Cloudflare 已清空设备子钥而真实返回“设备子钥未注册”，页面把该错误显示成“请先创建
  热钱包”。这不是订阅状态机或扣款逻辑故障，而是手机旧数据没有实际清空造成的跨端旧态。
  已按用户既有授权再次对 Pixel 8a 的主用户与私密空间精确执行
  `pm clear org.citizenapp`，两处均返回成功；重启后真实回到权限页并进入“创建钱包 /
  导入助记词”首次启动页，旧钱包、聊天、会话和设备子钥材料均不再存在。
- 2026-07-26：公民币购买、平台订阅和创作者订阅尚未发送真实交易。继续验收必须先由用户
  在手机上创建并手抄备份新的真实助记词，或自行导入现有正式钱包；稳定币购买还会唤起
  外部钱包并支付真实 USDC/USDT，创作者订阅验收还需要链上已有创作者会员档。不得由验收
  脚本代生成、回显或保存助记词，也不得在用户未确认真实付款金额和目标会员档时擅自扣款。
  该历史门禁随后被用户“不进行真实购买，只完成测试”的最终要求取代。
- 2026-07-26：第8.3C-1新钱包只读基线确认当前 CitizenApp 正式钱包为 `Rhett`，
  `account_id=0xb805efd2399e6e6b08fc5527c07a963bb7fdee0181a348ce68b2d2dbaf235753`，
  SS58 为 `w5Fk5zp3WrLxDET9wUY6WJKtfmBuWTFfuQohmwzrp1efKPmid`；正式链余额
  `100000000` 分、nonce 0，production 设备子密钥和现有会话均已建立。该钱包是用户
  重新创建的正式钱包，后续清理和验收都禁止删除、重置、覆盖或卸载其应用数据。
- 2026-07-26：用户确认第8.3C-2只做无真实资金完整测试，不进行 USDC/USDT 真实购买。
  CitizenApp 静态检查通过且相关测试 64/64，Worker 174/174 与 TypeScript 检查通过，
  CitizenConsole 27/27，`square-post` 23/23，CitizenChain runtime SquarePost 集成
  4/4，总计 292 项全部通过。测试前后 `Rhett` 钱包余额、nonce、reserved/frozen 均不变；
  production 充值订单、平台订阅、创作者订阅和创作者档位仍全部为 0。正式链与
  `chain_clock` 从 #5 自然推进到 #6；没有账户签名、稳定币付款、发币、订阅、生产部署
  或数据库写入，也没有操作手机。
- 2026-07-26：第8.3D正式创世收口完成。节点 plain SSOT、创世状态包、CitizenApp
  chainspec/light-sync、公权机构分片和 Cloudflare 链身份一致性守卫通过；本机正式链
  best/finalized 均为 block #6、`isSyncing=false`。GitHub WASM/CitizenApp/CitizenChain
  CI 均为成功状态，正式创世唯一锚点已同步到架构、ADR、AI 真源和模块文档。未修改代码、
  runtime 或订阅逻辑，未写 Cloudflare，未操作手机或钱包，未产生交易。
