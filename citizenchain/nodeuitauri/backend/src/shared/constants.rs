// 全链共享常量，避免各模块重复定义导致一致性风险。
pub const SS58_PREFIX: u16 = 2027;
pub const EXPECTED_SS58_PREFIX: u64 = 2027;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ss58_prefix_values_consistent() {
        assert_eq!(SS58_PREFIX as u64, EXPECTED_SS58_PREFIX);
    }

    #[test]
    fn ss58_prefix_is_2027() {
        assert_eq!(SS58_PREFIX, 2027);
    }
}
