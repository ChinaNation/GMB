#![forbid(unsafe_code)]

/// Shared crate metadata for the desktop workspace member.
pub const CITIZENUI_FRONTEND_CRATE: &str = "citizenui-frontend";

#[cfg(test)]
mod tests {
    use super::CITIZENUI_FRONTEND_CRATE;

    #[test]
    fn crate_name_constant_is_stable() {
        assert_eq!(CITIZENUI_FRONTEND_CRATE, "citizenui-frontend");
    }
}
