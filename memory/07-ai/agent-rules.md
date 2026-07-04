# GMB Agent 规则

## 1. 统一交互规则

- 你可以在 Codex 或 Claude 聊天窗口中使用中文提出需求
- 对外输入统一为任务需求，不要求手工拆标题和目标
- 当前主聊天入口是默认总调度器
- 首轮默认先做需求分析，再决定是否进入执行
- 进入执行阶段后，当前主聊天入口必须根据任务所属模块，按需调度 `Blockchain Agent`、`OnChina Agent`、`Mobile Agent`、`Wallet Agent`
- 用户不需要手工指定分配给哪个 Agent，模块识别、任务拆分和调度由当前主聊天入口负责

## 2. Agent 角色

## 2.1 当前技术栈口径

- `citizenchain/node`：Rust + Substrate / Polkadot SDK + Tauri + React + TypeScript + Vite
- `citizenchain/runtime`：Rust + Substrate / Polkadot SDK
- `citizenchain/onchina`：公民链内置 OnChina，React + TypeScript + Vite 前端，Rust + Axum 后端，PostgreSQL
- `citizenapp`：Flutter + Dart + Isar
- `citizenwallet`：Flutter + Dart，负责离线签名和 QR_V1 签名响应

### Architect Agent

- 默认由当前主聊天入口主线程承担
- 负责读取 `memory/`
- 负责任务拆解
- 负责边界控制
- 负责发现需求歧义并及时沟通

### Blockchain Agent

- 由当前主聊天入口在任务涉及 `citizenchain` 时按需调度
- 负责 `citizenchain` 全域
- 包括 `node/` 原生节点、桌面端 Rust 后端、`frontend/` 前端与打包配置
- 包括 `runtime/`
- 包括区块链相关文档和打包流程

### OnChina Agent

- 由当前主聊天入口在任务涉及 `citizenchain/onchina`、多机构工作台、注册局业务、CID 号、机构登记、管理后台或链侧凭证时按需调度
- 负责 `citizenchain/onchina` 后端、前端、数据库、扫码签名、公开查询与文档

### Mobile Agent

- 由当前主聊天入口在任务涉及 `citizenapp` 时按需调度
- 负责 `citizenapp`
- 负责 Flutter 移动端与 Isar 本地存储

### Wallet Agent

- 由当前主聊天入口在任务涉及 `citizenwallet`、离线签名、冷钱包 QR 识别或签名响应时按需调度
- 负责 `citizenwallet`
- 负责公民钱包 Flutter 端、冷签名和扫码签名确认

### Review Agent

- 可由 Codex 或 Claude 承担
- 负责检查代码、指出问题、给出修复建议

### Release Agent

- 由 GitHub Actions 承担
- 负责自动测试、构建、打包、发布

## 3. 强制规则

> ### 最高优先级·两条硬性死规则（每次开工前默念，违反即返工）
>
> **死规则一·所有更改必须彻底，绝不保留兼容。** 链尚未运行、仍在开发期——禁止再就「是否需要 migration / 是否要重新创世 / 是否保留旧方案过渡 / spec_version 是否 bump」反复询问用户。默认答案永远是：直接按目标态一次性彻底改造、需要时直接重新创世、不留任何兼容/过渡/双轨/影子流程。发现旧代码、旧命名、旧目录、旧注释、旧文档、旧测试、旧 UI 文案、旧交易载荷、旧接口分支、旧生成物残留，必须在同一任务内全部清理，不允许有任何残留，不允许以「后续再处理」当作完成口径。
>
> **死规则二·所有目录、文件、字段命名必须全仓库统一且尽量精简。** 同一业务语义在 Rust / Dart / TypeScript / SQL / JSON / 文档 / 生成物中必须使用同一个名字，禁止另造 `name`/`label`/`display_name`/`type`/`status` 等泛化别名。命名要短、要直接表达语义；不确定是否同义时先全仓搜索 + 向用户确认，绝不擅自新建。

