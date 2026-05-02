# SHENG_ADMINS_TECHNICAL

> 2026-05-02 更新:代码目录统一使用下划线复数 `sheng_admins`。
> 非链上的省管理员后台业务放 `sfid/backend/sheng_admins/`;与链交互的省管理员能力放
> `sfid/backend/sheng_admins/chain_*.rs`。省管理员 main 公钥、Slot、三槽名册基线放
> `sfid/backend/sheng_admins/province_admins.rs`。

## 0. 区块链端方案对齐（冻结，优先级最高）
1. 本文档第 0 步严格按《SFID-Chain 五项能力对齐技术方案（Runtime 对齐版）》执行。
2. 功能 4 标准流程固定为：`SFID 审批完成 -> 授权 Origin 发链上登记交易 -> 回写 tx_hash/block_number`。
3. 机构识别码 `sfid_id`（内部字段 `site_sfid`）的长度、字符集、大小写规范必须由 SFID 与链侧双端校验。
4. 链上登记成功后，SFID 侧必须保留可验证回执（至少包含 `tx_hash`、`block_number`、回写时间）。
5. 若本文件其余章节与本节冲突，以本节为准。

## 1. 模块目标
- 本模块负责后台治理能力，覆盖两类资源：
1. 省级管理员目录（查询/更换）。
2. 机构管理（身份识别码生成、扫码录入公钥、查询、更新、禁用、撤销、删除）。

- 架构口径（冻结）：
1. 管理员功能与机构管理功能继续保持在 `sheng_admins` 模块内迭代。
2. 不新增独立”管理员模块”或”机构模块”。

- 代码归档：
1. `backend/sheng_admins/catalog.rs`：省级管理员目录治理。
2. `backend/sheng_admins/institutions.rs`：机构身份识别码与机构公钥治理。
3. `backend/sheng_admins/province_admins.rs`：43 省 main 公钥、Slot、ProvinceAdmins。
4. `backend/sheng_admins/signing_seed_store.rs`：省管理员签名 seed 加密持久化。
5. `backend/sheng_admins/chain_*.rs`：省管理员三槽名册和签名公钥的链交互入口。

## 2. 权限口径（当前冻结）
1. `SHENG_ADMIN`（省管理员）：
   - 可管理本省机构和本省市管理员。
   - 必须具备省域作用域（`admin_province` 不能为空）。
   - 省管理员三槽名册、签名公钥激活和轮换通过 `sheng_admins/chain_*.rs` 与链交互。
2. `SHI_ADMIN`（市管理员）：
   - 可读取自己作用域内的省/市管理员和机构信息。
   - 不具备创建、替换、禁用、撤销、删除等治理写权限。
3. 本系统不再存在全局密钥管理员角色；任何接口不得依赖全局管理员绕过省域边界。

## 3. 省域隔离口径
1. `SHENG_ADMIN` 受 `admin_province` 约束，只能管理本省机构、本省市管理员和本省链上三槽名册。
2. `SHI_ADMIN` 继承其上级省管理员的省域，并按市域进一步收窄日常业务视图。
3. 不存在跨省全局治理分支；跨省视图只能由只读统计、公开 pull 或链上查询承接，不能复活全局后台写权限。

## 4. API 矩阵（已实现）

### 4.1 省级省级管理员目录（catalog）
1. `GET /api/v1/admin/sheng-admins`
   - 权限：`SHENG_ADMIN | SHI_ADMIN`
   - 功能：查询省管理员列表，按登录管理员作用域过滤。
   - 对外行 `ShengAdminRow` 字段：`id / province / admin_pubkey / admin_name / built_in / created_at`。
   - **不返回 `status` 字段**：机构永久存在（43 个省份固定），省级管理员只是当前替机构发声的人，被替换即彻底失效，不存在停用 / 状态切换的概念。
2. `PUT /api/v1/admin/sheng-admins/:province`
   - 权限：`SHENG_ADMIN`
   - 功能：过渡期本省自治替换入口；正式三槽名册增删应走 `/api/v1/admin/sheng-admin/roster/*` 链交互端点。
   - 输入校验：`province` 必须在 43 省编码表内（含中枢省）；`admin_pubkey` 必须通过 `sr25519` 公钥格式校验。
   - 数据保持：保留 `created_at`，刷新 `updated_at`；底层 `AdminUser.status` 字段不对外暴露。

### 4.2 机构管理（institutions）
1. `GET /api/v1/admin/cpms-keys`
   - 权限：`SHENG_ADMIN`
   - 范围：仅本省机构。
   - 返回：分页对象（`total/limit/offset/rows`），列表行不包含 `init_qr_payload`。
