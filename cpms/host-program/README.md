# CPMS Host Program (MVP)

本目录是 CPMS 主机端程序（内网数据服务进程）MVP 骨架。

## 已实现接口
- `GET /api/v1/health`
- `POST /api/v1/auth/login`
- `POST /api/v1/auth/refresh`
- `POST /api/v1/archives`
- `GET /api/v1/archives`
- `GET /api/v1/archives/:archive_id`

## 运行
```bash
cd /Users/rhett/GMB/cpms/host-program
CPMS_SUPERADMIN_PASSWORD=your_password cargo run
```

可选环境变量：
- `CPMS_BIND`：监听地址，默认 `127.0.0.1:8080`
- `CPMS_SUPERADMIN_PASSWORD`：超级管理员密码，默认 `change-me`

## 说明
- 当前为内存存储版本（重启即清空），用于快速联调流程。
- `province_code` 使用常量表校验，常量复制自 `primitives/src/sheng_code.rs`（例如 `ZS`）。
- 下一步建议接入 `cpms/schema.sql` 对应 PostgreSQL 持久化。
