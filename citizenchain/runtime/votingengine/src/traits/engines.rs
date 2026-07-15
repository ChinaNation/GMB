//! 业务模块创建内部或联合投票提案的统一引擎入口。

use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;

use crate::types::InstitutionCode;

pub trait JointVoteEngine<AccountId> {
    fn create_joint_proposal(who: AccountId) -> Result<u64, DispatchError>;

    fn create_joint_proposal_with_data(
        who: AccountId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError>;

    fn create_joint_proposal_with_data_and_object(
        who: AccountId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
        object_kind: u8,
        object_data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError>;
}

impl<AccountId> JointVoteEngine<AccountId> for () {
    fn create_joint_proposal(_who: AccountId) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("JointVoteEngineNotConfigured"))
    }

    fn create_joint_proposal_with_data(
        _who: AccountId,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("JointVoteEngineNotConfigured"))
    }

    fn create_joint_proposal_with_data_and_object(
        _who: AccountId,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
        _object_kind: u8,
        _object_data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("JointVoteEngineNotConfigured"))
    }
}

/// 事项模块接入内部投票时,统一由投票引擎创建提案并返回真实提案 ID。
///
/// 内部投票是所有机构共用的投票程序，不代表所有机构都能发起每一种业务。
/// 投票引擎负责内部投票模式准入，转账、销毁、密钥变更等具体权限由对应业务模块
/// 校验；只有模式准入与业务权限同时通过，提案才可创建并执行。
///
/// 业务模块只能选择“提案语义”，不能传入“本次投票通过阈值”。
/// 阈值读取、快照、计票、自动赞成票与通过/否决判定全部归属投票引擎。
pub trait InternalVoteEngine<AccountId> {
    /// 创建一般内部投票提案。用于转账、销毁、GRANDPA key 更换等普通业务。
    fn create_general_internal_proposal_with_data(
        who: AccountId,
        institution_code: InstitutionCode,
        institution: AccountId,
        subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError>;

    /// 创建注册/注销生命周期内部投票提案。用于注销个人多签和机构多签。
    ///
    /// 生命周期投票由投票引擎按 active 管理员快照写入全员通过阈值。
    fn create_lifecycle_internal_proposal_with_data(
        _who: AccountId,
        _institution_code: InstitutionCode,
        _institution: AccountId,
        _subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("LifecycleVoteEngineNotConfigured"))
    }

    /// 创建注册个人多签/机构多签的特别内部投票提案。
    ///
    /// `dynamic_threshold` 是注册后普通业务使用的动态阈值配置，不是本次注册投票阈值。
    /// 本次注册投票阈值由投票引擎按 `admins.len()` 写全员通过快照。
    fn create_registered_account_create_proposal_with_data(
        _who: AccountId,
        _institution_code: InstitutionCode,
        _institution: AccountId,
        _subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
        _admins: sp_std::vec::Vec<AccountId>,
        _dynamic_threshold: u32,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other(
            "RegisteredAccountCreateVoteEngineNotConfigured",
        ))
    }

    /// 创建管理员集合变更内部投票提案。只允许 admins 模块 模块接入。
    ///
    /// 本次投票仍使用当前 active 阈值；`new_threshold` 只表示变更执行成功后
    /// 写入投票引擎的下一阶段动态阈值。
    fn create_admin_change_internal_proposal_with_data(
        _who: AccountId,
        _institution_code: InstitutionCode,
        _institution: AccountId,
        _subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
        _new_admins_len: u32,
        _new_threshold: u32,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other(
            "AdminSetMutationVoteEngineNotConfigured",
        ))
    }

    /// 特权直设动态阈值:绕过注册/变更提案,直接写入已激活动态阈值。
    ///
    /// 仅供 admins 模块在"联邦注册局直设市注册局管理员"(Step3 去中心化鉴权)时
    /// 同步阈值用。实现方必须按严格过半规则校验 `(admins_len, threshold)` 后写入,
    /// 失败回滚由调用方事务统一处理。默认未配置。
    fn register_active_dynamic_threshold_direct(
        _institution_code: InstitutionCode,
        _institution: AccountId,
        _admins_len: u32,
        _threshold: u32,
    ) -> DispatchResult {
        Err(DispatchError::Other(
            "RegisterActiveDynamicThresholdDirectNotConfigured",
        ))
    }

    /// 读取已激活动态阈值。只用于展示和业务事件，不参与业务模块计票。
    fn active_dynamic_threshold(
        _institution_code: InstitutionCode,
        _institution: AccountId,
    ) -> Option<u32> {
        None
    }

    /// 读取指定提案的 pending 阈值；不存在时再读取主体 active 阈值。
    /// 注册业务回调在核心提交执行成功副作用前发事件时使用。
    fn configured_dynamic_threshold(
        _proposal_id: u64,
        _institution_code: InstitutionCode,
        _institution: AccountId,
    ) -> Option<u32> {
        None
    }
}

impl<AccountId> InternalVoteEngine<AccountId> for () {
    fn create_general_internal_proposal_with_data(
        _who: AccountId,
        _institution_code: InstitutionCode,
        _institution: AccountId,
        _subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("InternalVoteEngineNotConfigured"))
    }
}
