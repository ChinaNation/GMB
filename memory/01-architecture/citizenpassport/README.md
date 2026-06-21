# CPMS 系统 README

CPMS（公民护照管理系统）是市公安局使用的离线公民档案管理系统，采用中心数据库 + 多终端浏览器访问的部署形态。

## 核心能力
- 使用 CID 签发的 `CID_CPMS_V1 / INSTALL` 安装码完成初始化。
- 安装后直接签发 `CID_CPMS_V1 / ARCHIVE` 公民档案二维码。
- 档案号不暴露省、市、CPMS 机构号。
- ARCHIVE 明文字段不暴露省、市、CPMS 机构号；归属信息只存在于加密 `geo_seal`。
- 初始管理员通过 `citizenapp` 名片二维码绑定产生。
- 操作员由管理员创建，负责录入档案和打印二维码。
- CPMS 不维护省市真源；离线显示名称来自 CID 安装码，省市代码只从 `cid_number` 内部解析。
- 资料统一存入离线内网中心数据库。

## 系统边界
- CPMS 与 CID 是两套独立系统。
- CPMS 完全离线运行，不连接互联网。
- CPMS 与 CID 无在线接口直连。
- CPMS 不直接对接区块链，区块链交互由 CID 负责。
- 未使用 CID 签发 INSTALL 初始化的伪 CPMS，无法生成可被 CID 验证通过的 ARCHIVE。

## 部署形态
- 一台内网主机：部署 CPMS backend、数据库、文件存储。
- 多台作业电脑：使用浏览器访问同一个 Web 登录页。
- 每台电脑同一时刻登录一个管理员账号。
- 后端只监听 `127.0.0.1:8080`，局域网入口统一由 nginx 提供 `https://www.citizenpassport.com/`。

## 交付与安装
- 正式安装包：`citizenpassport-ubuntu24-amd64.run` 或 `citizenpassport-ubuntu24-arm64.run`
- 主机安装：`sudo ./citizenpassport-ubuntu24-<arch>.run`
- 终端访问地址：`https://www.citizenpassport.com/login`
- 安装手册：安装后保存在 `/opt/citizenpassport/docs/CitizenPassport安装配置手册.md`

安装脚本负责：
- 安装并启动 PostgreSQL。
- 创建 CPMS 数据库与账号。
- 修正数据库 owner 和 schema 权限；正式数据库结构统一由后端 `MIGRATOR.run()` 创建。
- 安装前端静态文件到 `/opt/citizenpassport/frontend`，并写入 `CPMS_FRONTEND_DIR`，由后端 8080 直接托管页面。
- 安装并启动 `citizenpassport-backend` systemd 服务。

当前两码基线不提供旧库迁移兼容；旧数据库可清空后按当前基准结构重新初始化。

## 开发环境
- 开发/测试前先执行：`./scripts/dev_db_setup.sh`
- 前端本地联调使用 `cd citizenpassport/frontend && npm run dev`；正式部署不依赖 Vite dev server。

## 角色与权限
- `admins`：初始化绑定或由管理员新增，可创建管理员、编辑管理员姓名、删除非初始管理员、维护地址、修改公民状态和选举资格。
- `operators`：录入档案、查询档案、下载/打印 ARCHIVE 二维码。

## 密钥边界
- 管理员登录公钥：用于 CPMS 管理员登录与权限控制。
- ARCHIVE 签发密钥：CPMS 初始化时生成，用于签发档案二维码。
- `install_secret`：CID 生成并写入 INSTALL，用于 CPMS 生成 `geo_seal`。
- 上述密钥不得混用。

## 仓库结构
- `frontend/`：统一前端目录。
- `backend/`：后端服务。
- `backend/db/`：数据库脚本。
- `deploy/`：离线内网部署配置。
- `scripts/`：构建、启动、维护脚本。
- `memory/01-architecture/citizenpassport/CPMS_TECHNICAL.md`：完整技术开发文档。
