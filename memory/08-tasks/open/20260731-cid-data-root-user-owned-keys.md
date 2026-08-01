# CID 当前钱包直接控制与私有数据加密

状态：in_progress（2026-07-31，按用户确认的分步方案执行）

## 全仓第一死规则（2026-08-01 用户确认）

- 全仓库涉及 Substrate / Polkadot SDK 的概念必须严格使用官方类型和官方术语。
- 同一语义只能存在一个基础名称；跨语言只允许 `snake_case` 与 `lowerCamelCase` 的语法
  转换，禁止同义词、缩写、别名、兼容字段、双轨字段和影子字段。
- 当前换绑任务必须先按官方 `AccountId` 体系统一账户、签名、地址、origin、事件、协议、
  API、测试、注释和文档命名，再进入创世、CI 或部署。
- 存在任何把“换绑前链上当前账户”命名为含糊历史账户的字段残留时，本任务不得完成。

## 已确认最终口径

- `cid_number` 是用户唯一身份主键，也是动态、文章、粉丝、通讯录、会员等数据的唯一归属键。
- 只有链上当前绑定 `account_id` 能控制 CID；钱包账户承担签名授权与付款职责，不是第二身份主键。
- 不建立任何额外主钥、随机 CID 密钥、Worker 密钥、注册局密钥、节点密钥或恢复服务。
- 私有数据用途密钥只允许由当前绑定钱包账户的 child mini-secret 在 CitizenApp 本地直接派生。
- 同一钱包账户在新设备导入后能重新派生相同用途密钥。
- 换绑 finalized 后，新钱包账户立即接管 CID、公开数据、授权和付款，并派生自己的新用途密钥。
- 任何换绑方式只要换绑前链上当前账户能够签名，就在同一次换绑中由当前账户密钥解密
  此前私有数据、新账户密钥重新加密并继承；当前账户不可用时才无法继承此前私有密文。
- Worker 只做当前绑定鉴权和密文存取，不持有、生成、密封、恢复或下发用户私有数据密钥。

## 分步执行

### 第一步：删除额外密钥链路，建立当前钱包直接派生

状态：completed（2026-07-31）

- 删除 App 中的稳定主钥、包装、缓存、领取与安装逻辑。
- 删除 Worker API、Secret、D1 表和相关挑战协议。
- 建立 `AccountDataBinding` 公开元数据和 `AccountDataKeyDeriver` 唯一派生入口。
- HKDF 输入固定绑定 `genesis_hash + cid_number + binding_revision + account_id + purpose`
  及可选业务 context。
- 只接入 Chat、MLS、聊天附件和通讯录现有密钥消费者；用途密钥只在内存短期使用，
  禁止扩展到草稿或其它数据。
- 测试必须证明同账户跨设备可重建、用途隔离、版本防回退、新钱包不能直接解密此前密文。
- 同步更新安全、架构、钱包、协议、注释和旧任务说明，清除旧口径残留。

#### 第一步执行结果

- App 已删除稳定 CID 数据根、账户包装、缓存、领取、恢复和安装链路；本机只保存公开
  `AccountDataBinding`，私有数据用途密钥由 finalized 当前绑定账户 child 直接派生。
- Worker 已删除密钥领取/接管 API、挑战协议、环境 Secret 和 D1 密钥表；D1/R2 只承担
  已有密文或公开数据存取，不持有用户私有数据密钥。
- 绑定输入统一失败关闭：创世哈希、1-32 字节 CID、正绑定版本和规范 `account_id`
  任一不合法都不能保存或参与派生；绑定版本禁止回退，同版本字段冲突被拒绝。
- 换绑后新账户可接管 CID 的授权、公开数据与密文记录并派生自己的新用途密钥；测试已
  证明新账户不能直接解密换绑前当前账户加密的历史私有数据。
- 已更新安全边界、架构、钱包、用户、Chat、统一协议、统一命名、相关任务记录和中文
  注释；源码、测试、文档及本机私有 CitizenConsole 应用副本中的旧密钥链路残留已清理。

#### 第一步真实验收记录

