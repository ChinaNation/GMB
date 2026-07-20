use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
    fn publish_post() -> Weight;
    fn subscribe() -> Weight;
    fn cancel() -> Weight;
    fn set_creator_plans(tiers: u32) -> Weight;
    fn change_subscription_plan() -> Weight;
    fn propose_set_platform_price() -> Weight;
    /// 单笔到期续费（读价/收款方 + 转账 + 状态写 + 双向调度索引）；on_idle 按此估算每块可排空笔数。
    fn process_one_due() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn publish_post() -> Weight {
        Weight::from_parts(30_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(2, 2))
    }

    // 首扣：价格/CID/机构账户 + 转账 + Active 状态 + 双向调度索引。
    fn subscribe() -> Weight {
        Weight::from_parts(75_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(8, 6))
    }

    fn cancel() -> Weight {
        Weight::from_parts(25_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(3, 3))
    }

    fn set_creator_plans(tiers: u32) -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(Weight::from_parts(3_000_000, 0).saturating_mul(tiers.into()))
            .saturating_add(T::DbWeight::get().reads_writes(2, 1))
    }

    fn change_subscription_plan() -> Weight {
        Weight::from_parts(78_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(8, 6))
    }

    fn propose_set_platform_price() -> Weight {
        // 平台调价现按岗位权限构造 VotePlan；正式全调用 benchmark 补齐前，
        // 以已实测机构提案路径为基线取保守上界。
        Weight::from_parts(400_000_000, 700_000)
            .saturating_add(T::DbWeight::get().reads_writes(35, 30))
    }

    fn process_one_due() -> Weight {
        Weight::from_parts(85_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(8, 7))
    }
}

impl WeightInfo for () {
    fn publish_post() -> Weight {
        Weight::zero()
    }

    fn subscribe() -> Weight {
        Weight::zero()
    }

    fn cancel() -> Weight {
        Weight::zero()
    }

    fn set_creator_plans(_tiers: u32) -> Weight {
        Weight::zero()
    }

    fn change_subscription_plan() -> Weight {
        Weight::zero()
    }

    fn propose_set_platform_price() -> Weight {
        Weight::from_parts(400_000_000, 700_000).saturating_add(
            frame_support::weights::constants::RocksDbWeight::get().reads_writes(35, 30),
        )
    }

    fn process_one_due() -> Weight {
        Weight::zero()
    }
}
