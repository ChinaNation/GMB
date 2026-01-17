/// - 公民轻节点发行模块，负责根据公民轻节点的认证，发放认证奖励。

#[pallet::config]
pub trait Config: frame_system::Config {
    type RuntimeEvent: From<Event<Self>>
        + IsType<<Self as frame_system::Config>::RuntimeEvent>;
}