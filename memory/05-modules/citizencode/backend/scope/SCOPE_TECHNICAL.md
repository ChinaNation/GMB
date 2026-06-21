# scope/ — 注册局机构权限范围过滤

- 最后更新:2026-05-02
- 任务卡:
  - `memory/08-tasks/done/20260502-cid-models-scope边界整改.md`

## 定位

`scope` 只负责登录管理员的数据可见范围和省/市过滤规则。所有返回多条记录的
list API 应通过本模块派生 `VisibleScope` 并过滤,避免每个 handler 手写注册局机构分支。

## 当前结构

```text
citizencode/backend/scope/
├── mod.rs             # scope 聚合导出
├── rules.rs           # VisibleScope + get_visible_scope(ctx)
└── filter.rs          # HasProvinceCity + filter_by_scope
```

## 注册局机构范围

| 注册局机构 | 可见省 | 可见市 | 进 tab 跳过省列表 | 进 tab 跳过市列表 |
|---|---|---|---|---|
| FEDERAL_REGISTRY | 本省(`scope_province_name`) | 本省全部市 | 是 | 否 |
| CITY_REGISTRY | 本省 | 本市(`scope_city_name`) | 是 | 是 |

机构创建必须复用上述范围:联邦注册局机构的 `admins` 可在本省任意市创建手动机构,市注册局机构的 `admins` 只能在本市创建;业务模块不得再用“是否存在 `scope_city_name`”额外限制联邦注册局创建 G 公法人。

## 已迁出内容

```text
citizencode/backend/citizens/handler.rs
  # 公民列表和公开身份查询

citizencode/backend/audit.rs
  # 审计日志查询

citizencode/backend/citizenpassport/scope.rs
  # CPMS 站点省域判断

citizencode/backend/crypto/pubkey.rs
  # sr25519 pubkey 规范化与比较
```

## 使用方式

```rust
let ctx = require_admin_any(&state, &headers)?;
let scope = scope::get_visible_scope(&ctx);
let filtered = scope::filter_by_scope(&rows, &scope);
```

记录类型实现 `HasProvinceCity` trait 即可被 `filter_by_scope` 处理。

## 安全 fallback

`get_visible_scope` 在联邦注册局缺 `scope_province_name` 或市注册局缺
`scope_city_name` 时会 fallback 到零范围,确保数据不会被误放行。调用方应在
`require_admin_*` 里先校验必要字段。

## 铁律

- 禁止在 `scope` 中新增 HTTP handler。
- 禁止把具体业务模块的专用判断放入 `scope`。
- 禁止在 `scope` 目录恢复查询、审计、CPMS 专用判断或 pubkey 工具文件。
