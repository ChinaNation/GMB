# votingengine 双层 ID + 反向索引(PR-Y)

- **日期**: 2026-05-04(代码就绪) / 2026-05-05(spec_version 维持 0,激活待定)
- **模块**: Blockchain Agent — `citizenchain/runtime/votingengine`
- **优先级**: 高
- **依赖**: PR-X(20260504-100000) 已完成
- **状态**: ⏸️ **代码就绪未激活** — `migrations::v1::MigrateToV1` 已实现,但
  `runtime/src/configs/mod.rs::SingleBlockMigrations` 改回空 tuple,
  `runtime/src/lib.rs::spec_version = 0` 维持不升级。
  激活时只需:
  1. 在 `SingleBlockMigrations` 加入 `(votingengine::migrations::v1::MigrateToV1<Runtime>,)`
  2. spec_version 0 → 1
  3. 走链上 `setCode` 升级
  4. 验证 `ProposalDisplayId` + 4 张反向索引完成存量回填

## 背景

PR-X 完成 `votingengine` 提级 + 拆分 + 0 行为变化重构。PR-Y 落地真正的"提案 ID 体系彻底改造":

1. **现状硬上限**:`year × 1_000_000 + counter`,counter ≤ 999_999,**1000 万/年目标第一天就爆**(对照用户拍板的产线规模)。
2. **客户端事后过滤是死路**:`AllProposalsView` 全量拉提案再客户端过滤多签管理类,被 PR-X 暂时缓解但根因(无反向索引)未除。
3. **架构不诚实**:展示号"年份"语义混在主键里,以后改格式必须 spec_version bump + 全栈跟随。

## 决策(已与 user 确认)

1. **主键纯单调 u64** — `NextProposalId` 自增,实质无上限(1.84×10¹⁹)。
2. **展示号单独存** — `ProposalDisplayId[u64] = ProposalDisplayMeta { year: u16, seq_in_year: u32 }`。
3. **`YearProposalCounter` cap 解除** — u32(42.9 亿/年),实质无上限。
4. **反向索引四张** — 按 org / institution / owner(MODULE_TAG) / year 索引。
5. **storage 迁移走 on_runtime_upgrade v1** — 回填存量 ProposalDisplayId + 四张反向索引。
6. **spec_version 0 → 1** — 走链上 setCode 升级。

## 改动面

### 链端

#### `votingengine/src/types.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ProposalDisplayMeta {
    pub year: u16,
    pub seq_in_year: u32,
}
```

#### `votingengine/src/lib.rs`

加 storage:

```rust
#[pallet::storage]
pub type ProposalDisplayId<T: Config> =
    StorageMap<_, Blake2_128Concat, u64, ProposalDisplayMeta, OptionQuery>;

#[pallet::storage]
pub type ProposalsByOrg<T: Config> =
    StorageDoubleMap<_, Twox64Concat, u8, Twox64Concat, u64, (), OptionQuery>;

#[pallet::storage]
pub type ProposalsByInstitution<T: Config> =
    StorageDoubleMap<_, Twox64Concat, InstitutionPalletId, Twox64Concat, u64, (), OptionQuery>;

#[pallet::storage]
pub type ProposalsByOwner<T: Config> =
    StorageDoubleMap<_, Twox64Concat, BoundedVec<u8, T::MaxModuleTagLen>, Twox64Concat, u64, (), OptionQuery>;

#[pallet::storage]
pub type ProposalsByYear<T: Config> =
    StorageDoubleMap<_, Twox64Concat, u16, Twox64Concat, u64, (), OptionQuery>;
```

#### `votingengine/src/id.rs`

`allocate_proposal_id` 改双层:

```rust
pub(crate) fn allocate_proposal_id() -> Result<u64, DispatchError> {
    let now_ms = T::TimeProvider::now().as_millis();
    let secs = u64::try_from(now_ms / 1000).map_err(|_| Error::<T>::ProposalIdOverflow)?;
    let year = Self::unix_seconds_to_year(secs)?;

    // 主键单调 u64
    let id = NextProposalId::<T>::mutate(|n| -> Result<u64, DispatchError> {
        let cur = *n;
        *n = n.checked_add(1).ok_or(Error::<T>::ProposalIdOverflow)?;
        Ok(cur)
    })?;

    // 年内累加(cap 解除)
    let stored_year = CurrentProposalYear::<T>::get();
    let seq_in_year = if stored_year != year {
        CurrentProposalYear::<T>::put(year);
        YearProposalCounter::<T>::put(1u32);
        0u32
    } else {
        let c = YearProposalCounter::<T>::get();
        YearProposalCounter::<T>::put(
            c.checked_add(1).ok_or(Error::<T>::YearCounterOverflow)?,
        );
        c
    };

    ProposalDisplayId::<T>::insert(id, ProposalDisplayMeta { year, seq_in_year });
    Ok(id)
}
```

#### `votingengine/src/index.rs`

实现 `register_indexes` / `release_indexes`:

```rust
impl<T: pallet::Config> pallet::Pallet<T> {
    pub(crate) fn register_proposal_indexes(
        proposal_id: u64,
        org: u8,
        institution: InstitutionPalletId,
        module_tag: BoundedVec<u8, T::MaxModuleTagLen>,
        year: u16,
    ) {
        ProposalsByOrg::<T>::insert(org, proposal_id, ());
        ProposalsByInstitution::<T>::insert(institution, proposal_id, ());
        ProposalsByOwner::<T>::insert(module_tag, proposal_id, ());
        ProposalsByYear::<T>::insert(year, proposal_id, ());
    }

