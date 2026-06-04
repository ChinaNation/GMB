//! 非法人机构能力。
//!
//! 中文注释:非法人机构不是独立法人,必须从属于一个具有法人资格的主体。
//! 公权机构和私权机构都可以拥有从属非法人机构,所以能力放在 `subjects/uninorg`。

pub(crate) fn is_unincorporated_a3(a3: &str) -> bool {
    a3 == "FFR"
}

pub(crate) fn requires_parent(a3: &str) -> bool {
    is_unincorporated_a3(a3)
}

pub(crate) fn can_attach_to_parent_a3(parent_a3: &str) -> bool {
    matches!(parent_a3, "SFR" | "GFR")
}

pub(crate) fn parent_a3_requirement_message() -> &'static str {
    "所属法人必须是私法人(SFR)或公法人(GFR)"
}
