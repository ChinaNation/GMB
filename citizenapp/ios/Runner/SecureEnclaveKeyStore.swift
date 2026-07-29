import Foundation
import LocalAuthentication
import Security
import UIKit

/// iOS 硬件密钥的统一失败类型。code 与 Dart 金库现有错误映射逐字一致。
struct HardwareSecurityFailure: Error {
  let code: String
  let message: String

  static func invalidArguments(_ message: String) -> HardwareSecurityFailure {
    HardwareSecurityFailure(code: "invalidArguments", message: message)
  }

  static func unavailable(_ message: String) -> HardwareSecurityFailure {
    HardwareSecurityFailure(code: "secureStoreUnavailable", message: message)
  }
}

/// Secure Enclave P-256 私钥的唯一底座。
///
/// 私钥只允许在 Secure Enclave 内生成和使用，不提供导入/导出、软件密钥回退或旧标签兼容。
/// 严档钱包 KEK 与静默设备签名子钥共用此底座，但使用不同 application tag 与访问控制。
final class SecureEnclaveKeyStore {
  enum Protection {
    /// 私钥操作必须使用当前录入的 Face ID/Touch ID；生物集合变化后密钥失效。
    case currentBiometry
    /// 私钥只绑定本设备，首次解锁后可静默用于后台握手签名。
    case deviceOnly
  }

  static let encryptionAlgorithm =
    SecKeyAlgorithm.eciesEncryptionCofactorVariableIVX963SHA256AESGCM
  static let signatureAlgorithm =
    SecKeyAlgorithm.ecdsaSignatureMessageX962SHA256

  /// application tag 只包含稳定产品域、用途和 walletIndex，不写入账户或用户资料。
  static func applicationTag(namespace: String, walletIndex: Int) throws -> Data {
    guard walletIndex >= 0 else {
      throw HardwareSecurityFailure.invalidArguments("walletIndex 必须为非负整数")
    }
    return Data("org.citizenapp.\(namespace).\(walletIndex)".utf8)
  }

  static func lowerHex(_ data: Data) -> String {
    data.map { String(format: "%02x", $0) }.joined()
  }

  /// 只探测设备安全能力，不触发生物识别 UI，也不创建任何密钥。
  static func authenticationStatus() -> [String: Any] {
    var biometricError: NSError?
    let biometricContext = LAContext()
    let biometric = biometricContext.canEvaluatePolicy(
      .deviceOwnerAuthenticationWithBiometrics,
      error: &biometricError
    )

    var deviceError: NSError?
    let deviceContext = LAContext()
    let deviceSecure = deviceContext.canEvaluatePolicy(
      .deviceOwnerAuthentication,
      error: &deviceError
    )

    let majorVersion = Int(
      UIDevice.current.systemVersion.split(separator: ".").first ?? "0"
    ) ?? 0
    return [
      "sdk": majorVersion,
      "strongBiometricEnrolled": biometric,
      "deviceSecure": deviceSecure,
    ]
  }

  func encrypt(
    plaintext: Data,
    tag: Data,
    protection: Protection
  ) throws -> Data {
    let privateKey = try loadPrivateKey(
      tag: tag,
      protection: protection,
      createIfMissing: true,
      authenticationContext: nonInteractiveContext()
    )
    guard let publicKey = SecKeyCopyPublicKey(privateKey) else {
      throw HardwareSecurityFailure.unavailable("Secure Enclave 公钥不可用")
    }
    guard SecKeyIsAlgorithmSupported(publicKey, .encrypt, Self.encryptionAlgorithm) else {
      throw HardwareSecurityFailure.unavailable("设备不支持硬件 ECIES-AES-GCM")
    }

    var error: Unmanaged<CFError>?
    guard let encrypted = SecKeyCreateEncryptedData(
      publicKey,
      Self.encryptionAlgorithm,
      plaintext as CFData,
      &error
    ) else {
      throw map(error?.takeRetainedValue(), missingKeyIsInvalidated: false)
    }
    return encrypted as Data
  }

