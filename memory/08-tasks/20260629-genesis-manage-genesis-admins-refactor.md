# 20260629 创世机构与固定治理管理员中间方案

## 当前状态

- 状态：已被 2026-06-30 合并方案取代。
- 当前唯一口径：创世机构与初始管理员由 `citizenchain/runtime/genesis/src/institution.rs` 在创世时一次性写入。
- 运行期机构生命周期归 `public-manage` / `private-manage` / `personal-manage`。
- 运行期管理员治理归 `public-admins` / `private-admins` / `personal-admins`；固定治理机构也归 `public-admins`。

## 历史说明

- 本卡记录的是 2026-06-29 的过渡设计。该设计后来被用户否决，不再作为实现或文档依据。
- FRG 仍采用 43 个省级 5 人组，阈值固定为 3；链上 storage 和管理员更换入口均已归入 `PublicAdmins`。
- 后续验收以 2026-06-30 的注册局权力线与创世写入合并任务为准。
