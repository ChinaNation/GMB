# CPMS API 规范（v1，离线内网）

## 1. 约定
- Base Path: `/api/v1`
- 仅局域网访问，不暴露互联网。
- 本文接口用于“客户端 <-> 主机端程序（内网数据服务进程）”通信，不是互联网接口。
- Content-Type: `application/json`（文件上传接口除外）
- 成功响应：`{ "code": 0, "message": "ok", "data": ... }`
- 失败响应：`{ "code": <non-zero>, "message": "...", "trace_id": "..." }`

## 2. 鉴权
- 登录接口返回 `access_token`（短期）+ `refresh_token`（中期）。
- 受保护接口使用：`Authorization: Bearer <access_token>`。
- 高风险操作（重置密码、恢复备份、禁用账号）要求二次确认字段：
  - `confirm_password`（当前操作人密码）

## 3. 错误码建议
- `1001` 参数错误
- `1002` 状态不允许
- `1003` 索引号格式非法
- `1004` 文件类型不支持
- `1005` 文件大小超限
- `2001` 未登录或 token 失效
- `2002` 权限不足
- `2003` 账户已禁用
- `2004` 账户已锁定
- `3001` 用户名已存在
- `3002` 护照号已存在
- `3003` 档案索引号已存在
- `3004` 档案不存在
- `3005` 资产不存在
- `3006` 材料不存在
- `5001` 数据库错误
- `5002` 文件存储错误
- `5003` 备份包校验失败

## 4. 认证与账户

### 4.1 登录
`POST /auth/login`

请求体：
```json
{
  "username": "admin01",
  "password": "******"
}
```

返回：
```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "access_token": "jwt...",
    "refresh_token": "jwt...",
    "expires_in": 1800,
    "user": {
      "user_id": "u_xxx",
      "role": "ADMIN"
    }
  }
}
```

### 4.2 刷新令牌
`POST /auth/refresh`

### 4.3 退出登录
`POST /auth/logout`

### 4.4 修改本人密码
`POST /auth/change-password`

## 5. 管理员管理（仅超级管理员）

### 5.1 创建管理员
`POST /admin/users`

请求体：
```json
{
  "username": "admin02",
  "password": "******",
  "role": "ADMIN"
}
```

### 5.2 管理员列表
`GET /admin/users?status=ACTIVE&page=1&page_size=20`

### 5.3 禁用/启用管理员
`POST /admin/users/{user_id}/status`

请求体：
```json
{
  "status": "DISABLED",
  "confirm_password": "******"
}
```

### 5.4 重置管理员密码
`POST /admin/users/{user_id}/reset-password`

## 6. 公民档案

### 6.1 新建档案
`POST /archives`

请求体：
```json
{
  "province_code": "GD",
  "full_name": "张三",
  "birth_date": "1990-01-15",
  "gender_code": "M",
  "height_cm": 175.5,
  "passport_no": "P123456789"
}
```

返回包含系统生成的 `archive_index_no`。

### 6.2 档案详情
`GET /archives/{archive_id}`

### 6.3 档案列表检索
`GET /archives?archive_index_no=...&passport_no=...&full_name=...&page=1&page_size=20`

### 6.4 更新档案
`PUT /archives/{archive_id}`

### 6.5 档案状态变更
`POST /archives/{archive_id}/status`

请求体：
```json
{
  "status": "SUSPENDED",
  "remark": "manual review"
}
```

## 7. 照片与指纹（生物资料）

### 7.1 上传照片
`POST /archives/{archive_id}/biometrics/photo`

- `multipart/form-data`
- 文件字段：`file`

### 7.2 上传指纹
`POST /archives/{archive_id}/biometrics/fingerprint`

- `multipart/form-data`
- 文件字段：`file`

### 7.3 生物资料列表
`GET /archives/{archive_id}/biometrics`

### 7.4 删除生物资料（软删除）
`DELETE /biometrics/{asset_id}`

## 8. 档案材料

### 8.1 上传材料
`POST /archives/{archive_id}/materials`

- `multipart/form-data`
- 字段：`material_type`, `title`, `file`

### 8.2 材料列表
`GET /archives/{archive_id}/materials`

### 8.3 删除材料（软删除）
`DELETE /materials/{material_id}`

## 9. 审计与备份

### 9.1 审计日志查询
`GET /audit/logs?action=...&operator_user_id=...&time_from=...&time_to=...&page=1&page_size=50`

### 9.2 创建备份包
`POST /system/backup`

请求体：
```json
{
  "target_path": "/data/backup/cpms_20260218.pkg",
  "confirm_password": "******"
}
```

### 9.3 导入恢复包
`POST /system/restore`

- `multipart/form-data`
- 字段：`file`, `confirm_password`

### 9.4 备份/恢复记录
`GET /system/backup-records?page=1&page_size=20`

## 10. 索引号生成与校验约束
- 主机端程序自动生成 `archive_index_no`，客户端不可直接指定。
- 规则：`省代码 + 性别码 + 生日码 + 档案编号`
- 正则：`^[A-Z]{2}(M|W)[0-9]{8}[0-9]{6}$`

## 11. 幂等与审计
- 新建档案建议支持 `Idempotency-Key` 请求头。
- 所有写操作必须写入审计日志，至少包含：
  - 操作人、动作、目标对象、结果、时间、trace_id。
