# OnChina 前端技术文档

## 1. 功能需求

OnChina 前端是公民链内置多机构控制台，负责管理员登录、管理员目录、公民电子护照、机构登记、机构账户、资料库、审计和扫码签名确认。

## 2. 当前结构

```text
citizenchain/onchina/frontend/
├── App.tsx                    # 路由壳和一级页面入口
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
└── utils/                     # 通用 HTTP 和 notice，不放业务 API
```

## 3. 前端目录规则

- 功能模块自己的后端 API 调用放在所属功能目录的 `api.ts`。
- 通用 HTTP 封装只允许放在 `frontend/utils/http.ts`，不得承载业务接口。
- 二维码解析、生成、签名响应识别和确认页字段展示必须走 `core/qr/`。
- 业务组件不得自己解析 `QR_V1`，不得自己翻译扫码端字段名。
- 前端不得恢复独立 `frontend/api/` 或 `frontend/chain/` 业务目录。

## 4. 页面和文案规则

- 公权机构、公安局和私权机构列表必须展示连续序号。
- 机构详情页身份字段统一显示为 `身份ID`。
- 机构详情页不得展示 `SubjectProperty 类型` 或机构链上状态。
- 账户链上状态只允许在机构账户列表展示。
- 扫码确认页左侧分类名必须是中文，右侧内容必须是用户能核对的值。
- 账户字段必须展示 SS58 地址，不得把原始公钥 hex 当作普通用户字段展示。

## 5. 提示入口

所有用户提示统一由 `citizenchain/onchina/frontend/utils/notice.ts` 管理。业务组件只允许调用统一 notice 方法，禁止直接调用 Ant Design `message.*`、`Modal.confirm`、`Modal.warning` 或浏览器 `alert`。

统一入口负责：

- 同一时刻只显示一个提示。
- 将扫码签名、网络和后端错误翻译为中文。
- 将用户取消类错误显示为取消提示或静默。
- 将无法识别的英文错误降级为中文兜底提示。

业务组件捕获异常时必须把原始错误对象传给 notice 入口，不得先取 `error.message` 再传入。

`NotAllowedError` 是浏览器通用取消/拒绝错误，不得在 notice 全局层直接翻译成摄像头权限错误。摄像头扫码、passkey/WebAuthn 等浏览器能力必须在各自客户端封装中给出具体中文原因。

## 6. 首次 HTTPS 信任

登录页和已登录后台必须都提供机构 CA 证书下载入口，指向 `/api/v1/platform/ca-certificate`。未信任本节点证书时，页面应明确提示先安装机构 CA 证书并重开浏览器；安装完成前不得把 passkey 失败误提示为摄像头权限问题。

macOS 导入机构 CA 时必须提示导入“系统”钥匙串并将证书设为“始终信任”；如出现 `-25294`，先在钥匙串中删除同名旧证书，再下载当前节点的新 CA 证书重新导入。

passkey 客户端在调用 `navigator.credentials.create/get` 前必须检查 `window.isSecureContext`、`PublicKeyCredential` 和 `navigator.credentials`，并分别提示“证书未信任 / 浏览器不支持 / 用户取消”。

摄像头扫码客户端在调用 `getUserMedia` 前必须检查 HTTPS 安全上下文和 `navigator.mediaDevices.getUserMedia`，避免把证书未信任误判成摄像头权限被拒绝。

## 7. 扫码签名

管理员扫码登录、Passkey 更新、管理员集合变更和链写动作统一使用 `QR_V1`。

- OnChina 页面生成 `sign_request`。
- CitizenWallet 扫描并生成 `sign_response`。
- OnChina 页面扫描或回收签名响应并完成验签。

CitizenApp 不承担管理员登录 QR 职责。前端文案不得引导用户使用 CitizenApp 处理管理员登录签名请求。

## 8. Tab 权限

前端只按后端会话下发的 `capabilities` 渲染 tab，不在组件内重新推导业务权限。当前目标状态如下：

- `FRG` 登录且已设置 passkey 后显示完整业务 tab，包含首页、私权机构、教育机构、公权机构、市注册局和联邦注册局。
- `CREG` 登录且已设置 passkey 后也显示联邦注册局 tab，但该 tab 只能展示本省联邦注册局管理员列表，不能显示编辑、更换等操作入口。
- `NJD`、普通公权机构、私权机构和非法人组织登录后只显示“本机构管理员”tab；该 tab 只读展示本机构链上 active admin 列表，当前登录管理员自己那一行显示设置 / 更新 passkey 按钮。
- `NRC`、`PRC`、`PRB` 不显示前端 tab，登录阶段返回节点桌面端专用错误。
- `PMUL` 和其它个人主体不显示前端 tab，登录阶段返回个人多签不支持错误。
- 注册局管理员未设置 passkey 时，只显示自己机构的管理员列表入口，用于先完成本机 passkey 设置；设置完成后再显示完整业务 tab。
- 联邦注册局管理员列表的操作列只允许 `FRG` 看到；`CREG` 进入同一 tab 时必须是只读表格。

## 9. 验收

```text
npm --prefix citizenchain/onchina/frontend run build
rg "旧独立身份系统名" citizenchain/onchina/frontend --glob '!node_modules/**' --glob '!dist/**'
rg "NotAllowedError.*摄像头" citizenchain/onchina/frontend --glob '!node_modules/**' --glob '!dist/**'
```

涉及登录、权限、扫码或页面展示的变更，必须启动真实本地服务并检查真实页面；只通过 `npm run build` 不算完成。
