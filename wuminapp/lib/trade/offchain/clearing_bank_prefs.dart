import 'package:shared_preferences/shared_preferences.dart';

/// 扫码支付 Step 2c-ii-a:**用户绑定的清算行 `shenfen_id` 本地缓存**。
///
/// 中文注释:
/// - 链上 `OffchainTransactionPos::UserBank[user]` 存的是**主账户** `AccountId32`
///   (32 字节),**不是** SFID `shenfen_id` 字符串。但 wuminapp 在收款 QR 里需要
///   的是 `shenfen_id`(付款方付款时会通过 SFID 公开 API 反查主账户做同行校验)。
/// - 链上 → `shenfen_id` 的反向映射在 SFID 后端存,公开搜索 API 只支持 keyword
///   匹配,没有"按主账户 hex 精确反查"端点。为避免 Step 2c-ii-a 进度依赖 SFID
///   Agent 侧改动,本步用 SharedPreferences 做本地缓存:
///     - **写入点**:绑定页在链上绑定成功后写入。
///     - **读取点**:原收款码生成页在构造 QR 前读;2026-04-23 收款码页已随清算行
///       入口重构下线,本缓存目前仅由绑定流程写入,等后续清算行功能重上线再接读端。
/// - 缓存按 `walletIndex` 隔离,同 App 多钱包互不干扰。
/// - **失去缓存的处置**(App 重装 / 清数据 / 用户之前在 CLI / 另一台设备绑的):
///   收款页显示"请先在本 App 绑定清算行以生成收款码"。Step 3 改为 SFID 后端
///   补反查 API 后自动回填。
class ClearingBankPrefs {
  ClearingBankPrefs._();

  static const String _keyPrefix = 'clearing_bank_shenfen_id_';

  /// 写入 `walletIndex` 绑定的清算行 `shenfen_id`。空串等价于 `clear`。
  static Future<void> save(int walletIndex, String shenfenId) async {
    final prefs = await SharedPreferences.getInstance();
    final trimmed = shenfenId.trim();
    if (trimmed.isEmpty) {
      await prefs.remove('$_keyPrefix$walletIndex');
    } else {
      await prefs.setString('$_keyPrefix$walletIndex', trimmed);
    }
  }

  /// 读取,未绑定/未写入返回 `null`。
  static Future<String?> load(int walletIndex) async {
    final prefs = await SharedPreferences.getInstance();
    final v = prefs.getString('$_keyPrefix$walletIndex');
    return (v == null || v.isEmpty) ? null : v;
  }

  /// 清除(切换清算行后由 bind 页主动覆盖,或用户手动解绑时调)。
  static Future<void> clear(int walletIndex) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove('$_keyPrefix$walletIndex');
  }
}
