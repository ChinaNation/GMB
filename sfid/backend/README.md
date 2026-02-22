# SFID Backend (最小闭环版)

## 目标流程（严格按当前需求）
1. WuminApp 发起绑定请求并上传公钥
2. SFID 管理员登录后在列表中查询/搜索
3. 管理员输入档案索引号完成绑定，系统生成 SFID 码
4. WuminApp 查询绑定结果并拿到绑定成功消息
5. WuminApp 发起投票验证，SFID 返回该公钥是否已绑定且有投票资格

## 接口
- `GET /api/v1/admin/auth/check`
- `POST /api/v1/bind/request`
- `GET /api/v1/admin/citizens?keyword=...`
- `POST /api/v1/admin/bind/confirm`
- `POST /api/v1/admin/bind/unbind`
- `GET /api/v1/bind/result?account_pubkey=...`
- `POST /api/v1/vote/verify`
- `GET /api/v1/health`

## 管理员登录（当前简化实现）
管理接口通过请求头鉴权：
- `x-admin-user`
- `x-admin-password`

默认账号来自环境变量：
- `SFID_ADMIN_USER`（默认：`admin`）
- `SFID_ADMIN_PASSWORD`（默认：`admin123`）

系统启动会预置 1 条演示绑定数据，用于页面首屏展示。

## 启动
```bash
cd sfid/backend
cargo run
```
默认监听：`127.0.0.1:8899`

如果前端跑在 `127.0.0.1:5179`，后端已内置该来源的 CORS 放行。