- 逻辑不清必须先沟通
- 真实开发任务必须先创建任务卡；包含 `检查为什么报错` 的只读报错诊断请求例外，只输出检查结果
- 未获得用户明确允许时，任何 AI 线程不得新建任何目录或文件；这条规则覆盖代码文件、文档文件、任务卡、测试文件、配置文件、生成物和临时文件。需要新建目录或文件时，必须先在回复中列出完整路径、用途和原因，等用户明确同意后才能创建
- 仓库新增文件/目录确认硬规则：任何 AI 线程在仓库中新增文件或目录前，必须在当前任务中先列出完整路径、用途、原因、是否会被 Git 跟踪，并得到用户明确确认；不得把历史确认、泛化授权或对其它目标的同意推断为新增文件/目录许可，任务卡、测试文件、临时文件和生成物同样适用
- GitHub 推送硬规则：未获得用户在当前任务中的明确允许，任何 AI 线程一律禁止执行 `git push`、`git push --force`、`git push --mirror`、删除或改写远端 ref、创建/更新 PR、通过 GitHub API/CLI 触发远端 workflow 或任何会启动 GitHub Actions/CI/CD 的操作；即使是安全修复、历史清理、脚本清理、紧急问题处理，也必须先单独说明将推送的远端、分支/tag/ref、预计触发的 CI 范围和风险，并等用户明确回复允许推送后才能执行。未获允许时只能保留本地提交或本地补丁，不得触碰 GitHub 远端。
- 根目录脚本保密硬规则：仓库根目录 `scripts/` 是本机私密脚本区，必须被 Git 忽略，任何 AI 线程不得 `git add -f scripts/`、不得提交、不得推送；凡涉及私钥、助记词、服务器部署账户、生产 SSH、token、keystore 或其他密钥材料的脚本，必须移动到根目录 `scripts/`。各软件模块自己的 `scripts/` 目录允许被 Git 跟踪并推送到 GitHub，但只能放构建、CI、生成、开发辅助等不含密钥材料的脚本。
- 禁止兼容硬规则：除非用户在当前任务中明确要求兼容，否则任何设计、实现、修复、数据处理、文档更新都不得保留旧流程、旧格式、旧数据、旧命名、旧目录、旧注释、旧文案、旧交易载荷、旧接口分支、过渡兼容、双轨兼容或影子旧流程
- 彻底改造硬规则：所有问题必须按目标状态一次性彻底改造；发现旧代码、旧注释、旧文档、旧配置、旧数据、旧目录、旧测试、旧任务描述或旧 UI 文案残留时，必须在同一任务内清理，不能以“后续再处理”作为完成口径
- 真实验收硬规则：完成开发任务前必须执行真实运行态验收；只通过编译、类型检查、单元测试或前端 build 不算完成。涉及 API、数据库、登录、权限、扫码或页面展示时，必须用真实本地服务、真实数据库、真实 HTTP 接口或真实页面验证目标行为
- 每次输出技术方案都必须包含：更新文档、完善注释、清理残留
- 每次输出技术方案都必须包含“预计修改目录”清单；清单中每个目录必须附中文注释，说明该目录的修改用途、边界和是否涉及代码、文档或残留清理
- 代码必须补中文注释
- 产品命名硬规则：公民（在线/热钱包）= 英文名 `CitizenApp`、模块 id/目录 `citizenapp`、中文名“公民”；公民钱包（离线/冷钱包）= 英文名 `CitizenWallet`、模块 id/目录 `citizenwallet`、中文名“公民钱包”。任何历史旧名及非目标中英文产品名一律废弃，不得在代码、文档、命名、注释中生成；改名进度见任务卡 `20260620-product-rename-citizenapp-citizenwallet`
- 管理员命名硬规则：所有机构和个人多签的管理员唯一字段统一为 `admins`；OnChina 登录态只允许用 `registry_org_code=FEDERAL_REGISTRY/CITY_REGISTRY` 表达当前账户所属注册局机构，不得恢复独立管理员身份表、授权真源或授权分支。
- 全仓字段同名硬规则：同一个业务语义字段在全仓库必须使用同一个命名；不得在 Rust、Dart、TypeScript、SQL、JSON、文档或生成物中为同一含义另造 `name`、`label`、`display_name`、`type`、`status` 等泛化别名。字段名必须尽量精简但直接表达业务语义；例如行政区名称必须按层级使用 `country_name`、`province_name`、`city_name`、`town_name`，泛行政区才允许使用 `division_name`；国家名称使用 `country_full_name` / `country_short_name`；机构实体名称和机构码对应中文名统一使用 `cid_full_name` / `cid_short_name`。不确定是否同义时必须先全仓搜索并向用户确认，不得自行命名。
- runtime 二次确认硬规则：任何涉及 `citizenchain/runtime/` 的修改，无论是业务逻辑、常量、权重、runtime primitives、注释、格式化、生成物还是仅由格式化工具造成的无逻辑 diff，都必须在执行前单独向用户说明完整路径、预计改动内容和原因，并得到用户明确的第二次确认；没有二次确认时，禁止读写工具、格式化命令或批量命令产生 runtime diff。
- 代码更新后必须更新文档
- 代码更新后必须清理残留
- **死规则：每次编码执行完成后，必须立即执行一次文档更新、完善注释、清理残留，不得跳过、不得延后、不得合并到下一次任务**
- 每次执行技术方案后都必须更新文档、完善注释、清理残留；未完成这三项不得标记任务完成
- 每次设计、编程、改协议、改命名、改文档、改流程前，必须先读取并遵守 `memory/07-ai/unified-required-reading.md`
- 不允许擅自突破模块边界
- 投票职责边界硬规则：所有业务模块不得实现、复刻、绕过或内嵌任何投票流程；所有投票流程统一归属投票引擎。业务模块只允许调用投票引擎已经提供的内部投票、联合投票、选举投票、立法投票模块接口来创建或绑定投票，不得自行处理人口快照、投票资格、联合签名、投票状态推进、计票、通过/否决判定
- 涉及 `citizenchain/runtime` 且会影响 `citizenapp` 在线端或 `citizenwallet` 公民钱包二维码签名/验签兼容性的任务，必须作为跨模块任务处理；不得只改单侧 runtime
- 上述任务必须同时装载 `citizenchain` 与 `citizenapp` / `citizenwallet` 上下文，并同步更新双端代码、文档与测试；未完成双端更新前，不允许继续 runtime 改动
- 不允许跳过契约直接扩展系统规则
- 涉及扫码、交易载荷、接口契约、字段顺序、签名验签、nonce、era、pallet/call index、storage key、subject id 的任务，必须先读取并遵守 `memory/07-ai/unified-protocols.md`
- 设计或修改任何协议/载荷/接口契约前，必须先更新 `memory/07-ai/unified-protocols.md`；扫码协议只有 `QR_V1`，内层交易载荷格式不得被称为新增扫码协议
- 检查 citizenapp 轻节点连接问题时，禁止把未部署 bootNodes 的 DNS/握手失败当作根因；只要存在有效 peer 且 best/finalized 状态可读或推进，就应判断区块链网络已连接
- 检查 citizenapp 轻节点连接问题时，禁止把本地开发期 `30334` bootnode/ADB reverse 作为必要条件；默认真机连接不依赖本地 `30334`
- 涉及新建或重命名目录、文件、字段、变量、类、模块、API 字段、storage 字段、扫码端解码展示字段、任务卡文件名、文档文件名的任务，必须先读取并遵守 `memory/07-ai/unified-naming.md`
- 所有新命名必须尽量精简；不确定的命名必须先报告用户确认，不得擅自新建
- 不允许删除、迁出或重命名 AI 编程系统核心基础设施
- 文件名长度不得超过 80 字符（含扩展名），详细描述放在文件内容里，不要塞进文件名
- `memory/08-tasks/` 下的任务卡文件名（含 `.md` 扩展名）不得超过 160 个 UTF-8 字节；标题与详细需求写入文件内容，文件名只保留短 slug
- 相同功能必须在前后端创建相同文件夹；功能不大时直接在对应文件夹下创建相同文件，功能过大时再按需下钻一级同名子文件夹；不确定边界时必须先询问用户
- OnChina 后端不得新建或恢复 `backend/src/` 源码壳；后端源码直接以 `citizenchain/onchina/src/` 为根目录展开
- OnChina 系统不得新建或恢复独立 `backend/chain/`、`frontend/chain/` 业务目录；各功能模块如需和区块链交互，必须在所属功能模块目录中新建 `chain_` 开头文件；跨模块链底层工具只允许放 `core/chain_*`
- OnChina 前端不得新建或恢复独立 `frontend/api/` 业务目录；某功能需要后端 API 时,必须在所属功能模块目录中新建 `api.ts`；通用 HTTP 请求封装只允许放 `frontend/utils/http.ts`,且不得承载业务接口

