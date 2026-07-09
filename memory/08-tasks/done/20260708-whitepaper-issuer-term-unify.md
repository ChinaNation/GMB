# 白皮书发行方术语统一 + 发行方主体对齐宪法

## 任务需求
1. 白皮书「发行机构」统一改为「发行方」（术语，整篇统一）。
2. 1.3 节发行方取值：国家储委会 → 中华民族联邦共和国公民储备委员会联合会议。
3. 排查公民宪法是否有类似条款。

## 所属模块
- citizenweb：src/whitepaper.md（唯一真源）
- citizenchain：node/frontend/generated/local-docs.generated.ts（桌面端内置白皮书，由真源生成）

## 实现（均在 /Users/rhett/GMB）
- whitepaper.md 改 3 处（全部 发行机构 出处）：
  - L10 目录：`[1.3. 发行方](#13发行方)`
  - L132 标题：`## 1.3.发行方` + 英文 `1.3. Issuer`（发行机构 Issuing Institution → 发行方 Issuer，经用户确认）
  - L134 正文：`发行方为中华民族联邦共和国公民储备委员会联合会议；` / `The issuer is the Joint Meeting of the Citizen Reserve Committee of the Federal Republic of the China Nation.`（英文口径取自宪法）
- 其余 9 处 `国家储委会`（账户/创世管理员/提案权等执行角色）保持不动。
- 重跑 generate-local-docs.mjs，桌面端内置白皮书同步（含新表述、无残留）。

## 宪法排查结论（只读，无需改宪法）
- 宪法正文真源=链上 legislation-yuan law_id=0（constitution.scale，SCALE 编码）。
- 宪法已有权威条款且印证本次改动：「中华民族联邦共和国公民储备委员会联合会议是铸币权的最高决策机构」「法定货币新增发行由公民储备委员会联合会议决议发行」；国家公民储备委员会（=国家储委会简称）是行权机构。
- 即：发行/铸币的权威主体本就是联合会议，白皮书原「发行机构为国家储委会」与宪法不一致，本次改动是对齐宪法。
- 宪法无「发行机构/发行方」措辞，无需改宪法（改宪法=重新创世+受守卫条款）。

## 验证（2026-07-08）
- whitepaper.md：发行机构 0 处 / 发行方 3 处。
- 生成文件含新表述、发行机构 0 残留。
- `npm run build`、`npm run lint` 通过。
- 浏览器（main 5205）DOM 核对：1.3.发行方/Issuer 标题在，正文中英文新表述正确，全页无「发行机构」；控制台无 error。

## 结论
- 完成并验证。国家储委会=国家公民储备委员会（简称），非不一致，未额外改动。