2. `POST /api/v1/admin/cpms-keys/sfid/generate`
   - 权限：`SHENG_ADMIN`
   - 功能：调用 `sfid` 生成机构身份识别码（`A3=GFR`,`P1=0`），并生成 SFID 签名初始化二维码。
   - 链侧字段对齐：机构识别码对链口径统一为 `sfid_id`，对应本系统内部字段 `site_sfid`。
   - `sfid_id` 规范：长度、字符集、大小写规则由 SFID 与链侧双端校验（同一规则集）。
3. `POST /api/v1/admin/cpms/register`
   - 权限：`SHENG_ADMIN`
   - 功能：扫码录入 CPMS 初始化后产生的机构公钥二维码，生成 proof 字段 `genesis_hash + sfid_id + register_nonce + signature`，并调用链上 `DuoqianManage.register_sfid_institution(sfid_id, register_nonce, signature)`，成功后写审计 `CPMS_KEYS_REGISTER_SCAN`。
   - 主公钥约束：初始化二维码验签与功能 4 proof 签名统一只认当前 SFID `MAIN`；备用公钥不能代替功能 4 出具 proof。
   - 并发控制：同一登记二维码 `replay_token` 在链上提交阶段采用进程内 in-flight 占位，重复并发请求直接拒绝（`register qr is being processed`），避免双重链上提交。
   - 链上等待：`submit_and_watch -> wait_for_finalized` 设置 120 秒超时，防止 HTTP 请求长期挂起。
   - 失败处理：链上提交失败写审计（`CPMS_KEYS_REGISTER_SCAN` + `CHAIN_SUBMIT_FAILED`）并立即持久化，再返回网关错误。
   - 返回：必须包含 proof 字段 `genesis_hash | sfid_id | register_nonce | signature`，以及链上回执字段 `chain_register_tx_hash`、`chain_register_block_number`。
4. `PUT /api/v1/admin/cpms-keys/:site_sfid`
   - 权限：`SHENG_ADMIN`
   - 功能：仅允许 `ACTIVE` 机构更新三把公钥；写审计 `CPMS_KEYS_UPDATE`。
   - 输入校验：三把公钥必须通过 `sr25519` 格式校验，且三把公钥必须互不相同。
   - 审计详情：记录更新前后三把公钥（old/new）。
5. `PUT /api/v1/admin/cpms-keys/:site_sfid/disable`
   - 权限：`SHENG_ADMIN`
   - 功能：机构状态置为 `DISABLED`；写审计 `CPMS_KEYS_STATUS_UPDATE`。
6. `PUT /api/v1/admin/cpms-keys/:site_sfid/enable`
   - 权限：`SHENG_ADMIN`
   - 功能：仅允许 `DISABLED -> ACTIVE`；写审计 `CPMS_KEYS_STATUS_UPDATE`。
7. `PUT /api/v1/admin/cpms-keys/:site_sfid/revoke`
   - 权限：`SHENG_ADMIN`
   - 功能：机构状态置为 `REVOKED`；写审计 `CPMS_KEYS_STATUS_UPDATE`。
8. `DELETE /api/v1/admin/cpms-keys/:site_sfid`
   - 权限：`SHENG_ADMIN`
   - 功能：仅允许删除 `PENDING` 机构记录；写审计 `CPMS_KEYS_DELETE`。

### 4.3 市级管理员（operators）
1. `POST /api/v1/admin/operators`
   - 权限：`SHENG_ADMIN`
   - 输入：`admin_pubkey`（hex）、`admin_name`、`city`（必填）、`created_by`（可选）
   - `city` 校验：必须属于 `created_by` 对应省级管理员的省份且 `code != "000"`（不可为省辖市），通过 `sfid::province::city_code_by_name` 查表
   - `created_by` 解析：
     - `SHENG_ADMIN` 调用：传了必须等于自己 pubkey，否则 403
     - 不传 → 默认为调用者自身
2. `PUT /api/v1/admin/operators/:id`
   - 权限：`SHENG_ADMIN`
   - 输入：`admin_pubkey?`、`admin_name?`、`city?`（均可选）
   - `city` 校验同上，省份从 `operator.created_by` 反查
3. `GET /api/v1/admin/operators` 权限为 `SHENG_ADMIN | SHI_ADMIN`，按 scope 过滤；`DELETE /:id` / `PUT /:id/status` 权限为 `SHENG_ADMIN`。
4. **`OperatorRow` 行**：`id / admin_pubkey / admin_name / role / status / built_in / created_by / created_by_name / created_at / city`
   - 与 `ShengAdminRow` 不同，**`OperatorRow` 保留 `status` 字段**：市级管理员有启用/停用语义

