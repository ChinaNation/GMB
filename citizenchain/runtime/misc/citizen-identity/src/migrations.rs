//! citizen-identity 的 runtime 升级迁移。

use crate::{CidCount, CidRecordStatus, CidRegistry, Config};
use core::marker::PhantomData;
use frame_support::{
    traits::{Get, OnRuntimeUpgrade},
    weights::Weight,
};

/// 回填 [`CidCount`]：按 `CidRegistry` 里的 Active 记录数一次性写入当前有效 CID 数。
///
/// `CidCount` 随本次升级新增。不回填的话它以 `ValueQuery` 默认值 0 起步，升级前已经占号的
/// CID 全部漏计。墓碑（`Revoked`）不计入，与占号 +1 / 吊销 −1 的口径一致。
/// 迁移只在升级那一个区块执行一次，不进常规出块路径。
pub struct InitCidCount<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for InitCidCount<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut scanned: u64 = 0;
        let mut active: u64 = 0;
        for record in CidRegistry::<T>::iter_values() {
            scanned = scanned.saturating_add(1);
            if record.status == CidRecordStatus::Active {
                active = active.saturating_add(1);
            }
        }
        CidCount::<T>::put(active);
        // 读 = 逐条登记记录，写 = CidCount 单值。
        <T as frame_system::Config>::DbWeight::get().reads_writes(scanned, 1)
    }
}
