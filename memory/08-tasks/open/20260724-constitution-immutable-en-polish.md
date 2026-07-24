# 公民宪法不可修改条款英文润色（第3条五民排比 + 第24条人类重复）

任务需求：
中英核对后，8 条不可修改条款（1/2/3/17/19/24/34/42）中文与英文含义全部正确、忠实、无漏译。仅两处英文有**纯润色**空间（非错误），用户确认优化：

| 条 | 字段 | 问题 | 旧值片段 | 新值片段 |
|---|---|---|---|---|
| 3 | body_en | 「五民」列表排比不齐（首项带动词名词化，其余三项省动词） | `... "Citizenism": governance of the State by citizens (mínzhì), democratic republicanism (mínzhǔ), citizens' rights (mínquán), a society of citizens' livelihood security (mínshēng), and the revival of ethnic cultures (mínzú).` | `... "Citizenism": the governance of the State by citizens (mínzhì), the practice of democratic republicanism (mínzhǔ), the protection of citizens' rights (mínquán), the building of a society of citizens' livelihood (mínshēng), and the revival of ethnic cultures (mínzú).` |
| 24 | body_en | `human rights of humanity` 重复拗口；两处「人类」译法不一 | `... to safeguard humanity's right to survival from infringement, and to safeguard human rights of humanity from infringement.` | `... to safeguard humankind's right to survival from infringement, and to safeguard the human rights of humankind from infringement.` |

排比对应：治理→the governance / 实行→the practice / 保障→the protection / 建设→the building / 复兴→the revival。中文不动。

所属模块：citizenchain / legislation-yuan（宪法创世内容 constitution.scale）

输入文档：
- citizenchain/runtime/public/legislation-yuan/src/constitution.scale（宪法全文 SCALE 二进制，唯一真源；无生成器）
- citizenchain/runtime/public/legislation-yuan/src/lib.rs（Chapter/Section/Article/Clause 结构 + genesis_build:442-483 从 CONSTITUTION_SCALE 现算不可修改清单）

必须遵守：
- 只改 number∈{3,24} 两条的 body_en；其余 139 条、两条的中文 body、标题、款一律不动
- 编码器安全闸：先断言 encode(decode(原文))==原文 逐字节相等（226347B），才动数据
- 补丁在**该条 body_en 字符串对象内**做定向子串替换（非全局二进制替换），curly 撇号用 ’ 转义，重音拼音（mínzhì 等）原样不动
- 节点守卫/ check-constitution-genesis.py 不硬编码文本或哈希 → 改后无需同步；条级摘要变化由 genesis 现算
- 重烤 chainspec / 重新创世属部署步骤，本次不管（开发期零用户）
- 白皮书 whitepaper.md:173 有同款五民英文（独立文档），本次不动，另行询问是否同步

输出物：
- 改后的 constitution.scale（单文件二进制）
- 一次性 decode→patch→re-encode 脚本（scratchpad，不入库）
- 无 .rs / 测试改动（清单常量 [1,2,3,17,19,24,34,42] 不变）

验收标准：
- 重新解码：仅第 3/24 条 body_en 为新值、其余全等、0 剩余字节
- cargo check -p legislation-yuan 通过（include_bytes! 仍可 genesis_build 解码）
- 不可修改清单 IMMUTABLE_CONSTITUTION_ARTICLES 不变；第十九条正文自述条号不变

## 进度

- [x] 编码器 + 安全闸（encode∘decode 逐字节幂等，226347B 相等）
- [x] 打补丁写回 constitution.scale（226347→226398，Δ+51B，手算吻合：第3条+45/第24条+6）
- [x] 重新解码校验：仅第 3/24 条 body_en 变、余 139 条全等、0 剩余；两条 body_en 637B/651B ≪ MaxTextLen 8192
- [x] `cargo test -p legislation-yuan constitution` 11/11 通过（含 decode_well_formed / genesis_seeds / 全部 immutable-guard）
- [x] 白皮书同步：whitepaper.md:173 五民英文改为新排比版；重跑 generate-local-docs.mjs 刷新生成物 local-docs.generated.ts（含 sha256）；全仓无旧版残留
- [ ] 部署：重烤 chainspec + 重新创世（开发期零用户，按 regenesis-deploy 流程，待用户触发）
