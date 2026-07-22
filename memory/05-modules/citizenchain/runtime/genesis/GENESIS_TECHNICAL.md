# GenesisPallet 技术文档

## 1. 模块职责

`genesis-pallet` 只负责：

- 保存 `Genesis` / `Operation` 链阶段；
- 保存开发者能否直接升级 runtime 的一次性开关；
- 在 block#0 写入创世宣言、国家宣言和创世人口；
- 调用 runtime 注入的机构 seeder 写入创世机构、固定岗位、固定岗位权限、创世任职和管理员钱包集合。

本模块不提供 extrinsic，不保存 PoW 出块时间，也不向节点提供出块时间 Runtime API。
PoW 六分钟是 `primitives::pow_const::POW_TARGET_BLOCK_TIME_MS` 固定的难度调整平均目标，
与链阶段无关；有效工作量证明找到后立即出块，没有最短等待或最晚期限。

### 创世机构子模块

```text
runtime/genesis/src/institution/
├── mod.rs          # 对外只暴露 build 入口，声明职责边界
├── fixed_roles.rs  # 89 个公权受保护创世机构的固定岗位、席位与既有钱包索引映射；不写 storage
└── seeder.rs       # 唯一写入方：公权写 public-*；公民链基金会写 private-*
```

- 岗位协议常量来自 `primitives::governance_skeleton`，钱包来自既有 `CHINA_*` 常量；
- 构建前断言固定钱包数量等于席位总数，且固定岗位钱包不得重复；
- 全部机构必须写入唯一 `LR / 法定代表人` 岗位；岗位可以没有任职，公开 `legal_representative` 结构可为 `None`，不得从管理员首位、机构主账户或其它钱包推导；
- 后续依法任命法定代表人属于 entity 运行期流程，不属于 genesis 职责。

私权创世机构“公民链技术发展基金会”是 SFGY 非营利法人，也是唯一在创世明确携带法定代表人的例外：

- 基金会简称“公民链基金会”，英文全称 `CitizenChain Technology Development Foundation`，英文简称 `CitizenChain Technology Foundation`；CID `GZ018-SFGYR-201206100-2026`，主账户 `0xe86aa3cd794651257dea9b7cad1abc4f0ce05940c1aecccd2ed8dd2fc9907023`，费用账户 `0xaa23304c7b663ba25a9d3a2fb1efafdd650ecf2504a2caedc228fe81b46b4333`；
- 法定代表人为程伟，公民 CID 引用 `GZ000-CTZN6-198805200-2026`，法定代表人账户 `0xd6d73cfd7d6b7c5692749b7c46fd3fe398f16f84283910dbf15f74472e1e3938`；创世不伪造第二份公民记录，后续由对应注册局依法从链上公民真源核验；
- 只创建一条程伟管理员人员记录 `Admin { admin_account, family_name, given_name }`；同一钱包分别任职 `LR / 法定代表人`、`GENESIS_PRODUCT_MANAGER / 创世产品经理`、`GENESIS_PROGRAMMER / 创世程序员`，每岗一席。三岗的岗位码、岗位名和岗位权限永久固定，但基金会仍可增加普通动态岗位；
- `PrivateAdmins::AdminAccounts`、三项 `PrivateManage` 岗位/任职和 `PrivateManage::InstitutionGovernanceThresholds[cid_number] = 2` 在同一创世构建中写入。阈值 2 是机构阈值，不与唯一管理员钱包数量绑定；
- 基金会身份、主/费用协议账户以及三个固定岗位治理骨架受 NodeGuard 保护，成员依法轮换必须通过同一原子治理结果同步更新 admins 和岗位任职，不能裸改其中一侧；NodeGuard 不得禁止新增普通动态岗位。

## 1.1 ADR-039 目标创世权限（实现中）

- 创世 seeder 必须为全部创世固定岗位写入固定 `RoleBusinessPermission`，不能只写岗位码、名称和席位。
- 创世 admins 只作为可任职人员集合，不直接取得业务权限；业务权限必须来自固定岗位的有效创世任职。
- 全部创世固定岗位使用既定固定码；动态岗位码生成使用 `GMB_ROLE_V1`，但 genesis 不用动态算法替代固定码。
- 普通机构不由 genesis seeder 创建；其运行期创建必须原子建立 LR、至少一个初始治理岗位及权限、任职和投票规则。
- 本节是 ADR-039 目标，runtime/genesis 与 NodeGuard 代码迁移分别在任务卡第 3、4 步执行。

## 2. 五个受守卫字段

