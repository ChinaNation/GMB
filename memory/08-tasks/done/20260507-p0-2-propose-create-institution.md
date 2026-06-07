# P0-2 统一机构创建交易载荷

## 任务目标

执行重新创世前总审计 P0-2：统一 `OrganizationManage(17).propose_create_institution(5)` 的交易载荷、冷钱包展示字段和测试口径。

## 统一协议

以 `memory/07-ai/unified-protocols.md` 的 `P-TX-001` 为唯一真源：

```text
sfid_number
institution_name
accounts
admin_count
duoqian_admins
threshold
register_nonce
signature
province
signer_admin_pubkey
```

禁止继续使用旧机构创建字段：

```text
account_name
amount
subject_property
sub_type
parent_sfid_number
```

## 预计修改目录

- `wuminapp/lib/wallet/capabilities/`：新增 SFID `/registration-info` 响应模型和读取入口；只涉及前端 API 数据层，不改后端协议。
- `wuminapp/lib/duoqian/`：修正机构多签创建页面与链端编码；涉及 Flutter 业务代码和冷钱包展示字段。
- `wumin/lib/signer/`：修正冷钱包 payload decoder 与 pallet 注释；涉及冷钱包离线签名校验，不改 QR 外层协议。
- `wumin/test/signer/`：更新冷钱包 decoder 回归测试，确认旧尾随字段被拒绝。
- `wuminapp/test/duoqian/`：恢复机构创建 call_data 字节级测试，确认在线端编码与统一协议一致。
- `citizenchain/node/src/offchain/organization_manage/`：补齐节点端冷钱包展示字段；只改 display，不改 runtime。
- `memory/08-tasks/open/`：回写本任务执行结果和总审计记录；只涉及文档。

## 执行清单

- [x] 在线端读取 `registration-info`，使用后端下发的 `account_names/register_nonce/signature/province/signer_admin_pubkey`。
- [x] 在线端按 10 字段编码 `17.5`，不再使用旧 `17.0` 单账户入口。
- [x] 在线端创建页面改为按 `registration-info.account_names` 顺序填写每个初始账户金额。
- [x] 冷钱包 decoder 删除 `subject_property/sub_type/parent_sfid_number` 解析，末尾不得有多余字节。
- [x] 冷钱包和在线端展示字段统一为 `sfid_number/institution_name/admin_count/threshold/total_amount_yuan/amount_<账户名>/province/signer_admin_pubkey`。
- [x] 增加或更新 P0-2 回归测试。
- [x] 更新总审计文档并运行验收。

## 验收标准

- `wuminapp` 编码输出以 `0x11 0x05` 开头。
- call data 字段顺序与 `P-TX-001` 完全一致。
- 冷钱包 decoder 能解当前 10 字段载荷。
- 冷钱包 decoder 拒绝带旧 `subject_property/sub_type/parent_sfid_number` 尾巴的载荷。
- `git diff --cached --check` 通过。

## 执行结果

- `wuminapp/lib/duoqian/shared/duoqian_manage_service.dart` 已改为 `OrganizationManage(17).propose_create_institution(5)`，字段顺序与 P-TX-001 一致。
- `wuminapp/lib/wallet/capabilities/api_client.dart` 已新增 `fetchInstitutionRegistrationInfo`，读取 SFID `/api/v1/app/institutions/:sfid_number/registration-info`。
- `wuminapp/lib/duoqian/institution/institution_duoqian_create_page.dart` 已改为按机构账户列表填写多账户初始资金，提交前再用 `registration-info.account_names` 顺序构造 `accounts`。
- `wumin/lib/signer/payload_decoder.dart` 已删除旧尾字段解析，解码后要求无剩余字节，并输出 `total_amount_yuan` 与 `amount_<账户名>`。
- `citizenchain/node/src/offchain/organization_manage/signing.rs` 已补齐 `province` 与 `signer_admin_pubkey` display 字段。

## 验收记录

- `flutter test test/duoqian/duoqian_manage_service_test.dart`：通过。
- `flutter test test/signer/payload_decoder_test.dart`：通过。
- `flutter analyze lib/wallet/capabilities/api_client.dart lib/duoqian/shared/duoqian_manage_service.dart lib/duoqian/institution/institution_duoqian_create_page.dart test/duoqian/duoqian_manage_service_test.dart`：通过。
- `flutter analyze lib/signer/payload_decoder.dart lib/signer/pallet_registry.dart test/signer/payload_decoder_test.dart`：通过。
- `rustfmt --check citizenchain/node/src/offchain/organization_manage/signing.rs`：通过。
- `cargo check -p node`：已执行，但 runtime build script 按仓库硬规则要求 `WASM_FILE`，本地未设置而中止；未到本次 Rust 文件编译错误阶段。
