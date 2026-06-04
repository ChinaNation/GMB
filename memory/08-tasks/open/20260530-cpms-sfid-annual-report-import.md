# CPMS 年度报告导出与 SFID 导入收口

## 任务需求

CPMS 年度报告以 CPMS 为公民档案号、钱包地址、公民状态、投票状态的真源；SFID 以身份 ID 为真源。CPMS 导出年度报告后，SFID 导入时按档案号覆盖本地绑定状态，但不得突破身份 ID 唯一生成边界。

## 预计修改目录

- `cpms/backend/src/dangan`：调整年度报告导出 DTO、哈希内容、导出查询与中文注释，只导出已绑定钱包的公民绑定记录和硬删除释放记录。
- `cpms/backend/db`：收口年度报告计数字段命名，不保留旧导出字段作为当前基准。
- `cpms/frontend/super_admin`：同步年度报告前端类型字段。
- `sfid/backend/citizens`：新增年度报告导入 handler，覆盖同档案号绑定状态并处理硬删除释放。
- `sfid/backend/store`：在所属 Store 模型中增加导入幂等记录持久化字段。
- `sfid/backend/main.rs`：挂载 SFID 导入年度报告 API，移除旧 CPMS 状态扫码路由残留。
- `sfid/frontend/citizens`：在公民身份列表增加导入年度报告按钮和导入弹窗，清理旧状态扫码残留。
- `memory/05-modules/cpms`：更新 CPMS 年度报告导出协议和边界文档。
- `memory/05-modules/sfid`：更新 SFID 年度报告导入协议、前后端入口和字段语义文档。

## 验收清单

- [x] CPMS 导出字段统一为 `citizen_binding_records` 与 `binding_release_records`。
- [x] CPMS 导出只包含有钱包绑定的档案；硬删除释放记录不包含护照号。
- [x] SFID 导入校验 CPMS 授权、公钥绑定、签名、哈希和字段约束。
- [x] SFID 导入按档案号覆盖钱包、公民状态、投票状态，保留原身份 ID。
- [x] SFID 导入硬删除释放记录时释放档案号、钱包地址、身份 ID 绑定索引。
- [x] SFID 前端首页公民身份列表增加导入年度报告按钮。
- [x] 清理旧 CPMS 状态扫码入口、`ABNORMAL/异常` 残留和文档旧字段。
- [x] 运行后端测试、前端构建和残留扫描。
