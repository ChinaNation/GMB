# Node 机构管理员读取与本机激活技术文档

最新更新：2026-07-13。

## 模块定位

Node 桌面端只负责读取 `admin_name + admin_account` 管理员事实、展示岗位任职和激活本机管理员钱包。Node 不提供机构管理员集合直接变更入口；机构岗位或任职变化由 `entity` 独立更新，不得派生或覆盖 `admins`。

个人多签管理员属于 CitizenApp 的独立个人多签业务，不进入本模块。

## 代码边界

```text
citizenchain/node/src/admins/management/
├── mod.rs          # 模块导出与边界
├── account_id.rs   # AccountId 和管理员钱包 hex 规范化
├── activation.rs   # 本机管理员激活签名、校验和本地加密记录
├── codec.rs        # 机构 admins 账户值严格 SCALE 解码
├── commands.rs     # 只读查询、余额查询和激活 Tauri 命令
├── storage.rs      # admins + entity 同一 finalized 快照联合读取
└── types.rs        # 管理员钱包、岗位任职和账户状态 DTO

citizenchain/node/frontend/admins/
├── index.ts
├── types.ts
├── api.ts
├── AdminListPage.tsx
├── InstitutionAssignmentCard.tsx
└── styles.css
```

- `admins/management` 不构造机构管理员变更 call data，不提交机构岗位或任职交易。
- `governance` 和 `transaction/offchain` 只消费统一的管理员钱包与岗位任职 DTO。
- 二维码底座仍归 `frontend/shared/qr/`；本模块只使用它完成本机管理员激活。

## 链上联合读取

机构管理员展示必须在同一个 finalized block hash 上联合读取：

1. 从 `PublicAdmins` 或 `PrivateAdmins::AdminAccounts` 读取机构 CID、机构码、管理员钱包集合和状态。
2. 从对应 `PublicManage` 或 `PrivateManage::InstitutionRoles` 读取有效岗位。
3. 从对应 `InstitutionRoleAssignments` 读取有效任职。
4. 按管理员钱包聚合全部有效岗位任职；同一钱包在同一机构只显示一张卡片。
5. 任一管理员钱包没有有效任职，或有效任职引用了不存在/停用岗位时，返回链上状态不一致错误，不伪造空资料。

非法人机构不能只凭机构码猜测公权或私权归属；按账户同时探测两个 admins 模块，以真实命中的模块确定 entity 路由。同一账户若同时命中两个模块，直接拒绝。

## DTO 契约

`InstitutionAdminInfo`：

| 字段 | 中文说明 |
|---|---|
| `account` | 管理员唯一钱包账户，hex 不含 `0x` |
| `assignments` | 该钱包在本机构的全部有效岗位任职 |

`InstitutionRoleAssignmentInfo`：

| 字段 | 中文说明 |
|---|---|
| `role_code` | 机构内稳定岗位代码 |
| `role_name` | 岗位公开名称 |
| `term_required` | 岗位是否强制任期 |
| `term_start` / `term_end` | 任期起止日，自纪元以来天数 |
| `assignment_source` | 强类型任职来源编码 |
| `assignment_source_label` | 创世、注册局、普选、互选或提名任免 |
| `assignment_source_ref` | 对应登记、选举、投票或任免结果引用 |

机构管理员 DTO 不包含管理员姓名、公民 CID、`creator`、`created_at` 或 `updated_at`。普通公民身份资料不能从机构管理员钱包反推或展示。

## 本机管理员激活

管理员激活只建立本机可操作凭证，不改变任何链上机构、岗位、任职或管理员集合。

```text
GMB(3B) || OP_SIGN_ACTIVATE_ADMIN(0x18)
+ account_id(32)
+ institution_code([u8;4])
+ kind(u8)
+ pubkey(32)
+ timestamp(u64 LE)
+ nonce(16)
```

- 激活前必须在 finalized 链上状态中确认钱包属于该机构当前管理员集合且具有有效岗位任职。
- 验签后写入 `{app_data}/activated-admin-accounts.json`；记录按机构账户、机构码、类型和钱包归档。
- 每次读取已激活管理员时重新与链上当前管理员集合交叉校验，失效任职不再赋予本地操作资格。
- 动态机构必须提供 `accountHex`；只有固定治理机构可用内置 CID 派生账户。

## Tauri 命令

- `get_admin_account_state`：读取机构管理员账户及岗位任职。
- `get_admin_account_balances`：在同一 finalized 块批量读取管理员钱包余额。
- `build_activate_admin_request`：验证链上任职后构建本地激活签名请求。
- `verify_activate_admin`：验证冷钱包签名并保存本地激活凭证。
- `get_activated_admins`：读取仍与链上有效任职一致的本机激活管理员。
- `deactivate_admin`：删除本机激活凭证，不改变链上状态。

不存在 `build_admin_set_change_request`、`submit_admin_set_change` 或机构“换管理员”页面。未来新增任免、选举等业务时，只能调用既有投票引擎并向 entity 提交治理结果，不得恢复直接改管理员集合流程。

## 前端展示

`InstitutionAssignmentCard` 固定按一个管理员钱包展示：

- 顶部：序号和激活/投票操作状态。
- 账户区：管理员钱包 SS58 地址、finalized 余额。
- 任职区：逐条显示岗位名称、岗位代码、任期、来源和来源依据。

卡片不展示管理员姓名或公民 CID。同一钱包有多个岗位时在同一卡片内分条展示；投票资格仍按钱包唯一计算，不能按岗位重复生成同一机构内的票。

## 验收规则

- Rust 必须通过 Node 编译和测试，前端必须通过 TypeScript/Vite 生产构建。
- 旧 `AdminProfile`、机构管理员集合编辑器、差异卡片、直接变更 API/命令和“换管理员”入口必须为零。
- 真实验收使用节点 RPC 与实际前端产物；若用户明确暂缓重新创世，只能基于现有链规格验证 Node Guard、RPC、metadata 和页面，不得把重新创世标记为已完成。
