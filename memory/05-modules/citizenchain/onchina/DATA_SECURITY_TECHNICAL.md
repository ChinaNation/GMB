# OnChina 数据与安全技术文档

## 1. 功能需求

本文件集中登记 OnChina 的行政区、CID 号、权限、扫码签名、错误码和高并发数据边界。它承接旧 CID 文档中仍然有效的数据安全规则，并删除独立产品部署和旧路径口径。

## 2. 行政区数据

- 开发真源：`citizenchain/onchina/src/cid/china/china.sqlite`
- 生产读取：`ONCHINA_CHINA_DB` 指向随包只读 SQLite
- 省级常量：`citizenchain/runtime/primitives/cid/code.rs`
- 镇下完整地址：`addresses` 单表保存当前有效地址；开发库随安装包发布，链上 `AddressRegistry` 记录单条地址变更事实和当前哈希

加载时必须校验：

- SQLite 省表与 runtime primitives 省码一致。
- 省名和市名全国唯一。
- `(province_code, city_code, town_code)` 不重复。
- 镇下地址使用 `address_name_code(3位) + address_local_no(4位)` 模型。
- 旧地址结构、墓碑表和变更日志表必须清除。
- 地址库只保存当前有效数据,不保留旧地址历史。
- 地址链上变更只同步对应的地址名称或完整地址，不全量上链地址库。

## 3. CID 号

CID 号格式为 `R5-K3P1C1-N9-D4`。

- `R5`：省码 2 位 + 市码 3 位。
- `K3`：主体属性 `K1` + 机构类型 `T2`。
- `P1`：盈利属性。
- `C1`：校验位。
- `N9`：9 位稳定散列序列。
- `D4`：年份。

CID 号生成和校验唯一源码目录为 `citizenchain/onchina/src/cid/`。任何端不得维护第二份号码格式、机构码表或省码表。

### 3.1 公民 CID 和护照号

- 公民 CID 的机构代码固定为 `CTZN`;个人码不携带办理市码,R5 市段固定为 `000`。
- 公民护照号由 `citizenchain/onchina/src/domains/citizens/passport_no.rs` 生成,格式为省码 2 位 + Crockford Base32 主体 8 位 + 校验位 1 位。
- 护照号终身唯一;`passport_numbers` 负责全局查重。
- 护照号资源回收只允许通过 `passport_number_recycle_pool` 回收号码本身,不得保存旧公民姓名、出生地、钱包、公民 CID 或其它个人资料。
- 公民档案本地创建阶段允许没有 `wallet_address` 和内部 `wallet_pubkey`;儿童或暂未开户公民不得被强制生成钱包。
- `wallet_address / wallet_pubkey / wallet_sig_alg` 只在链上公民身份推送准备阶段写入,且必须先验证目标公民钱包对 `VotingIdentityPayload` 的签名;前端、审计展示和普通 DTO 只展示 SS58 地址。
- 公民选举/被选举范围由出生地、居住地和投票规则共同决定,不在公民档案中保存独立范围字段。

## 4. 权限范围

OnChina 管理端只承认当前节点 active binding 绑定机构的链上 active admin 登录。登录态必须携带 `institution_code`、`admin_level`、`scope_province_name`、`scope_city_name`、`scope_town_name` 和后端下发的 `capabilities`。

所有业务列表和 CRUD 必须将登录态转换为后端 scope 条件：

- FRG：按本节点绑定的省级组限制。
- CREG：按本节点绑定的市级范围限制。
- 省 / 市 / 镇级机构：按 `admin_level` 派生的省 / 市 / 镇范围限制。
- 私权和非法人机构：按本机构链上身份限制。
- NJD、普通公权、私权和非法人组织本期只开放“本机构管理员”只读能力，不开放机构登记、账户、资料或地址库写能力。
- NRC / PRC / PRB 使用节点桌面端，不进入 OnChina 网页控制台。
- PMUL 和其它个人主体不进入 OnChina 网页控制台。

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
| `ONCHINA_TLS_CA_UNAVAILABLE` | 机构 CA 证书暂不可用，请确认链上中国平台已正常启动 |
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
| `ONCHINA_LOGIN_DESKTOP_GOVERNANCE_UNSUPPORTED` | 国家储委会、省储委会、省储行使用节点桌面端管理，不支持登录链上中国平台 |
| `ONCHINA_LOGIN_PERSONAL_MULTISIG_UNSUPPORTED` | 个人多签账户不支持登录链上中国平台 |
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
