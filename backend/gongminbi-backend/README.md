GMB/
└─ services/
   └─ gongminbi_backend/                # 访客轻节点后端
      ├─ Cargo.toml                     # Rust 项目配置文件
      └─ src/
          ├─ main.rs                    # 后端启动入口
          ├─ config.rs                  # 配置文件（数据库、限流设置、缓存等）
          ├─ db/                        # 链下数据库
          │   ├─ models/                # 数据模型
          │   │   ├─ transaction.rs     # 访客链下交易记录
          │   │   └─ session.rs         # 会话信息、访问控制
          │   └─ repository.rs          # 数据库接口（CRUD操作）
          ├─ routes/                    # API路由
          │   ├─ transaction_routes.rs  # 发起交易、查询交易接口
          │   ├─ wallet_routes.rs       # 钱包账户查询接口（只读）
          │   └─ notification_routes.rs # 消息/提示接口（交易状态、限额提醒）
          ├─ services/                  # 业务逻辑
          │   ├─ transaction_service.rs # 交易处理、限额控制
          │   ├─ wallet_service.rs      # 钱包查询、余额查询（只读）
          │   └─ notification_service.rs # 通知逻辑
          ├─ utils/                     # 工具函数
          │   ├─ crypto.rs              # 加密工具（只读验证、签名检查）
          │   ├─ parser.rs              # 交易解析、金额解析
          │   └─ formatter.rs           # 金额格式化、公民币单位转换
          └─ errors.rs                  # 错误类型