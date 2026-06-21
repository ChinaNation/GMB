# 统一 SFID 机构字段与账户字段命名

## 任务需求

- 全仓库统一机构名称字段:`sfid_full_name` 表示机构全称,`sfid_short_name` 表示机构简称。
- 全仓库统一机构账户字段:`main_account`、`fee_account`、`stake_account`、`duoqian_account` 等,不再使用 `*_address` 表达机构账户。
- 行政区名称字段统一为 `province_name`、`city_name`、`town_name`;代码字段保留 `province_code`、`city_code`、`town_code`。
- 治理主体统一表达永久 `sfid_number`、永久 `main_account` 和可变 `sfid_full_name/sfid_short_name`;链上治理账户参数使用 `governance_account`。
- 全仓库所有机构/个人多签管理员集合统一为 `admins`;链上唯一真源为 `admins-change::AdminAccounts`。
- 删除独立管理员角色体系;联邦注册局、市注册局、公安局、国储会、省储会、省储行、普通机构和个人多签均只通过本机构/账户 `admins` 判断管理员身份。
- CPMS 删除旧本地管理员角色;市公安局管理员来自该公安局机构 `admins` 快照,CPMS 本机只维护 `OPERATOR` 操作员。
- runtime 凭证签发模型统一为 `issuer_sfid_number + issuer_main_account + signer_pubkey`;`scope_*` 仅表示业务作用域,不表示签发人身份。
- 本次按重新创世处理,不做兼容、迁移、双轨或旧字段保留。

## 涉及模块

- `citizenchain/runtime/`:runtime primitives、机构多签、个人多签、治理载荷与测试字段统一。
- `citizenchain/node/`:节点端读取和展示 runtime 常量与治理主体字段统一。
- `sfid/backend/`:数据库 schema、DTO、公开接口、公权机构生成和账户派生字段统一。
- `sfid/frontend/`:管理端字段、表单和展示字段统一。
- `wuminapp/`:公民端公权机构包、治理静态注册表、页面展示和解码字段统一。
- `wumin/`:公民钱包展示、签名解码和静态机构字段统一。
- `cpms/`:离线行政区包和地址字段命名统一。
- `tools/`:生成器输出字段统一。
- `memory/`:统一命名、统一协议和模块技术文档同步。

## 执行规则

- 验收时以旧机构名称字段、旧机构账户字段、旧行政区名称字段、旧管理员字段和旧角色为扫描对象；目标协议只允许 `sfid_full_name`、`sfid_short_name`、`*_account`、`province_name`、`city_name`、`town_name`、`admins`、`operators`、`signer_pubkey`。
- 不新增兼容分支、旧字段别名、过渡格式或迁移适配。
- 涉及 `citizenchain/runtime/**` 的管理员/凭证统一修改已经单独列出路径和原因,并已获得用户二次确认。
- 改代码后必须同步文档、完善必要中文注释并清理残留。

## 验收计划

- 全仓库搜索旧字段残留。
- 全仓库搜索并删除旧注册局角色、旧 CPMS 角色、旧签发花名册和旧凭证字段残留。
- 执行受影响生成器,重新生成静态数据包和代码生成物。
- 运行各模块格式化、类型检查、测试或构建。
- 涉及数据库和公开包的部分用真实 SQLite/JSON 数据检查字段结构。

## 执行记录

- 已统一 runtime、node、SFID 后端、SFID 前端、wuminapp、公民钱包、工具脚本和公开机构包中的机构全称、机构简称、机构账户和行政区名称字段。
- 已将 runtime 机构注册凭证、投票凭证、人口快照凭证和签发上下文统一为 `issuer_sfid_number + issuer_main_account + signer_pubkey + scope_*`。
- 已删除 `sfid-system` 内独立签发花名册目录和相关旧文档,签发管理员真源统一由 `admins-change::AdminAccounts.admins` 提供。
- 已将 CPMS 本地角色统一为 `ADMIN / OPERATOR`,并删除 `super_admin / operator_admin` 旧目录、旧角色值和旧文案。
- 已将节点端、wuminapp、公民钱包的机构注册、联合投票和人口快照凭证字段统一到 issuer/scope 新字段集。
- 已删除临时批量改名脚本,避免后续误用旧脚本重复改写。
- 已重新生成 wuminapp 公权机构包和治理静态注册表。
- 已删除 wuminapp 行政区字典包、公权机构包 loader 中的旧 manifest 回退分支,当前只接受省级版本表格式。
- 已把 SFID 后端主体模型、列表 DTO、CPMS 安装码输入、公权机构公开接口版本响应和审计载荷同步到 `province_name/city_name/town_name`。
- 已同步白皮书中的 `stake_account/main_account` 字段和当前 runtime 常量路径,并重新生成 node 前端本地文档。
- 已同步更新统一命名、统一协议、SFID 后端链交互、runtime sfid-system、wuminapp 治理和相关模块技术文档。

## 验收记录

- 残留扫描:旧机构字段、旧账户字段、旧 storage 名、旧地址 helper 名在代码和当前技术文档中无命中。
- 签发模型扫描:旧独立签发花名册、旧省份加签名人组合、旧签发管理员字段和旧签发环境变量在当前代码和当前技术文档中无命中。
- CPMS 旧角色扫描:`SUPER_ADMIN / OPERATOR_ADMIN / super_admin / operator_admin / 超级管理员 / 操作管理员` 在当前代码和当前技术文档中无命中。
- Runtime 扫描:`citizenchain/runtime/**/*.rs` 中无裸 `province`、`city`、`town` 字段残留。
- 严格 manifest 扫描:wuminapp 行政区/公权机构 loader 与测试中无旧格式 manifest 回退残留。
- 格式化:`cargo fmt --manifest-path citizenchain/Cargo.toml --all`、CPMS 后端 `cargo fmt`、wumin `dart format` 已执行。
- 构建/检查:`cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain`、citizenchain node `cargo check`、SFID 后端 `cargo check`、CPMS 后端 `cargo check`、SFID 前端 `npm run build`、node 前端 `npm run build`、CPMS 前端 `npm run build`、wuminapp `flutter analyze` 已通过。
- 测试:`cargo test --manifest-path citizenchain/Cargo.toml -p sfid-system`、`cargo test --manifest-path citizenchain/Cargo.toml -p citizen-issuance`、wumin `flutter test test/signer/payload_decoder_test.dart`、wuminapp `flutter test test/governance/organization-manage/institution_manage_service_test.dart` 已通过。
- 补充检查:`git diff --check` 已通过。
