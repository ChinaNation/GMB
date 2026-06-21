任务需求：
- CPMS 公民档案区分签发机构归属、居住地、出生地和选举地域精度。
- CPMS 公民详情页把现有地址展示改为居住语义，并在出生日期下方、护照号上方展示出生地。
- 新增公民时从 CID 行政区唯一真源的 CPMS 随包只读拷贝中选择全国省、市、镇作为出生地，保存后不可更改。
- “设置投票账户”弹窗新增“注册市选举公民”和“注册镇选举公民”两个开关；默认关闭，开启镇必须同时开启市。
- 公民详情页投票账户下方只读展示市/镇选举公民注册结果，不允许在详情页直接勾选。
- CPMS 档案码按选举地域精度携带居住地和出生地的省、市、镇代码；CID 录入档案码后按精度落库，为后续投票区域权限判断准备数据。
- CID 公民详情不展示居住地代码、出生地代码；投票范围按居住地展示，参选范围按出生地展示。
- CID 录入档案码和导入年度报告时，只保留正常、有投票资格且已设置投票钱包的公民；无资格、注销或释放绑定的公民记录必须从 CID 删除。

所属模块：
- CPMS
- CID
- CID_CPMS_V1 协议文档

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/workflow.md
- memory/07-ai/context-loading-order.md
- memory/07-ai/document-boundaries.md
- memory/07-ai/definition-of-done.md
- memory/07-ai/pre-submit-checklist.md
- memory/05-modules/citizenpassport/backend/dangan/DANGAN_TECHNICAL.md
- memory/05-modules/citizencode/CID-CPMS-QR-v1.md
- memory/05-modules/citizencode/backend/citizens/CITIZENS_TECHNICAL.md
- memory/05-modules/citizencode/backend/citizenpassport/CITIZENPASSPORT_TECHNICAL.md
- memory/07-ai/module-checklists/citizenpassport.md
- memory/07-ai/module-checklists/citizencode.md
- memory/07-ai/module-definition-of-done/citizenpassport.md
- memory/07-ai/module-definition-of-done/citizencode.md

必须遵守：
- CPMS 永不联网；出生地选择数据只读来自 CPMS 随包内嵌的 CID 行政区唯一真源拷贝。
- ARCHIVE 档案码不得明文暴露中文省市镇名称；只携带行政区代码。
- CID 不保存原始实名数据，不接管 CPMS 实名信任根。
- 不突破模块边界，不在 CPMS/CID 业务模块内实现投票流程。
- 修改代码后必须更新文档、完善中文注释、清理残留并执行真实运行态验收。

输出物：
- CPMS 后端和前端代码
- CID 后端和前端代码
- 中文注释
- 测试或真实运行态验收记录
- 文档更新
- 残留清理

验收标准：
- CPMS 新增公民可选择全国任意出生省、市、镇。
- CPMS 公民详情按居住语义展示地址，并展示出生地。
- 出生地保存后不可通过编辑接口修改。
- 投票地域开关满足默认省级、可开市、开镇必开市的规则。
- CPMS 档案码只携带代码，并按选举地域精度裁剪市镇。
- CID 扫描档案码后按精度落库居住地和出生地代码。
- CID 公民详情按行政区真源名称展示投票范围和参选范围，不展示行政区代码。
- CID 档案码录入必须拒绝 `citizen_status != NORMAL`、`voting_eligible=false` 或未携带投票钱包的公民。
- CID 导入 CPMS 年度报告时，对注销、无投票资格或释放绑定的匹配公民执行删除，保证 CID 公民库只保留真正可投票公民。
- 现有公民管理省市查询不被破坏。
- 文档已同步，残留旧文案已清理。

执行记录：
- 已新增 CPMS 全国出生地只读接口，数据只来自 CPMS 随包的 CID `china.sqlite` 行政区唯一真源。
- 已把 CPMS 新增公民页区分为居住地和出生地；出生地只在创建时提交，编辑接口不接收出生地字段。
- 已把 CPMS 详情页地址标签改为居住语义，并在出生日期下方、护照号上方显示出生地。
- 已将市/镇选举公民注册开关放入“设置投票账户”弹窗；镇级开启会强制市级开启，关闭市级会清掉镇级。
- 已将公民详情页投票账户下方改为只读展示市/镇选举公民注册结果。
- 已把 ARCHIVE 明文字段外的地域信息放入加密 `geo_seal`，只携带行政区代码，不携带中文地名。
- 已在 CID 扫码验档案码后校验选举地域精度，并把居住地/出生地代码落入公民库。
- 已将 CID 公民详情从代码展示改为 `投票范围 / 参选范围` 展示：投票范围按居住地，参选范围按出生地，名称来自 CID 行政区唯一真源。
- 已在 CID 档案码绑定 challenge 入口增加硬门槛：仅 `NORMAL + voting_eligible=true + 钱包字段齐全` 可进入绑定流程。
- 已将 CID 年度报告导入改为只保留正常且有投票资格的公民；无资格记录和释放记录会删除 CID 本地公民绑定。
- 已增加 CID 启动期历史残留清理，删除注销、无投票资格、未绑定钱包或非完整绑定的旧公民行。
- 已同步 CPMS/CID 技术文档和 CID-CPMS-QR-v1 协议文档。