- `npm --prefix citizenapp/cloudflare run typecheck`：通过。
- `npm --prefix citizenapp/cloudflare test`：32 个测试文件、217 项通过、0 失败。
- `dart analyze lib test`：0 问题。
- `flutter test --concurrency=1`：1051 项通过、5 项因本机缺少既有原生测试库跳过、
  0 失败。
- 本地 D1 从当前单一 schema 建库：60 条命令成功、27 张目标表；废弃密钥表计数为 0。
- 真实本地 Worker：`GET /health` 返回 200；废弃密钥接管路径返回 404。
- `git diff --check`：通过；`citizenchain/runtime/` 无差异；未推送、未触发 CI、未部署、
  未创世。

### 第二步：单次换绑完成当前账户与新账户双签及私有数据重加密

状态：completed（2026-08-01，用户已二次确认并完成验收）

- 公民 App 自主换绑只读取一次 finalized 上下文，在同一个钱包选择流程中完成当前账户
  移交签名和新账户接管签名。
- 注册局换绑的一个 `QR_V1` 响应同时承载当前账户与新账户签名；注册局权限、管理员冷签和无当前
  钱包时的强制换绑能力不变。
- 有当前账户签名时，客户端只对聊天和通讯录执行“当前账户钥解密 → 新账户钥加密 → 新账户钥回读验证”；明文只在
  内存短暂存在，Worker、D1、R2 均不接触密钥或明文。
- 云端密文仍只归 `cid_number`，以 `binding_revision` 区分当前与目标版本；新账户接管并
  回读成功后清理此前版本密文。
- 只盘点聊天和通讯录在 Isar、本地文件、D1、R2 中的字段和对象，补齐测试、注释、文档
  与错误口径残留；禁止扩展到草稿或其它数据。

#### 第二步执行结果

- runtime、CitizenApp、CitizenWallet、OnChina、QR registry、TypeScript、JSON、测试、
  注释和文档统一使用 `current_account_id/currentAccountId`、
  `current_account_signature/currentAccountSignature`、`new_account_id/newAccountId`、
  `new_account_signature/newAccountSignature`；换绑完成事件使用
  `previous_account_id/previousAccountId`。SCALE 字段顺序、call index、权限与防重放模型未变。
- 自主换绑在一次钱包选择流程中完成当前账户授权签名和新账户交易签名。注册局的同一个
  `QR_V1` 响应可同时携带当前、新账户签名；当前账户签名只决定能否交接 Chat、通讯录，
  不改变注册局与管理员的强制换绑权限。
- Chat 正文、会话摘要、搜索索引、MLS 状态和聊天附件在交易提交前旁路重加密并用新账户
  密钥回读；finalized 后幂等切换目标密文。提交/丢弃阶段不构造任何占位密钥，密钥只在
  内存短期存在。
- 通讯录在 finalized 前只把目标账户密文保存在本地交接清单，Worker 明确拒绝未来绑定
  预写；finalized 后由新账户当前 Session 上传、完整回读解密，再删除相邻的此前版本。
  D1 复合主键固定为 `(cid_number,binding_revision,account_id,contact_id)`，其中 CID 仍是
  唯一数据属主，版本与账户只作为密文派生上下文。
- Cloudflare 只保留唯一 `schema/citizenapp.sql`，schema 初始版本为 `v1.0.0`；中心 CID
  数据根、领取/接管路由、恢复挑战、环境 Secret 与 D1 密钥表已删除。加密范围没有扩展
  到草稿、设置或其它数据。
- 仓库 AI 第一死规则已经写入统一入口、必读链、工作流、完成定义和提交检查：Substrate /
  Polkadot SDK 语义必须使用官方类型与术语，同一语义跨语言只允许大小写语法转换。

#### 第二步验收记录

- CitizenApp 换绑、Chat、MLS、附件、通讯录与钱包定向测试：135 项通过；最终命名修正
  后受影响子集 24 项再次通过；`dart analyze lib test` 零问题。