  func decrypt(
    ciphertext: Data,
    tag: Data,
    reason: String
  ) throws -> Data {
    let context = LAContext()
    context.localizedReason = reason
    context.localizedFallbackTitle = ""
    context.touchIDAuthenticationAllowableReuseDuration = 0

    let privateKey = try loadPrivateKey(
      tag: tag,
      protection: .currentBiometry,
      createIfMissing: false,
      authenticationContext: context
    )
    guard SecKeyIsAlgorithmSupported(privateKey, .decrypt, Self.encryptionAlgorithm) else {
      throw HardwareSecurityFailure.unavailable("设备不支持硬件 ECIES-AES-GCM")
    }

    var error: Unmanaged<CFError>?
    guard let decrypted = SecKeyCreateDecryptedData(
      privateKey,
      Self.encryptionAlgorithm,
      ciphertext as CFData,
      &error
    ) else {
      throw map(error?.takeRetainedValue(), missingKeyIsInvalidated: true)
    }
    return decrypted as Data
  }

  func publicKey(tag: Data, protection: Protection) throws -> Data {
    let privateKey = try loadPrivateKey(
      tag: tag,
      protection: protection,
      createIfMissing: true,
      authenticationContext: nonInteractiveContext()
    )
    guard let publicKey = SecKeyCopyPublicKey(privateKey) else {
      throw HardwareSecurityFailure.unavailable("Secure Enclave 公钥不可用")
    }
    var error: Unmanaged<CFError>?
    guard let representation = SecKeyCopyExternalRepresentation(publicKey, &error) else {
      throw map(error?.takeRetainedValue(), missingKeyIsInvalidated: false)
    }
    return representation as Data
  }

  func sign(message: Data, tag: Data) throws -> Data {
    let privateKey = try loadPrivateKey(
      tag: tag,
      protection: .deviceOnly,
      createIfMissing: true,
      authenticationContext: nonInteractiveContext()
    )
    guard SecKeyIsAlgorithmSupported(privateKey, .sign, Self.signatureAlgorithm) else {
      throw HardwareSecurityFailure.unavailable("设备不支持硬件 P-256 ECDSA")
    }
    var error: Unmanaged<CFError>?
    guard let signature = SecKeyCreateSignature(
      privateKey,
      Self.signatureAlgorithm,
      message as CFData,
      &error
    ) else {
      throw map(error?.takeRetainedValue(), missingKeyIsInvalidated: false)
    }
    return signature as Data
  }

  func delete(tag: Data) throws {
    let status = SecItemDelete(baseQuery(tag: tag) as CFDictionary)
    guard status == errSecSuccess || status == errSecItemNotFound else {
      throw map(status: status, missingKeyIsInvalidated: false)
    }
  }

  private func loadPrivateKey(
    tag: Data,
    protection: Protection,
    createIfMissing: Bool,
    authenticationContext: LAContext
  ) throws -> SecKey {
    var query = baseQuery(tag: tag)
    query[kSecReturnRef as String] = true
    query[kSecMatchLimit as String] = kSecMatchLimitOne
    query[kSecUseAuthenticationContext as String] = authenticationContext

    var item: CFTypeRef?
    let status = SecItemCopyMatching(query as CFDictionary, &item)
    if status == errSecSuccess, let item {
      return item as! SecKey
    }
    if status != errSecItemNotFound {
      throw map(status: status, missingKeyIsInvalidated: !createIfMissing)
    }
    guard createIfMissing else {
      throw HardwareSecurityFailure(
        code: "keyPermanentlyInvalidated",
        message: "Secure Enclave 私钥不存在或已失效"
      )
    }
    return try createPrivateKey(tag: tag, protection: protection)
  }

