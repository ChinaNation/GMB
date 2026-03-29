# 任务卡：全面仔细检查一遍 shengbank-stake-interest 模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

- 任务编号：20260328-073618
- 状态：done
- 所属模块：citizenchain/runtime/issuance/shengbank-stake-interest
- 当前负责人：Codex
- 创建时间：2026-03-28 07:36:18

## 任务需求

全面仔细检查一遍 shengbank-stake-interest 模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- <补充该模块对应技术文档路径>

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已核对模块实现、runtime 接线、制度常量、白皮书口径、benchmark/weights、测试覆盖与文档一致性
- 已执行 `cargo test -p shengbank-stake-interest`，结果通过
- 已执行 `cargo test -p shengbank-stake-interest --features runtime-benchmarks`，结果失败，失败原因为 benchmark test suite 引用了私有 `new_test_ext()`

## 审查结论

- 未发现直接的权限绕过、任意账户收款或非 Root 越权补结算漏洞
- 已确认固定收款地址、顺序结算、失败停留当前年度、Root 补结算/强制推进边界这些主路径基本成立
- 发现 1 个高优先级功能偏差：年度结算周期错误地绑定到创世期 30 秒常量，和运行期 6 分钟出块口径不一致
- 发现 1 个中优先级工程问题：`runtime-benchmarks` 变体当前无法通过测试编译，benchmark 维护链路不完整
- 文档与流程存在缺口：任务卡未自动挂载该模块技术文档路径，`load-context.sh` 也无法识别该模块

## 主要发现

1. 年度结算周期使用 `pow_const::BLOCKS_PER_YEAR` 编译期常量，而该常量在当前仓库口径中只是创世期 30 秒出块的占位值，不是运行期真实出块时间。模块却直接在 runtime 配置中把它当成年周期使用，和白皮书“每年 87600 块”口径冲突，运行期若为 6 分钟出块，则实际一轮“年度结算”会被拉长约 12 倍。
2. `runtime-benchmarks` 变体编译失败。`benchmarks.rs` 使用 `impl_benchmark_test_suite!(..., crate::tests::new_test_ext(), ...)`，但 `new_test_ext()` 在测试模块里是私有函数，导致 `cargo test -p shengbank-stake-interest --features runtime-benchmarks` 直接失败。
3. 失败分支测试覆盖不足。现有测试覆盖主要集中在正常发放、Root 权限、参数校验和自动补结算上限，没有覆盖地址解码失败、身份编码失败、主金额转换溢出、年度结算失败以及 dust/零利息分支。
4. 文档/流程完整性不足。任务卡仍保留“<补充该模块对应技术文档路径>”占位，说明模块技术文档虽然存在，但自动上下文装载链路没有登记到位。

## 完成信息

- 完成时间：2026-03-28 07:41:47
- 完成摘要：完成 shengbank-stake-interest 模块审查：确认 1 个高优先级年度周期偏差、1 个 benchmark 编译问题、1 组失败分支测试缺口，以及 1 个上下文装载/文档登记缺口；普通单元测试通过，未发现直接权限绕过或任意收款漏洞。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
