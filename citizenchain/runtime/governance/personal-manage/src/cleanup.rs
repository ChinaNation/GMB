//! cleanup_rejected_proposal extrinsic 的业务体。
//!
//! 用于解决投票引擎 on_initialize 超时 reject 后,本模块无法自动收到通知导致
//! Pending(PersonalDuoqians / PendingPersonalCreate / PendingCloseProposal)残留。
//! 任意签名账户均可调用,但仅对 status == STATUS_REJECTED 的提案生效。

use codec::Decode;
use frame_support::ensure;
use sp_runtime::DispatchResult;

use crate::pallet::{
    CloseDuoqianActionOf, Config, CreateDuoqianActionOf, Error, PendingCloseProposal,
};
use crate::ACTION_CLOSE;
use crate::ACTION_CREATE;
use votingengine::STATUS_REJECTED;

pub(crate) fn do_cleanup_rejected_proposal<T: Config>(proposal_id: u64) -> DispatchResult {
    // 读取提案数据,校验 MODULE_TAG 后判断操作类型
    let raw = votingengine::Pallet::<T>::get_proposal_data(proposal_id)
        .ok_or(Error::<T>::ProposalActionNotFound)?;
    let tag = crate::MODULE_TAG;
    ensure!(
        raw.len() > tag.len() && &raw[..tag.len()] == tag,
        Error::<T>::ProposalActionNotFound
    );
    let action_tag = raw[tag.len()];

    // 校验投票引擎状态必须为 REJECTED
    let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
        .ok_or(Error::<T>::ProposalActionNotFound)?;
    ensure!(
        proposal.status == STATUS_REJECTED,
        Error::<T>::ProposalNotRejected
    );

    match action_tag {
        ACTION_CREATE => {
            let action = CreateDuoqianActionOf::<T>::decode(&mut &raw[tag.len() + 1..])
                .map_err(|_| Error::<T>::ProposalActionNotFound)?;
            crate::execute::cleanup_pending_create::<T>(proposal_id, &action, true);
        }
        ACTION_CLOSE => {
            let action = CloseDuoqianActionOf::<T>::decode(&mut &raw[tag.len() + 1..])
                .map_err(|_| Error::<T>::ProposalActionNotFound)?;
            PendingCloseProposal::<T>::remove(&action.duoqian_address);
        }
        _ => return Err(Error::<T>::ProposalActionNotFound.into()),
    }

    Ok(())
}
