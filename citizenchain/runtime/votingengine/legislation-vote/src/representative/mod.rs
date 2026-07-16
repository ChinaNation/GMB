//! 代表机构表决机制。

pub mod sequential;
pub mod single;
pub mod tally;
use crate::*;

impl<T: Config> Pallet<T> {
    pub(crate) fn stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        use sp_runtime::traits::SaturatedConversion;
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    /// 机构码只允许由 CID 解析，调用载荷不得另带一份可冲突的身份字段。
    pub(crate) fn institution_code_for_cid(
        cid_number: &votingengine::types::CidNumber,
    ) -> Result<InstitutionCode, DispatchError> {
        let text = core::str::from_utf8(cid_number.as_slice())
            .map_err(|_| Error::<T>::InvalidInstitutionCid)?;
        votingengine::types::institution_code_from_cid_number(text)
            .ok_or_else(|| Error::<T>::InvalidInstitutionCid.into())
    }

    /// 特别案人口作用域只由发起机构 CID 推导，调用载荷不得再携带第二份 scope。
    pub(crate) fn population_scope_for_actor(
        actor_code: InstitutionCode,
        actor_cid_number: &votingengine::types::CidNumber,
    ) -> Result<PopulationScope, DispatchError> {
        match actor_code {
            code if code == *b"NRP\0" || code == *b"NED\0" => Ok(PopulationScope::Country),
            code if code == *b"PRP\0" => {
                let (province_code, _) =
                    primitives::cid::number::cid_scope_codes(actor_cid_number.as_slice())
                        .map_err(|_| Error::<T>::InvalidInstitutionCid)?;
                let province_code = province_code
                    .to_vec()
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidInstitutionCid)?;
                Ok(PopulationScope::Province(province_code))
            }
            code if code == *b"CSLF" || code == *b"CEDU" => {
                let (province_code, city_code) =
                    primitives::cid::number::cid_scope_codes(actor_cid_number.as_slice())
                        .map_err(|_| Error::<T>::InvalidInstitutionCid)?;
                let province_code = province_code
                    .to_vec()
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidInstitutionCid)?;
                let city_code = city_code
                    .to_vec()
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidInstitutionCid)?;
                Ok(PopulationScope::City(province_code, city_code))
            }
            _ => Err(Error::<T>::InvalidInstitutionCid.into()),
        }
    }

    pub(crate) fn resolve_subject_cid_numbers(
        actor_cid_number: &votingengine::types::CidNumber,
        route: &RepresentativeRoute,
        additional_subjects: ProposalSubjectCidNumbers,
        additional_institutions: &[votingengine::types::CidNumber],
    ) -> Result<ProposalSubjectCidNumbers, DispatchError> {
        let mut raw: sp_runtime::sp_std::vec::Vec<sp_runtime::sp_std::vec::Vec<u8>> =
            additional_subjects
                .into_iter()
                .map(|cid| cid.into_inner())
                .collect();
        raw.push(actor_cid_number.to_vec());
        for cid_number in route.bodies() {
            raw.push(cid_number.to_vec());
        }
        for cid_number in additional_institutions {
            raw.push(cid_number.to_vec());
        }
        <votingengine::Pallet<T>>::bound_subject_cid_numbers(raw)
    }

    /// 校验路线并返回首个表决机构。路线中的机构不得重复。
    pub(crate) fn validate_representative_route(
        route: &RepresentativeRoute,
    ) -> Result<(InstitutionCode, votingengine::types::CidNumber), DispatchError> {
        let bodies = route.bodies();
        ensure!(!bodies.is_empty(), Error::<T>::InvalidRepresentativeRoute);
        match route {
            RepresentativeRoute::Single(_) => {}
            RepresentativeRoute::Sequential(sequence) => ensure!(
                sequence.len() >= 2 && sequence.len() <= MAX_REPRESENTATIVE_BODIES as usize,
                Error::<T>::InvalidRepresentativeRoute
            ),
        }
        for (index, body) in bodies.iter().enumerate() {
            ensure!(
                !bodies[..index].iter().any(|existing| existing == body),
                Error::<T>::InvalidRepresentativeRoute
            );
        }
        for body in &bodies {
            Self::institution_code_for_cid(body)?;
        }
        let first_cid_number = bodies[0].clone();
        Ok((
            Self::institution_code_for_cid(&first_cid_number)?,
            first_cid_number,
        ))
    }
}

