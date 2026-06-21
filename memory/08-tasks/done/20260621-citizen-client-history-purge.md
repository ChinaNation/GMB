# CitizenApp / CitizenWallet 历史命名污染清理

状态:done

任务需求:
删除已归档任务记录中会把 AI 上下文带回旧客户端命名的历史卡和迁移记录,确保当前权威命名只保留 CitizenApp / citizenapp = 公民、CitizenWallet / citizenwallet = 公民钱包。

改动范围:
- `memory/08-tasks/done/`:删除包含旧客户端命名 token 的历史任务卡和产品改名迁移记录。
- `memory/08-tasks/open/`:保留本执行卡,不写旧客户端 token,避免再次污染检索。

边界:
- 不修改 `citizenchain/runtime/`。
- 不修改业务代码。
- 不改当前命名权威源;当前权威源已经是 CitizenApp / citizenapp 与 CitizenWallet / citizenwallet。

验收:
- 归档历史任务卡中旧客户端命名 token 不再命中。
- 活跃规则仍指向 CitizenApp / citizenapp 与 CitizenWallet / citizenwallet。
