//! 非法人机构能力。
//!
//! 中文注释:非法人机构不是独立法人,必须从属于一个具有法人资格的主体。
//! 公权机构和私权机构都可以拥有从属非法人机构,所以能力放在 `subjects/uninorg`。

pub(crate) fn is_unincorporated_subject(subject_property: &str) -> bool {
    subject_property == "F"
}

pub(crate) fn requires_parent(subject_property: &str) -> bool {
    is_unincorporated_subject(subject_property)
}

pub(crate) fn can_attach_to_parent_subject(parent_subject_property: &str) -> bool {
    matches!(parent_subject_property, "S" | "G")
}

pub(crate) fn parent_subject_requirement_message() -> &'static str {
    "所属法人必须是私法人(S)或公法人(G)"
}
