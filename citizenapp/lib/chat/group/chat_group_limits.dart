/// 私密小群单群成员硬上限(单源)。
///
/// 与 Rust `chat_mls.rs` 的 `MAX_GROUP_MEMBERS` 一致;发送端(Dart)与
/// 密码学层(MLS 实际成员数)双拦,任一超限即拒。
const int kMaxGroupMembers = 1989;