impl<T: Config> Pallet<T> {
    /// 创建通用代表机构表决提案。业务模块只提供路线、门槛、受影响主体和 owner 数据。
    #[allow(clippy::too_many_arguments)]
    pub fn do_create_representative_proposal(
        who: T::AccountId,
        actor_cid_number: votingengine::types::CidNumber,
        route: RepresentativeRoute,
        rule: RepresentativeVoteRule,
        procedure: VoteProcedure,
        additional_subjects: ProposalSubjectCidNumbers,
        legislation_meta: Option<pallet::LegislationMeta>,
    ) -> Result<u64, DispatchError> {
        let (first_code, first_cid_number) = Self::validate_representative_route(&route)?;
        let actor_code = Self::institution_code_for_cid(&actor_cid_number)?;
        ensure!(
            <T as votingengine::Config>::InternalAdminProvider::is_institution_admin(
                actor_code,
                actor_cid_number.as_slice(),
                &who,
            ),
            votingengine::Error::<T>::NoPermission
        );
        ensure!(
            !(procedure == VoteProcedure::RepresentativeOnly
                && rule == RepresentativeVoteRule::Special),
            Error::<T>::InvalidRepresentativeRule
        );
        ensure!(
            (procedure == VoteProcedure::Legislation) == legislation_meta.is_some(),
            Error::<T>::ProposalMetaMissing
        );

        let mut additional_institutions = sp_runtime::sp_std::vec::Vec::new();
        if let Some(meta) = legislation_meta.as_ref() {
            additional_institutions.push(meta.executive.clone());
            if let Some(legislature) = meta.legislature.as_ref() {
                additional_institutions.push(legislature.clone());
            }
        }
        let subject_cid_numbers = Self::resolve_subject_cid_numbers(
            &actor_cid_number,
            &route,
            additional_subjects,
            &additional_institutions,
        )?;

        let population_scope = if rule == RepresentativeVoteRule::Special {
            Some(Self::population_scope_for_actor(
                actor_code,
                &actor_cid_number,
            )?)
        } else {
            None
        };
        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::stage_duration());

