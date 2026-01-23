GMB/
└─ services/
   └─ wuminapp_backend/                 # 公民轻节点后端
      ├─ Cargo.toml                     # Rust 项目配置文件
      └─ src/
          ├─ main.rs                    # 后端启动入口
          ├─ config.rs                  # 配置文件（数据库连接、消息队列、文件存储路径等）
          ├─ db/                        # 链外数据库
          │   ├─ models/                # 数据模型
          │   │   ├─ user.rs            # 用户信息（WuminApp ID、PeerID、头像、昵称、CIIC码等）
          │   │   ├─ transaction.rs     # 用户链下交易记录
          │   │   └─ session.rs         # 会话信息、登录状态
          │   └─ repository.rs          # 数据库接口（CRUD操作）
          ├─ routes/                    # API路由
          │   ├─ user_routes.rs         # 用户注册、登录、查询接口
          │   ├─ transaction_routes.rs  # 交易发起、查询接口
          │   ├─ peer_routes.rs         # 节点认证、PeerID绑定接口
          │   └─ message_routes.rs      # 消息、通知接口
          ├─ services/                  # 业务逻辑
          │   ├─ auth_service.rs        # 用户身份认证逻辑（CIIC码绑定、登录校验等）
          │   ├─ wallet_service.rs      # 钱包管理（助记词、账户生成、签名辅助）
          │   ├─ transaction_service.rs # 交易处理、校验
          │   ├─ peer_service.rs        # 节点管理、节点状态检测
          │   └─ notification_service.rs # 消息推送、提醒
          ├─ utils/                     # 工具函数
          │   ├─ crypto.rs              # 加密、签名、验证工具
          │   ├─ parser.rs              # CIIC码解析、交易解析等
          │   └─ formatter.rs           # 金额格式化、公民币单位转换等
          └─ errors.rs                  # 错误类型