//! `citizen-identity` FRAME benchmark。
//!
//! 身份写入使用 fresh spec-genesis 中真实 FRG 省专员岗位完成授权；人口维护四项按
//! 可独立组合的最重完整路径生成安全上界，避免日期推进或集中到期被低估。

#![cfg(feature = "runtime-benchmarks")]

use alloc::{format, vec, vec::Vec};

use frame_benchmarking::v2::*;
use frame_support::weights::Weight;
use frame_system::RawOrigin;

use crate::{
    pallet::{
        CidRegistry, Config, PopulationMaintenanceFault, PopulationReadyDate, VotingIdentityByCid,
        WalletAccountByCid,
    },
    AreaCodeBound, Call, CandidateIdentityPayload, CidNumberBound, CidOccupyItem,
    CidOccupyItemsBound, CidRecord, CidRecordStatus, CitizenIdentityAuthority, CitizenSex,
    CitizenStatus, FamilyName, GivenName, Pallet, RoleCodeBound, VotingIdentityPayload,
    MAX_CID_OCCUPY_BATCH,
};

const BENCHMARK_TIMESTAMP_MILLIS: u64 = 1_800_000_000_000;
const ONE_DAY_MILLIS: u64 = 86_400_000;

type Authority<AccountId> = (
    AccountId,
    CidNumberBound,
    RoleCodeBound,
    AreaCodeBound,
    AreaCodeBound,
);

fn authority<T: Config>() -> Authority<T::AccountId> {
    T::CitizenIdentityAuthority::benchmark_authority()
        .expect("runtime benchmark must provide a real registrar role subject")
}

fn set_time<T: Config>(timestamp_millis: u64) -> u32 {
    T::CitizenIdentityAuthority::benchmark_set_timestamp(timestamp_millis);
    let date = Pallet::<T>::current_date_int();
    assert_ne!(
        date, 0,
        "benchmark timestamp must resolve to a calendar date"
    );
    date
}

fn citizen_cid(tag: u32) -> CidNumberBound {
    primitives::cid::generator::generate_cid_number(
        primitives::cid::generator::GenerateCidNumberInput {
            account_pubkey: &format!("benchmark-{tag}"),
            p1: "1",
            province_code: "ZS",
            province_name: "中枢省",
            city_code: "001",
            city_name: "基准市",
            year: "2027",
            institution: "CTZN",
        },
    )
    .expect("benchmark citizen CID must satisfy the production CID protocol")
    .into_bytes()
    .try_into()
    .expect("benchmark citizen CID must fit the runtime bound")
}

fn signature<T: Config>() -> crate::pallet::SignatureOf<T> {
    vec![1u8; 64]
        .try_into()
        .expect("runtime signature bound must accept sr25519 length")
}

fn voting_payload<T: Config>(
    wallet_account: T::AccountId,
    cid_number: CidNumberBound,
    province: AreaCodeBound,
    city: AreaCodeBound,
    town: &[u8],
    valid_from: u32,
    valid_until: u32,
) -> VotingIdentityPayload<T::AccountId> {
    VotingIdentityPayload {
        cid_number,
        wallet_account,
        citizen_age_years: 18,
        passport_valid_from: valid_from,
        passport_valid_until: valid_until,
        citizen_status: CitizenStatus::Normal,
        residence_province_code: province,
        residence_city_code: city,
        residence_town_code: town.to_vec().try_into().expect("benchmark town code fits"),
    }
}

fn candidate_payload<T: Config>(
    voting: VotingIdentityPayload<T::AccountId>,
) -> CandidateIdentityPayload<T::AccountId> {
    CandidateIdentityPayload {
        birth_province_code: voting.residence_province_code.clone(),
        birth_city_code: voting.residence_city_code.clone(),
        birth_town_code: voting.residence_town_code.clone(),
        family_name: FamilyName::try_from("基准".as_bytes().to_vec())
            .expect("benchmark family name fits"),
        given_name: GivenName::try_from("公民".as_bytes().to_vec())
            .expect("benchmark given name fits"),
        citizen_sex: CitizenSex::Female,
        birth_date: 19900101,
        voting,
    }
}

fn seed_occupied<T: Config>(
    actor_cid_number: &CidNumberBound,
    cid_number: &CidNumberBound,
    province: &AreaCodeBound,
    city: &AreaCodeBound,
    tag: u32,
) {
    let mut commitment = [0u8; 32];
    commitment[..4].copy_from_slice(&tag.to_le_bytes());
    CidRegistry::<T>::insert(
        cid_number,
        CidRecord {
            registrar_cid_number: actor_cid_number.clone(),
            commitment,
            residence_province_code: province.clone(),
            residence_city_code: city.clone(),
            status: CidRecordStatus::Active,
            registered_at: frame_system::Pallet::<T>::block_number(),
            revoked_at: None,
        },
    );
}

