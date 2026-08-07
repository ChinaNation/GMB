/// 账户标识文本格式校验 —— 冷端单源。
///
/// 全仓 `account_id` 的文本编码恒为小写 `0x` 加 64 位十六进制(sr25519 公钥原字节)。
/// 与热端 `citizenapp/lib/citizen/shared/account_derivation.dart` 的同名函数逐字一致。
///
/// **禁止**在各处另写 `RegExp(r'^0x[0-9a-f]{64}$')`:同形正则散落各处必然漂移
/// (曾经 QR body、签名消息、载荷解码各写一份)。
///
/// **同形异语义禁并入**:32 字节哈希(blake2/交易哈希等)与 `account_id` 文本形态
/// 完全相同,但语义不同,不得复用本函数,也不得把它们的校验并进来。
library;

final RegExp _accountIdPattern = RegExp(r'^0x[0-9a-f]{64}$');

/// `value` 是否为规范 `account_id` 文本。
bool isAccountIdText(String value) => _accountIdPattern.hasMatch(value);
