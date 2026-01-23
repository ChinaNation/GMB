GMB/
└─ frontend/
    └─ gongminbi/                 # 访客轻节点前端
        ├─ package.json           # Node / React Native 项目依赖管理
        ├─ app.json               # React Native 配置文件
        ├─ src/                   # 源代码文件夹
        │   ├─ App.tsx            # 应用入口文件
        │   ├─ components/        # 可复用 UI 组件（按钮、交易列表、弹窗等）
        │   ├─ pages/             # 页面模块
        │   │   ├─ Wallet/        # 钱包页面（生成临时钱包、公私钥管理）
        │   │   ├─ Transactions/  # 支付、转账、交易历史
        │   │   ├─ Info/          # 公民币信息、链上数据查询
        │   ├─ hooks/             # 自定义 Hooks（状态管理、RPC 调用）
        │   ├─ utils/             # 工具函数（金额格式化、公民币单位转换、签名校验）
        │   └─ assets/            # 图片、图标、样式文件
        ├─ android/               # Android 平台原生配置和源码
        ├─ ios/                   # iOS 平台原生配置和源码
        └─ node_modules/          # 前端依赖库（React Native、Nova WalletSDK 等）