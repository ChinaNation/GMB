# OnChina 公民新增字段与护照号迁移

## 目标

- 公民由注册局管理员一次性录入,不再手填身份 CID。
- 公民身份 CID 自动生成,机构代码固定为 `CTZN`。
- 护照号生成逻辑从归档 CPMS 复制到 OnChina,不引用归档目录。
- 新增公民时钱包账户必填;前端只展示 SS58 地址,系统内部保存公钥哈希材料。
- 出生省市镇必填且创建后不可改;居住省市来自当前办理注册局上下文。
- 护照有效期按出生日期自动计算:年满 16 周岁 10 年,未满 16 周岁 5 年。
- 删除新流程里的 `bind_status`、`election_scope_level`、`bound_at` 等旧绑定/选举范围字段。

## 修改范围

- `citizenchain/onchina/src/domains/citizens/`:公民字段、护照号、创建接口和查询 DTO。
- `citizenchain/onchina/src/core/db.rs`:PG 表结构初始化和旧字段清理。
- `citizenchain/onchina/src/cid/china/`:出生地/居住地镇列表只读接口。
- `citizenchain/onchina/frontend/citizens/`:新增公民弹窗、列表和详情展示。
- `citizenchain/onchina/frontend/china/`:行政区 API 类型补充。
- `citizenapp/lib/my/myid/`:电子护照状态接口消费、页面展示和扫码签名文案同步。
- `memory/05-modules/citizenchain/onchina/`:后端、前端和数据安全文档更新。
- `memory/01-architecture/citizenapp/`、`memory/01-architecture/onchina/`:产品架构文档更新。

## 验收

- `cargo check -p onchina`
- `npm run build`
- `flutter analyze`
- `flutter test test/myid_page_test.dart`
- CitizenApp 电子护照页不展示内部 `wallet_pubkey`,只展示 SS58 钱包地址。
- OnChina 本地服务以真实 PostgreSQL 启动并通过 `/api/v1/health`。
- 搜索确认前端不再展示 `wallet_pubkey`,新公民流程不再出现旧绑定态和旧选举范围字段。

## 状态

- 2026-06-30:已完成。

## 验收记录

- `cargo check --manifest-path citizenchain/Cargo.toml -p onchina`
- `npm --prefix citizenchain/onchina/frontend run build`
- `cargo build --manifest-path citizenchain/Cargo.toml -p onchina`
- 临时 PostgreSQL + OnChina 本地服务启动成功,`GET /api/v1/health` 返回 `status=UP`。
- `flutter analyze`（`citizenapp`）
- `flutter test test/myid_page_test.dart`（`citizenapp`）
