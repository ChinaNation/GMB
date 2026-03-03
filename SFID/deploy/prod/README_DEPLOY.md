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
   - 必填：`SFID_SIGNING_SEED_HEX`、`SFID_KEY_ID`、`SFID_CHAIN_TOKEN`、`SFID_CHAIN_SIGNING_SECRET`、`SFID_PUBLIC_SEARCH_TOKEN`、`SFID_PII_KEY`。
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
