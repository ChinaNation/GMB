# 任务卡：统一 SFID 管理端权限命名为 LOGIN_STATE / PASSKEY / PASSKEY_CHALLENGE，新增 admins/operation_auth，调整删除机构账户/文档为通行密钥+挑战签名，修改省市管理员为登录态，清理旧命名和残留。

- 任务编号：20260531-092106
- 状态：done
- 所属模块：sfid
- 当前负责人：Codex
- 创建时间：2026-05-31 09:21:06

## 任务需求

统一 SFID 管理端权限命名为 LOGIN_STATE / PASSKEY / PASSKEY_CHALLENGE，新增 admins/operation_auth，调整删除机构账户/文档为通行密钥+挑战签名，修改省市管理员为登录态，清理旧命名和残留。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- sfid/README.md
- sfid/SFID_TECHNICAL.md

## 模块模板

- 模板来源：memory/08-tasks/templates/sfid-backend.md

### 默认改动范围

- `sfid/backend`
- 必要时联动 `sfid/deploy`

### 先沟通条件

- 修改 permit 模型
- 修改账户绑定规则
- 修改数据库结构


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/sfid.md

# SFID 模块执行清单

- 不保存原始实名
- permit、绑定、数据库结构变化前必须先沟通
- 关键接口和数据模型必须补中文注释
- 文档与残留必须一起收口


## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/sfid.md

# SFID 完成标准

- 仍然满足 SFID 不保存原始实名
- 关键接口、数据模型与边界判断已补中文注释
- 文档已同步更新
- permit、绑定、数据库结构变化已先沟通
- 残留已清理


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
- 2026-05-31：新增 `sfid/backend/admins/operation_auth.rs`，统一登记 `LOGIN_STATE / PASSKEY / PASSKEY_CHALLENGE` 权限类型。
- 2026-05-31：后端安全动作输出字段改为 `auth_type`，删除旧二级权限命名；`prepare/commit` 拒绝登录态操作进入安全动作通道。
- 2026-05-31：新增省/市管理员姓名登录态 PATCH 接口；新增/删除管理员保留 Passkey + 冷钱包挑战签名。
- 2026-05-31：删除机构账户、删除机构文档前端改为 Passkey + 冷钱包挑战签名取得 grant 后再调用业务删除接口。
- 2026-05-31：清理前端登录测试残留，更新 SFID 技术文档、前后端目录文档和统一命名登记。

## 完成信息

- 完成时间：2026-05-31 09:33:53
- 完成摘要：完成 SFID 管理端权限统一：新增 operation_auth，统一 auth_type=LOGIN_STATE/PASSKEY/PASSKEY_CHALLENGE，删除机构账户和文档升级为 Passkey+挑战签名，省/市管理员姓名修改改为登录态接口，清理旧命名、前端登录测试残留并更新文档。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
