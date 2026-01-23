GMB/
└─ services/
   └─ manage_backend/                # 管理端后端（链外，仅辅助功能）
      ├─ Cargo.toml                  # Rust 项目配置文件
      └─ src/
          ├─ main.rs                 # 后端启动入口
          ├─ config.rs               # 配置文件，例如数据库连接
          ├─ db/                     # 数据库相关（链外辅助数据）
          │   ├─ models/             # 数据模型（仅链外信息）
          │   │   └─ logs.rs         # 操作日志、报表记录
          │   └─ repository.rs       # 数据库接口（链外日志、报表）
          ├─ routes/                 # API 路由（仅链外查询接口）
          │   └─ reports.rs           # 报表、日志接口
          ├─ utils/                  # 工具函数（格式化、校验等）
          │   ├─ format.rs            # 金额、时间格式化
          │   └─ validator.rs         # 数据合法性校验
          └─ errors.rs               # 错误类型定义