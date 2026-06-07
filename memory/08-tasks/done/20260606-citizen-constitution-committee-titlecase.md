# 任务卡：公民宪法英文委员会专名 Title Case（二次统一 Bucket A）

## 任务需求

承接 `20260606-citizen-constitution-naming-fix`（#9 大小写）。用户二次统一只选 Bucket A：把 reserve / self-governing / education 三类委员会的省市镇校级英文专名统一 Title Case。不动 government / administrative region（Bucket B）与市局 Municipal 前缀（Bucket C）。

## 修改范围

- `citizenchain/runtime/primitives/src/CitizenConstitution.html`

## 执行记录

- perl 全局 Title Case（约 127 处）：
  - reserve：provincial reserve committee(s)(21) → Provincial Reserve Committee(s)。
  - self-governing：municipal(27)/town(20)/municipal citizen(2)/town citizen(2) self-governing committee(s) + generic（popularly elected ... members, 4）→ Title Case。
  - education：municipal(16)/school(25)/university school(6)/municipal citizen(2)/school citizen(1) education committee(s) + generic plural（held by ...，1）→ Title Case。
- 保持不变：已大写的 National/Citizen Education Committee、National Reserve Committee、Reserve Committee 等；self-governing government（自治政府，非委员会）；Bucket B 的 government / administrative region。

## 验证记录

- 残留小写委员会专名 `grep -oE "[a-z][a-z-]* (reserve|self-governing|education) committee"` = **0**。
- 同句一致样例：
  - 第8条第四款：`the National Reserve Committee, Provincial Reserve Committees, and Citizen Reserve Banks`。
  - 第48条：`popularly elected Self-Governing Committee members`。
- 未误伤：self-governing government 12、provincial administrative region 32、federal government 17 均保持小写不变（Bucket B/C 本轮不做）。
- 结构完好：article 锚点 140、`<script>` 1。

## 后续（非本卡范围）

- Bucket B（government / administrative region）、Bucket C（市局 Municipal 前缀）按用户裁定不做。
- 与父卡一并，正式链生效需发布一次 runtime 升级（setCode）；`citizen_constitution_blake2_256` 摘要将变化。
