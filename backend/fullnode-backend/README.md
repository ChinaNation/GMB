GMB/
└─ services/
   └─ fullnode_backend/                 # 全节点后端（链外辅助功能）
      ├─ Cargo.toml                     # Rust 项目配置文件
      └─ src/
          ├─ main.rs                    # 后端启动入口
          ├─ config.rs                  # 配置文件，例如日志路径、数据库连接
          ├─ db/                        # 链外数据库（辅助数据存储）
          │   ├─ models/                # 数据模型（链外信息）
          │   │   ├─ performance.rs     # 性能监控数据结构
          │   │   ├─ logs.rs            # 节点操作日志数据结构
          │   │   └─ metrics.rs         # 统计指标数据结构
          │   └─ repository.rs          # 链外数据库接口
          ├─ routes/                    # API 路由
          │   ├─ performance.rs         # 性能监控接口
          │   ├─ logs.rs                # 日志查询接口
          │   └─ metrics.rs             # 节点指标查询接口
          ├─ services/                  # 业务逻辑（链外功能）
          │   ├─ log_service.rs         # 日志收集、存储、分析
          │   ├─ metrics_service.rs     # 性能数据采集与处理
          │   └─ alert_service.rs       # 异常告警、通知
          ├─ utils/                     # 工具函数
          │   ├─ format.rs              # 时间、数值格式化
          │   ├─ validator.rs           # 数据合法性校验
          │   └─ parser.rs              # 节点日志解析等
          └─ errors.rs                  # 错误类型定义