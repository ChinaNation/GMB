/// - 投票新增省储行/省储会节点
mod proposal;
mod voting;
mod issuance;
mod utils;

impl<T: Config> Pallet<T> {
    // 对外暴露接口
}

#[pallet::config]
pub trait Config: frame_system::Config {
    type RuntimeEvent: From<Event<Self>>
        + IsType<<Self as frame_system::Config>::RuntimeEvent>;
}