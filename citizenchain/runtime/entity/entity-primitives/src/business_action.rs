//! 机构业务动作与受保护创世岗位固定权限目录。
//!
//! `BusinessActionId` 是跨业务模块、创世和原生节点守卫共同解释的稳定协议，不能把
//! pallet/call 索引临时转换成权限，也不能由客户端自造字符串。这里仅冻结动作身份和
//! 创世固定岗位权限；业务使用哪个投票引擎、如何构造 `VotePlan` 仍由对应业务模块决定。

extern crate alloc;

use alloc::{vec, vec::Vec};

use primitives::{
    cid::{
        china::citizenchain::{
            is_citizenchain_technology_identity, ROLE_CODE_GENESIS_PRODUCT_MANAGER,
            ROLE_CODE_GENESIS_PROGRAMMER,
        },
        code::{InstitutionCode, FRG, NJD, NRC, PRB, PRC, PROVINCE_CODE_INFOS},
    },
    governance_skeleton::{
        fixed_institution_by_identity, fixed_role_seats_by_identity,
        province_commissioner_role_code, ROLE_CODE_CHIEF_JUSTICE, ROLE_CODE_COMMITTEE_MEMBER,
        ROLE_CODE_CONSTITUTION_GUARD, ROLE_CODE_DEPUTY_CHIEF_JUSTICE, ROLE_CODE_DIRECTOR,
        ROLE_CODE_JUSTICE,
    },
    institution_constraints::ROLE_CODE_LEGAL_REPRESENTATIVE,
};

use crate::RolePermissionOperation;

pub const MODULE_PUBLIC_MANAGE: &[u8] = b"pub-mgmt";
pub const MODULE_PRIVATE_MANAGE: &[u8] = b"pri-mgmt";
pub const MODULE_RUNTIME_UPGRADE: &[u8] = b"rt-upg";
pub const MODULE_RESOLUTION_ISSUANCE: &[u8] = b"res-iss";
pub const MODULE_RESOLUTION_DESTROY: &[u8] = b"res-dst";
pub const MODULE_GRANDPA_KEY_CHANGE: &[u8] = b"gra-key";
pub const MODULE_MULTISIG: &[u8] = b"multisig";
pub const MODULE_ONCHAIN_ISSUANCE: &[u8] = b"onc-iss";
pub const MODULE_LEGISLATION_YUAN: &[u8] = b"leg-yuan";
pub const MODULE_SQUARE_SUBSCRIPTION: &[u8] = b"sqr-sub";
/// 第 7 步接入投票前先冻结公民身份业务标签，禁止复用为其它业务。
pub const MODULE_CITIZEN_IDENTITY: &[u8] = b"cit-id";
/// 第 7 步接入投票前先冻结地址登记业务标签，禁止复用为其它业务。
pub const MODULE_ADDRESS_REGISTRY: &[u8] = b"addr-reg";
/// 第 6 步原子机构登记模块的预留稳定标签。
pub const MODULE_INSTITUTION_REGISTRATION: &[u8] = b"ins-reg";

