# DUOQIAN 机构注册信息协议对齐

- 创建时间:2026-05-02
- 状态:done

## 需求

区块链软件创建机构多签时必须和 SFID 系统 `registration-info` 新协议对齐。
注册业务字段只使用 `sfid_id / institution_name / account_names[]`,不再把
`a3 / sub_type / parent_sfid_id` 放进机构注册 payload。区块链软件应能基于
SFID 资格候选查询选择私法人股份公司及其所属非法人,再用注册信息凭证完成链上注册。

## 边界规则

- SFID 查询与注册分开:查询详情仍走 `/api/v1/app/institutions/:sfid_id`,注册信息只走 `/registration-info`。
- DUOQIAN 机构注册 payload 只保留三项业务字段和凭证安全字段。
- 节点前端账户列表必须由 SFID 返回的 `account_names[]` 生成,不再写死账户名。
- 清理旧 DTO、旧注释、旧参数和旧文档残留。
- 改代码后更新文档、补充中文注释并运行检查。

## 预计修改目录

- `citizenchain/runtime/transaction/duoqian-manage/`
  - 中文注释:收口机构注册 extrinsic 与 storage/action 类型,删除注册 payload 中的机构类型三件套。
- `citizenchain/runtime/src/`
  - 中文注释:清理清算行资格对 DUOQIAN 机构类型元数据的依赖,避免旧链上元数据残留。
- `citizenchain/node/src/offchain/duoqian_manage/`
  - 中文注释:节点 Tauri 后端改读 SFID `registration-info`,补齐 `signer_admin_pubkey` 并按账户名列表构造 call_data。
- `citizenchain/node/src/offchain/common/`
  - 中文注释:更新 Tauri 对前端 DTO,删除旧注册字段。
- `citizenchain/node/frontend/offchain/`
  - 中文注释:节点前端创建机构多签页改用 `account_names[]`,删除旧参数透传。
- `memory/05-modules/`
  - 中文注释:更新 DUOQIAN 与节点清算行技术文档。
- `memory/08-tasks/`
  - 中文注释:记录执行、验证与残留清理结果。

## 验收

- 节点软件注册信息拉取只调用 `/api/v1/app/institutions/:sfid_id/registration-info`。
- `propose_create_institution` 的业务注册字段只剩 `sfid_id / institution_name / account_names[]`。
- 节点端 call_data 与 runtime 参数顺序一致,包含 `province + signer_admin_pubkey`。
- 前端账户输入行来自 SFID `account_names[]`。
- 旧 `a3/sub_type/parent_sfid_id` 不再出现在机构注册 payload/DTO/文档中。
- 相关 Rust/TypeScript 检查通过。

## 执行记录

- `duoqian-manage` runtime:
  - 删除机构注册参数、storage/action 中的 `a3/sub_type/parent_sfid_id`。
  - `SfidInstitutionVerifier` 改为校验 `sfid_id / institution_name / account_names[] / nonce / province / signer_admin_pubkey`。
  - 清算行资格不再读取链上机构类型元数据,由 SFID `eligible-search` 负责候选资格。
- 节点软件 Rust:
  - SFID 拉取入口改为 `fetch_clearing_bank_institution_registration_info`。
  - DTO 改为 `InstitutionRegistrationInfoResp { sfid_id, institution_name, account_names, credential }`。
  - `propose_create_institution` call_data 编码改为 10 字段,`province` 为普通 `Vec<u8>`,`signer_admin_pubkey` 为裸 32B。
- 节点前端:
  - 创建机构多签页调用 `fetchInstitutionRegistrationInfo`。
  - 账户输入行由 `registration_info.account_names` 生成,不再写死主账户/费用账户。
  - 提交参数删除 `a3/subType/parentSfidId`,新增 `signerAdminPubkey`。
- 文档:
  - 更新节点清算行、DUOQIAN、offchain-transaction、sfid-system 技术说明。
  - 补充中文注释并清理旧注册字段残留。

## 验证记录

- `cargo fmt`
- `cargo check -p duoqian-manage --tests`
- `cargo check -p duoqian-transfer --tests`
- `cargo check -p offchain-transaction --tests`
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/ci-wasm/citizenchain.compact.compressed.wasm cargo check -p node`
  - 通过,仅保留仓库既有 unsafe/dead_code 警告。
- `npm run build` in `citizenchain/node/frontend`

## 残留检查

- 注册链路 active code 已无 `InstitutionCredentialResp`、`fetch_clearing_bank_institution_credential`、旧 `a3/sub_type/parent_sfid_id` 注册参数。
- `eligible-search` DTO 仍保留 `a3/sub_type/parent_sfid_id`,这是 SFID 候选资格查询展示字段,不是注册 payload。
- `target/` 下旧编译产物未作为源码残留处理。

- 状态：done

## 完成信息

- 完成时间：2026-05-02 15:38:46
- 完成摘要：完成 DUOQIAN registration-info 协议对齐:runtime 注册字段收口,节点端改读 registration-info,前端账户列表来自 account_names,文档与残留扫描已更新
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