## 5. 机构数据模型
`CpmsSiteKeys` 关键字段：
1. `site_sfid`（对链字段名：`sfid_id`）
2. `pubkey_1 | pubkey_2 | pubkey_3`
3. `status`：`PENDING | ACTIVE | DISABLED | REVOKED`
4. `version`：内部版本号（用于状态/更新追踪）
5. `last_register_issued_at`
6. `init_qr_payload`：仅在 `PENDING` 阶段保留，用于登记校验；登记成功后清空（置 `None`）
7. `admin_province`
8. `created_by | created_at`
9. `updated_by | updated_at`
10. 必须回写字段（链上登记对账）：`chain_register_tx_hash | chain_register_block_number | chain_register_at`

## 6. 关键流程（机构）

### 6.1 生成机构身份识别码
1. `SHENG_ADMIN` 在机构管理页发起”生成身份识别码”。
2. 后端调用 `sfid` 生成 `site_sfid`：
   - 不输入机构公钥。
   - `A3` 固定 `GFR`，`P1` 固定 `0`。
3. 后端生成 `purpose=cpms_init` 的 SFID 签名二维码（`qr_payload`）。
4. 后端写入机构记录为 `PENDING`，并保存 `init_qr_payload`。

### 6.2 CPMS 初始化与扫码录入机构
1. 用户携带 SFID 系统生成的机构二维码去 CPMS 做初始化。
2. CPMS 用该二维码完成初始化并生成自身机构公钥登记二维码（含 3 把公钥 + `init_qr_payload` 绑定信息）。
3. `SHENG_ADMIN` 在 SFID 机构页扫码录入。
4. SFID 校验点：
   - `register` 二维码结构与时间窗口。
   - `checksum` 必须绑定 `init_qr_payload` 哈希，且必须是 `64` 位十六进制字符串。
   - 三把机构公钥必须通过 `sr25519` 格式校验，且三把公钥互不相同。
   - `init_qr_payload` 必须是 SFID 可信签名公钥签发且验签通过。
   - `SHENG_ADMIN` 必须具备省域作用域，且作用域必须等于 `init_qr_payload.province`。
   - `site_sfid` 必须已存在且当前为 `PENDING`。
   - 录入时提交的 `init_qr_payload` 必须与该 `site_sfid` 生成阶段保存值一致。
5. 通过首轮校验后，写入 in-flight 占位（按 `replay_token`）再提交链上机构登记交易（`register_sfid_institution`）；提交 signer 必须与链上当前 `MAIN` 完全一致。
6. 链上提交阶段若失败：清理 in-flight 占位，写审计 `CHAIN_SUBMIT_FAILED` 并持久化，返回 `BAD_GATEWAY`。
7. 链上成功后进入二次提交校验（再次验证 `PENDING` 与 `init_qr_payload` 一致性），通过后机构置为 `ACTIVE`，写入 3 把公钥，清空 `init_qr_payload`，并回写 `chain_register_tx_hash | chain_register_block_number | chain_register_at`。
8. 成功路径完成后：写 `SUCCESS` 审计、持久化运行时状态，并清理 in-flight 占位。

### 6.3 与区块链“机构 SFID 登记（多签前置）”对齐口径
1. 本模块负责生成并治理机构识别码：`site_sfid`（对链口径 `sfid_id`）。
2. 标准流程固定为：`SFID 审批完成 -> 授权 Origin 发链上登记交易 -> 回写 tx_hash/block_number`。
3. 本模块输出并冻结登记入参：`sfid_id`（即 `site_sfid`）及必要机构主数据；链上最终登记状态为唯一真值。
4. 链侧登记成功后，SFID 必须回写 `tx_hash`、`block_number` 与回写时间，供审计与对账。
5. 省管理员三槽名册和签名公钥生命周期由 `sheng_admins/` 与 `sheng_admins/chain_*.rs` 共同维护。

## 7. 前端对接口径（机构页）
1. 列表列名为“身份识别码”，展示 `site_sfid`。
2. 行内支持小二维码预览与下载。
3. 顶部按钮为“生成身份识别码”（不显示“扫码录入机构”顶栏按钮）。
4. 行操作保留“禁用、删除、扫码”；去掉“撤销”按钮。
   - 约束：删除操作仅对 `PENDING` 行可用。
