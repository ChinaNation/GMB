# 电子护照三端绑定闭环与旧流程清理

## 状态

已完成

## 任务需求

彻底修复 CPMS、SFID、wuminapp 三端电子护照绑定闭环。绑定必须由 wuminapp 对 SFID 绑定签名请求签名，SFID 验签后直接完成电子护照绑定并向 wuminapp 状态查询返回结果；绑定公民电子护照流程到 SFID 验签落库即结束。

## 强制边界

- 绑定公民电子护照必须使用 wuminapp 签名。
- SFID 完成本地绑定后直接返回状态。
- 相同概念三端字段必须保持相同命名。
- 删除旧主动注册、旧绑定模式、旧字段名和旧文档口径。
- CPMS 永不联网，ARCHIVE 不包含实名原文。
- SFID 不保存原始实名。

## 影响范围

- `cpms/backend/src/dangan/`：ARCHIVE payload 与签名原文。
- `cpms/backend/src/operator_admin/`：档案码生成、更新、打印和投票账户保存。
- `cpms/frontend/operator_admin/`：投票账户与档案码 UI 文案。
- `sfid/backend/cpms/`：ARCHIVE 验真与字段解析。
- `sfid/backend/citizens/`：电子护照绑定 challenge、验签、状态查询。
- `sfid/frontend/citizens/`：扫描档案码、签名请求、签名回执、绑定完成 UI。
- `wuminapp/lib/my/myid/`：电子护照钱包选择、扫码签名、状态查询。
- `memory/`：协议、架构、模块文档和任务记录。

## 验收标准

- SFID 后端编译通过。
- CPMS 后端编译通过。
- SFID 前端构建通过。
- CPMS 前端构建通过。
- wuminapp 电子护照相关测试通过。
- 全仓电子护照绑定链路只描述 SFID 验签落库与 wuminapp 状态查询。
- 全仓电子护照绑定链路不再使用历史主动注册、历史双模式字段或历史注册路由。
- CPMS、SFID、wuminapp 使用统一字段：`archive_no`、`citizen_status`、`voting_eligible`、`vote_status`、`identity_status`、`valid_from`、`valid_until`、`status_updated_at`、`wallet_address`、`wallet_pubkey`、`wallet_sig_alg`、`sfid_code`、`bind_status`。

## 执行记录

- 已完成 CPMS `ARCHIVE` 载荷字段统一：`archive_no / citizen_status / voting_eligible / valid_from / valid_until / status_updated_at / wallet_address / wallet_pubkey / wallet_sig_alg`，并更新 CPMS 签名原文。
- 已完成 SFID `ARCHIVE` 验真解析、绑定 challenge、wuminapp 签名验签、本地绑定落库和 `/api/v1/app/myid/status` 查询。
- 已移除 SFID 电子护照绑定旧路由和历史修改入口。
- 已完成 SFID 前端 `citizens/api.ts / BindModal.tsx / CitizensView.tsx` 单一绑定流程改造。
- 已完成 wuminapp `myid` 状态查询字段、扫码签名入口和本地状态字段改造。
- 已按待绑定态 UI 要求调整 wuminapp：选择钱包但未完成绑定时左侧显示“更换钱包”，右侧显示“扫码签名”，两个按钮横向并排。
- 已按扫码页要求调整 wuminapp：页面标题为“扫码签名”，提示文案为“请扫描身份ID系统的鉴权签名码”，删除旧管理端提示文案并上移识别框与钱包地址。
- 已收紧 SFID 公民列表、公开查询和状态扫码：只展示/返回完整 `BOUND` 电子护照记录，历史半绑定记录不再暴露给后台或查询接口。
- 已清理 SFID 旧绑定回调 worker、回调环境变量文档和死字段；电子护照绑定结果只由 SFID 本地完成并由 wuminapp 状态接口查询。
- 已新增 SFID 启动清理：启动时物理删除历史非 `BOUND` 公民记录及其反向索引，并清除旧待绑定扫码缓存、旧按钱包生成 SFID 缓存字段。
- 已锁死更换绑定边界：更换绑定只允许更换钱包账户；`archive_no` 与 `sfid_code` 首次绑定后永久不变，扫描其他档案号会被 SFID 拒绝。
- 已修正 SFID 公民列表绑定入口文案：无记录入口固定为“新增身份ID绑定”，已有记录操作固定为“更换绑定”，弹窗标题和 wuminapp 签名请求摘要按 `create / replace` 区分。
- 已修复 `geo_seal cannot be decrypted` 持久化链路：SFID 启动时把 `store_cpms.cpms_site_keys` 恢复到 `sharded_store`；首次 ARCHIVE 验真绑定 `cpms_pubkey_hash / ACTIVE / USED` 时先写 `store_cpms`，再同步运行缓存。
- 已把 SFID 公民列表地址列改为“投票账户”，把列表状态改为由 `citizen_status + voting_eligible` 计算的“投票状态 正常/异常”；公民列表和详情响应不再下发签发地市归属，避免暴露签发地市。
- 已把更换绑定弹窗当前记录标签改为“档案号 / 身份ID / 投票账户”，并把签名请求展示字段改为“选举权利 / 公民状态 / 投票账户”。
- 已补齐 wuminapp 状态接口解析和本地缓存字段：`citizen_status / voting_eligible / vote_status`。
- 已更新协议、命名、SFID/CPMS/wuminapp 模块文档，并删除旧实现计划文档。
- 已验证：
  - `cargo test --manifest-path sfid/backend/Cargo.toml`
  - `cargo check --manifest-path cpms/backend/Cargo.toml`
  - `npm run build`（`sfid/frontend`）
  - `npm run build`（`cpms/frontend`）
  - `flutter test test/myid_page_test.dart test/sfid_api_config_test.dart`
