# OnChina 公民按市办理与姓名拆分

## 目标

- 公民姓名终态拆为 `citizen_family_name` 和 `citizen_given_name`,不保留旧姓名单字段兼容。
- 联邦注册局管理员进入公民 tab 时先选择分管省内城市,进入城市后再查询、新增和操作该市公民。
- 市注册局管理员保持直接进入本市公民列表。
- 新增公民弹窗中居住省市锁定为办理城市,只允许选择居住镇;出生省市镇必填。
- 投票账户输入框右侧使用扫码图标,扫码用户码后填入 SS58 地址。

## 修改范围

- `citizenchain/onchina/src/domains/citizens/`:调整公民 DTO、权限校验、档案哈希、读写字段。
- `citizenchain/onchina/src/core/`:调整 `citizens` 表终态字段和 SQL。
- `citizenchain/onchina/frontend/citizens/`:调整城市入口、新增弹窗、列表展示和 API 类型。
- `memory/`:更新 OnChina 公民字段与办理流程文档。

## 验收

- 联邦注册局管理员必须先进入城市后才能新增公民。
- 市注册局管理员仍直接在本市新增公民。
- 新增公民弹窗无旧姓名单字段残留。
- 投票账户可手输或通过扫码图标填入 SS58 地址。
- 后端拒绝越省、越市创建公民。
- 当前实现无旧姓名单字段残留。

## 状态

- 2026-06-30:已完成。

## 验收记录

- `cargo fmt --all`。
- `cargo check -p onchina`。
- `cargo test -p onchina`，80 个测试通过。
- `npm --prefix citizenchain/onchina/frontend run build`。
- `git diff --check -- citizenchain/onchina memory/01-architecture/onchina memory/05-modules/citizenchain/onchina memory/07-ai/unified-naming.md memory/08-tasks/done/20260630-onchina-citizen-city-name.md`。
- 临时 PostgreSQL + OnChina 本地服务启动成功，`GET /api/v1/health` 返回 `status=UP`。
- 临时库 `citizens` 表确认存在 `citizen_family_name`、`citizen_given_name`、居住省市镇字段，且不存在旧姓名单字段。
- 残留搜索确认旧姓名字段只出现在数据库启动清理逻辑中：删除旧列、校验旧列不存在。
