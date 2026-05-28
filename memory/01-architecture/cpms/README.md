# CPMS 系统 README

CPMS（公民护照管理系统）是市公安局使用的离线公民档案管理系统，采用中心数据库 + 多终端浏览器访问的部署形态。

## 核心能力
- 使用 SFID 签发的 `SFID_CPMS_V1 / INSTALL` 安装码完成初始化。
- 安装后直接签发 `SFID_CPMS_V1 / ARCHIVE` 公民档案二维码。
- 档案号不暴露省、市、CPMS 机构号。
- ARCHIVE 明文字段不暴露省、市、CPMS 机构号；归属信息只存在于加密 `geo_seal`。
- 超级管理员通过 `wuminapp` 名片二维码绑定产生。
- 操作员由超级管理员创建，负责录入档案和打印二维码。
- CPMS 不维护省市真源；离线显示名称来自 SFID 安装码，省市代码只从 `sfid_number` 内部解析。
- 资料统一存入离线内网中心数据库。

## 系统边界
- CPMS 与 SFID 是两套独立系统。
- CPMS 完全离线运行，不连接互联网。
- CPMS 与 SFID 无在线接口直连。
- CPMS 不直接对接区块链，区块链交互由 SFID 负责。
- 未使用 SFID 签发 INSTALL 初始化的伪 CPMS，无法生成可被 SFID 验证通过的 ARCHIVE。

## 部署形态
- 一台内网主机：部署 CPMS backend、数据库、文件存储。
- 多台作业电脑：使用浏览器访问同一个 Web 登录页。
- 每台电脑同一时刻登录一个管理员账号。
- 后端监听地址需对局域网开放，建议设置 `CPMS_BIND=0.0.0.0:8080`。

## 交付与安装
- 安装包构建：`./scripts/build_linux_host_installer.sh`
- 主机安装：解压安装包后执行 `sudo ./install_host.sh`
- 终端访问地址：`http://<主机内网IP>:8080/login`

安装脚本负责：
- 安装并启动 PostgreSQL。
- 创建 CPMS 数据库与账号。
- 导入 `schema.sql` 与 `seed.sql`。
- 安装并启动 `cpms-backend` systemd 服务。

当前两码基线不提供旧库迁移兼容；旧数据库可清空后按当前基准结构重新初始化。

## 开发环境
- 本地快速联调可使用 `deploy/docker-compose.yml` 与 `scripts/install_and_start.sh`。
- 开发/测试前先执行：`./scripts/dev_db_setup.sh`

## 角色与权限
- `SUPER_ADMIN`：绑定产生，可创建/禁用操作员、维护地址、修改公民状态和选举资格。
- `OPERATOR_ADMIN`：录入档案、查询档案、下载/打印 ARCHIVE 二维码。

## 密钥边界
- 管理员登录公钥：用于 CPMS 管理员登录与权限控制。
- ARCHIVE 签发密钥：CPMS 初始化时生成，用于签发档案二维码。
- `install_secret`：SFID 生成并写入 INSTALL，用于 CPMS 生成 `geo_seal`。
- 上述密钥不得混用。

## 仓库结构
- `frontend/`：统一前端目录。
- `backend/`：后端服务。
- `backend/db/`：数据库脚本。
- `deploy/`：离线内网部署配置。
- `scripts/`：构建、启动、维护脚本。
- `memory/01-architecture/cpms/CPMS_TECHNICAL.md`：完整技术开发文档。