pub const ACTION_INSTITUTION_GOVERNANCE: u32 = 3;
pub const ACTION_RUNTIME_UPGRADE: u32 = 0;
pub const ACTION_RESOLUTION_ISSUANCE: u32 = 0;
pub const ACTION_RESOLUTION_DESTROY: u32 = 0;
pub const ACTION_GRANDPA_KEY_CHANGE: u32 = 0;
pub const ACTION_MULTISIG_TRANSFER: u32 = 0;
pub const ACTION_SAFETY_FUND_TRANSFER: u32 = 1;
pub const ACTION_FEE_SWEEP_TO_MAIN: u32 = 2;
pub const ACTION_MONITOR_FREEZE: u32 = 10;
pub const ACTION_MONITOR_UNFREEZE: u32 = 11;
pub const ACTION_MONITOR_CONFISCATE: u32 = 12;
pub const ACTION_MONITOR_FORCE_TRANSFER: u32 = 13;
pub const ACTION_MONITOR_FORCE_CLOSE: u32 = 14;
pub const ACTION_AMEND_LAW: u32 = 1;
pub const ACTION_PLATFORM_PRICE: u32 = 5;
pub const ACTION_REGISTER_VOTING_IDENTITY: u32 = 0;
pub const ACTION_UPGRADE_CANDIDATE_IDENTITY: u32 = 1;
pub const ACTION_UPDATE_VOTING_IDENTITY: u32 = 2;
pub const ACTION_UPDATE_CANDIDATE_IDENTITY: u32 = 3;
pub const ACTION_REVOKE_IDENTITY: u32 = 4;
pub const ACTION_OCCUPY_CID: u32 = 6;
pub const ACTION_OCCUPY_CIDS_BATCH: u32 = 7;
pub const ACTION_REVOKE_CID: u32 = 8;
pub const ACTION_SET_ADDRESS_CATALOG: u32 = 0;
pub const ACTION_SET_ADDRESS_NAME: u32 = 1;
pub const ACTION_REMOVE_ADDRESS_NAME: u32 = 2;
pub const ACTION_SET_ADDRESS: u32 = 3;
pub const ACTION_REMOVE_ADDRESS: u32 = 4;
pub const ACTION_REGISTER_INSTITUTION: u32 = 0;

/// 不带机构主体的固定权限模板；创世写入时由 entity 补齐准确 CID 和岗位码。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedRolePermissionSpec {
    pub module_tag: &'static [u8],
    pub action_code: u32,
    pub operation: RolePermissionOperation,
}

fn push_permission(
    out: &mut Vec<FixedRolePermissionSpec>,
    module_tag: &'static [u8],
    action_code: u32,
    operation: RolePermissionOperation,
) {
    out.push(FixedRolePermissionSpec {
        module_tag,
        action_code,
        operation,
    });
}

fn push_both(out: &mut Vec<FixedRolePermissionSpec>, module_tag: &'static [u8], action_code: u32) {
    push_permission(
        out,
        module_tag,
        action_code,
        RolePermissionOperation::Propose,
    );
    push_permission(out, module_tag, action_code, RolePermissionOperation::Vote);
}

fn push_actions_both(
    out: &mut Vec<FixedRolePermissionSpec>,
    module_tag: &'static [u8],
    action_codes: &[u32],
) {
    for action_code in action_codes {
        push_both(out, module_tag, *action_code);
    }
}

