import Flutter
import Foundation

/// `org.citizenapp/device_subkey` 原生实现。
///
/// 每个 walletIndex 只有一把 Secure Enclave P-256 子钥；私钥不可导出、不可同步，
/// 且不设置生物门禁，以满足登录和 Chat 握手的静默签名要求。
final class DeviceSubkeyChannel {
  private static let channelName = "org.citizenapp/device_subkey"

  private let channel: FlutterMethodChannel
  private let keyStore = SecureEnclaveKeyStore()

  init(binaryMessenger: FlutterBinaryMessenger) {
    channel = FlutterMethodChannel(
      name: Self.channelName,
      binaryMessenger: binaryMessenger
    )
    channel.setMethodCallHandler { [weak self] call, result in
      guard let self else {
        result(FlutterError(
          code: "secureStoreUnavailable",
          message: "iOS 设备子钥通道已释放",
          details: nil
        ))
        return
      }
      self.handle(call, result: result)
    }
  }

  static func validateUncompressedPublicKey(_ data: Data) throws {
    guard data.count == 65, data.first == 0x04 else {
      throw HardwareSecurityFailure.unavailable("P-256 公钥不是 65 字节未压缩点")
    }
  }

  private func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    do {
      let arguments = try requireArguments(call)
      let walletIndex = try requireWalletIndex(arguments)
      let tag = try SecureEnclaveKeyStore.applicationTag(
        namespace: "device_subkey",
        walletIndex: walletIndex
      )

      switch call.method {
      case "publicKey":
        let publicKey = try keyStore.publicKey(tag: tag, protection: .deviceOnly)
        try Self.validateUncompressedPublicKey(publicKey)
        result(SecureEnclaveKeyStore.lowerHex(publicKey))
      case "sign":
        let payload = try requirePayload(arguments)
        let signature = try keyStore.sign(message: payload, tag: tag)
        result(SecureEnclaveKeyStore.lowerHex(signature))
      case "delete":
        try keyStore.delete(tag: tag)
        result(nil)
      default:
        result(FlutterMethodNotImplemented)
      }
    } catch let failure as HardwareSecurityFailure {
      result(FlutterError(code: failure.code, message: failure.message, details: nil))
    } catch {
      result(FlutterError(
        code: "secureStoreUnavailable",
        message: error.localizedDescription,
        details: nil
      ))
    }
  }

  private func requireArguments(_ call: FlutterMethodCall) throws -> [String: Any] {
    guard let arguments = call.arguments as? [String: Any] else {
      throw HardwareSecurityFailure.invalidArguments("缺少原生通道参数")
    }
    return arguments
  }

  private func requireWalletIndex(_ arguments: [String: Any]) throws -> Int {
    guard let walletIndex = arguments["walletIndex"] as? Int, walletIndex >= 0 else {
      throw HardwareSecurityFailure.invalidArguments("walletIndex 不合法")
    }
    return walletIndex
  }

  private func requirePayload(_ arguments: [String: Any]) throws -> Data {
    guard
      let encoded = arguments["payload"] as? String,
      !encoded.isEmpty,
      let payload = Data(base64Encoded: encoded, options: []),
      !payload.isEmpty
    else {
      throw HardwareSecurityFailure.invalidArguments("签名 payload 不合法")
    }
    return payload
  }
}
