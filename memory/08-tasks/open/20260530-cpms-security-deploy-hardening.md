# CPMS 安全与部署加固

## 任务需求

- CPMS 机构管理员 15 分钟无活动强制退出；operators 30 分钟无活动强制退出。
- 正式安装包必须包含前端构建产物，安装后通过后端 8080 直接访问页面。
- 删除 Docker 残留；保留 CPMS 编译期引用 CID 工具行政区数据唯一真源。
- 删除档案签名失败的所有校验分支都写失败审计，且不消费 challenge、不删除档案。
- 增加登录、初始化、删除签名、资料上传等高风险入口的本机限流。
- 启动时直接验证 `CPMS_KEY_ENCRYPT_SECRET` 能解密现有 `install_secret` 与 ARCHIVE 私钥。
- 增加后端统一安全响应头。
- 删除登录旧 alias 和 `challenge_payload` 残留。
- 开发脚本不改多电脑访问；部署后保证多电脑通过主机 8080 访问。

## 预计修改目录

- `citizenpassport/backend/login`：调整管理员会话空闲过期时间，删除旧登录字段残留，接入限流。
- `citizenpassport/backend/authz`：所有管理员统一执行过期检查和滑动续期。
- `citizenpassport/backend/rate_limit.rs`：新增本机内存限流模块。
- `citizenpassport/backend/main.rs`：挂载限流器、安全响应头和前端目录校验。
- `citizenpassport/backend/initialize`：启动时解密验证本机密钥材料，并给初始化入口接入限流。
- `citizenpassport/backend/dangan`：删除签名失败全分支写失败审计，删除完成和资料上传接入限流。
- `citizenpassport/backend/db`：删除无用 `sign_requests.challenge_payload` 字段。
- `citizenpassport/scripts`：正式打包时构建并复制前端 `dist`。
- `citizenpassport/deploy/linux`：安装 `/opt/citizenpassport/frontend` 并配置 `CPMS_FRONTEND_DIR`。
- `citizenpassport/deploy` 与 `citizenpassport/backend`：删除 Docker 残留文件。
- `citizenpassport/CITIZENPASSPORT_TECHNICAL.md` 与 `memory/05-modules/citizenpassport`：同步安全、部署、会话、限流、密钥说明。

## 执行清单

- [x] 创建任务卡。
- [x] 实现会话时长和滑动过期。
- [x] 实现本机限流和安全响应头。
- [x] 实现启动密钥解密验证。
- [x] 修复删除签名失败审计。
- [x] 删除登录旧字段残留。
- [x] 修复正式安装前端部署。
- [x] 删除 Docker 残留。
- [x] 更新文档和错误码。
- [x] 运行完整验证。
