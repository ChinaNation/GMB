# 完善公民链白皮书

任务需求：
- 根据当前《公民链白皮书》和仓库真实实现，分步骤补全白皮书。
- 每一步先制定方案，用户确认后再执行；每一步完成后回写本文档，再进入下一步方案。
- 白皮书要从“模块清单”完善为“制度、经济模型、技术架构、身份边界、安全边界、应用生态”完整说明。

所属模块：
- docs
- website
- citizenchain
- sfid
- cpms
- wumin
- wuminapp

输入文档：
- docs/《白皮书》.md
- website/src/pages/Whitepaper.tsx
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md
- memory/01-architecture/sfid/SFID_TECHNICAL.md
- memory/01-architecture/cpms/CPMS_TECHNICAL.md
- memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md
- memory/04-decisions/ADR-022-unified-pqc-crypto.md
- memory/05-modules/citizenchain/runtime/votingengine/VOTINGENGINE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/primitives/PRIMITIVES_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/transaction/offchain-transaction/STEP1_TECHNICAL.md
- memory/05-modules/citizenchain/node/offchain-clearing/L2_CLEARING_PROTOCOL.md
- memory/05-modules/sfid/SFID-CPMS-QR-v1.md
- memory/05-modules/sfid/backend/citizens/CITIZENS_TECHNICAL.md
- memory/05-modules/wuminapp/im/IM_TECHNICAL.md
- memory/05-modules/wumin/WUMIN_PQC_TECHNICAL.md
- memory/08-tasks/open/20260615-cpms-sfid-birthplace-election-scope.md

必须遵守：
- 使用中文沟通。
- 每一步先给方案，确认后执行。
- 不突破模块边界。
- 不在业务模块内实现或复刻投票流程；投票流程统一归属投票引擎。
- 默认不修改 `citizenchain/runtime/`；如确需修改 runtime，必须单独列明路径、内容、原因并获得二次确认。
- 不改用户或其他线程已有改动，不做无关重构。
- 不保留旧术语、旧路径、旧流程残留。
- 改白皮书后必须检查网页白皮书渲染。

输出物：
- docs/《白皮书》.md 正文补全。
- 必要时更新 website 白皮书渲染。
- 本任务卡进度回写。
- 残留术语、旧路径、旧流程清理。

验收标准：
- 白皮书核心概念与公民宪法、ADR、技术文档一致。
- 中文正文与英文说明一致。
- 白皮书不再混用旧清算模型和新清算模型。
- 白皮书中的源码路径均指向当前仓库路径。
- website 白皮书页面可正常渲染目录、表格、图片和新增正文。
- `git diff --check` 通过。

## 分步计划

1. 第 0 步：建立任务卡与白皮书基线审计。
2. 第 1 步：统一宪法/白皮书真源概念与术语。
3. 第 2 步：补全发行与经济模型。
4. 第 3 步：补全节点、清算与交易体系。
5. 第 4 步：补全治理、投票与运行时状态机。
6. 第 5 步：补全身份、CPMS/SFID、投票/参选资格边界。
7. 第 6 步：补全安全模型、钱包、PQC 与隐私通信。
8. 第 7 步：统一英文、图表、路径、网页渲染并验收。

## 第 0 步基线审计

状态：
- 已完成。

已确认白皮书源头：
- 白皮书 Markdown 真源：`docs/《白皮书》.md`
- 网站白皮书页面直接通过 raw import 引入该 Markdown：`website/src/pages/Whitepaper.tsx`
- 白皮书图片资产：`docs/assets/`

章节结构：
- 白皮书当前 713 行。
- 共有 6 个一级章节：总则、节点设置、发行与销毁、技术架构、运行时、其他。
- 多个章节只有 0 到 2 条要点，属于薄弱章节，尤其是：1.2、1.3、1.4、1.5、2.1、2.6、4.2、4.3、5.1、5.3、5.4、5.5、5.6、6.1 至 6.6。

必须优先修正的差异：
- `docs/《白皮书》.md:137` 仍写“国家名称与五民主义”，英文仍写 `Five Civic Principles`；公民宪法第三条已经改为“公民主义 / Citizenism”，白皮书必须同步。
- `docs/《白皮书》.md:270` 仍引用旧路径 `GMB/primitives/china/china_ch.rs`，应改为当前 `citizenchain/runtime/primitives/china/china_ch.rs`。
- `docs/《白皮书》.md:486` 仍引用旧路径 `primitives/src/count_const.rs`，应改为当前 `citizenchain/runtime/primitives/src/count_const.rs`。
- `docs/《白皮书》.md` 链下交易章节已改为当前 `citizenchain/runtime/primitives/china/china_cb.rs/main_account` 字段；后续仍需按当前清算模型整体重写制度表述。
- `docs/《白皮书》.md:218` 已写“全节点注册清算行节点”，但 `docs/《白皮书》.md:639` 到 `644` 仍按“省储行节点自行设定链下交易费率、手续费归省储行节点”表述；当前有效模型应统一为“注册清算行节点 + 收款方清算行主导 settlement”。
- 白皮书尚未系统说明 CPMS 离线实名真源、SFID 在线身份与投票资格、居住地投票范围、出生地参选范围、无资格公民从 SFID 删除等边界。
- 白皮书尚未说明 ADR-022 的 PQC 路线：当前 sr25519，未来通过 runtime 与钱包升级原地址在位切换到 ML-DSA，不换助记词、不换地址、不换余额归属。