- CitizenWallet 签名/解码测试：120 项通过；`dart analyze lib test` 零问题。
- Cloudflare Worker：32 个测试文件、218 项通过；类型检查通过。
- `citizen-identity`：70 项通过；`qr-protocol`：5 项单测、7 项 registry 一致性测试、
  1 项仓库协议守卫通过；CitizenApp Rust MLS：13 项通过；OnChina：182 项通过。
- OnChina 前端构建通过；仅保留既有 Vite 大 chunk 警告，已跟踪构建包更新为当前源码哈希。
- 全新本地 D1 执行唯一 schema 的 60 条命令成功，生成 26 张业务表；禁用密钥表计数为 0。
  真实本地 Worker `GET /health` 返回 200，两条废弃中心密钥接管路径均返回 404。
- `git diff --check` 通过；未推送 GitHub、未触发 CI、未部署、未创世。

### 第三步：完成换绑生命周期与真实运行态验收

状态：completed（2026-08-01，用户已确认并完成验收）

- finalized 新绑定驱动当前钱包派生上下文、设备子钥、Session 和 Chat 设备状态收敛。
- 清理此前账户的内存用途密钥和凭证；不要求此前账户、此前设备或此前助记词参与。
- 使用真实本地 App、Worker、D1 和 HTTP 路径验证目标行为。

#### 第三步执行结果

- CitizenApp 只接受同一个 `cid_number + binding_revision + account_id` finalized 三元组作为
  当前派生、Session、Chat 设备与本地密文上下文；新三元组激活时先关闭此前 Chat 上下文、
  HTTP 传输和广场 Session，再建立新上下文，迟到的此前登录结果不能覆盖当前绑定。
- Chat 会话、消息、MLS 状态和附件目录按 CID 与密文绑定元数据隔离；CID 始终是唯一数据
  属主，`binding_revision` 和 `account_id` 只标记密文所用派生上下文，不成为业务归属键。
- 当前账户与新账户在一次换绑中均签名时，Chat 与通讯录严格执行当前密文解密、新账户
  密文加密、回读验证和 finalized 切换；没有当前账户签名时保留此前永久密文但不向新账户
  暴露，并清理待发送、待接收、附件队列、路由、群镜像和 MLS 待提交等瞬时状态。
- Worker 每次敏感请求都重新核对 finalized 当前绑定。新设备子钥登记成功后清理此前绑定的
  challenge、Session、KeyPackage、Chat 设备、设备子钥和缓存；Durable Object 实时连接关闭
  失败时返回 503，禁止在旧连接尚未撤销时报告换绑收敛成功。
- Chat、MLS 和通讯录用途密钥按操作即时派生并在 `finally`/`dispose` 中清零，不持久缓存；
  Worker、D1、R2 不生成、不保存且不能取得用户私有数据密钥或明文。
- 已同步更新 Chat、用户、钱包、架构、安全、统一协议和中英文白皮书，重新生成节点内置
  白皮书，修正 seed、助记词、所谓“主钥”、解密缓存、钱包地址充当聊天身份及账户同义
  命名残留；没有扩大 Chat、通讯录以外的加密边界。

#### 第三步真实验收记录

- `dart analyze lib test`：零问题；`flutter test --concurrency=1`：1066 项通过、5 项因本机
  缺少既有原生测试库跳过、0 失败；最终注释与命名收口后的 29 项定向测试再次通过。
- Android 16 全新临时 AVD：当前 debug APK 编译、安装和 `MainActivity` 冷启动成功，进程
  保持存活且 Activity 位于前台，日志无 `FATAL EXCEPTION`；验收后已删除临时 AVD。
- CitizenChain 节点前端：从当前白皮书真源重新生成内置文档，TypeScript 检查和 Vite
  production build 通过；仅保留既有大 chunk 提示。
- Cloudflare Worker：32 个测试文件、220 项通过；TypeScript 类型检查通过。
- 全新本地 D1：唯一 schema 的 60 条命令成功、26 张业务表；不存在 CID 数据根或恢复密钥
  表，通讯录复合主键精确为 `(cid_number,binding_revision,account_id,contact_id)`。
- 真实本地 Worker：`GET /health` 返回 200；两条已删除的中心密钥接管路径返回 404；
  未携带 Session 的 Chat 设备登记返回 401。
