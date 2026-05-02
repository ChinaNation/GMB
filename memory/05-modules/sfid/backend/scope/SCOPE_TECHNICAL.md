# scope/ — 三角色权限范围过滤

## 定位

任务卡 2 引入的独立模块。所有返回多条记录的 list API **必须**经过本模块过滤,避免每个
handler 手写 `if ctx.role == ...` 分支。

## 文件结构

```
backend/scope/
├── mod.rs     — pub use 聚合
├── rules.rs   — VisibleScope 结构体 + get_visible_scope(ctx)
└── filter.rs  — HasProvinceCity trait + filter_by_scope 泛型函数
```

## 三角色范围

| 角色 | 可见省 | 可见市 | 进 tab 跳过省列表 | 进 tab 跳过市列表 |
|---|---|---|---|---|
| KeyAdmin   | 全国 | 全部 | 否 | 否 |
| ShengAdmin | 本省(`admin_province`) | 本省全部市 | 是 | 否 |
| ShiAdmin   | 本省 | 本市(`admin_city`) | 是 | 是 |

## 使用方式

```rust
let ctx = require_admin_any(&state, &headers)?;
let scope = scope::get_visible_scope(&ctx);
let filtered = scope::filter_by_scope(&rows, &scope);
```

记录类型实现 `HasProvinceCity` trait 即可被 `filter_by_scope` 处理:

```rust
impl HasProvinceCity for MyRecord {
    fn province(&self) -> &str { &self.province }
    fn city(&self) -> &str { &self.city }
}
```

## 安全 fallback

`get_visible_scope` 在 ShengAdmin 缺 `admin_province` 或 ShiAdmin 缺 `admin_city` 时会 fallback
到"零范围"(provinces=["__MISSING__"]),确保数据不会被误放行。调用方应在 `require_admin_*`
里先校验必要字段。

## 铁律

见 `feedback_scope_auto_filter.md`。

## 历史

- 2026-04-08 任务卡 2 落地


## ADR-008 Phase 23e 更新（2026-05-01）

KEY_ADMIN 整角色废止；省管理员 3-tier 自治（main / backup_1 / backup_2）。
本文档涉及 KEY_ADMIN / key-admins / chain_keyring / signing_seed_hex / known_key_seeds / public_key_hex / require_key_admin / require_institution_or_key_admin / KeyringRotate* 的章节均已失效，
实际行为以 `memory/04-decisions/ADR-008-sheng-admin-3tier-and-key-admin-removal.md` 与代码为准。
