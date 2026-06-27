//! # 选举投票 pallet (election-vote)
//!
//! 选举公职人员的多模式入口:普选(公民选) + 互选(机构成员内部互选)。
//!
//! 与 [`joint-vote::jointreferendum`] 不同 — jointreferendum 是联合投票被否决后的
//! 联合公投(yes/no);election-vote 用于按公民宪法选举各类公职人员,选民集视职位
//! 而定(普选=全体认证公民快照,互选=特定机构现任成员快照)。
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
