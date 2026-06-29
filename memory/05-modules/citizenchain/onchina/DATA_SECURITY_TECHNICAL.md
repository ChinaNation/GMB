# OnChina 数据与安全技术文档

## 1. 功能需求

本文件集中登记 OnChina 的行政区、CID 号、权限、扫码签名、错误码和高并发数据边界。它承接旧 CID 文档中仍然有效的数据安全规则，并删除独立产品部署和旧路径口径。

## 2. 行政区数据

- 开发真源：`citizenchain/onchina/src/cid/china/china.sqlite`
- 生产读取：`CID_CHINA_DB` 指向随包只读 SQLite
- 省级常量：`citizenchain/runtime/primitives/cid/code.rs`
- 市镇地址段：只能通过开发库变更并重新发布安装包

加载时必须校验：

- SQLite 省表与 runtime primitives 省码一致。
- 省名和市名全国唯一。
- `(province_code, city_code, town_code)` 不重复。
- 删除的市/镇 code 永久进入 tombstones，禁止复用。

## 3. CID 号

CID 号格式为 `R5-K3P1C1-N9-D4`。

- `R5`：省码 2 位 + 市码 3 位。
- `K3`：主体属性 `K1` + 机构类型 `T2`。
- `P1`：盈利属性。
- `C1`：校验位。
- `N9`：9 位稳定散列序列。
- `D4`：年份。

CID 号生成和校验唯一源码目录为 `citizenchain/onchina/src/cid/`。任何端不得维护第二份号码格式、机构码表或省码表。

## 4. 权限范围

OnChina 管理端只承认两类注册局机构登录态：

- `registry_org_code=FEDERAL_REGISTRY`
- `registry_org_code=CITY_REGISTRY`

所有业务列表和 CRUD 必须将登录态转换为 SQL 条件：

- 联邦注册局机构 `admins`：业务数据按 `province_code` 限制。
- 市注册局机构 `admins`：业务数据按 `province_code + city_code` 限制。

禁止先读取全量数据再在 Rust 或前端过滤。

## 5. 扫码签名和验签

扫码协议只有 `QR_V1`。OnChina 的登录、Passkey 更新、管理员集合变更、机构登记链写动作和其它需要冷钱包确认的动作，都必须使用统一 QR 组件。

业务模块只提供：

- 动作码。
- 签名原文或链上 call data。
- 签名摘要。
- 中文展示字段。

统一组件负责：

- 生成二维码。
- 解析二维码。
- 识别签名响应。
- 展示中文确认字段。
- 执行本地验签或提交前校验。

## 6. 错误码

后端统一输出稳定 `error_code`。HTTP 状态只表达传输和登录态语义，业务错误必须通过 `error_code` 区分。

- 登录态无效：`401`
- 权限不足：`403`
- 输入无效、签名失败、challenge 过期、账户不匹配：业务错误码，不得伪装为登录态失效。
- 数据库错误：日志必须展开 SQLSTATE、message、detail 和 hint。

前端只允许在统一 notice 入口翻译错误。业务组件不得直接显示后端英文错误或浏览器原始异常。

## 7. 投票职责边界

OnChina 只签发投票引擎已经定义的资格凭证、人口快照或身份凭证。OnChina 不实现投票流程，不处理计票、状态推进、通过/否决判定，也不得内嵌投票引擎逻辑。

## 8. 验收

```text
python3 citizenchain/onchina/src/cid/china/check_code_immutable.py
sqlite3 citizenchain/onchina/src/cid/china/china.sqlite "PRAGMA integrity_check"
rg "旧独立身份系统名|backend/src|frontend/api|frontend/chain" memory AGENTS.md citizenchain/onchina --glob '!memory/08-tasks/**' --glob '!memory/04-decisions/**' --glob '!**/node_modules/**' --glob '!**/dist/**'
```
