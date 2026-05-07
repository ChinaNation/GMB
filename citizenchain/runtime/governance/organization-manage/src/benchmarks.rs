//! 多签交易模块 Benchmark 定义。
//!
//! 投票统一走 `votingengine::internal_vote`,本模块不承担投票/聚合 extrinsic。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;
use sp_std::vec;
use votingengine::STATUS_PASSED;

use crate::{
    pallet::{
        AccountNameOf, AddressRegisteredSfid, DuoqianAdminsOf, InstitutionAccountNamesOf,
        RegisterNonceOf, RegisterSignatureOf, SfidIdOf, SfidRegisteredAddress,
    },
    BalanceOf, Call, Config, DuoqianAddressValidator, DuoqianReservedAddressChecker, Pallet,
    ProtectedSourceChecker,
};

fn find_safe_sfid<T: Config>() -> Result<(SfidIdOf<T>, T::AccountId), BenchmarkError> {
    for candidate in 0..2_048u32 {
        let mut raw = b"duoqian-benchmark-sfid-".to_vec();
        raw.extend_from_slice(&candidate.to_le_bytes());
        let sfid_id: SfidIdOf<T> = raw
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark sfid id should fit"))?;

        // benchmark 场景用 Role::Main 派生，哈希公式等价于历史空 account_name 路径。
        let Ok(duoqian_address) = Pallet::<T>::derive_institution_address(
            sfid_id.as_slice(),
            crate::InstitutionAccountRole::Main,
        ) else {
            continue;
        };

        if T::ReservedAddressChecker::is_reserved(&duoqian_address) {
            continue;
        }
        if T::ProtectedSourceChecker::is_protected(&duoqian_address) {
            continue;
        }
        if !T::AddressValidator::is_valid(&duoqian_address) {
            continue;
        }

        return Ok((sfid_id, duoqian_address));
    }

    Err(BenchmarkError::Stop(
        "failed to find a benchmark-safe sfid id",
    ))
}

fn bench_account_name<T: Config>() -> Result<AccountNameOf<T>, BenchmarkError> {
    b"Benchmark Institution"
        .to_vec()
        .try_into()
        .map_err(|_| BenchmarkError::Stop("benchmark account_name should fit"))
}

fn register_institution<T: Config>(
    relayer: &T::AccountId,
    sfid_id: &SfidIdOf<T>,
) -> Result<T::AccountId, BenchmarkError> {
    let account_name = bench_account_name::<T>()?;
    let register_nonce: RegisterNonceOf<T> = b"bench-register-nonce"
        .to_vec()
        .try_into()
        .map_err(|_| BenchmarkError::Stop("benchmark register nonce should fit"))?;
    let signature: RegisterSignatureOf<T> = vec![1u8; 64]
        .try_into()
        .map_err(|_| BenchmarkError::Stop("benchmark register signature should fit"))?;
    let account_names: InstitutionAccountNamesOf<T> = vec![account_name.clone()]
        .try_into()
        .map_err(|_| BenchmarkError::Stop("benchmark account_names should fit"))?;
    Pallet::<T>::register_sfid_institution(
        RawOrigin::Signed(relayer.clone()).into(),
        sfid_id.clone(),
        account_name.clone(),
        account_names,
        register_nonce,
        signature,
        b"LN".to_vec(),
        [1u8; 32],
    )?;
    SfidRegisteredAddress::<T>::get(sfid_id, &account_name)
        .ok_or(BenchmarkError::Stop("benchmark sfid should be registered"))
}

fn find_safe_beneficiary<T: Config>(
    duoqian_address: &T::AccountId,
) -> Result<T::AccountId, BenchmarkError> {
    for index in 0..64u32 {
        let beneficiary: T::AccountId = frame_benchmarking::account("beneficiary", index, 0);
        if &beneficiary == duoqian_address {
            continue;
        }
        if T::ReservedAddressChecker::is_reserved(&beneficiary) {
            continue;
        }
        if T::ProtectedSourceChecker::is_protected(&beneficiary) {
            continue;
        }
        if !T::AddressValidator::is_valid(&beneficiary) {
            continue;
        }
        return Ok(beneficiary);
    }

    Err(BenchmarkError::Stop(
        "failed to find a benchmark-safe beneficiary",
    ))
}

/// Benchmark 辅助:让指定提案通过(绕开投票路径,benchmark 只关心后续业务执行开销)。
fn pass_proposal<T: Config>(proposal_id: u64) -> Result<(), BenchmarkError> {
    votingengine::Proposals::<T>::mutate(proposal_id, |maybe| {
        if let Some(proposal) = maybe {
            proposal.status = STATUS_PASSED;
        }
    });
    let now = frame_system::Pallet::<T>::block_number();
    votingengine::ProposalExecutionRetryStates::<T>::insert(
        proposal_id,
        votingengine::ExecutionRetryState {
            manual_attempts: 0,
            first_auto_failed_at: now,
            retry_deadline: now,
            last_attempt_at: None,
        },
    );
    Ok(())
}

#[benchmarks(where
    T: Config,
    <T as frame_system::Config>::AccountId: Decode,
    BalanceOf<T>: Ord + sp_runtime::traits::Saturating + Copy,
)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_sfid_institution() -> Result<(), BenchmarkError> {
        let relayer: T::AccountId = frame_benchmarking::account("relayer", 0, 0);

        let (sfid_id, duoqian_address) = find_safe_sfid::<T>()?;
        let account_name = bench_account_name::<T>()?;
        let register_nonce: RegisterNonceOf<T> = b"bench-register-nonce"
            .to_vec()
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark register nonce should fit"))?;
        let signature: RegisterSignatureOf<T> = vec![1u8; 64]
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark register signature should fit"))?;
        let account_names: InstitutionAccountNamesOf<T> = vec![account_name.clone()]
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark account_names should fit"))?;

        #[extrinsic_call]
        register_sfid_institution(
            RawOrigin::Signed(relayer.clone()),
            sfid_id.clone(),
            account_name.clone(),
            account_names,
            register_nonce,
            signature,
            b"LN".to_vec(),
            [1u8; 32],
        );

        assert_eq!(
            SfidRegisteredAddress::<T>::get(&sfid_id, &account_name),
            Some(duoqian_address.clone())
        );
        assert!(AddressRegisteredSfid::<T>::contains_key(&duoqian_address));
        Ok(())
    }

    // propose_create (call_index=0) benchmark 已废弃 (2026-05-03):
    // 单账户机构创建入口已删除,机构最少 2 账户走 propose_create_institution。
    //
    // propose_close benchmark + propose_create_personal benchmark 已在 B 阶段
    // (personal-manage 拆分,2026-05-06) 删除:
    // - propose_close 原 setup 依赖个人多签;机构 propose_close 的 benchmark 重写
    //   需要完整的 register_sfid_institution + propose_create_institution + pass
    //   流水线,留待 follow-up 任务卡补齐。
    // - propose_create_personal 已迁至 personal-manage(benchmark 用例 follow-up)。
    // 当前 organization-manage 仅保留 register_sfid_institution + propose_create_institution
    // + cleanup_rejected_proposal 三个 benchmark;CI 运行时影响范围与 weights.rs 占位等价。
}
