# 任务卡:卡⑤ChainReadCache余额/storage共享缓存层

属 ADR-018(memory/04-decisions/ADR-018-wuminapp-unified-query-low-load.md)。

卡⑤ChainReadCache余额/storage共享缓存层

## 验收
- [ ] flutter analyze 0 + flutter test 全过
- [ ] 旧代码/文档/注释清理无残留

## 新窗口独立执行入口(2026-06-13)
- 前置已完工(勿重做):卡①统一提案查询、卡③N+1批量+广场去重、卡④轮询改订阅、卡⑦规则,均已真机验证(详见 ADR-018 与各卡完工记录)。
- 本卡自包含,新聊天窗口直接执行本卡即可,无需上轮对话历史。
- 全局规则见 memory/07-ai/agent-rules.md「死规则:wuminapp 链上查询(ADR-018)」R1/R2/R3。
- 完成标准:flutter analyze 0 + flutter test 全过 + 真机装机验证 + 清理旧代码/注释。
