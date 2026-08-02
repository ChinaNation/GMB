# Runtime 升级安全整改

## 任务目标

在下一次 runtime 升级前，逐项完成 F-2、F-1、O-1、O-2、O-3、O-4 整改。每一项必须先输出技术方案并取得用户确认；涉及 `citizenchain/runtime/` 的修改还必须取得二次确认。

完成每一项后必须更新文档、完善中文注释、完善测试并清理全仓残留，再输出下一项技术方案。全部问题完成后，统一更新 runtime 版本并按“WASM CI 成功 → 冻结该成功产物 → 其他软件 CI → 部署”的顺序发布。

## 已确认安全口径

- 全仓严格使用 Substrate / Polkadot SDK 官方类型和术语。
- benchmark 只能准备测试夹具，绝不能改变正式权限或验签结果。
- 候选 runtime 的安全性必须按最终 WASM 产物验证，不能只验证另一份本地编译结果。
- 不保留旧验签旁路、旧协议、兼容字段或双轨逻辑。
- 本任务不自行启用当前被 `CallFilter` 禁用的链下清算。

## 整改清单

### F-2：runtime-benchmarks 验签旁路

- 状态：已完成
- 级别：P0 / 发布前构建闸
- 用户确认：已确认方案、已批准创建任务卡、已完成 runtime 二次确认
- 目标：所有 feature 下均执行真实 sr25519 验签；正式 WASM 构建和候选 `:code` 双重拒绝 benchmark runtime；最终 WASM 对四类非空伪造签名全部拒绝。

### F-1：注册局换绑治理任职人账户

- 状态：已完成；只检查、完善测试、注释和文档，未修改换绑功能逻辑
- 用户确认口径：所有 CID 换绑均不增加治理投票。注册局入口继续按注册局管理员权限、
  岗位权限、实名公民居住辖区和新 `account_id` 签名直接办理，不要求当前账户签名；自主
  入口继续要求当前绑定账户签名。无论通过哪个入口，成功后都只认 CID 链上当前绑定
  `account_id`：新账户接管，旧账户失权。管理员岗位与投票资格归 CID，不因换绑新增一票。
- 本项边界：只锁定候选身份资料不变、辖区限制、新旧账户权限切换和投票防重；不新增
  受保护 CID、换绑审批、投票前置、额外签名、额外 Storage 或第二授权真源。

### O-1：清算 AccountId 与签名验证硬假设

- 状态：已完成
- 处理：offchain pallet 的 `Config` 在编译期限定官方 `AccountId32`；L3 与批次管理员验签
  共用完整32字节到 sr25519 `Public` 的唯一转换，删除两处不安全的泛型编码转换和违规
  旧函数名。未修改 Storage、Extrinsic、SCALE 字段、签名消息、扣费、清算规则或
  `CallFilter`。

### O-2：PaymentIntent 签名域

- 状态：待输出技术方案

### O-3：链下清算 CallFilter

- 状态：待输出技术方案

### O-4：平台订阅调价续费授权

- 状态：待输出技术方案

## F-2 验收条件

- `verify_citizen_signature`、`verify_rebind_signature`、`verify_occupy_signature`、`verify_admin_rebind_signature` 不存在 feature 条件旁路。
- `runtime-benchmarks` 使用临时 keystore 生成的真实签名，禁止固定非空字节夹具。
- 正式 WASM 源码构建同时启用 `runtime-benchmarks` 时必须失败。
- 正式候选 WASM 不包含 Benchmark runtime API。
- 最终候选 WASM 对四类 64 字节非空伪造签名全部拒绝，且不修改身份或绑定状态。
- 真实签名、账户不匹配、跨操作码重放、载荷篡改和错误长度均有测试。
- 重新生成并核验包含真实验签开销的 `citizen-identity` 权重。
- 文档、注释、测试和全仓残留清理完成。

## 执行记录

- 2026-08-02：完成全仓只读核查，确认 F-2 真实存在；冻结创世 WASM 来源未启用 `runtime-benchmarks`，但发布前仍需读取链上实际 `:code` 复核。
- 2026-08-02：用户确认逐项整改流程，批准本任务卡并对 F-2 runtime 修改作出二次确认。
- 2026-08-02：删除四个 feature 条件验签旁路，普通 runtime 与 benchmark runtime 统一
  使用 sr25519、唯一 `signing_message(op_tag)` 和精确账户公钥验证；benchmark 改为临时
  keystore 生成真实签名。
- 2026-08-02：正式 WASM CI 增加显式 feature 构建闸、Benchmark runtime API 检查和
  四签名域最终 WASM 行为探针；探针使用 actor CID 的机构费用账户扣费，不退化为管理员
  账户代付。
- 2026-08-02：以 50 steps × 20 repeats 重跑 `citizen_identity` benchmark 并更新正式
  权重；最终无 benchmark feature 的压缩 WASM 通过 NodeGuard 全部行为探针。
- 2026-08-02：普通/benchmark runtime 验签回归测试通过；`citizen-identity` 78 项测试、
  Node Clippy、目标 pallet Clippy、全仓 Rust 格式检查通过。全 runtime Clippy 另被
  `pow-difficulty/src/benchmarks.rs` 两处既有 `clone_on_copy` 阻断，不属于 F-2 改动。
- 2026-08-02：F-1 原治理投票建议经用户纠正后删除；只读复核确认现有注册局换绑没有
  治理投票前置，投票引擎已有新账户继承岗位票权、旧账户失权、规范账户防双投测试。
  用户二次确认后，仅补候选身份、真实辖区和管理员 CID 当前账户解析回归覆盖。
- 2026-08-02：F-1 定向与全量回归通过：`citizen-identity` 78 项、`public-admins` 12 项、
  `citizenchain` runtime 55 项全部通过；投票引擎换绑防双投定向测试通过。真实 runtime
  用例确认跨省 FRG 岗位换绑被拒、对应省 FRG 无需治理投票直接办理、新账户接管、旧账户
  失权，投票/候选资料仍由原 CID 持有。未新增或修改换绑鉴权、Storage、Event 或流程。
- 2026-08-02：O-1 完成。删除 L3 settlement 与批次管理员验签中的不安全泛型编码转换，
  改为 `AccountId32` 编译期类型等式和唯一完整字节转换。
  新增错误账户的合法64字节签名、签名后篡改收款账户及失败整批回滚测试。`offchain`
  29 项测试、no_std check、全特性 Clippy、`citizenchain` runtime 55 项测试全部通过。
