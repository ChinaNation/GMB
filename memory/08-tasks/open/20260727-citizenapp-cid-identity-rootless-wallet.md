# citizenapp CID 身份层 + 无根多账户钱包(全量跨模块)

状态:open(2026-07-30 全仓审查、真实验收、CI-WASM preview、Node 20 清零与 WASM
供应链固定已完成；等待固定新候选并重跑唯一 WASM CI)
所属模块:Chain(citizenchain runtime)+ OnChina(注册局)+ Mobile(citizenapp);
citizenwallet 的注册局扫码签名联动已在 S4+S6 交接卡完成，本地静止态加密属于独立任务
关联/推翻:
- 承接并**重塑** `20260727-citizenwallet-modelb-index-derivation.md` 的 Step 2(原「citizenapp 保留种子/助记词、单热钱包」决策被本卡**推翻**)
- Step 1(citizenwallet 无根多账户)已完成,本卡把同一无根 `//index` 模型搬到 citizenapp
- ADR-026 统一签名 [[unified-signing-protocol-adr026]];CID 格式 [[cid-code-table-level-final]] / [[china-code-immutable]](本卡改公民 CID 前缀 → 需同步)

## 2026-07-29 用户最终身份契约（覆盖本卡全部旧决定）

- 公民 App 用户身份唯一主键是永久 `cid_number`；当前绑定 `account_id` 只负责签名、鉴权和付款。
- 一个 CID 同时只允许一个有效钱包账户，一个钱包账户同时不得绑定多个 CID。
- 非投票、非竞选 CID 可在公民 App 自主绑定/换绑，也可经注册局绑定/换绑。
- 投票公民和竞选公民只能经注册局绑定/换绑；CID 一旦升级为投票公民，永久禁止退回自主换绑路径。
- 自主换绑由旧账户授权、新账户提交；注册局换绑由新账户证明控制权，不要求旧账户签名。
- 链上 finalized 换绑即代表 CID 全部控制权转移到新钱包；本地密钥接管、会话撤销和缓存清理只是该链上事实的派生动作，不得另造第二套旧账户清理授权。
- 绑定防重放以 CID 为键，使用单调 `binding_revision`，签名同时绑定创世哈希、预期旧/新账户和过期时间。
- 数据按三层实现：CID 永久业务数据、CID 稳定数据根、当前钱包账户派生包装密钥；新包装验证成功后才删除旧包装。
- 订阅关系归 CID，续费和收款每次使用 CID 当前绑定账户；换绑后直接由新账户付款。
- 个人多签不属于 CID 身份换绑：其 `admins` 继续是明确登记的 `account_id` 签名账户集合，CID 换绑不得修改个人多签管理员或投票快照。
- 账户原生余额不随 CID 换绑迁移。
- 全部修复、测试和真实运行态验收通过后才生成唯一正式创世；随后按“WASM CI → 其他软件 CI → 部署”顺序推进，每次远端推送和生产操作另行确认。

### 分步实施门禁

1. 链上绑定权限、绑定版本、防重放与跨端签名协议。
2. CID 控制权转移与三层密钥接管。
3. CitizenApp 本地数据、Chat、MLS、草稿、通讯录全面改为 CID 分区。
4. Cloudflare 会话、设备、WebSocket、KeyPackage 和联系人账户更新。
5. OnChina 全链 CID 查询、注册局办理、finalized 投影。
6. 投票、候选人、奖励资格和 NodeGuard 统一保护 CID。
7. 全仓命名、注释、文档、测试和残留终审。
8. 真实整系统验收、正式创世和冻结。
9. 获得逐次远端许可后依次执行 WASM CI、其他软件 CI 和部署。

每一步必须先提交技术方案并得到用户确认；执行完成后更新文档、完善中文注释和测试、清理本步残留，再输出下一步技术方案。遇到无法从代码、文档或真实运行结果确定的问题必须停止并沟通。

### 第 1 步完成记录（2026-07-29）

- 链上绑定权限已收敛：非投票/竞选 CID 允许自主或注册局办理；投票、竞选以及
  曾成为投票公民的 CID 永久只允许注册局办理；个人多签 `admins` 未改动。
- `BindingRevisionByCid`、创世哈希、预期账户、过期时间和 finalized 状态已成为
  四端一致的绑定防重放与控制权转移契约；初绑版本为 1，换绑/撤销单调递增。
- CitizenApp、CitizenWallet、OnChina 与链端的 SCALE 载荷、调用参数、finalized
  两阶段恢复和失败关闭行为已统一；旧账户二次清理授权与旧接口残留已删除。
- 链端目标测试、完整 `citizenchain` 测试、真实 benchmark 权重生成、release WASM
  重建、CitizenApp/Worker、CitizenWallet、OnChina 测试与静态检查均通过。
- 已用全新临时双节点、真实 OnChina HTTP 服务和临时 PostgreSQL 完成运行态验收；
  临时服务、数据与 Rust 增量产物已清理。未生成或冻结正式创世，未提交、推送或触发 CI。

### 第 2 步完成记录（2026-07-29）

- Worker 新增按 `cid_number` 唯一存储的稳定数据根，服务端以
  只存在于 Secret 的 `CID_DATA_ROOT_RECOVERY_KEY` 按创世、CID 与密钥版本派生 KEK
  后密封；同一 CID 换绑前后数据根与 `data_root_hash` 不变，激活版本只允许单调推进。
- 接管接口使用当前 finalized 新账户的一次性挑战签名；挑战绑定创世哈希、CID、
  `account_id`、`binding_revision`、随机 challenge 和过期时间，且在验签前后两次读取
  finalized 绑定。旧挑战、旧 revision、错误账户和重复消费全部失败关闭。
- CitizenApp 数据根金库固定执行“新账户包装 → 读回摘要校验 → 激活四元组标记 →
  派生 CID 用途子钥 → 清理低版本包装”。接管函数没有此前账户参数，也不读取此前账户
  私钥、公钥、签名或设备。
- 自主换绑授权未被削弱：runtime 调用仍由当前绑定账户签授权摘要、新账户作为交易
  origin 提交；注册局换绑仍只走注册局/管理员权限与新账户控制证明。两条链上路径
  finalized 后统一由当前新账户独立接管。
- `IdentityAccountCache` 与 Worker 受保护路由均改为 finalized 当前绑定失败关闭；
  本地完成标记由账户单值升级为
  `(cid_number, binding_revision, account_id, data_root_hash)`，崩溃后按链上真值幂等补齐。
- 删除账户间通讯录搬运、旧账户重封装、钱包创建时生成随机数据根等错误流程；Chat、
  MLS、附件、通讯录本地/云端和草稿用途域统一从 CID 稳定数据根派生。
- 验收：Worker `npm run typecheck` 通过，Vitest 32 文件 222 项全绿，其中接管接口
  已通过完整 Worker HTTP 入口与真实本地 D1/KV binding；基线 SQL + `0003` 也已在
  内存 SQLite 实际执行通过。CitizenApp
  `flutter analyze` 通过，Flutter 1036 项通过、5 项因本机无原生 smoldot 条件性跳过、
  0 项失败。未修改 runtime，未创世、未推送、未触发 CI、未部署。

#### 2026-07-31 复审纠正

提交 `87d97365` 曾错误地把稳定数据根改为由钱包母种子派生，导致旧钱包完全丢失后新
钱包无法接管旧密文；本轮已恢复上面的独立恢复层终态，并删除助记词补录、旧账户解包
和旧设备依赖。当前实现与验收结果以任务卡
`20260731-cid-data-root-user-owned-keys.md` 的完成记录为准；此前关于母种子派生或“服务端
不持有任何恢复密钥材料”的阶段性说明全部作废。

### 第 3 步完成记录（2026-07-29）

- Chat 会话、消息、出站队列、待投递媒体、待处理入站、路由、群镜像和乱序 Commit
  全部改为 `ownerCidNumber` 复合唯一/查询/删除；消息信封、MLS 名册和投递统一使用
  CID，当前绑定 `account_id` 不再作为本机数据归属键或 Chat 身份。
- Chat 明文、搜索索引、附件和 MLS 状态统一从 CID 稳定数据根派生用途子钥；
  路径固定为 `chat/by_cid/<cid_number>/...`。同一 CID 换绑新账户后可直接解密
  原本地数据，不读取或要求此前账户、私钥或设备。
- 广场草稿模型、KV 前缀和媒体目录全部按 `cid_number` 分区；通讯录本地 KV 固定为
  `contact_book_by_cid:`、`contact_pending_by_cid:`、`contact_sync_by_cid:`。
  删除普通旧账户不再删除 CID 通讯录；只有用户明确清空或删除当前热钱包时才执行
  本机隐私擦除。
- 公权机构关注改为 `subscriberCidNumber + institutionCidNumber`；身份徽章快照、
  平台订阅/创作者订阅 finalized 待提交证明、创作者档位镜像和有限交易历史均按
  永久 CID 分区。当时的 `signer_account_id` 只保留为不可变交易事实，新绑定账户的
  同一 CID 会话可继续提交旧 finalized 证明。
- Isar 生成模型已重建；中文注释、模块技术文档、统一协议和关联任务卡已同步，
  本步废弃账户分区命名与错误换绑重建描述已清除。
- 验证：`flutter analyze --no-pub` 零问题；草稿、通讯录、真实 Isar 机构关注和完整
  Chat 定向套件 208 项通过，4 项原生 OpenMLS 因桌面宿主无动态库按既有条件跳过；
  身份、创作者、机构页和钱包生命周期定向 77 项通过。最终全量 Flutter 套件
  1038 项通过、5 项按原生库/链烟测环境条件跳过、0 项失败；同时修复
  `bootstrap_test.dart` 全局 Navigator key 未卸载造成的顺序依赖。Worker
  `npm run typecheck` 通过，Vitest 32 文件 222 项全绿；`git diff --check`
  通过。未修改 runtime、未创世、未推送、未触发 CI、未部署。

### 第 4 步完成记录（2026-07-29）

- Worker 登录挑战、Session、设备子钥、动作挑战、Chat 设备、KeyPackage 和 WebSocket
  全部绑定 `cid_number + binding_revision + account_id`；每次受保护请求和投递均重新
  核验 finalized 当前绑定，旧 revision、旧账户和反向闭环不一致全部失败关闭。
- CID 数据根接管固定由 finalized 新账户完成挑战签名；新包装读回摘要验证并激活后，
  才清理旧 Session、设备子钥、Chat 设备、KeyPackage 和旧三元组 WebSocket。流程没有
  旧账户、旧私钥或旧设备参数，CID 业务数据和稳定数据根不删除、不重建。
- Chat Protobuf、Dart/Rust OpenMLS 凭证、群名册、Isar、HTTP 与推送路由统一以 CID
  为唯一身份键。当前账户只负责 Session、设备登记签名、本地 CID 数据根包装和付款；
  换绑不改变会话、消息、群成员、附件或 MLS 目录的归属。
- 通讯录关系和私人备注继续以联系人 CID 归属；页面加载后在同一 finalized 块批量校验
  CID 正向绑定、Active 记录、单调 revision 与账户反向绑定，再更新账户/SS58 快照。
  转账前再次按联系人 CID 严格解析最新绑定，失败时禁止回退旧快照；私信只传联系人 CID。
- 公开资料缺失时的默认昵称、头像和背景统一优先按 CID 稳定生成；完整 AccountId、
  SS58 和截断身份键均不能充当公开昵称。纯访客没有 CID 时才允许用当前账户作展示种子。
