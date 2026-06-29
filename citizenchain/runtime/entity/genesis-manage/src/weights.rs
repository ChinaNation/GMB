//! genesis-manage 权重占位。
//!
//! 中文注释：本 pallet 只在创世构建时写入创世机构档案，不提供外部 extrinsic。
//! 保留 WeightInfo trait 是为了与 runtime 其他 pallet 的配置形态保持一致。

pub trait WeightInfo {}

impl WeightInfo for () {}

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T> WeightInfo for SubstrateWeight<T> {}
