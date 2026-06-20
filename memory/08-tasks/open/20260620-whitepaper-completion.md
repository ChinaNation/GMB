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
- `docs/《白皮书》.md:643` 仍引用 `primitives/src/shengbank_nodes_const.rs/main_address`，需要按当前账户/清算模型重新核对后改写。
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
- `docs/《白皮书》.md` 中链下交易章节的 `primitives/src/shengbank_nodes_const.rs/main_address` 与“省储行节点自行设定费率/手续费归省储行”绑定，属于第 3 步“节点、清算与交易体系”整体重写范围，不能只替换路径造成语义假同步。

验收结果：
- `rg` 已确认 `五民主义`、`Five Civic Principles`、`GMB/primitives/china`、旧 `primitives/src/count_const.rs` 不再出现在白皮书。
- 精确残留检查已确认旧 `primitives/china/china_ch.rs` 独立路径不再出现在白皮书。
- `git diff --check -- docs/《白皮书》.md memory/08-tasks/open/20260620-whitepaper-completion.md` 通过。
- `website` 执行 `npm run build` 通过，白皮书 Markdown raw import 和新增术语表可正常参与生产构建。
- `website/dist` 未产生 git 改动。

## 第 2 步待确认方案

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
