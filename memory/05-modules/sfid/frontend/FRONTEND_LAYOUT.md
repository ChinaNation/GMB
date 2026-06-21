# SFID 前端目录布局

- 最后更新:2026-06-14
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-duoqian-info-layout.md`
  - `memory/08-tasks/open/20260502-114447-按业务边界重新设计并落地-sfid-联邦管理员相关前后端与-runtime-目录结构.md`
  - `memory/08-tasks/open/20260502-sfid-chain目录归并功能模块.md`
  - `memory/08-tasks/done/20260502-sfid-cpms-sheng目录整改.md`
  - `memory/08-tasks/done/20260502-sfid-frontend-api归并功能模块.md`
  - `memory/08-tasks/done/20260502-sfid-sheng-tabs.md`
  - `memory/08-tasks/done/20260502-sfid-sheng-backup-admin-ui.md`
  - `memory/08-tasks/done/20260525-sfid-cpms-archive-simplify.md`
  - `memory/08-tasks/done/20260525-sfid-bind-upload-qr.md`
  - `memory/08-tasks/done/20260525-sfid-bind-sign-request-citizenwallet-scan.md`
  - `memory/08-tasks/done/20260525-sfid-bind-copy-myid-scan-square.md`
  - `memory/08-tasks/done/20260525-175008-sfid绑定签名回执与citizenapp启动anr修复.md`
  - `memory/08-tasks/open/20260530-sfid-admins-module-unify.md`
  - `memory/08-tasks/open/20260530-sfid-province-admin-governance-passkey.md`
  - `memory/08-tasks/done/20260530-sfid-admin-permission-step2.md`
  - `memory/08-tasks/done/20260531-sfid-admin-ui-closeout.md`
  - `memory/08-tasks/done/20260531-sfid-admin-model-no-status.md`
  - `memory/08-tasks/open/20260531-sfid-signature-modal-stack.md`
  - `memory/08-tasks/open/20260601-sfid-official-institutions.md`
  - `memory/08-tasks/open/20260603-sfid-gov-private-subjects.md`
  - `memory/08-tasks/done/20260603-sfid-remove-institutions-china-sqlite.md`
  - `memory/08-tasks/done/20260604-sfid-core-number-store-refactor.md`
  - `memory/08-tasks/done/20260612-181650-重构-sfid-私权机构架构-保留身份id格式-私权机构按个体经营-合伙企业-股权公司-股份公司-公益组织-注册协.md`
  - `memory/08-tasks/done/20260612-194131-sfid-private-real-module-refactor.md`
  - `memory/08-tasks/open/20260613-sfid-institution-detail-nav.md`
  - `memory/08-tasks/open/20260613-sfid-institution-list-audit-accounts.md`
  - `memory/08-tasks/open/20260614-sfid-education-classification.md`

## 当前边界

SFID 前端 `src` 源码壳、`views` 壳、全局业务 API 目录、全局链目录已删除。
前端不再保留“src + views”这层空壳,也不再维护全局业务 API 或全局链目录。
所有页面、hook、通用组件、业务 API 和链交互 API 都直接按业务目录放在 `sfid/frontend/` 下。

```text
sfid/frontend/
├── main.tsx
├── App.tsx
├── vite-env.d.ts
├── accounts/                  # 机构账户组件
├── auth/                      # 登录、AuthContext、登录态类型、auth/api.ts
├── china/                     # 行政区只读元数据 API 和确定性列表缓存
├── citizens/                  # 公民首页、绑定弹窗、citizens/api.ts
├── core/                      # 跨业务复用组件,含 CITIZEN_QR_V1 签名面板/弹窗、QR 协议、机构共享表单和详情导航布局
│   └── institution/           # 私权/教育共用机构新增表单,不承载业务 API
│   └── qr/                    # CITIZEN_QR_V1 前端解析器
├── cpms/                      # CPMS 系统管理组件和 cpms/api.ts
├── docs/                      # 机构资料库前端出口
├── education/                 # 教育机构页面入口,统一管理 JY 教育委员会、法人学校和 F+JY 分支机构
├── gov/                       # 公权机构页面入口、机构操作记录共享组件,前后端统一使用 gov 命名
├── hooks/                     # useAuth / useScope / useSfidMeta 等
├── private/                   # 六类私权机构 Shell,只负责省市选择和详情跳转
│   ├── common/                # 六类私权机构共用 API、列表、创建弹窗和单类型页面壳
│   ├── sole/                  # 个体经营页面、API、类型边界:F+GT
│   ├── partnership/           # 合伙企业页面、API、类型边界:无限合伙 F+GP / 有限合伙 S+LP
│   ├── company/               # 股权公司页面、API、类型边界:S+GQ
│   ├── corporation/           # 股份公司页面、API、类型边界:S+GF
│   ├── welfare/               # 公益组织页面、API、类型边界:S+GY
│   └── association/           # 注册协会页面、API、类型边界:S+AS
├── subjects/                  # 身份主体共享类型、字段标签和链端公开查询封装
├── admins/                    # 联邦/市管理员页面、Passkey.tsx、city_admins_api.ts、admin_security_api.ts
├── theme/
└── utils/                     # 通用工具,http.ts 只放请求封装,不放业务 API
```

## API 目录规则

- 前端不再维护独立 `api/` 目录。某个功能需要后端 API 时,直接在所属功能目录新建 `api.ts`。
- `utils/http.ts` 只放 `request`、`adminRequest`、`adminHeaders` 和 401 拦截,不得放业务接口。
- `utils/http.ts` 收到 `401` 必须抛 `AuthExpiredError` 并触发全局退出;其他业务错误抛
  `ApiError`,页面按 `errorCode` 展示,不得返回 `undefined as T`。
- 登录/会话接口放 `auth/api.ts`;登录态和角色类型放 `auth/types.ts`。
- 行政区和编码元数据接口放 `china/api.ts`;省市清单走 `/api/v1/admin/china/cities`,
  编码选项走 `/api/v1/admin/number/meta`。
- `subjects/api.ts` 只保留跨公权/私权共用的数据类型和公共边界;业务 CRUD API
  分别放 `gov/api.ts`、`private/<type>/api.ts`、`education/api.ts`、`accounts/api.ts`、`docs/api.ts`。
  机构与区块链交互继续放 `subjects/chain_duoqian_info.ts`。
- 公权机构页面入口放 `gov/`,前后端统一使用 `gov` 命名;前端不得新建 `public/` 业务目录。
- 六类私权机构入口放 `private/`;其下按 `common/sole/partnership/company/corporation/welfare/association`
  拆分六类私权机构前端边界。教育机构(JY 教育体系机构)页面入口放 `education/`;
  身份主体公共出口放 `subjects/`;账户和资料库出口分别放 `accounts/`、`docs/`。
- 不得恢复 `institutions/` 前端目录;公权 UI 归 `gov/`,私权 UI 归 `private/`,教育 UI 归
  `education/`,账户和资料库分别归 `accounts/`、`docs/`。`subjects/` 不再承载机构聚合页面组件。
- CPMS 系统管理接口放 `cpms/api.ts`;CPMS 组件放 `cpms/`。
- 公民电子护照绑定和 CPMS 状态扫码接口放 `citizens/api.ts`。
- 联邦/市管理员本地后台接口统一放 `admins/`;联邦管理员目录接口放 `admins/api.ts`,
  市管理员列表接口放 `admins/city_admins_api.ts`,Passkey 更新工具放 `admins/Passkey.tsx`。
- `core/CitizenSignaturePanel.tsx` 与 `core/CitizenSignatureModal.tsx` 是统一签名 UI;
  登录页、Passkey 更新和管理员重要操作都复用登录页同款“左二维码 + 右扫码窗口”布局。
- 管理员扫码登录页面必须明确引导 `citizenwallet` 公民钱包生成登录回执;`citizenapp` 不处理
  `login_challenge / login_receipt`,不得在登录页文案中引导到 `citizenapp`。
- `core/institution/CreateInstitutionForm.tsx` 是私权/公权/教育新增弹窗唯一表单实现;
  `private/PrivateCreateModal.tsx`、`gov/GovCreateModal.tsx` 和 `education/EducationCreateModal.tsx`
  只做本模块 API 注入,不得再复制表单逻辑。
- `core/InstitutionDetailNavLayout.tsx` 是机构详情页“BrixUI 风格连接式左侧导航 + 右侧内容区”的唯一共享布局。
  公权机构、公安局、注册局、教育机构和私权机构详情页都必须接入该布局;详情标题、身份ID、
  状态徽标、左侧图标 tab 和右侧连接式内容区都由共享布局统一承载,不得在业务详情页另行实现 tab UI。
  从列表进入详情时左侧第一个入口显示“返回列表”,市管理员直达详情时不显示返回入口。详情 tab 固定按
  “机构信息 / 管理员列表 / 账户列表 / 资料库 / 操作记录”组织,其中没有管理员数据的机构不得显示
  “管理员列表”tab。共享导航的“返回列表”图标与 CPMS 公民详情左侧导航保持一致。
- `gov/OperationRecords.tsx` 是机构操作记录唯一共享组件。所有机构详情页都必须展示“操作记录”tab,
  不得把审计日志表格重新内嵌回某个业务详情页组件。组件必须按 `target_sfid` 精确读取该机构审计,
  操作范围覆盖机构创建、详情编辑、账户创建/删除、资料上传/下载/删除和 CPMS 安装授权状态。
- `core/modalStack.ts` 是 SFID 前端弹窗层级唯一入口。普通业务弹窗固定在业务层,
  扫码账户弹窗在其上,Passkey 公民钱包签名弹窗固定在最高安全层。
- `core/qr/citizenQr.ts` 是前端 CITIZEN_QR_V1 envelope 解析唯一入口;不得恢复独立
  `frontend/qr/` 目录。
- 管理端权限类型统一为 `LOGIN_STATE / PASSKEY / PASSKEY_CHALLENGE`;前端类型必须与后端
  `admins/operation_auth.rs` 对齐,不得恢复二级权限命名。
- `PASSKEY` 业务写操作不得直接裸调用 CRUD 端点;必须先通过
  `admins/admin_security_api.ts` 触发浏览器 Passkey 并取得一次性 grant。
- `PASSKEY_CHALLENGE` 写操作必须通过 `admins/admin_security_api.ts` 的 Passkey +
  `CITIZEN_QR_V1` 公民钱包签名流程取得一次性 grant。
- `PASSKEY_CHALLENGE` 写操作触发 Passkey + 公民钱包签名时,不得为了规避遮挡而关闭编辑、新增或删除确认弹窗。
  正确顺序是:底层业务弹窗保持打开并进入 loading/禁用状态,浏览器 Passkey 原生验证完成后,
  `CitizenSignatureModal` 以最高安全层展示在所有业务弹窗前面;签名成功后先关闭签名弹窗,
  再关闭或刷新原业务弹窗。失败或取消时底层业务弹窗保留,方便用户修改后重试。
- 签名弹窗扫码按钮不得复用底层业务 loading。底层业务 loading 只负责防止重复提交;
  扫码按钮只在已经识别到签名回执并提交 `commitAdminAction` 时进入 loading/禁用,
  Passkey 完成后刚打开签名弹窗时必须允许用户点击“开启扫码”。
- Passkey 更新流程固定为 `start -> confirm -> complete`:先扫描公民钱包签名请求并确认当前管理员,
  再调用浏览器 WebAuthn 创建凭据,最后提交后端落库;不得恢复先注册浏览器凭据再公民钱包确认的流程。

## 公民绑定弹窗 UI 口径

- `citizens/BindModal.tsx` 只保留单一绑定流程：扫描/上传 CPMS 档案码、展示 citizenapp `sign_request`、扫描 citizenapp `sign_response`、提交 SFID 绑定。
- 扫码框提示统一为“点击扫描档案码”；签名回执页提示为“点击扫描签名回执”。
- 进入签名二维码展示步骤后，弹窗标题切换为“citizenapp 签名”；进入签名回执扫描页后，弹窗标题切换为“扫描签名回执”。
- 绑定签名回执的 `sign_request.id` 必须与后端保存的 `challenge_id` 完全一致;
  不得给公民绑定挑战额外添加 `bind-` 前缀,否则 SFID 后端会查不到 challenge。
- “扫描档案码”步骤同时支持摄像头扫码和上传二维码图片;上传入口只在本地用
  `utils/cameraScanner.ts` 的 `BarcodeDetector` 解析图片,解析出的二维码原文继续走同一条档案码绑定流程,
  不把图片文件上传到后端。
- “上传二维码”按钮保持纯文字按钮;同一按钮组内的“开启扫码”没有图标,上传入口也不得额外增加图标。
- `citizens/CitizensView.tsx` 公民列表中 `sfid_number` 列标题显示为“身份ID”,不改变底层字段名。
- `citizens/CitizensView.tsx` 公民列表中 `wallet_address` 列标题显示为“投票账户”；列表状态列显示“投票状态”，由 `citizen_status + voting_eligible` 计算。
- `citizens/CitizensView.tsx` 登录后不得自动加载公民全量列表；管理员输入投票账户、档案号或身份ID后，前端调用服务端精确查询并使用 `next_cursor` 翻页。
- 公民详情只展示“身份ID / 档案号 / 投票账户 / 绑定状态 / 选举权利 / 公民状态 / 有效期”，不得接收或展示签发地市归属。
- 公民身份列表右上角提供“导入年度报告”按钮，开放给所有已登录管理员；搜索框右侧内置搜索图标,
  点击图标或回车触发查询,不得再保留独立“查询”按钮；有写权限时搜索框右侧显示“新增公民”按钮。
- 更换绑定弹窗的当前记录摘要只展示“档案号 / 身份ID / 投票账户”；签名请求摘要使用“选举权利 / 公民状态 / 投票账户”。
- 绑定弹窗生成签名挑战时只提交 `mode / archive_code_payload / citizen_id`；钱包字段只能来自 CPMS `ARCHIVE` 档案码。
- `citizens/CitizensView.tsx` 的表格行点击只负责打开详情;操作栏按钮必须阻止事件冒泡,
  点击“更换绑定”不得同时触发公民详情弹窗；顶部新增入口固定显示“新增公民”。
- 本 UI 边界必须使用后端绑定协议字段：`wallet_pubkey / wallet_address / citizen_status / voting_eligible / vote_status / bind_status`。
- `china/metaCache.ts` 是 SFID 前端确定性元数据缓存边界；只允许缓存省份元数据、城市清单、公安局确定性展示列表、公权机构确定性展示列表、教育机构市详情直显的确定性市公民教育委员会列表和机构详情快照，不得缓存普通公民或普通机构精确搜索结果。
- `core/CityGrid.tsx`、市注册局城市表格和机构新增弹窗读取市清单时必须走 `loadCachedSfidCities`；市注册局城市表格和通用城市网格在已有缓存时必须先同步读取 `readCachedSfidCities` 直接显示，不得先闪出“暂无城市数据”。市注册局城市表格读取身份ID时必须调用公权机构列表的 `org_code=CITY_REGISTRY` 后端精确过滤,不得省级拉取前 300 条后前端过滤。机构类 Tab 读取省份元数据时必须走 `loadCachedSfidMeta`。
- `private/PrivateListTable.tsx` 不做普通机构本地分页承载大数据；私权机构列表必须由服务端按精确搜索条件返回分页对象，前端只按 `next_cursor` 请求下一页。`education/EducationListTable.tsx` 分两路读取:市详情空搜索直接显示本市确定性市公民教育委员会,有本地缓存时先显示缓存再后台刷新,有搜索词时按名称或身份ID精确查询法人学校和 F+JY 分支机构。`gov/GovListTable.tsx` 承载公安局确定性列表和公权机构浏览目录(自动目录 + 手动公权机构 + 公权下属非法人),进入市详情时直接显示,有缓存时先显示缓存再后台刷新只读查询结果。
- 公权机构 tab 手动新增两能力(市公安局页面无新增入口):G 公法人=新公权机构
  (代码仅 `ZF/LF/SF/JC`,排除储备体系自动目录代码,机构名称必填同市查重)/ F 非法人=公权下属非法人
  (机构代码锁死中国 `ZG`,不开放他国)。
  普通公权目录仍由后端自动生成,手动公权机构与挂公法人的非法人都进浏览目录。
  联邦管理员可在本省任意市创建,市管理员只能在本市创建;前端通过省/市详情页传入目标市,后端仍做最终 scope 校验。
- JY 教育机构统一在教育机构 tab(`education/`)管理:列表走
  `category=EDUCATION_INSTITUTION`;市详情空搜索直接展示本市市公民教育委员会,国家公民教育委员会不跨市直显,法人学校和 F+JY 分支机构按名称或身份ID精确搜索。
  私权/公权列表同步排除全部 `JY` 教育体系机构,自动生成的国家/市公民教育委员会不再留在公权目录。
- 非法人(F)不是全部从属:个体经营(F+GT)和无限合伙(F+GP)是独立非法人,创建时不选择所属法人;
  教育分校和公权下属非法人仍必须选择所属法人。P1 盈利属性由私权类型规则或所属法人继承规则决定;
  所属法人搜索由后端按 `subjects/uninorg` 地域规则预过滤——分校→本市学校本部,
  公权→本市市级/本省省级/国家级公法人。
- 法定代表人身份ID搜索必须调用 `citizens/api.ts::searchLegalRepresentativeCitizens` 并传目标机构上下文:
  新增弹窗传省、市、主体属性、机构代码、教育分类和所属法人;详情页传 `target_sfid_number`,若同一编辑表单正在改挂所属法人则临时传更新后的父级上下文。前端只做候选辅助,后端创建/更新接口仍做最终本省/本市/全国校验。
- 教育机构新增表单:机构锁死“公民教育委员会(JY)”。主体属性 G/S 学校必须单选教育机构类型
  `初学/小学/中学/大学`;主体属性 F 分支机构不使用教育机构类型字段,仍按所属法人=本市法人教育机构创建。
- 学校内部部门不是 SFID 机构主体,前端不得提供创建入口、列表项或详情 tab。
- **账户地址统一完整 SS58**:公钥(0x hex)是系统的,前端凡展示账户一律 `tryEncodeSs58` 转
  SS58(prefix=2027)且**完整显示不截断**(小号等宽字体+break-all 换行);机构操作记录
  「操作者账户」、机构账户列表「账户地址」、公民「投票账户」均按此口径;交易哈希不是账户,允许截断。
  机构账户列表必须在“账户名称”左侧显示序号列;联邦注册局账户新建入口只对联邦管理员开放,市管理员只读。
- **默认账户展示口径**:所有机构默认展示“主账户 / 费用账户”;省公民储备银行额外展示“永久质押”;
  国储会额外展示“安全基金 / 两和基金”。这些默认账户均不可删除,账户名称按系统实际保留名展示。
- **系统代码不上前端**:主体属性/盈利属性/机构代码/市代码等系统编码只在 value 与后端流转,
  前端展示与下拉选项一律纯中文(如“公法人”“盈利”“中国”“公民教育委员会”);
  私权六类新增弹窗的主体属性必须按当前 `private_type` 规则显示“私法人/非法人”,
  不得因锁定 Select 选项缺失而显示 `S/F` 代码;
  详情页“盈利属性/机构”两字段只显示中文,机构中文映射缺失时回退原代码仅作异常兜底。储备体系展示为
  “公民储备委员会 / 国储会 / 省储会 / 省储行”,教育体系展示为“公民教育委员会 / 市教委会”。
  机构操作记录「操作」列经 `gov/OperationRecords.tsx` 内的 AUDIT_ACTION_LABEL 映射中文,单一来源=后端
  append_audit_log 调用点 action 字面量,后端新增 action 必须同步补映射,未知值回退原标识。
- **审计 detail 事实与展示分离**:audit 表 detail 为 JSONB,后端写入点只存结构化事实
  (键小写蛇形,值为系统原值,禁写展示文案/Debug 格式);人话翻译全在前端
  AUDIT_DETAIL_KEY_LABEL/VALUE_LABEL 渲染器,后端新增字段须同步补键名映射,
  未知键「键名: 值」兜底,旧文本行原样显示仅作异常兜底。
- `private/PrivateShell.tsx` 不承载六类业务逻辑;六类新增、列表和 API 必须在
  `private/sole|partnership|company|corporation|welfare|association/` 内实现。
- 顶层直接显示六个私权机构 Tab:个体经营、合伙企业、股权公司、股份公司、公益组织、注册协会。
  新增时前端提交 `private_type` 和必要的 `partnership_kind`,后端锁定 `subject_property / institution / p1`;
  前端不得把 `ZG/TG` 当作私权机构选项。
- 私权机构列表按 `private_type` 精确过滤;JY 教育机构仍归教育 tab,公权下属非法人仍归公权 tab。
- 机构链上状态前端只保留“未注册 / 已注册 / 已注销”,不得出现第四状态筛选或文案。

## 管理员目录规则

- `admins/`:放联邦管理员列表、注册局视图、市管理员维护和管理员安全写操作。
- 注册局-联邦管理员列表页面由 `FederalAdminSubTab.tsx` 承接,按“序号 / 姓名 / 账户 / 操作”表格展示。
- 联邦管理员采用同级模型;每省最多 5 人,仅内置初始联邦管理员拥有删除新增联邦管理员的权限。
- 市管理员列表必须显示 `本市市管理员：x / 30`;市注册局城市表格“管理员数”列显示该市 `x / 30`;
  达到 30 人的市禁用新增按钮和新增弹窗里的市选项,但最终上限仍以后端校验为准。
- `city_admins_api.ts` 保留市管理员列表读取和姓名登录态修改。
- `admins/FederalAdminsView.tsx` 的联邦管理员列表和市管理员列表有本地缓存时必须先显示缓存,再后台刷新后端数据,避免进入注册局详情时反复空白转圈。
- `admins/FederalAdminsView.tsx` 首次按登录角色自动定位所属省时不得覆盖用户已经点击的“联邦管理员列表”页签；只有用户真正切换省份时才重置回默认市列表页签。
- `admin_security_api.ts` 负责 Passkey 注册、写操作 prepare/commit、浏览器 WebAuthn、
  `PASSKEY` grant、`PASSKEY_CHALLENGE` 公民钱包签名回执提交和管理员新增错误码文案转换。
- 管理员新增失败时，前端只能按 `ApiError.errorCode` 展示角色级重复、联邦管理员上限和市管理员上限提示，禁止解析后端
  `message`。
- 联邦管理员和市管理员都在管理员列表操作栏通过“更新密钥”使用 `Passkey.tsx`
  生成或重新生成本人 Passkey。
- 当 `auth.passkey_bound === false` 时,`Passkey.tsx` 只在当前登录管理员本人那一行的
  “更新密钥”按钮右上角显示红色角标;更新成功并刷新登录态后角标自动消失。
- 联邦/市管理员新增、删除必须走 `runSecuredAction`;编辑姓名只走登录态 PATCH 接口。
- 管理员列表不得展示状态栏,也不得保留启用/停用按钮。
- 编辑市管理员只允许调整管理员姓名;账户地址和市归属只读展示,不得在前端提交修改。
- 删除市管理员确认弹窗必须展示 SS58 地址,不直接展示 hex 公钥。
- 联邦管理员签名维护页不再作为 `App.tsx` 顶层 Tab 暴露,对应独立页面文件已删除。
- 登录角色和会话辅助类型放在 `auth/types.ts`。
- 本地开发的 Vite host 固定为 `localhost`;Passkey 开发配置依赖
  `http://localhost:5179`,不得用 `127.0.0.1` 或局域网 IP 打开前端注册 Passkey。
