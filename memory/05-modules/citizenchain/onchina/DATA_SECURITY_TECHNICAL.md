# OnChina 数据与安全技术文档

## 1. 功能需求

本文件集中登记 OnChina 的行政区、CID 号、权限、扫码签名、错误码和高并发数据边界。它承接旧 CID 文档中仍然有效的数据安全规则，并删除独立产品部署和旧路径口径。

## 2. 行政区数据

- 开发真源：`citizenchain/onchina/src/cid/china/china.sqlite`
- 生产读取：`ONCHINA_CHINA_DB` 指向随包只读 SQLite
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

### 6.1 登录错误码

OnChina 管理员登录必须使用登录专用错误码，禁止继续把登录验签错误映射到绑定类 `ONCHINA_BIND_*` 口径。

| 错误码 | 中文提示 |
|---|---|
| `ONCHINA_LOGIN_CAMERA_UNSUPPORTED` | 当前浏览器不支持摄像头扫码，请更换新版浏览器 |
| `ONCHINA_LOGIN_CAMERA_INSECURE_CONTEXT` | 当前页面不是 HTTPS 安全环境，无法使用摄像头 |
| `ONCHINA_LOGIN_CAMERA_PERMISSION_DENIED` | 摄像头权限被拒绝，请在浏览器中允许摄像头权限 |
| `ONCHINA_LOGIN_CAMERA_OPEN_FAILED` | 无法打开摄像头，请检查摄像头权限或设备占用 |
| `ONCHINA_LOGIN_QR_EMPTY` | 请先生成登录二维码 |
| `ONCHINA_LOGIN_QR_PARSE_FAILED` | 签名二维码解析失败，请重新扫码 |
| `ONCHINA_LOGIN_QR_NOT_RESPONSE` | 扫到的不是登录签名响应二维码 |
| `ONCHINA_LOGIN_QR_MISSING_FIELD` | 签名二维码缺少必要字段，请重新扫码 |
| `ONCHINA_LOGIN_QR_BAD_PROTO` | 二维码协议不正确，请使用新版公民钱包扫码 |
| `ONCHINA_LOGIN_QR_BAD_KIND` | 二维码类型不正确，请扫描公民钱包生成的签名响应 |
| `ONCHINA_LOGIN_QR_BAD_PUBKEY` | 签名账户格式无效 |
| `ONCHINA_LOGIN_QR_BAD_SIGNATURE` | 签名格式无效 |
| `ONCHINA_LOGIN_IDENTITY_QR_REQUIRED` | 请先扫描管理员身份二维码 |
| `ONCHINA_LOGIN_ADMIN_ACCOUNT_REQUIRED` | 管理员账户缺失，请重新扫码登录 |
| `ONCHINA_LOGIN_ORIGIN_REQUIRED` | 登录来源缺失，请刷新页面后重试 |
| `ONCHINA_LOGIN_SESSION_REQUIRED` | 登录会话缺失，请刷新页面后重试 |
| `ONCHINA_LOGIN_DOMAIN_REQUIRED` | 登录域名缺失，请使用 `https://onchina.local:8964` 访问 |
| `ONCHINA_LOGIN_ADMIN_NOT_FOUND` | 非管理员禁止登录本系统 |
| `ONCHINA_LOGIN_ADMIN_SCOPE_MISSING` | 管理员省级权限范围缺失，无法登录 |
| `ONCHINA_LOGIN_ADMIN_QUERY_FAILED` | 管理员信息查询失败，请稍后重试 |
| `ONCHINA_LOGIN_SYSTEM_SIGN_FAILED` | 登录二维码签发失败，请检查节点平台配置 |
| `ONCHINA_LOGIN_CHALLENGE_CREATE_FAILED` | 登录请求保存失败，请稍后重试 |
| `ONCHINA_LOGIN_REQUEST_INVALID` | 登录请求内容不完整，请重新扫码 |
| `ONCHINA_LOGIN_RESULT_PARAM_REQUIRED` | 登录轮询参数缺失，请刷新页面后重试 |
| `ONCHINA_LOGIN_CHALLENGE_NOT_FOUND` | 登录二维码不存在或已失效，请重新生成 |
| `ONCHINA_LOGIN_CHALLENGE_CONSUMED` | 登录二维码已使用，请重新生成 |
| `ONCHINA_LOGIN_SESSION_MISMATCH` | 登录会话不匹配，请关闭多余页面后重新生成二维码 |
| `ONCHINA_LOGIN_CHALLENGE_EXPIRED` | 登录二维码已过期，请重新生成 |
| `ONCHINA_LOGIN_SIGNER_MISMATCH` | 签名账户和登录账户不一致 |
| `ONCHINA_LOGIN_CONTEXT_MISMATCH` | 登录上下文不匹配，请重新生成二维码 |
| `ONCHINA_LOGIN_SIGNATURE_VERIFY_FAILED` | 签名验签失败，请重新扫码签名 |
| `ONCHINA_LOGIN_COMPLETE_FAILED` | 登录签名响应处理失败，请查看服务日志 |
| `ONCHINA_LOGIN_RESULT_SAVE_FAILED` | 登录结果保存失败，请稍后重试 |
| `ONCHINA_LOGIN_RESULT_QUERY_FAILED` | 查询登录结果失败，请稍后重试 |
| `ONCHINA_LOGIN_VERIFY_FAILED` | 登录签名校验失败，请重新生成二维码 |
| `ONCHINA_LOGIN_ADMIN_NOT_ONCHAIN` | 当前钱包不是本机构链上有效管理员 |
| `ONCHINA_LOGIN_CHAIN_UNREACHABLE` | 无法连接区块链节点，请确认节点已启动并同步 |
| `ONCHINA_LOGIN_NODE_BINDING_REQUIRED` | 请先确认本节点绑定机构 |
| `ONCHINA_LOGIN_NODE_BINDING_MISSING` | 本节点尚未绑定机构，请重新扫码登录并确认绑定 |
| `ONCHINA_LOGIN_NODE_BINDING_INVALID` | 节点机构绑定状态异常，无法登录 |
| `ONCHINA_LOGIN_NODE_BINDING_QUERY_FAILED` | 节点机构绑定状态查询失败，请稍后重试 |
| `ONCHINA_LOGIN_NODE_BINDING_ALREADY_INACTIVE` | 节点机构绑定已解除，请重新扫码登录 |
| `ONCHINA_LOGIN_NODE_BINDING_CHALLENGE_NOT_FOUND` | 节点机构绑定请求不存在，请重新扫码登录 |
| `ONCHINA_LOGIN_NODE_BINDING_CHALLENGE_CONSUMED` | 节点机构绑定请求已使用，请重新扫码登录 |
| `ONCHINA_LOGIN_NODE_BINDING_CHALLENGE_EXPIRED` | 节点机构绑定请求已过期，请重新扫码登录 |
| `ONCHINA_LOGIN_NODE_BINDING_REQUEST_INVALID` | 节点机构绑定请求不完整，请重新扫码登录 |
| `ONCHINA_LOGIN_NODE_BINDING_CANDIDATE_NOT_FOUND` | 所选机构不在本次登录候选中，请重新扫码登录 |
| `ONCHINA_LOGIN_NODE_BINDING_ADMIN_MISMATCH` | 当前管理员已不属于所选机构，无法绑定本节点 |
| `ONCHINA_LOGIN_PERSIST_FAILED` | 登录会话保存失败，请稍后重试 |

## 7. 投票职责边界

OnChina 只签发投票引擎已经定义的资格凭证、人口快照或身份凭证。OnChina 不实现投票流程，不处理计票、状态推进、通过/否决判定，也不得内嵌投票引擎逻辑。

## 8. 验收

```text
python3 citizenchain/onchina/src/cid/china/check_code_immutable.py
sqlite3 citizenchain/onchina/src/cid/china/china.sqlite "PRAGMA integrity_check"
rg "旧独立身份系统名|backend/src|frontend/api|frontend/chain" memory AGENTS.md citizenchain/onchina --glob '!memory/08-tasks/**' --glob '!memory/04-decisions/**' --glob '!**/node_modules/**' --glob '!**/dist/**'
```
