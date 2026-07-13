import 'package:citizenapp/wallet/core/secure_seed_store.dart';

/// 硬件金库签名失败（[SecureSeedException]）→ 面向用户的中文提示。
///
/// 密封类穷尽匹配：新增子型时编译器强制补齐，杜绝「漏一种就静默」。
/// 调用方在 signWithWallet 的 catch 里用它给出反馈，替代此前只捕获
/// WalletAuthException、导致取消 / 无锁屏 / 金库错误被无声吞没的问题。
String seedSignErrorMessage(SecureSeedException e) => switch (e) {
      AuthCancelled() => '已取消签名',
      NoDeviceCredential() => '请先在系统设置开启锁屏（密码 / 指纹 / 人脸）后再签名',
      SecureStoreUnavailable() => '读取签名密钥失败，请重试',
      SeedKeyInvalidated() => '签名密钥已失效，请用助记词重新导入钱包',
    };
