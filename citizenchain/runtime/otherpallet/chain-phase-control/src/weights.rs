//! 链阶段控制模块 weight 定义。
//! 本 pallet 无 extrinsic、无 hooks，weight 为空壳。

/// Weight trait（目前无需任何函数）。
pub trait WeightInfo {}

/// 正式 runtime 使用的 weight 实现。
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {}

/// 测试用 weight 实现。
impl WeightInfo for () {}
