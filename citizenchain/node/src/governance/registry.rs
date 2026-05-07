// 治理机构注册表：直接读取 runtime 常量，避免 node 侧再维护过期地址副本。

use primitives::china::china_cb::{ChinaCb, CHINA_CB, NRC_ANQUAN_ADDRESS};
use primitives::china::china_ch::{ChinaCh, CHINA_CH};

use super::types::{GovernanceOverview, InstitutionListItem, OrgType};

#[derive(Clone, Copy)]
pub(crate) enum InstitutionRef {
    Nrc(&'static ChinaCb),
    Prc(&'static ChinaCb),
    Prb(&'static ChinaCh),
}

impl InstitutionRef {
    pub(crate) fn name(self) -> &'static str {
        match self {
            InstitutionRef::Nrc(item) | InstitutionRef::Prc(item) => item.sfid_name,
            InstitutionRef::Prb(item) => item.sfid_name,
        }
    }

    pub(crate) fn sfid_number(self) -> &'static str {
        match self {
            InstitutionRef::Nrc(item) | InstitutionRef::Prc(item) => item.sfid_number,
            InstitutionRef::Prb(item) => item.sfid_number,
        }
    }

    pub(crate) fn org_type(self) -> OrgType {
        match self {
            InstitutionRef::Nrc(_) => OrgType::Nrc,
            InstitutionRef::Prc(_) => OrgType::Prc,
            InstitutionRef::Prb(_) => OrgType::Prb,
        }
    }

    pub(crate) fn main_address_hex(self) -> String {
        match self {
            InstitutionRef::Nrc(item) | InstitutionRef::Prc(item) => hex::encode(item.main_address),
            InstitutionRef::Prb(item) => hex::encode(item.main_address),
        }
    }

    pub(crate) fn fee_address_hex(self) -> String {
        match self {
            InstitutionRef::Nrc(item) | InstitutionRef::Prc(item) => hex::encode(item.fee_address),
            InstitutionRef::Prb(item) => hex::encode(item.fee_address),
        }
    }

    pub(crate) fn staking_address_hex(self) -> Option<String> {
        match self {
            InstitutionRef::Prb(item) => Some(hex::encode(item.stake_address)),
            InstitutionRef::Nrc(_) | InstitutionRef::Prc(_) => None,
        }
    }

    pub(crate) fn anquan_address_hex(self) -> Option<String> {
        match self {
            InstitutionRef::Nrc(_) => Some(hex::encode(NRC_ANQUAN_ADDRESS)),
            InstitutionRef::Prc(_) | InstitutionRef::Prb(_) => None,
        }
    }

    pub(crate) fn to_list_item(self) -> InstitutionListItem {
        let org_type = self.org_type();
        InstitutionListItem {
            name: self.name().to_string(),
            sfid_number: self.sfid_number().to_string(),
            org_type: org_type as u8,
            org_type_label: org_type.label().to_string(),
            main_address: self.main_address_hex(),
        }
    }
}

pub(crate) fn governance_overview() -> GovernanceOverview {
    GovernanceOverview {
        national_councils: CHINA_CB
            .first()
            .map(|item| InstitutionRef::Nrc(item).to_list_item())
            .into_iter()
            .collect(),
        provincial_councils: CHINA_CB
            .iter()
            .skip(1)
            .map(|item| InstitutionRef::Prc(item).to_list_item())
            .collect(),
        provincial_banks: CHINA_CH
            .iter()
            .map(|item| InstitutionRef::Prb(item).to_list_item())
            .collect(),
        warning: None,
    }
}

pub(crate) fn find_institution(sfid_number: &str) -> Option<InstitutionRef> {
    if let Some(index) = CHINA_CB
        .iter()
        .position(|item| item.sfid_number == sfid_number)
    {
        return Some(if index == 0 {
            InstitutionRef::Nrc(&CHINA_CB[index])
        } else {
            InstitutionRef::Prc(&CHINA_CB[index])
        });
    }

    CHINA_CH
        .iter()
        .find(|item| item.sfid_number == sfid_number)
        .map(InstitutionRef::Prb)
}

pub(crate) fn find_institution_name(sfid_number: &str) -> Option<&'static str> {
    find_institution(sfid_number).map(|item| item.name())
}
