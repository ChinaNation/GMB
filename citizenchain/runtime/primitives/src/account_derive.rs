//! 账户地址派生唯一真源。
//! preimage = `GMB || op_tag || ss58_le || payload`,结果为 32 字节 AccountId。

use crate::core_const::GMB; // 域共享(签名也用)
use sp_core::hashing::blake2_256;
use sp_std::vec::Vec;

// 地址派生 op_tag(0x00-0x0F)。
pub const OP_MAIN: u8 = 0x00; // 所有机构主账户 · input: cid_number
pub const OP_FEE: u8 = 0x01; // 所有机构费用账户 · input: cid_number
pub const OP_STAKE: u8 = 0x02; // 永久质押 · input: cid_number
pub const OP_SAFETY: u8 = 0x03; // 安全基金 · input: cid_number
pub const OP_HE: u8 = 0x04; // 两和基金 · input: cid_number
pub const OP_PERSONAL: u8 = 0x05; // 个人多签账户 · input: creator_32 || account_name
pub const OP_CLEARING: u8 = 0x06; // 清算账户(私法人股份公司专属) · input: cid_number
pub const OP_NAME: u8 = 0x07; // CID 机构自定义命名账户 · input: cid_number || account_name

/// 机构账户受限保留名唯一字面源。
pub const RESERVED_NAME_MAIN_STR: &str = "主账户";
pub const RESERVED_NAME_FEE_STR: &str = "费用账户";
pub const RESERVED_NAME_STAKE_STR: &str = "永久质押";
pub const RESERVED_NAME_SAFETYFUND_STR: &str = "安全基金";
pub const RESERVED_NAME_HE_STR: &str = "两和基金";
pub const RESERVED_NAME_CLEARING_STR: &str = "清算账户";

pub const RESERVED_NAME_MAIN: &[u8] = RESERVED_NAME_MAIN_STR.as_bytes();
pub const RESERVED_NAME_FEE: &[u8] = RESERVED_NAME_FEE_STR.as_bytes();
pub const RESERVED_NAME_STAKE: &[u8] = RESERVED_NAME_STAKE_STR.as_bytes();
pub const RESERVED_NAME_SAFETYFUND: &[u8] = RESERVED_NAME_SAFETYFUND_STR.as_bytes();
pub const RESERVED_NAME_HE: &[u8] = RESERVED_NAME_HE_STR.as_bytes();
pub const RESERVED_NAME_CLEARING: &[u8] = RESERVED_NAME_CLEARING_STR.as_bytes();

/// 全部受限保留名。
pub const RESERVED_ACCOUNT_NAMES: [&[u8]; 6] = [
    RESERVED_NAME_MAIN,
    RESERVED_NAME_FEE,
    RESERVED_NAME_STAKE,
    RESERVED_NAME_SAFETYFUND,
    RESERVED_NAME_HE,
    RESERVED_NAME_CLEARING,
];

/// 机构协议账户类别。
///
/// 这里只表达协议账户的业务角色，不携带 CID，也不作为独立机构身份。
/// 每个机构需要哪些协议账户由 `institution_constraints` 唯一决定。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InstitutionProtocolAccountKind {
    Main,
    Fee,
    Stake,
    SafetyFund,
    He,
    /// 清算账户:仅私法人股份公司(SFGF)自动拥有,承载扫码支付 L2 清算资金。
    Clearing,
}

/// 是否为禁止注册的制度专属保留名。
pub fn is_forbidden_account_name(name: &[u8]) -> bool {
    name == RESERVED_NAME_STAKE
        || name == RESERVED_NAME_SAFETYFUND
        || name == RESERVED_NAME_HE
        || name == RESERVED_NAME_CLEARING
}

/// op_tag 与 payload schema 的唯一映射。
#[derive(Clone, Copy, Debug)]
pub enum AccountKind<'a> {
    InstitutionMain {
        cid_number: &'a [u8],
    },
    InstitutionFee {
        cid_number: &'a [u8],
    },
    InstitutionStake {
        cid_number: &'a [u8],
    },
    InstitutionSafetyFund {
        cid_number: &'a [u8],
    },
    InstitutionHe {
        cid_number: &'a [u8],
    },
    InstitutionClearing {
        cid_number: &'a [u8],
    },
    InstitutionNamed {
        cid_number: &'a [u8],
        account_name: &'a [u8],
    },
    Personal {
        creator_account_id: &'a [u8; 32],
        account_name: &'a [u8],
    },
}

impl<'a> AccountKind<'a> {
    /// 账户种类对应的 op_tag。
    pub const fn op_tag(&self) -> u8 {
        match self {
            AccountKind::InstitutionMain { .. } => OP_MAIN,
            AccountKind::InstitutionFee { .. } => OP_FEE,
            AccountKind::InstitutionStake { .. } => OP_STAKE,
            AccountKind::InstitutionSafetyFund { .. } => OP_SAFETY,
            AccountKind::InstitutionHe { .. } => OP_HE,
            AccountKind::InstitutionClearing { .. } => OP_CLEARING,
            AccountKind::InstitutionNamed { .. } => OP_NAME,
            AccountKind::Personal { .. } => OP_PERSONAL,
        }
    }