- `git diff --check` 通过；当前分支为 `main`；第三步未新增 runtime 差异；未推送 GitHub、
  未触发远端 CI、未部署、未创世。

### 第四步：创世前全仓最终审计与冻结候选确认

状态：in progress（CI、宪法三层编号和 OnChina 管理员换绑接管已完成；等待下一项方案确认）

- 只读复核 CID 唯一身份、当前 `account_id` 控制、双签交接、奖励双重防重、投票/竞选、
  机构管理员、宪法编号唯一性、数据加密边界和 Substrate 官方命名是否仍有漏洞或残留。
- 已删除单独 `ai-guardrails.yml`，目前严格只有 `citizenchain-ci.yml`、
  `citizenchain-wasm.yml`、`citizenapp-ci.yml`、`citizenwallet-ci.yml` 四个 workflow；官网无 CI。
- AI Guardrails 和 pallet 注册表检查已并入 CitizenChain PR job；CitizenChain 变更还会执行
  全 Rust workspace fmt/check/test/clippy、宪法 SCALE 自检、OnChina 前端和节点前端构建，
  成功后才进入四平台桌面端 matrix。
- Cloudflare 已并入 CitizenApp CI，覆盖绑定类型时效、TypeScript、Vitest 和全新本地
  D1 schema；同时清理 npm script 中过期的 `citizenapp-square-db` 本地名，统一使用
  Wrangler 绑定 `DB`。
- 本地验收：四个 workflow 数量精确；`actionlint`、YAML、启动协议、pallet 注册表、
  宪法 SCALE 自检通过；Cloudflare 32 个测试文件/220 项通过，全新临时 D1 的
  60 条 schema 命令全部成功。
- CitizenChain 新 CI 命令已真实执行：两个前端 build、`cargo fmt --all -- --check`、
  `cargo check --workspace --all-targets --locked` 和
  `cargo test --workspace --all-targets --locked` 全部通过。Clippy 曾拦下
  `runtime/misc/citizen-identity/src/tests/mod.rs` 第 49、420 行的两处冗余字段初始化；
  用户完成本项 runtime 二次确认后已统一改为 Rust 字段简写，不改变逻辑、SCALE、字段顺序、
  权限或安全模型。`citizen-identity` 70 项测试通过，随后
  `cargo clippy --workspace --all-targets -- -D warnings` 全量退出 0，原 CI 阻断已消除。
- 宪法三层编号唯一性已在用户确认和 runtime 二次确认后完成：章号全文唯一、同章节号唯一、
  条号全文唯一；不同章允许复用节号，不要求编号连续或从 1 开始。runtime 在创世、修宪提案、
  投票结果最终写入三层失败关闭，原生 ConstitutionGuard 和 Python 创世检查执行相同口径；
  未修改 `constitution.scale`、冻结 chainspec 或任何创世数据。
- 本项验收：`legislation-yuan` 45 项、`core::constitution` 47 项及稳定源码下全 Rust workspace
  测试全部通过；全 workspace check/Clippy、宪法 SCALE 自检、pallet 注册表和 AI
  启动协议通过。`citizen-identity` 两处冗余字段初始化已在独立 runtime 二次确认后修复，
  全 workspace、全部 target、`-D warnings` 的严格 Clippy 当前通过。
  冻结主网 chainspec 使用其内嵌 GitHub CI WASM 在全新 `--tmp` 目录真实启动，节点守卫自检通过，
  RPC 返回创世哈希 `0x278e68bced2dabf9690701188272da22d216fdaa2c617e7dcbe100df3e8bcbfa`
  且 `isSyncing=false`；钉住 block#0 的宪法创世检查通过。未推送、未触发 CI、未部署、未创世。
- OnChina 管理员授权已统一为单一 finalized 快照：带 CID 管理员通过
  `AccountIdByCid / CidByAccountId` 闭环解析当前绑定账户；私权 LR 在名册 CID 为空时读取
  本机构法定代表人 CID；冻结公权无 CID 管理员和私权非 LR 无 CID 管理员按名册账户；
  个人多签保持账户治理且不进入 OnChina。岗位任职继续以名册账户关联，API 和鉴权输出当前
  可签名账户，换绑后不重写岗位即可由新账户接管。
