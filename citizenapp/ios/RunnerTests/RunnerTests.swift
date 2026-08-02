import Flutter
import UIKit
import XCTest
@testable import Runner

class RunnerTests: XCTestCase {

  func testSecureEnclaveTagsAreStableAndDomainSeparated() throws {
    let vault = try SecureEnclaveKeyStore.applicationTag(
      namespace: "hw_seed_vault.strict",
      walletIndex: 7
    )
    let device = try SecureEnclaveKeyStore.applicationTag(
      namespace: "device_subkey",
      walletIndex: 7
    )
    let deviceData = try SecureEnclaveKeyStore.applicationTag(
      namespace: "device_data_key",
      walletIndex: 7
    )

    XCTAssertEqual(String(data: vault, encoding: .utf8), "org.citizenapp.hw_seed_vault.strict.7")
    XCTAssertEqual(String(data: device, encoding: .utf8), "org.citizenapp.device_subkey.7")
    XCTAssertEqual(
      String(data: deviceData, encoding: .utf8),
      "org.citizenapp.device_data_key.7"
    )
    XCTAssertNotEqual(vault, device)
    XCTAssertNotEqual(vault, deviceData)
    XCTAssertNotEqual(device, deviceData)
  }

  func testSecureEnclaveTagRejectsNegativeWalletIndex() {
    XCTAssertThrowsError(
      try SecureEnclaveKeyStore.applicationTag(
        namespace: "device_subkey",
        walletIndex: -1
      )
    )
  }

  func testVaultEnvelopeAcceptsOnlyFinalVersion() throws {
    let ciphertext = Data([0x00, 0x01, 0xfe, 0xff])
    let blob = HardwareBoundSeedVaultChannel.encodeBlob(ciphertext)

    XCTAssertTrue(blob.hasPrefix("ios-se:"))
    XCTAssertEqual(try HardwareBoundSeedVaultChannel.decodeBlob(blob), ciphertext)
    XCTAssertThrowsError(
      try HardwareBoundSeedVaultChannel.decodeBlob(
        "legacy:\(ciphertext.base64EncodedString())"
      )
    )
  }

  func testDevicePublicKeyRequiresUncompressedP256Point() {
    var valid = Data(repeating: 0, count: 65)
    valid[0] = 0x04

    XCTAssertNoThrow(try DeviceSubkeyChannel.validateUncompressedPublicKey(valid))
    XCTAssertThrowsError(
      try DeviceSubkeyChannel.validateUncompressedPublicKey(Data(repeating: 0, count: 65))
    )
    XCTAssertThrowsError(
      try DeviceSubkeyChannel.validateUncompressedPublicKey(Data([0x04]))
    )
  }

  func testLowerHexIsCanonical() {
    XCTAssertEqual(
      SecureEnclaveKeyStore.lowerHex(Data([0x00, 0x0a, 0xfe, 0xff])),
      "000afeff"
    )
  }

  func testDeviceDataEnvelopeRequiresExactAad() throws {
    let aad = Data("binding-a|chat".utf8)
    let plaintext = Data(repeating: 0x5a, count: 32)
    let envelope = DeviceDataKeyVaultChannel.encodeEnvelope(
      aad: aad,
      plaintext: plaintext
    )

    XCTAssertEqual(
      try DeviceDataKeyVaultChannel.decodeEnvelope(envelope, expectedAad: aad),
      plaintext
    )
    XCTAssertThrowsError(
      try DeviceDataKeyVaultChannel.decodeEnvelope(
        envelope,
        expectedAad: Data("binding-b|chat".utf8)
      )
    )
  }

}
