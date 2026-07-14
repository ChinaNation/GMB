# OnChina 前端技术文档

## 1. 功能需求

OnChina 前端是公民链内置多机构工作台，负责管理员登录、工作台分发、管理员目录、公民电子护照、机构登记、机构账户、资料库、审计和扫码签名确认。

## 2. 当前结构

```text
citizenchain/onchina/frontend/
├── App.tsx                    # 登录态刷新、全局布局和 workspace 路由壳
├── auth/                      # 登录、AuthContext、登录态类型和 api.ts
├── admins/                    # 注册局管理员列表、本机构管理员只读页和扫码签名前端流程
├── accounts/                  # 机构账户组件
├── address/                   # 地址库查询和地址链写 call data 生成页面
├── china/                     # 行政区划元数据 API 与本地缓存
├── citizens/                  # 公民电子护照管理界面
├── core/                      # 通用组件、共享 UI、扫码签名面板和 QR 工具
│   └── qr/                    # QR_V1 解析、生成和签名响应识别
├── docs/                      # 机构资料库组件
├── gov/                       # 公权机构页面入口
├── private/                   # 私权机构页面入口和六类私权机构子模块
├── subjects/                  # 主体共享类型、字段标签和链端公开查询封装
├── theme/                     # 主题变量和样式边界
├── utils/                     # 通用 HTTP 和 notice，不放业务 API
└── workspace/                 # 多机构工作台路由、通用壳和机构专属 UI 挂载
    ├── registry/              # 注册局工作台挂载层,只搬迁既有注册局 UI 调度
    ├── judicial/              # 司法院工作台,按操作/显示/记录分类
    └── generic/               # 普通机构工作台兜底
```

## 3. 前端目录规则

- 功能模块自己的后端 API 调用放在所属功能目录的 `api.ts`。
- 通用 HTTP 封装只允许放在 `frontend/utils/http.ts`，不得承载业务接口。
- 二维码解析、生成、签名响应识别和确认页字段展示必须走 `core/qr/`。
- 业务组件不得自己解析 `QR_V1`，不得自己翻译扫码端字段名。
- 前端不得恢复独立 `frontend/api/` 或 `frontend/chain/` 业务目录。
- 机构差异不得继续塞进 `App.tsx`。新增机构 UI 必须进入 `workspace/<机构类>/` 或对应业务目录，`App.tsx` 只保留登录态、布局和工作台路由。

## 4. 页面和文案规则

