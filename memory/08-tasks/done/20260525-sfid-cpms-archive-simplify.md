# CPMS/SFID 档案码简化与公民列表绑定热区修复

- 状态：done
- 模块：cpms / sfid / memory
- 创建日期：2026-05-25

## 任务需求

按最终简化协议改造 CPMS 与 SFID 的 `SFID_CPMS_V1 / ARCHIVE` 档案码：

- 档案码字段只保留 `proto / type / ano / cs / valid_from / valid_until / cpms_pubkey / geo_seal / sig`。
- 删除档案码中的 `status_updated_at` 字段。
- 不新增 `code_id`。
- 不新增 `usage_limit`。
- 不新增“已消费档案码”记录。
- SFID 侧继续按“钱包地址先存在，扫描档案码后绑定 `ano + sfid_code + wallet_pubkey` 三者一对一”的目标模型处理。

同时修复 SFID 前端公民信息列表 bug：

- 点击列表中的“绑定”按钮时，不应同时弹出公民详细信息弹窗和扫码绑定弹窗。
- 公民信息行的点击热区不应覆盖操作栏和绑定按钮。

## 影响范围

- `cpms/backend/src/dangan/`：调整 ARCHIVE 载荷结构与签名原文。
- `cpms/backend/src/operator_admin/`：清理生成、打印审计中对协议字段 `status_updated_at` 的输出。
- `sfid/backend/cpms/`：调整 ARCHIVE DTO、验签原文和字段校验。
- `sfid/backend/citizens/`：清理旧状态时间字段残留，确认绑定前已有钱包地址。
- `sfid/frontend/citizens/`：修复列表行点击热区与绑定按钮事件冒泡。
- `memory/05-modules/cpms/`：更新 CPMS 档案码文档。
- `memory/05-modules/sfid/`：更新 SFID_CPMS_V1 协议文档。
- `memory/07-ai/`：补充或更新统一协议登记。

## 主要风险点

- CPMS 与 SFID 的签名原文必须完全一致，否则所有新档案码都会验签失败。
- 删除 `status_updated_at` 后，不能残留任何要求 SFID 解析该字段的逻辑。
- 不引入一次性码机制，重复绑定必须继续依赖 SFID 的 `ano / wallet_pubkey / sfid_code` 唯一关系。
- 前端修复必须只收窄行点击热区，不破坏绑定按钮、详情查看和其他操作按钮。

## 验收标准

- CPMS 生成的 ARCHIVE JSON 不包含 `status_updated_at / code_id / usage_limit`。
- CPMS 与 SFID 使用同一份新签名原文。
- SFID 能验签并绑定新档案码。
- SFID 后端不再保存或返回旧状态时间字段。
- 公民列表点击绑定按钮只打开绑定/扫码弹窗，不打开详情弹窗。
- 文档同步更新，旧字段残留清理完成。
- 相关 Rust / 前端检查尽量通过；如环境限制导致无法执行，必须说明原因。

## 完成信息

- 完成时间：2026-05-24 18:34:22
- 完成摘要：完成 CPMS/SFID ARCHIVE 档案码简化、SFID 三者绑定口径收敛、公民列表操作栏热区修复、文档更新和残留清理。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