        with_transaction(|| {
            // 只有特别案创建人口快照。快照与提案、法律对象及自动投票处于
            // 同一外层事务，后续任一写入失败都不会留下孤立快照。
            let (eligible_total, population_snapshot_id) = match population_scope.as_ref() {
                Some(scope) => {
                    let (snapshot_id, eligible_total) =
                        match <votingengine::Pallet<T>>::create_population_snapshot(scope) {
                            Ok(value) => value,
                            Err(err) => return TransactionOutcome::Rollback(Err(err)),
                        };
                    if eligible_total == 0 {
                        return TransactionOutcome::Rollback(Err(
                            Error::<T>::CitizenEligibleTotalNotSet.into(),
                        ));
                    }
                    (eligible_total, Some(snapshot_id))
                }
                None => (0, None),
            };
            let proposal = Proposal {
                kind: PROPOSAL_KIND_LEGISLATION,
                stage: STAGE_LEG_REPRESENTATIVE,
                status: STATUS_VOTING,
                internal_code: Some(first_code),
                actor_cid_number: Some(actor_cid_number),
                execution_account: None,
                subject_cid_numbers,
                start: now,
                end,
                citizen_eligible_total: eligible_total,
            };
            let id = match <votingengine::Pallet<T>>::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            if let Err(err) =
                votingengine::limit::try_add_active_proposals::<T>(proposal.subject_keys(), id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            // 立法提案可能关联多机构,互斥锁以所有关联 CID 为主体占用。
            for subject in proposal.subject_keys() {
                if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                    id,
                    subject,
                    InternalProposalMutexKind::Regular,
                ) {
                    return TransactionOutcome::Rollback(Err(err));
                }
            }
            if let Err(err) = <votingengine::Pallet<T>>::snapshot_institution_admins(
                id,
                first_code,
                first_cid_number,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }
            pallet::RepresentativeMetas::<T>::insert(
                id,
                pallet::RepresentativeMeta {
                    route,
                    current_body: 0,
                    rule,
                    procedure,
                },
            );
            if let Some(meta) = legislation_meta {
                pallet::LegislationMetas::<T>::insert(id, meta);
            }
            Proposals::<T>::insert(id, proposal);
            if let Some(snapshot_id) = population_snapshot_id {
                if let Err(err) =
                    <votingengine::Pallet<T>>::bind_population_snapshot(id, snapshot_id)
                {
                    return TransactionOutcome::Rollback(Err(err));
                }
            }
            if let Err(err) = <votingengine::Pallet<T>>::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::emit_proposal_created(
                id,
                PROPOSAL_KIND_LEGISLATION,
                STAGE_LEG_REPRESENTATIVE,
                end,
            );
            Self::deposit_event(pallet::Event::<T>::RepresentativeProposalCreated {
                proposal_id: id,
                rule,
                bodies: pallet::RepresentativeMetas::<T>::get(id)
                    .map(|meta| meta.route.len() as u32)
                    .unwrap_or_default(),
                procedure,
            });
            TransactionOutcome::Commit(Ok(id))
        })
    }

    /// 对当前代表机构投票；同一账户可在不同机构阶段分别依法投票。
    pub fn do_cast_representative_vote(
        who: T::AccountId,
        proposal_id: u64,
        approve: bool,
    ) -> DispatchResult {
        let proposal = <votingengine::Pallet<T>>::ensure_open_proposal(proposal_id)?;
        ensure!(
            proposal.kind == PROPOSAL_KIND_LEGISLATION,
            votingengine::Error::<T>::InvalidProposalKind
        );
        ensure!(
            proposal.stage == STAGE_LEG_REPRESENTATIVE,
            votingengine::Error::<T>::InvalidProposalStage
        );
        let meta = pallet::RepresentativeMetas::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalMetaMissing)?;
        let institution_cid_number = meta
            .route
            .body(meta.current_body)
            .ok_or(Error::<T>::InvalidRepresentativeRoute)?;
        let vote_key = (meta.current_body, who.clone());
        ensure!(
            !pallet::RepresentativeVotesByAccount::<T>::contains_key(proposal_id, &vote_key),
            votingengine::Error::<T>::AlreadyVoted
        );
        ensure!(
            <votingengine::Pallet<T>>::is_admin_in_snapshot(
                proposal_id,
                votingengine::ProposalSubject::InstitutionCid(institution_cid_number.clone()),
                &who,
            ),
            votingengine::Error::<T>::NoPermission
        );

        pallet::RepresentativeVotesByAccount::<T>::insert(proposal_id, vote_key, approve);
        let tally =
            pallet::RepresentativeTallies::<T>::mutate(proposal_id, meta.current_body, |t| {
                if approve {
                    t.yes = t.yes.saturating_add(1);
                } else {
                    t.no = t.no.saturating_add(1);
                }
                *t
            });
        Self::deposit_event(pallet::Event::<T>::RepresentativeVoteCast {
            proposal_id,
            body_index: meta.current_body,
            who,
            approve,
        });

        let admins_len = <votingengine::Pallet<T>>::snapshot_admins_len(
            proposal_id,
            votingengine::ProposalSubject::InstitutionCid(institution_cid_number),
        )
        .ok_or(votingengine::Error::<T>::MissingAdminSnapshot)?;
        match representative_decided(meta.rule, admins_len, tally.yes, tally.no) {
            Some(true) => match meta.route {
                RepresentativeRoute::Single(_) => {
                    Self::finish_single_representative_vote(proposal_id)
                }
                RepresentativeRoute::Sequential(_) => {
                    Self::advance_sequential_representative_vote(proposal_id)
                }
            },
            Some(false) => {
                <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)
            }
            None => Ok(()),
        }
    }

    /// 顺序路线推进至下一个代表机构；全部完成后进入配置的后续程序。
    pub(crate) fn advance_representative_body_or_finish(proposal_id: u64) -> DispatchResult {
        let meta = pallet::RepresentativeMetas::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalMetaMissing)?;
        let next = meta.current_body.saturating_add(1);
        if (next as usize) < meta.route.len() {
            let next_cid_number = meta
                .route
                .body(next)
                .ok_or(Error::<T>::InvalidRepresentativeRoute)?;
            let next_code = Self::institution_code_for_cid(&next_cid_number)?;
            let now = <frame_system::Pallet<T>>::block_number();
            let end = now.saturating_add(Self::stage_duration());
            with_transaction(|| {
                // 各机构计票按 body_index 永久隔离到提案清理，不删除前一阶段审计记录。
                pallet::RepresentativeMetas::<T>::mutate(proposal_id, |maybe| {
                    if let Some(m) = maybe {
                        m.current_body = next;
                    }
                });
                if let Err(err) = <votingengine::Pallet<T>>::snapshot_institution_admins(
                    proposal_id,
                    next_code,
                    next_cid_number,
                ) {
                    return TransactionOutcome::Rollback(Err(err));
                }
                let old_end =
                    match Proposals::<T>::try_mutate(
                        proposal_id,
                        |maybe| -> Result<
                            frame_system::pallet_prelude::BlockNumberFor<T>,
                            DispatchError,
                        > {
                            let p = maybe
                                .as_mut()
                                .ok_or(votingengine::Error::<T>::ProposalNotFound)?;
                            let old = p.end;
                            p.internal_code = Some(next_code);
                            p.start = now;
                            p.end = end;
                            Ok(old)
                        },
                    ) {
                        Ok(v) => v,
                        Err(err) => return TransactionOutcome::Rollback(Err(err)),
                    };
                let old_expiry = old_end.saturating_add(One::one());
                ProposalsByExpiry::<T>::mutate(old_expiry, |ids| ids.retain(|&i| i != proposal_id));
                if let Err(err) =
                    <votingengine::Pallet<T>>::schedule_proposal_expiry(proposal_id, end)
                {
                    return TransactionOutcome::Rollback(Err(err));
                }
                Self::deposit_event(pallet::Event::<T>::RepresentativeBodyAdvanced {
                    proposal_id,
                    next_body: next,
                });
                TransactionOutcome::Commit(Ok(()))
            })
        } else {
            Self::finish_representative_route(proposal_id)
        }
    }

    /// 所有代表机构通过后按强类型程序进入终局或法律专属阶段。
    pub(crate) fn finish_representative_route(proposal_id: u64) -> DispatchResult {
        let meta = pallet::RepresentativeMetas::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalMetaMissing)?;
        match meta.procedure {
            VoteProcedure::RepresentativeOnly => {
                <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_PASSED)
            }
            VoteProcedure::Legislation if meta.rule == RepresentativeVoteRule::Special => {
                Self::advance_to_referendum(proposal_id)
            }
            VoteProcedure::Legislation => Self::advance_to_sign(proposal_id),
        }
    }
}