可直接引用的当前真源：
- 清算模型当前有效描述：`memory/05-modules/citizenchain/node/offchain-clearing/L2_CLEARING_PROTOCOL.md`
- 投票引擎职责和状态机：`memory/05-modules/citizenchain/runtime/votingengine/VOTINGENGINE_TECHNICAL.md`
- CPMS/SFID 档案码与资格边界：`memory/05-modules/sfid/SFID-CPMS-QR-v1.md`
- SFID citizens 模块边界：`memory/05-modules/sfid/backend/citizens/CITIZENS_TECHNICAL.md`
- 抗量子唯一真源：`memory/04-decisions/ADR-022-unified-pqc-crypto.md`

第 0 步未修改：
- 未修改 `docs/《白皮书》.md`。
- 未修改 `website/`。
- 未修改 `citizenchain/runtime/`。

## 第 1 步：统一宪法/白皮书真源概念与术语

目标：
- 统一白皮书核心概念、术语和旧路径，先消除最明显的真源不一致。

预计修改：
- `docs/《白皮书》.md`
  - 将“国家名称与五民主义”改为“国家名称与公民主义”。
  - 将英文 `Five Civic Principles` 改为 `Citizenism`。
  - 将 1.4 中对应中文句式同步到公民宪法第三条当前表达。
  - 增补一小节“术语与命名约定”，统一公民链、公民币、公民钱包、公民、SFID、CPMS、清算行、投票资格、参选范围等中英文称谓。
  - 修正白皮书中已发现的旧源码路径引用。
- `memory/08-tasks/open/20260620-whitepaper-completion.md`
  - 回写第 1 步执行结果和验收。

不修改：
- 不改 website 代码，除非第 1 步正文导致 Markdown 渲染异常。
- 不改 runtime。
- 不改业务代码。

验收：
- `rg` 检查白皮书不再出现 `五民主义`、`Five Civic Principles`、`GMB/primitives/china`、旧 `primitives/src/count_const.rs` 引用。
- `git diff --check` 通过。

状态：
- 已完成。

已执行：
- 在 `docs/《白皮书》.md` 的 1.2 下新增 `1.2.1.术语与命名约定`。
- 统一术语：公民主义/Citizenism、公民链/CitizenChain、公民币/Citizen Coin、公民钱包/wumin、公民/wuminapp、SFID、CPMS、档案码、清算行、投票资格、投票范围、参选范围。
- 将 1.4 中“国家名称与五民主义 / Five Civic Principles”同步为“国家名称与公民主义 / Citizenism”，并按公民宪法第三条当前表达同步中英文。
- 将 `GMB/primitives/china/china_ch.rs` 改为 `citizenchain/runtime/primitives/china/china_ch.rs`。
- 将 `primitives/china/china_ch.rs/citizens_number` 改为 `citizenchain/runtime/primitives/china/china_ch.rs/citizens_number`。
- 将 `primitives/src/count_const.rs` 改为 `citizenchain/runtime/primitives/src/count_const.rs`。

第 1 步特意不处理：
- `docs/《白皮书》.md` 中链下交易章节已经清理旧主账户路径并改为 `china_cb.rs/main_account`；“省储行节点自行设定费率/手续费归省储行”仍属于第 3 步“节点、清算与交易体系”整体重写范围。

验收结果：
- `rg` 已确认 `五民主义`、`Five Civic Principles`、`GMB/primitives/china`、旧 `primitives/src/count_const.rs` 不再出现在白皮书。
- 精确残留检查已确认旧 `primitives/china/china_ch.rs` 独立路径不再出现在白皮书。
- `git diff --check -- docs/《白皮书》.md memory/08-tasks/open/20260620-whitepaper-completion.md` 通过。
- `website` 执行 `npm run build` 通过，白皮书 Markdown raw import 和新增术语表可正常参与生产构建。
- `website/dist` 未产生 git 改动。

## 第 2 步：补全发行与经济模型

目标：
- 补全发行与经济模型，统一发行总览、固定发行、释放条件、账户归属、销毁、ED 与费用流向。

预计修改：
- `docs/《白皮书》.md`
  - 在 1.3 或 3.0 增补发行总览：已确定发行总额、决议发行的治理边界、各发行类型是否流通、是否质押、是否按时间/区块释放。
  - 规范两和基金金额写法，将 `1958,1850,1966.00元` 改为易读的 `195,818,501,966.00元`，保留 1958/1850/1966 的象征解释。
  - 补全 3.1 至 3.7 的触发条件、执行模块、持有账户、释放条件、停止条件、销毁规则和 ED 规则。
  - 同步中英文表格，避免中文和英文单位解释不一致。
- `memory/08-tasks/open/20260620-whitepaper-completion.md`
  - 回写第 2 步执行结果和验收。

不修改：
- 不改 runtime。
- 不改发行代码。
- 不改 website 代码，除非表格渲染出现问题。

验收：
- `rg` 检查金额旧格式残留和发行章节旧表述。
- `git diff --check` 通过。
- 视修改幅度执行 website 白皮书构建或渲染检查。

状态：
- 已完成。

