import Flutter
import Foundation

/// Chat、MLS、附件和通讯录用途钥的 Secure Enclave 静默封装通道。
///
/// 本通道使用独立 `device_data_key` application tag，不复用 P-256 设备签名子钥或钱包
/// 严档 KEK。密文内部绑定调用方 AAD；绑定不一致、硬件钥失效或设备尚不可用时失败关闭。
final class DeviceDataKeyVaultChannel {
  private static let channelName = "org.citizenapp/device_data_key_vault"

  private let channel: FlutterMethodChannel
  private let keyStore = SecureEnclaveKeyStore()

  init(binaryMessenger: FlutterBinaryMessenger) {
    channel = FlutterMethodChannel(name: Self.channelName, binaryMessenger: binaryMessenger)
    channel.setMethodCallHandler { [weak self] call, result in
      guard let self else {
        result(FlutterError(
          code: "secureStoreUnavailable",
          message: "iOS 设备数据钥通道已释放",
          details: nil
        ))
        return
      }
      self.handle(call, result: result)
    }
  }

  private func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    do {
      let arguments = try requireArguments(call)
      let walletIndex = try requireWalletIndex(arguments)
      let tag = try SecureEnclaveKeyStore.applicationTag(
        namespace: "device_data_key",
        walletIndex: walletIndex
      )
      switch call.method {
      case "seal":
        let plaintext = try requireData(arguments, key: "plaintext")
        let aad = try requireData(arguments, key: "aad")
        let envelope = Self.encodeEnvelope(aad: aad, plaintext: plaintext)
        let encrypted = try keyStore.encrypt(
          plaintext: envelope,
          tag: tag,
          protection: .deviceOnly
        )
        result(encrypted.base64EncodedString())
      case "open":
        let ciphertext = try requireData(arguments, key: "blob")
        let aad = try requireData(arguments, key: "aad")
        let envelope = try keyStore.decrypt(
          ciphertext: ciphertext,
          tag: tag,
          protection: .deviceOnly,
          reason: ""
        )
        result(try Self.decodeEnvelope(envelope, expectedAad: aad).base64EncodedString())
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

  /// ECIES 已保证完整性；把 AAD 原文放入信封并在解封时逐字节复核，防止跨绑定替换。
  static func encodeEnvelope(aad: Data, plaintext: Data) -> Data {
    var aadLength = UInt32(aad.count).bigEndian
    var output = Data(bytes: &aadLength, count: MemoryLayout<UInt32>.size)
    output.append(aad)
    output.append(plaintext)
    return output
  }

  static func decodeEnvelope(_ envelope: Data, expectedAad: Data) throws -> Data {
    let headerSize = MemoryLayout<UInt32>.size
    guard envelope.count >= headerSize else {
      throw HardwareSecurityFailure.unavailable("设备数据钥信封损坏")
    }
    let aadLength = envelope.prefix(headerSize).reduce(UInt32(0)) {
      ($0 << 8) | UInt32($1)
    }
    let aadEnd = headerSize + Int(aadLength)
    guard aadEnd <= envelope.count else {
      throw HardwareSecurityFailure.unavailable("设备数据钥 AAD 长度损坏")
    }
    let storedAad = envelope.subdata(in: headerSize..<aadEnd)
    guard storedAad == expectedAad else {
      throw HardwareSecurityFailure.unavailable("设备数据钥绑定上下文不匹配")
    }
    return envelope.subdata(in: aadEnd..<envelope.count)
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

  private func requireData(_ arguments: [String: Any], key: String) throws -> Data {
    guard
      let encoded = arguments[key] as? String,
      !encoded.isEmpty,
      let data = Data(base64Encoded: encoded, options: []),
      !data.isEmpty
    else {
      throw HardwareSecurityFailure.invalidArguments("\(key) 不合法")
    }
    return data
  }
}
