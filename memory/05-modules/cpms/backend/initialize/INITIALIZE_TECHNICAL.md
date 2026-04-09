# CPMS Initialize 模块技术文档

## 1. 模块定位
`backend/src/initialize/` 负责 CPMS 安装初始化全流程。

该模块统一承载“首次安装引导、安装码验签、初始化数据入库、省级管理员绑定”能力。

## 2. 负责范围
- 安装引导状态查询：`/api/v1/install/status`
- 使用 SFID 初始化二维码执行首次初始化：`/api/v1/install/initialize`
- 省级管理员绑定：`/api/v1/install/super-admin/bind`
- 初始化密钥材料生成（固定 3 把二维码签名密钥：`K1/K2/K3`）
- 初始化阶段省级管理员固定映射（`K1/K2/K3 -> u_super_admin_01/02/03`）

## 3. 数据落库（PostgreSQL）
- `system_install`：安装状态与 `site_sfid`
- `qr_sign_keys`：机构二维码签名密钥（含用途、状态、公钥、私钥材料）
- `admin_users`：超级管理员绑定结果

## 4. 安全约束
- CPMS 为离线系统，初始化时不验 QR1 签名（无 SFID 公钥）
- `system_install.site_sfid` 已存在时拒绝重复初始化
- 超级管理员绑定数量上限为 1
- `admin_pubkey` 不允许重复绑定
- QR2 生成和 QR3 盲化需要 SUPER_ADMIN 认证（登录后操作）

## 5. 模块边界
- 初始化相关路由与辅助算法全部集中在 `initialize` 模块
- 登录认证在 `login` 模块
- 权限校验在 `authz` 模块
- 业务管理在 `super_admin` / `operator_admin` 模块

## 6. 与主程序关系
主程序只做路由装配、数据库连接池与迁移初始化；初始化能力通过 `initialize::router()` 统一挂载。