已执行：
- 在白皮书目录中新增 `3.0.发行与销毁总览`。
- 将 1.3.2 发行表从简单金额表补全为“金额/释放状态/执行模块/账户边界”表。
- 将两和基金金额从 `1958,1850,1966.00元` 规范为 `195,818,501,966.00元`，同时保留 1958、1850、1966 的象征解释。
- 补充固定发行合计：`2,229,386,218,778.00元`，明确该合计不包含后续决议发行。
- 补全创世发行、省储行创立发行与质押利息、全节点发行、公民发行、两和基金发行、决议发行、销毁与 ED 的触发条件、停止条件、账户归属、执行边界。
- 同步 5.4 发行模组中的重复旧表述，避免 3.6 与 5.4.4 对决议发行发起方、收款账户和校验边界描述冲突。
- 清理 5.4 中“资助民运人士”的旧表述，统一为“资助公民运动人士”。

第 2 步未修改：
- 未修改 `citizenchain/runtime/`。
- 未修改发行代码。
- 未修改 `website/` 源码。

验收结果：
- `rg` 已确认白皮书不再出现 `1958,1850,1966`、旧两和基金金额格式、`民运人士`、`democracy-movement`、`Five Civic Principles`、`五民主义`、`GMB/primitives` 和已处理的旧 primitives 路径残留。
- `rg` 已确认白皮书包含固定发行合计、两和基金规范金额、决议发行防重放/限额/允许收款账户集合、ED 与一次性身份奖励边界。
- `git diff --check -- docs/《白皮书》.md memory/08-tasks/open/20260620-whitepaper-completion.md` 通过。
- `website` 执行 `npm run build` 通过，白皮书新增表格和章节可正常参与生产构建。
- `website/dist` 未产生 git 改动。

## 第 3 步：补全节点、清算与交易体系

目标：
- 补全节点、清算与交易体系，统一白皮书中的全节点、清算行、链上交易、链下清算、手续费归属和旧清算路径。

预计修改目录：
- `docs/`
  - 用于修改 `docs/《白皮书》.md`，补全 2.x、5.5 交易模组和 6.x 相关说明；清理旧清算模型和旧路径残留。涉及文档修改，不涉及代码。
- `memory/08-tasks/`
  - 用于回写本任务卡第 3 步执行结果、验收记录和下一步方案。涉及文档修改，不涉及代码。
- `website/`
  - 默认不修改源码；仅在白皮书 Markdown 渲染或构建暴露问题时再按需修复。可能涉及前端代码或样式，但第 3 步先以文档修正为主。

预计修改：
- `docs/《白皮书》.md`
  - 补全 2.1 至 2.6 的节点职责：国储会权威节点、省储会权威节点、省储行权益节点、归档全节点、普通全节点、通信全节点、公民轻节点和访客轻节点。
  - 统一“全节点注册清算行”的准入条件、职责边界和收益边界。
  - 重写 5.5 交易模组，区分链上交易与链下清算：链上交易费按 80/10/10 分配，链下清算费由清算网络规则处理，不进入链上 80/10/10 分配。
  - 清理链下交易章节中仍按省储行节点主导清算、旧主账户路径、手续费归省储行节点的旧流程残留。
  - 同步中英文说明。

不修改：
- 不改 `citizenchain/runtime/`。
- 不改清算代码。
- 不新增清算模块目录。

验收：
- `rg` 检查白皮书不再出现旧清算路径、旧清算主体和旧手续费归属表述。
- `git diff --check` 通过。
- 执行 `website` 构建，确认白皮书页面仍可正常构建。

状态：
- 已完成。

已执行：
- 重写白皮书 2.1 节点概览表，新增清算行节点列，并将省储行权益节点职责从“链下交易验证”改为“永久质押与省储行治理”。
- 修正 2.4 省储行权益节点：删除省储行费用账户作为链下清算网络账户、接收用户存款、支付链下付款、收取支付交易费等旧职责；删除“更换链下交易验证密钥”旧职责。
- 补全 2.5 全节点：明确清算行不是新机构类型，而是完成 SFID 资格和链上节点声明后的全节点链下清算角色；补充 PeerId、RPC 端点、收款方清算行手续费归属。
- 重写 5.5.1 链上交易：补全 0.1%、最低 0.1 元、投票固定 1 元、未知费用类型拒绝、80%:10%:10% 分账和失败份额销毁规则。
- 重写 5.5.2 链下交易：将旧省储行清算模型改为注册清算行全节点模型；补充绑定、充值、提现、切换清算行、PaymentIntent、`submit_offchain_batch_v2`、收款方清算行主导 settlement、链下手续费 0.01% 至 0.1% 且最低 0.01 元。
- 补全 5.5.3 多签名链上交易：明确多签转账只处理链上授权转账，不处理链下清算流程。
- 补充 6.2 公民：说明公民提供清算行绑定、充值、提现和扫码支付入口，并清理错误旧称呼。

第 3 步未修改：
- 未修改 `citizenchain/runtime/`。
- 未修改清算代码、交易代码、节点代码。
- 未修改 `website/` 源码。

验收结果：
- `rg` 已确认白皮书不再出现旧省储行清算模型、旧链下交易接口、旧远程待处理状态和错误产品称谓等残留。
- `rg` 已确认白皮书包含 `注册清算行`、`收款方清算行`、`PaymentIntent`、`submit_offchain_batch_v2`、`80%`、`10%`、`安全基金`、`清算行节点` 和链下交易手续费边界。
- `git diff --check -- docs/《白皮书》.md memory/08-tasks/open/20260620-whitepaper-completion.md` 通过。
- `website` 执行 `npm run build` 通过，白皮书节点表和交易章节可正常参与生产构建。
- `website/dist` 未产生 git 改动。

