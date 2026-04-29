# 任务卡：全仓库检查机构相关账户计算是否正确，优先核查 runtime/primitives 作为真源定义，并追踪其在 runtime 及相关模块中的使用

- 任务编号：20260421-092827
- 状态：open
- 所属模块：citizenchain/runtime/primitives
- 当前负责人：Codex
- 创建时间：2026-04-21 09:28:27

## 任务需求

全仓库检查机构相关账户计算是否正确，优先核查 runtime/primitives 作为真源定义，并追踪其在 runtime 及相关模块中的使用

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
- 已核查 `citizenchain/runtime/primitives` 的账户真源定义：
  - `core_const.rs` 明确机构主账户/费用账户/命名账户的派生命名空间分别对应 `OP_MAIN` / `OP_FEE` / `OP_INSTITUTION`
  - `count_const.rs` 的机构数量与联合投票总票数计算通过单元测试校验
  - `genesis.rs`、`china_ch.rs` 中的人口、创世发行、质押本金汇总通过单元测试校验
- 已追踪 runtime 消费路径：
  - `duoqian-manage` 是机构地址派生的唯一实现入口
  - `offchain-transaction` 通过 `(sfid_id, "费用账户")` 反查费用账户
  - `duoqian-transfer` 直接使用机构 `main_address` / `fee_address` / `NRC_ANQUAN_ADDRESS`
  - `runtime/src/configs/mod.rs` 负责机构账户保护与保留地址判定
- 已执行验证：
  - `cargo test -p primitives --lib` 通过（7/7）
  - `cargo test -p duoqian-manage --lib` 通过（15/15）
  - `cargo test -p shengbank-interest --lib` 通过（18/18）
  - `cargo test -p offchain-transaction --lib` 通过（20/20）
  - `cargo test -p duoqian-transfer --lib` 通过（22/22）
- 额外静态校验：
  - `china_zb.rs` 的 408 个制度保留地址当前有序、无重复，可满足 `binary_search` 前提
  - 408 个保留地址与各机构族地址总量吻合：
    - 政府 `54`
    - 监察 `47`
    - 立法 `44`
    - 司法 `44`
    - 教育 `1`
    - 储委会主账户 `44`
    - 储委会费用账户 `44`
    - 省储行主账户 `43`
    - 省储行费用账户 `43`
    - 省储行质押账户 `43`
    - 国储会安全基金 `1`
- 当前审计结论：
  - 尚未发现 `runtime/primitives` 到已核查 runtime 消费方之间存在已证实的机构账户计算错误
  - 仍存在测试覆盖盲区：尚未看到专门验证 `"主账户"` / `"费用账户"` 保留名派生到固定地址的单元测试
  - 仍存在可维护性风险：`is_reserved_main_address` / `is_reserved_main_account` 命名偏窄，实际覆盖的不仅是主账户，还包括 fee/stake/安全基金等制度保留地址，后续维护容易误判
- 已补测试盲区：
  - 在 `duoqian-manage` 新增保留角色名测试，验证 `"主账户"` / `"费用账户"` 注册时会强制派生到固定 `OP_MAIN` / `OP_FEE` 地址，且不能伪装成 `Named(...)` 落到 `OP_INSTITUTION`
  - 在 `china_zb.rs` 新增制度保留地址表测试，验证 408 个地址严格递增且无重复，持续满足 `binary_search` 前提
- 补测结果：
  - `cargo test -p primitives --lib` 通过（8/8）
  - `cargo test -p duoqian-manage --lib` 通过（17/17）
- 已继续核查 `sfid` / `wuminapp` 跨模块口径：
  - `sfid/backend` 不再本地重算机构地址，创建/激活时只把 `account_name` 原样交给链端，注册成功后再从 `DuoqianManage::SfidRegisteredAddress(sfid_id, account_name)` 回读 `duoqian_address`
  - `sfid/frontend` 只管理 `account_name`、状态和展示，不持有第二套机构地址派生公式
  - `wuminapp` 创建机构多签提案时，优先使用 SFID 返回的 `duoqian_address`，否则回退到链上查询 `SfidRegisteredAddress(sfid_id, account_name)`，也没有单独重算主账户/费用账户地址
- 新发现并已修复的跨模块问题：
  - `sfid` 公开接口返回的 `chain_status` 为 `INACTIVE/PENDING/REGISTERED/FAILED`
  - `wuminapp` 旧代码仍按 `Pending/Confirmed/Failed` 解释，导致已注册账户不会命中“直接信任后端返回地址”分支，而会多做一次链上查询
  - 该问题不会改变机构账户计算结果，但会造成状态口径不一致和一次不必要的二次查询
- 本次修复：
  - 在 `wuminapp/lib/wallet/capabilities/api_client.dart` 增加 `chain_status` 归一化逻辑，兼容旧值并统一折叠到 `INACTIVE/PENDING/REGISTERED/FAILED`
  - 在 `wuminapp/lib/governance/duoqian_create_proposal_page.dart` 改为使用归一化后的 `isRegistered` 判定
- 已继续修复 `citizenchain/node` 治理页机构账户显示链路：
  - 删除 `node/src/governance/mod.rs` 中过期的主账户静态地址表，治理首页和详情页统一改为直接读取 `runtime/primitives/china/china_cb.rs`、`china_ch.rs`、`NRC_ANQUAN_ADDRESS`
  - 新增 `node/src/governance/registry.rs`，把 `国储会 / 省储会 / 省储行` 的 `主账户 / 费用账户 / 安全基金账户 / 永久质押账户` 地址统一收口到 runtime 常量真源
  - `get_institution_detail()` 改为先取同一个 finalized block hash，再按该块高查询当前页面全部账户余额，避免一页内混入不同块高金额
  - 新增 `node/src/governance/balance_watch.rs`，详情页打开后会持续监听 finalized 新块，并通过 Tauri 事件 `governance-balance-updated` 只刷新链上金额与告警
  - 前端 `InstitutionDetailPage.tsx` 仅补监听和 state 覆盖逻辑，页面 UI、卡片布局、显示顺序保持不变
- 本轮验证：
  - `npm run build`（`citizenchain/node/frontend`）通过
  - `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo check -p node --manifest-path /Users/rhett/GMB/citizenchain/Cargo.toml` 通过
  - 验证后已删除临时生成的 `citizenchain/node/frontend/dist`，避免构建残留留在仓库
