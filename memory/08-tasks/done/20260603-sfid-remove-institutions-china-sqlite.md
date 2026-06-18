任务需求：
彻底删除 SFID 前后端 `institutions` 目录，按 `subjects/gov/private/accounts/docs` 物理拆分机构功能；非法人机构功能归入 `sfid/backend/subjects/uninorg`；将 SFID 编码协议迁入 `sfid/backend/sfid_number`；将中国行政区划迁入 `sfid/backend/china` 并使用 SQLite 数据源，删除 `sfid/backend/sfid` 与 Rust 行政区常量目录。

所属模块：
SFID

必须遵守：
- 不保留 `sfid/backend/institutions`、`sfid/frontend/institutions`、`sfid/backend/sfid`。
- 不保留 `crate::institutions`、`../institutions`、`mod institutions`、`crate::sfid` 业务引用。
- 非法人机构能力统一放在 `subjects/uninorg`。
- SFID 编码协议统一放在 `sfid_number`。
- 中国行政区划统一放在 `china`，行政区数据使用 SQLite，不再编译 43 个省 Rust 常量文件。
- `sfid_number` 仍然是唯一且不可变身份，不新增第二身份键。
- 不恢复 `backend/src`、独立 `backend/chain`、独立 `frontend/api`。

预计修改目录：
- `sfid/backend/subjects/`：承接主体模型、主体详情、链端公开查询、非法人功能，涉及代码。
- `sfid/backend/subjects/uninorg/`：承接非法人机构校验和从属关系能力，涉及代码。
- `sfid/backend/gov/`：承接公权机构、公安局、自动目录能力，涉及代码。
- `sfid/backend/private/`：承接私权机构注册和私权列表能力，涉及代码。
- `sfid/backend/accounts/`：承接机构账户、默认账户和 DUOQIAN 地址派生，涉及代码。
- `sfid/backend/docs/`：承接机构资料库，涉及代码。
- `sfid/backend/sfid_number/`：承接 SubjectProperty、机构码、SFID 生成和格式校验，涉及代码。
- `sfid/backend/china/`：承接中国行政区划 SQLite 数据源和查询接口，涉及代码和数据。
- `sfid/frontend/subjects/`：承接主体详情、主体列表公共组件和主体类型，涉及代码。
- `sfid/frontend/gov/`：承接公权机构 UI，涉及代码。
- `sfid/frontend/private/`：承接私权机构 UI，涉及代码。
- `sfid/frontend/accounts/`：承接账户 UI，涉及代码。
- `sfid/frontend/docs/`：承接资料库 UI，涉及代码。
- `memory/05-modules/sfid/`：更新架构文档、目录边界和验收规则，涉及文档。

验收标准：
- `test ! -d sfid/backend/institutions`
- `test ! -d sfid/frontend/institutions`
- `test ! -d sfid/backend/sfid`
- `test -d sfid/backend/china`
- `test -d sfid/backend/sfid_number`
- `test -d sfid/backend/subjects/uninorg`
- `rg "crate::institutions|mod institutions|\\.\\./institutions|from './institutions|crate::sfid::|mod sfid;|city_codes" sfid` 无业务残留。
- `cargo fmt --manifest-path sfid/backend/Cargo.toml`
- `cargo check --manifest-path sfid/backend/Cargo.toml`
- `cargo test --manifest-path sfid/backend/Cargo.toml`
- `npm run build --prefix sfid/frontend`

执行记录：
- 已删除 `sfid/backend/institutions`、`sfid/frontend/institutions`、`sfid/backend/sfid`。
- 已新增 `sfid/backend/subjects/uninorg`、`sfid/backend/sfid_number`、`sfid/backend/china`。
- 已将 43 省 Rust 行政区静态表转换为 `sfid/backend/china/china.sqlite`。
- 已更新 SFID 后端、前端和架构文档,并完成残留搜索。
- `cargo check --manifest-path sfid/backend/Cargo.toml` 通过。
- `cargo test --manifest-path sfid/backend/Cargo.toml` 通过,74 个测试全部通过。
- `npm run build --prefix sfid/frontend` 通过。