  private func createPrivateKey(tag: Data, protection: Protection) throws -> SecKey {
    let accessFlags: SecAccessControlCreateFlags
    let accessible: CFString
    switch protection {
    case .currentBiometry:
      accessFlags = [.privateKeyUsage, .biometryCurrentSet]
      accessible = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
    case .deviceOnly:
      accessFlags = [.privateKeyUsage]
      accessible = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
    }

    var accessError: Unmanaged<CFError>?
    guard let accessControl = SecAccessControlCreateWithFlags(
      nil,
      accessible,
      accessFlags,
      &accessError
    ) else {
      throw map(accessError?.takeRetainedValue(), missingKeyIsInvalidated: false)
    }

    let privateAttributes: [String: Any] = [
      kSecAttrIsPermanent as String: true,
      kSecAttrApplicationTag as String: tag,
      kSecAttrAccessControl as String: accessControl,
    ]
    let attributes: [String: Any] = [
      kSecAttrKeyType as String: kSecAttrKeyTypeECSECPrimeRandom,
      kSecAttrKeySizeInBits as String: 256,
      kSecAttrTokenID as String: kSecAttrTokenIDSecureEnclave,
      kSecPrivateKeyAttrs as String: privateAttributes,
    ]

    var error: Unmanaged<CFError>?
    guard let key = SecKeyCreateRandomKey(attributes as CFDictionary, &error) else {
      throw map(error?.takeRetainedValue(), missingKeyIsInvalidated: false)
    }
    return key
  }

  private func baseQuery(tag: Data) -> [String: Any] {
    [
      kSecClass as String: kSecClassKey,
      kSecAttrKeyType as String: kSecAttrKeyTypeECSECPrimeRandom,
      kSecAttrKeyClass as String: kSecAttrKeyClassPrivate,
      kSecAttrApplicationTag as String: tag,
      kSecUseDataProtectionKeychain as String: true,
    ]
  }

  private func nonInteractiveContext() -> LAContext {
    let context = LAContext()
    context.interactionNotAllowed = true
    return context
  }

  private func map(
    _ error: CFError?,
    missingKeyIsInvalidated: Bool
  ) -> HardwareSecurityFailure {
    guard let error else {
      return HardwareSecurityFailure.unavailable("Security.framework 返回未知错误")
    }
    return map(nsError: error as Error as NSError, missingKeyIsInvalidated: missingKeyIsInvalidated)
  }

  private func map(
    status: OSStatus,
    missingKeyIsInvalidated: Bool
  ) -> HardwareSecurityFailure {
    if status == errSecItemNotFound && missingKeyIsInvalidated {
      return HardwareSecurityFailure(
        code: "keyPermanentlyInvalidated",
        message: "Secure Enclave 私钥不存在或已失效"
      )
    }
    if status == errSecUserCanceled {
      return HardwareSecurityFailure(code: "userCancelled", message: "用户取消了生物识别")
    }
    if status == errSecAuthFailed {
      return HardwareSecurityFailure(code: "userCancelled", message: "生物识别验证失败")
    }
    if status == errSecInteractionNotAllowed {
      return HardwareSecurityFailure(code: "notEnrolled", message: "当前环境不允许安全认证")
    }
    let text = SecCopyErrorMessageString(status, nil) as String? ?? "OSStatus \(status)"
    return HardwareSecurityFailure.unavailable(text)
  }

  private func map(
    nsError: NSError,
    missingKeyIsInvalidated: Bool
  ) -> HardwareSecurityFailure {
    if nsError.domain == LAError.errorDomain, let code = LAError.Code(rawValue: nsError.code) {
      switch code {
      case .userCancel, .appCancel, .systemCancel:
        return HardwareSecurityFailure(code: "userCancelled", message: nsError.localizedDescription)
      case .biometryLockout:
        return HardwareSecurityFailure(code: "lockout", message: nsError.localizedDescription)
      case .biometryNotAvailable, .biometryNotEnrolled, .passcodeNotSet:
        return HardwareSecurityFailure(code: "notEnrolled", message: nsError.localizedDescription)
      default:
        break
      }
    }
    return map(
      status: OSStatus(nsError.code),
      missingKeyIsInvalidated: missingKeyIsInvalidated
    )
  }
}
