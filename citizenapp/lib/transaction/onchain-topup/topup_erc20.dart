// ERC-20 `transfer(address,uint256)` calldata 编码。
//
// selector = keccak256("transfer(address,uint256)")[0..4] = 0xa9059cbb;
// 参数各右对齐补齐到 32 字节。App 用它构造 eth_sendTransaction 的 data。

const String _transferSelector = 'a9059cbb';

/// 构造 ERC-20 transfer 的 0x calldata。
/// [toEvmAddress] 收款 EVM 地址（0x + 40 位 hex）；[amount] 为稳定币最小单位数额。
String encodeErc20Transfer(String toEvmAddress, BigInt amount) {
  final to = toEvmAddress.trim().toLowerCase().replaceFirst('0x', '');
  if (!RegExp(r'^[0-9a-f]{40}$').hasMatch(to)) {
    throw ArgumentError('收款地址不是合法 EVM 地址');
  }
  if (amount < BigInt.zero) {
    throw ArgumentError('金额不能为负');
  }
  final amountHex = amount.toRadixString(16);
  if (amountHex.length > 64) {
    throw ArgumentError('金额超出 uint256 范围');
  }
  final toWord = to.padLeft(64, '0');
  final amountWord = amountHex.padLeft(64, '0');
  return '0x$_transferSelector$toWord$amountWord';
}
