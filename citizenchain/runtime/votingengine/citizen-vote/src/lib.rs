//! # 公民投票 pallet (citizen-vote)
//!
//! 公民投票多模式入口(选举 / 公投 / 审批投票 / RCV / ...)。
//!
//! 与 [`joint-vote::jointreferendum`] 不同 — jointreferendum 是联合投票被否决后的
//! 联合公投(yes/no),citizen-vote 是公民个人为公共事务发起的多候选 / 多模式选举。
//!
//! 本 pallet 当前仅占位骨架,具体投票模式待接入。

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::event]
    pub enum Event<T: Config> {
        /// 占位事件,待替换为真正的 ElectionVoteCast / ReferendumVoteCast 等。
        Placeholder { who: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 占位错误,待替换为真正的模式 specific 错误。
        Placeholder,
    }
}
