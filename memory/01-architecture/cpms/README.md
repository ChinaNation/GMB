# CPMS 系统 README
* 每个模块下面都有一个技术文档；
* 每次编写代码后都要更新技术文档；
* 需要其他模块和仓库联测的同步更新对应的技术文档；
* 编写代码的同时必须完善中文注释；
* CPMS系统只有两个md文档，一个是需求文档（README）、一个是技术文档（CPMS_TECHNICAL）。

CPMS（公民护照管理系统）是完全离线的公民资料管理系统，采用“统一前端目录（Web + 桌面壳）”形态，支持楼内多台电脑并行作业，并共享同一份中心数据。

核心能力：
- 前端 UI 体系与 SFID、CitizenNode 保持统一，统一采用 Tabler 风格。
- 省级管理员（`SHENG_ADMIN`）通过 `wuminapp` 扫码绑定产生，最多 `3` 个（`K1/K2/K3` 各对应 1 个）。
- 支持多个市级管理员（`SHI_ADMIN`），由省级管理员增删改查。
- 登录机制对齐 SFID：公钥账户 + 扫码 challenge 签名登录。
- 安装初始化时先扫码 SFID 下发的机构初始化二维码，验签通过后生成本机构 `3` 把二维码签名私钥。
- 每生成一把签名密钥即提供对应”省级管理员绑定二维码”，由 `wuminapp` 扫码后回传绑定签名。
- 当前版本（v1）密钥策略：机构内 3 把密钥并存（主用/备用/应急），按机构独立管理。
- 公钥登记二维码包含 `site_sfid`、`keys[]`、`sign_key_id`、`signature`，用于 SFID 省级管理员录入。
- 仅当“SFID 签发初始化二维码 -> CPMS 使用该码完成初始化 -> SFID 录入并激活该机构公钥”闭环完成后，SFID 才信任该机构后续业务二维码。
- 若 CPMS 未使用 SFID 签发二维码初始化，或后续录入时初始化链路不一致，SFID 必须拒绝该机构业务二维码。
- 市级管理员录入公民资料（照片、指纹、档案号等）。
- 档案号规则：`省2 + 市3 + 校验1 + 随机9 + 日期8(YYYYMMDD)`（日期为档案号创建时间）。
- 档案号校验位算法与 SFID `sfid_code` 统一：`BLAKE2b` 摘要字节和 `mod 10`。
- 省市代码源与 SFID 共用：CPMS 编译时从 SFID `sfid` 读取并内置到程序。
- 资料统一存入离线内网中心数据库，敏感数据加密存储。
- 录入完成后打印二维码，交给用户用于 SFID 绑定。

## 系统边界
- CPMS 与 SFID 是两套独立系统。
- CPMS 完全离线运行，不连接互联网。
- CPMS 与 SFID 无在线接口直连。
- CPMS 不直接对接区块链，区块链交互由 SFID 负责。

## 部署形态
- 一台内网主机：部署 CPMS backend、数据库、文件存储。
- 多台作业电脑：安装 CPMS 桌面壳，打开同一个 Web 登录页。
- 每台电脑同一时刻登录一个管理员账号。
- 后端监听地址需对局域网开放，建议设置 `CPMS_BIND=0.0.0.0:8080`（默认即为该值）。

## 交付与安装（主机 Linux，终端 Windows）
- 交付模型：只在一台 Linux 主机安装 CPMS；Windows 电脑仅用浏览器访问，不安装数据库。
- 安装包构建：`./scripts/build_linux_host_installer.sh`
- 产物：
  - `dist/cpms-host-linux-x64/`
  - `dist/cpms-host-linux-x64.tar.gz`
- 主机安装步骤：
  - 解压安装包后执行：`sudo ./install_host.sh`
  - 安装脚本自动完成：
    - 安装并启动 PostgreSQL（无需用户单独安装）
    - 创建 CPMS 数据库与账号
    - 导入 `schema.sql` 与 `seed.sql`
    - 安装并启动 `cpms-backend` systemd 服务
- 终端访问地址：`http://<主机内网IP>:8080/login`

## 定时备份到储存电脑（推荐）
- 目标：主机数据库每天自动备份到另一台大容量储存电脑。
- 安装包已包含备份脚本与 systemd timer。
- 配置步骤（在主机执行）：
  - 编辑：`sudo nano /etc/cpms/backup.env`
  - 设置：
    - `STORAGE_HOST`：储存电脑 IP
    - `STORAGE_USER`：储存电脑 SSH 用户
    - `STORAGE_PATH`：储存路径（例如 `/data/cpms-backups`）
  - 启用定时任务：`sudo systemctl enable --now cpms-backup.timer`
  - 手工测试一次：`sudo /opt/cpms/bin/backup_to_storage.sh`
  - 查看计划：`systemctl list-timers cpms-backup.timer`
- 默认策略：
  - 每天 `02:15` 自动执行备份
  - 远端与主机本地默认都为永久保留（`RETENTION_DAYS=0`、`LOCAL_RETENTION_DAYS=0`）
- 备份内容：
  - PostgreSQL 全库 `pg_dump`（custom format）
  - CPMS 运行时文件 `/var/lib/cpms/runtime`

## 开发环境（可选）
- 如需本地快速联调，可使用 `deploy/docker-compose.yml` 与 `scripts/install_and_start.sh`。

## 开发阶段数据库（必须先建库）
- 开发/测试前先执行：`./scripts/dev_db_setup.sh`
- 该脚本会自动：
  - 启动本地 PostgreSQL 容器
  - 创建开发库
  - 导入 `backend/db/schema.sql` 与 `backend/db/seed.sql`

## 角色与权限
- 省级管理员（`SHENG_ADMIN`）：
  - 来源于 `wuminapp` 扫码绑定，不由安装过程自动生成。
  - 固定上限 `3` 个，分别绑定 `K1/K2/K3`。
  - 可增删改查市级管理员。
  - 可查看审计日志与系统配置。
- 市级管理员（`SHI_ADMIN`）：
  - 可登录并执行公民资料录入、修改、查询、二维码打印。
  - 不可管理管理员账号。

## 密钥边界
- 省级管理员登录公钥：用于 CPMS 管理员登录与权限控制。
- 省级管理员密钥来源：`wuminapp` 持有并签名绑定，CPMS 仅保存公钥账户。
- SFID 验签公钥：来源于各机构 CPMS 初始化生成的 `3` 把签名私钥对应公钥。
- 二维码签名私钥来源：每个机构安装初始化时生成，不参与省级管理员初始化流程。
- 上述两套密钥体系必须分离，不复用同一密钥对。

## 仓库结构
- `frontend/`：统一前端目录
  - `frontend/web/`：Web 前端页面（当前可继续完善）
  - `frontend/desktop-shell/`：桌面壳（桌面图标与独立窗口）
- `backend/`：后端服务
  - `backend/db/`：数据库脚本（schema、seed）
- `deploy/`：离线内网部署配置
- `scripts/`：构建、启动、冒烟脚本
- `memory/01-architecture/cpms/README.md`：项目总览（当前文件）
- `memory/01-architecture/cpms/CPMS_TECHNICAL.md`：完整技术开发文档

接口规范与详细设计统一见 `memory/01-architecture/cpms/CPMS_TECHNICAL.md`（第 9 章 API 设计）。