5. 每个公钥列单独提供“更新”按钮。
6. “登记人”显示为”`{省名}省级管理员`”。
7. 不展示“版本”列。

## 8. 前端导航标签页
- 顶层导航顺序：`首页 | 公权机构 | 多签管理 | 密钥管理 | 机构管理`
- 旧"省级管理员 / 市级管理员"独立标签页已合并到"机构管理"主 tab：
  - **`SHENG_ADMIN`** 进入"机构管理"自动跳到自己的机构详情页（无返回按钮）。
  - **`SHI_ADMIN`** 进入"机构管理"自动跳到所属机构的详情页（只读）。
- 机构详情页内置 sub-tab：`市级管理员列表`（默认） / `省级管理员`。
  - "省级管理员" sub-tab 展示省管理员基本信息和三槽名册入口。
  - "市级管理员列表" sub-tab 显示该机构的市级管理员表格（受控分页 10/页），完整 CRUD 仅本省省管理员可写。
- 新增市级管理员通过居中 Modal 弹窗触发，表单包含：姓名 / 市（下拉，过滤省辖市 code=000）/ 账户（SS58）。

## 9. 安全与一致性
1. 本模块写接口统一使用 `require_sheng_admin`；读接口按需使用 `require_admin_any` 后再走 scope 过滤。
2. `SHENG_ADMIN` 写接口必须执行 `admin_province` 非空防御校验。
3. 省域隔离由 `in_scope_cpms_site` 强校验。
4. 机构登记二维码有防重放 token（24 小时窗口）。
5. 只有 `ACTIVE` 机构可用于后续 CPMS 业务二维码验签。
6. 机构登记面与链上多签发起面解耦：本模块只治理 `sfid_id` 主数据，但链上 `origin` 已冻结为“仅当前 MAIN 可提交”。
7. 机构登记状态必须支持“链上已登记”可验证回执，至少包含 `tx_hash/block_number`。
8. 机构登记链上提交阶段使用 in-flight 占位（按 `replay_token`）防止并发双提。
9. 链上 finalized 等待有 120 秒超时保护。
10. 链上签名 seed 在提交链上交易时以 `SensitiveSeed` 形态传递，不转为普通 `String` 暴露。

## 10. 审计事件
1. `SUPER_ADMIN_REPLACE`（更换省级管理员）
2. `OPERATOR_CREATE`（创建市级管理员）
3. `OPERATOR_UPDATE`（更新市级管理员）
4. `OPERATOR_DELETE`（删除市级管理员）
5. `OPERATOR_STATUS_UPDATE`（市级管理员状态变更）
6. `CPMS_SFID_GENERATE`
7. `CPMS_KEYS_REGISTER_SCAN`
   - 结果可为 `SUCCESS` 或 `CHAIN_SUBMIT_FAILED`（同 action，通过 result 区分）。
8. `CPMS_KEYS_UPDATE`
9. `CPMS_KEYS_STATUS_UPDATE`
10. `CPMS_KEYS_DELETE`

## 11. 输入校验规则

| 字段 | 最小 | 最大 | 说明 |
|------|------|------|------|
| province | 1 字符 | MAX_PROVINCE_CHARS | trim 后非空 |
| city | 1 字符 | MAX_CITY_CHARS | trim 后非空 |
| institution | 1 字符 | MAX_INSTITUTION_CHARS | trim 后非空 |
| admin_pubkey | 64 hex | 64 hex | 合法 sr25519 公钥 |
| admin_name | 1 字符 | 256 字符 | trim 后非空 |
| sfid_id | 符合 SFID 格式 | SFID_ID_MAX_BYTES | 5 段式格式校验 |

## 12. 路由挂载与文件索引
1. 路由定义：`backend/main.rs`（`/api/v1/admin/*`）。
2. 模块导出：`backend/sheng_admins/mod.rs`。
3. 业务实现：
   - `backend/sheng_admins/catalog.rs`（省级管理员目录）
   - `backend/sheng_admins/institutions.rs`（机构管理）
   - `backend/sheng_admins/province_admins.rs`（省管理员 main 公钥与三槽模型）
   - `backend/sheng_admins/chain_*.rs`（省管理员链交互）
4. 省域判定：`backend/scope/admin_province.rs`
5. CPMS 状态扫码联动：`backend/citizens/status.rs`


## ADR-008 Phase 23e 更新（2026-05-01）

全局密钥管理员角色已废止；省管理员采用 3-tier 自治（main / backup_1 / backup_2）。
本文档正文已按当前代码口径更新；历史任务卡中的旧角色、旧目录和旧 keyring 名称只作为变更记录保留。