/// 返回准确创世机构固定岗位的永久权限。
///
/// 返回空数组有两种合法含义：该岗位固定为空权限，或输入不是受保护创世固定岗位。
/// 调用方必须先用共享固定岗位规格确认岗位身份，不能把空数组当作一般岗位判断依据。
pub fn fixed_role_permission_specs(
    institution_code: InstitutionCode,
    cid_number: &[u8],
    role_code: &[u8],
) -> Vec<FixedRolePermissionSpec> {
    let is_public_fixed = fixed_institution_by_identity(institution_code, cid_number).is_some();
    let is_technology = is_citizenchain_technology_identity(institution_code, cid_number);
    if !is_public_fixed && !is_technology {
        return Vec::new();
    }

    // 普通受保护公权机构的 LR 永久为空权限；技术公司 LR 是准确 CID 的独立规格。
    if role_code == ROLE_CODE_LEGAL_REPRESENTATIVE && !is_technology {
        return Vec::new();
    }

    let mut out = Vec::new();
    match institution_code {
        NRC if role_code == ROLE_CODE_COMMITTEE_MEMBER => {
            push_both(
                &mut out,
                MODULE_PUBLIC_MANAGE,
                ACTION_INSTITUTION_GOVERNANCE,
            );
            push_both(&mut out, MODULE_RUNTIME_UPGRADE, ACTION_RUNTIME_UPGRADE);
            push_both(
                &mut out,
                MODULE_RESOLUTION_ISSUANCE,
                ACTION_RESOLUTION_ISSUANCE,
            );
            push_both(
                &mut out,
                MODULE_RESOLUTION_DESTROY,
                ACTION_RESOLUTION_DESTROY,
            );
            push_both(
                &mut out,
                MODULE_GRANDPA_KEY_CHANGE,
                ACTION_GRANDPA_KEY_CHANGE,
            );
            push_actions_both(
                &mut out,
                MODULE_MULTISIG,
                &[
                    ACTION_MULTISIG_TRANSFER,
                    ACTION_SAFETY_FUND_TRANSFER,
                    ACTION_FEE_SWEEP_TO_MAIN,
                ],
            );
            push_actions_both(
                &mut out,
                MODULE_ONCHAIN_ISSUANCE,
                &[
                    ACTION_MONITOR_FREEZE,
                    ACTION_MONITOR_UNFREEZE,
                    ACTION_MONITOR_CONFISCATE,
                    ACTION_MONITOR_FORCE_TRANSFER,
                    ACTION_MONITOR_FORCE_CLOSE,
                ],
            );
        }
        PRC if role_code == ROLE_CODE_COMMITTEE_MEMBER => {
            push_both(
                &mut out,
                MODULE_PUBLIC_MANAGE,
                ACTION_INSTITUTION_GOVERNANCE,
            );
            // 协议升级与决议发行相同，由 NRC + 43 个 PRC 委员岗位共同拥有。
            push_both(&mut out, MODULE_RUNTIME_UPGRADE, ACTION_RUNTIME_UPGRADE);
            push_both(
                &mut out,
                MODULE_RESOLUTION_ISSUANCE,
                ACTION_RESOLUTION_ISSUANCE,
            );
            push_both(
                &mut out,
                MODULE_RESOLUTION_DESTROY,
                ACTION_RESOLUTION_DESTROY,
            );
            push_both(
                &mut out,
                MODULE_GRANDPA_KEY_CHANGE,
                ACTION_GRANDPA_KEY_CHANGE,
            );
            push_both(&mut out, MODULE_MULTISIG, ACTION_MULTISIG_TRANSFER);
        }
        PRB if role_code == ROLE_CODE_DIRECTOR => {
            push_both(
                &mut out,
                MODULE_PUBLIC_MANAGE,
                ACTION_INSTITUTION_GOVERNANCE,
            );
            // 协议升级与决议发行使用同一联合投票主体：PRB 董事仅参与投票，不能发起。
            push_permission(
                &mut out,
                MODULE_RUNTIME_UPGRADE,
                ACTION_RUNTIME_UPGRADE,
                RolePermissionOperation::Vote,
            );
            push_permission(
                &mut out,
                MODULE_RESOLUTION_ISSUANCE,
                ACTION_RESOLUTION_ISSUANCE,
                RolePermissionOperation::Vote,
            );
            push_both(
                &mut out,
                MODULE_RESOLUTION_DESTROY,
                ACTION_RESOLUTION_DESTROY,
            );
            push_actions_both(
                &mut out,
                MODULE_MULTISIG,
                &[ACTION_MULTISIG_TRANSFER, ACTION_FEE_SWEEP_TO_MAIN],
            );
        }
        NJD if role_code == ROLE_CODE_CHIEF_JUSTICE => {
            push_both(
                &mut out,
                MODULE_PUBLIC_MANAGE,
                ACTION_INSTITUTION_GOVERNANCE,
            );
        }
        NJD if role_code == ROLE_CODE_DEPUTY_CHIEF_JUSTICE || role_code == ROLE_CODE_JUSTICE => {
            push_permission(
                &mut out,
                MODULE_PUBLIC_MANAGE,
                ACTION_INSTITUTION_GOVERNANCE,
                RolePermissionOperation::Vote,
            );
        }
        NJD if role_code == ROLE_CODE_CONSTITUTION_GUARD => {
            push_permission(
                &mut out,
                MODULE_PUBLIC_MANAGE,
                ACTION_INSTITUTION_GOVERNANCE,
                RolePermissionOperation::Vote,
            );
            push_permission(
                &mut out,
                MODULE_LEGISLATION_YUAN,
                ACTION_AMEND_LAW,
                RolePermissionOperation::Vote,
            );
        }
        FRG if fixed_role_seats_by_identity(institution_code, cid_number, role_code).is_some() => {
            push_both(
                &mut out,
                MODULE_PUBLIC_MANAGE,
                ACTION_INSTITUTION_GOVERNANCE,
            );
            push_both(
                &mut out,
                MODULE_INSTITUTION_REGISTRATION,
                ACTION_REGISTER_INSTITUTION,
            );
            push_actions_both(
                &mut out,
                MODULE_CITIZEN_IDENTITY,
                &[
                    ACTION_REGISTER_VOTING_IDENTITY,
                    ACTION_UPGRADE_CANDIDATE_IDENTITY,
                    ACTION_UPDATE_VOTING_IDENTITY,
                    ACTION_UPDATE_CANDIDATE_IDENTITY,
                    ACTION_REVOKE_IDENTITY,
                    ACTION_OCCUPY_CID,
                    ACTION_OCCUPY_CIDS_BATCH,
                    ACTION_REVOKE_CID,
                ],
            );
            push_actions_both(
                &mut out,
                MODULE_ADDRESS_REGISTRY,
                &[
                    ACTION_SET_ADDRESS_CATALOG,
                    ACTION_SET_ADDRESS_NAME,
                    ACTION_REMOVE_ADDRESS_NAME,
                    ACTION_SET_ADDRESS,
                    ACTION_REMOVE_ADDRESS,
                ],
            );
        }
        _ if is_technology && role_code == ROLE_CODE_LEGAL_REPRESENTATIVE => {
            push_both(
                &mut out,
                MODULE_PRIVATE_MANAGE,
                ACTION_INSTITUTION_GOVERNANCE,
            );
            push_permission(
                &mut out,
                MODULE_SQUARE_SUBSCRIPTION,
                ACTION_PLATFORM_PRICE,
                RolePermissionOperation::Vote,
            );
        }
        _ if is_technology && role_code == ROLE_CODE_GENESIS_PRODUCT_MANAGER => {
            push_both(
                &mut out,
                MODULE_PRIVATE_MANAGE,
                ACTION_INSTITUTION_GOVERNANCE,
            );
            push_both(&mut out, MODULE_SQUARE_SUBSCRIPTION, ACTION_PLATFORM_PRICE);
        }
        _ if is_technology && role_code == ROLE_CODE_GENESIS_PROGRAMMER => {
            push_both(
                &mut out,
                MODULE_PRIVATE_MANAGE,
                ACTION_INSTITUTION_GOVERNANCE,
            );
            push_permission(
                &mut out,
                MODULE_SQUARE_SUBSCRIPTION,
                ACTION_PLATFORM_PRICE,
                RolePermissionOperation::Vote,
            );
        }
        _ => {}
    }
    out
}

