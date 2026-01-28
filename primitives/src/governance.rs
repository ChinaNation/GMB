//! 投票治理核心常量（国储会、省储会、省储行、公投）

/// 国储会管理员数量（固定：19）
pub const NRC_ADMIN_COUNT: u32 = 19;

/// 国储会阈值（>=13 管理员同意）
pub const NRC_THRESHOLD: u32 = 13;

/// 每个省储会管理员数量（固定：9）
pub const PRC_ADMIN_COUNT: u32 = 9;

/// 省储会阈值（>=6 管理员同意）
pub const PRC_THRESHOLD: u32 = 6;

/// 每个省储行管理员数量（固定：9）
pub const PRB_ADMIN_COUNT: u32 = 9;

/// 省储行阈值（>=6 管理员同意）
pub const PRB_THRESHOLD: u32 = 6;

/// 公投需要超过 50% 的公民轻节点投票同意
pub const CITIZEN_VOTE_PASS_PERCENT: u32 = 50;