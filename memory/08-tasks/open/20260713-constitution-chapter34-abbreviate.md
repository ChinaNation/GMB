# 公民宪法第三/四章标题极简化（教委会 / 储委会）

任务需求：
把宪法第三章「公民教育委员会」、第四章「公民储备委员会」的**章标题**改为极简形式，与其他章(总则/政府/立法院/司法院/监察院)风格对齐。中英同改：

| 章 | 字段 | 旧值 | 新值 |
|---|---|---|---|
| 3 | title | 第三章 公民教育委员会 | 第三章 教委会 |
| 3 | title_en | Chapter III Citizen Education Committee | Chapter III Education Committee |
| 4 | title | 第四章 公民储备委员会 | 第四章 储委会 |
| 4 | title_en | Chapter IV Citizen Reserve Committee | Chapter IV Reserve Committee |

所属模块：citizenchain / legislation-yuan（宪法创世内容 constitution.scale）

输入文档：
- citizenchain/runtime/public/legislation-yuan/src/constitution.scale（宪法全文 SCALE 二进制，唯一真源）
- citizenchain/runtime/public/legislation-yuan/src/lib.rs（Chapter 结构 + genesis_build）
- citizenchain/node/src/core/constitution/render.rs（目录/正文章标题同取 chapter.title）

必须遵守：
- 只改 number∈{3,4} 两章的 title / title_en；节/条/款正文、正文里 148 处全称、同名机构(NED/NRC 注册名)一律不动
- 目录条目与正文章标题是同一 chapter.title 字段，同步变（预期）
- 编码器安全闸：先断言 encode(decode(原文))==原文 逐字节相等，才动数据
- 章标题不参与「条」编码 → 不可修改条文摘要/manifest 不受影响
- 创世/重烤 chainspec/重新创世本次不管（用户明确）

输出物：
- 改后的 constitution.scale（单文件二进制）
- 一次性 decode→patch→re-encode 脚本（scratchpad，不入库）
- 残留清理（无 .rs/测试改动）

验收标准：
- 重新解码：仅第 3/4 章中英标题为新值、其余 5 章全等、0 剩余字节
- cargo check -p legislation-yuan 通过（include_bytes! 仍可被 genesis_build 解码）
- 条级不可修改条文摘要与 check-constitution-genesis.py 口径不变

## 进度

- [x] 编码器 + 安全闸（encode∘decode 幂等，226387 字节逐字节相等）
- [x] 打补丁写回 constitution.scale（226387→226347，-40 字节，符合手算：两章中文各-12/英文各-8）
- [x] 解码验证 7 章标题（仅第3/4章中英变新值，其余5章+节/条数+条号区间全等，0 剩余字节；两章正文字节不变已断言）
- [x] cargo check -p legislation-yuan 通过（1.13s，include_bytes! 仍有效）
- [ ] 【延后·用户明确不管】重构 WASM + 重烤 chainspec + 重新创世后，跑 check-constitution-genesis.py 现场核对