- 登录反查、管理员列表、敏感操作复核和周期会话撤权复用同一解析器。真实 HTTP 负例发现并
  修复“遍历到任意节点桌面治理机构就误报普通账户”的顺序问题：现在必须先命中目标签名账户
  的管理员名册，才返回桌面端或个人多签边界错误。
- 本项验证：OnChina 191 项测试通过，`cargo clippy -p onchina --all-targets -- -D warnings`
  通过；冻结 plain chainspec 创世哈希核对正确，临时 OnChina 完成 49,593 个机构、99,232 个
  账户投影与 34 项抽样对账。真实登录第一步中 FRG 冻结管理员和基金会当前绑定账户返回 200，
  无在册账户返回 403 / `ONCHINA_LOGIN_ADMIN_NOT_ONCHAIN`。临时节点、PostgreSQL 和服务已关闭，
  临时数据库移入废纸篓；未修改 runtime、创世、远端 CI 或部署状态。
- 汇总拟冻结提交、候选 tag、唯一 WASM workflow、预计触发范围与风险，单独取得远端推送
  许可；本步骤不推送、不触发 CI、不执行 `bake --finalize`。

#### 第四步最终只读复审追加（2026-08-01）

状态：blocked（当前 HEAD 不能作为新的创世冻结候选；本次仅审计并记录，不修改代码、runtime、
创世资产、workflow 或部署状态）

- 复审基线为本地主分支 `main` 的 `2c04394576e36dc5e8ed6dea517ca8e970b45aee`；该提交领先
  `origin/main` 5 个提交，GitHub 上没有该提交的 CitizenChain CI 或 CitizenChain WASM 成功记录。
- 当前 runtime 相对已冻结的 `origin/main` 存在实质代码变化；仓库现有冻结 plain chainspec、
  创世哈希和 WASM 来源仍明确指向 CI runtime source `9f61e986...`、资产提交 `369cbc5a...` 与
  WASM run `30593994910`。既有冻结资产内部校验仍通过，但不能作为当前 HEAD 的 CI WASM 证明。
- 工作树存在本任务之外的未提交修改 `citizenwallet/lib/ui/home_page.dart`；本次未触碰该文件。
  未提交工作树也不满足冻结候选必须可复现、不可变和可精确追溯的条件。
- 新发现的确定 CI 阻断：`cargo fmt --all -- --check` 在
  `citizenchain/runtime/primitives/tests/scale_codec_golden.rs` 第 46、61、116 行对应代码处失败。
  本次只读审计未执行格式修复。
- 四条 workflow 数量和职责仍正确：CitizenApp CI 已包含 Cloudflare，CitizenChain CI 包含 Rust、
  OnChina、节点前端及打包覆盖，官网没有独立 CI；`actionlint` 通过。
- CID 换绑、奖励 CID + `account_id` 双重防重、投票和候选 CID 唯一键、竞选内容资格、机构管理员
  finalized 链上真源、个人多签账户治理、宪法“章全局唯一、节同章唯一、条全局唯一”均按已确认
  安全模型实现；宪法 SCALE 自检、启动守卫、pallet 注册表和 AI Guardrails 通过。
- Chat、通讯录继续由当前钱包账户材料在客户端按 CID、绑定版本、`account_id` 和用途直接派生；
  本地与云端保存密文，Worker 没有用户数据主密钥、恢复密钥或解密路径。存在旧账户签名时可在
  同一次换绑交互中完成旧密文解密、新账户重加密；不存在旧账户签名时新账户不能解密历史私密数据。
- 签名协议命名曾存在硬规则残留：Chat 的 protobuf 包名、在线消息类型、统一协议文档和 PQC
  草案仍使用额外版本化标识，与“全仓唯一允许的版本化协议标识是 `QR_V1`”冲突。
