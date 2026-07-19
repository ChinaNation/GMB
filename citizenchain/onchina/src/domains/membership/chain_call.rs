//! `SquarePost::propose_set_platform_price` 唯一 call-data 编码器。

use codec::Encode;

pub(crate) const PROPOSE_PLATFORM_PRICE_ACTION: u16 = 0x2205;
const SQUARE_POST_PALLET_INDEX: u8 = 34;
const PROPOSE_PLATFORM_PRICE_CALL_INDEX: u8 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformMembershipLevel {
    Freedom,
    Democracy,
    Spark,
}

impl PlatformMembershipLevel {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "freedom" => Some(Self::Freedom),
            "democracy" => Some(Self::Democracy),
            "spark" => Some(Self::Spark),
            _ => None,
        }
    }

    fn scale_discriminant(self) -> u8 {
        match self {
            Self::Freedom => 0,
            Self::Democracy => 1,
            Self::Spark => 2,
        }
    }
}

/// 字段顺序必须与 runtime call 完全一致：CID、会员档位、价格。
pub(crate) fn build_propose_platform_price_call(
    actor_cid_number: &str,
    membership_level: PlatformMembershipLevel,
    new_price_fen: u128,
) -> Result<Vec<u8>, String> {
    let actor_cid_number = actor_cid_number.trim();
    if actor_cid_number.is_empty() {
        return Err("actor_cid_number is required".to_string());
    }
    if new_price_fen == 0 {
        return Err("new_price_fen must be positive".to_string());
    }
    let mut call = vec![SQUARE_POST_PALLET_INDEX, PROPOSE_PLATFORM_PRICE_CALL_INDEX];
    call.extend(actor_cid_number.as_bytes().to_vec().encode());
    call.push(membership_level.scale_discriminant());
    call.extend(new_price_fen.encode());
    Ok(call)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_price_call_has_exact_indices_and_field_order() {
        let cid = "GD001-SFGQ0-000000001-2026";
        let call =
            build_propose_platform_price_call(cid, PlatformMembershipLevel::Democracy, 123_456)
                .expect("build call");
        assert_eq!(&call[..2], &[34, 5]);
        assert_eq!(call[2], (cid.len() as u8) << 2);
        assert_eq!(&call[3..3 + cid.len()], cid.as_bytes());
        assert_eq!(call[3 + cid.len()], 1);
        assert_eq!(&call[4 + cid.len()..], &123_456_u128.to_le_bytes());
    }

    #[test]
    fn zero_price_is_rejected() {
        assert!(build_propose_platform_price_call(
            "GD001-SFGQ0-000000001-2026",
            PlatformMembershipLevel::Freedom,
            0,
        )
        .is_err());
    }
}
