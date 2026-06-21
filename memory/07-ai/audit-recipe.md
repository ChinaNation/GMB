# GMB 审计任务铁律(audit recipe)

> 由 PR-C(2026-05-07)清链重启前彻底审计的踩坑经验直接产出。

## 背景

仓库审计常涉及多产品(citizenchain / citizenwallet / citizenapp / sfid / cpms)+ 大量历史叙述(ADR / 阶段记录 / OBSOLETE 标记)。把审计完全外包给 Explore subagent **会出大问题**:

- subagent prompt 里的"背景叙述"是经过我简化的二手信息
- subagent 据此做模式匹配,把所有命中"OLD 名"/"已废弃"字眼的位置全部标 [DEAD]
- 但很多命中其实**正在使用**或**故意保留为历史档案**(LEGACY md / 已激活的 ADR-007 三阶段)
- 报告聚合时若不复核,大批误判被当成结论吸收

清链重启前审计 v1 报告就吃了这个亏:6 个 subagent 标了 ~187 项,其中**约 60% 是误判**(把活跃概念当 dead code)。

## 铁律

### 1. subagent 输出 = 线索池(leads),不是结论(conclusions)

每条 finding **必须回原文核验** 之后才能进入正式报告:

- `[RENAME]` / `[STORAGE]`:打开 `file:line` 看上下文,确认是当前活跃命名还是历史叙述
- `[DEAD]`:`grep` 该符号是否真无 emit 路径 / 调用点;不能仅凭"注释提到 OBSOLETE"判断
- `[DOC]` 跨文件 / 跨产品:对比权威 ADR / project memory,确认是真漂移还是历史档案

### 2. 审计报告每条 finding 必须给「证据锚点」

格式: `[标记] file:line — 一句描述 (证据: ADR-000 第 N 节 / 项目记忆 X)`

无锚点的 finding **直接不写进正式报告**。

### 3. 误判要显式撤回,不能默不作声

报告必须有一节 **「v1 误判撤回」**,列每条最初列出后**经核验认定不成立**的 finding,以及推翻的依据。

否则下一轮审计的人会重复同款误判。

### 4. 历史 task card / OBSOLETE md 的处理边界

- `memory/08-tasks/done/*` :历史档案,**永远不动**(改了等于篡改历史)
- `memory/08-tasks/open/*` 但已超期 1 个月:加 aging banner,标 P3-2 待 owner 验证;不擅自移到 done/
- `*_LEGACY.md`:整文件标 LEGACY 是合法状态,**不当 dead code 删**
- ADR `状态:Accepted` + 阶段进度有 ✅:在跑的 work-in-progress,不当 OBSOLETE 标

### 5. 与代码强相关的 audit 必须 `cargo check` / `flutter analyze` 兜底

注释 / doc 改完后跑一遍编译 + 静态检查,确保没误碰逻辑。如果跑出新警告,要逐一定位到本次改动还是 pre-existing baseline。

### 6. fork/vendor 残留标记必须单独统计

`citizenapp/smoldot-pow/` 和 `citizenchain/node/vendor/` 属于收编 fork/vendor,保留上游注释脉络。普通清理任务只统计这些目录的残留标记,不和 GMB 自有代码清零门禁混算。具体边界见 `memory/07-ai/fork-vendor-baseline.md`。

## 适用范围

- 本铁律对所有"清扫类"任务卡(命名统一 / OBSOLETE 清理 / 跨产品对账)生效
- 不适用于一次性的 bugfix / feature(那些有具体 spec,没有"模糊匹配"风险)

## 历史教训点(给未来 auditor 看)

| 时间 | 误判 | 真相 |
|---|---|---|
| 2026-05-07 PR-C v1 | 「ClearingBank 整套删除」 | 清算行属于链上组织治理概念;SFID 身份系统不得保留清算行相关入口 |
| 2026-05-07 PR-C v1 | 「SafetyFund / Sweep UI 删除」 | duoqian-transfer extrinsic 真实存在,**保留** |
| 2026-05-07 PR-C v1 | 「ADR-010 时间逆序」 | "A 阶段(2026-05-04)前的派生协议" + ADR 日期 2026-05-06 是合理的 post-fact 文档,无逆序 |
| 2026-05-07 PR-C v1 | 「PR-A 范围里 [9, 0] 投票 call_data」 | **真 bug**(audit 漏检) — sub-pallet 拆分后 InternalVote 在 22.0 而非 9.0,PR-B 顺手修复 |

最后一条说明审计反向:audit 不仅会**误判 false-positive**,也会**漏掉 true-positive**。所以代码级 bug 还要靠 cargo check 跑一轮 + 实地核对 metadata 才能闭环。
