1️⃣ 项目目录：manage是前端管理软件，是国储会、省储会、省储行使用的前端操作软件；
GMB/
└─ frontend/                     # 所有节点前端统一放在这个文件夹
    └─ manage/                   # 管理端前端（国储会 / 省储会 / 省储行共用）
        ├─ tauri.conf.json       # Tauri 框架配置文件
        ├─ package.json          # Node / React 项目依赖和配置
        ├─ src/                  # React 源代码文件夹
        │   ├─ main.tsx          # 应用入口文件
        │   ├─ App.tsx           # 根组件
        │   ├─ components/       # 可复用 UI 组件（按钮、表格、弹窗等）
        │   ├─ pages/            # 页面模块
        │   │   ├─ NationalBank/     # 国储会页面
        │   │   ├─ ProvincialBank/   # 省储会页面
        │   │   ├─ ProvincialBranch/ # 省储行页面
        │   ├─ hooks/            # 自定义 Hooks（状态管理、API 请求）
        │   ├─ utils/            # 工具函数（金额格式化、校验、公民币单位转换等）
        │   └─ assets/           # 静态资源（图片、图标、样式等）
        ├─ public/               # 公共资源文件（index.html、favicon、静态图标等）
        └─ node_modules/         # 前端依赖库（React、MUI 等）

2️⃣ 技术栈与框架选择
	•	Tauri：打包桌面端（macOS / Linux），调用系统 API，安全轻量。
	•	React：实现动态界面和组件化开发。
	•	MUI (Material-UI)：UI 组件库，美观、开源、安全。
	•	TypeScript：代码强类型，更易维护。
	•	状态管理：可以用 Redux Toolkit 或 React Context 来区分不同节点角色的权限状态。

⸻
3️⃣ 角色区分设计
	•	国储会
	•	功能：
	•	提案决议发行公民币
	•	扩表 / 缩表
	•	降息 / 降准
	•	提案、投票和销毁操作
	•	提案增删改新省储会 / 省储行节点
	•	省储会 (ProvincialBank)
	•	功能：
	•	投票
	•	借贷管理
	•	处理辖区省储行借贷利息
	•	省储行 (ProvincialBranch)
	•	功能：
	•	接收省储会借贷
	•	投票决策
	•	管理辖区商业银行借贷利息

每个角色页面在 src/pages 下独立目录，通过 登录账号 + 权限判断 显示不同功能。

⸻

4️⃣ 权限与路由设计
	•	使用 React Router 管理页面路由
	•	登录后获取用户角色（国储会 / 省储会 / 省储行）
	•	根据角色控制路由和按钮权限：