| 存储项 | 类型 | 创世 RAW 形态 | 永久规则 |
|---|---|---|---|
| `Phase` | `ChainPhase` | 缺省即 `Genesis` | 只允许一次切换为 `Operation` |
| `DeveloperUpgradeEnabled` | `bool` | 缺省即 `true` | 只允许与阶段同步切换为 `false` |
| `CitizensDeclaration` | `BoundedVec<u8, MaxDeclarationLen>` | `CITIZENS` 的准确 UTF-8 字节 | 永久逐字冻结 |
| `CountryDeclaration` | `BoundedVec<u8, MaxDeclarationLen>` | `COUNTRY` 的准确 UTF-8 字节 | 永久逐字冻结 |
| `CitizenMax` | `u64` | `1_443_497_378` | 永久冻结 |

FRAME `StorageVersion` 必须保持 0。旧 `TargetBlockTimeMs` 已删除，同前缀未知 RAW key
（包括该旧字段）由 NodeGuard fail-closed 拒绝，不保留兼容或影子状态。

## 3. 一次性阶段状态机

合法创世状态：

```text
(Phase, DeveloperUpgradeEnabled) = (Genesis, true)
```

唯一合法目标状态：

```text
(Phase, DeveloperUpgradeEnabled) = (Operation, false)
```

约束：

- 两个字段只能在同一个包含 `:code` 变化的 runtime 升级区块中原子写入；
- 禁止普通区块修改、部分修改、显式写回创世默认值、反向切换和重新启用开发者直升；
- 转为 `Operation` 后永久冻结；
- 本轮没有自动执行阶段切换，正式切换仍需单独确认迁移和治理授权。

## 4. 公共接口

模块只保留：

```rust
pub trait DeveloperUpgradeCheck {
    fn is_enabled() -> bool;
}
```

`runtime-upgrade` 使用该接口选择当前允许的升级授权路径。旧 `GenesisPalletApi`、
`TargetBlockTime` trait、`TargetBlockTimeChanged` 事件以及未被调用的阶段事件已经删除。

## 5. 创世固定真源

固定值来自 `runtime/primitives/src/genesis.rs`：

- `CITIZENS`：创世宣言；
- `COUNTRY`：国家宣言；
- `GENESIS_CITIZEN_MAX = 1_443_497_378`；
- `GENESIS_ISSUANCE = 14_434_973_780_000` 分。

`runtime/src/genesis.rs` 把前三项写入 `GenesisConfig`。NodeGuard 使用相同的节点编译期
真源重新构造 RAW key 和 SCALE 值，不信任 runtime metadata、getter 或 Runtime API。

## 6. NodeGuard 执法

`node/src/core/node_guard/genesis_pallet.rs` 在四条路径执行：

1. 节点启动：读取 block#0 的整个 `GenesisPallet` 前缀，确认创世事实和缺省阶段状态；
2. 普通区块：三个创世事实和 StorageVersion 任何触碰都拒绝；
3. runtime 升级：只接受两字段唯一原子单向转换，并在 `:code` 后复核完整状态；
4. 完整状态导入：整个 pallet 前缀进入共享单遍分区，未知 key、缺失值、错误 SCALE、
   尾随字节和非规范状态全部拒绝。

## 7. 测试与验收

- `genesis-pallet` 单元测试：默认阶段、开发者开关、trait、阶段模拟和创世配置；
- NodeGuard 策略测试：RAW key、两种规范状态、三个固定事实、未知旧字段、畸形 SCALE、
  非规范默认写回、合法原子转换、无 `:code`、部分转换、反向转换和固定事实触碰；
- NodeGuard 真实 runtime 创世完整状态测试确认五字段策略参与共享扫描和拒绝链路；
- 最终结果以任务卡中的本轮编译、WASM 和 fresh 节点真实验收记录为准。

## 8. 文件索引

- `citizenchain/runtime/genesis/src/lib.rs`：类型、存储、创世构建和开发者升级查询；
- `citizenchain/runtime/genesis/src/institution/fixed_roles.rs`：89 个公权受保护创世机构的固定岗位、席位和钱包索引映射；
- PRS、NLG、NSN、NRP、NSP、NED 六个国家级单例在 block#0 写精确机构身份、制度账户和唯一空缺 `LR / 法定代表人` 岗位，不写成员岗位、任职、admins 或动态阈值。首次组成前先独立登记 admins；其中 NSN、NRP、NED 还受法定成员岗位与人数区间约束。
- `citizenchain/runtime/genesis/src/institution/seeder.rs`：公私权创世机构、岗位、任职和管理员钱包唯一写入方；公民链基金会写入 `private-manage` / `private-admins`，不污染公权目录；
- `citizenchain/runtime/genesis/src/tests/mod.rs`：pallet 单元测试；
- `citizenchain/runtime/primitives/src/genesis.rs`：三个创世事实的固定真源；
- `citizenchain/runtime/src/genesis.rs`：真实 runtime genesis patch；
- `citizenchain/node/src/core/node_guard/genesis_pallet.rs`：节点独立永久规则。
