# 任务卡：机构管理员、岗位和最小首次登记三步改造

## 当前状态

- 状态：进行中
- 当前步骤：正式创世前统一改造第6步私权创世机构接入已完成；2026-07-20 该机构已按最终确认重新定义为“中国公民链技术发展基金会”，下一步为第7步正式创世阻塞审计
- 用户确认：2026-07-17
- 执行规则：每一步先确认方案；执行完成后立即更新文档、完善中文注释、清理残留，再输出下一步技术方案

## 任务需求

机构唯一主键继续使用 `cid_number`，但管理员和岗位必须彻底分离：

- 管理员是人，机构和个人多签统一使用字段名 `admins`；公权项为 `PublicAdmin { admin_account, cid_number, family_name, given_name }`，私权和个人多签项为 `Admin { admin_account, family_name, given_name }`。
- `admin_account` 是钱包账户；机构业务授权还必须同时满足机构 CID、岗位码、有效任职和岗位业务权限，管理员账户本身没有业务权限。
- 公权管理员的公民 CID、姓、名当前允许为空；非空 CID 必须由 `citizen-identity` 唯一真源确认 CID↔钱包绑定。私权管理员姓名保持非空展示字段。
- 机构管理员集合至少一人；机构治理阈值由 entity 独立保存，不得按管理员人数、岗位数或席位数推导。
- 岗位是机构职位，不是管理员；管理员可无岗位，岗位可空缺。
- 每个机构必须默认且唯一存在 `LR / 法定代表人` 岗位；该岗位不可删除、停用、改名或改码。
- 首次创建不自动把管理员任命为法定代表人，法定代表人三字段保持 `None`。
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

