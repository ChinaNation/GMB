use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
    fn publish_square_post() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn publish_square_post() -> Weight {
        Weight::from_parts(30_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(2, 2))
    }
}

impl WeightInfo for () {
    fn publish_square_post() -> Weight {
        Weight::zero()
    }
}
