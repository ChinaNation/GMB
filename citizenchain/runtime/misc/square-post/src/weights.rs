use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
    fn publish_post() -> Weight;
    fn subscribe() -> Weight;
    fn cancel() -> Weight;
    fn charge_due() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn publish_post() -> Weight {
        Weight::from_parts(30_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(2, 2))
    }

    // 订阅=首扣：读价/CID/机构账户 + 转账 + 写订阅态。占位权重，第4步基准替换。
    fn subscribe() -> Weight {
        Weight::from_parts(45_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(4, 3))
    }

    fn cancel() -> Weight {
        Weight::from_parts(20_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(1, 1))
    }

    // 续扣：读订阅态 + 现读价/收款方 + 转账 + 写订阅态。占位权重，第4步基准替换。
    fn charge_due() -> Weight {
        Weight::from_parts(45_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(5, 3))
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

    fn charge_due() -> Weight {
        Weight::zero()
    }
}