## 第 4 步：补全治理、投票与运行时状态机

目标：
- 补全治理、投票与运行时状态机，统一白皮书中的内部投票、联合投票、公民投票、提案生命周期、模块职责边界和投票引擎统一管控规则。
- 修正联合投票边界：联合公投阶段属于联合投票模块内部阶段，不是公民投票模块；公民投票模块是独立模块。

预计修改目录：
- `docs/`
  - 用于修改 `docs/《白皮书》.md`，补全 5.2 投票引擎、5.3 治理模组、相关运行时状态机说明；清理业务模块内复刻投票流程的旧表述。涉及文档修改，不涉及代码。
- `memory/08-tasks/`
  - 用于回写本任务卡第 4 步执行结果、验收记录和下一步方案。涉及文档修改，不涉及代码。
- `website/`
  - 默认不修改源码；仅在白皮书新增表格或流程图导致构建/渲染问题时再按需修复。

预计修改：
- `docs/《白皮书》.md`
  - 补全投票引擎为唯一投票流程归属：内部投票模块、联合投票模块、公民投票模块均归投票引擎统一管控。
  - 明确联合投票模块包含机构联合投票阶段和联合公投阶段，联合公投阶段不得写成公民投票模块。
  - 补全提案生命周期：创建、投票中、通过、否决、执行中、已执行、执行失败可重试、超期否决或进入公投。
  - 补全内部投票、联合投票和联合公投的适用范围、阈值和回调边界。
  - 对 5.3 治理模组逐项补足职责：协议升级、管理员更换、决议销毁、GRANDPA 密钥更换、个人多签管理、机构多签管理。
  - 清理“业务模块自行处理投票流程”的残留表达，统一改为业务模块只提交提案数据并接收投票引擎回调。
  - 同步中英文说明。

不修改：
- 不改 `citizenchain/runtime/`。
- 不改投票引擎代码。
- 不改治理代码。

验收：
- `rg` 检查白皮书不存在业务模块自行执行投票流程、旧投票流程分散表述。
- `rg` 确认投票引擎统一管控、三类投票模块、联合公投阶段、提案状态机和回调边界均已出现。
- `git diff --check` 通过。
- 执行 `website` 构建，确认白皮书页面仍可正常构建。

状态：
- 已完成。

已执行：
- 重写白皮书 5.2 投票引擎说明，明确投票引擎是链上投票流程唯一归属，业务模块不得自行实现投票、计票、人口快照、资格判断、状态推进、执行重试或取消流程。
- 明确投票引擎包含三个独立模块：内部投票模块、联合投票模块、公民投票模块。
- 修正联合投票边界：联合投票模块包含“机构联合投票阶段”和“联合公投阶段”；机构联合投票非全票或超时进入联合投票模块内部的联合公投阶段，不进入公民投票模块。
- 明确公民投票模块是独立模块，主要用于公权机构选举等纯公民投票事项，不与联合公投阶段混写。
- 新增投票模块边界表，补全内部投票、联合投票、公民投票的适用事项、阈值和业务模块边界。
- 重写联合投票流程图，将“内部投票阶段”修正为“机构联合投票阶段”，将“进入联合公投阶段”明确为“进入联合投票模块内的联合公投阶段”。
- 新增提案状态机：`VOTING -> PASSED / REJECTED`，`PASSED -> EXECUTED / EXECUTION_FAILED`；明确 `PASSED` 是可执行/可重试态，不是终态。
- 补全 callback 与 retry 边界：投票通过后由投票引擎回调 owner 模块，owner 只返回 `Executed`、`RetryableFailed` 或 `FatalFailed` 等结果；手动重试和取消统一走投票引擎入口。
- 补全 5.3 治理模组的 owner / `MODULE_TAG` 边界，以及 runtime-upgrade、admins-change、resolution-destro、grandpakey-change、personal-manage、organization-manage 的职责和投票引擎回调关系。

第 4 步未修改：
- 未修改 `citizenchain/runtime/`。
- 未修改投票引擎代码。
- 未修改治理代码。
- 未修改 `website/` 源码。

验收结果：
- `rg` 已确认白皮书没有 `进入公民投票阶段`、`进入公民投票模块` 等把联合公投阶段写成公民投票模块的错误表述。
- `rg` 已确认白皮书包含 `投票引擎是链上投票流程的唯一归属`、`联合投票模块`、`联合公投阶段`、`公民投票模块`、`VOTING`、`PASSED`、`EXECUTED`、`EXECUTION_FAILED`、`callback`、`retry`、`ProposalOwner`、`MODULE_TAG` 和 `业务模块不得` 等关键边界。
- `git diff --check -- docs/《白皮书》.md memory/08-tasks/open/20260620-whitepaper-completion.md` 通过。
- `website` 执行 `npm run build` 通过，白皮书新增表格和状态机可正常参与生产构建。
- `website/dist` 未产生 git 改动。

## 第 5 步：补全身份、CPMS/SFID、投票资格与范围边界

状态：已完成。

