import Flutter
import Foundation

/// `org.citizenapp/hw_seed_vault` 原生实现。
///
/// 最终信封只接受 `ios-se-v1:`；iOS 此前没有原生金库，不提供旧格式、软件密钥或
/// 设备密码回退。明文 child mini-secret 仅在本次 MethodChannel 调用内短暂存在。
final class HardwareBoundSeedVaultChannel {
  private static let channelName = "org.citizenapp/hw_seed_vault"
  private static let strictTier = "strict"
  private static let blobPrefix = "ios-se-v1:"
  private static let plaintextPattern = try! NSRegularExpression(
    pattern: "^[0-9a-f]{64}$"
  )

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
          message: "iOS 硬件金库已释放",
          details: nil
        ))
        return
      }
      self.handle(call, result: result)
    }
  }

  static func encodeBlob(_ ciphertext: Data) -> String {
    blobPrefix + ciphertext.base64EncodedString()
  }

  static func decodeBlob(_ blob: String) throws -> Data {
    guard blob.hasPrefix(blobPrefix) else {
      throw HardwareSecurityFailure.invalidArguments("硬件金库密文版本不合法")
    }
    let base64 = String(blob.dropFirst(blobPrefix.count))
    guard !base64.isEmpty, let data = Data(base64Encoded: base64, options: []) else {
      throw HardwareSecurityFailure.invalidArguments("硬件金库密文格式不合法")
    }
    return data
  }

  private func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    do {
      switch call.method {
      case "authStatus":
        result(SecureEnclaveKeyStore.authenticationStatus())
      case "encrypt":
        let arguments = try requireArguments(call)
        let walletIndex = try requireWalletIndex(arguments)
        try requireStrictTier(arguments)
        let plaintext = try requireCanonicalMiniSecret(arguments)
        let tag = try SecureEnclaveKeyStore.applicationTag(
          namespace: "hw_seed_vault.strict",
          walletIndex: walletIndex
        )
        let encrypted = try keyStore.encrypt(
          plaintext: Data(plaintext.utf8),
          tag: tag,
          protection: .currentBiometry
        )
        result(Self.encodeBlob(encrypted))
      case "decrypt":
        let arguments = try requireArguments(call)
        let walletIndex = try requireWalletIndex(arguments)
        try requireStrictTier(arguments)
        let ciphertext = try Self.decodeBlob(try requireString(arguments, key: "blob"))
        let tag = try SecureEnclaveKeyStore.applicationTag(
          namespace: "hw_seed_vault.strict",
          walletIndex: walletIndex
        )
        let plaintextData = try keyStore.decrypt(
          ciphertext: ciphertext,
          tag: tag,
          reason: "验证身份以读取钱包私钥"
        )
        guard
          let plaintext = String(data: plaintextData, encoding: .utf8),
          Self.isCanonicalMiniSecret(plaintext)
        else {
          throw HardwareSecurityFailure.unavailable("硬件金库明文校验失败")
        }
        result(plaintext)
      case "deleteKey":
        let arguments = try requireArguments(call)
        let walletIndex = try requireWalletIndex(arguments)
        try requireStrictTier(arguments)
        let tag = try SecureEnclaveKeyStore.applicationTag(
          namespace: "hw_seed_vault.strict",
          walletIndex: walletIndex
        )
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

  private func requireStrictTier(_ arguments: [String: Any]) throws {
    guard arguments["tier"] as? String == Self.strictTier else {
      throw HardwareSecurityFailure.invalidArguments("只允许 strict 硬件金库")
    }
  }

  private func requireCanonicalMiniSecret(_ arguments: [String: Any]) throws -> String {
    let plaintext = try requireString(arguments, key: "plaintext")
    guard Self.isCanonicalMiniSecret(plaintext) else {
      throw HardwareSecurityFailure.invalidArguments(
        "child mini-secret 必须为 64 位小写十六进制"
      )
    }
    return plaintext
  }

  private func requireString(_ arguments: [String: Any], key: String) throws -> String {
    guard let value = arguments[key] as? String, !value.isEmpty else {
      throw HardwareSecurityFailure.invalidArguments("\(key) 不合法")
    }
    return value
  }

  private static func isCanonicalMiniSecret(_ value: String) -> Bool {
    let range = NSRange(value.startIndex..<value.endIndex, in: value)
    return plaintextPattern.firstMatch(in: value, range: range) != nil
  }
}