## 4. 开发闭环

```text
先分析需求
→ 读文档
→ 读契约
→ 生成计划
→ 技术方案包含预计修改目录(逐项中文注释)、文档更新、注释完善、残留清理
→ 写代码
→ 跑测试
→ 更新文档
→ 完善注释
→ 清理残留
→ 提交审查
→ 修复问题
```

只读报错诊断闭环：

```text
用户输入包含“检查为什么报错”
→ 读取相关上下文
→ 只读检查错误来源
→ 输出问题原因、影响范围和建议修复方向
→ 不创建任务卡
→ 不修改代码
```

## 5. 配套入口

- 角色执行说明：`memory/07-ai/agent-playbooks.md`
- 文档边界说明：`memory/07-ai/document-boundaries.md`
- 上下文装载顺序：`memory/07-ai/context-loading-order.md`
- 目录级 CI 路由：`memory/07-ai/ci-path-routing.md`
- 启动协议验收：`memory/07-ai/startup-acceptance.md`
- 审计任务铁律：`memory/07-ai/audit-recipe.md`(subagent 输出仅作 leads,正式报告每条必须回原文核验)
- 需求分析入口：`bash scripts/analyze-requirement.sh --requirement "..."`
- 启动协议检查：`bash scripts/check-startup-acceptance.sh`
- 执行入口：`bash scripts/architect-entry.sh --requirement "..." --execute`
- 新建任务卡：`bash scripts/new-task.sh --module "<模块>" --requirement "..."`
- 装载模块上下文：`bash scripts/load-context.sh <模块>`
- 归档任务卡：`bash scripts/complete-task.sh memory/08-tasks/open/<任务卡>.md "完成摘要"`

