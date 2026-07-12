# CitizenApp 链数据统一与 Cloudflare 残留清理

状态：已完成（2026-07-12）

## 任务需求

- CitizenApp 的公民、机构、管理员和清算行身份统一使用链数据。
- 公民身份、机构、清算行统一使用链数据；Cloudflare 和设备缓存只能保存链数据派生结果，不能成为真源。
- 保留 `chain.crcfrcn.com`、`www.crcfrcn.com/api/*` 和 `www.crcfrcn.com/api-staging/*`。
- 删除旧身份 DNS、残留 Cloudflare Access 应用和旧服务令牌，最终严格保留八条 DNS。
- 完成后更新文档、完善中文注释，并彻底清理旧流程、旧配置、旧测试和旧文案。

## 所属模块

- `citizenapp`
- Cloudflare 控制面
- `memory`

## 输入文档

- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/repo-map.md`
- `memory/03-security/security-rules.md`
- `memory/07-ai/unified-naming.md`
- `memory/07-ai/unified-protocols.md`
- `memory/07-ai/definition-of-done.md`
- `memory/07-ai/module-definition-of-done/citizenapp.md`
- CitizenApp 身份、机构、钱包、清算行和 RPC 技术文档

## 目标状态

- 公民身份直接读取 `VotingIdentityByAccount` 和 `CandidateIdentityByAccount`。
- 机构目录使用 finalized 链快照、链状态更新和 Isar 端侧索引，不再调用 CID HTTP 接口。
- 清算行使用 `ClearingBankNodes`、`UserBank` 和链规则确定的机构主账户。
- CitizenApp 只保留链读取、链快照和设备本地派生索引。
- Cloudflare DNS 严格保留 `www`、`chain` 和六条节点记录。
- Cloudflare Access 只保留 staging API 与链入口，服务令牌只保留 `CitizenChain`。

## 必须遵守

- 不修改 `citizenchain/runtime/`；如确需修改，停止执行并申请单独二次确认。
- 不保留兼容入口、双轨读取、旧缓存转换或影子旧流程。
- 不修改 OnChina 模块边界，不恢复 OnChina 后端源码壳。
- 不执行 Git 推送或触发远端 CI。
- 所有关键链读取和缓存重建逻辑补充中文注释。

## 输出物

- CitizenApp 链数据读取代码
- 删除 CID/SFID 残留后的配置、脚本和测试
- Cloudflare 控制面清理结果
- 更新后的架构和模块技术文档
- 单元测试、静态检查和真实运行态验收记录

## 验收标准

- 真机或真实 Flutter 运行态可读取公民、机构和清算行链数据。
- production/staging 的 `www` API 可用，`chain` Tunnel 和 Access 链路可用。
- Cloudflare DNS 总数严格为八条。
- 全仓搜索不存在 CitizenApp 旧身份域名、旧客户端、旧配置、旧注释或旧文档口径。
- 测试通过、文档已更新、中文注释完善、残留已彻底清理。

## 执行记录

- [x] 用户确认技术方案和任务卡文件创建。
- [x] 完成 CitizenApp 链数据统一。
- [x] 删除 CID/SFID 代码、配置、测试和文档残留。
- [x] 完成 Cloudflare 控制面清理。
- [x] 完成真实运行态验收和全仓残留复查。

## 完成记录

- CitizenApp 机构目录、管理员标签和清算行目录均改为 finalized 链读取；公权机构安装包由真实本地节点 `127.0.0.1:9944` 生成，共 43 省、49,593 条机构。
- 删除 CID/SFID 域名、HTTP 客户端、同步服务、启动参数、旧测试、旧收款页说明和相关任务残留；精确全仓搜索为零。
- Cloudflare DNS 为严格 8 条；Access 只保留 `CitizenApp API Staging` 与 `chain`；Service Token 只保留 `CitizenChain`。
- App、CitizenChain plain chainspec 与 Worker default/staging/production 统一只保留 6 个已部署 bootnode。staging 部署版本 `692d472a-49ec-47e5-912d-51cf6e178545`，production 部署版本 `418f3d65-ea13-4d40-a045-a66ba84822cc`。
- 真机 `ONEPLUS A6013` 清除旧 App 数据后完成 debug APK 编译、安装和冷启动，首次权限页正常显示。production bootstrap 返回 200 和 6 个目标 bootnode，staging 未登录返回 302，两个旧身份域名均为 NXDOMAIN。
- 验证结果：Flutter 全量测试 511 passed / 5 skipped / 0 failed；Worker 124 passed；Worker TypeScript 类型检查通过；`flutter analyze` 无错误，仅剩一条与本任务无关的 `prefer_const_constructors` info。
