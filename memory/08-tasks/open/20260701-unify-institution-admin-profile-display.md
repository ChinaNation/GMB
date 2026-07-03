任务需求：
- 统一公民、链上中国、node 所有机构管理员展示为链上 `AdminProfile` 完整资料。
- 所有机构管理员卡片固定为顶部“序号/操作状态”、第 1 行“姓名/职务”、第 2 行“任期/来源”、第 3 行“身份CID”、第 4 行“账户”、第 5 行“余额”；字段值为空时只留空值区域，不隐藏字段标签、不显示旧默认值。
- 三端管理员余额都必须真实读取 finalized `System.Account.free`；0 余额正常显示，查询失败、账户不存在或账户无效时只留空余额值。
- 修复 node 端仍按旧 `BoundedVec<AccountId32>` 解码机构管理员集合导致管理员错位、余额不一致和三端展示不一致的问题。
- 链上中国注册局、本机构、机构详情管理员列表都改为链上管理员资料展示，不再以本地管理员姓名作为展示真源。
- CitizenApp 已有 `AdminProfile` 模型，需把仍只显示 raw hex / SS58 的管理员入口统一为完整资料组件。
- 2026-07-02 追加：node 端管理员激活和其它所有 CitizenWallet 冷钱包扫码签名流程统一使用一个扫码签名组件，左侧签名请求二维码、右侧签名响应扫码框；删除各页面重复 `QRCodeSVG + QrScanner` 签名 UI。
- 2026-07-02 追加：三端管理员卡片字段标签统一显示冒号，例如 `姓名:`、`任期:`、`身份CID:`。

所属模块：
- citizenapp
- citizenchain/node
- citizenchain/onchina

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/chat-protocol.md
- memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md
- memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md
- memory/01-architecture/onchina/ONCHINA_TECHNICAL.md

必须遵守：
- 本任务预计不修改 `citizenchain/runtime/`；如发现必须修改 runtime，需按 runtime 二次确认硬规则单独取得确认。
- 不保留旧管理员展示分支，不继续使用本地 `admin_name` 作为跨端展示真源。
- 字段同名规则：链端语义字段统一为 `account`、`admin_cid_number`、`name`、`admin_role`、`term_start`、`term_end`、`source`；前端可按语言约定映射为 camelCase，但不得新增同义字段。
- 个人多签不是机构管理员资料，链上仍只有账户；如 UI 复用统一组件，只以账户-only 模式展示，不改变链上/投票结构。
- 改代码后必须同步更新文档、补中文注释、清理残留。

预计修改目录：
- `citizenchain/node/src/admins/admin_management/`：修正管理员集合 SCALE 解码，输出 `AdminProfile` 完整资料；涉及代码和注释。
- `citizenchain/node/src/governance/`：机构详情聚合返回管理员资料 + 可选余额；涉及代码和旧字段残留清理。
- `citizenchain/node/frontend/admins/admin-management/`：新增统一管理员资料卡片，替换管理员列表、编辑、差异展示；涉及代码、样式和注释。
- `citizenchain/node/frontend/shared/qr/`：新增 node 桌面端统一扫码签名面板和弹窗；涉及代码和中文注释。
- `citizenchain/node/frontend/app/styles/`：新增统一扫码签名弹窗/面板样式，清理旧两段式签名样式残留；涉及样式和残留清理。
- `citizenchain/node/frontend/governance/`：机构详情和提案投票状态列表使用统一管理员资料展示；涉及代码。
- `citizenchain/onchina/src/core/`：补齐链上管理员资料投影字段；涉及代码和注释。
- `citizenchain/onchina/src/auth/`：注册局管理员、本机构管理员 API 返回链上资料投影；涉及代码和本地姓名残留清理。
- `citizenchain/onchina/frontend/admins/`：新增统一管理员资料卡片，替换联邦注册局、市注册局、本机构管理员列表；涉及代码和样式。
- `citizenchain/onchina/frontend/gov/`：机构详情管理员 tab 接收统一管理员展示；涉及代码。
- `citizenapp/lib/citizen/shared/`：新增统一管理员资料卡片；涉及代码和注释。
- `citizenapp/lib/citizen/institution/`、`citizenapp/lib/citizen/public/`、`citizenapp/lib/citizen/proposal/admins-change/`：替换重复管理员卡片和 raw hex 展示；涉及代码和残留清理。
- `memory/01-architecture/`、`memory/05-modules/`：同步记录管理员资料展示统一契约；涉及文档。

