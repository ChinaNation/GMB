# **<<后端模块/backend>>**  开发文档
# 目录  

- <details>
  <summary>1. 管理后端/manage-backend</summary>

  [1.1. 功能需求](#11功能需求) 
  [1.2. 技术开发](#12技术开发)
  </details>

- <details>
  <summary>2. 全节点后端/fullnode-backend</summary>

  [2.1. 功能需求](#21功能需求)
  [2.2. 技术开发](#22技术开发)
  </details>

- <details>
  <summary>3. 轻节点后端/wuminapp-backend</summary>

  [3.1. 功能需求](#31功能需求)
  [3.2. 技术开发](#32技术开发)
  </details>

- <details>
  <summary>4. 联储会后端/fcrc-backend</summary>

  [4.1. 功能需求](#41功能需求)
  [4.2. 技术开发](#42技术开发)
  </details>

# 1.管理后端/manage-backend
## 1.1.功能需求
## 1.2.技术开发
* 初步架构
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

****
# 2.全节点后端/fullnode-backend
## 2.1.功能需求
## 2.2.技术开发
* 初步架构
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

****
# 3.轻节点后端/wuminapp-backend
## 3.1.功能需求
## 3.2.技术开发
* 初步架构
GMB/
└─ services/
   └─ wuminapp_backend/                 # 轻节点后端
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

****
# 4.联储会后端/fcrc-backend
## 4.1.功能需求
* 模块简介：联邦公民储备委员会模块是公民币的最终进化方向，成为主权区块链上的国家央行，负责新币发行后的借贷、缩表扩表、降准降息等操作；
* 中华民族联邦共和国公民储备委员会架构：
![alt text](https://raw.githubusercontent.com/ChinaNation/GMB/main/wenku/联储会架构图.png)

## 4.2.技术开发
* 借贷模块/lending；借贷利息计算/interest；资产负债表/assets