    /// 返回机构协议账户类别；自定义机构账户和个人多签账户返回 `None`。
    pub const fn institution_protocol_kind(&self) -> Option<InstitutionProtocolAccountKind> {
        match self {
            AccountKind::InstitutionMain { .. } => Some(InstitutionProtocolAccountKind::Main),
            AccountKind::InstitutionFee { .. } => Some(InstitutionProtocolAccountKind::Fee),
            AccountKind::InstitutionStake { .. } => Some(InstitutionProtocolAccountKind::Stake),
            AccountKind::InstitutionSafetyFund { .. } => {
                Some(InstitutionProtocolAccountKind::SafetyFund)
            }
            AccountKind::InstitutionHe { .. } => Some(InstitutionProtocolAccountKind::He),
            AccountKind::InstitutionClearing { .. } => {
                Some(InstitutionProtocolAccountKind::Clearing)
            }
            AccountKind::InstitutionNamed { .. } | AccountKind::Personal { .. } => None,
        }
    }

    /// 只有机构自定义命名账户允许进入机构账户关闭流程。
    pub const fn is_closable_institution_account(&self) -> bool {
        matches!(self, AccountKind::InstitutionNamed { .. })
    }

    /// payload 字段拼装。
    fn payload(&self) -> Vec<u8> {
        match self {
            AccountKind::InstitutionMain { cid_number }
            | AccountKind::InstitutionFee { cid_number }
            | AccountKind::InstitutionStake { cid_number }
            | AccountKind::InstitutionSafetyFund { cid_number }
            | AccountKind::InstitutionHe { cid_number }
            | AccountKind::InstitutionClearing { cid_number } => cid_number.to_vec(),
            AccountKind::InstitutionNamed {
                cid_number,
                account_name,
            } => {
                let mut payload = Vec::with_capacity(cid_number.len() + account_name.len());
                payload.extend_from_slice(cid_number);
                payload.extend_from_slice(account_name);
                payload
            }
            AccountKind::Personal {
                creator_account_id,
                account_name,
            } => {
                let mut payload = Vec::with_capacity(creator_account_id.len() + account_name.len());
                payload.extend_from_slice(*creator_account_id);
                payload.extend_from_slice(account_name);
                payload
            }
        }
    }

    /// 账户地址唯一派生入口。
    pub fn derive(&self, ss58: u16) -> [u8; 32] {
        let ss58_le = ss58.to_le_bytes();
        let payload = self.payload();
        let mut preimage = Vec::with_capacity(GMB.len() + 1 + ss58_le.len() + payload.len());
        preimage.extend_from_slice(GMB);
        preimage.push(self.op_tag());
        preimage.extend_from_slice(&ss58_le);
        preimage.extend_from_slice(&payload);
        blake2_256(&preimage)
    }
}

/// 协议账户类别对应的唯一保留名。
pub const fn institution_protocol_account_name(
    kind: InstitutionProtocolAccountKind,
) -> &'static [u8] {
    match kind {
        InstitutionProtocolAccountKind::Main => RESERVED_NAME_MAIN,
        InstitutionProtocolAccountKind::Fee => RESERVED_NAME_FEE,
        InstitutionProtocolAccountKind::Stake => RESERVED_NAME_STAKE,
        InstitutionProtocolAccountKind::SafetyFund => RESERVED_NAME_SAFETYFUND,
        InstitutionProtocolAccountKind::He => RESERVED_NAME_HE,
        InstitutionProtocolAccountKind::Clearing => RESERVED_NAME_CLEARING,
    }
}

/// 按账户名识别协议账户类别；普通自定义账户返回 `None`。
pub fn institution_protocol_kind_by_name(name: &[u8]) -> Option<InstitutionProtocolAccountKind> {
    if name == RESERVED_NAME_MAIN {
        return Some(InstitutionProtocolAccountKind::Main);
    }
    if name == RESERVED_NAME_FEE {
        return Some(InstitutionProtocolAccountKind::Fee);
    }
    if name == RESERVED_NAME_STAKE {
        return Some(InstitutionProtocolAccountKind::Stake);
    }
    if name == RESERVED_NAME_SAFETYFUND {
        return Some(InstitutionProtocolAccountKind::SafetyFund);
    }
    if name == RESERVED_NAME_HE {
        return Some(InstitutionProtocolAccountKind::He);
    }
    if name == RESERVED_NAME_CLEARING {
        return Some(InstitutionProtocolAccountKind::Clearing);
    }
    None
}

/// 机构账户名到派生种类的路由。
pub fn institution_kind_by_name<'a>(
    cid_number: &'a [u8],
    name: &'a [u8],
) -> Option<AccountKind<'a>> {
    if name.is_empty() {
        return None;
    }
    if name == RESERVED_NAME_MAIN {
        return Some(AccountKind::InstitutionMain { cid_number });
    }
    if name == RESERVED_NAME_FEE {
        return Some(AccountKind::InstitutionFee { cid_number });
    }
    if name == RESERVED_NAME_STAKE {
        return Some(AccountKind::InstitutionStake { cid_number });
    }
    if name == RESERVED_NAME_SAFETYFUND {
        return Some(AccountKind::InstitutionSafetyFund { cid_number });
    }
    if name == RESERVED_NAME_HE {
        return Some(AccountKind::InstitutionHe { cid_number });
    }
    if name == RESERVED_NAME_CLEARING {
        return Some(AccountKind::InstitutionClearing { cid_number });
    }
    Some(AccountKind::InstitutionNamed {
        cid_number,
        account_name: name,
    })
}

/// account_name 是否可作为机构自定义命名账户注册。
pub fn is_registrable_custom_name(name: &[u8]) -> bool {
    !name.is_empty()
        && name != RESERVED_NAME_MAIN
        && name != RESERVED_NAME_FEE
        && !is_forbidden_account_name(name)
}
