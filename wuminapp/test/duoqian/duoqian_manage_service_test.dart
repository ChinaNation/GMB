// `propose_create_institution`(OrganizationManage 17.5,14 参)的 SCALE 字节序列
// 一致性校验已统一到 step2d fixture(`test/fixtures/step2d_credential_payload.json`),
// 那里跟链端 commit + wumin 冷钱包 decoder 三端逐字节对齐。
//
// 历史的 `DuoqianManageService.buildProposeCreateInstitutionCallForTest`
// 与 `InstitutionInitialAccountInput` 是为本文件而设的内部 helper,sub-pallet
// 拆分前已从 lib 端删除,本文件 3 个测试随之失效。覆盖目标已被 fixture 替代,
// 这里清空保留文件作为占位以便 CI path 触发器仍然识别本目录。

void main() {}
