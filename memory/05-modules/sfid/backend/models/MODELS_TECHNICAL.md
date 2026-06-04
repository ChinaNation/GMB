# MODELS 模块技术文档

- 最后更新:2026-05-31
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-models-scope边界整改.md`
  - `memory/08-tasks/done/20260525-sfid-cpms-store.md`
  - `memory/08-tasks/done/20260530-sfid-admin-permission-step2.md`
  - `memory/08-tasks/open/20260531-sfid-admin-model-no-status.md`

## 1. 模块定位

- 路径:`sfid/backend/models`
- 职责:只维护 SFID 后端跨业务共享的数据结构。
- 边界:不得继续承载具体业务模块 DTO。

## 2. 当前结构

```text
sfid/backend/models/
├── mod.rs      # 全局共享模型 facade
├── error.rs    # ApiResponse / ApiError / HealthData
├── role.rs     # AdminRole / AdminUser / 省级管理员与市级管理员 DTO
└── store.rs    # Store 聚合体类型、审计、指标、链请求回执、回调、奖励、投票缓存
```

## 3. 已归还的业务模型

```text
sfid/backend/citizens/model.rs
  # 公民身份、绑定状态机、绑定/解绑 DTO、投票账户 DTO、扫码 QR 载荷

sfid/backend/cpms/model.rs
  # CPMS 安装授权、INSTALL/ARCHIVE DTO、档案验真结果

sfid/backend/sfid_number/model.rs
  # SFID 编码元信息 DTO

sfid/backend/subjects/model.rs
  # 机构链状态 InstitutionChainStatus / MultisigChainStatus
```

## 4. 使用方式

- `main.rs` 继续 `pub(crate) use models::*` 暴露全局共享类型。
- 业务模型由对应模块导出,例如 `citizens::model::*`、`cpms::model::*`。
- `Store` 仍作为内存聚合体类型使用,但持久化由 `main.rs` 拆到
  `store_citizens / store_cpms / store_subjects / store_ops` 四张模块快照表。
- `Store` 可以引用业务模块模型,但业务 DTO 不反向塞回 `models`。
- 管理员安全分级的跨模块共享状态放在 `Store`:
  - `admin_passkeys_by_credential_id`:省/市管理员浏览器 Passkey 凭据。
  - `admin_passkey_registration_challenges`:Passkey 绑定过程中的 WebAuthn + 冷钱包确认挑战。
  - `admin_action_challenges`:管理员治理和业务安全动作 prepare 阶段的短期挑战。
  - `admin_security_grants`:业务写接口消费的一次性短期授权。

## 5. 铁律

- 新增业务 DTO 放到对应功能模块的 `model.rs`。
- 只有真正跨模块共享且没有明确业务归属的模型才能放入 `models`。
- 禁止在 `models` 中承载数据库结构初始化或 Store 持久化细节。
- 禁止在 `models` 目录恢复公民、CPMS、SFID 元信息、空权限占位、
  空会话占位或省管理员槽位 facade 文件。
