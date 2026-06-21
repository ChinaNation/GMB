# CitizenPassport 安装配置手册

## 1. 适用范围

本手册适用于 CPMS 市公安局离线主机安装包。

支持的正式安装包：

- `citizenpassport-ubuntu24-amd64.run`：Ubuntu 24.04 amd64 主机。
- `citizenpassport-ubuntu24-arm64.run`：Ubuntu 24.04 arm64 主机。

CPMS 是局域网 Web 系统，不是桌面 App。安装完成后，主机通过 systemd 自动运行后端、PostgreSQL 和 nginx，工作人员使用浏览器访问：

```text
https://www.citizenpassport.com/login
```

## 2. 安装前准备

主机要求：

- Ubuntu Server 24.04 LTS。
- CPU 架构必须与安装包一致。
- 安装时使用具有 `sudo` 权限的本机账号。
- 安装包已经复制到主机本地目录。
- CPMS 是完全离线系统，安装过程不依赖互联网。

确认主机架构：

```bash
dpkg --print-architecture
```

输出 `amd64` 使用 `citizenpassport-ubuntu24-amd64.run`；输出 `arm64` 使用 `citizenpassport-ubuntu24-arm64.run`。

## 3. 主机安装

进入安装包所在目录后执行：

```bash
chmod +x citizenpassport-ubuntu24-amd64.run
sudo ./citizenpassport-ubuntu24-amd64.run
```

arm64 主机使用：

```bash
chmod +x citizenpassport-ubuntu24-arm64.run
sudo ./citizenpassport-ubuntu24-arm64.run
```

安装完成后终端会显示：

```text
CitizenPassport host install complete.
Login page: https://www.citizenpassport.com/login
Root CA certificate: /etc/citizenpassport/certs/citizenpassport-root-ca.crt
Install guide: /opt/citizenpassport/docs/CitizenPassport安装配置手册.md
Service status: systemctl status citizenpassport-backend
Nginx status: systemctl status nginx
Backup config: /etc/citizenpassport/backup.env
```

## 4. 健康检查

检查后端服务：

```bash
systemctl status citizenpassport-backend --no-pager
```

检查 nginx：

```bash
systemctl status nginx --no-pager
```

检查 HTTPS 入口：

```bash
curl -k https://www.citizenpassport.com/api/v1/health
```

正常返回：

```json
{"code":0,"message":"ok","data":{"status":"ok"}}
```

如果需要在 CPMS 主机本机临时测试域名解析，可以执行：

```bash
sudo sh -c 'echo "127.0.0.1 www.citizenpassport.com" >> /etc/hosts'
```

局域网客户端不能把 `www.citizenpassport.com` 配成 `127.0.0.1`，必须解析到 CPMS 主机的局域网 IP。

## 5. 局域网 DNS 配置

CPMS 正式入口固定为：

```text
https://www.citizenpassport.com/login
```

公安局内网 DNS 需要把：

```text
www.citizenpassport.com
```

解析到 CPMS 主机的局域网 IP。

示例：

```text
www.citizenpassport.com -> 192.168.1.20
```

如果没有内网 DNS，可在每台客户端的 hosts 文件中配置同样映射。不要使用 IP、`localhost` 或 `127.0.0.1` 作为正式访问地址，否则会出现证书域名不匹配。

## 6. 根证书信任

安装时会生成本机私有 Root CA：

```text
/etc/citizenpassport/certs/citizenpassport-root-ca.crt
```

浏览器提示“连接不安全”通常是因为客户端尚未信任该 Root CA。

CPMS 主机会在安装时自动把该 Root CA 加入本机系统信任库；局域网客户端电脑仍需要按下面步骤手动导入。

Linux 客户端导入：

```bash
sudo cp citizenpassport-root-ca.crt /usr/local/share/ca-certificates/citizenpassport-root-ca.crt
sudo update-ca-certificates
```

Windows 客户端导入：

1. 双击 `citizenpassport-root-ca.crt`。
2. 选择“安装证书”。
3. 选择“本地计算机”。
4. 选择“将所有的证书都放入下列存储”。
5. 选择“受信任的根证书颁发机构”。
6. 完成导入后重启浏览器。

macOS 客户端导入：

1. 打开“钥匙串访问”。
2. 将 `citizenpassport-root-ca.crt` 导入“系统”钥匙串。
3. 打开该证书，设置“使用此证书时：始终信任”。
4. 重启浏览器。

Firefox 如果使用独立证书库，需要在 Firefox 设置中导入：

```text
设置 -> 隐私与安全 -> 证书 -> 查看证书 -> 证书颁发机构 -> 导入
```

## 7. 常见问题

### 7.1 502 Bad Gateway

502 表示 nginx 正常运行，但后端 `citizenpassport-backend` 没有正常监听 `127.0.0.1:8080`。

检查：

```bash
systemctl status citizenpassport-backend --no-pager -l
sudo journalctl -u citizenpassport-backend -n 120 --no-pager -l
```

如果日志出现 `permission denied for table system_install`，说明安装包版本存在数据库对象归属问题。新版安装包已经取消正式安装导入 `schema.sql`，数据库结构只由后端 migration 创建，并会修正旧安装残留对象的 owner。

### 7.2 浏览器提示连接不安全

通常是客户端未导入 Root CA，或没有通过 `https://www.citizenpassport.com/login` 访问。

处理：

- 确认访问地址是 `https://www.citizenpassport.com/login`。
- 确认客户端 DNS 指向 CPMS 主机局域网 IP。
- 确认客户端已经信任 `/etc/citizenpassport/certs/citizenpassport-root-ca.crt`。

### 7.3 域名无法访问

检查客户端 DNS 或 hosts：

```bash
ping www.citizenpassport.com
```

返回 IP 必须是 CPMS 主机的局域网 IP。

### 7.4 端口被占用

CPMS nginx 使用 80 和 443，后端使用本机回环 `127.0.0.1:8080`。

检查：

```bash
ss -lntp | grep -E ':80|:443|:8080'
```

## 8. 备份

备份配置文件：

```text
/etc/citizenpassport/backup.env
```

配置完成后启用定时备份：

```bash
sudo systemctl enable --now citizenpassport-backup.timer
```

备份脚本会导出 PostgreSQL dump，并同步 `/var/lib/citizenpassport/runtime` 与 `/var/lib/citizenpassport/materials`。

## 9. 重要路径

```text
/opt/citizenpassport/bin/citizenpassport-backend
/opt/citizenpassport/frontend
/opt/citizenpassport/docs/CitizenPassport安装配置手册.md
/etc/citizenpassport/citizenpassport-backend.env
/etc/citizenpassport/certs/citizenpassport-root-ca.crt
/var/lib/citizenpassport/materials
/var/backups/citizenpassport
```

不要手工删除 `/etc/citizenpassport`、`/var/lib/citizenpassport` 或 PostgreSQL 数据库，避免丢失已录入档案数据。
