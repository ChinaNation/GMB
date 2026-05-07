# 任务卡：建立 AI 系统统一协议文件

- 任务编号：20260507-ai-unified-protocols
- 状态：open
- 所属模块：memory / AI 编程系统

## 任务需求

在 AI 系统中先建立“统一协议文件”，以后所有涉及协议、载荷格式、接口契约、字段顺序、签名/验签规则的设计，都必须先登记到该文件，再进入代码、文档或测试修改。

本次只创建统一协议文件，不同时创建统一命名文件和统一必读文件。

## 预计修改目录

- `memory/07-ai/`
  - 中文注释：新增统一协议文件，并把它纳入 AI 上下文装载顺序和开发硬规则；只涉及 AI 系统文档。
- `memory/08-tasks/open/`
  - 中文注释：新增本任务卡，并更新重新创世总审计记录；只涉及任务记录。

## 执行清单

- [x] 新增 `memory/07-ai/unified-protocols.md`
- [x] 明确“扫码协议只有 WUMIN_QR_V1，一个内层交易载荷格式不是新的扫码协议”
- [x] 建立协议登记模板
- [x] 登记当前 P0-2 相关的 `OrganizationManage.propose_create_institution` 载荷格式
- [x] 更新 AI 装载顺序和开发硬规则
- [x] 回写重新创世审计记录

## 完成记录

2026-05-07：

- 已创建 `memory/07-ai/unified-protocols.md`
- 已把它纳入 `agent-rules.md`、`context-loading-order.md`、`document-boundaries.md`
- 已登记 `WUMIN_QR_V1`、`sign_request`、`OrganizationManage.propose_create_institution`、`SFID institution registration-info credential`
- 已回写重新创世审计记录
