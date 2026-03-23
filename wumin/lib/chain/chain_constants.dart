/// CitizenChain 链级常量。
///
/// 所有与链相关的固定参数统一在此维护，
/// 避免散落在多个文件中导致升级遗漏。
class ChainConstants {
  const ChainConstants._();

  /// SS58 地址前缀（CitizenChain 注册编号）。
  static const int ss58Prefix = 2027;
}
