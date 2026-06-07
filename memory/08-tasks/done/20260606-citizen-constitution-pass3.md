# 任务卡：公民宪法第三轮修订（第34条护照主体澄清 + 院长译名）

## 任务需求

承接前两卡的复检。本轮两项：
1. 第34条护照/入籍主体澄清：把"公民安全局"明确为"任意公民安全局"，去掉"全国唯一机构 + 下设分支机构"的暗示，与第39条、第64条（市级局）口径一致。
2. 院长 vs 总统英文译名是否区分。

## 修改范围

- `citizenchain/runtime/primitives/src/CitizenConstitution.html`

## 执行记录

- 第34条（已改，中英 4 处）：
  - CN 正文：由"中华民族联邦共和国公民安全局颁发" → "中华民族联邦共和国任意公民安全局颁发"。
  - CN 第一款：由"公民安全局下设的任意分支机构提出入籍申请" → "任意公民安全局提出入籍申请"。
  - EN 正文：issued by the Citizen Security Bureau → issued by any Citizen Security Bureau。
  - EN 第一款：submitted to any branch institution under the Citizen Security Bureau → submitted to any Citizen Security Bureau。
- 院长译名：用户选 A，**维持 `President of the X Yuan`（院长）/ `President`（总统）**，不改。理由：ROC 立法/司法/监察三院官方英译即 President of the X Yuan，与总统 President 靠完整官称区分，最正式；本宪法无行政院（无 Premier 场景），下级 Speaker/Chief Justice/Chairperson/Director 均已占用，"President of the X Yuan" 为最优。

## 验证记录

- 残留旧形态 0：`下设的任意分支机构` / `branch institution under the Citizen Security Bureau` 均 0。
- 新形态在位：`由中华民族联邦共和国任意公民安全局颁发`、`需向中华民族联邦共和国任意公民安全局提出入籍申请`、`issued by any Citizen Security Bureau ...`、`submitted to any Citizen Security Bureau ...`。
- 结构完好：article 锚点 140。

## 复检其余结论（归档）

- 调查署署长 2 届：总统提名所致，有意设计，不改。
- 院长/总统译名：维持惯例（见上），不改。
- 简繁自指（第5条正体字 vs 简体正文）：体例决策，不改。

## 后续

- 连同前两卡，正式链生效需发布一次 runtime 升级（setCode）；`citizen_constitution_blake2_256` 摘要将变化。
