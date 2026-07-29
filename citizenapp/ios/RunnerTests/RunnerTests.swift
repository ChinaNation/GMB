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

    XCTAssertEqual(String(data: vault, encoding: .utf8), "org.citizenapp.hw_seed_vault.strict.7")
    XCTAssertEqual(String(data: device, encoding: .utf8), "org.citizenapp.device_subkey.7")
    XCTAssertNotEqual(vault, device)
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

    XCTAssertTrue(blob.hasPrefix("ios-se-v1:"))
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

}