目标：
- 补全身份、CPMS/SFID、档案码、投票资格、投票范围和参选范围边界，统一白皮书中的离线实名真源、在线身份系统、居住地投票、出生地参选和无资格公民剔除规则。

实际修改目录：
- `docs/`
  - 修改 `docs/《白皮书》.md`，补全 5.6.1 身份识别码校验、6.3 身份识别码系统、6.4 护照管理系统。涉及文档修改，不涉及代码。
- `memory/08-tasks/`
  - 回写本任务卡第 5 步执行结果、验收记录和第 6 步方案。涉及文档修改，不涉及代码。
- `website/`
  - 未修改源码，仅执行构建验收，确认白皮书新增内容可正常参与生产构建。

实际修改：
- `docs/《白皮书》.md`
  - 在 5.6.1 补充身份识别码校验只处理身份绑定凭证、投票资格凭证和人口快照凭证，不内嵌投票流程；投票创建、资格快照、计票和通过/否决判定统一归属投票引擎。
  - 在 6.3 补全 SFID 录入档案码时必须校验档案码签名、CPMS 授权分区、加密地区封印、公民状态、投票资格、投票账户、钱包地址、公钥和签名算法。
  - 明确 SFID 只保存可验证身份 ID、CPMS 授权安装分区、居住地行政区代码、出生地行政区代码、选举登记精度和钱包绑定关系，不保存姓名等完整实名档案。
  - 明确 CPMS 安装码中的省市只表示被授权安装和签发档案码的公安局所属省市，不等同于公民居住地或出生地。
  - 明确投票范围按居住地判断：全国、居住省、市级精度下的居住市、镇级精度下的居住镇。
  - 明确参选范围按出生地判断：全国、出生省、市级精度下的出生市、镇级精度下的出生镇，迁居不改变出生地对应的参选范围。
  - 明确 CPMS 新增公民时居住地可变，出生地从 SFID 唯一行政区划真源随包嵌入的只读行政区划数据中选择，保存后不得更改。
  - 明确档案码明文字段不得暴露姓名、证件、完整住址或中文地区名称；居住地和出生地只以行政区划代码写入加密地区封印。
  - 明确 CPMS 与 SFID 不在线直连；SFID 导入年度状态报告时重新校验状态、资格、账户、签名和有效期，校验不通过的数据删除，不保留影子旧记录。

第 5 步未修改：
- 未修改 `sfid/` 代码。
- 未修改 `cpms/` 代码。
- 未修改 `citizenchain/runtime/`。
- 未修改 `website/` 源码。

验收结果：
- `rg` 已确认白皮书包含 `投票范围按居住地判断`、`参选范围按出生地判断`、`投票账户`、`年度状态报告`、`行政区划代码`、`加密地区封印`、`安装码中的省市`、`SFID 唯一行政区划真源` 和 `投票引擎` 等关键边界。
- `rg` 已确认白皮书没有把出生地写成投票范围、把居住地写成参选范围的混乱表述。
- `git diff --check -- docs/《白皮书》.md memory/08-tasks/open/20260620-whitepaper-completion.md` 通过。
- `website` 执行 `npm run build` 通过，白皮书页面可正常参与生产构建。
- `website/dist` 未产生 git 改动。

## 第 6 步：补全钱包、安全、后量子迁移与去中心化通信边界

状态：已完成。

目标：
- 补全钱包、安全、后量子迁移和去中心化通信边界，让 6.1 公民钱包、6.2 公民、以及相关安全说明和仓库实现保持一致。

实际修改目录：
- `docs/`
  - 修改 `docs/《白皮书》.md`，补全公民钱包、公民、钱包签名、冷/热钱包边界、后量子签名迁移、去中心化通信和隐私安全说明。涉及文档修改，不涉及代码。
- `memory/08-tasks/`
  - 回写第 6 步执行结果、验收记录和第 7 步方案。涉及文档修改，不涉及代码。
- `website/`
  - 未修改源码，仅执行构建验收，确认白皮书新增内容可正常参与生产构建。

实际修改：
- `docs/《白皮书》.md`
  - 补全 6.1 公民钱包：明确 wumin 是公民链离线冷钱包，只负责账户创建、账户导入、助记词和私钥本地保存、离线签名、扫码识别签名请求和输出签名结果。
  - 明确公民钱包不承担轻节点、链上查询、交易广播、治理浏览、即时通信、清算行绑定或投票交互职责。
  - 明确二维码签名请求必须展示账户、收款方、金额、治理动作、登录动作或身份绑定动作等用户可理解语义，不得诱导签署黑盒载荷。
  - 补全后量子签名升级：以 ADR-022 为唯一真源，未来通过公民链 runtime 升级和公民钱包、公民客户端升级，在不更换助记词、钱包、账户地址和余额归属的前提下，在位切换到 ML-DSA-65；AccountId 仍为身份锚点，签名算法只是授权方式。
  - 补全 6.2 公民：明确 wuminapp 是公民链轻节点、热钱包、链上状态查询、交易提交、身份绑定、公民投票、治理交互、清算支付和去中心化通信入口。
  - 明确热钱包负责联网广播、余额查询、清算行绑定、扫码支付和投票交互；资产、身份绑定、投票或治理敏感动作必须经过账户签名。
  - 明确钱包私钥不得交给 SFID、CPMS、通信全节点、清算行、网站前端或任何链下服务。
  - 补全去中心化通信：通信不上链，不依赖 SFID，不使用中心化消息服务器；通信全节点是私人节点，只服务自己的手机和收件箱，只保存密文 mailbox，不解密消息，不替第三方存消息，不做公共中继。
  - 明确聊天账户使用钱包地址，IM 设备密钥与钱包账户分层，钱包私钥只用于证明设备属于该钱包地址，不用于 OpenMLS 消息加密。
  - 补全隐私边界：CPMS 离线保存完整实名档案，SFID 在线只保存可验证身份、资格、行政区代码和钱包绑定关系，链上只接收账户地址、签名、凭证、哈希和必要状态。