- 公权机构、公安局和私权机构列表必须展示连续序号。
- 机构详情页身份字段统一显示为 `身份ID`。
- 公民列表和详情页身份字段统一显示为 `身份CID`;护照号字段显示为 `护照号`。
- 公民列表姓名由 `citizen_family_name + citizen_given_name` 组合展示;新增弹窗必须拆成“姓”和“名”两个必填输入框。
- 机构详情页不得展示 `SubjectProperty 类型` 或机构链上状态。
- 账户链上状态只允许在机构账户列表展示。
- 扫码确认页左侧分类名必须是中文，右侧内容必须是用户能核对的值。
- 账户字段必须展示 SS58 地址，不得把原始公钥 hex 当作普通用户字段展示。
- 联邦注册局管理员进入公民入口时先显示本省城市卡片,进入某市后默认分页显示该市全部公民;市注册局管理员直接进入本市公民列表。
- 公民列表页标题上方显示 `xx省 · xx市`,列表工具栏左侧显示“公民列表”,右侧放搜索框和“新增公民”按钮;搜索框为空时表示当前市全部公民。
- 公民列表使用 cursor 分页,不得恢复 offset 分页或“空搜索清空列表”的旧行为。
- 点击公民列表行进入公民详情页,不得再弹出公民详情 Modal。
- 新增公民弹窗不得出现手填身份 CID、手填护照有效期、居住省市选择或投票账户公钥输入框。
- 新增公民弹窗必须展示当前办理城市对应的居住省市,只允许选择居住镇;出生省市镇必须选择。
- 新增公民请求只提交 `province_name / city_name / town_code` 和 `birth_*` 字段;不得向后端发送旧的第二套居住字段。
- 公民详情页负责链上身份上链:未满 16 周岁、无选举资格或档案非正常时禁用推送;推送时必须先选择“投票身份”或“参选身份”,再录入钱包账户、生成目标公民钱包签名二维码,验签后展示注册局管理员链上交易二维码。
- “投票身份”提交 `identity_level=voting`,链交易为 `CitizenIdentity.register_voting_identity(10.0)`;“参选身份”提交 `identity_level=candidate`,链交易为 `CitizenIdentity.upgrade_to_candidate_identity(10.1)`。
- 公民详情页底部必须显示公民独立资料库,资料类型固定为“护照相片 / 出生证明 / 监护人护照 / 其他材料”。该区域只调用 `citizens/api.ts` 的公民资料接口,不得复用机构资料库 `docs/DocumentLibrary.tsx`。
- 投票账户只有一个输入框,用于填写 SS58 地址或点击扫码图标回填账户;提交后列表和详情只显示 SS58 地址。
- 机构管理员列表使用 `admins/InstitutionAssignmentCard.tsx` 展示管理员钱包、岗位、任期、任职来源和余额；同一钱包在同一机构有多个岗位时按任职分别展示。岗位权限不作为卡片字段，由对应业务模块按硬规则决定。
- 注册局管理员列表保持既有表格布局。非注册局机构的本机构管理员列表必须使用卡片墙，桌面端一行两张管理员卡片，小屏一行一张；不得再显示“管理员信息 / 操作”两列表头。
- 非注册局本机构管理员卡片中，当前登录管理员自己的 passkey 按钮文案固定为“密钥”，按钮放在“余额”行右侧靠右；未设置 passkey 时继续用红点提示。
- 管理员列表不得提供本地 `admin_name` 编辑入口；联邦注册局管理员岗位目录完全只读，换届由治理业务写入 entity 后自动反映。市注册局本地登记目录仍可新增/删除，但不得成为链上管理员资格或岗位真源。
- 非注册局工作台“显示”页必须调用 `/api/v1/admin/own-institution` 展示本机构完整信息，至少包括机构全称、简称、身份ID、机构码、机构类别、主体状态、行政层级、辖区、盈利属性、主账户、主账户地址、主账户状态、账户数量和创建时间；法定代表人姓名、CID、账户读取 entity 链上公开字段，有值时显示；证件照片仍为 OnChina 链下字段。教育分类、私权类型、合伙类型、所属法人等字段有值时再显示。
- 立法法律列表和详情页的版本显示必须优先使用后端 `LawView.versionTitle/versionTitleEn`；只有链上版本标签为空时才显示 `vN`，不得在前端硬编码 `v1=创世版`。

## 5. 提示入口

所有用户提示统一由 `citizenchain/onchina/frontend/utils/notice.ts` 管理。业务组件只允许调用统一 notice 方法，禁止直接调用 Ant Design `message.*`、`Modal.confirm`、`Modal.warning` 或浏览器 `alert`。

统一入口负责：

- 同一时刻只显示一个提示。
- 将扫码签名、网络和后端错误翻译为中文。
- 优先按后端 `ApiError.error_code` 映射业务提示；例如注册局主账户缺失使用 `ONCHINA_REGISTRY_MAIN_ACCOUNT_MISSING`,不得让稳定业务错误退化为通用 400 文案。
- 将用户取消类错误显示为取消提示或静默。
- 将无法识别的英文错误降级为中文兜底提示。

业务组件捕获异常时必须把原始错误对象传给 notice 入口，不得先取 `error.message` 再传入。

