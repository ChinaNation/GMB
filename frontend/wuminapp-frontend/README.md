* 公民轻节点前端软件（iOS、Android）
GMB/
└─ frontend/
    └─ wuminapp/                 # 公民轻节点前端
        ├─ package.json          # Node / React Native 项目依赖管理
        ├─ app.json              # React Native 配置文件
        ├─ src/                  # 源代码文件夹
        │   ├─ App.tsx           # 应用入口文件
        │   ├─ components/       # 可复用 UI 组件（按钮、表格、弹窗、投票列表等）
        │   ├─ pages/            # 页面模块
        │   │   ├─ Auth/         # 认证页面（CIIC、公民身份绑定）
        │   │   ├─ Wallet/       # 钱包页面（助记词、种子、公私钥管理）
        │   │   ├─ Voting/       # 投票页面（权威节点投票、轻节点投票）
        │   │   ├─ Transactions/ # 支付、转账、交易历史
        │   │   ├─ Chat/         # 私密通信页面（Matrix/端到端加密）
        │   ├─ hooks/            # 自定义 Hooks（状态管理、API 请求）
        │   ├─ utils/            # 工具函数（金额格式化、公民币单位转换、签名校验）
        │   └─ assets/           # 图片、图标、样式文件
        ├─ android/              # Android 平台原生配置和源码
        ├─ ios/                  # iOS 平台原生配置和源码
        └─ node_modules/         # 前端依赖库（React Native、Nova Wallet SDK、Matrix SDK 等）