第 6 步未修改：
- 未修改 `wumin/` 代码。
- 未修改 `wuminapp/` 代码。
- 未修改 `sfid/` 代码。
- 未修改 `cpms/` 代码。
- 未修改 `citizenchain/runtime/`。
- 未修改 `website/` 源码。
- 未新增安全协议或兼容旧流程。

验收结果：
- `rg` 已确认白皮书包含 `公民钱包`、`wumin`、`公民（wuminapp）`、`冷钱包`、`热钱包`、`离线签名`、`PaymentIntent`、`后量子`、`ML-DSA-65`、`AccountId`、`OpenMLS`、`通信全节点`、`中心化消息服务器`、`隐私边界` 和 `钱包私钥` 等关键边界。
- `rg` 已确认白皮书没有把 wumin 写成轻节点、没有把 wuminapp 写成离线冷钱包、没有错误产品称谓。
- `rg` 已确认白皮书没有把钱包私钥写成可交给通信节点、SFID、CPMS、清算行或链下服务。
- `git diff --check -- docs/《白皮书》.md memory/08-tasks/open/20260620-whitepaper-completion.md` 通过。
- `website` 执行 `npm run build` 通过，白皮书页面可正常参与生产构建。
- `website/dist` 未产生 git 改动。

## 第 7 步：白皮书全局收口

状态：已完成。

目标：
- 对白皮书做全局收口，统一章节编号、目录、术语、中英文一致性、旧残留和构建验收，确保前 1-6 步补全内容形成一份可发布的完整白皮书。

实际修改目录：
- `docs/`
  - 修改 `docs/《白皮书》.md`，做全局术语、中英文、目录、章节衔接和残留清理。涉及文档修改，不涉及代码。
- `memory/08-tasks/`
  - 回写第 7 步执行结果、最终验收记录和任务完成状态。涉及文档修改，不涉及代码。
- `website/`
  - 未修改源码，仅执行构建验收，确认白皮书页面可正常参与生产构建。

实际修改：
- `docs/《白皮书》.md`
  - 修正目录 2.6 的显示文字，将半角冒号统一为正文标题使用的中文冒号，并通过脚本确认目录锚点全部可匹配正文标题。
  - 将 4 章中“感谢 Polkadot 团队的奉献”从未编号二级标题改为普通说明条目，避免干扰目录和章节编号结构。
  - 清理发布稿中的过渡口径，把“旧称谓/legacy/no longer/不再作为旧模型对比”等表述改为目标状态陈述。
  - 同步修正管理员更换、机构多签、决议发行、链下交易、CPMS/SFID 年度状态等段落的中英文表述。
  - 保留发行达到上限后的制度含义，同时改写为“后续认证节点无奖励”“此后全节点铸造新块不获得铸块奖励”等目标态描述。
  - 复核投票引擎、联合投票/联合公投、公民投票、居住地投票范围、出生地参选范围、钱包安全和通信隐私边界，未发现互相矛盾。

第 7 步未修改：
- 未修改任何业务代码。
- 未修改 `citizenchain/runtime/`。
- 未修改 `sfid/`、`cpms/`、`wumin/`、`wuminapp/`。
- 未修改 `website/` 源码。
- 未新建白皮书拆分文件。
- 未新增协议或兼容旧流程。

最终验收结果：
- 目录锚点脚本检查通过，白皮书目录链接均可匹配正文标题。
- `awk` 检查条目英文对应关系通过，未发现新增条目缺英文说明。
- `rg` 已确认白皮书关键旧残留为 0。
- `rg` 已确认白皮书不出现错误产品名、不出现联合公投阶段进入公民投票模块、不出现出生地投票范围/居住地参选范围、不出现钱包私钥错误托管或通信节点错误职责。
- `rg` 已确认公民主义、公民链、公民币、公民钱包、公民、SFID、CPMS、清算行、投票引擎、联合投票、公民投票、投票范围、参选范围、后量子签名等关键目标术语均出现。
- `git diff --check -- docs/《白皮书》.md memory/08-tasks/open/20260620-whitepaper-completion.md` 通过。
- `website` 执行 `npm run build` 通过，白皮书页面可正常参与生产构建。
- `website/dist` 未产生 git 改动。

任务状态：
- 白皮书补全任务完成。

## 追加修正：白皮书列表英文另起一行显示

状态：已完成。

目标：
- 将白皮书列表项中的英文说明稳定显示在中文下面，避免 Markdown/GitHub 预览把缩进的英文 `<span>` 接在中文后面同段显示。
- 保持网站页面中英文之间原有紧凑间距，不因 Markdown 空行额外拉开视觉距离。