`NotAllowedError` 是浏览器通用取消/拒绝错误，不得在 notice 全局层直接翻译成摄像头权限错误。摄像头扫码、passkey/WebAuthn 等浏览器能力必须在各自客户端封装中给出具体中文原因。

## 6. 首次 HTTPS 信任

登录页和已登录后台必须在当前页面不是可信 HTTPS 安全上下文时提供机构 CA 证书下载入口，指向 `/api/v1/platform/ca-certificate`。当前页面已经是 `https:` 且 `window.isSecureContext=true` 时，说明浏览器已信任当前 OnChina HTTPS 页面，不得继续显示 CA 下载安装提示。未信任本节点证书时，页面应明确提示先安装机构 CA 证书并重开浏览器；安装完成前不得把 passkey 失败误提示为摄像头权限问题。

macOS 导入机构 CA 时必须提示导入“系统”钥匙串并将证书设为“始终信任”；如出现 `-25294`，先在钥匙串中删除同名旧证书，再下载当前节点的新 CA 证书重新导入。

passkey 客户端在调用 `navigator.credentials.create/get` 前必须检查 `window.isSecureContext`、`PublicKeyCredential` 和 `navigator.credentials`，并分别提示“证书未信任 / 浏览器不支持 / 用户取消”。

摄像头扫码客户端在调用 `getUserMedia` 前必须检查 HTTPS 安全上下文和 `navigator.mediaDevices.getUserMedia`，避免把证书未信任误判成摄像头权限被拒绝。

## 7. 扫码签名

管理员扫码登录、Passkey 更新、管理员集合变更和链写动作统一使用 `QR_V1`。

- OnChina 页面生成 `sign_request`。
- CitizenWallet 扫描并生成 `sign_response`。
- OnChina 页面扫描或回收签名响应并完成验签。

CitizenApp 不承担管理员登录 QR 职责。前端文案不得引导用户使用 CitizenApp 处理管理员登录签名请求。

## 8. 工作台权限

前端只按后端会话下发的 `workspace` 和 `capabilities` 渲染工作台，不在组件内重新推导业务权限。当前目标状态如下：

- `FRG` 登录且已设置 passkey 后进入 `registry` 工作台，显示完整注册局业务入口，包含公民、私权机构、教育机构、公权机构、市注册局和联邦注册局。
- `CREG` 登录且已设置 passkey 后进入 `registry` 工作台，也显示联邦注册局入口，但该入口只能展示本省联邦注册局管理员列表，不能显示编辑、更换等操作入口。
- `NJD` 登录后进入 `judicial` 工作台，不复用注册局 tab。页面按“操作 / 显示 / 记录”分类；显示页只读展示本机构完整信息和本机构链上 active admin 卡片列表，当前登录管理员自己那张卡片显示“密钥”按钮。
- 普通公权机构、私权机构和非法人组织登录后进入 `generic` 工作台；当前至少在显示页只读展示本机构链上 active admin 列表，后续专属 UI 按机构能力接入。
- `NRC`、`PRC`、`PRB` 不显示前端工作台入口，登录阶段返回节点桌面端专用错误。
- `PMUL` 和其它个人主体不显示前端工作台入口，登录阶段返回个人多签不支持错误。
- 注册局管理员未设置 passkey 时，只显示自己机构的管理员列表入口，用于先完成本机 passkey 设置；设置完成后再显示完整注册局业务入口。
- 联邦注册局管理员列表的操作列只允许 `FRG` 看到；`CREG` 进入同一入口时必须是只读表格。

## 9. 验收

```text
npm --prefix citizenchain/onchina/frontend run build
rg "旧独立身份系统名" citizenchain/onchina/frontend --glob '!node_modules/**' --glob '!dist/**'
rg "NotAllowedError.*摄像头" citizenchain/onchina/frontend --glob '!node_modules/**' --glob '!dist/**'
```

涉及登录、权限、扫码或页面展示的变更，必须启动真实本地服务并检查真实页面；只通过 `npm run build` 不算完成。
