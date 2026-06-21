# CID 生产部署（主库 + 备库）

## 目标拓扑
- CID 应用云服务器：运行 `citizencode-backend`。
- PostgreSQL 主库云服务器：所有写入都进入主库。
- PostgreSQL 备库云服务器：由主库流复制同步。
- 主库定时备份文件同步到备库（额外灾备层）。

## 脚本说明
- `scripts/install_postgres_primary.sh`：主库初始化与复制账号配置。
- `scripts/install_postgres_standby.sh`：备库初始化（`pg_basebackup` + 流复制）。
- `scripts/install_citizencode_app.sh`：应用安装与 systemd 服务安装。
- `scripts/backup_to_standby.sh`：主库导出并同步到备库。
- `systemd/citizencode-backend.service`：CID 后端服务。
- `systemd/cid-backup.service` + `systemd/cid-backup.timer`：定时备份任务。

## 执行顺序
1. 在主库执行：`install_postgres_primary.sh`。
2. 在备库执行：`install_postgres_standby.sh`。
3. 在应用服务器执行：`install_citizencode_app.sh /opt/citizencode /path/to/citizencode-backend`。
   该脚本会同步 `/opt/citizencode/china/china.sqlite`,再执行
   `citizencode-backend reconcile-gov --changed-only` 和 `citizencode-backend check-gov --strict`;
   任一命令失败都不得启动新版服务。
4. 在应用服务器修改 `/etc/citizencode/citizencode.env`：
   - `DATABASE_URL` 指向主库（建议 `sslmode=verify-full`）。
   - 基础站点必填：`CID_SIGNING_SEED_HEX`、`CID_KEY_ID`、`CID_REDIS_URL`、`CID_CHINA_DB`
   - 链交互启用前必填：`CID_CHAIN_TOKEN`、`CID_CHAIN_SIGNING_SECRET`、`CID_CHAIN_WS_URL`、`CID_CHAIN_GENESIS_HASH`、`CID_RUNTIME_ISSUER_CID_NUMBER`、`CID_RUNTIME_ISSUER_MAIN_ACCOUNT`、`CID_RUNTIME_SIGNER_PUBKEY`
   - 可后补的扩展项：`CID_PUBLIC_SEARCH_TOKEN`、`CID_PII_KEY`
5. 启动服务：
   - `systemctl daemon-reload`
   - `systemctl enable --now citizencode-backend`
6. 健康检查：
   - `curl http://127.0.0.1:8899/api/v1/health`

## 备份定时任务
1. 将 `scripts/backup_to_standby.sh` 安装到 `/opt/citizencode/scripts/backup_to_standby.sh`。
2. 安装并启用 timer：
   - `systemctl enable --now cid-backup.timer`
3. 查看下次执行时间：
   - `systemctl list-timers | grep cid-backup`

## 注意事项
- 生产必须使用强密码并限制安全组/IP 白名单。
- `/etc/citizencode/citizencode.env` 禁止保留 `CHANGE_ME` 占位值；安装脚本会拒绝继续执行。
- `citizencode-backend.service` 已按非 root 用户 `cid` 运行并启用 systemd 沙箱加固。
- 应用只连接主库，不要写备库。
- 备库用于容灾与只读核验，故障切换需要明确 SOP。
- 若 Redis 与应用部署在同一台机器，推荐 `CID_REDIS_URL=redis://127.0.0.1:6379/0`。
- `CID_CHINA_DB` 是行政区随包只读 SQLite，正式部署固定 `/opt/citizencode/china/china.sqlite`。
  行政区变更只能来自开发库 `citizencode/backend/china/china.sqlite` 后重新发布安装包。
- 公权机构目录由行政区和模板确定性派生。安装新版行政区后必须先让运行库中的
  `gov.source='GENERATED'` 目录对账到当前 `china.sqlite`,并确认全局 `gov_manifest`
  为 `OK`;手动公权机构 `MANUAL` 不属于自动清理范围。
- CID 是由联邦注册局运维的中心化独立系统,链配置只影响投票凭证、人口快照凭证、机构链注册凭证等链交互接口,不得阻断基础站点启动、登录和机构管理。
- 如果当前目标只是“先把网页、登录和基础后台跑起来”，可以先不填写链对接参数；这些只影响链相关接口，不影响站点基础启动。

## 自动部署（GitHub Push -> 云服务器自动更新）
适用场景：你推送 `citizencode/**` 代码到 GitHub `main` 后，自动把服务器更新到最新版本。

### 一次性服务器准备
1. 先按上面的“执行顺序”完成首装，确保下面两项已经存在：
   - `/etc/citizencode/citizencode.env`
   - `systemd` 服务 `citizencode-backend`
2. 确保部署账号具备免密 `sudo` 能力，至少允许：
   - 写 `/opt/citizencode`
   - 写前端静态目录（默认 `/var/www/cid`）
   - `systemctl restart citizencode-backend`
   - `systemctl reload nginx`
3. 若要一起自动发布前端，建议让站点静态根目录指向：
   - `/var/www/citizencode/current`
4. 推荐反向代理规则：
   - 站点根目录：`/var/www/citizencode/current`
   - `/api/` 反代到：`http://127.0.0.1:8899`

参考 Nginx 片段：
```nginx
server {
    server_name cid.crcfrcn.com;

    root /var/www/citizencode/current;
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
- `.github/workflows/cid-deploy.yml`

必须配置的 GitHub Secrets：
- `CID_DEPLOY_USER`：部署服务器 SSH 用户名
- `CID_DEPLOY_SSH_KEY`：该用户私钥

可选 Secrets：
- `CID_DEPLOY_KNOWN_HOSTS`：已固定的服务器 host key；不配时 workflow 会执行 `ssh-keyscan`

可选 GitHub Variables（不配时使用默认值）：
- `CID_DEPLOY_HOST`：默认 `147.224.14.117`
- `CID_DEPLOY_DOMAIN`：默认 `cid.crcfrcn.com`
- `CID_DEPLOY_PORT`：默认 `22`
- `CID_DEPLOY_APP_HOME`：默认 `/opt/citizencode`
- `CID_DEPLOY_FRONTEND_ROOT`：默认 `/var/www/cid`
- `CID_DEPLOY_SERVICE`：默认 `citizencode-backend`
- `CID_DEPLOY_ENV_FILE`：默认 `/etc/citizencode/citizencode.env`
- `CID_DEPLOY_WEB_SERVICE`：默认 `nginx`
- `CID_DEPLOY_HEALTHCHECK_URL`：默认 `http://127.0.0.1:8899/api/v1/health`
- `CID_FRONTEND_API_BASE_URL`：默认 `/api`

### Passkey 生产域名约束

生产 `/etc/citizencode/citizencode.env` 必须固定:

```bash
CID_ENV=production
CID_PASSKEY_RP_ID=cid.crcfrcn.com
CID_PASSKEY_ORIGIN=https://cid.crcfrcn.com
```

后端启动期会校验生产环境 Passkey 域名,不得把 `localhost`、`127.0.0.1`、
局域网 IP 或其它 origin 混入生产配置。

### 自动部署行为
1. GitHub Actions 构建 `citizencode-backend`
2. 构建 `frontend/dist`
3. 上传发布包到服务器临时目录
4. 执行 `update_citizencode_app.sh`
5. 同步后端二进制、行政区 SQLite 和前端静态资源
6. 对账 CID 运行库中的确定性公权机构,并执行 `check-gov --strict`
7. 重启 `citizencode-backend`
8. 本地健康检查通过后结束

### 数据库结构
CID 后端启动时直接创建当前目标结构；发布包不携带独立 SQL 脚本。