## 死规则:禁止排查 DNS 解析

- **永远不允许**把"DNS 解析失败/域名解析不了"当作问题原因来排查或汇报。
- DNS 解析永远没有错。chainspec 里部分省级 bootnode 子域名无 A 记录是已知且无害的(只用现存的几个活节点即可同步),logcat 里的 "No address associated with hostname" 是正常噪音,**不是故障**。
- 出现"读不到数据/连不上"类问题时,**严禁**检查 DNS、严禁说"手机没连上网络/没连上节点"。必须先看 App 自报的 peerCount/finalized 状态确认是否已同步,已同步就从业务代码路径找原因。
- 对比定位法优先:同样读链上数据,A 处能读、B 处读不到,直接对比 A/B 两条代码路径的差异,不要到处大海捞针。

## 死规则:citizenapp 链上查询(ADR-018)

citizenapp 是轻节点(smoldot),所有链上读取强制遵守(详见 `memory/04-decisions/ADR-018`):

- **R1 统一字段查询**:列表类数据一律"短 key 索引(`ProposalsByYear`/整表扫描)取一次 → 客户端按已解码字段过滤"。**禁止**对嵌 32 字节 AccountId 或 `blake2_128(x)+x` 的**长前缀**做 `getKeysPagedFinalized`(轻节点静默返回空)。精确整键 `fetchStorage(完整key)` 不受限,可用。
- **R2 降低全节点依赖**:① 多 key 一律 `fetchStorageBatch`/`fetchFinalizedBalances`,**禁止循环内逐条** `fetchStorage`/`fetchFinalizedBalance`(N+1);② 同一数据跨页面取一次进共享缓存复用;③ 链状态页用 finalized 头订阅驱动刷新,禁止 `Timer.periodic` 轮询查链;④ 能用已缓存/已解码数据客户端算出的,不再联网。
- **R3 外部后端(CID/HTTP)缓存**:health/catalog/机构注册证/电子护照状态等读取加 Isar + TTL 缓存。
- **豁免**:交易提交管线(nonce/dry-run/submit/runtime-version/genesis/提交用 best 块)+ UI 倒计时 Timer。

## 死规则:行政区/机构代码唯一真源 + code 不可变不复用(ADR-021)

- **常量唯一真源**:国家码、省级行政区码和 CID 机构码只允许维护在 `citizenchain/runtime/primitives/cid/code.rs`。国家用 `CountryCode=CN`;省用 `ProvinceCode` 两位大写字母;机构用 `InstitutionCode` 三/四位大写字母。OnChina 只能通过 `crate::cid` / `primitives::cid` 引用,不得恢复第二份机构码表或省码表。
- **行政区运行数据唯一真源**:市、镇和镇下地址只有一个入口 = `citizenchain/onchina/src/cid/china/`。开发库 `citizenchain/onchina/src/cid/china/china.sqlite` 是市镇地址权威源；生产 `ONCHINA_CHINA_DB` 只指向随包只读 SQLite。**任何地方不得独立维护第二套市镇地址名字**。SQLite 省表必须与 primitives `PROVINCE_CODE_INFOS` 一致,加载时断言。
- **发布消费**:市镇地址变更必须修改开发库；OnChina 与 citizenapp 发布包都从开发库派生本地只读快照。国家码、省码、机构码变更属于 runtime primitives 变更,必须走 runtime 二次确认。不得恢复行政区管理 tab,不得恢复 `/api/v1/app/admin-divisions/*`,citizenapp 不联网拉取行政区新版。
- **目录红线**:不得恢复 `citizenchain/onchina/src/cid/china/data/`。`check_code_immutable.py` 和 `china.sqlite` 直接位于 `citizenchain/onchina/src/cid/china/`。
- **code 当前态唯一**:省 code 固定在 primitives；市/镇 code 按当前创世基线确定。地址库不保留旧数据、旧表、墓碑或变更日志。改名只改 `province_name/city_name/town_name` 不改 code。
- **校验**:`cid/china/store.rs::load_provinces` 加载即断言 SQLite 省表与 primitives 一致、省名和市名全国唯一、(省,市,镇) code 无重复；CI `citizenchain/onchina/src/cid/china/check_code_immutable.py` 检查活跃 code 无重复且旧地址表/墓碑表/变更日志表不存在。
- **红线**:市镇地址开发库变更不直接修改 `citizenchain/runtime/`。地址变更需要同步全网时只走 `AddressRegistry` 单条变更事实；国家码、省级行政区码、机构码和 `runtime/primitives/cid/china/` 保护机构常量需要变更时,必须走 runtime 升级二次确认。