验收标准：
- node 国家储委会、省储委会、省储行管理员列表来自链上 `AdminProfile` 正确解码，不再出现管理员错位导致的余额不一致。
- 公民、链上中国、node 所有机构管理员展示为统一卡片结构：顶部序号/操作状态，内容依次为姓名/职务、任期/来源、身份CID、账户、余额。
- 管理员资料字段值为空时，该字段标签仍显示，值区域留空。
- 管理员账户统一显示完整 SS58；余额统一来自 finalized `System.Account.free`，余额为空只留空值区域。
- 管理员更换、投票、激活等业务逻辑仍只使用 `account` 作为签名/权限真源，不把姓名/职务等展示字段用于权限判断。
- node 端管理员激活、管理员更换、投票、转账、多签提案、清算行节点声明和 runtime 升级签名流程不再直接实现二维码和扫码框，统一复用 `CitizenSignaturePanel` 或 `CitizenSignatureModal`。
- 管理员卡片字段标签必须显示冒号，字段值为空时仍只留空值区域。
- 相关静态检查、单元测试或真实运行态验收完成并记录结果。

执行记录：
- node 后端 `AdminAccounts.admins` 解码改为机构管理员 `AdminProfile`、个人多签 account-only；治理详情和清算行详情统一返回管理员完整资料，余额仅作为 node 附加展示字段。
- node 前端 `AdminProfileCard.tsx` 改为顶部序号/操作状态 + 五行字段结构；管理员列表、管理员集合编辑、差异卡、治理提案投票状态、清算行管理员解锁统一显示姓名/职务、任期/来源、身份CID、账户、余额，并补批量 finalized 余额读取。
- OnChina 后端链读投影补齐 `admin_cid_number / admin_role / term_start / term_end / source / source_label`，注册局和本机构管理员 API 返回链上资料；移除联邦注册局、市注册局本地管理员姓名 PATCH 动作。
- OnChina 前后端为联邦注册局、市注册局、本机构管理员列表补 `balance_fen`，前端 `admins/AdminProfileCard.tsx` 改为深色桌面卡片和五行字段结构，并移除本地管理员姓名编辑入口。
- CitizenApp `lib/citizen/shared/admin_profile_card.dart` 改为顶部序号/激活状态 + 五行字段结构；治理机构管理员页、公开机构管理员页、管理员账户详情、管理员集合编辑和变更差异统一批量读取 finalized 余额后展示。
- 架构文档和模块文档已同步记录链上 `AdminProfile` 展示契约、真实余额读取口径、固定标签、缺值留空和旧本地姓名编辑入口移除。
- 2026-07-02 追加：node 端新增 `frontend/shared/qr/CitizenSignaturePanel.tsx` 和 `CitizenSignatureModal.tsx`；管理员激活弹窗改为居中弹层，签名请求二维码和签名响应扫码框并排展示。
- 2026-07-02 追加：node 端管理员更换、治理投票、协议升级、开发升级、清算行节点声明、链上转账、机构多签转账提案、安全基金提案和手续费划转提案全部迁移到统一扫码签名面板。
- 2026-07-02 追加：Node、OnChina、CitizenApp 管理员资料卡片字段标签统一追加冒号，并同步调宽标签列避免 `身份CID:` 换行。
- 2026-07-02 追加修正：Node、OnChina、CitizenApp 管理员资料卡片取消固定标签列宽，改为标签内容自适应宽度，标签和值之间仅保留小间距，避免 `账户:` 与账户值距离过远。
- 2026-07-02 追加修正：node 端统一签名弹窗改为业务标题居中，左右面板固定为“扫码签名 / 识别签名”，说明文案统一使用“公民钱包”；内部 request id 和签名账户地址都不展示。
- 2026-07-02 追加修正：node 端统一签名弹窗删除识别框下方取消按钮，右上角关闭按钮改为 flex 居中，二维码框和识别框统一为等尺寸方框。
- 2026-07-02 追加修正：node 端统一签名面板的提交中、成功、失败状态框也统一为等尺寸方框，避免扫码识别区在状态切换时改变高度。
- 2026-07-02 追加修正：`QrScanner` 使用 ref 保存最新扫码回调，倒计时刷新不再导致摄像头 stop/start，避免签名弹窗识别框黑屏闪烁。
- 2026-07-02 追加修正：node 端统一签名面板删除签名账户地址展示和相关传参；弹窗标题固定在头部中间列第一行横向居中，避免标题被挤到左侧竖排显示。
- 2026-07-02 追加：文档已同步记录 node 端扫码签名 UI 统一组件边界；地址扫码填入和通信节点配对二维码不属于签名组件范围。

