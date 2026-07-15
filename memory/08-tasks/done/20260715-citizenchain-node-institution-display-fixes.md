# citizenchain 桌面端 机构显示两修复(删岗位码·标题只留全称)

任务需求(用户指令,明确授权改 citizenchain 桌面端前端):
1. 删掉管理员卡里岗位码的显示。
2. 机构详情标题只显示一个全称,去掉简称徽章(简称/类型已在下方「机构类型 /身份ID」卡=orgTypeLabel+cidNumber 呈现)。

安全边界:
- 纯前端展示层(node/frontend React/TS);不动 Tauri 命令、不动链端 runtime、不动数据。
- roleCode 仍作 assignment 列表项 React key(不显示),roleName 保留。

修复清单:
- node/frontend/admins/InstitutionAssignmentCard.tsx:74 删除 `<code>{assignment.roleCode}</code>`(roleName 保留;key 仍用 roleCode)。
- node/frontend/governance/InstitutionDetailPage.tsx:
  - 删除标题短名徽章 `{showShortName && <span className="institution-short-name">{cidShortName}</span>}`(:159-161)。
  - 删除随之无用的 `showShortName` 常量(:146-149)。
  - `institution-short-name` 无 CSS 规则,无残桩。

验收标准:
- `tsc --noEmit` 通过(node/frontend)。
- 管理员卡不再显示岗位码;NRC/PRC/PRB 详情标题均只显全称,简称在「机构类型 /身份ID」卡照旧。

执行记录:
- 2026-07-15:诊断确认岗位码显示于 InstitutionAssignmentCard、双名为详情标题「全称+简称徽章」且 orgType!==0 只对省级生效;用户下达两条修复指令。
- 2026-07-15:落地。删 InstitutionAssignmentCard.tsx 的 `<code>{roleCode}</code>`(roleName/key 保留);删 InstitutionDetailPage.tsx 标题短名徽章 + 无用 showShortName 常量。
- 2026-07-15:node/frontend `tsc --noEmit` 通过(305 文件、零错误)。institution-short-name 无 CSS 无残桩。

结论:两条指令完成并类型校验通过(纯前端展示层)。桌面端 Tauri+连链,浏览器无法跑真实数据,以 tsc 绿 + JSX 删除的确定性为验收;实机效果需重建桌面端查看。