- 代码、中文注释、Chat/USER/钱包/安全/统一协议文档和关联任务卡已同步；旧
  `sender_account_id`、`recipient_account_id`、`peer_account_id`、账户群成员及
  `grp:acct*` Chat 残留已从实现和目标文档清除。Worker 瞬时投递内部保存的当前授权
  账户也明确命名为 `recipient_binding_account_id`，不得被解释为收件人身份主键；
  历史任务卡中的阶段性账户身份说明已经按 CID 终态订正。
- 验收：`flutter analyze --no-pub` 零问题；CitizenApp 全量测试 1042 项通过、5 项因
  桌面宿主无 native smoldot/OpenMLS 动态库按既有条件跳过、0 项失败。Worker
  `npm run typecheck` 通过，Vitest 32 文件 222 项全绿；真实本地 D1/KV/R2 Worker
  HTTP 入口覆盖 CID 数据根接管与换绑后数据延续。Rust OpenMLS 13 项全绿，
  `chat_mls.rs` 独立 rustfmt 检查通过，增量产物已清理；`git diff --check` 通过。
  收尾复跑时额外修复充值 intent 篡改单测偶发未真正改字节的问题，最终 Worker 仍为
  32 文件、222 项全绿。
  未修改 runtime、未创世、未推送、未触发 CI、未部署。

### 第 5 步完成记录（2026-07-30）

- OnChina 新增公民 finalized 身份快照唯一读取入口；同一 finalized 区块原子读取
  `CidRegistry`、CID→账户、绑定 revision、账户→CID、投票身份和竞选身份，并严格校验
  正反闭环、revision 和身份层级。创世块缺少 `Timestamp.Now` 按区块号 0 精确处理，
  非创世块缺失仍失败关闭。
- 占号、注册局换绑、投票/竞选身份推送和吊销都以目标 extrinsic 所在 finalized 区块的
  完整快照回写本地，不再信任扫码账户、会话账户或本地旧账户。投影只允许 revision 前进
  或同版本同账户幂等写入；吊销清空当前账户但保留 revision 和 finalized 锚点。
- 创世法定代表人本地投影删除硬编码管理员账户和 revision=0 的旧逻辑，改为读取链上
  finalized 公民身份快照；只有 Active 绑定才作为当前账户，审计创建来源不得充当控制权。
- `citizens` 删除墙钟式 `account_verified_at`，终态字段改为 `binding_revision`、
  `binding_finalized_block_number`、`binding_finalized_block_hash`。没有保留 ALTER、
  双读或旧字段兼容；旧业务库必须按最终 schema 重建。
- 新增注册局全局链上绑定接口
  `GET /api/v1/admin/citizens/:cid_number/binding`。任一已登录 FRG/CREG 都可查询同一
  finalized 公开绑定，接口不读取或暴露链下档案。详情页在占号、换绑、身份推送和吊销后
  重新查询该接口，当前钱包和版本不做乐观猜测。
- 真实验收使用当前源码 fresh runtime、独立链和两套独立 PostgreSQL/OnChina 实例。
  两个不同市注册局对同一 CID 返回逐字段一致的 Active 绑定、revision=1、当前账户和
  finalized 锚点；两套机构投影均为 49,593 个机构、99,232 个账户。隔离 fresh 哈希
  `0x49622cb851a0815af75573e281b992565cb31df509c2e3d3b847858c351ef46e`
  只作本步验收，不是正式创世。
- 现有旧冻结链缺少 `BindingRevisionByCid` 元数据，新 OnChina 已验证会失败关闭而不回退。
  因而新 OnChina 不得独立部署到旧 runtime，必须等待后续正式创世统一切换。未修改
  runtime 源码、未生成正式创世、未推送、未触发 CI、未部署。
- 后端 179 项测试、`clippy --all-targets --no-deps -D warnings`、前端 TypeScript 检查和
  本步 Rust 文件独立格式检查均通过。源码及编译包残留扫描通过；Git 跟踪的前端编译包已
  替换为当前源码哈希 `index-DtHhRXFp.js`。现有 HTTPS OnChina 真实加载该脚本和既有 CSS，
  登录页完整渲染且浏览器控制台无错误或警告。

## 历史漏洞（已由当前契约修复）
旧模型曾把钱包账户当身份唯一主键，私钥泄漏会迫使用户更换身份并丢失动态、文章、粉丝等
CID 业务数据。当前终态已改为 **CID 号是唯一身份主键**，钱包账户只是可换绑的签名、
鉴权和付款凭证；个人多签 `admins` 是独立账户集合，不得与公民 CID 换绑混为一谈。

## 三决策(用户 2026-07-27 拍板)
- **D1 自助·自付费**:公民本人 `Signed` origin 直接占号,自付最低链上费(`ONCHAIN_MIN_FEE=10`),绕过注册局。新增 permissionless `self_occupy_cid`。
- **D2 两条换绑路径（2026-07-29 最终覆盖）**：非投票/竞选 CID 的自主换绑由当前绑定
  账户签名授权、新账户提交；注册局换绑由在册机构 `admins` 权限和新账户控制证明完成，
  不要求当前旧账户签名。投票/竞选以及曾成为投票公民的 CID 只能走注册局路径。
- **D3 人主体 CID 去地域化 + 号段扩容(人/机构彻底分开)**:**人主体(公民 CTZN / 居民 NATP / 智能人 SMTP)统一**(用户 2026-07-27 修正:不留任何 person 走省码;NATP=居民,非自然人)R5 **位 1-2=`CN` 国家码;位 3-5 由固定 `000` 改为承载号码高 3 位**(原 000 对人无意义,回收作号段)。人号段 = R5 位3-5(高 3 位)+ N9(低 9 位)= **12 位 = 1e12/年**;人只吃公钥/码/年,不吃省市。机构 CID 才用省码+市码(`is_person_code` 分流,`CN` 前缀天然区分)。居住省/市仍单独存于 `VotingIdentity`/`CidRecord`(与 CID 号解耦)。[[unify-means-zero-exceptions]] 零例外。

## 实施前链侧基线（历史侦察，`runtime/misc/citizen-identity/src/lib.rs`）
- 存储**已全在位**:`CidRegistry`(CID→记录,含 `commitment[32]`/residence/status Active·Revoked)、`AccountIdByCid`+`CidByAccountId`(双向绑定闭环 `bind_account_id` L1407 / `ensure_account_id_binding_available` L1375)、`VotingIdentityByCid`、`CandidateIdentityByCid`、四级人口计数。**匿名 CID = 有 CidRegistry+双向绑定、无 VotingIdentityByCid**,存储天然可表达。
- 缺口:①全部 extrinsic **注册局门**(`CitizenIdentityAuthority::can_manage_voting_identity`,CREG/FRG);无自助。②`occupy_cid`(call 6,L1085)**只写号不绑账户**;账户到 `register_voting_identity`(call 0)才绑。③**无换绑**(`update_*` 禁改账户 `ensure_current_account_id_binding` L1395)。
- CID **客户端生成**(`primitives/cid/generator.rs`),链只格式+查重仲裁(`do_occupy_cid` L1237);现个人码强制市 `000`、R5=省+000、N9=hash(公钥|码|省名|000|年)。**D3 改为 R5=CN000。**
- 费:citizen_identity 全走 `institution_onchain_route`(机构付费,`configs.rs fee_route` L431+);**自助需新自付费 arm**(`signer_onchain_route` 式,签名人自付,min 10)。

## 跨模块设计

### ① 链 citizen-identity(Blockchain)
- **CID 格式改 D3**:`primitives/cid/{generator.rs,number.rs,code.rs}` 公民 CID R5=`CN`+号码高3位;12 位号段 = `hash(公钥/种子|CTZN|年) mod 1e12`+nonce probe,高3→R5[2:5]、低9→N9;`parse_cid_number_parts` 放行 `CN` 前缀且 R5 位3-5 为数字(跳过省/市码表查);核心段校验位含 R5 自然覆盖。最终公式钉金标。**单源,冷热链共用。**
- **新 `self_occupy_cid`**(Signed 自助):参数 `cid_number`(客户端 CTZN/CN000)+`commitment[32]=blake2_256(pubkey)`;占号即绑账户(写 CidRegistry+双向绑定,复用闭环校验);**不写 VotingIdentity**;自付费;新 op_tag 入 `primitives::sign`。
- **新 `rebind_cid_account`**(自助轮换):当前绑定账户签名授权 → 换绑到新账户;更新双向绑定;旧账户失效。
- **改 `occupy_cid`**(注册局路径,req 5 第1步):加账户绑定参数,占即绑。
- `register_voting_identity`/`upgrade_to_candidate_identity` → 匿名 CID 之上**可选升级**(req 5 第2步「创建档案」;「匿名」=止于占号+绑定)。
- `fee_route` 加自付费 arm。**dev 期重新创世** [[chain-in-dev]] + 注册表重生 [[registry-regen-after-genesis]]。新业务逻辑留 pallet,不进 primitives 核心常量 [[no-business-types-in-primitives]]。

### ② onchina 注册局(OnChina)
- `src/domains/citizens/occupy.rs`:占号第1步加钱包账户(占即绑);第2步 handler/UI 分「创建公民档案」/「匿名」两支。
- CID 生成同步 D3(`CN000`)。

### ③ citizenapp(Mobile,两层)
- **钱包层**:无根多账户派生(逐字节复用 Step 1 `//0//1//2` 金标)+ 只存账户公私钥对、不存种子/助记词;单钱包;删 App 内「创建/导入钱包」入口 + 删「默认账户/默认用户」;钱包空→踢回初始化;账户详情加「添加账户」(输助记词+自动填充,派生下一 `//index`);列表**热账户 + 冷钱包共存**(冷钱包代码不动);创建提示手抄助记词或在 citizenwallet 保管。
- **身份层**:`电子护照`→`身份`改名;首卡`注册身份·访客`(匿名图标留);初始化后跳身份页;右上`刷新`→`注册`/`更换`按钮 + 下拉刷新;点注册=生成 CN000 CTZN CID→签名→`self_occupy_cid` 自付费交易→绑定;已注册首卡下显`身份cid号 xxxxx`、下两卡`公民cid号`→`身份cid号`;对外 CID 主、账户辅;**身份主键 `getDefaultWallet()`→CID-bound account**(20+ 调用方联动:8964/chat/my/membership/contact/myid);聊天/广场/订阅/创作者/通讯录 **门控到已注册 CID**(与四层门禁 [[citizenapp-four-gate-entry-failclosed]] 合并厘清)。

## 小步执行序(每小步先出技术方案待确认再执行;`dart analyze` 0 + `flutter test` 全绿 / `cargo test` 全绿为门禁)
- **S1 链**:公民 CID 号段方案(CN 前缀 + 位3-5 承载号码高位 + N9 = 12 位/年;primitives/cid 生成+校验 + node 共识守卫)+ 金标向量。**✅ 完成(2026-07-27,全工作区绿)**。
- **S2 链**:`self_occupy_cid`(自助+自付费+占即绑+匿名)+ op_tag + 测试。**✅ 完成(2026-07-27,全工作区绿)**。
- **S3 链**:`self_rebind_cid_account_id`(自助轮换,origin=新账户+旧账户签名,OP_SIGN_CID_REBIND)+ 测试。**✅ 完成(2026-07-27,全工作区绿)**。
- **S4+S6 合并(注册局流程,链+OnChina+双钱包+前端四端)**：occupy 占即绑、
  `admin_rebind_cid_account_id`、CTZN|NATP、QR 完整授权模板、冷热钱包原位填账户槽、
  OnChina 两次扫码与 finalized 两阶段恢复均归自包含交接卡
  `20260727-registrar-cid-flow-s4s6.md`。匿名 CID 任一在册 CREG/FRG 可办，civic CID
  由同市 CREG/对应省 FRG 办理；跨注册局链上换绑不是后期项。