/// 固定创世机构 CID 顶层能力白名单。
///
/// 固定岗位权限必须是本白名单的子集。动态岗位后续需要新增业务时，应先由对应业务步骤
/// 明确扩展 CID 顶层能力，再创建新岗位；不得借此函数给固定岗位追加权限。
pub fn fixed_institution_capability_allows(
    institution_code: InstitutionCode,
    cid_number: &[u8],
    module_tag: &[u8],
    action_code: u32,
    operation: RolePermissionOperation,
) -> bool {
    let role_codes: Vec<Vec<u8>> = match institution_code {
        NRC | PRC => vec![ROLE_CODE_COMMITTEE_MEMBER.to_vec()],
        PRB => vec![ROLE_CODE_DIRECTOR.to_vec()],
        NJD => vec![
            ROLE_CODE_CONSTITUTION_GUARD.to_vec(),
            ROLE_CODE_CHIEF_JUSTICE.to_vec(),
            ROLE_CODE_DEPUTY_CHIEF_JUSTICE.to_vec(),
            ROLE_CODE_JUSTICE.to_vec(),
        ],
        FRG => PROVINCE_CODE_INFOS
            .iter()
            .map(|province| province_commissioner_role_code(province.province_code))
            .collect(),
        _ if is_citizenchain_technology_identity(institution_code, cid_number) => vec![
            ROLE_CODE_LEGAL_REPRESENTATIVE.to_vec(),
            ROLE_CODE_GENESIS_PRODUCT_MANAGER.to_vec(),
            ROLE_CODE_GENESIS_PROGRAMMER.to_vec(),
        ],
        _ => Vec::new(),
    };
    role_codes.into_iter().any(|role_code| {
        fixed_role_permission_specs(institution_code, cid_number, &role_code)
            .into_iter()
            .any(|permission| {
                permission.module_tag == module_tag
                    && permission.action_code == action_code
                    && permission.operation == operation
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::cid::china::{china_cb::CHINA_CB, china_ch::CHINA_CH, china_sf::CHINA_SF};

    fn has(
        permissions: &[FixedRolePermissionSpec],
        module_tag: &[u8],
        action_code: u32,
        operation: RolePermissionOperation,
    ) -> bool {
        permissions.iter().any(|permission| {
            permission.module_tag == module_tag
                && permission.action_code == action_code
                && permission.operation == operation
        })
    }

    #[test]
    fn runtime_upgrade_is_shared_by_nrc_and_prc_committees() {
        for entry in CHINA_CB {
            let permissions = fixed_role_permission_specs(
                primitives::cid::code::institution_code_from_cid_number(entry.cid_number)
                    .expect("CHINA_CB CID encodes institution code"),
                entry.cid_number.as_bytes(),
                ROLE_CODE_COMMITTEE_MEMBER,
            );
            assert!(has(
                &permissions,
                MODULE_RUNTIME_UPGRADE,
                ACTION_RUNTIME_UPGRADE,
                RolePermissionOperation::Propose,
            ));
            assert!(has(
                &permissions,
                MODULE_RUNTIME_UPGRADE,
                ACTION_RUNTIME_UPGRADE,
                RolePermissionOperation::Vote,
            ));
        }
    }

    #[test]
    fn protocol_upgrade_and_resolution_issuance_give_prb_vote_only() {
        let entry = &CHINA_CH[0];
        let permissions =
            fixed_role_permission_specs(PRB, entry.cid_number.as_bytes(), ROLE_CODE_DIRECTOR);
        for (module_tag, action_code) in [
            (MODULE_RUNTIME_UPGRADE, ACTION_RUNTIME_UPGRADE),
            (MODULE_RESOLUTION_ISSUANCE, ACTION_RESOLUTION_ISSUANCE),
        ] {
            assert!(has(
                &permissions,
                module_tag,
                action_code,
                RolePermissionOperation::Vote,
            ));
            assert!(!has(
                &permissions,
                module_tag,
                action_code,
                RolePermissionOperation::Propose,
            ));
        }
    }

    #[test]
    fn judicial_and_technology_roles_are_least_privilege() {
        let njd = &CHINA_SF[0];
        let chief =
            fixed_role_permission_specs(NJD, njd.cid_number.as_bytes(), ROLE_CODE_CHIEF_JUSTICE);
        let justice =
            fixed_role_permission_specs(NJD, njd.cid_number.as_bytes(), ROLE_CODE_JUSTICE);
        assert!(has(
            &chief,
            MODULE_PUBLIC_MANAGE,
            ACTION_INSTITUTION_GOVERNANCE,
            RolePermissionOperation::Propose,
        ));
        assert!(!has(
            &justice,
            MODULE_PUBLIC_MANAGE,
            ACTION_INSTITUTION_GOVERNANCE,
            RolePermissionOperation::Propose,
        ));

        let company = primitives::cid::china::citizenchain::CITIZENCHAIN_TECHNOLOGY;
        let product = fixed_role_permission_specs(
            *b"SFGQ",
            company.cid_number.as_bytes(),
            ROLE_CODE_GENESIS_PRODUCT_MANAGER,
        );
        let programmer = fixed_role_permission_specs(
            *b"SFGQ",
            company.cid_number.as_bytes(),
            ROLE_CODE_GENESIS_PROGRAMMER,
        );
        assert!(has(
            &product,
            MODULE_SQUARE_SUBSCRIPTION,
            ACTION_PLATFORM_PRICE,
            RolePermissionOperation::Propose,
        ));
        assert!(!has(
            &programmer,
            MODULE_SQUARE_SUBSCRIPTION,
            ACTION_PLATFORM_PRICE,
            RolePermissionOperation::Propose,
        ));
    }
}