验收结果：
- 2026-07-02 补充验收：`npm --prefix citizenchain/node/frontend run build`：通过；构建产生的前端生成物无残留 diff。
- 2026-07-02 补充验收：`npm --prefix citizenchain/onchina/frontend run build`：通过；构建产生的 `dist/` 哈希文件和 `index.html` 已清理回源码状态。
- 2026-07-02 补充验收：`cargo check --manifest-path citizenchain/Cargo.toml -p node`：通过。
- 2026-07-02 补充验收：`cargo check --manifest-path citizenchain/Cargo.toml -p onchina`：通过。
- 2026-07-02 补充验收：`dart analyze ...` 与 `flutter analyze ...`（本次改动的 7 个 Dart 文件）：通过。
- 2026-07-02 运行态冒烟：启动 node 前端预览 `http://127.0.0.1:5181/` 和 OnChina 前端预览 `http://127.0.0.1:5182/`，入口 HTML 与关键 JS/CSS 资源均可通过 HTTP 访问；预览服务已停止。登录后的真实管理员数据页仍需 Tauri `invoke`、真实 OnChina 登录态、链节点和账户数据环境做人工复核。
- 2026-07-02 追加验收：`npm --prefix citizenchain/node/frontend run build`：通过；`QRCodeSVG / QrScanner / qr-container / transfer-qr-box / 已签名，扫描响应` 残留扫描确认签名页面已统一到组件，地址扫码和通信配对二维码保留。
- 2026-07-02 追加验收：`npm --prefix citizenchain/onchina/frontend run build`：通过；构建产生的 `dist/` 哈希文件和 `index.html` 已清理回源码状态。
- 2026-07-02 追加验收：`dart analyze citizenapp/lib/citizen/shared/admin_profile_card.dart`：通过。
- 2026-07-02 追加运行态预览：启动 node 前端预览 `http://127.0.0.1:5181/` 并用浏览器打开，页面标题、导航和资源正常渲染，控制台无 error；普通浏览器缺少 Tauri `invoke`，首页数据区出现预期的 `invoke` 缺失提示，预览服务已停止。
- 2026-07-02 追加二次验收：`npm --prefix citizenchain/node/frontend run build` 已通过；扫描确认 `requestId=`、`actions=`、旧签名标题、`请求 ID`、旧 CitizenWallet/冷钱包文案在 node 前端源码中无残留；`git diff --check` 通过；再次启动 node 前端预览 `http://127.0.0.1:5181/` 并用浏览器打开，入口资源正常加载、控制台无 error，预览服务已停止。
- 2026-07-02 追加三次验收：删除 node 端统一签名面板的签名账户地址展示后，`npm --prefix citizenchain/node/frontend run build` 通过；源码与相关文档扫描确认旧展示字段名无残留；再次启动 node 前端预览 `http://127.0.0.1:5181/`，入口页面正常加载、控制台无 error、页面文本无旧展示字段，预览服务已停止。
- `npm --prefix citizenchain/node/frontend run build`：通过。
- `npm --prefix citizenchain/onchina/frontend run build`：通过；构建产生的 `dist/` 生成物已清理回本任务前状态。
- `cargo check --manifest-path citizenchain/Cargo.toml -p onchina`：通过。
- `cargo check --manifest-path citizenchain/Cargo.toml -p node`：通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p node admins::admin_management::codec -- --nocapture`：通过，3 个 codec 测试覆盖机构 `AdminProfile` 和个人多签 account-only 解码。
- `dart analyze ...` 与 `flutter analyze ...`（本任务改动的 7 个 Dart 文件）：通过。
- 真实页面预览：`npm preview` 启动 node `http://localhost:5181/` 和 OnChina `http://localhost:5182/`；node 首页/国家储委会 tab 可加载但普通浏览器缺少 Tauri `invoke`，管理员数据页无法直连；OnChina 可加载管理员扫码登录页且控制台无 error。管理员列表的真实数据页仍需 Tauri/真实 OnChina 登录态和链节点环境做人工运行态复核。

残留清理：
- 清理旧前端 `updateCityRegistryName`、`updateFederalRegistryName` API 封装和编辑入口。
- 清理 OnChina 旧 `UPDATE_SUBORDINATE_REGISTRY`、`UPDATE_GOVERNING_REGISTRY` 动作枚举、路由、handler、apply/preview 函数。
- 清理 node 旧 account-only 管理员列表读口 `fetch_admins_by_cid_number` / `fetch_admins`，避免绕过完整资料投影。
- 扫描确认 `adminsSs58/admins_ss58`、旧本地姓名 PATCH 动作和旧改名 API 无残留。
- 清理 node 端签名页面旧的 `QRCodeSVG + QrScanner` 两段式 UI、`已签名，扫描响应` 按钮、`qr-container`、`transfer-qr-box` 等旧样式残留；仅保留统一签名组件、地址扫码和通信配对二维码。