验收记录：
- `cargo fmt`：CPMS 后端、CID 后端通过。
- `cargo check -q`：CPMS 后端、CID 后端通过。
- `npm run build`：CPMS 前端、CID 前端通过；CID 前端仅保留 Vite 既有 chunk-size 警告。
- `git diff --check`：通过。
- 本轮新增 CID 年度报告判定测试：`cargo test -q citizens::status_export_import` 通过，5 个测试通过。
- 本轮重新执行 CID 后端 `cargo check -q` 通过，CID 前端 `npm run build` 通过，仅保留 Vite 既有 chunk-size 警告，`git diff --check` 通过。
- 本轮临时启动当前 CID 后端 `127.0.0.1:18899` 成功，`/api/v1/health` 返回 `UP`；启动初始化完成且无无资格公民清理 SQL 错误。
- 本轮真实数据库校验：`citizens` 当前 1 条，不合格残留查询结果为 0。
- 本轮临时启动当前 CID 前端预览 `http://localhost:5189/`，浏览器打开生产构建页面成功，标题为 `CID 管理端`，登录页可渲染，控制台无 error。
- CPMS 临时后端 `127.0.0.1:18080` 启动成功，`/api/v1/health` 返回 `ok`。
- CID 临时后端 `127.0.0.1:18899` 启动成功，`/api/v1/health` 返回 `UP`，并连接本地链节点。
- CPMS 真实数据库 `_sqlx_migrations` 已到版本 `2`，`archives` 已存在出生省/市/镇和选举范围字段。
- CID 真实数据库 `citizens` 已存在居住地、出生地和选举范围字段，并存在 `idx_citizens_residence_scope`、`idx_citizens_birth_scope` 索引。
- CPMS 全国行政区接口用临时登录态真实调用成功：广东省代码 `GD`、广州市代码 `001`、镇级样本 `001:沙面镇`。
- CPMS 当前静态前端和当前后端组成的临时实例 `127.0.0.1:18081` 浏览器验收通过：新增页显示居住省份/居住城市/居住地址和出生省份/出生城市/出生镇；详情页显示出生地和市/镇选举注册结果，设置投票账户弹窗显示两个选举注册开关。
- CPMS 当前静态前端和当前后端组成的临时实例 `127.0.0.1:18081` 资格门禁验收通过：无选举资格档案保存投票账户接口返回 `409 archive voting ineligible`；页面中投票账户地址框和扫码按钮均不可操作，恢复为有选举资格后扫码按钮可打开“设置投票账户”弹窗。
- CPMS 当前静态前端和当前后端组成的临时实例 `127.0.0.1:18081` 出生日期不可变验收通过：直接调用编辑接口提交 `birth_date` 返回 `400 birth data cannot be updated`，数据库出生日期保持 `1990-05-11`；详情编辑态出生日期控件 `disabled=true/readOnly=true`，可编辑出生日期输入数量为 0。
- 验收后已删除临时 session，并停止临时后端与临时 cookie 跳转服务。

纠偏记录：
- 已移除公民详情页投票账户下方的市/镇选举注册勾选入口，详情页只读展示“已注册 / 未注册”结果。
- 已将“注册市选举公民 / 注册镇选举公民”开关移动到“设置投票账户”弹窗内；扫码只填入投票账户，不再立即保存。
- 已将 CPMS 投票账户接口调整为一次保存 `wallet_address + election_scope_level`，普通编辑档案接口不再更新 `election_scope_level`。
- 已重新执行 CPMS 前端 build、CPMS 后端 `cargo check -q`，并用当前静态前端 + 当前后端临时实例完成浏览器验收：详情页无 checkbox，弹窗内有 2 个 checkbox，开镇自动开市，关市自动关镇。
- 已调整“设置投票账户”弹窗样式：标题居中、弹窗加宽、投票账户输入框保持单行正常高度并铺满宽度、市/镇选举注册开关左右并排、底部取消和保存按钮居中，保存按钮文案改为“保存”。
- 已删除投票账户输入框右侧多余扫码按钮，扫码只保留弹窗下方统一的“开启扫码”按钮。
- 已将“设置投票账户”弹窗收敛为 500px 投票账户输入宽度，弹窗对应缩短，并使用均匀分布让市/镇注册项左侧、中间、右侧间距一致。
- 已将新建公民档案页的“创建档案 / 取消”按钮移动到“新建公民档案”标题同一行右侧。
- 已按选举资格收紧投票账户设置入口：无选举资格公民的投票账户按钮不可操作，弹窗打开和保存流程二次拦截，后端保存钱包接口同步拒绝无选举资格或非正常状态档案。
- 已将出生日期纳入出生信息不可变规则：公民详情编辑态只读展示出生日期，保存请求不再携带 `birth_date`，后端编辑接口拒绝 `birth_date / birth_*` 字段并移除出生日期更新 SQL。
