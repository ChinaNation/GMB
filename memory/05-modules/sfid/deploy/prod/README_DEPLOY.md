# SFID 生产部署（主库 + 备库）

## 目标拓扑
- SFID 应用云服务器：运行 `sfid-backend`。
- PostgreSQL 主库云服务器：所有写入都进入主库。
- PostgreSQL 备库云服务器：由主库流复制同步。
- 主库定时备份文件同步到备库（额外灾备层）。

## 脚本说明
- `scripts/install_postgres_primary.sh`：主库初始化与复制账号配置。
- `scripts/install_postgres_standby.sh`：备库初始化（`pg_basebackup` + 流复制）。
- `scripts/install_sfid_app.sh`：应用安装、迁移执行、systemd 服务安装。
- `scripts/backup_to_standby.sh`：主库导出并同步到备库。
- `systemd/sfid-backend.service`：SFID 后端服务。
- `systemd/sfid-backup.service` + `systemd/sfid-backup.timer`：定时备份任务。

## 执行顺序
1. 在主库执行：`install_postgres_primary.sh`。
2. 在备库执行：`install_postgres_standby.sh`。
3. 在应用服务器执行：`install_sfid_app.sh /opt/sfid /path/to/sfid-backend`。
4. 在应用服务器修改 `/etc/sfid/sfid.env`：
   - `DATABASE_URL` 指向主库（建议 `sslmode=verify-full`）。
   - 基础站点必填：`SFID_SIGNING_SEED_HEX`、`SFID_KEY_ID`、`SFID_RUNTIME_META_KEY`、`SFID_REDIS_URL`
   - 可后补的链对接项：`SFID_CHAIN_TOKEN`、`SFID_CHAIN_SIGNING_SECRET`、`SFID_CHAIN_WS_URL`、`SFID_CHAIN_GENESIS_HASH`
   - 可后补的扩展项：`SFID_PUBLIC_SEARCH_TOKEN`、`SFID_PII_KEY`
5. 启动服务：
   - `systemctl daemon-reload`
   - `systemctl enable --now sfid-backend`
6. 健康检查：
   - `curl http://127.0.0.1:8899/api/v1/health`

## 备份定时任务
1. 将 `scripts/backup_to_standby.sh` 安装到 `/opt/sfid/scripts/backup_to_standby.sh`。
2. 安装并启用 timer：
   - `systemctl enable --now sfid-backup.timer`
3. 查看下次执行时间：
   - `systemctl list-timers | grep sfid-backup`

## 注意事项
- 生产必须使用强密码并限制安全组/IP 白名单。
- `/etc/sfid/sfid.env` 禁止保留 `CHANGE_ME` 占位值；安装脚本会拒绝继续执行。
- `sfid-backend.service` 已按非 root 用户 `sfid` 运行并启用 systemd 沙箱加固。
- 应用只连接主库，不要写备库。
- 备库用于容灾与只读核验，故障切换需要明确 SOP。
- 若 Redis 与应用部署在同一台机器，推荐 `SFID_REDIS_URL=redis://127.0.0.1:6379/0`。
- 如果当前目标只是“先把网页、登录和基础后台跑起来”，可以先不填写链对接参数；这些只影响链相关接口，不影响站点基础启动。

## 自动部署（GitHub Push -> 云服务器自动更新）
适用场景：你推送 `sfid/**` 代码到 GitHub `main` 后，自动把服务器更新到最新版本。

### 一次性服务器准备
1. 先按上面的“执行顺序”完成首装，确保下面两项已经存在：
   - `/etc/sfid/sfid.env`
   - `systemd` 服务 `sfid-backend`
2. 确保部署账号具备免密 `sudo` 能力，至少允许：
   - 写 `/opt/sfid`
   - 写前端静态目录（默认 `/var/www/sfid`）
   - `systemctl restart sfid-backend`
   - `systemctl reload nginx`
3. 若要一起自动发布前端，建议让站点静态根目录指向：
   - `/var/www/sfid/current`
4. 推荐反向代理规则：
   - 站点根目录：`/var/www/sfid/current`
   - `/api/` 反代到：`http://127.0.0.1:8899`

参考 Nginx 片段：
```nginx
server {
    server_name sfid.crcfrcn.com;

    root /var/www/sfid/current;
    index index.html;

    location /api/ {
        proxy_pass http://127.0.0.1:8899;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location / {
        try_files $uri /index.html;
    }
}
```

### GitHub 侧需要配置
新增 workflow：
- `.github/workflows/sfid-deploy.yml`

必须配置的 GitHub Secrets：
- `SFID_DEPLOY_USER`：部署服务器 SSH 用户名
- `SFID_DEPLOY_SSH_KEY`：该用户私钥

可选 Secrets：
- `SFID_DEPLOY_KNOWN_HOSTS`：已固定的服务器 host key；不配时 workflow 会执行 `ssh-keyscan`

可选 GitHub Variables（不配时使用默认值）：
- `SFID_DEPLOY_HOST`：默认 `147.224.14.117`
- `SFID_DEPLOY_DOMAIN`：默认 `sfid.crcfrcn.com`
- `SFID_DEPLOY_PORT`：默认 `22`
- `SFID_DEPLOY_APP_HOME`：默认 `/opt/sfid`
- `SFID_DEPLOY_FRONTEND_ROOT`：默认 `/var/www/sfid`
- `SFID_DEPLOY_SERVICE`：默认 `sfid-backend`
- `SFID_DEPLOY_ENV_FILE`：默认 `/etc/sfid/sfid.env`
- `SFID_DEPLOY_WEB_SERVICE`：默认 `nginx`
- `SFID_DEPLOY_HEALTHCHECK_URL`：默认 `http://127.0.0.1:8899/api/v1/health`
- `SFID_FRONTEND_API_BASE_URL`：默认 `/api`

### 自动部署行为
1. GitHub Actions 构建 `sfid-backend`
2. 构建 `frontend/dist`
3. 上传发布包到服务器临时目录
4. 执行 `update_sfid_app.sh`
5. 同步后端二进制、前端静态资源、迁移脚本
6. 自动执行“未应用”的数据库迁移
7. 重启 `sfid-backend`
8. 本地健康检查通过后结束

### 迁移策略
自动部署使用 `apply_sfid_migrations.sh`：
- 首次执行时会创建 `schema_migrations`
- 后续只执行未执行过的 SQL
- 若同名迁移文件内容被改动，会直接失败，避免静默污染数据库