- 安全测试覆盖残留：黄金向量同步脚本虽退出 0，但报告 CitizenApp 镜像缺少 CID 换绑 `0x11`
  和 GRANDPA 换钥 `0x1e` 两条 canonical 向量；其中 CitizenApp 实际实现 CID 自助换绑签名，
  `0x11` 应升级为跨端同步的强制向量，不能仅依赖算法单测。
- 注释和文档残留：fullnode 手续费收款钱包绑定注释仍声称必须先真实出块；个人多签注释仍声称
  管理员完整保存姓、名；Chat 本地存储注释仍使用“明文”旧口径；两张黄金向量任务卡实施和验收
  已全部完成但仍留在 `open/`；本任务卡此前记录的“全 workspace fmt 通过”已被当前 HEAD 的
  新增格式错误推翻，以本追加记录为准。
- 结论：在格式阻断、协议命名残留、关键换绑黄金向量缺口、注释文档残留、工作树不干净以及当前
  HEAD 尚无 GitHub CI/WASM 证明全部处理并重新复审前，不得把当前 HEAD 冻结、创世或部署。

### 第五步：冻结候选阻断清理与本地验收

状态：completed（2026-08-01 本地修复、残留清理和全量验收完成；未推送、未冻结、未部署）

- 仅修复 `scale_codec_golden.rs` 的 Rust 格式，不改变测试取值、SCALE、runtime 逻辑或创世状态。
- 清理 fullnode 手续费收款钱包绑定和个人多签管理员的错误注释，不改变既有安全模型。
- 全仓删除 GMB 自定义的非 `QR_V1` 版本化协议标识；Chat 的 protobuf、Worker、App、测试和文档
  使用无版本业务名，不保留旧消息类型兼容。Substrate 官方 JSON-RPC 方法名按官方原名保留。
- CitizenApp 签名域镜像补齐 canonical `0x11`、`0x1e`，并把该组覆盖从提示升级为缺失即失败。
- SCALE 字符串金标中的中国国旗字符改为中性四字节 Unicode 测试值，并在 AI 门禁和规则中
  永久禁止全仓代码、测试、夹具、注释、文档、配置及生成物恢复中国国旗字符。
- 完成本地全量门禁、测试和真实运行验收后，先输出候选 tag、远端 ref、WASM workflow 和风险，
  单独取得推送许可；本步骤不推送、不触发远端 CI、不冻结、不部署。

#### 第五步验收记录

- 全仓中国国旗字符残留为 0；SCALE 金标改用中性四字节 Unicode 测试值，Rust 金标 3 项通过。
  AI 门禁已将该禁令覆盖代码、测试、夹具、注释、文档、配置和生成物。
- `cargo fmt --all -- --check`、全 workspace/all-targets `cargo check`、`cargo test` 和
  `cargo clippy -- -D warnings` 全部退出 0；CID、奖励双重防重、投票/候选、宪法三层编号、
  个人多签、矿工收款账户绑定、OnChina 管理员与订阅换绑扣费相关测试均通过。
- CitizenApp `flutter analyze` 零问题，1092 项 Flutter 测试通过、5 项按既有原生库条件跳过；
  Rust OpenMLS 11 项通过。Pixel 8a / Android 16 真机完成 Keystore 子钥删除后重建新密钥的
  仪器测试，`:app:connectedDebugAndroidTest` 与 ARM/ARM64 Debug APK 构建均成功。
- CitizenWallet `flutter analyze` 零问题，282 项测试和 Debug APK 构建通过；OnChina 前端构建
  与 5 项测试、节点前端构建通过。
- Cloudflare Worker 绑定类型和 TypeScript 检查通过，33 个测试文件、256 项测试全部通过；
  全新临时 D1 的 60 条 schema 命令成功。真实本地 Worker 的 `/health` 返回 200，未携带钱包
  Session 写入 Chat 信封返回 401；Worker 不取得用户私有数据密钥或明文。
- 金标镜像强制覆盖 canonical `0x11` 和 `0x1e`，14 条签名、2 + 2 条二进制前缀、10 条账户
  派生向量全部一致；pallet 注册表 34 项、宪法 SCALE、自启动协议、冻结 chainspec、AI 门禁、
  `actionlint`、`git diff --check` 和额外版本化协议残留扫描全部通过。
