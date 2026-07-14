# CitizenChain CI 手动发布与桌面端点击更新

## 任务需求

- `citizenchain.yml` 支持本地推送代码触发构建检查，但推送触发不得发布 GitHub Release，也不得部署服务器。
- 只有正式 Release 操作才发布桌面端 GitHub Release；历史“同一 workflow 部署6台服务器”已经废弃，服务器部署现由本机 `deploy/` 控制台逐节点执行。
- 区块链软件打开后检查 GitHub Release 更新；有新版本时，在设置 tab 的“节点程序版本”数字前显示“更新”按钮。
- 用户点击“更新”后，才停止当前节点、安装更新并重启区块链软件。

## 修改边界

- CI 边界：workflow 只负责构建检查与正式发布产物；服务器部署不得恢复到 workflow。
- 桌面端边界：仅改 `citizenchain/node` 的 Tauri 更新插件、设置页更新按钮和节点停止准备命令。
- 文档边界：同步更新 CitizenChain node 构建发布文档和任务索引。

## 验收标准

- push 触发 workflow 时只执行桌面端打包构建，不创建 Release，不部署服务器。
- 正式 Release 时创建 GitHub Release、上传安装包和 updater 元数据；服务器部署由独立本机控制台处理。
- App 启动后只检查更新，不自动安装。
- 设置 tab 存在新版本时，在“节点程序版本”版本号前显示“更新”按钮。
- 点击“更新”才调用 Tauri updater 下载/安装，并重启软件。
- 改代码后文档和残留同步清理。
