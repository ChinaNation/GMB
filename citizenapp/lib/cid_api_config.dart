/// citizenapp 访问 OnChina 后端的唯一地址策略。
///
/// 这里刻意只允许生产域名和本地 USB 调试两条路径，禁止局域网 IP、
/// 任意 API URL 注入、旧端口默认值以及失败自动回退，避免电子护照账户被发送到
/// 不同信任环境。
class CidApiConfig {
  const CidApiConfig._();

  static const String environmentDefineName = 'CITIZENAPP_ONCHINA_ENV';
  static const String productionEnvironment = 'prod';
  static const String devUsbEnvironment = 'dev_usb';

  static const String productionBaseUrl = 'https://cid.crcfrcn.com';
  static const String devUsbBaseUrl = 'http://127.0.0.1:8899';

  static String get defaultBaseUrl {
    const environment = String.fromEnvironment(
      'CITIZENAPP_ONCHINA_ENV',
      defaultValue: productionEnvironment,
    );
    return baseUrlForEnvironment(environment);
  }

  static String baseUrlForEnvironment(String environment) {
    switch (environment.trim()) {
      case productionEnvironment:
        return productionBaseUrl;
      case devUsbEnvironment:
        return devUsbBaseUrl;
      default:
        throw UnsupportedError(
          '$environmentDefineName 只允许 prod 或 dev_usb，禁止配置其他 OnChina API 地址',
        );
    }
  }

  static String connectionErrorMessage(String baseUrl) {
    if (baseUrl == devUsbBaseUrl) {
      return '当前使用本地 USB 开发路径 $devUsbBaseUrl，请确认电脑 OnChina 后端已启动，并已执行 adb reverse tcp:8899 tcp:8899';
    }
    return '当前使用生产 OnChina 路径 $productionBaseUrl，请检查手机网络、HTTPS 证书或生产网关';
  }
}