实际修改目录：
- `docs/`
  - 修改 `docs/《白皮书》.md`，给列表项内 `whitepaper-en` 英文说明前补空行，使 Markdown 渲染为独立段落。涉及文档修改，不涉及代码。
- `website/`
  - 修改 `website/src/index.css`，压掉白皮书列表项内段落默认 margin，避免空行导致中英文视觉间距变大。涉及前端样式修改，不涉及业务逻辑。
- `memory/08-tasks/`
  - 回写本次追加修正、修改范围和验收结果。涉及文档修改，不涉及代码。

实际修改：
- `docs/《白皮书》.md`
  - 批量在列表项内 `whitepaper-en` 英文说明前补空行，让列表中文和英文在 Markdown 渲染层面形成上下两段。
- `website/src/index.css`
  - 增加白皮书列表项内段落间距控制，清除 `li > p` 的默认 margin。
  - 进一步将列表项段落行高收紧为 1.55，将 `.whitepaper-en` 顶部间距收紧为 1px、行高收紧为 1.5，减少中文和英文之间的视觉间隔。
  - 根据截图复核后改为“组内紧、组间松”：同一条列表内中文和英文用 `line-height: 1.3`、`margin-top: -3px`、列表内英文 `line-height: 1.34` 压紧；不同列表项之间改由 `li` 的 `18px` 下边距拉开。

未修改：
- 未修改任何业务代码。
- 未修改 `citizenchain/runtime/`。
- 未修改 `sfid/`、`cpms/`、`wumin/`、`wuminapp/`。
- 未新建文件或目录。

验收结果：
- 脚本检查 `docs/《白皮书》.md` 中 184 处列表内 `whitepaper-en` 英文说明前均已有空行，未发现仍与上一行中文相邻的条目。
- 使用 `marked` 抽样渲染确认，补空行后英文说明会生成列表项内独立段落。
- 本地 Vite 服务检查确认，白皮书页面运行态 CSS 已包含列表项段落 `line-height: 1.55`、英文说明 `margin-top: 1px` 和英文说明 `line-height: 1.5`。
- 截图复核后再次调整列表视觉层级，本地 Vite 服务检查确认运行态 CSS 已包含 `li { margin: 0 0 18px; }`、列表段落 `line-height: 1.3`、同组英文段 `margin-top: -3px` 和列表英文 `line-height: 1.34`。
- `rg` 已确认白皮书未重新出现错误产品名、联合公投错误阶段、出生地/居住地投票参选边界反写、钱包私钥托管错误或通信全节点解密消息等旧残留。
- `git diff --check -- docs/《白皮书》.md website/src/index.css` 通过。
- `website` 执行 `npm run build` 通过，白皮书页面可正常参与生产构建。

## 追加修正：白皮书中英文结构级分组

状态：已完成。

目标：
- 标题类英文（如 `1. General Principles`、`1.1. Purpose`）必须显示在对应中文标题下面，分割线必须位于中英文标题组下方，不能夹在中文和英文之间。
- 列表项中同一句中文和英文必须紧挨着显示；英文和下一条中文之间保留较大条目间距。

实际修改目录：
- `website/`
  - 修改 `website/src/pages/Whitepaper.tsx`，在白皮书渲染阶段规整 Markdown 生成的标题和列表 HTML 结构。涉及前端渲染代码修改，不涉及业务模块。
  - 修改 `website/src/index.css`，针对标题组和双语列表项设置明确样式。涉及前端样式修改，不涉及业务模块。
- `memory/08-tasks/`
  - 回写本次结构级修复、修改范围和验收结果。涉及文档修改，不涉及代码运行逻辑。

实际修改：
- `website/src/pages/Whitepaper.tsx`
  - 将标题下一行的 `whitepaper-title-en` / `whitepaper-heading-en` 收进对应 `h1/h2/h3` 内，形成中文标题与英文标题同组结构。
  - 将 Markdown 生成的 `li > p + p > .whitepaper-en` 规整为 `li.whitepaper-bilingual-item > .whitepaper-li-zh + .whitepaper-en`，避免同一列表项中英文被段落包装撑开。
- `website/src/index.css`
  - 将标题分割线放到 `h1/h2/h3` 自身底部，使分割线位于中英文标题组下方。
  - 增加 `whitepaper-bilingual-item`、`whitepaper-li-zh` 的样式，让同一条中文和英文压紧，列表项之间继续由 `li` 下边距拉开。
  - 清理结构规整前遗留的 `li > p + p` 段落间距 fallback，页面样式只按新的双语列表项结构生效。

未修改：
- 未修改 `docs/《白皮书》.md` 正文内容。
- 未修改任何业务代码。
- 未修改 `citizenchain/runtime/`。
- 未修改 `sfid/`、`cpms/`、`wumin/`、`wuminapp/`。
- 未新建文件或目录。