impl<T: Config> Pallet<T> {
    /// 当前代表机构阶段超时结算：按强类型门槛计票，通过则推进，否则否决。
    pub fn do_finalize_representative_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            proposal.stage == STAGE_LEG_REPRESENTATIVE,
            votingengine::Error::<T>::InvalidProposalStage
        );
        ensure!(
            proposal.status == STATUS_VOTING,
            votingengine::Error::<T>::ProposalAlreadyFinalized
        );
        ensure!(
            <frame_system::Pallet<T>>::block_number() > proposal.end,
            votingengine::Error::<T>::VoteNotExpired
        );
        let meta = pallet::RepresentativeMetas::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalMetaMissing)?;
        let institution_cid_number = meta
            .route
            .body(meta.current_body)
            .ok_or(Error::<T>::InvalidRepresentativeRoute)?;
        let admins_len = <votingengine::Pallet<T>>::snapshot_admins_len(
            proposal_id,
            votingengine::ProposalSubject::InstitutionCid(institution_cid_number),
        )
        .ok_or(votingengine::Error::<T>::MissingAdminSnapshot)?;
        let tally = pallet::RepresentativeTallies::<T>::get(proposal_id, meta.current_body);
        if representative_final_passed(meta.rule, admins_len, tally.yes, tally.no) {
            match meta.route {
                RepresentativeRoute::Single(_) => {
                    Self::finish_single_representative_vote(proposal_id)
                }
                RepresentativeRoute::Sequential(_) => {
                    Self::advance_sequential_representative_vote(proposal_id)
                }
            }
        } else {
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)
        }
    }
}
