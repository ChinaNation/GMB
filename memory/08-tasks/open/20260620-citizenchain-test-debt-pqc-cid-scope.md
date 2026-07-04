# citizenchain 测试债修复 — PQC + CID-scope 签名扩展

- 状态：完成（GMB 工作区 629 passed / 0 failed，排除 vendored grandpa）
- 完成记录(2026-06-20)：
  - 60 编译错误全修（test mock 补 CID-scope 参 + 调用点补 issuer_main_account/scope_province/scope_city + EmptyProvince→EmptyScopeProvinceName + CidInstitutionVerifier 补第 4 泛型），只改测试码、生产 lib 0 改动、断言无弱化（cases.rs assert 71→71）
  - 修复构建后暴露的 2 个既有逻辑测试 bug：① runtime `resolution_destro_internal_vote_flow` 投票循环 `0..13`→`1..13`（提案人 propose 时已自动一票,与同文件 8 个 1..N 测试一致）② node `compact_u128_big_integer` 测试值 `1_000_000`→`1_073_741_824`(2^30，原值 <2^30 实为 4 字节模式非 big-integer；生产 encode_compact_u128 正确)
  - 已修(2026-06-20)：vendored `sc-consensus-grandpa` lib-test 曾报 E0432（observer.rs 引用被裁掉的 communication::tests）——上游 vendor 测试债，仅在缓存失效时暴露（本次由并发产品改名触碰 node/vendor 触发）；GMB 测试命令应 `--exclude sc-consensus-grandpa` 或单独修 vendor，未擅改上游码
- 历史状态：进行中
- 创建：2026-06-20
- 背景：PQC/CID-scope 在途整改把生产签名扩了参，但 `#[cfg(test)]` 测试 mock/调用点没跟上。`cargo check --workspace`(lib) = 0 错误（生产码完整），但 `cargo test --workspace` 60 编译错误、8 个测试 crate 不构建。与 admins 统一无关。
- 根因签名（生产为准）：
  - 投票资格统一由 `CitizenIdentityReader::can_vote(who, scope)` 读取链上公民身份。
  - `verify_population_snapshot` 9 参：who/eligible_total/nonce/signature/issuer_cid_number/issuer_main_account/signer_pubkey/scope_province_name/scope_city_name→bool
  - `propose_create_institution`（organization-manage/src/lib.rs:501）15 参，新增 issuer_cid_number/issuer_main_account/signer_pubkey/scope_province_name/scope_city_name；其它 extrinsic 同理（11/9 参的 close/personal create）
  - 错误变体 `EmptyProvince`→`EmptyScopeProvinceName`
  - `CidInstitutionVerifier`（organization-manage/src/traits.rs）4 泛型 `<AccountId,AccountName,Nonce,Signature>`，mock 只给 3
- 红线：只改测试码，不动生产 lib；不弱化/删除断言；mock 照抄生产签名补 `_` 参保留返回值；调用点补的新参值要与各测试既有 issuer/省市设定一致（空省测试继续传空省并断言 EmptyScopeProvinceName）
- 涉及 crate：admins-change / personal-manage / organization-manage / duoqian-transfer / resolution-destro / resolution-issuance / grandpakey-change / internal-vote + runtime/src/tests
- 验收：`cargo test --workspace --no-fail-fast` 全构建全过
