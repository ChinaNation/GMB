# 任务卡：机构管理员、岗位和最小首次登记三步改造

## 当前状态

- 状态：进行中
- 当前步骤：第1步——管理员人员模型、岗位独立模型、默认法定代表人岗位和最小首次登记
- 用户确认：2026-07-17
- 执行规则：每一步先确认方案；执行完成后立即更新文档、完善中文注释、清理残留，再输出下一步技术方案

## 任务需求

机构唯一主键继续使用 `cid_number`，但管理员和岗位必须彻底分离：

- 管理员是人，统一保存在 `admins`，每项由 `admin_name + admin_account` 组成。
- `admin_account` 是钱包账户和唯一签名授权字段；`admin_name` 只用于公开展示，不参与授权。
- 管理员钱包能从 OnChina 公民资料解析姓名时使用公民姓名；无法解析时名称统一为“管理员”。
- 普通机构始终至少有两个管理员；固定治理机构继续遵守制度精确人数。
- 岗位是机构职位，不是管理员；管理员可无岗位，岗位可空缺。
- 每个机构必须默认且唯一存在 `LR / 法定代表人` 岗位；该岗位不可删除、停用、改名或改码。
- 首次创建不自动把管理员任命为法定代表人，法定代表人三字段保持 `None`。
- 岗位任职不能再反向派生或覆盖 `admins`。
- 首次机构登记只提交最小身份资料与管理员，runtime 自动创建制度账户、默认法定代表人岗位和严格多数阈值。
- 注册协会 `SFAS` 的盈利属性按实例选择，不能固定为非盈利。
- runtime、Node、OnChina、公民、CitizenWallet 五端同步，不保留旧载荷或兼容分支。

## 所属模块

- `citizenchain/runtime/admins`
- `citizenchain/runtime/entity`
- `citizenchain/runtime/primitives`
- `citizenchain/runtime/genesis`
- `citizenchain/runtime/src`
- `citizenchain/node`
- `citizenchain/onchina`
- `citizenapp`
- `citizenwallet`
- `memory`

## 输入文档

- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/repo-map.md`
- `memory/03-security/security-rules.md`
- `memory/07-ai/unified-protocols.md`
- `memory/07-ai/unified-naming.md`
- `memory/05-modules/citizenchain/runtime/admins/ADMINS_TECHNICAL.md`
- `memory/05-modules/citizenchain/runtime/entity/entity-primitives/ENTITY_PRIMITIVES_TECHNICAL.md`
- `memory/05-modules/citizenchain/runtime/entity/public-manage/PUBLIC_MANAGE_TECHNICAL.md`
- `memory/05-modules/citizenchain/runtime/entity/private-manage/PRIVATE_MANAGE_TECHNICAL.md`
- `memory/01-architecture/onchina/ONCHINA_TECHNICAL.md`
- `memory/05-modules/citizenchain/onchina/BACKEND_TECHNICAL.md`
- `memory/05-modules/citizenchain/onchina/FRONTEND_TECHNICAL.md`

## 三步范围

### 第1步

- `admins` 改为管理员姓名与钱包账户的人员集合。
- 删除“岗位有效任职并集派生 admins”的链上逻辑。
- 所有机构自动建立唯一 `LR / 法定代表人` 岗位，允许空缺。
- 首次创建载荷收紧为最小身份字段、管理员集合和注册局授权字段。
- runtime 自动派生机构码、全部强制协议账户和严格多数阈值。
- OnChina 按钱包解析公民姓名，无法解析时使用“管理员”。
- `SFAS` 盈利属性改为实例必选。
- CitizenWallet、Node、CitizenApp 同步新 storage/call 契约。

### 第2步

- 机构管理员新增、删除、换人和姓名更新。
- 普通岗位新增、变更、停用和删除。
- 管理员与岗位任职维护。
- 法定代表人任命、更换、解除及三字段原子更新。
- 普通岗位短随机码唯一生成。

### 第3步

- 五端读侧统一、界面收口和全仓残留审计。
- 真实本地链、PostgreSQL、OnChina 页面和二维码签名全链路验收。
- 完成最终文档和任务归档。

## 第1步验收标准

- [ ] `admins` 每项只使用 `admin_name + admin_account`，授权只比较账户。
- [ ] 普通机构管理员少于2人时拒绝。
- [ ] 没有任何岗位任职的管理员仍然拥有机构管理员签名权限。
- [ ] 岗位新增或清空任职不会改变管理员集合。
- [ ] 每个运行期及创世机构都有且只有一个 `LR / 法定代表人` 岗位。
- [ ] `LR` 岗位允许空缺，首次创建不伪造法定代表人。
- [ ] 最小首次创建call不再携带法定代表人、账户数组、完整岗位/任职或手工阈值。
- [ ] runtime 自动创建完整强制协议账户集合，初始余额为零。
- [ ] 注册局管理员只签名，0.1元费用只从注册局费用账户扣除。
- [ ] `SFAS` 支持盈利和非盈利两类CID，未选择时拒绝。
- [ ] Node、OnChina、CitizenApp、CitizenWallet按新协议编译和测试通过。
- [ ] 使用真实本地服务、数据库、链和页面完成运行态验收。
- [ ] 文档已更新、中文注释已完善、旧代码和旧口径已清理。

## 强制约束

- 不建立第二套管理员授权真源。
- 不按管理员姓名鉴权。
- 不把岗位名称当业务权限标识。
- 不从 `admins[0]` 推导法定代表人。
- 不保留旧call、旧SCALE布局、旧二维码解码或旧数据库写入流程。
- 不在链确认前写入OnChina正式机构投影。
- 不修改个人多签管理员的数据模型。
- 不推送GitHub、不部署、不重新创世，除非用户另行授权。

## 输出物

- runtime、Node、OnChina、CitizenApp、CitizenWallet代码
- 中文注释
- 单元、集成和真实运行态测试
- `memory`协议与模块文档更新
- 旧载荷、旧字段、旧注释、旧文案和旧测试残留清理

## 执行记录

- 2026-07-17：用户确认第1步、新任务卡创建及指定runtime路径二次修改权限。
