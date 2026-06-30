任务需求：
收敛 citizenapp 访问 CID 系统的地址路径，只保留生产域名与本地 USB 开发两条路径，删除局域网 IP、自定义 API 地址、旧 8787 默认值等多余方式。

状态：
已执行

所属模块：
citizenapp CID API 连接边界

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/unified-required-reading.md
- memory/07-ai/agent-rules.md
- memory/07-ai/chat-protocol.md
- memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md

必须遵守：
- 生产路径只能访问 `https://cid.crcfrcn.com`。
- 开发路径只能使用 USB 调试：App 内部访问 `http://127.0.0.1:8899`，由 `adb reverse tcp:8899 tcp:8899` 转发到本机 CID 后端。
- 不允许保留局域网 IP、`CID_PUBLIC_BASE_URL`、`ONCHINA_BIND_ADDR` 拼客户端地址、`CITIZENAPP_API_BASE_URL` 任意注入、`ONCHINA_BASE_URL` 任意注入、`127.0.0.1:8787` 旧默认值。
- 生产失败不得回退开发，开发失败不得回退生产。
- 改代码后同步更新文档、补必要中文注释并清理残留。

输出物：
- citizenapp CID API 地址策略收敛。
- citizenapp 本地启动脚本固定为 USB reverse 开发路径。
- 本地 CID 开发环境删除手机局域网访问地址残留。
- citizenapp 技术文档同步更新。
- 残留路径搜索与测试验证记录。

验收标准：
- 代码中 citizenapp 访问 CID 只剩 `prod` 与 `dev_usb` 两种环境。
- `prod` 固定映射到 `https://cid.crcfrcn.com`。
- `dev_usb` 固定映射到 `http://127.0.0.1:8899`，脚本必须建立 `adb reverse tcp:8899 tcp:8899`。
- 旧的 `CITIZENAPP_API_BASE_URL`、`ONCHINA_BASE_URL`、`CID_PUBLIC_BASE_URL` 客户端路径不再存在。
- 相关测试、格式化或静态检查通过；若无法运行需说明原因。

执行记录：
- 已新增 `citizenapp/lib/cid_api_config.dart`，统一 citizenapp 访问 CID 的地址策略，只允许 `prod` 与 `dev_usb`。
- 已将电子护照、钱包能力、链下扫码支付、清算行设置页的 CID 地址来源统一改为 `CidApiConfig.defaultBaseUrl`。
- 已删除 citizenapp 客户端旧的 `CITIZENAPP_API_BASE_URL`、`ONCHINA_BASE_URL`、`127.0.0.1:8787` 默认路径。
- 已将 `citizenapp/scripts/citizenapp-run.sh` 固定为 USB 开发路径：注入 `CITIZENAPP_ONCHINA_ENV=dev_usb`，并强制建立 `adb reverse tcp:8899 tcp:8899`。
- 已将 `citizencode/.env.dev.local` 的本地监听改回 `127.0.0.1:8899`，删除手机局域网访问地址残留。
- 已同步更新 citizenapp 架构文档、钱包模块文档、链下交易相关文档。
- 已新增 `citizenapp/test/cid_api_config_test.dart` 覆盖两路径白名单。
- 验证：`flutter test test/cid_api_config_test.dart` 通过；`flutter analyze` 通过；`bash -n scripts/citizenapp-run.sh` 通过；旧客户端地址变量残留搜索通过。
