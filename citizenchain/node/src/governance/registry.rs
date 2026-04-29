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
            InstitutionRef::Nrc(item) | InstitutionRef::Prc(item) => item.shenfen_name,
            InstitutionRef::Prb(item) => item.shenfen_name,
        }
    }

    pub(crate) fn shenfen_id(self) -> &'static str {
        match self {
            InstitutionRef::Nrc(item) | InstitutionRef::Prc(item) => item.shenfen_id,
            InstitutionRef::Prb(item) => item.shenfen_id,
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
            shenfen_id: self.shenfen_id().to_string(),
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

pub(crate) fn find_institution(shenfen_id: &str) -> Option<InstitutionRef> {
    if let Some(index) = CHINA_CB
        .iter()
        .position(|item| item.shenfen_id == shenfen_id)
    {
        return Some(if index == 0 {
            InstitutionRef::Nrc(&CHINA_CB[index])
        } else {
            InstitutionRef::Prc(&CHINA_CB[index])
        });
    }

    CHINA_CH
        .iter()
        .find(|item| item.shenfen_id == shenfen_id)
        .map(InstitutionRef::Prb)
}

pub(crate) fn find_institution_name(shenfen_id: &str) -> Option<&'static str> {
    find_institution(shenfen_id).map(|item| item.name())
}
