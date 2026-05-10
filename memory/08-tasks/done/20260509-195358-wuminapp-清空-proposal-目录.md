任务需求：
wuminapp 清空 lib/proposal/ 目录,4 类内容分别归位:
- A: proposal_types_page.dart → lib/governance/governance_proposals_page.dart(同时改 class ProposalTypesPage → GovernanceProposalsPage)
- B 提案抽象层 5 文件 → lib/common/proposal/
- B 跨子模块 detail page(duoqian_manage_detail_page.dart)→ lib/governance/
- C VotingEngine 客户端 4 文件 → lib/votingengine/internal-vote/(含 proposal_vote_widgets.dart 投票UI)
- D 3 个空目录(grandpakey_change/resolution_destroy/resolution_issuance)全删
- 最后 rmdir lib/proposal/shared/ + lib/proposal/

所属模块：
- wuminapp
- 受影响:lib/common/(新增 proposal/ 子目录)、lib/governance/(新增 2 文件)、lib/votingengine/(新建)、lib/proposal/(全删)、外部 21 文件 import 替换、test 同步

输入文档：
- memory/AGENTS.md
- memory/07-ai/chat-protocol.md
- 用户在本对话中确认的方案 + A2 选项(file + class 改名)

必须遵守：
- 0 行为变化:文件搬家 + 路径替换 + 单一 class 改名,禁止动业务逻辑
- 不动链端
- 不动其他 lib/ 模块结构

输出物：
- 代码:
  1. 11 文件搬家
  2. 1 文件改名(A2:proposal_types_page → governance_proposals_page,class 同名变更)
  3. 3 空目录删除 + rmdir lib/proposal/{shared,}
  4. import 路径全工程替换(多个 sed pattern)
  5. ProposalTypesPage class consumer 同步改名(institution_detail_page.dart line 1064)
- 测试:flutter analyze 0 + flutter test 全过
- 文档更新:任务卡迁入 done/、追加 memory 项目记录
- 残留清理:grep `wuminapp_mobile/proposal/` 与 `ProposalTypesPage`/`_ProposalTypesPageState` 全为 0

验收标准：
- flutter analyze 0 issue
- flutter test 全过(允许 personal_pending_create_lookup_test Isar flake)
- 残留 grep 0
- lib/proposal/ 目录已删除
- lib/votingengine/internal-vote/ 4 文件就位
- lib/common/proposal/ 5 文件就位
- lib/governance/ 多 2 文件(governance_proposals_page.dart + duoqian_manage_detail_page.dart)

不做：
- UI 标题"发起提案"改"治理提案"(下一轮做,与加 3 张卡片整改一起)
- 转账按钮位置整改(下一轮)
- transaction/issuance/citizen-vote 等其他模块对齐(下一轮)
