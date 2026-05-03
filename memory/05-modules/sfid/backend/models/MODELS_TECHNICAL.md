# MODELS 模块技术文档

- 最后更新:2026-05-02
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-models-scope边界整改.md`

## 1. 模块定位

- 路径:`sfid/backend/models`
- 职责:只维护 SFID 后端跨业务共享的数据结构。
- 边界:不得继续承载具体业务模块 DTO。

## 2. 当前结构

```text
sfid/backend/models/
├── mod.rs      # 全局共享模型 facade
├── error.rs    # ApiResponse / ApiError / HealthData
├── role.rs     # AdminRole / AdminStatus / AdminUser / 操作员 DTO
└── store.rs    # Store 聚合体、审计、指标、链请求回执、回调、奖励、投票缓存
```

## 3. 已归还的业务模型

```text
sfid/backend/citizens/model.rs
  # 公民身份、绑定状态机、绑定/解绑 DTO、投票账户 DTO、扫码 QR 载荷

sfid/backend/cpms/model.rs
  # CPMS 站点、安装 token、QR1/QR2/QR3/QR4、匿名证书 DTO

sfid/backend/sfid/model.rs
  # SFID 管理页元信息 DTO

sfid/backend/institutions/model.rs
  # 机构链状态 InstitutionChainStatus / MultisigChainStatus
```

## 4. 使用方式

- `main.rs` 继续 `pub(crate) use models::*` 暴露全局共享类型。
- 业务模型由对应模块导出,例如 `citizens::model::*`、`cpms::model::*`。
- `Store` 可以引用业务模块模型,但业务 DTO 不反向塞回 `models`。

## 5. 铁律

- 新增业务 DTO 放到对应功能模块的 `model.rs`。
- 只有真正跨模块共享且没有明确业务归属的模型才能放入 `models`。
- 禁止在 `models` 目录恢复公民、CPMS、SFID 元信息、空权限占位、
  空会话占位或省管理员槽位 facade 文件。
