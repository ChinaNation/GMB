# 任务卡：全面仔细检查一遍 pow-difficulty 模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

- 任务编号：20260328-100300
- 状态：done
- 所属模块：citizenchain/runtime/otherpallet/pow-difficulty
- 当前负责人：Codex
- 创建时间：2026-03-28 10:03:00

## 任务需求

全面仔细检查一遍 pow-difficulty 模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

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
- 已核对模块实现、runtime 接线、节点侧消费、白皮书口径、benchmark/weights、测试覆盖与文档一致性
- 已执行 `cargo test -p pow-difficulty`，结果通过
- 已执行 `cargo test -p pow-difficulty --features runtime-benchmarks`，结果通过

## 审查结论

- 未发现外部账户可直接改写难度、窗口状态或绕过 Runtime API 的权限漏洞
- 已确认动态难度主公式、首窗口时序、上下限夹紧、零难度自修复、Runtime API 接线与节点侧消费主路径基本成立
- 发现 1 个高优先级问题：模块在 `on_finalize` 中用 `assert!` 执行“空块不允许上链”，把运营/挖矿策略升级成了 runtime panic 风险
- 发现 2 个中低优先级问题：技术文档存在多处陈旧口径；上下文装载/任务卡自动挂载仍缺少该模块登记
- 发现 1 个低优先级工程风险：当前 weight 描述没有显式反映调整路径中的 `ExtrinsicCount` 与 `TargetBlockTimeMs` 读取

## 主要发现

1. `on_finalize` 在所有非创世块都对 `frame_system::extrinsic_count()` 执行 `assert!(> 1)`。节点侧虽然已经有“交易池为空就不挖矿”的门控，但 runtime 里再用 panic 型断言兜底，会把一个出块策略问题升级成状态机拒块/停链风险，也和模块文档“异常存储值不能 panic 停链”的鲁棒性口径不一致。
2. 模块技术文档已不完全准确：
   - 文档仍写 `MILLISECS_PER_BLOCK` 当前是 6 分钟，但 `pow_const.rs` 中该常量实际是创世期 30 秒占位值，运行期真实目标时间来自 `genesis-pallet`
   - 文档仍写 `weights.rs` 是“保守手写值”，但仓库中的 `weights.rs` 已是 benchmark CLI 自动生成产物
   - 文档写当前测试覆盖 8 项，但实际单测输出为 9 项，已经包含 `rejects_empty_block`
3. 任务卡仍保留“<补充该模块对应技术文档路径>”占位，`load-context.sh` 也无法识别该模块，说明该模块虽然已有技术文档，但没有接入自动上下文装载链路。
4. 当前 adjustment 路径的代码会读取 `frame_system::extrinsic_count()` 与 `genesis_pallet::target_block_time_ms()`，但 `weights.rs` 的存储说明没有显式体现这两项。这个问题我没有定性为已证实的现网 underweight，但它值得在后续重新 benchmark 时重点核对。

## 完成信息

- 完成时间：2026-03-28 10:07:36
- 完成摘要：完成 pow-difficulty 模块审查：确认 1 个高优先级 runtime 断言风险、2 个文档/流程口径问题和 1 个低优先级 weight 核对风险；普通单测与 runtime-benchmarks 变体均通过，未发现外部账户可直接改写难度或窗口状态的权限漏洞。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
