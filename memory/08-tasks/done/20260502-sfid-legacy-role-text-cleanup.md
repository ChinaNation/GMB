# SFID 旧角色表述清理

## 任务来源

用户要求彻底清理已废止管理员角色相关表述。

## 本次目标

- 清理当前 SFID 代码、注释、文档、任务索引里的旧角色残留。
- 将旧 ADR、旧任务卡文件名和正文改为当前省管理员三槽自治表述。
- 数据库迁移改为当前二角色基线，不再保留旧对象创建或刷新逻辑。

## 影响范围

- `sfid/`
- `citizenchain/runtime/otherpallet/sfid-system/`
- `memory/04-decisions/`
- `memory/05-modules/sfid/`
- `memory/08-tasks/`

## 验收标准

- 残留扫描无旧角色关键词。
- 当前代码仍通过后端与前端校验。
- 文档仍准确表达当前二角色与省管理员三槽自治结构。

## 完成记录

- 已清理当前代码、注释、前端提示、技术文档、ADR、任务卡正文中的旧角色表述。
- 已重命名旧 ADR、旧任务卡和数据库迁移文件,避免文件名继续残留。
- 已将数据库迁移收口为当前二角色基线,不再创建或刷新旧角色对象。
- 已执行 `cargo fmt`、`cargo check --manifest-path sfid/backend/Cargo.toml`、
  `npm run build`、内容残留扫描和文件名残留扫描。