fn setup_registration<T: Config>(
    tag: u32,
    valid_from: u32,
    valid_until: u32,
) -> (Authority<T::AccountId>, VotingIdentityPayload<T::AccountId>) {
    let authority = authority::<T>();
    let cid_number = citizen_cid(tag);
    seed_occupied::<T>(&authority.1, &cid_number, &authority.3, &authority.4, tag);
    let payload = voting_payload::<T>(
        account("citizen", tag, 0),
        cid_number,
        authority.3.clone(),
        authority.4.clone(),
        b"ZS01001",
        valid_from,
        valid_until,
    );
    (authority, payload)
}

fn register<T: Config>(
    authority: &Authority<T::AccountId>,
    payload: VotingIdentityPayload<T::AccountId>,
) {
    Pallet::<T>::register_voting_identity(
        RawOrigin::Signed(authority.0.clone()).into(),
        authority.1.clone(),
        authority.2.clone(),
        payload,
        signature::<T>(),
    )
    .expect("benchmark voting identity registration must succeed");
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_voting_identity() {
        let today = set_time::<T>(BENCHMARK_TIMESTAMP_MILLIS);
        PopulationReadyDate::<T>::put(today);
        let (authority, payload) = setup_registration::<T>(1, today, 20991231);
        let cid_number = payload.cid_number.clone();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(authority.0),
            authority.1,
            authority.2,
            payload,
            signature::<T>(),
        );

        assert!(VotingIdentityByCid::<T>::contains_key(cid_number));
    }

    #[benchmark]
    fn upgrade_to_candidate_identity() {
        let today = set_time::<T>(BENCHMARK_TIMESTAMP_MILLIS);
        PopulationReadyDate::<T>::put(today);
        let (authority, voting) = setup_registration::<T>(2, today, 20991231);
        register::<T>(&authority, voting.clone());
        let payload = candidate_payload::<T>(voting);
        let cid_number = payload.voting.cid_number.clone();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(authority.0),
            authority.1,
            authority.2,
            payload,
            signature::<T>(),
        );

        assert!(crate::pallet::CandidateIdentityByCid::<T>::contains_key(
            cid_number
        ));
    }

    #[benchmark]
    fn update_voting_identity() {
        let today = set_time::<T>(BENCHMARK_TIMESTAMP_MILLIS);
        PopulationReadyDate::<T>::put(today);
        let (authority, initial) = setup_registration::<T>(3, today, 20991231);
        register::<T>(&authority, initial.clone());
        let mut payload = initial;
        payload.residence_town_code = b"ZS01002".to_vec().try_into().expect("town code fits");
        let cid_number = payload.cid_number.clone();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(authority.0),
            authority.1,
            authority.2,
            payload,
            signature::<T>(),
        );

        assert!(VotingIdentityByCid::<T>::contains_key(cid_number));
    }

    #[benchmark]
    fn update_candidate_identity() {
        let today = set_time::<T>(BENCHMARK_TIMESTAMP_MILLIS);
        PopulationReadyDate::<T>::put(today);
        let (authority, voting) = setup_registration::<T>(4, today, 20991231);
        register::<T>(&authority, voting.clone());
        let initial = candidate_payload::<T>(voting);
        Pallet::<T>::upgrade_to_candidate_identity(
            RawOrigin::Signed(authority.0.clone()).into(),
            authority.1.clone(),
            authority.2.clone(),
            initial.clone(),
            signature::<T>(),
        )
        .expect("benchmark candidate upgrade must succeed");
        let mut payload = initial;
        payload.voting.residence_town_code =
            b"ZS01002".to_vec().try_into().expect("town code fits");
        let cid_number = payload.voting.cid_number.clone();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(authority.0),
            authority.1,
            authority.2,
            payload,
            signature::<T>(),
        );

        assert!(crate::pallet::CandidateIdentityByCid::<T>::contains_key(
            cid_number
        ));
    }

    #[benchmark]
    fn revoke_identity() {
        let today = set_time::<T>(BENCHMARK_TIMESTAMP_MILLIS);
        PopulationReadyDate::<T>::put(today);
        let (authority, voting) = setup_registration::<T>(5, today, 20991231);
        let cid_number = voting.cid_number.clone();
        register::<T>(&authority, voting.clone());
        let candidate = candidate_payload::<T>(voting);
        Pallet::<T>::upgrade_to_candidate_identity(
            RawOrigin::Signed(authority.0.clone()).into(),
            authority.1.clone(),
            authority.2.clone(),
            candidate,
            signature::<T>(),
        )
        .expect("benchmark candidate upgrade must succeed");

        #[extrinsic_call]
        _(
            RawOrigin::Signed(authority.0),
            authority.1,
            authority.2,
            cid_number.clone(),
        );

        assert_eq!(
            CidRegistry::<T>::get(cid_number).map(|record| record.status),
            Some(CidRecordStatus::Revoked)
        );
    }

    #[benchmark]
    fn occupy_cid() {
        let authority = authority::<T>();
        let cid_number = citizen_cid(6);

        #[extrinsic_call]
        _(
            RawOrigin::Signed(authority.0),
            authority.1,
            authority.2,
            cid_number.clone(),
            [6u8; 32],
            authority.3,
            authority.4,
        );

        assert!(CidRegistry::<T>::contains_key(cid_number));
    }

    #[benchmark]
    fn occupy_cids_batch(n: Linear<1, MAX_CID_OCCUPY_BATCH>) {
        let authority = authority::<T>();
        let items: Vec<CidOccupyItem> = (0..n)
            .map(|index| CidOccupyItem {
                cid_number: citizen_cid(10_000 + index),
                commitment: [index as u8; 32],
            })
            .collect();
        let first_cid = items[0].cid_number.clone();
        let items: CidOccupyItemsBound = items.try_into().expect("benchmark batch fits");

        #[extrinsic_call]
        _(
            RawOrigin::Signed(authority.0),
            authority.1,
            authority.2,
            items,
            authority.3,
            authority.4,
        );

        assert!(CidRegistry::<T>::contains_key(first_cid));
    }

    #[benchmark]
    fn revoke_cid() {
        let today = set_time::<T>(BENCHMARK_TIMESTAMP_MILLIS);
        PopulationReadyDate::<T>::put(today);
        let (authority, voting) = setup_registration::<T>(7, today, 20991231);
        let cid_number = voting.cid_number.clone();
        register::<T>(&authority, voting);

        #[extrinsic_call]
        _(
            RawOrigin::Signed(authority.0),
            authority.1,
            authority.2,
            cid_number.clone(),
        );

        assert_eq!(
            CidRegistry::<T>::get(cid_number).map(|record| record.status),
            Some(CidRecordStatus::Revoked)
        );
    }

    #[benchmark]
    fn population_maintenance_base() {
        set_time::<T>(BENCHMARK_TIMESTAMP_MILLIS);
        PopulationMaintenanceFault::<T>::put(crate::PopulationFault::InvalidReadyDate);

        #[block]
        {
            Pallet::<T>::process_population_maintenance(Weight::MAX);
        }

        assert!(PopulationMaintenanceFault::<T>::get().is_some());
    }

    #[benchmark]
    fn initialize_population_date() {
        let today = set_time::<T>(BENCHMARK_TIMESTAMP_MILLIS);
        PopulationReadyDate::<T>::kill();

        #[block]
        {
            Pallet::<T>::process_population_maintenance(Weight::MAX);
        }

        assert_eq!(PopulationReadyDate::<T>::get(), today);
    }

    #[benchmark]
    fn advance_population_day() {
        let yesterday = set_time::<T>(BENCHMARK_TIMESTAMP_MILLIS - ONE_DAY_MILLIS);
        PopulationReadyDate::<T>::put(yesterday);
        let today = set_time::<T>(BENCHMARK_TIMESTAMP_MILLIS);

        #[block]
        {
            Pallet::<T>::process_population_maintenance(Weight::MAX);
        }

        assert_eq!(PopulationReadyDate::<T>::get(), today);
    }

    #[benchmark]
    fn process_population_transition() {
        let yesterday = set_time::<T>(BENCHMARK_TIMESTAMP_MILLIS - ONE_DAY_MILLIS);
        PopulationReadyDate::<T>::put(yesterday);
        let (authority, voting) = setup_registration::<T>(8, 20000101, yesterday);
        let cid_number = voting.cid_number.clone();
        register::<T>(&authority, voting);
        let today = set_time::<T>(BENCHMARK_TIMESTAMP_MILLIS);

        #[block]
        {
            Pallet::<T>::process_population_maintenance(Weight::MAX);
        }

        assert_eq!(PopulationReadyDate::<T>::get(), today);
        assert!(WalletAccountByCid::<T>::contains_key(cid_number));
    }
}
