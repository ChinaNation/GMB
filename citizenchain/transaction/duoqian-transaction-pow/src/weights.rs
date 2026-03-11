//! 多签交易模块权重定义。
//!
//! 当前为保守手动估算值，后续应由 `frame-benchmarking` 自动生成替换。

use frame_support::weights::Weight;

/// 权重接口：由 runtime 注入实现。
pub trait WeightInfo {
    /// 机构登记开销主要来自双向映射写入与地址派生。
    fn register_sfid_institution() -> Weight;
    /// 创建开销随管理员数量和提交的审批签名数量线性增长。
    fn create_duoqian(admin_count: u32, approval_count: u32) -> Weight;
    /// 注销开销同样受管理员数量和审批签名数量影响。
    fn close_duoqian(admin_count: u32, approval_count: u32) -> Weight;
}

/// 默认保守估算实现。
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register_sfid_institution() -> Weight {
        Weight::from_parts(40_000_000, 1_024)
    }

    fn create_duoqian(admin_count: u32, approval_count: u32) -> Weight {
        Weight::from_parts(120_000_000, 4_096)
            .saturating_add(Weight::from_parts(8_000_000, 128).saturating_mul(admin_count as u64))
            .saturating_add(
                Weight::from_parts(25_000_000, 256).saturating_mul(approval_count as u64),
            )
    }

    fn close_duoqian(admin_count: u32, approval_count: u32) -> Weight {
        Weight::from_parts(95_000_000, 3_072)
            .saturating_add(Weight::from_parts(8_000_000, 128).saturating_mul(admin_count as u64))
            .saturating_add(
                Weight::from_parts(25_000_000, 256).saturating_mul(approval_count as u64),
            )
    }
}

/// 单元测试用实现。
impl WeightInfo for () {
    fn register_sfid_institution() -> Weight {
        Weight::from_parts(40_000_000, 1_024)
    }

    fn create_duoqian(admin_count: u32, approval_count: u32) -> Weight {
        Weight::from_parts(120_000_000, 4_096)
            .saturating_add(Weight::from_parts(8_000_000, 128).saturating_mul(admin_count as u64))
            .saturating_add(
                Weight::from_parts(25_000_000, 256).saturating_mul(approval_count as u64),
            )
    }

    fn close_duoqian(admin_count: u32, approval_count: u32) -> Weight {
        Weight::from_parts(95_000_000, 3_072)
            .saturating_add(Weight::from_parts(8_000_000, 128).saturating_mul(admin_count as u64))
            .saturating_add(
                Weight::from_parts(25_000_000, 256).saturating_mul(approval_count as u64),
            )
    }
}