验收结果：
- 脚本复现白皮书渲染流程确认，64 个标题均已生成标题组结构，63 个章节英文均在对应标题内部，未发现分离在标题外的 `whitepaper-heading-en`。
- 脚本复现白皮书渲染流程确认，列表双语项已规整为 `whitepaper-bilingual-item`，未发现残留的 `li > p + p > .whitepaper-en` 结构。
- `website` 执行 `npm run build` 通过。
- 本地 Vite 服务运行态资源检查通过，页面源码包含标题组和双语列表项规整逻辑，CSS 包含标题组、`whitepaper-bilingual-item` 和 `whitepaper-li-zh` 样式，且不再包含旧的 `li > p + p` 段落间距 fallback。
- Chrome headless 打开 `http://127.0.0.1:5174/whitepaper` 后 DOM 验收通过：`1.总则` 与 `1. General Principles` 位于同一 `h1` 内，`1.1.目的` 与 `1.1. Purpose` 位于同一 `h2` 内，列表项已生成 `whitepaper-bilingual-item`，未发现残留的 `li > p + p > .whitepaper-en` 结构。

## 最终修正：白皮书源文档统一排版结构

状态：已完成。

目标：
- 所有打开白皮书的入口必须使用相同排版结构，包括 VS Code/GitHub Markdown 预览、官网白皮书页和公民链桌面节点白皮书 tab。
- 不能只在官网或桌面端渲染层做补救；`docs/《白皮书》.md` 必须作为唯一真源直接表达中英文分组关系。

实际修改目录：
- `docs/`
  - 修改 `docs/《白皮书》.md`，把标题英文合并到同一 Markdown 标题行，把列表项英文合并到同一列表项行，均使用 `<br><span ...>` 表达换行。涉及文档源结构修改。
- `website/`
  - 修改 `website/src/pages/Whitepaper.tsx` 和 `website/src/index.css`，按新的源结构解析标题副文本和列表英文，删除官网侧双语列表二次规整逻辑。涉及官网展示代码和样式修改。
- `citizenchain/node/frontend/`
  - 修改 `other/other-tabs/LocalDocViewer.tsx`、`app/styles/global.css` 和重新生成 `generated/local-docs.generated.ts`，使公民链桌面节点白皮书 tab 消费同一源结构。涉及桌面节点前端展示代码、样式和生成物更新。
- `memory/05-modules/citizenchain/node/`
  - 更新白皮书 tab 和首页模块技术文档，明确白皮书源文档必须自带统一中英文排版结构。涉及模块文档修改。
- `memory/08-tasks/`
  - 回写本次最终修正、修改范围和验收结果。涉及任务文档修改。

实际修改：
- `docs/《白皮书》.md`
  - 将 65 个标题中英文对改为 `# 中文标题<br><span class="whitepaper-heading-en">English Title</span>` 或标题页对应 `whitepaper-title-en`。
  - 将 181 个列表中英文对改为 `* 中文内容<br><span class="whitepaper-en">English content</span>`。
  - 清理 3 个非列表英文说明的误缩进。
- `website/src/pages/Whitepaper.tsx`
  - 标题解析改为从同一标题行中的 `<br><span>` 读取中文标题和英文副标题。
  - 删除官网侧 `normalizeWhitepaperHtml` 双语列表规整逻辑，避免官网与源文档形成两套排版规则。
- `website/src/index.css`
  - 列表项内 `whitepaper-en` 直接按源文档结构显示，清理 `whitepaper-bilingual-item` / `whitepaper-li-zh` 样式残留。
- `citizenchain/node/frontend/other/other-tabs/LocalDocViewer.tsx`
  - 目录剥离和文档标题识别改为只读取标题主文本，避免 `目录<br>Table of Contents` 被误判为普通章节。
  - 删除标题英文搬移逻辑，桌面端白皮书 tab 直接消费源文档中的标题组结构。
- `citizenchain/node/frontend/app/styles/global.css`
  - 收紧列表项内英文行距，使同一条中文和英文紧挨着显示。
- `citizenchain/node/frontend/generated/local-docs.generated.ts`
  - 通过 `node scripts/generate-local-docs.mjs` 重新生成，内置新的白皮书源结构。

未修改：
- 未修改任何 `citizenchain/runtime/` 文件。
- 未修改 `sfid/`、`cpms/`、`wumin/`、`wuminapp/`。
- 未新建文件或目录。

验收结果：
- 脚本检查 `docs/《白皮书》.md`：65 个标题英文均已内联到标题行，181 个列表英文均已内联到列表项，旧的“标题下一行英文”“列表空行英文段落”和误缩进英文均为 0。
- 使用 `marked` 按标准 Markdown 渲染抽样确认：`1.总则` 与 `1. General Principles` 位于同一个 `h1`，`1.1.目的` 与 `1.1. Purpose` 位于同一个 `h2`，列表项中文和英文位于同一个 `li`，未生成 `li > p + p > .whitepaper-en`。
- 已重新生成公民链桌面节点白皮书 tab 的 `generated/local-docs.generated.ts`。
- 官网执行 `npm run build` 通过；Chrome headless 打开 `http://127.0.0.1:5174/whitepaper` 验收通过，标题英文位于同一标题节点内，列表英文位于同一列表项内，未发现 `whitepaper-bilingual-item` 或 `li > p + p > .whitepaper-en`。
- 公民链桌面节点前端执行 `npm run build` 通过；Chrome DevTools 自动打开 `http://127.0.0.1:5175/` 并点击“白皮书”tab 后 DOM 验收通过，`1.总则` 与 `1. General Principles` 位于同一 `h1`，`1.1.目的` 与 `1.1. Purpose` 位于同一 `h2`，列表英文为同一 `li` 的直接子节点，目录未误包含“目录”章节。
- 本地 Vite 服务已关闭，Chrome headless 临时目录已清理。