- [x] `admins` 按公权四字段、私权/个人三字段分流；账户本身不直接拥有机构业务权限。
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
- 2026-07-17：runtime 法定代表人治理补齐 `Set/Clear`，解除时 `InstitutionInfo.legal_representative_name/cid_number/account` 三字段原子清空；OnChina API 新增 `clear_legal_representative`，CitizenWallet 已同步解码。
- 2026-07-17：第3步组件级验收通过：`cargo check -p onchina`、`cargo fmt -p onchina -- --check`、`cargo test -p onchina core::institution_call`、OnChina 前端 `npm run build`、`git diff --check`。
- 2026-07-17：真实运行态补验：`WASM_BUILD_FROM_SOURCE=1 cargo build -p node --bin citizenchain` 通过；当前源码 `citizenchain-fresh --tmp` 在 RPC `127.0.0.1:19944` 启动成功，`system_health.isSyncing=false`，genesis `0x17280b79d2136bb45813890a6effbb2c9b78ea46b6f77e05226e6de1140d3b63`，metadata hex 长度 `416058`。OnChina 使用临时内嵌 PostgreSQL `127.0.0.1:15433` 和 HTTP `127.0.0.1:18964` 启动成功，链投影 `subjects=49,593`、`accounts=99,231`，首页 `/` 返回 200，旧 `legal_rep_*` 列数量为 0，新 `legal_representative_*` 三字段列齐备且当前投影非空值为 0；验收后 OnChina、内嵌 PG 和 fresh 节点均已停止。
- 2026-07-17：修复 OnChina 新增机构第 1 步“请求内容不正确”：删除 `INSTITUTION_CREATE` prepare 阶段残留 `threshold` 校验；公权、教育、私权三个前端入口统一复用 `buildInstitutionCreatePayload`，扫码授权 payload 与正式提交 body 完全一致；授权始终只认管理员账户。
- 2026-07-17：修复 OnChina 新增机构正式提交“创建机构失败”：原因是前端只提交 `x-cid-security-grant`，未提交后端 `PASSKEY_COLD_SIGN` 安全门要求的 `X-Passkey-Assertion`。统一新增 `createColdSignSubmitHeaders/securityGrantSubmitHeaders` 正式提交入口，创建机构、创建/删除账户、公民身份上链、机构资料上传/删除、机构详情更新均改为同时携带冷签 grant 与 Passkey assertion；资料和详情更新的授权 payload 已按后端 `grant_payload` 逐字段同形清理，删除业务模块手写半套安全头残留。
- 2026-07-17：修复 OnChina 新增机构“双钱包签名”模型错误：创建机构不再生成 `INSTITUTION_CREATE` 安全动作、不再使用 `a=8 institution_create_credential`、不再携带 `register_nonce/signature/credential_signer_pubkey/scope_*` 内层凭证；后端只生成最终链交易签名会话，管理员钱包签一次后直接提交统一链交易 submit。
- 2026-07-18：正式创世前最终协议取代旧迁移方案：删除 `public-admins` / `private-admins` 历史存储翻译，不保留纯账户、单姓名或双轨布局。
- 2026-07-18：当时完成三字段 `Admin` 统一；2026-07-20 公权部分已被 `PublicAdmin` 四字段目标彻底取代，私权、个人多签和私权创世基金会继续使用三字段 `Admin`。
- 2026-07-18：第1步验证通过：runtime 及 9 个相关 crate 生产编译、`runtime-benchmarks` 编译、127 项专项单测、格式化与 diff 检查全部通过；runtime 中旧合并姓名字段、旧管理员类型和历史迁移代码残留为 0。
- 2026-07-18：当时的 Node 三字段验收已由 2026-07-20 公权四字段/私权三字段分流实现取代；管理员允许没有岗位，但只有有效岗位任职和岗位业务权限才能构成机构授权。
- 2026-07-18：第2步验证通过：`cargo check -p node`、Node 280 项测试、Node 前端 TypeScript 与 Vite 生产构建、旧管理员结构 / 纯账户布局 / 合并姓名字段 / DTO 泛化账户字段残留检查全部通过。当前源码强制重建 WASM 后，隔离 `citizenchain-fresh` 真实启动成功，block#0 `0xc1dc759689aed0a8f8361dc3cb0e39c1faf19cfc55c7611b02ccc79ce04524c6`、`stateRoot=0x967155d28abe492052ef4bfd59a1ddbebce8cdaa57d9baaad446028848061a5e`、`isSyncing=false`；节点已停止并清理临时数据，本步未烘焙正式 chainspec 或切换正式数据。
- 2026-07-19：OnChina 当时完成三字段跨端验收；2026-07-20 公权链上部分已改为四字段 `PublicAdmin`，私权仍为三字段 `Admin`，业务授权改按完整岗位主体校验。
- 2026-07-19：第3步验证通过：OnChina 137 项测试、后端生产编译、前端 TypeScript/Vite 生产构建和旧字段残留检查通过。隔离 fresh 节点 block#0 为 `0xc1dc759689aed0a8f8361dc3cb0e39c1faf19cfc55c7611b02ccc79ce04524c6`、`stateRoot=0x967155d28abe492052ef4bfd59a1ddbebce8cdaa57d9baaad446028848061a5e`、`isSyncing=false`；临时 PostgreSQL 实测旧合并姓名列为 0，并验证旧单列重启后被直接删除、默认落为“管理”“员”。链投影 49,593 个机构、99,231 个账户，健康接口和首页正常。所有验收进程已停止，临时数据已移入废纸篓；未烘焙正式 chainspec、未切换正式节点数据。
- 2026-07-19：CitizenApp 当时完成三字段验收；2026-07-20 已按目标 pallet 分流公权四字段与私权/个人三字段解码，公权身份资料为空时不再伪造展示值。
- 2026-07-19：第4步的旧公权三字段 fresh 验收只保留为历史记录，不再代表当前存储布局；当前 fresh 链必须以四字段 `PublicAdmin` 重新验收。
- 2026-07-19：第5步统一 QR 三字段结论的公权部分已被 2026-07-20 四字段协议取代；CitizenWallet 当前按公权 `PublicAdmin`、私权/个人 `Admin` 分流严格解码。
- 2026-07-20：CitizenWallet 已按实际 runtime call 布局分流解码公权/私权机构治理和注册局登记；公权身份字段允许暂空，非空 CID 只接受 CTZN 结构，旧内层凭证字段不再属于这两个 call。
- 2026-07-19：CitizenWallet 同一已扫描请求只允许一次密钥调用：签名进行中或已生成响应二维码时拒绝重复触发；同一次业务操作不叠加姓名确认签名或其它第二签名。统一 action registry 已重新生成 CitizenApp/CitizenWallet 两端产物，个人多签创建的必显字段补齐 `admins`。
- 2026-07-19：第5步验证通过：`qr-protocol` 6 项一致性/守卫测试、CitizenWallet `flutter analyze`、179 项全量测试、Android arm64 debug 构建、CitizenApp `flutter analyze` 及 QR/签名 53 项测试全部通过。Pixel 8a 真机安装后 `org.citizenwallet/.MainActivity` 真实启动、进程存活并渲染“还没有钱包”首屏；设备无钱包，未创建测试私钥、未伪造扫码签名或链交易。Android 16 同时报告现有 Flutter/插件原生库未满足 16 KB 页面对齐，关闭系统提示后应用正常渲染；该独立发布风险留待正式创世前阻塞审计处理。
- 2026-07-19：第5步旧字段、旧载荷和旧文档口径已清理；全仓非 runtime 仅保留 OnChina 启动时 `DROP COLUMN IF EXISTS` 删除旧数据库合并姓名列的清理 SQL，不存在兼容读取。本步未修改 runtime、未烘焙 chainspec、未切换节点数据、未部署或推送。
- 2026-07-20：私权创世机构最终重新定义为 SFGY 非营利法人“中国公民链技术发展基金会”：CID `GZ018-SFGYR-201206100-2026`，主/费用账户按最终 CID 权威派生；法定代表人程伟引用公民 CID `GZ000-CTZN6-198805200-2026`，不伪造第二份公民记录。block#0 在 `PrivateManage/PrivateAdmins` 只写一名程伟管理员，同一钱包分别任职 `LR / GENESIS_PRODUCT_MANAGER / GENESIS_PROGRAMMER` 三岗，每岗一席，机构阈值保持 2。
- 2026-07-19：公民链基金会身份、协议账户和治理骨架纳入 NodeGuard，总保护数为 90（89 公权 + 1 私权）；固定岗位不能增删、改名或停用，法定代表人、创世产品经理和创世程序员各固定一席。任职账户必须属于 `admins`，同一账户允许跨岗位兼任；基金会当前由一名私权 `Admin` 同时担任三个岗位，法定代表人三字段与 `LR` 任职账户一致。依法换人时，岗位任职和法定代表人字段必须原子同步；新任人员尚不在 `admins` 时才同时更新名册。
- 2026-07-19：第6步验证通过：primitives 72 项、private-admins/private-manage 17 项、runtime 45 项、Node 281 项、OnChina 137 项测试全部通过；生产/no-default-features 编译、WASM 强制重建、格式和残留检查通过。当前源码以隔离 `citizenchain-fresh --tmp` 真实启动，节点守卫启动自检通过，block#0 `0x1732f0f1005d7e8ee7f9292e35a036698ece48569f6aca8e01f56f264761083d`、`stateRoot=0x37379726b7245af3123618fe032fa5355e17ec847159703daf6a9b5322a04fd3`、`isSyncing=false`。本步未烘焙正式 chainspec、未更新 CitizenApp/Cloudflare 正式资产、未切换节点数据、未部署或推送。
- 2026-07-17：OnChina、CitizenWallet、CitizenApp 已同步删除 `institution_create_credential` 动作码；CitizenWallet 对 `0x1e05/0x1f05` 按新 call-data 顺序解码并统一中文展示，创建机构链路不再存在内层凭证 Option 分支。
- 2026-07-17：本轮验收通过：`cargo check -p citizenchain`、`cargo check -p onchina`、`cargo test -p public-manage -p private-manage`、`cargo test -p onchina core::institution_call`、OnChina 前端 `npm run build`、CitizenWallet `flutter analyze`、CitizenWallet `flutter test test/signer/payload_decoder_test.dart test/signer/field_labels_test.dart`、CitizenApp `flutter test test/qr/qr_router_test.dart`、CitizenApp `flutter analyze`、`git diff --check`。OnChina test 构建仍有既有 `GENESIS_CITIZEN_MAX` 未用常量警告，与本次创建机构签名链路无关。
