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
- [x] 核对并建立独立 Stripe Sandbox、四档 test Price 与 staging webhook 配置。
- [x] 实现 webhook 事件/付款幂等、跨路线乱序保护和 USDC Crypto 约束。
- [x] 完善部署控制台全链路测试驱动及刷新/标签交接后的日志恢复。
- [x] 完成 21 个测试文件、167 项 Worker 测试和 staging 部署；Worker 版本 `1d516002-e731-4b07-bdad-0c174ee29473`。
- [x] 完成卡付款、webhook 授权、真签名取消、真实 Sepolia USDC 付款、重复 event、卡→USDC、USDC→卡和切换后取消验收；控制台结果 `PASS=8 / BLOCKED=0 / FAIL=0`。
- [ ] 使用真实 voting/candidate 链上身份钱包完成两档成功订阅，以及身份不匹配冻结/解冻验收；当前访客钱包不具备条件，未计入 PASS。
- [x] 更新会员协议、ADR、统一命名和架构文档；本轮测试数据清理为 `challenge=0 / membership=0 / payment=0`，production 零变更。

2026-07-14 真实验收记录：
- Sandbox：`acct_1Trr2qQlQZ1x0Cw8`，`livemode=false`；真实签名钱包 `w5CKZGRpzB8ztn7MEn6uKVYiJbdDQD8me5UzBAjg779mynSgP`。
- 卡订阅 Checkout `cs_test_a1ba7BD0pMT6PPnHp025FYz9tLSOiSbc5gX3bKAz0SUudA6b1BhuyoGJxb`：付款、subscription webhook、D1 freedom 授权和真签名到期取消均通过。
- Crypto Checkout `cs_test_a1PBQanUBmGumuSoe3fZPky06B6goptQmu1occNCfcrO8R7GJuBiFw4NBm`：账户 `0xc8d89abb0b769C6d34e506D9ea41be85fd19E666` 真实支付 8.97 Sepolia USDC，链上余额由 20.00 降为 11.03；checkout webhook 授权通过，真实 event 重放两次未延长时长。
- USDC→卡 Checkout `cs_test_a1cefCoquyvTi2jSzqXoMahrwQxy8nEpkvl1rvL880ssTyV1rfSHIlwVUp`：Stripe 展示 122 天免费并从 2026-11-14 开始月扣，trial 与 D1 卡订阅均落地，随后真签名取消通过。
- 首轮真实支付暴露卡→USDC webhook 乱序：旧卡 `customer.subscription.updated` 在 Crypto Checkout 后到达并覆盖预付行。修复后普通旧卡事件返回 `subscription_superseded`，只有服务端 `payment_switch=usdc_to_stripe` 可完成反向切换；本轮整车复测未再复现覆盖。
- 旧报告中的 Worker→Stripe 525 在独立 Sandbox 路径已不再复现，不能继续作为当前代码或 Cloudflare 出口故障结论；本轮真实建单、取消、Crypto Checkout 和 webhook 全部通过。
