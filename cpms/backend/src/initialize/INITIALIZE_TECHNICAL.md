# CPMS Initialize 模块技术文档

## 1. 模块定位
`backend/src/initialize/` 负责 CPMS 安装初始化全流程。

该模块统一承载“首次安装引导、安装码验签、初始化数据落盘、超级管理员绑定”能力。

## 2. 负责范围
- 安装引导状态查询：`/api/v1/install/status`
- 使用 SFID 初始化二维码执行首次初始化：`/api/v1/install/initialize`
- 超级管理员绑定：`/api/v1/install/super-admin/bind`
- 安装文件读写与校验（`cpms_install_init.json`）
- 初始化密钥材料生成（固定 3 把二维码签名密钥：`K1/K2/K3`）
- 初始化阶段超级管理员固定映射（`K1/K2/K3 -> u_super_admin_01/02/03`）

## 3. 关键数据
- `BootstrapInstallData`：安装期持久化数据模型
- `RuntimeInstallData`：运行期初始化快照
- `QrSignKeyRuntime`：运行期签名密钥（含 secret_bytes）

## 4. 安全约束
- `SFID_ROOT_PUBKEY` 必须存在，初始化二维码必须验签通过
- 安装文件已存在时拒绝重复初始化
- 超级管理员绑定数量上限为 3
- 同一 `managed_key_id` 和 `admin_pubkey` 不允许重复绑定
- 安装文件写入后按 Unix `0600` 权限收敛

## 5. 模块边界
- 初始化相关路由与辅助算法全部集中在 `initialize` 模块
- 登录认证在 `login` 模块
- 权限校验在 `authz` 模块
- 业务管理在 `super_admin` / `operator_admin` 模块

## 6. 与主程序关系
主程序只做路由装配与应用启动，初始化能力通过 `initialize::router()` 统一挂载。