- 构建成功后已把 `citizenchain/target` 与 `citizenapp/rust/target` 内全部 `incremental` 目录移入
  废纸篓，复查为 0；工作树没有混入构建产物或临时文件。未推送 GitHub、未触发远端 CI、
  未冻结、未创世、未部署。

### 第六步：WASM 候选推送与 GitHub 证明

状态：in progress（最新 `main` 已推送；等待本次显示名称为“创世”的 WASM CI 成功）

- 先核对 `main`、候选提交、候选 tag、远端 `origin` 和唯一 `citizenchain-wasm.yml` 触发范围；
  把所有本地修改形成可复现提交，但不提前冻结创世资产。
- 仅推送用于 WASM CI 的候选 ref，等待 GitHub `CitizenChain WASM` 对该精确提交成功；不得用本地
  WASM、旧 run、其他提交或未完成的 run 代替。
- WASM CI 成功后再输出 run、commit、artifact 摘要和创世冻结技术方案，等待下一次确认；在冻结
  完成前不推送其他软件 CI，冻结完成后才按顺序推送 `main` 并等待其余三条软件 CI 全绿。
- 本步骤不部署；只有 WASM 证明、创世冻结和其他软件 CI 全部成功后，才另行输出部署方案并等待确认。
- 本次 WASM run 临时使用 `run-name: 创世`；它只允许服务当前候选，WASM 成功并冻结时必须删除，
  后续运行恢复标准名称 `CitizenChain WASM`。此前取消的 run 均不构成 WASM 成功证明。

## 第一步预计修改目录

- `citizenapp/lib/security/`：代码；建立当前钱包直接派生和公开绑定元数据，删除旧密钥模型。
- `citizenapp/lib/wallet/`：代码与中文注释；从当前账户 child 派生并收口钱包删除清理。
- `citizenapp/lib/my/myid/`：代码与中文注释；按 finalized 精确绑定激活派生上下文。
- `citizenapp/lib/chat/`、`citizenapp/rust/`：代码与注释；接入 Chat、MLS、附件现有密钥消费者。
- `citizenapp/lib/8964/services/`：代码与残留清理；删除私有数据密钥领取客户端。
- `citizenapp/cloudflare/src/`、`citizenapp/cloudflare/schema/`：代码、schema 与残留清理；
  删除服务端密钥 API、环境绑定和数据库存储。
- `citizenapp/test/`、`citizenapp/cloudflare/test/`：测试；更新派生、换绑、密文和旧接口测试。
- `memory/00-vision/`、`memory/01-architecture/`、`memory/03-security/`、
  `memory/05-modules/`、`memory/07-ai/`、`memory/08-tasks/`：文档、协议、任务记录和残留清理。
- `citizenconsole/`：本机私有残留清理；只修改现有文件，不纳入 Git，不新增文件。

## 第一步验收门槛

- App 和 Worker 无任何额外用户私有数据主钥、服务端密钥、领取接口或密封表。
- 同一账户 child + 同一绑定上下文 + 同一用途得到相同密钥；用途、账户或绑定版本变化得到不同密钥。
- B 钱包接管 CID 后可读取 CID 公共数据和云端密文记录；有 A 钱包当前账户签名时，
  Chat、通讯录先完成重加密交接；没有该签名时，B 钱包不能解密此前历史私有密文。
- Worker 类型检查、Vitest、Dart 分析、Flutter 定向/全量测试、schema 和真实本地 Worker/D1
  HTTP 验收通过。
- 文档、中文注释和全仓残留搜索与最终口径一致。

## 边界

- 第二步仅修改 `citizenchain/runtime/misc/citizen-identity/` 的换绑协议字段、事件字段、
  测试与基准命名；不改变 SCALE 字段顺序、call index、权限或安全模型。该 runtime 修改
  已在 2026-08-01 获得用户确认和二次确认。
- 未经单独确认不推送 GitHub、不触发远端 CI、不部署、不创世。
- 每一步完成后立即更新文档、完善注释、完善测试、清理残留，并先输出下一步技术方案等待确认。
