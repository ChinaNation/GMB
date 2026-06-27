# runtime 模组重组：admins/public/private 三模组归位 + admins-change 升格 admin-management

任务需求：
- 把 runtime 四个治理模块按模组/模块两层结构归位，彻底搬迁、零残留：
  1. `governance/admins-change` → `admins/admin-management`（新建 admins 模组；crate+标识符改名 admins-change/admins_change → admin-management/admin_management）
  2. `governance/legislation-yuan` → `public/legislation-yuan`（不改名）
  3. `governance/personal-manage` → `private/personal-manage`（不改名）
  4. `governance/organization-manage` → `private/organization-manage`（不改名）
- node + frontend 侧对应搬迁：有功能的才迁，无功能不建空壳
  - node Rust：`governance/admins_change` → `admins/admin_management`；`governance/organization_manage` → `private/organization_manage`
  - frontend：`governance/admins-change` → `admins/admin-management`；`governance/organization-manage` → `private/organization-manage`
  - personal/legislation node 侧无功能 → 不建壳
- 链上 pallet 名 `AdminsChange`（storage 前缀）保持不变，避免破坏 citizenapp/citizenwallet 存储查询；只改 crate/目录/Rust 标识符
- 独立 admins 模组目的：各机构管理员数量/阈值不同 + 后续给管理员加字段，独立便于扩展

所属模块：Blockchain Agent（citizenchain/runtime + node + frontend 全域）

必须遵守：
- 死规则一：彻底改造、不保留兼容、零残留（旧目录删空、旧标识符全清）
- 死规则二：命名全仓统一且精简
- 链未运行，不做 migration，不问创世
- MODULE_TAG / pallet index / 签名 op_tag 保持不动（纯重构，零链行为变更）
- dist 构建产物不手改，重新 build 覆盖

输出物：
- 代码搬迁 + 全仓引用改名
- 中文注释
- 编译/测试通过
- 文档更新（runtime 结构相关 memory 文档）
- 残留清理

验收标准：
- `cargo build`（runtime）绿
- node Rust 编译 + frontend build 绿
- 全仓 grep 无 `admins-change` / `admins_change` 残留（admins 字段除外）
- 旧 governance 子目录已删空
- 文档已更新、残留已清理
