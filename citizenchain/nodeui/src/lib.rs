#![forbid(unsafe_code)]

/// Shared crate metadata for the desktop workspace member.
pub const NODEUI_FRONTEND_CRATE: &str = "nodeui-frontend";

#[cfg(test)]
mod tests {
    use super::NODEUI_FRONTEND_CRATE;

    #[test]
    fn crate_name_constant_is_stable() {
        assert_eq!(NODEUI_FRONTEND_CRATE, "nodeui-frontend");
    }
}
