# 20260630 注册局权力线与机构管理员生命周期闭环

## 任务目标

- 将 `genesis-manage` / `genesis-admins` 收回为只负责创世机构与创世管理员写入。
- 将非创世机构注册、修改、注销，以及初始管理员登记统一归属注册局权力线。
- 联邦注册局(FRG)管理员可注册管理本省所有非创世机构。
- 市注册局(CREG)管理员可注册管理本市除市注册局自身以外的所有非创世机构。
- 公权机构生命周期归 `public-manage`，公权机构管理员归 `public-admins`。
- 私权机构生命周期归 `private-manage`，私权机构管理员归 `private-admins`。
- 机构创建后的自治管理员更换按来源区分为注册局登记、内部投票、普选、互选，不由业务模块复刻投票流程。
- 修复 OnChina 机构/管理员链写闭环：本地授权、链交易二维码、钱包提交、indexer 回写职责分离。

## 当前问题

- `genesis-admins` 仍暴露运行期注册局特权入口，职责越过创世管理员初始化边界。
- `public-manage` / `private-manage` 创建机构时要求交易发起人属于新机构管理员集合，不符合“注册局注册机构”的制度模型。
- OnChina 已生成链交易二维码字段，但前端仍只展示普通管理动作签名二维码。
- 机构创建 prepare 顺序错误：创建前没有 `cid_number` / 管理员集合 / 阈值，却尝试生成链交易。
- FRG 能力模型仍保守，只显示注册局管理入口，未开放本省全机构注册能力。

## 预计修改目录

- `citizenchain/runtime/admins/genesis-admins/`
  - 收敛创世管理员模块边界，清理注册局运行期特权桥。
- `citizenchain/runtime/entity/genesis-manage/`
  - 核对创世机构模块边界并补充必要注释。
- `citizenchain/runtime/entity/public-manage/`
  - 调整公权机构注册授权，从“发起人必须属于目标机构管理员”改为“发起人必须是合法注册局管理员”。
- `citizenchain/runtime/admins/public-admins/`
  - 承接公权机构管理员登记与自治回写。
- `citizenchain/runtime/entity/private-manage/`
  - 调整私权机构注册授权。
- `citizenchain/runtime/admins/private-admins/`
  - 承接私权机构管理员登记与自治回写。
- `citizenchain/onchina/src/`
  - 修复注册局权限、机构草稿、prepare、链交易 QR、indexer 回写。
- `citizenchain/onchina/frontend/`
  - 开放 FRG 本省机构注册入口，拆分普通授权 QR 与链交易 QR。
- `memory/`
  - 更新 ADR、模块技术文档和本任务验收记录。

## 验收口径

- FRG 管理员能注册本省 CREG，并登记 CREG 管理员。
- FRG 管理员能注册本省普通公权、私权、教育、非法人机构。
- CREG 管理员能注册本市普通公权、私权、教育、非法人机构。
- CREG 管理员不能注册 CREG。
- CREG 管理员跨市注册被拒绝。
- 非注册局管理员不能执行注册局注册动作。
- 创建机构时注册局管理员不需要属于目标机构管理员集合。
- 目标机构创建完成后，目标机构管理员能按自治流程更换管理员。
- 普通管理动作签名与链交易签名职责清楚，链上确认由 indexer 回写。

## 执行记录

- 2026-06-30 创建任务卡，开始执行。
- 2026-06-30 runtime 已改为注册局直登模型：`public-manage` / `private-manage` 创建机构时校验注册局权限，交易成功即写入机构、账户和初始管理员 Active；注册局管理员不需要属于目标机构管理员集合，目标机构管理员也不再对初始创建投票。
- 2026-06-30 `genesis-admins` 已移除市注册局管理员运行期直设入口；市注册局及其初始管理员改由注册局通过机构创建交易一次性写入。
- 2026-06-30 OnChina 创建输入已加入 `admins` + `threshold`；创建接口在本地写入机构、默认账户和初始管理员后，返回 `propose_create_institution` 链交易二维码，链投影置 `PENDING_ON_CHAIN`。
- 2026-06-30 OnChina 前端创建弹窗已加入初始管理员合集和阈值；FRG 省级管理员在公权创建入口可选择创建本省城市的市注册局，CREG 管理员仍不能创建 CREG。
- 2026-06-30 清理旧链写残留：移除市注册局管理员直设相关 OnChina 编码和 prepare 输出。
- 2026-06-30 验证：`cargo test -p public-manage -p private-manage --lib`、`cargo test -p genesis-admins --lib`、`cargo check -p citizenchain`、`cargo check -p onchina`、`cargo test -p onchina`、`npm run build` 通过。
- 2026-06-30 真实运行态验收：启动 OnChina 前端 preview 并访问 `http://localhost:5179/`，首页和新构建 JS 资源均返回 200；本次未提交真实链交易，真实链交易仍需本地链、OnChina 数据库和 CitizenWallet 联动环境。
- 2026-06-30 残留清理验收：已清理旧 `genesis-admins` 市注册局管理员直设入口、OnChina 旧 admin-set 编码和创建机构投票执行残留；`git diff --check` 通过。
