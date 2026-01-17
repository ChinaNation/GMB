/// - 决议发行提案投票模块，负责提案的创建、投票操作
mod proposal; // 提案模块
mod voting; // 投票模块

#[pallet::config]
pub trait Config: frame_system::Config {
    type RuntimeEvent: From<Event<Self>>
        + IsType<<Self as frame_system::Config>::RuntimeEvent>;
}