- **S5 链**:创世身份代码 **✅ 完成**；正式 chainspec、CitizenApp 轻链资产、公权
  机构快照和 Cloudflare 创世哈希的同源重生仍待执行。冷热链一致性与真机业务闭环
  属重生后的测试，不得与发布资产生成混为同一步。
- **S6 onchina**:已并入 S4+S6 合并交接卡(见上）。
- **S7 citizenapp 钱包层**:无根多账户重构(独立于链)。**✅ 完成(2026-07-27,S7.1/7.2/7.3 全绿)**。
- **S8 citizenapp 身份层**:改名 + 注册按钮 + 自助占号交易 + CID 主键切换 + 门控。拆三子步:
  - **S8.1 客户端 CID 生成 + 占号/换绑 RPC**(纯离线编码,逐字节镜像链,不依赖 S4+S6)。**✅ 完成(2026-07-27,848 绿)**。
  - **S8.2 身份页改造**(电子护照→身份改名 + 首卡`注册身份·访客` + 初始化跳身份页 + 右上`注册`/`更换`按钮 + 下拉刷新 + CID 展示)。**✅ 完成(2026-07-27,867 绿)**。**决策裁定(用户)**:①**不新增徽章色**,保持现有 3 色(visitor/voting/candidate),匿名占号后仍访客色,仅**第 1 卡显已注册 CID 号**;②「更换」**含**,与注册一起做(匿名 CID 可自助换绑选目标账户;voting/candidate 提示去注册局);③初始化后**跳身份页但绝不动底部 5-tab 结构**(onboarding 放行后一次性 push MyIdPage,返回回落广场)。
  - **S8.3 身份主键切换 + 门控 + 删默认用户**(`getDefaultWallet()`→CID-bound account、`signWithWallet` re-plumb 到 accountId、聊天/广场/订阅/创作者/通讯录门控到已注册 CID、删「默认用户」)。**决策裁定(用户,2026-07-27,grounding 后校准)**:
    - **①时机=现在全做**;**验证=本地开发链重新创世**(本地起含本会话 S1-S3 `self_occupy_cid` 的 citizenchain 矿工端重新创世,app 连它走 新建钱包→自助占号→匿名已注册→门控解锁 全链路)。**⚠️纠正前提**:创世公民(法代 `CN220-CTZN2-198805200-2026`/账户 `0x0cb1d05c…4b06b`)在 `citizen-identity` **无 GenesisConfig 记录**=链上纯访客(`readByAccountId` 返回 null),**不能用它验证门控/主键**;助记词也不在仓。
    - **②门控=全功能门控 CID**,未注册即引导注册,访客无匿名可用面。
    - **③身份账户=CID 当前绑定的钱包账户(可任意 `//n`,非恒账户0)**——身份主键始终是
      CID；账户只负责签名、鉴权、付款和包装 CID 数据根。设备登记证明由当前账户签，
      业务数据与用途密钥归 CID，换绑不搬迁业务数据。
    - **架构发现**:身份主键类型从 `WalletProfile`(钱包级)降到 `Account`(账户级);27 处 `getDefaultWallet`+27 处 `signWithWallet` 按用途分流(身份→身份账户 / 付款治理机构→保留账户0或所选)。QR 签名服务(citizen_identity/square_action)只遍历 WalletProfile,子账户身份有 gap 需扩。
    - **拆子步**:S8.3a 身份账户单源 `getIdentityAccount()` + 注册选绑定账户
      **✅完成**；S8.3b finalized 当前账户鉴权与 CID 数据根接管 **✅由本任务第 2 步
      最终收口**；S8.3c 全功能门控 CID；S8.3d 删默认用户；S8.3e 并入正式创世前 e2e。

## S1 落地(2026-07-27,完成 ✅)
> **修正(2026-07-27,用户)**:CN 方案由「仅 CTZN」扩为「**全部 `is_person_code`(CTZN/NATP/SMTP)**」——删 `is_citizen_code`,generator/`cid_scope_codes` 守卫/node `valid_province_city` 均改 `is_person_code`,generator else 分支删原 person city=000 死逻辑,人主体金标测试覆盖三码。CTZN 金标值不变。**`cargo test --workspace` 全绿(81 批次 0 failed / 0 panicked)。**
**改动(仅链侧单源,主检出 `citizenchain/`)**:
- `runtime/primitives/cid/code.rs`:新增 `is_citizen_code`(仅 CTZN;NATP/SMTP 仍省+000)。
- `runtime/primitives/cid/generator.rs`:CTZN 分支产 `CN`+高3位、N9 低9位,12 位号 = `hash_u64(公钥|CTZN|年) mod 1e12`(新增 `hash_u64` 取 blake2 前 8 字节);公民分支**不再要求省/市**(自助占号无居住地);省市机构分支逻辑不变。
- `runtime/primitives/cid/number.rs`:`cid_scope_codes` 加公民 fail-closed 守卫(CTZN 传入即 Err,杜绝把号段误读成区划码);doc 更新。
- `node/src/core/node_guard/cid_lifecycle.rs`:**共识守卫**(独立第二 CID 校验实现)`valid_province_city` 加公民分支——CTZN 校验 R5=`CN`+3数字,而非省+市(否则节点会拒 CN 公民 CID)。**CN 方案的第二处必改点**,由全工作区回归暴露并修复。
- `onchina/src/cid/generator.rs`:仅测试断言改 CN(包装逻辑不变,仍向 primitive 传省份,被 CTZN 分支忽略)。
- 金标:`generator.rs` `citizen_cid_number_golden` 钉死 dev //0 公钥 `0x2afba927…` → **`CN951-CTZN1-539598435-2026`**(12 位号=951539598435);另加 `citizen_cid_needs_no_province`、`number.rs::citizen_cid_parses_but_has_no_scope`。
- **测试**:`cargo test --workspace` 全绿(primitives 78 / citizen-identity / node bin 294 / 其余全部批次 0 failed;金标钉死)。

**留待 S5 重新创世 / Step 3 重生(非本步,记录防遗漏)**:旧**省码式**公民 CID 字面仍能 parse+validate(parser 不查省码),但语义已过时,须随重生改 CN 前缀:
- 创世常量:`runtime/primitives/cid/china/china_zf.rs:30 FSC_GENESIS_ADMIN_CID_NUMBER`、`.../citizenchain.rs:22 LEGAL_REPRESENTATIVE_CITIZEN_CID_NUMBER`(均 `CN220-CTZN2-198805200-2026`)。
- 测试夹具:square-post / private-manage / public-manage / admin-primitives / public-admins / runtime/src/tests 里的 `GZ000-CTZN6-…` `GD001-CTZN1-…`(结构合法,S5 统一改 CN 提高真实度)。
- 校验(已核):链侧无 live code 把公民 CID 传 `cid_scope_codes`(守卫兜底);`legislation-yuan` 的 `parts.r5` 只用于**公权机构** CID 路由(`ensure_route_institution_*`,不碰 CTZN)——无 live 隐患。

**S6 onchina R5-split 适配 —— ④c 完成 ✅(2026-07-27,交接卡④c段)**:onchina 三处本地「从 CID R5 切省/市」全收敛到 primitives 权威单源 `cid_scope_codes`(自带人主体 fail-closed):`genesis_projection.rs::split_province_city`、`projection.rs::r5_province_city`、`main.rs::Db::scope_codes_from_cid`(实际调用点非预估的 `docs/handler.rs`,喂机构文档 CID)。**订正**:程伟创世常量**已是 CN 号**(`CN220-CTZN2-…`,非「旧省码式今可用」),旧切割已把省误读成 `CN` 误播种 → 是 live bug 非未来隐患;已改程伟省市取自基金会机构 CID(GZ/018,创世同址)。`cargo test --workspace` 81 批次 0 failed。**第 4 处 `runtime_ops.rs::append_audit_log` 亦已修**(用户拍板「办理局归属办理局、禁兜底」):审计分区改按本节点 `resolve_node_scope`(原按目标 R5 会把 CN 人主体算成省"CN"→无分区→审计静默丢失的 live bug),详见交接卡④c段。

## S2 落地(2026-07-27,完成 ✅)
> **q3 修正(2026-07-27,用户)**:自助占号类型从「仅 CTZN」放宽为「**CTZN(公民)/ NATP(居民)用户自选**」(投票/竞选公民只能经注册局线下注册=q1)。`do_occupy_cid` 移除 CTZN 硬校验并下沉到调用方(注册局 occupy/batch=`ensure_valid_citizen_cid` CTZN;自助=新 `ensure_valid_self_registrable_cid` CTZN|NATP;SMTP/机构码拒)。`cargo test -p citizen-identity` 43 绿(+NATP happy /+SMTP 拒);**`cargo test --workspace` 全绿(81 批次 0 failed)。**
**`self_occupy_cid`(自助占号+占即绑+匿名+自付费),仅链侧,主检出 `citizenchain/`:**
- `runtime/misc/citizen-identity/src/lib.rs`:新 extrinsic `self_occupy_cid(origin, cid_number)` **`call_index(5)`**(复用已退役槽,不留空位);`ensure_signed`→`account_id`(命名对齐 [[wallet-account-naming-account-id]]);**commitment 链上算** `blake2_256(account_id.encode())`(客户端不传);复用 `ensure_account_id_binding_available`(一账户一 CID 闭环)+`do_occupy_cid`(SELF registrar + 空 residence,格式+CTZN+原子查重)+`bind_account_id`(占即绑);**不写 VotingIdentity**(匿名)。新增 `pub const SELF_OCCUPY_REGISTRAR=b"SELF"` + 事件 `CidSelfOccupied{cid_number,account_id}` + `use sp_io::hashing::blake2_256`。
- `weights.rs`:trait + SubstrateWeight + unit 三处 `self_occupy_cid()`(手工权重 reads3/writes3,dev,待 benchmark 精化)。
- `benchmarks.rs`:`self_occupy_cid` 基准(`whitelisted_caller`),`--features runtime-benchmarks` 编译绿。
- `runtime/src/configs.rs`:fee_route 加 `CitizenIdentity(self_occupy_cid{..}) => signer_onchain_route(who,0)`(**签名人自付** min 10),置于机构付费 arm 前。
- `tests/mod.rs`:5 用例(绑定+匿名+SELF记录+commitment / 幂等 / 一账户一CID拒 / 他人占同CID拒 / 非CTZN拒)——**`cargo test -p citizen-identity` 41 全绿**。
- **node 守卫**:`validate_citizen_record` 放行 SELF(非空 registrar)+ 空 residence + Active + 无 VotingIdentity → **无需改守卫**(已验)。
- **回归**:`cargo test --workspace` = 19 批次 0 failed;node bin 唯一失败 `admin_unlock::lock_decrypted_admin_removes_entry` 是 **pre-existing 并行竞态 flaky**(共享全局 `decrypted_map()`,`list_decrypted_admins_filters_by_cid` 结尾 `clear()` 整表),**与 S2 无关**——单线程 `admin_unlock` 6/6 绿证。已记独立修复任务。

## S3 落地(2026-07-27,完成 ✅)
**`self_rebind_cid_account_id`(匿名 CID 自助换绑),仅链侧,主检出 `citizenchain/`:**
- 接口:`self_rebind_cid_account_id(origin=新账户, cid_number, old_account_signature)` **`call_index(9)`**。新账户作 origin(自签证受控+自付费);旧账户从 `AccountIdByCid[cid]` **反查、不传**(取不到=`NotBoundToAnyCid`);payload=`(cid_number,new_account_id).encode()`,旧账户对其 op-tag 签名(D2「当前绑定账户签名授权」)。
- 门:**匿名限定(q1)** —— 有 `VotingIdentityByCid` 即 `CivicRebindRequiresRegistrar`(civic 走 S4 注册局);**一账户一 CID** —— 新账户已绑他 CID 即 `AccountIdAlreadyBoundToAnotherCid`(新账户任意、能签即可,q 用户确认)。
- 换绑原语 `rebind_account_id`(删旧反向索引+写新双向绑定,old==new 幂等);事件 `CidAccountIdRebound{cid,old,new}`;错误 `NotBoundToAnyCid`/`CivicRebindRequiresRegistrar`/`InvalidRebindSignature`。
- **签名域**:`primitives/src/sign.rs` 新增 `OP_SIGN_CID_REBIND=0x11` + 进 `SIGN_OP_TAGS[12]` + fixture `signing_domain_vectors.json` 加 0x11 金标向量(`SIGN_GOLDEN_UPDATE=1` 回填)。**四端契约**:citizenapp(S8)/其余端签此换绑须同 op_tag。
- `CitizenIdentityAuthority` 加 `verify_rebind_signature`(**4 处实现全覆盖**:trait/`()`、configs Runtime 用 OP_SIGN_CID_REBIND、citizen-identity mock、citizen-issuance 集成 mock;后两 `==b"valid"`)——trait 加方法波及全部 impl,漏 citizen-issuance 集成测试 mock 曾致编译错、已补;pallet helper `ensure_rebind_signature`。
- `configs.rs` fee_route:`self_rebind_cid_account_id => signer_onchain_route`(自付费);weights 3 处 + benchmark(`account`+`whitelisted_caller`)。
- 测试:换绑成功(旧账户反向索引清、新账户接管)/ 未占拒 / 旧签名无效拒 / 新账户已绑他 CID 拒 / **civic 自助换绑拒** / **NATP 居民不能升级投票公民**(锁 q3 约束)。
- **状态**:**全绿** —— `cargo test -p citizen-identity` 49(+6 S3)、benchmark 编译、`sign_golden` 0x11 锁定、`cargo test --workspace` **81 批次 0 failed**。node 守卫放行 rebind(只校验绑定闭环一致性、不禁变更;registrar/residence 不可变只针对 `CidRegistry` 记录,rebind 不改该记录)。
- **q1/q2/q3 锁定(2026-07-27)**:q1 投票/竞选公民只能注册局线下注册(自助只出匿名);q2 自助换绑失败者去线下注册局,管理员可直接换绑任意 CID(=S4 `admin_rebind_cid_account_id`);q3 自助/注册局注册均让用户自选 CTZN(公民)/NATP(居民),且 **NATP 永不可升投票/竞选**(现有 CTZN 校验已保护)。

## S7 决策(D7,2026-07-27,本窗口进行中)
- **框架订正**:citizenwallet 现盘已被另一会话**反转成「存种子+助记词」**(`masterSeedHexV1`/`masterMnemonicV1`,已无 `accountMiniSecretV1`)。故可**逐字节复用的只是 `//index` 派生金标**(冷热共享);「无根存储(每账户 child mini-secret、不存种子/助记词)」在 **citizenapp 新建**,非照抄 citizenwallet 现盘。
- **D7a 存储+鉴权**:child mini-secret 存**硬件金库 strict 层**(citizenapp 唯一鉴权凭证);**签名/读密钥强制 `biometricOnly=true` fail-closed**,无生物识别/验证失败/取消一律拒(连创建/导入也拒)——[[biometric-only-mandatory]] 扩到 citizenapp 热端。
- **D7b 无根 + 角色分工**:助记词/种子**不持久化**;初始化(创建或导入)只在本机生成**公私钥对**,提示用户手抄助记词或在 citizenwallet 保管。分工定稿:**citizenapp=公私钥对日常用(无根)、citizenwallet=种子/助记词冷签+安全保管**。addAccount 须重输助记词(归属校验后派生下一 //index)。
- **D7c 边界**:CID 主键切换 / 删默认用户 / 签名改按 accountId → **S8**;S7 过渡期 `getDefaultWallet`/`signWithWallet(walletIndex)` 解析到账户0,§6 全部身份/签名调用方零改。
- **D7-合并**:**S7.1 = 派生 `//index` + 无根存储**(合并一子步,二者耦合)。

## S7.1 落地(2026-07-27,完成 ✅)
**citizenapp 无根单账户核心(派生 //0 + 无根存储),仅主检出 `citizenapp/`,冷钱包/§6 调用方零改:**
- **派生**:`wallet_manager.dart` 删 bare 根 `_deriveSr25519FromSeed`;新增 `_childMiniSecret`(逐字节移植金标)+`_deriveAccount(seed,index)`,账户0=`//0`。金标 `test/wallet/derivation_golden_test.dart`(//0//1//2 == 冷端,ss58 2027)**spike 全绿**。
- **无根存储**：`secure_seed_store.dart` 接口为
  `putAccountKey/readAccountKey/hasAccountKey/deleteAccountKey`（按 `account_id`，不保存
  根种子/助记词）；`hardware_bound_seed_vault.dart` 信封键固定为
  `account_child_key_{account_id}`，KEK 仍按 `walletIndex`（同钱包多账户共 KEK、各账户
  独立信封），仅允许 strict biometricOnly；不读取旧键。
- **wallet_manager**:createWallet/importWallet 只存账户0 child、母种子 finally 清零、助记词一次性返回不落库;`signWithWallet`/`_loadSigningKey` 读 child(生物识别)+校验+清零;**删 `_selfHealSeedFromMnemonic`/`_readSeedHexWithSelfHeal`/`getMnemonic`**(无根无自愈,失效/缺失→`WalletAuthException` 重导入 fail-closed);`getSeedHex`→账户0 child;contact key/`_registerDeviceSubkey` 从 child;`isUsableHotWallet` 用 `hasAccountKey`;clear/delete/rollback 用 `deleteAccountKey`。
- **UI**:`wallet_page.dart` 删「查看助记词」菜单(唯一 `getMnemonic` 调用方);「查看私钥」留。
- **三条安全不变量**(逐条核实):①不存种子(母种子仅内存、finally 清零)②不存助记词(无存储层)③签名/读密钥强制 strict 生物识别、取消/失效/缺失 fail-closed。
- **门禁**:`dart analyze` **No issues** + `flutter test` **813 passed / 5 skipped / 0 failed**(本人独立复跑 `FLUTTER_EXIT=0`,非仅信 agent);残留扫 0;金标 spike 全绿。
- **范围收敛**:S7.1 保单账户 WalletProfile(不动 app_isar),AccountEntity/多账户 → S7.2。

## S7.2 落地(2026-07-27,完成 ✅)
**citizenapp 多账户 + 单钱包(批量指定序号),主检出 `citizenapp/`,冷钱包/§6 调用方零改:**
- **数据模型**:`app_isar.dart` 新增 `AccountEntity`(masterId indexed / accountIndex / accountId unique / ss58Address unique / accountName / createdAtMillis)+ `WalletProfileEntity` 加 `masterId`(=账户0 accountId);account0 建钱包时同写一条 AccountEntity;`AccountEntitySchema` 注册;`build_runner` 重生 `app_isar.g.dart`。
- **多账户 API**(`wallet_manager.dart`):`getAccounts(masterId)`/`getAccountByAccountId`;**`addAccounts(masterId, mnemonic, List<int> indices)`**(归属校验 //0==masterId → 序号 `[1,1989]`+输入去重+不与既有冲突 → 逐个派生 → **批量原子**:一事务写全 AccountEntity + 逐个 putAccountKey,任一失败整批回滚删行+deleteAccountKey → 母种子清零);`addNextAccount`(max+1);`getAccountPrivateKey(accountId)`(读该账户 child,生物识别);`signForAccount(accountId, payload)`(多账户签名,校验公钥+清零);`deleteAccount(accountId)`(账户0 锚点守卫:有兄弟拒单删,删至空级联删钱包);**单钱包约束**(已有热钱包拒建/拒导,`本设备已存在热钱包...`)。默认身份/`signWithWallet(walletIndex)` 仍账户0(interim,§6 零改)。
- **测试**:新增 `test/wallet/wallet_multi_account_test.dart`(11 用例:连续 `[1,2,3]`/断续 `[1,5,9]` 存各自 child、去重/越界/已存在/归属拒、addNextAccount、getAccountPrivateKey、signForAccount 签名可验、deleteAccount 锚点+级联);修 `wallet_manager_reorder_test.dart` fixture 缺 masterId。
- **门禁**:`dart analyze` **No issues** + `flutter test` **824 passed / 5 skipped / 0 failed**(本人独立复跑 `FLUTTER_EXIT=0`);残留扫 0。
- **执行插曲**:实现 agent 因 API 连接中断中途失败(停在写测试前),留 2 处 reorder fixture 缺 masterId(LateInitializationError)+ 多账户测试未写;**本人接手**:补 fixture masterId、写 11 条多账户测试、独立复核批量原子/锚点/契约后转绿。

## S7.3 落地(2026-07-27,完成 ✅)—— S7 钱包层收官
**citizenapp 多账户钱包 UI(纯 UI,复用 S7.1/S7.2 API,冷钱包/§6 零改):**
- `wallet_page.dart`:「＋」入口 + 空态**删「创建钱包」「导入热钱包」**,只留「导入冷钱包」(onboarding 仍是首启唯一热钱包引导);我的钱包列表**热账户 + 冷钱包共存**;账户点击进详情。
- 新 `lib/wallet/pages/account_detail_page.dart`:账户 SS58 + **私钥展示**(默认隐藏→确认→`getAccountPrivateKey` 生物识别→`0x`+64hex 纯 Text + `ScreenshotGuard`);顶部「添加账户」。
- 新 `lib/wallet/widgets/add_account_sheet.dart`:两模式「添加下一个」/「指定序号」;**多序号空格分隔**(`split(RegExp(r'\s+'))`,连续 `1 2 3`/断续 `1 5 9`)→ `addAccounts`;助记词重录(无根);业务校验单源交 `addAccounts` 兜底不抄两份。
- `create_wallet_flow.dart`:备份文案改「手抄助记词或在公民钱包保管」。
- widget 测试 `test/wallet/pages/wallet_multi_account_ui_test.dart`(入口已删/账户列表/序号解析/私钥默认隐藏/冷钱包共存)。
- **执行插曲**:agent 完成实现但收尾测试未跑;本人复核发现 1 条「1 5 9 端到端 UI→addAccounts→读 isar」widget 用例**在 testWidgets fake-async 下卡 10 分钟超时**(经 UI 触发的真实 isar 磁盘异步不可驱动);其覆盖已由 `parseAccountIndices('1 5 9')` 解析单测 + `wallet_multi_account_test` 的 `addAccounts([1,5,9])` 落库单测全覆盖,故将该脆弱用例改为**不碰 isar 的轻量渲染测试**(渲染需 `Scaffold` 供 Material)。
- **门禁**:`dart analyze` **No issues** + `flutter test` **838 passed / 5 skipped / 0 failed**(本人独立 `FLUTTER_EXIT=0`);残留扫 0。

## S7 钱包层收官(2026-07-27)
S7.1(无根派生+存储 biometricOnly)+ S7.2(多账户批量+单钱包)+ S7.3(多账户 UI)全部完成并全绿。citizenapp = 一设备一钱包多 `//index` 账户、只存 child mini-secret(不存种子/助记词)、biometricOnly fail-closed;冷钱包共存;§6 身份/签名调用方过渡解析账户0。**下一步 S8 身份层**(CID 主键 + 注册接 `self_occupy_cid` + 门控,依赖另一会话 S4+S6)。

## S8.1 落地(2026-07-27,完成 ✅)—— 客户端 CID 生成 + 占号/换绑 RPC(纯离线编码,逐字节镜像链)
**citizenapp 侧新增两只离线原语 + 两条 extrinsic 构造,主检出 `citizenapp/`,不依赖 S4+S6、不碰 UI:**
- **CID 生成器**(新 `lib/citizen/cid/cid_generator.dart`):`generateCitizenCid({accountId, institution, year})` **逐字节移植链 `generator.rs` 人主体分支**——`input="{accountId}|{institution}|{year}"` → `blake2_256` 前 8 字节**小端 u64**(BigInt 承载防 Dart 有符号溢出)→ `mod 1e12` → `high3=/1e9`(R5=`CN`+高3补零)`low9=%1e9`(N9 补零)→ M1 校验位 `'0'+(Σ(i+1)*alphabetIndex % 10)`(字母表外→0,对齐 `number.rs`)→ `"{R5}-{CTZN/NATP}{M1}-{N9}-{year}"`。仅 CTZN/NATP,`blake2b256` 用 polkadart `Hasher.blake2b256`(纯,无 key)。
- **占号/换绑 RPC**(新 `lib/rpc/citizen_identity_rpc.dart`):`CitizenIdentityRpc`,两条自签自付 extrinsic,复用 `SignedExtrinsicBuilder`(immortal/自动 nonce·specVersion·genesisHash)+ `signWithWallet`:
  - `selfOccupyCid`:callData=`[10][5]` ++ `compact(len)++cid.utf8`(CidNumberBound=`BoundedVec<u8,ConstU32<32>>`)。
  - `selfRebindCidAccount`:callData=`[10][9]` ++ CID BoundedVec ++ `compact(64)++64B`(SignatureOf=`BoundedVec<u8>`);旧账户授权摘要 `buildRebindSigningDigest`=`signingMessage(0x11, compact(len(cid))++cid.utf8++new_account(32B))`=`blake2_256(GMB++0x11++payload)`,**逐字节 == 链 `(cid,new_account).encode()` + `verify_rebind_signature`**;旧账户从链上 `AccountIdByCid[cid]` 反查、不上送,`signForAccount(oldAccountId, digest)` 出授权签名。
- **登记**:`pallet_registry.dart` 加 `citizenIdentityPallet=10`/`selfOccupyCidCall=5`/`selfRebindCidAccountCall=9`;`signer/signing.dart` 加 `kOpSignCidRebind=0x11`(`kGmbSignDomain=[0x47,0x4D,0x42]`="GMB" 已核 == 链 `core_const::GMB`)。
- **逐字节四验(本人独立复核,非仅信 agent)**:①CID 金标 `CN951-CTZN1-539598435-2026`(dev //0 公钥,== 链 `citizen_cid_number_golden`)②occupy callData 字节序 ③rebind 摘要 = `signing_message(0x11,…)` 对齐链 verify ④算法读源逐行核对 generator/number.rs。
- **测试**:新 `test/citizen/cid/cid_generator_golden_test.dart`(金标 + CTZN/NATP + 非法码/年份拒)+ `test/rpc/citizen_identity_rpc_test.dart`(callData 字节 + rebind 摘要 + 长度守卫 + accountId 校验)。
- **门禁**:`dart analyze` **No issues** + `flutter test` **848 passed / 5 skipped / 0 failed**(本人独立 `FLUTTER_EXIT=0`);残留扫 0。
- **范围收敛**:S8.1 只出**离线可测的生成/编码/提交原语**,不碰身份页 UI(S8.2)、不碰主键切换/门控(S8.3);上链真跑待 S5 重新创世 + S4+S6 注册局流程合流。

## S8.2 落地(2026-07-27,完成 ✅)—— 身份页改造 + 匿名占号注册/换绑全链路
**citizenapp 身份页(原电子护照)接入 S8.1 离线原语,仅主检出 `citizenapp/`,不动底部 5-tab、冷钱包/§6 零改:**
- **链读扩展匿名态**(`my/myid/citizen_identity_chain_reader.dart`):`CitizenIdentityChainSnapshot.votingIdentity` 改**可空** + `isAnonymous` getter;`readByAccountId` 三态分流——`CidByAccountId` 无/绑定不闭环→`null`(纯访客);闭环但无 `VotingIdentityByCid`(或布局损坏,**安全降级不误升**)→匿名快照(仅 cidNumber);闭环+合法 voting→投票/竞选。**关键**:旧 reader 无 voting 即返回 null,把匿名 CID 与纯访客混同,故必扩。
- **状态分流 + 动作**(`my/myid/myid_service.dart`):**不新增 tier/色**(决策①),匿名态 = `visitor` + 填 `cidNumber` + `isAnonymousRegistered` getter;`getState` 四分流(纯访客/匿名/投票/竞选);新增 `registerAnonymousCid`(取默认用户→`generateCitizenCid`(**UTC 当前年** `cidYearProvider`)→`selfOccupyCid`)、`rebindCidTo`(旧=默认用户、新=所选,`selfRebindCidAccount`;目标==自身拒)、`listRebindTargets`(默认用户外全部账户);注入 `identityRpc`/`cidYearProvider` 可测。
- **RPC 修正**(`rpc/citizen_identity_rpc.dart`):**修 S8.1 接入隐患**——`selfOccupyCid`/`selfRebindCidAccount` 由 `signWithWallet(walletIndex)`(interim 恒解析账户0)改 **`signForAccount(accountId)`**(按 accountId 精确取私钥),删 `walletIndex`/`signerPublicKey` 冗余参;否则换绑非0新账户会被账户0私钥签、与声称发送方不符致链上验签必败。static `buildXxx` 编码未动(金标不变)。
- **UI**(`my/myid/myid_page.dart` + `my/user/user.dart` 入口):**改名**电子护照→身份(AppBar/入口/错误文案/注释,UI 残留清零,仅留 1 行历史说明);首卡 `访客轻节点`→`注册身份·访客`(匿名图标留),匿名态卡内补显 `身份CID号 xxxxx`;voting/candidate 卡 `公民CID号`→`身份CID号`;右上 icon 刷新→语义 `注册`/`更换` 按钮(纯访客注册、匿名/投票/竞选更换,civic 点击提示走注册局,queryFailed 无按钮),`RefreshIndicator` 下拉刷新;新 `widgets/register_identity_sheet.dart`(选 CTZN/NATP+风险提示)、`widgets/rebind_account_sheet.dart`(选目标账户,空态提示)。
- **初始化跳身份页**(`wallet/wallet_gate.dart`):加可注入 `onInitialized`(默认 `push(MyIdPage)`);`onCreated`(本次会话新建/导入,子路由已 pop 干净)后 `postFrame` 引导一次,**冷启动即有钱包不跳**;盖广场 tab 之上、返回回落,**不动 5-tab**(决策③)。
- **下游适配**(`8964/chain/square_chain_service.dart`):`votingIdentity` 可空后,匿名态在广场身份判定归 `visitor`(不升级徽章,合决策①)。
- **测试**:service +7(匿名态解码/纯访客区分/注册金标编排/换绑参数/目标==自身拒/listRebindTargets)、widget +8(改名/注册·更换按钮/匿名 CID 展示/下拉刷新/字段文案,`descendant` 限定卡内避非当前卡字段名干扰)、gate +2(引导触发/冷启动不触发,注入 no-op 挡真链)、两 sheet +5。
- **门禁**:`dart analyze` **No issues** + `flutter test` **867 passed / 5 skipped / 0 failed**(本人独立 `FLUTTER_EXIT=0`);旧文案(电子护照/访客轻节点/公民CID号)UI+注释残留清零。
- **范围收敛**:S8.2 身份页锚点仍 `getDefaultWallet()`(账户0 interim);**换绑后身份归新账户但身份页读账户0 会暂显访客**——身份页跟随 CID-bound account 是 S8.3;上链真跑待 S5 重新创世 + S4+S6 合流。

## S8.3a 落地(2026-07-27,完成 ✅)—— 身份账户单源 + 注册选绑定账户
**把身份页数据源 + 注册/换绑收敛到「CID 绑定账户」(非恒账户0),仅主检出 `citizenapp/`:**
- **身份账户单源**(新 `lib/my/myid/identity_account_resolver.dart`):`IdentityAccountResolver.resolve()` 返回 `ResolvedIdentity{accountId,ss58,accountIndex,snapshot}`——**优先账户0**(命中即 1 次链读)、未命中遍历子账户、首个 `readByAccountId` 闭环命中者即身份账户;全未命中→回退账户0(`isRegistered=false` 未注册);无热钱包→null;**链读异常上抛不吞**(fail-closed)。
- **身份绑定通知**(`wallet_manager.dart`):`notifyIdentityBindingChanged()` 复用 `walletsRevision`(占号/换绑改身份账户但钱包列表没变,须显式广播常驻页重读);注释同步纳入 CID 占号/换绑。
- **MyIdService 收敛**:`getState` 改走 `resolve()`(身份从 getDefaultWallet账户0 → CID 绑定账户,`votingAccountId`/徽章快照全锚身份账户);`registerAnonymousCid({institution, bindAccountId})` **注册时选绑定账户**(默认账户0,可选子账户,用所选账户生成 CID+`self_occupy_cid`+成功后通知);`rebindCidTo` 旧账户改用 `resolve()` 解析当前 CID 绑定账户(非恒账户0)+成功通知;`listRebindTargets` 排除身份账户;新 `listBindableAccounts`;移除对 `CitizenIdentityChainReader` 的直接持有。
- **UI**:`register_identity_sheet` 加「绑定钱包账户」单选(多账户才露出,默认账户0)+ 返回 `RegisterChoice{institution,bindAccountId}`;`myid_page._onRegister` 先 `listBindableAccounts` 再弹面板。
- **注释同步**:myid 层「默认用户/getDefaultWallet 账户0」过时表述改「身份账户/CID 绑定账户」(comment rot 清理);chat/square/user/wallet 层「默认用户」残留留 S8.3b/S8.3d。
- **测试**:新 `identity_account_resolver_test.dart`(账户0命中/子账户//5命中/全未命中回退/无钱包null/链读异常上抛)5 + service 扩展(注册绑子账户//5非账户0、listBindableAccounts)+ sheet 多账户选择;换绑/列举目标测试注入 `_FakeIdentityResolver` 绕真链序列。
- **门禁**:`dart analyze` **No issues** + `flutter test` **875 passed / 5 skipped / 0 failed**(本人独立 `FLUTTER_EXIT=0`)。
- **范围收敛**:S8.3a 只切身份页数据源 + 注册 + 单源基建;**其他身份展示调用方(chat/square/user)+ 设备子钥/广场会话/通讯录密钥迁移 = S8.3b**——常态身份=账户0 时无差异,换绑到子账户的全端一致性待 S8.3b。上链真跑待 S8.3e 本地链。

## S8.3b-1 落地(2026-07-27,完成 ✅)—— 身份账户缓存基建 + 广场身份
- **`identity_account_cache.dart`**:`IdentityAccountCache` 给 chat/广场/会话等高频调用方
  提供身份账户低成本入口；第 2 步已删除账户0乐观回退，链读失败、禁止链读时缓存未命中
  和绑定闭环不完整全部返回失败，不能把钱包账户误当成 CID 当前授权账户。
- **`square_identity_state`**:广场身份 `accountId`/`ss58`/`cid` 跟随身份账户,`walletIndex`(设备子钥锚点)/`walletName` 保持钱包级;`square_home_page_test` setUp 注入 `_NullIdentityCache`(回退 wallet.accountId,行为不变)。
- 门禁:`dart analyze` No issues + `flutter test` **883 绿**。
- **⚠️ 子步组织调整(grounding 后)**:切广场身份时确认「**本人 accountId」全 App 统一**(getDefaultWallet().accountId 同喂 身份展示/chat会话/广场会话/设备子钥绑定/社交签名),**纯展示与会话/签名无法干净拆**;且 `ensureSessionFor` 被「钱包名同步」复用、设备子钥按 walletIndex 绑 accountId、后端 device/register 证明归属——切身份账户须连带**设备子钥重绑**。故原 b-1/b-2「展示/会话」拆分作废,**S8.3b-2 重定义 = 本人 accountId 统一切换 + 设备子钥/会话/社交签名同步(自洽单元)**,开工前补 grounding 会话/设备子钥完整链路。
- **最终原则**：CID 是唯一身份主键，`getDefaultWallet` 只保留钱包元数据用途；
  全 App 当前授权账户由 finalized CID 绑定解析，失败关闭。业务数据和用途子钥归 CID，
  当前账户仅负责签名、鉴权、付款与包装数据根；换绑不搬运 CID 数据。

## S8.3b-2 落地(2026-07-28,完成 ✅)—— 本人 accountId 全 App 彻底切身份账户
**彻底改造(不半切/不留账户0):正交轴 accountId→身份账户、walletIndex/ss58/walletName/设备子钥存储→钱包级。全量 883 绿 + analyze clean。**
- **基建**:`identity_account_cache.dart` `IdentityAccountCache`(walletsRevision 失效 + 并发合并 + `allowChainRead=false` 广场浏览不启 smoldot + `debugInstance` 测试覆盖 + 8 单测);后续第 2 步已把“链读失败时回退账户0”彻底改为失败关闭。`WalletManager.rebindDeviceSubkeyToAccountId`(换绑用:P-256 子钥不动、身份账户 child 重签绑定证明、后端归属身份账户)。
- **步1 会话解耦**:`ensureSession()` accountId=身份账户、签名用钱包 walletIndex 子钥(解耦),不复用 `ensureSessionFor`(留钱包名同步)。
- **步2 会话+展示+chat 页**:8 文件本人 accountId 切身份账户(user.dart 同步 getter→异步 state 重构 + 徽章快照口径收口身份账户、`_refreshIdentityAfterChainOperational` 身份 dedupe+钱包级再入守卫修 //n 徽章不刷新 bug;user_profile/square_home 比对键;4 chat 页 self accountId);4 测试注 `_NullIdentityCache`。
- **步3 社交签名**:5 service(subscription×3/creator_subscribe×3/creator/compose/注销)链上交易三件套切 `signForAccountId(身份账户)`;`_requireIdentity()` 一次 resolve;修 creator `session(身份)`vs`wallet(账户0)` 误拦 //n latent bug。
- **步4 chat 运行态**:`chat_runtime.readAccountId`/`_readAccount` 切身份账户、walletIndex/walletName 钱包级、`expectedAccountId` 校验改身份账户;`ChatRuntime` 加 `IdentityAccountCache` 注入。
- **通讯录鉴权切换**：本人签名账户统一取 CID finalized 当前绑定；第 2 步进一步把本地与
  云端通讯录子钥改为 CID 稳定数据根派生，账户 child 不再直接决定通讯录密钥。
- **剩余**：S8.3b-4 已由 finalized 当前账户接管最终实现覆盖；S8.3c/S8.3d 已完成，
  S8.3e 留待正式创世前整体验收。

## S8.3b-4 finalized 当前账户接管（2026-07-29，最终实现 ✅）

- 自主换绑在 runtime 内由当前绑定账户授权、新账户提交；注册局换绑由注册局权限和新账户
  控制证明授权。链上 finalized 后，两条路径都只认 CID 的当前新账户。
- 接管编排为：新账户挑战签名 → 取得同一 CID 数据根 → 新包装读回校验 → 派生用途子钥
  → 设备子钥用新账户重签登记 → 广播绑定变化 → 写四元组完成标记。
- 冷启动按 finalized `cid_number + binding_revision + account_id` 与数据根摘要对账；
  任一步失败不写完成标记，下次幂等重试。不存在旧账户参与的接管意图、通讯录搬运或
  旧账户重封装。
- 业务关系、聊天、动态、文章、粉丝、会员和通讯录均继续归 CID；旧账户仅能作为完成
  接管后的账户级缓存/包装删除目标。

## S8.3c 全功能门控 CID(2026-07-28,完成 ✅)—— 访客无匿名可用面
**未注册 CID 时聊天/广场/订阅/创作者/通讯录五大功能硬门引导注册。分块跑 891 全绿(块A 8964+chat 282 / 块B 其余 609)+ analyze clean。**
- **现状(grounding)**:五大功能原先无一按 CID 门控,唯一强门=`WalletGate`(有热钱包);四层门禁 [[citizenapp-four-gate-entry-failclosed]] 全在服务端会话/发布/上传层,**无一查 CID**——CID 注册门是正交新门,门在功能入口层。
- **判据单源**:`IdentityAccountResolver.resolve()` 的 `ResolvedIdentity.isRegistered`(占任意 CID 即放行,匿名占号即过=CID 门非投票身份门),**严格链读路径**;链读失败→queryFailed 提示重试(fail-closed,不放行不误判),**不用** `IdentityAccountCache`(乐观回退会误判未注册)。
- **统一门组件** `lib/my/myid/widgets/identity_registration_gate.dart`(照 `CreatorGateView` 样式):四态 loading/registered(放行 child)/unregistered(引导「去注册身份」→跳唯一注册页 `MyIdPage`)/queryFailed(重试)+ noWallet;监听 `WalletManager.walletsRevision` 注册/换绑广播即自动重判放行。
- **五大入口接入**:聊天 `chat_tab`(包 ColoredBox 内)、广场 `square_home_page`(包 Scaffold 外,未注册无 FAB)、订阅 `membership_page`(删「无身份门槛」包 body)、创作者 `creator_page`(CID 门前置会员门)、通讯录 `contact_book_page`(包 body 留 AppBar);push 子页均保留 AppBar 返回。
- **测试**:gate 单测 5 绿(三态+无钱包+注册广播重跑);gate 加 `@visibleForTesting debugResolver` 全局注入点 + 共享 helper `test/support/identity_gate_test_util.dart` `useRegisteredIdentityGate()`,4 个 gated 页面测试(square/chat/membership/contact)注入放行避免真 smoldot 链读。
- **环境注**:满并发全 suite 会 OOM(SIGKILL)/超 12min 全局超时;分块 `--concurrency=4` 跑取干净信号(非代码问题,隔离复跑全绿)。
- **剩余**:~~S8.3d 删默认用户~~(✅ 见下)/ S8.3e 本地开发链 e2e。

## S8.3d 删默认用户(2026-07-28,完成 ✅)—— 概念/徽标/措辞零残留
**身份主键已 CID 化,`getDefaultWallet()` 退为纯钱包访问入口;删「默认用户=身份」的徽标+概念+措辞。分块跑 884 全绿(块A 282/块B 602)+ analyze clean;全仓「默认用户」0 残留。**
- **核心删除(`wallet_page.dart`)**:`_WalletBadge(label:'默认用户')` ×2(钱包卡+账户行)+ `isDefault` 字段/参数(两 tile)+ 计算(`walletIndex==defaultWalletIndex`/`accountIndex==0`)+ `defaultUserWalletIndex()` helper(徽标专用,连带死);条件 `if(isIdentityWallet||isDefault)`→`if(isIdentityWallet)`。**保留** `isIdentityWallet`→「身份钱包」徽标(已准确标 CID 绑定账户,verified 图标,用 `_identityAccountId`)。
- **措辞更新(注释-only,8 文件零行为)**:onchain_payment/chat_tab/chat_runtime/square_home_page/user.dart/contact_book_page/contact_service「默认用户(钱包)」→「身份账户」;`wallet_manager.getDefaultWallet` 文档删「统一身份来源」改「默认热钱包=钱包访问入口,已退出身份主键角色,身份账户经 IdentityAccountResolver 解析」。
- **测试**:删 `wallet_list_tile_test` 的 `defaultUserWalletIndex` 组(4)+ 徽标测试(2)+ `pumpTile` isDefault 参数;删 `wallet_multi_account_ui_test` 徽标测试(1)+ isDefault 用法;7 处测试名/注释「默认用户」→账户0/身份账户。891−7=884。
- **范围纪律**:[[no-scope-expansion]] 字面删「默认用户」;**未**新增账户级「身份账户」徽标(账户可视化归 MyIdPage)。`getDefaultWallet()` 本身保留(钱包元数据入口)。
- **注**:wallet_manager `_contactKeyStore`/`ContactKeyMaterial` 文档仍写「账户0 child」(S8.3b-2 后实为身份账户 child),属 S8.3b 注释精度遗留、非本步范围,未动。

## S8.3e 本地开发链 e2e —— 决策:并入 S5 重新创世合流(2026-07-28,用户裁定)
**S8.3 代码四子步(a/b/c/d)全部完成、单测全绿;S8.3e 链上真跑验证不单独起本地链,顺延到 S5 全网重新创世事件一次性验证。**
- **理由**:e2e 需活链+真 app(smoldot native 不在测试环境,非 `flutter test` 可覆盖);创世公民无 GenesisConfig 不可用须现场新占 CID;已绑 S5 重新创世,另一会话(modelb)正做创世管理员重派+注册表重生([[regenesis-deploy]]),不宜各起各的链。
- **正式创世前验证清单**：App 连唯一候选创世链走
  **新建钱包→自助占号→匿名已注册→五大功能门控解锁→自主换绑当前账户授权+
  新账户提交→finalized 新账户独立接管数据根/设备/会话→CID 数据连续可读** 全链路；
  另走注册局换绑验证无需旧账户签名。
- **前提校正**:创世公民法代 `CN220-CTZN2-…`/账户 `0x0cb1d05c…4b06b` 在 citizen-identity 无 GenesisConfig=链上纯访客,不能用于验证;须现场占新 CID。

## S8.3 收口(2026-07-28)—— 身份主键 CID 化代码完成
- **S8.3a** 身份账户单源+注册选账户 ✅ / **S8.3b** finalized 当前账户鉴权与 CID
  数据根接管 ✅ / **S8.3c** 全功能门控 CID ✅ / **S8.3d** 删默认用户 ✅ /
  **S8.3e** 链上真跑留待正式创世前整体验收。
- **全系不变量**：非链功能唯一身份主键是 CID；当前 `account_id` 只负责签名、鉴权、
  付款和包装数据根。业务用途钥从 CID 稳定根派生，换绑不迁移 CID 数据。

## S8.3 审计整改(2026-07-28,完成 ✅)—— 3 审计 agent(换绑安全/门控/命名注释)发现 2 CRITICAL+4 HIGH,全修
**client 分块 892 绿(块A 282/块B 610)+ worker vitest 179 绿 + 双端 analyze/tsc clean + 0 残留。**
- **C1(CRITICAL)换绑后旧账户云端凭证失效**：2026-07-29 最终方案以 finalized
  `AccountIdByCid + CidByAccountId + BindingRevisionByCid` 作为唯一控制权转移事实。
  客户端旧账户签名 outbox、换绑后吊销 API 和专用 revoker 已删除，不得恢复第二授权协议。
  App 先由 finalized 当前新账户恢复同一 CID 数据根、验证新包装并安装用途子钥，再登记
  新钱包设备子钥；Worker 确认新子钥落库后才删除旧账户级登录/设备/Chat 凭证并关闭旧
  实时连接。动态、文章、粉丝、关注、通讯录、会员和 CID 稳定数据根继续归 CID，
  不删除、不迁移。
- **C2(CRITICAL)Finalized 后崩溃窗口**：最终改为 finalized 四元组对账。
  `IdentitySyncedAccountStore` 只在数据根安装、设备登记和广播全部成功后写
  `(cid_number, binding_revision, account_id, data_root_hash)`；未完成则由
  `reconcileFinalizedBindingTakeover` 幂等补齐。
- **H1 通讯录扫码绕过门**:AppBar 扫码键在 gate 外(未注册可写联系人)。**修**:contact_book/membership 整 Scaffold 包 gate(新 `scaffoldTitle` 未放行态带返回键)。
- **H2 gate 冷启动卡死**:gate 只听 walletsRevision,广场落地页 smoldot 未就绪→queryFailed 卡死。**修**:gate 加 `healthListenable` 监听,未就绪停 loading、operational 自动重判。
- **H3 广播顺序**：只在当前新账户已安装并验证 CID 数据根、派生用途子钥和登记设备
  之后广播绑定变化；不存在账户间通讯录搬运。
- **H4 观测**:对账式使每次 getState 自愈重试(不再永久静默),reconcile 失败区分日志。
- **M1 并发锁**：`_runBindingTakeover` 使用 `_takeoverInflight` 去重直接触发与冷启动
  对账并发。**M2** gate debugResolver 加 `kReleaseMode` 硬忽略。**M3** `_reresolve`
  加代际号丢弃旧响应。**M5** 会员刷新键随整 Scaffold 包 gate。
- **注释 A1-A6**:wallet_manager signWithWallet/signForAccountId 文档身份签名路由订正、_deriveContactKeys/_contactKeyStore/ContactKeyMaterial「账户0 child」→身份账户 child、wallet_page:912「默认/身份徽标」→身份徽标、verifyWalletAccess 文档删已删概念(方法保守保留)。
- **后续审计订正**：客户端旧账户安全 outbox 与换绑后吊销 API 均已删除；旧账户凭证
  失效只由 finalized 当前绑定校验和后续可信事件消费者处理，不形成第二授权真源。

## S8.4 通讯录 CID 真源与昵称/备注分离（2026-07-28，完成 ✅）

- **模型彻底切换**：`UserContact` 固定为 `cid_number + account_id + ss58_address + contact_remark + created_at + updated_at`；CID 必填且是关系主键，私人备注允许空值，公开昵称/头像/签名不复制进通讯录。旧 `contact_name` JSON 不兼容。
- **索引与加密契约**：增删改、待同步、跨端冲突合并全部按 CID；
  `contact_id = HMAC(index_key, target cid_number)`。本地/云端密钥改由 CID 稳定数据根
  的 `contacts-local` / `contacts-cloud` 用途域派生；换绑后密钥不变。
- **扫码入口**：本阶段先在入库前按二维码 SS58 派生 `account_id`，再经链上双向绑定
  读取永久 CID；随后 `20260728-citizenapp-nickname-contact-profile-qr` 已把
  `QR_V1/k=3 user_contact` 四端统一为
  `cid_number + ss58_address + display_name`。未绑定或声明 CID 不一致均拒绝；
  收款码不再兼作联系人码，二维码公开昵称不写入私人备注。
- **换绑行为**：关系与云端密文归属 CID，不分页删除、不重新上传、不搬到新账户；
  finalized 后当前新账户接管同一数据根即可继续读取。
- **第 3 步已完成**：本地 KV、Chat/MLS/附件、草稿、通讯录、机构关注、身份徽章和
  finalized 镜像均按 owner CID 分区；当前账户只承担签名、鉴权、付款或不可变交易事实。
- **UI 口径**：通讯录卡片公开昵称为主标题；`备注：`、`CID：`、`SS58：` 分行展示；修改动作只编辑私人备注。资料页和公开资料缓存直接按联系人 CID 寻址，不再对已有联系人重复做 account→CID 链读。

## 待办 / 未决
- **S5 创世协同**：程伟永久 CID
  `CN220-CTZN2-198805200-2026` 已作为身份主键固定保留，不再按当前授权账户重派；
  `citizen-identity` 创世配置现已原子写入 Active 登记、CID→AccountId 与
  AccountId→CID 三项闭环。剩余的是使用同一份 CI WASM 重生正式 chainspec、
  CitizenApp 轻链资产、公权机构快照与 Cloudflare 创世哈希；禁止各端自行生成不同
  创世锚点。
- **后置测试**：正式资产重生后再执行自助占号、finalized 换绑接管、五大功能门控和真实管理员
  QR 会话闭环；当前不以本地 fresh 链结果冒充正式发布验收。
- **独立任务**：联系人、聊天正文、MLS 密钥和附件的本地静止态加密归
  `20260728-citizenapp-chat-local-data-encryption`，不并入本卡。

## 验收
冷热链逐字节一致(citizenapp `//0//1//2` == Step 1 金标 == 链读);CID CN000 金标钉死;自助占号/换绑链上闭环;门控 fail-closed;全端 analyze/test 绿;残留复扫 0。

## S5 创世管理员身份闭环（2026-07-29，完成）

- `citizen-identity` 新增 `initial_cid_bindings` 创世配置；每项固定为
  `(cid_number, account_id, registrar_cid_number)`，创世阶段原子写入
  `CidRegistry`、`AccountIdByCid`、`CidByAccountId`，重复 CID、重复 AccountId 或
  空登记来源直接拒绝烤链。
- 程伟永久 CID 保持 `CN220-CTZN2-198805200-2026`，授权账户保持
  `0x0cb1d05c0c9c7f05679b60d6f24c7e5719a3985264e41c5e899d4822dca4b06b`；
  登记来源复用联邦注册局 `ZS001-FRG07-249474503-2026`。只建立匿名 Active
  身份闭环，不生成 VotingIdentity 或 CandidateIdentity。
- `citizen-identity` 全量 62 项、runtime 全量 52 项通过；当前源码
  `WASM_BUILD_FROM_SOURCE=1 cargo build -p node` 成功，证明 `no_std` WASM 与原生
  节点均能编译。
- 当前源码 fresh 链真实创世哈希为
  `0xb8a32949a2d0527e3522d9db315049de7b6ecbd3adee33979b7d5df795d79d6a`。
  RPC 逐字节读取确认三张 storage 正反闭环、登记状态 Active、注册高度 0、
  `commitment=blake2_256(account_id)`、居住省市为空且无吊销。
- fresh 链进程已停止，RPC 19944 已关闭；临时链目录从 `/tmp` 移入 macOS 废纸篓，
  源路径不存在且仍可恢复。

## S6 奖励、投票、候选人与竞选发布安全收口（2026-07-30，完成）

- 公民认证奖励的唯一身份键改为完整规范 `cid_number`，账户保留为第二防重键和收款账户。
  `IdentityRewardClaimed` / `PendingIdentityRewardClaimed` 直接使用 CID；
  `AccountRewarded` / `PendingAccountRewarded` 保持不变。任一 CID 或账户已有永久/本块记录
  即拒绝，只有新 CID + 新账户才可发放；没有 CID 不会触发身份登记回调。
- NodeGuard 使用同一完整 CID SCALE/RAW key 复核连续队列、投票身份首次出现、CID↔账户
  双向绑定和 CID/账户两套永久及临时标记。专项测试明确覆盖 CID 已领但换新账户、账户已领
  但换新 CID、同块 CID 重复和同块账户重复。
- 创世数量断言补齐公私权边界：49,593 个机构 / 99,232 个协议账户仅指公权目录；
  私权公民链基金会另计 1 个机构和主、费 2 个协议账户，因此全创世为 49,594 个机构 /
  99,234 个机构协议账户；管理员个人钱包不计入该账户数。
- Popular 公民投票继续按 `(proposal_id, voter cid_number)` 防重；候选快照和结果保存
  `CitizenSubject`，候选计票表以候选 CID 为唯一键。候选人必须通过
  `candidate_subject`，仅有普通投票身份不能进入候选快照。
- Mutual 机构互选继续使用 `institution cid_number + role_code + 当前管理员 account_id`
  的岗位票据；管理员记录唯一布局保持
  `account_id + cid_number + family_name + given_name`，没有恢复账户即权限或机构全体
  admins 快照。
- `SquarePost::publish_post` 的 `campaign` 类别改为 runtime、CitizenApp、Worker 三层
  统一要求竞选身份；仅有投票身份或匿名 CID 均拒绝。runtime 错误统一为
  `CampaignRequiresCandidateIdentity`。
- NodeGuard 将 `CandidateIdentityByCid` 纳入 RAW 生命周期保护：候选身份必须依附 Active
  CID 与 Normal 投票身份；出生地区、性别和出生日期不可改；姓名可依法更新；删除只允许
  CID 与投票身份一并进入吊销终态。
- 使用当前源码 WASM 和 FRAME Benchmark CLI 53.0.0、50 steps、20 repeats 重算
  `citizen-issuance`、`citizen-identity`、`election-vote` 权重；`square-post::publish_post`
  实测为 8 reads / 2 writes / proof 3,834 bytes，并已把现有 benchmark 挂入 runtime 注册表。
- 回归结果：`citizen-issuance` 16 项单测 + 7 项集成、发行 NodeGuard 9 项、
  候选生命周期 NodeGuard 6 项、`election-vote` 17 项、`square-post` 27 项、
  runtime 52 项全部通过；生产 release Node 从当前源码与 WASM 构建成功。
- 临时 `citizenchain-fresh --tmp` 真实启动并通过 NodeGuard 自检；RPC 为 0 peers、
  `isSyncing=false`、best/finalized #0，六项项目 runtime 版本均为 0。block #0 为
  `0x6d2d148acba895a197e2be4eb9aa6df95b6f66036fd43556e2e004059bea1734`，
  state root 为
  `0xe3a5c53f1a834169cec9800a19825f6f3abf709862ee8531ba140fa2d9c963df`，
  metadata 为 228,802 字节；节点已停止，RPC 19944 已关闭。
- 本步没有执行正式创世、冻结、GitHub 推送、WASM CI、其它软件 CI 或部署。下一步必须先
  输出正式创世与首次冻结技术方案，取得用户确认和 runtime 二次确认后才能执行。

## S6.1 创世前格式残留清零（2026-07-30，完成）

- 对 S6 改动执行全工作区 `cargo fmt --all`，只产生机械格式调整：
  `onchina/src/auth/actions.rs`、`onchina/src/auth/login/onchain_gate.rs`、
  `onchina/src/auth/repo.rs`、`onchina/src/core/runtime_ops.rs`、
  `runtime/misc/pow-difficulty/src/lib.rs`、
  `runtime/primitives/cid/generator.rs`、`runtime/primitives/cid/number.rs`。
  未改变业务逻辑、存储布局、签名域、交易载荷或协议。
- `cargo fmt --all -- --check` 与 `git diff --check` 均通过；格式和空白残留已清零。
- 回归验证：OnChina 179 项、CID primitives 80 项、PoW 13 项、runtime 52 项全部通过。
  精确创世数量、CID 身份闭环、候选身份、竞选发布和费用路由断言继续通过。
- 使用 `WASM_BUILD_FROM_SOURCE=1`、单构建任务和关闭增量缓存的生产 release Node 构建成功；
  项目根 `target/debug/incremental` 与 `target/release/incremental` 均无跨次残留。
- 本步没有正式创世、冻结、提交、GitHub 推送、WASM CI、其它软件 CI 或部署。

## S7A 创世 preview 候选与第一次冻结（2026-07-30，完成）

- 两次确认后准备生成候选时，`main` 已由其它线程更新为 `6a75cbc8`，新增 runtime、
  OnChina、CitizenApp、二维码协议与 CI 门禁改动。没有沿用旧审查结论直接烤链，而是先
  对最新提交重新执行扩展审查。
- 最新基线验证：Rust workspace Clippy `-D warnings` 与全量测试通过；NodeGuard
  299 项、runtime 53 项、OnChina 179 项、QR protocol 5+7+1 项、CitizenApp Rust FFI
  13 项、Worker 222 项、Flutter 1042 项通过。Flutter 另有 5 项按测试声明因纯 Dart
  环境缺少宿主原生库而跳过；对应 Rust FFI 已通过。CitizenApp analyze、OnChina 前端
  build、官网 lint/build 和原正式冻结资产交叉检查均通过。
- 审查发现 4 个 OnChina 文件存在纯格式残留；经单独方案确认后只执行注释缩进、
  import 排序和换行调整，OnChina 179 项复测通过。本地 `main` 候选源码提交固定为
  `450e47af19851b9176a5e8bda128aba455bda482`，没有推送。
- 使用关闭增量、单构建任务并强制从源码生成 WASM 的 production release Node 冷构建
  成功，用时 30 分 10 秒。随后以 `bake-chainspec.sh` 默认模式生成
  `artifact_stage=preview` 候选，未使用 `--finalize`。
- 候选锚点：
  - `genesis_hash=0x37a3913895b6b2eda0e9fe242639338cc55ee1023e3caf25e31a178af990fa90`
  - `state_root=0xfdc23210da6d85e69eb2e107e291a1be2edf4c413516af0e12c35a40be4ed2f4`
  - `runtime_wasm_hash=4f553d22433d1348a133e44468d128ad47803c86e0e3542e14b69ec58a39413e`
  - `chainspec_hash=ea951f6372facc3d9ad9b748a6a436f516b36a2a4b614c3688ae0525897e5cb8`
  - `light_sync_state_hash=0f3e54e599cfe987596894278f35d64e7a9077ea93c260f2e0d2d20ed09afa16`
  - `public_institution_root=c21f99f5bd40bc3c9fcee9439de9f6902c98212b2510dd7440c9630284ab939f`
- 块 0 物化用时 131 秒，宪法与候选产物白名单检查通过；43 个省级公权分片及 manifest
  全部交叉校验。隔离节点直接复制候选 genesis-state 启动后为 0 peers、
  `isSyncing=false`，RPC storage 实证为公权 49,593 个机构 / 99,232 个协议账户，
  基金会 1 个机构 / 2 个协议账户，全创世 49,594 个机构 / 99,234 个机构协议账户。
- 隔离节点已停止，RPC 19945 已关闭；仓库外临时目录已移入 macOS 废纸篓，可恢复。
  preview 只保存在忽略目录 `citizenchain/target/chainspec/`；本步没有覆盖正式
  chainspec/App/Cloudflare 资产，没有推送、CI、正式 `--finalize` 或部署。
- 全量 debug 测试产生的 `citizenchain/target/debug/incremental/` 已整体移入 macOS
  废纸篓且可恢复；`target/debug/incremental` 与 `target/release/incremental` 均无跨次残留。
- 下一步按既定顺序申请单独 GitHub 远端操作许可：把本地 `main` 当前提交固定为不可变
  候选 tag，只推该 tag 而不移动 `origin/main`，然后手动运行 `CitizenChain WASM`
  CI。CI 成功并取得 artifact/run ID/head SHA 后，先完成 CI-WASM preview 核验，再
  输出正式 `--finalize` 技术方案并等待确认。

## S7B-A WASM CI 正式创世证据链加固（2026-07-30，完成）

- 审查发现原 workflow 在上传前删除全部同名历史 artifact，且上传步骤允许
  `continue-on-error`；可能出现旧证据已删除、新产物上传失败、整次运行仍显示成功，
  不满足正式创世必须使用 GitHub CI 成功 WASM 的要求。
- workflow 已改为上传失败即失败关闭；构建后逐一核验原始、compact 和 compressed
  三个 WASM 存在且非空，并把字节数和 Blake2-256 写入 GitHub Step Summary。历史
  artifact 不再主动删除，只按 30 天 retention 自动过期。
- `download-wasm.sh` 不再查询“最新成功”运行，强制传入 run ID、40 位 head SHA 和
  候选 tag；下载前确认远端轻量 tag 直接指向预期提交，并核对 workflow 名称、`workflow_dispatch`、`completed/success`、
  head SHA、ref 与唯一未过期 artifact。产物先下载到系统临时目录，三个预期 WASM
  完整且数量精确后才替换 `target/wasm-ci/`。
- `actionlint`、`shellcheck`、Bash 语法、diff 空白检查均通过；使用历史成功 run 配置
  错误候选 tag 的失败关闭测试在下载前正确拒绝，未改动已有本地 WASM。
- 正式创世唯一允许使用候选 tag 上本次 GitHub Actions 成功运行产出的
  `citizenchain.compact.compressed.wasm`；本地源码 WASM 只作预检，不得作为
  `--finalize` 输入。
- 下一步另行申请远端操作许可：只推不可变候选 tag，不移动 `origin/main`，避免提前
  触发其它软件 CI；随后以 `runtime_upgrade=false` 且不提供目标链参数，手动运行
  `CitizenChain WASM`。取得精确 run ID、artifact ID、head SHA 和三个 WASM 摘要后，
  先使用该 CI WASM 生成非 finalize preview 并核对，再提出正式冻结方案。

## S7B-B 唯一 WASM CI 与 CI-WASM preview（2026-07-30，完成但存在前置清理项）

- 轻量 tag `genesis-wasm-candidate-20260730-b77ca3c1` 已只身推送，直接指向
  `b77ca3c1ce7e12fe9df87e15a29444f7650bff7c`；`origin/main` 未移动，tag 推送没有
  触发任何自动 workflow。
- 只手动运行一次 `CitizenChain WASM` 普通源码构建：run `30589266930`、job
  `91027744279`、artifact `8777906747`；来源 tag/head SHA 精确匹配，
  `runtime_upgrade=false` 且没有升级链参数。编译、三文件摘要校验和上传全部成功，
  耗时 9 分 39 秒，没有运行其它软件 CI、发布或部署。
- 加固下载脚本核对远端 tag、workflow、event、status/conclusion、head SHA/ref 和唯一
  artifact 后下载成功。正式候选 compressed WASM 为 1,162,535 字节，
  SHA-256 `eecd43eb87815e2fe7601ef02856717b3ba7a1204f59998321887a3388fa4e91`，
  Blake2-256 `8d92e92ccd52693bce9ae915bae74600d58f6581d8e800396ef9bcfbf0b5f93e`。
- 同提交、同锁文件、同 Rust 1.97.1 的 macOS ARM 与 Ubuntu WASM 字节因函数/类型排列
  和调试名称布局不同；`runtime_version`、全部 runtime API、producer 和 target
  features 一致。按门禁停止核对后，继续只以 GitHub CI compressed WASM 为权威输入。
- CI-WASM 非 finalize preview：
  - `genesis_hash=0x278e68bced2dabf9690701188272da22d216fdaa2c617e7dcbe100df3e8bcbfa`
  - `state_root=0xa5386e7c0a0222fd030250b533bf73e78e947aec9f6a98dea7c1d5d64881c8c2`
  - `chainspec_hash=df2e5a28d99084ec5bcbed28db21ec3eecacbf364b421ce1fb47628c897387fe`
  - `light_sync_state_hash=a1a5d43046b379e8168a9651c41a7bbadf1299971252b4e9f99e7701056f8045`
  - `public_institution_root=c21f99f5bd40bc3c9fcee9439de9f6902c98212b2510dd7440c9630284ab939f`
- `:code` 与 CI compressed WASM 逐字节一致；宪法、43 个公权分片、状态包白名单及
  节点/App/checkpoint/Cloudflare 交叉校验通过。隔离节点真实 RPC 返回 0 peers、
  `isSyncing=false`、`specVersion=0`；公权 49,593 个机构 / 99,232 个账户，基金会
  1 个机构 / 2 个账户，全创世总计 49,594 个机构 / 99,234 个账户。节点已停止，
  19945 已关闭，临时验收目录已移入 macOS 废纸篓。
- 唯一残留：GitHub 注解显示 `actions/upload-artifact@v4` 仍声明 Node 20，只是被
  runner 强制改用 Node 24。本次 artifact 完整且运行成功，但为满足“没有问题后再正式
  创世”，必须先升级到官方当前原生 Node 24 版本、重新固定候选 tag 并重跑唯一 WASM
  CI。当前 run 不用于正式 `--finalize`。

## S7B-C1 GitHub Actions Node 20 全仓清零（2026-07-30，完成）

- 全仓共更新 11 处 Node 20 action 引用：
  - 5 处 `upload-artifact` 固定到官方 `v7.0.1` commit
    `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a`
  - 1 处 `download-artifact` 固定到官方 `v8.0.1` commit
    `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c`
  - 3 处 `setup-node` 固定到官方 `v7.0.0` commit
    `820762786026740c76f36085b0efc47a31fe5020`
  - 1 处 `setup-java` 固定到官方 `v5.6.0` commit
    `03ad4de0992f5dab5e18fcb136590ce7c4a0ac95`
  - 1 处 `setup-android` 固定到官方 `v4.0.1` commit
    `40fd30fb8d7440372e1316f5d1809ec01dcd3699`
- 每个 SHA 都通过 GitHub API 与对应官方 release tag 反向核验，`action.yml` 均明确
  `using: node24`。`setup-android v4` 的默认 cmdline-tools 20.0 与现有无自定义参数
  用法兼容，CitizenApp 后续仍显式安装 Android Platform 36。
- 其余外部 action 逐项检查：`checkout@v5`、`setup-node@v5` 和
  `Swatinem/rust-cache@v2` 已是 Node 24；`dtolnay/rust-toolchain@1.97.1` 与
  `subosito/flutter-action@v2` 是 composite。全仓没有 Node 20 action 残留。
- 全部 workflow `actionlint`、固定引用数量断言、官方 tag/SHA/runtime 反向核验和
  `git diff --check` 通过。本步没有修改 runtime、业务代码或正式创世资产，没有推送或
  触发 CI。
- 原 run `30589266930` 产生于 action 升级前，保留为审计证据但不用于正式创世。下一步
  必须基于本步提交固定新候选 tag，只重跑 `CitizenChain WASM`，取得无 Node 20 注解的
  成功 artifact 后重新执行 CI-WASM preview。

## S7B-C2 WASM 构建供应链固定（2026-07-30，完成）

- `CitizenChain WASM` runner 从可移动的 `ubuntu-latest` 收敛为明确
  `ubuntu-24.04`；上一成功 run 实际也使用 Ubuntu 24.04，未改变目标平台。
- `actions/checkout@v5` 固定为官方 `v5.1.0` commit
  `fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09`。
- `dtolnay/rust-toolchain@1.97.1` 经 GitHub API 查明实际是可移动的
  `refs/heads/1.97.1`，已固定到当前生成提交
  `46511b1c83438f0dd37c02d843619ece5a4abb5b`；该提交的 composite action 内容明确
  `toolchain: 1.97.1`，targets/components 参数保持不变。
- 构建前新增 `cargo metadata --locked --no-deps`，正式编译改为
  `cargo build --locked --release -p citizenchain`；Cargo.lock 需要变动时失败关闭，
  禁止 CI 临时解析未提交依赖。
- GitHub Step Summary 新增固定环境证据：runner label/OS/arch/image 版本、checkout
  与 Rust action SHA、Cargo.lock SHA-256、Rust/Cargo/Protobuf/Clang 完整版本。
- `actionlint`、`git diff --check`、`cargo metadata --locked`、官方 checkout tag/SHA、
  Rust 分支/SHA/硬编码 1.97.1 核验均通过；内嵌摘要脚本已在本机真实运行并输出完整
  证据，临时摘要文件已移入 macOS 废纸篓。
- 本步没有修改 runtime、业务代码或正式创世资产，没有推送或触发 CI。下一步必须把
  本步提交固定为新的轻量候选 tag，只触发唯一 `CitizenChain WASM` 普通源码构建；
  成功后精确下载新 run artifact 并重新执行非 finalize preview。