- `npm run dev` 使用 `vite preview --host localhost --port 5179 --strictPort`;端口被占用时
  必须失败,不得自动漂移到其它端口。

## 链交互目录规则

前端不再维护独立 `chain/` 目录。只要某功能模块需要和区块链交互,就在该功能
模块目录中创建 `chain_` 开头的文件。

| 前端文件 | 后端文件 | 职责 |
|---|---|---|
| `subjects/chain_duoqian_info.ts` | `subjects/chain_duoqian_info.rs` | 机构查询、注册信息凭证、账户列表 |

联邦/市管理员治理 Passkey/签名挑战不列入链交互表。
CPMS 系统管理也不列入链交互表,归 `cpms/`。

### `subjects/chain_duoqian_info.ts` 边界

- 不放 SFID 内部机构创建/修改页面,这些归 `frontend/gov/`、`frontend/private/`、
  `frontend/accounts/`、`frontend/docs/`。
- 不再提供“备案”按钮、备案弹窗或备案状态组件。
- SFID 前端不展示清算行相关状态;清算行属于链上组织治理概念,不属于 SFID 身份设计。
- 市公安局 Tab(标签固定"市公安局")不显示搜索框，不复用普通机构精确搜索；首次进入调用 `/api/v1/institutions/public-security`，成功后按管理员账户、角色、省市范围写入 `sfid:public-security:public-security-v1:*` 本地缓存，再次进入优先展示缓存。公安局列表前端固定每页 20 条，显示“共 X 页 / 第 Y 页”“共 N 条”“上一页”“下一页”，不得展示手动刷新按钮，本地翻页不得触发后端 cursor 请求。公安局表格列固定为“序号 / 身份ID / 公安局名称 / 所属行政区 / 业务状态”，表头和数据居中对齐，序号按当前公安局排序跨页连续编号。“业务状态”是唯一状态列(后端由 CPMS 站点+安装码+公钥绑定派生单轴):待生成安装码 → 待安装 → 待绑定身份码 → 可办理,外加 已禁用/已吊销;不得再分列展示 CPMS 状态/安装码状态等派生输入。
- 公民身份列表搜索框只允许输入档案号、身份ID、投票账户地址或投票账户公钥；SFID 前端不得出现“按姓名检索公民”的文案。
- 当前封装公开查询:
  - `getInstitutionInfo(sfidNumber)`:机构展示详情。
  - `getInstitutionRegistrationInfo(sfidNumber)`:链端注册信息凭证。
- 注册信息凭证的业务字段只有 `sfid_number / sfid_full_name / account_names[]`;
  `credential` 下字段仅用于链端验签与防重放。

## TypeScript 覆盖

`sfid/frontend/tsconfig.json` 必须覆盖根层入口与一级业务目录:

```json
[
  "main.tsx",
  "App.tsx",
  "vite-env.d.ts",
  "auth",
  "accounts",
  "china",
  "citizens",
  "core",
  "cpms",
  "docs",
  "education",
  "gov",
  "hooks",
  "private",
  "subjects",
  "admins",
  "theme",
  "utils"
]
```
