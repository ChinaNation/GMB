# Stripe Sandbox 会员全链路验收

任务需求：
- 使用独立 Stripe Sandbox，从真实钱包签名、真实 Checkout 测试支付、真实 webhook、staging D1 授权一直验收到换档、支付方式切换、身份冻结、解冻和取消。
- 所有测试从根 `deploy/` 本地部署控制台执行；控制台能力不足时在本任务内完善。

所属模块：
- `citizenapp/cloudflare`
- `citizenweb`
- `deploy`

输入文档：
- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- `memory/03-security/security-rules.md`
- `memory/04-decisions/ADR-034-usdc-prepaid-membership-route.md`
- `memory/07-ai/unified-naming.md`
- `memory/07-ai/unified-protocols.md`
- `memory/07-ai/module-definition-of-done/citizenapp.md`

必须遵守：
- production Stripe、production Worker 和 production D1 不得触碰。
- Stripe 与 Cloudflare Secret 只允许保存在 macOS Keychain / Cloudflare Secret，不得写入仓库、命令行参数或日志。
- USDC 用例必须走 Stripe Crypto 和测试网 USDC，不能用银行卡 Checkout 冒充。
- webhook 必须具备 D1 原子幂等和事件乱序保护，重复事件不得重复授时长。
- voting / candidate 成功用例必须使用真实链上身份和真实钱包签名，不得伪造 D1 身份。
- 每个 PASS 必须同时具备 API/页面、Stripe 对象、真实 webhook、staging D1/权益四侧证据。
- 结束时清理本轮 Stripe 可变资源、staging D1/KV 测试数据和临时运行数据。

输出物：
- Stripe Sandbox 配置与 staging 隔离配置。
- 部署控制台 Stripe Sandbox 全链路测试动作。
- webhook 幂等、乱序保护和 USDC Crypto 约束代码。
- 自动化测试、真实运行报告、中文注释、文档更新和残留清理。

验收标准：
- Worker typecheck、自动化测试、迁移检查和部署控制台语法检查通过。
- staging webhook 可由 Stripe 直接访问，非法签名被拒，合法事件真实落库。
- 卡与 USDC 的付款、订阅/预付授权、换档、双向切换、冻结/解冻、取消均完成真实运行态验证。
- webhook 重复和乱序不会重复授权或回滚新状态。
- 最终报告 `BLOCKED=0`、`FAIL=0`；未具备真实身份或人工支付条件的项目不得计入 PASS。
- 文档已更新、中文注释完整、残留已清理，production 零变更。

当前进度：
- [x] 需求分析与新增文件授权确认。
- [ ] 核对并建立 Stripe Sandbox 与 webhook 配置。
- [ ] 实现 webhook 幂等、乱序保护和 USDC Crypto 约束。
- [ ] 完善部署控制台全链路测试驱动。
- [ ] 完成自动化测试和 staging 部署。
- [ ] 完成真实支付、webhook、权益、切换、冻结和取消验收。
- [ ] 更新文档、完善注释、清理残留并出具报告。