    pub(crate) fn release_proposal_indexes(proposal_id: u64) {
        // 反查 Proposal 与 ProposalOwner 拿 (org, institution, module_tag),从四张索引删除
        // ...
    }
}
```

#### `votingengine/src/data.rs::register_proposal_data`

末尾追加索引写入。

#### `votingengine/src/cleanup.rs` + lib.rs cleanup paths

终态/90 天清理路径调 `release_proposal_indexes`。

#### `votingengine/src/migrations/v1.rs`

```rust
pub struct MigrateToV1<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut weight = Weight::zero();
        for (proposal_id, proposal) in Proposals::<T>::iter() {
            // 反推 (year, seq_in_year):旧格式 id = year × 1_000_000 + seq
            let year = (proposal_id / 1_000_000) as u16;
            let seq_in_year = (proposal_id % 1_000_000) as u32;
            ProposalDisplayId::<T>::insert(
                proposal_id,
                ProposalDisplayMeta { year, seq_in_year },
            );

            // 反向索引:org/institution 来自 Proposal,module_tag 来自 ProposalOwner
            if let Some(org) = proposal.internal_org {
                ProposalsByOrg::<T>::insert(org, proposal_id, ());
            }
            if let Some(inst) = proposal.internal_institution {
                ProposalsByInstitution::<T>::insert(inst, proposal_id, ());
            }
            if let Some(owner) = ProposalOwner::<T>::get(proposal_id) {
                ProposalsByOwner::<T>::insert(owner, proposal_id, ());
            }
            ProposalsByYear::<T>::insert(year, proposal_id, ());

            weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 5));
        }
        weight
    }
}
```

#### `runtime/src/lib.rs`

```rust
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_version: 1,  // ← bump from 0
    // ...
};

pub type Migrations = (votingengine::migrations::v1::MigrateToV1<Runtime>,);
```

### 客户端(本卡不做,留给 PR-Z)

- wuminapp `_loadFirstPage` 走反向索引而非全量扫
- 删 `id ~/ 1_000_000` 类硬编码
- 渲染 ID 走 `ProposalDisplayId` 查询
- Tauri 节点前端同步

## 实现步骤

1. types.rs 加 `ProposalDisplayMeta`
2. lib.rs 加 5 张 storage(`ProposalDisplayId` + 4 张反向索引)
3. id.rs 改 `allocate_proposal_id` 为双层 + 解 cap + 加 `YearCounterOverflow` Error
4. index.rs 实现 `register_proposal_indexes` / `release_proposal_indexes`
5. data.rs `register_proposal_data` 末尾加索引写入
6. cleanup.rs + lib.rs cleanup paths 调 `release_proposal_indexes`
7. migrations/v1.rs 实现存量回填
8. runtime/src/lib.rs spec_version bump + Migrations 注册
9. 加测试覆盖(主键单调 / counter 解 cap / 反向索引正确性 / migration round-trip)
10. cargo test + 9 pallet 全套验证

## 验收标准

- [ ] votingengine 现有 74 测试 + 新增 PR-Y 测试全过
- [ ] 9 个依赖 pallet 单元测试全过(245 + 新增)
- [ ] `--features runtime-benchmarks` / `--features try-runtime` 编译过
- [ ] `runtime/src/lib.rs` `spec_version = 1`
- [ ] `Migrations` tuple 注册了 `votingengine::migrations::v1::MigrateToV1`
- [ ] 链上 storage prefix `'VotingEngine'` 不动(客户端 0 改动)
- [ ] 工作树 grep `999_999` 在 votingengine 生产代码中 0 残留(原 cap 字面量解除)
- [ ] `allocate_proposal_id` 测试覆盖:连续 N 个调用主键严格 +1,跨年 `seq_in_year` 重置但主键仍递增

## 上下文加载

- memory/00-vision/project-goal.md
- memory/01-architecture/repo-map.md
- memory/01-architecture/citizenchain-target-structure.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/08-tasks/open/20260504-100000-citizenchain-votingengine-提级与目录拆分.md
- citizenchain/CITIZENCHAIN_TECHNICAL.md
