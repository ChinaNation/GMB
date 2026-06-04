任务需求：
SFID 机构架构按精简命名拆分，前后端统一使用 `gov` 表示公权机构，保留现有 `admins` 权限模块，不新增 `registry_admins`。公民继续使用 `citizens` 命名，智能人当前不上线，不建设智能人表。数据库从第一版目标结构开始按省分区设计，不再写“数据大了以后再分区”。

所属模块：
SFID

必须遵守：
- 机构唯一身份只认 `sfid_number`，不得新增 `identity_key`、`generation_key` 或第二身份键。
- 表名和目录名保持精简：`subjects / citizens / gov / private / accounts / docs / admins`。
- 前后端命名统一：公权机构目录均使用 `gov`。
- 省市管理员权限继续由现有 `admins` 与 `scope` 承接，不新增 `registry_admins`。
- 智能人功能当前不上线，不创建智能人模块或数据表。
- 不恢复 `backend/src/`、独立 `backend/chain/`、独立 `frontend/api/` 或独立链业务目录。
- 不涉及投票流程。

预计修改目录：
- `sfid/backend/subjects/`：身份主体索引与分区表结构说明，涉及代码。
- `sfid/backend/gov/`：公权机构模块外壳与接口归属，涉及代码。
- `sfid/backend/private/`：私权机构模块外壳与接口归属，涉及代码。
- `sfid/backend/accounts/`：机构账户模块外壳与接口归属，涉及代码。
- `sfid/backend/docs/`：机构资料库模块外壳与接口归属，涉及代码。
- `sfid/backend/admins/`：保留现有权限能力，不新增注册局管理员目录。
- `sfid/backend/subjects/`：身份主体共享模型、公共详情、非法人能力和残留清理，涉及代码。
- `sfid/frontend/subjects/`、`sfid/frontend/gov/`、`sfid/frontend/private/`、`sfid/frontend/accounts/`、`sfid/frontend/docs/`：前端功能目录拆分，涉及代码。
- `memory/05-modules/sfid/`：同步架构、数据库分区、前后端目录边界，涉及文档。

验收标准：
- 后端编译通过。
- 前端构建通过。
- 新目标数据库表从初始化阶段即按 `p_code` 分区。
- `gov` 命名前后端一致。
- `institutions` 前后端业务目录不存在；业务职责已拆到 `gov/private/accounts/docs/subjects`。
- 文档已更新，注释已补充，生成物残留已清理。

执行记录：
- 已新增后端 `subjects/gov/private/accounts/docs` 模块边界；`gov` 与前端命名保持一致。
- 已删除后端旧主体聚合 handler 文件，内部路由归属拆到 `subjects::admin`、`gov::handler`、`private::handler`、`accounts::handler`、`docs::handler`。
- 已把公权自动目录、公安局对账和宪法常量读取迁到 `backend/gov/service.rs`；`subjects/service.rs` 只保留公共校验和默认账户能力。
- 已把清算行资格纯规则迁到 `backend/private/clearing.rs`。
- 已保留现有 `admins` 权限模块，不新增 `registry_admins`。
- 已新增目标分区表初始化：`ids / subjects / citizens / gov / private / accounts / docs / audit`，启动阶段一次性创建 `CN + 43` 个省级分区。
- 已将机构持久化快照改为 `store_subjects`。
- 已停止创建、写入和查询旧机构行表与旧机构账户行表；机构列表改从 `subjects + accounts + admins` 查询。
- 已将公民精确查询写入和查询同步到 `citizens` 目标分区表，保持精简命名。
- 已新增前端 `gov/private/subjects/accounts/docs` 目录入口，`App.tsx` 通过 `GovView` 和 `PrivateView` 渲染机构页。
- 已把前端 `gov`、`private` 页面组件从旧 `subjects` 聚合组件拆出为各自目录真实组件，并删除旧主体聚合页面组件。
- 已新增 `frontend/gov/api.ts`、`frontend/private/api.ts`、`frontend/accounts/api.ts`、`frontend/docs/api.ts`，`subjects/api.ts` 只保留共享类型，不再承载业务请求函数。
- 已更新 `sfid/frontend/tsconfig.json`，显式覆盖新前端目录。
- 已确认 `sfid/backend/institutions`、`sfid/frontend/institutions`、`sfid/backend/sfid` 不再存在。
- 已运行 `cargo check`、`npm run build`。
- 已将私权机构精确列表改为按登录 scope 解析 `p_code / c_code` 后查询目标分区表,不再按中文省市字段或内存全量过滤。
- 已新增公安局和公权机构确定性列表 StoreHandle 只读查询,GET 接口不再执行 backfill、reconcile、写库或分片同步。
- 已将 `gov/private/accounts/docs/subjects` 重复的 HTTP helper 抽到 `sfid/backend/subjects/http.rs`,并清理旧复制函数。
- 已将公权/私权新增弹窗表单抽到 `sfid/frontend/common/institution/CreateInstitutionForm.tsx`,`gov` 与 `private` 仅保留 API 包装。
- 已修复注册局页签中“省管理员列表”首次点击被加载流程重置回市列表的问题。
- 已更新 SFID 后端布局、前端布局、技术总览、统一命名、统一协议和相关活跃任务/决策文档。
- 已再次运行 `cargo check`、`npm run build`。
